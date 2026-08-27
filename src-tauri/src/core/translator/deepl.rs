use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use serde_json::{json, Value};

use crate::core::constants::LanguageInfo;
use crate::core::sanitizer::polish_translation;
use crate::core::shield::unmask_translation;
use crate::core::translator::traits::{EngineCallbacks, EngineItem, TranslationEngine};

pub struct DeepLEngine {
    client: Client,
    api_key: String,
    url: String,
    dictionary: HashMap<String, String>,
}

impl DeepLEngine {
    pub fn new(api_key: String, dictionary: HashMap<String, String>) -> Self {
        let trimmed_key = api_key.trim().to_string();
        let url = if trimmed_key.ends_with(":fx") {
            "https://api-free.deepl.com/v2/translate".to_string()
        } else {
            "https://api.deepl.com/v2/translate".to_string()
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_key: trimmed_key,
            url,
            dictionary,
        }
    }
}

#[async_trait::async_trait]
impl TranslationEngine for DeepLEngine {
    async fn translate_batch(
        &self,
        items: HashMap<String, EngineItem>,
        target_lang: &LanguageInfo,
        callbacks: &EngineCallbacks,
    ) -> HashMap<String, String> {
        if items.is_empty() {
            return HashMap::new();
        }

        let is_free = self.api_key.ends_with(":fx");
        callbacks.log(
            format!(
                "🌐 DeepL {} → {}",
                if is_free { "Free (api-free.deepl.com)" } else { "Pro (api.deepl.com)" },
                target_lang.deepl_code
            ),
            "dim",
        );

        let mut results = HashMap::new();
        let all_keys: Vec<String> = items.keys().cloned().collect();

        for chunk_keys in all_keys.chunks(40) {
            if !callbacks.should_run() {
                break;
            }
            callbacks.wait_if_paused().await;

            let texts: Vec<&str> = chunk_keys.iter().map(|k| items[k].masked.as_str()).collect();
            let body = json!({
                "text": texts,
                "target_lang": target_lang.deepl_code
            });

            let req = self.client.post(&self.url)
                .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            match req {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(json_val) = resp.json::<Value>().await {
                            if let Some(translations) = json_val.get("translations").and_then(|v| v.as_array()) {
                                for (idx, k) in chunk_keys.iter().enumerate() {
                                    if let Some(t_obj) = translations.get(idx) {
                                        if let Some(raw) = t_obj.get("text").and_then(|t| t.as_str()) {
                                            let unmasked = unmask_translation(raw, &items[k].mapping);
                                            let polished = polish_translation(&unmasked, &self.dictionary);
                                            results.insert(k.clone(), polished);
                                            continue;
                                        }
                                    }
                                    results.insert(k.clone(), items[k].original.clone());
                                }
                            }
                        }
                    } else {
                        let status = resp.status();
                        let body = resp.text().await.unwrap_or_default();
                        callbacks.log(format!("❌ Ошибка DeepL: {} — {}", status, body.chars().take(200).collect::<String>()), "red");
                        if status.as_u16() == 403 {
                            if !self.api_key.ends_with(":fx") {
                                callbacks.log("💡 Ключ не заканчивается на :fx — возможно, это Free-ключ. Скопируйте его полностью из deepl.com (Ключи API и лимиты)".to_string(), "gold");
                            } else {
                                callbacks.log("💡 403: Ключ скопирован верно? Проверьте его в deepl.com → Ключи API и лимиты. Также возможна блокировка по IP — попробуйте VPN.".to_string(), "gold");
                            }
                        }
                        for k in chunk_keys {
                            results.insert(k.clone(), items[k].original.clone());
                        }
                    }
                }
                Err(err) => {
                    callbacks.log(format!("❌ Сетевая ошибка DeepL: {}", err), "red");
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
