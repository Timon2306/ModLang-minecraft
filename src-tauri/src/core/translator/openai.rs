use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::constants::LanguageInfo;
use crate::core::sanitizer::polish_translation;
use crate::core::shield::{placeholders_match, unmask_translation};
use crate::core::translator::traits::{EngineCallbacks, EngineItem, TranslationEngine};

pub struct OpenAiEngine {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
    site_url: String,
    app_name: String,
    context: String,
    mode: String,
    dictionary: HashMap<String, String>,
}

impl OpenAiEngine {
    pub fn new(
        api_url: String,
        api_key: String,
        model: String,
        site_url: String,
        app_name: String,
        context: String,
        mode: String,
        dictionary: HashMap<String, String>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_url: if api_url.trim().is_empty() {
                "https://openrouter.ai/api/v1/chat/completions".to_string()
            } else {
                api_url
            },
            api_key,
            model,
            site_url,
            app_name,
            context,
            mode,
            dictionary,
        }
    }

    fn build_prompt(&self, payload: &HashMap<String, String>, lang_name: &str) -> String {
        let blob = serde_json::to_string_pretty(payload).unwrap_or_default();
        if self.mode == "context" && !self.context.is_empty() {
            format!(
                "Ты профессиональный локализатор Minecraft. Переведи значения JSON-строк мода/квеста «{}» на {}.\n\
                ПРАВИЛА:\n\
                1. Сохраняй игровой стиль, лор и терминологию Minecraft.\n\
                2. НЕ переводи JSON-ключи.\n\
                3. ВСЕ маркеры вида [#N#] (например [#0#], [#1#]) сохраняй ТОЧНО как в исходнике: не удаляй, не добавляй, не дублируй и не изменяй.\n\
                4. Верни ТОЛЬКО валидный JSON-объект с теми же ключами.\n\n\
                Данные: {}",
                self.context, lang_name, blob
            )
        } else {
            format!(
                "Translate JSON string values from English to {}.\n\
                RULES:\n\
                1. Do not translate JSON keys.\n\
                2. Preserve every [#N#] placeholder exactly (e.g. [#0#], [#1#]). Do not add, remove, duplicate, or rename placeholders.\n\
                3. Return ONLY valid JSON with the exact same keys.\n\n\
                Data: {}",
                lang_name, blob
            )
        }
    }

    async fn call_api(&self, prompt: &str) -> Result<String, String> {
        let mut retries = 3;
        let mut wait_secs = 4;

        while retries > 0 {
            let mut req = self.client.post(&self.api_url)
                .header("Content-Type", "application/json");

            if !self.api_key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", self.api_key));
            }
            if !self.site_url.is_empty() {
                req = req.header("HTTP-Referer", &self.site_url);
            }
            if !self.app_name.is_empty() {
                req = req.header("X-Title", &self.app_name);
            }

            let body = json!({
                "model": self.model,
                "messages": [
                    { "role": "user", "content": prompt }
                ],
                "temperature": 0.1,
                "max_tokens": if self.mode == "context" { 4096 } else { 2048 }
            });

            match req.json(&body).send().await {
                Ok(resp) => {
                    if resp.status().as_u16() == 429 {
                        tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                        wait_secs *= 2;
                        retries -= 1;
                        continue;
                    }
                    if !resp.status().is_success() {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();
                        return Err(format!("API Error {}: {}", status, text));
                    }
                    let json_val = resp.json::<Value>().await.map_err(|e| e.to_string())?;
                    if let Some(choices) = json_val.get("choices").and_then(|v| v.as_array()) {
                        if let Some(first) = choices.first() {
                            if let Some(content) = first.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                                return Ok(content.to_string());
                            }
                        }
                    }
                }
                Err(err) => {
                    if retries == 1 {
                        return Err(err.to_string());
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(1000)).await;
            retries -= 1;
        }

        Err("Failed to get response from API".to_string())
    }
}

#[async_trait::async_trait]
impl TranslationEngine for OpenAiEngine {
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
        let chunk_size = if self.mode == "context" { 30 } else { 20 };
        let all_keys: Vec<String> = items.keys().cloned().collect();

        for chunk_keys in all_keys.chunks(chunk_size) {
            if !callbacks.should_run() {
                break;
            }
            callbacks.wait_if_paused().await;

            let mut payload = HashMap::new();
            for k in chunk_keys {
                payload.insert(k.clone(), items[k].masked.clone());
            }

            callbacks.status(
                format!("⏳ LLM ({}): пакет {} строк...", self.model, chunk_keys.len()),
                None,
            );

            let prompt = self.build_prompt(&payload, &target_lang.name);
            match self.call_api(&prompt).await {
                Ok(content) => {
                    let clean_json = content
                        .trim()
                        .trim_start_matches("```json")
                        .trim_start_matches("```")
                        .trim_end_matches("```")
                        .trim();

                    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(clean_json) {
                        for k in chunk_keys {
                            if let Some(trans_val) = map.get(k).and_then(|v| v.as_str()) {
                                if placeholders_match(trans_val, &items[k].masked) {
                                    let unmasked = unmask_translation(trans_val, &items[k].mapping);
                                    let polished = polish_translation(&unmasked, &self.dictionary);
                                    results.insert(k.clone(), polished);
                                } else {
                                    callbacks.log(
                                        format!("⚠️ Искажены маркеры в [{}], оставлен оригинал", k),
                                        "yellow",
                                    );
                                    results.insert(k.clone(), items[k].original.clone());
                                }
                            } else {
                                results.insert(k.clone(), items[k].original.clone());
                            }
                        }
                    } else {
                        callbacks.log("❌ Неверный JSON от LLM", "red");
                        for k in chunk_keys {
                            results.insert(k.clone(), items[k].original.clone());
                        }
                    }
                }
                Err(err) => {
                    callbacks.log(format!("❌ Ошибка LLM API: {}", err), "red");
                    for k in chunk_keys {
                        results.insert(k.clone(), items[k].original.clone());
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        results
    }
}
