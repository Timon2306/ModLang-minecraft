use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::core::constants::LanguageInfo;
use crate::core::sanitizer::polish_translation;
use crate::core::shield::unmask_translation;
use crate::core::translator::traits::{EngineCallbacks, EngineItem, TranslationEngine};

pub struct GoogleEngine {
    client: Client,
    workers: usize,
    mode: String,
    delay_ms: u64,
    dictionary: HashMap<String, String>,
}

impl GoogleEngine {
    const API_URL: &'static str = "https://translate.googleapis.com/translate_a/single";
    const BATCH_SEP: &'static str = " |~| ";

    pub fn new(
        workers: usize,
        mode: String,
        delay_ms: u64,
        dictionary: HashMap<String, String>,
    ) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(12))
            .build()
            .unwrap_or_default();

        Self {
            client,
            workers: workers.clamp(2, 4),
            mode,
            delay_ms: delay_ms.max(80),
            dictionary,
        }
    }

    fn finalize(&self, raw: &str, item: &EngineItem) -> String {
        let unmasked = unmask_translation(raw, &item.mapping);
        polish_translation(&unmasked, &self.dictionary)
    }
}

#[async_trait::async_trait]
impl TranslationEngine for GoogleEngine {
    async fn translate_batch(
        &self,
        items: HashMap<String, EngineItem>,
        target_lang: &LanguageInfo,
        callbacks: &EngineCallbacks,
    ) -> HashMap<String, String> {
        if items.is_empty() {
            return HashMap::new();
        }

        let mut results = HashMap::new();
        let api_code = target_lang.api_code.clone();
        let total_items = items.len();
        let completed_counter = Arc::new(AtomicUsize::new(0));

        if self.mode == "batch" {
            let sep_regex = Regex::new(r"\s*\|\s*~\s*\|\s*").unwrap();
            let mut chunks: Vec<(Vec<String>, String)> = Vec::new();
            let mut current_keys: Vec<String> = Vec::new();
            let mut current_text = String::new();

            for (key, item) in &items {
                if current_text.len() + item.masked.len() > 1400 || current_keys.len() >= 20 {
                    chunks.push((current_keys.clone(), current_text.clone()));
                    current_keys.clear();
                    current_text.clear();
                }
                current_keys.push(key.clone());
                if current_text.is_empty() {
                    current_text.push_str(&item.masked);
                } else {
                    current_text.push_str(Self::BATCH_SEP);
                    current_text.push_str(&item.masked);
                }
            }
            if !current_keys.is_empty() {
                chunks.push((current_keys, current_text));
            }

            let semaphore = Arc::new(Semaphore::new(self.workers));
            let mut tasks = Vec::new();
            let pacing_delay = self.delay_ms;

            for (chunk_keys, chunk_text) in chunks {
                let sem = semaphore.clone();
                let api_c = api_code.clone();
                let cb = callbacks.clone();
                let engine_client = self.client.clone();
                let sep_re = sep_regex.clone();
                let counter = completed_counter.clone();
                let chunk_items: Vec<(String, EngineItem)> = chunk_keys
                    .iter()
                    .map(|k| (k.clone(), items[k].clone()))
                    .collect();

                tasks.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    if !cb.should_run() {
                        return Vec::new();
                    }
                    cb.wait_if_paused().await;
                    tokio::time::sleep(Duration::from_millis(pacing_delay)).await;

                    let mut retries = 5;
                    let mut wait_ms = 800u64;
                    let mut chunk_raw = None;

                    while retries > 0 && cb.should_run() {
                        let res = engine_client
                            .get(Self::API_URL)
                            .query(&[
                                ("client", "gtx"),
                                ("sl", "en"),
                                ("tl", api_c.as_str()),
                                ("dt", "t"),
                                ("q", chunk_text.as_str()),
                            ])
                            .send()
                            .await;

                        match res {
                            Ok(resp) => {
                                if resp.status().as_u16() == 429 {
                                    cb.log(format!("   [ЛИМИТ] Google запросил микро-паузу (429), ожидание {:.1} сек...", wait_ms as f64 / 1000.0), "yellow");
                                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                                    wait_ms = (wait_ms * 2).min(6000);
                                } else if resp.status().is_success() {
                                    if let Ok(json_val) = resp.json::<Value>().await {
                                        if let Some(arr) = json_val.get(0).and_then(|v| v.as_array()) {
                                            let mut full = String::new();
                                            for part in arr {
                                                if let Some(part_str) = part.get(0).and_then(|p| p.as_str()) {
                                                    full.push_str(part_str);
                                                }
                                            }
                                            if !full.is_empty() {
                                                chunk_raw = Some(full);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                                wait_ms = (wait_ms * 2).min(6000);
                            }
                        }
                        retries -= 1;
                    }

                    let mut chunk_results = Vec::new();
                    if let Some(raw) = chunk_raw {
                        let parts: Vec<&str> = sep_re.split(&raw).collect();
                        if parts.len() == chunk_items.len() {
                            for (idx, (k, itm)) in chunk_items.iter().enumerate() {
                                let clean_part = parts[idx].trim();
                                let unmasked = unmask_translation(clean_part, &itm.mapping);
                                chunk_results.push((k.clone(), unmasked));
                            }

                            let done = counter.fetch_add(chunk_items.len(), Ordering::SeqCst) + chunk_items.len();
                            let capped_done = done.min(total_items);
                            cb.status(format!("Перевод: {} / {} строк", capped_done, total_items), None);

                            return chunk_results;
                        }
                    }

                    // Fallback to single requests with retry for each item
                    for (k, itm) in chunk_items {
                        if !cb.should_run() {
                            break;
                        }
                        let mut single_res = None;
                        let mut s_retries = 3;
                        let mut s_wait = 500u64;

                        while s_retries > 0 && cb.should_run() {
                            if let Ok(resp) = engine_client
                                .get(Self::API_URL)
                                .query(&[
                                    ("client", "gtx"),
                                    ("sl", "en"),
                                    ("tl", api_c.as_str()),
                                    ("dt", "t"),
                                    ("q", itm.masked.as_str()),
                                ])
                                .send()
                                .await
                            {
                                if resp.status().is_success() {
                                    if let Ok(json_val) = resp.json::<Value>().await {
                                        if let Some(arr) = json_val.get(0).and_then(|v| v.as_array()) {
                                            let mut full = String::new();
                                            for part in arr {
                                                if let Some(p_str) = part.get(0).and_then(|p| p.as_str()) {
                                                    full.push_str(p_str);
                                                }
                                            }
                                            if !full.is_empty() {
                                                single_res = Some(unmask_translation(&full, &itm.mapping));
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            tokio::time::sleep(Duration::from_millis(s_wait)).await;
                            s_wait *= 2;
                            s_retries -= 1;
                        }

                        let final_str = single_res.unwrap_or(itm.original);
                        chunk_results.push((k, final_str));
                        let done = counter.fetch_add(1, Ordering::SeqCst) + 1;
                        cb.status(format!("Перевод: {} / {} строк", done.min(total_items), total_items), None);
                    }

                    chunk_results
                }));
            }

            for task in tasks {
                if let Ok(chunk_res) = task.await {
                    for (k, text) in chunk_res {
                        let polished = polish_translation(&text, &self.dictionary);
                        results.insert(k, polished);
                    }
                }
            }
        } else {
            // Single Mode
            let semaphore = Arc::new(Semaphore::new(self.workers));
            let mut tasks = Vec::new();

            for (key, item) in items.clone() {
                let sem = semaphore.clone();
                let api_c = api_code.clone();
                let cb = callbacks.clone();
                let engine_client = self.client.clone();
                let counter = completed_counter.clone();

                tasks.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    if !cb.should_run() {
                        return (key, None);
                    }
                    cb.wait_if_paused().await;

                    let mut retries = 4;
                    let mut text_result = None;
                    let mut wait_ms = 400u64;

                    while retries > 0 && cb.should_run() {
                        let res = engine_client
                            .get(Self::API_URL)
                            .query(&[
                                ("client", "gtx"),
                                ("sl", "en"),
                                ("tl", api_c.as_str()),
                                ("dt", "t"),
                                ("q", item.masked.as_str()),
                            ])
                            .send()
                            .await;

                        if let Ok(resp) = res {
                            if resp.status().is_success() {
                                if let Ok(json_val) = resp.json::<Value>().await {
                                    if let Some(arr) = json_val.get(0).and_then(|v| v.as_array()) {
                                        let mut full = String::new();
                                        for part in arr {
                                            if let Some(p_str) = part.get(0).and_then(|p| p.as_str()) {
                                                full.push_str(p_str);
                                            }
                                        }
                                        if !full.is_empty() {
                                            text_result = Some(full);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                        wait_ms *= 2;
                        retries -= 1;
                    }

                    let done = counter.fetch_add(1, Ordering::SeqCst) + 1;
                    cb.status(format!("Перевод: {} / {} строк", done.min(total_items), total_items), None);

                    (key, text_result)
                }));
            }

            for task in tasks {
                if let Ok((key, maybe_raw)) = task.await {
                    if let Some(raw) = maybe_raw {
                        results.insert(key.clone(), self.finalize(&raw, &items[&key]));
                    } else {
                        results.insert(key.clone(), items[&key].original.clone());
                    }
                }
            }
        }

        results
    }
}
