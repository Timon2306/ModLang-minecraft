use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::constants::{LanguageInfo, GEMINI_API_URL};
use crate::core::sanitizer::polish_translation;
use crate::core::shield::{placeholders_match, unmask_translation};
use crate::core::translator::traits::{EngineCallbacks, EngineItem, TranslationEngine};

pub struct GeminiEngine {
    client: Client,
    api_key: String,
    model: String,
    context: String,
    mode: String,
    delay_ms: u64,
    dictionary: HashMap<String, String>,
}

impl GeminiEngine {
    pub fn new(
        api_key: String,
        model: String,
        context: String,
        mode: String,
        delay_ms: u64,
        dictionary: HashMap<String, String>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_key,
            model,
            context,
            mode,
            delay_ms: delay_ms.max(500),
            dictionary,
        }
    }

    fn build_prompt(&self, payload: &HashMap<String, String>, lang_name: &str) -> String {
        let blob = serde_json::to_string_pretty(payload).unwrap_or_default();
        if self.mode == "context" && !self.context.is_empty() {
            format!(
                "Ты профессиональный локализатор Minecraft. Переведи значения JSON-строк мода/квеста «{}» на {}.\n\
                ПРАВИЛА:\n\
                1. Сохраняй игровой стиль, лор и терминологию Minecraft (Dust->Пыль, Ingot->Слиток, Plate->Пластина, Gear->Шестерня).\n\
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
                3. Preserve Minecraft game terminology and technical units.\n\
                4. Return ONLY valid JSON with the exact same keys.\n\n\
                Data: {}",
                lang_name, blob
            )
        }
    }

    async fn call_gemini(&self, prompt: &str, callbacks: &EngineCallbacks) -> Result<String, String> {
        let url = format!("{}/{}:generateContent?key={}", GEMINI_API_URL, self.model, self.api_key);
        
        let body = json!({
            "contents": [
                {
                    "parts": [
                        { "text": prompt }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": 0.1,
                "responseMimeType": "application/json"
            }
        });

        let mut retries = 4;
        let mut wait_delay = 5000u64;

        while retries > 0 && callbacks.should_run() {
            callbacks.wait_if_paused().await;

            let resp = self.client.post(&url)
                .json(&body)
                .send()
                .await;

            match resp {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let json_val = response.json::<Value>().await.map_err(|e| e.to_string())?;
                        if let Some(candidates) = json_val.get("candidates").and_then(|v| v.as_array()) {
                            if let Some(first) = candidates.first() {
                                if let Some(parts) = first.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                                    if let Some(text_part) = parts.first().and_then(|t| t.get("text")).and_then(|t| t.as_str()) {
                                        return Ok(text_part.to_string());
                                    }
                                }
                            }
                        }
                        return Err("Empty response from Gemini".to_string());
                    } else if status.as_u16() == 429 || status.as_u16() == 503 {
                        callbacks.log(
                            format!("⏳ Лимит/нагрузка Gemini ({}) — ожидание {} сек перед повтором...", status, wait_delay / 1000),
                            "yellow"
                        );
                        tokio::time::sleep(Duration::from_millis(wait_delay)).await;
                        wait_delay = (wait_delay * 2).min(20000);
                        retries -= 1;
                        continue;
                    } else {
                        let text = response.text().await.unwrap_or_default();
                        return Err(format!("Gemini API Error {}: {}", status, text));
                    }
                }
                Err(e) => {
                    callbacks.log(format!("⚠️ Ошибка соединения: {} — повтор через 3 сек...", e), "yellow");
                    tokio::time::sleep(Duration::from_millis(3000)).await;
                    retries -= 1;
                }
            }
        }

        Err("Превышено количество попыток запроса к Gemini".to_string())
    }
}

#[async_trait::async_trait]
impl TranslationEngine for GeminiEngine {
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
        let chunk_size = if self.mode == "context" { 40 } else { 20 };
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
                format!("⏳ Gemini ({}): пакет {} строк...", self.model, chunk_keys.len()),
                None,
            );

            let prompt = self.build_prompt(&payload, &target_lang.name);
            match self.call_gemini(&prompt, callbacks).await {
                Ok(content) => {
                    let clean_json = content
                        .trim()
                        .trim_start_matches("```json")
                        .trim_start_matches("```")
                        .trim_end_matches("```")
                        .trim();

                    if let Ok(parsed) = serde_json::from_str::<HashMap<String, String>>(clean_json) {
                        for (k, trans_raw) in parsed {
                            if let Some(item) = items.get(&k) {
                                if placeholders_match(&item.masked, &trans_raw) {
                                    let unmasked = unmask_translation(&trans_raw, &item.mapping);
                                    let polished = polish_translation(&unmasked, &self.dictionary);
                                    results.insert(k, polished);
                                } else {
                                    callbacks.log(
                                        format!("⚠️ Искажены маркеры в [{}], оставлен оригинал", k),
                                        "yellow",
                                    );
                                    results.insert(k, item.original.clone());
                                }
                            }
                        }
                    } else {
                        callbacks.log("❌ Ошибка парсинга JSON от Gemini".to_string(), "red");
                    }
                }
                Err(e) => {
                    callbacks.log(format!("❌ Ошибка сети Gemini: {}", e), "red");
                }
            }

            // Rate-limiting delay between chunks (e.g. 4.1s for 14.6 RPM)
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        }

        results
    }
}
