use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::core::constants::LanguageInfo;

#[derive(Debug, Clone)]
pub struct EngineItem {
    pub key: String,
    pub original: String,
    pub masked: String,
    pub mapping: HashMap<String, String>,
}

#[derive(Clone)]
pub struct EngineCallbacks {
    pub cancel_token: CancellationToken,
    pub pause_flag: Arc<parking_lot::RwLock<bool>>,
    pub on_log: Arc<dyn Fn(String, String) + Send + Sync>,
    pub on_status: Arc<dyn Fn(String, Option<f64>) + Send + Sync>,
}

impl EngineCallbacks {
    pub fn should_run(&self) -> bool {
        !self.cancel_token.is_cancelled()
    }

    pub async fn wait_if_paused(&self) {
        while *self.pause_flag.read() && !self.cancel_token.is_cancelled() {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }
    }

    pub fn log(&self, msg: impl Into<String>, color: impl Into<String>) {
        (self.on_log)(msg.into(), color.into());
    }

    pub fn status(&self, text: impl Into<String>, progress: Option<f64>) {
        (self.on_status)(text.into(), progress);
    }
}

#[async_trait::async_trait]
pub trait TranslationEngine: Send + Sync {
    async fn translate_batch(
        &self,
        items: HashMap<String, EngineItem>,
        target_lang: &LanguageInfo,
        callbacks: &EngineCallbacks,
    ) -> HashMap<String, String>;
}
