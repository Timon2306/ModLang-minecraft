use std::collections::HashMap;
use std::sync::Arc;

use crate::core::cache::TranslationCache;
use crate::core::config::AppConfig;
use crate::core::constants::LanguageInfo;
use crate::core::shield::{apply_smart_glue, mask_protected_fragments};
use crate::core::translator::deepl::DeepLEngine;
use crate::core::translator::gemini::GeminiEngine;
use crate::core::translator::google::GoogleEngine;
use crate::core::translator::openai::OpenAiEngine;
use crate::core::translator::traits::{EngineCallbacks, EngineItem, TranslationEngine};

pub struct TranslationService {
    engine: Arc<dyn TranslationEngine>,
    cache: TranslationCache,
    smart_glue: bool,
}

impl TranslationService {
    pub fn new(
        config: &AppConfig,
        cache: TranslationCache,
        dictionary: HashMap<String, String>,
        context: &str,
    ) -> Self {
        let engine: Arc<dyn TranslationEngine> = match config.engine.as_str() {
            "gemini" => Arc::new(GeminiEngine::new(
                config.gemini_api_key.clone(),
                config.gemini_model.clone(),
                context.to_string(),
                config.ai_mode.clone(),
                config.gemini_delay_ms,
                dictionary,
            )),
            "ai" | "openrouter" | "openai" => Arc::new(OpenAiEngine::new(
                config.openrouter_api_url.clone(),
                config.openrouter_api_key.clone(),
                config.openrouter_model.clone(),
                config.openrouter_site_url.clone(),
                config.openrouter_app_name.clone(),
                context.to_string(),
                config.ai_mode.clone(),
                dictionary,
            )),
            "deepl" => Arc::new(DeepLEngine::new(
                config.deepl_key.clone(),
                dictionary,
            )),
            "opencode" => Arc::new(OpenAiEngine::new(
                crate::core::constants::OPENCODE_GO_API.to_string(),
                config.opencode_api_key.clone(),
                config.opencode_model.clone(),
                String::new(),
                "ModLang Minecraft Translator".to_string(),
                context.to_string(),
                config.ai_mode.clone(),
                dictionary,
            )),
            _ => Arc::new(GoogleEngine::new(
                config.google_workers,
                config.google_mode.clone(),
                config.google_delay_ms,
                dictionary,
            )),
        };

        Self {
            engine,
            cache,
            smart_glue: config.smart_glue,
        }
    }

    pub async fn translate_dict(
        &self,
        strings: HashMap<String, String>,
        target_lang: &LanguageInfo,
        callbacks: &EngineCallbacks,
    ) -> HashMap<String, String> {
        if strings.is_empty() {
            return HashMap::new();
        }

        let mut results = HashMap::new();
        let mut pending = HashMap::new();
        let mut cached_count = 0usize;

        for (key, mut text) in strings {
            if !callbacks.should_run() {
                break;
            }
            callbacks.wait_if_paused().await;

            if self.smart_glue {
                text = apply_smart_glue(&text);
            }

            if let Some(hit) = self.cache.get(&target_lang.api_code, &text) {
                results.insert(key, hit);
                cached_count += 1;
                continue;
            }

            let (masked, mapping) = mask_protected_fragments(&text);
            if masked.is_empty() {
                results.insert(key, text);
                continue;
            }

            pending.insert(
                key.clone(),
                EngineItem {
                    key,
                    original: text,
                    masked,
                    mapping,
                },
            );
        }

        if cached_count > 0 {
            callbacks.log(format!("   [КЭШ] Из кэша: {} строк", cached_count), "dim");
        }

        if pending.is_empty() || !callbacks.should_run() {
            return results;
        }

        let translated = self.engine.translate_batch(pending.clone(), target_lang, callbacks).await;

        for (key, trans_text) in translated {
            if let Some(item) = pending.get(&key) {
                self.cache.set(&target_lang.api_code, &item.original, &trans_text);
                results.insert(key, trans_text);
            }
        }

        for (key, item) in pending {
            if !results.contains_key(&key) {
                results.insert(key, item.original);
            }
        }

        self.cache.save();
        results
    }
}
