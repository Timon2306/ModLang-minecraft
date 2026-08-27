use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Clone)]
pub struct TranslationCache {
    filepath: PathBuf,
    data: Arc<RwLock<HashMap<String, String>>>,
    dirty: Arc<AtomicBool>,
    changes_count: Arc<AtomicUsize>,
}

impl TranslationCache {
    pub fn new<P: AsRef<Path>>(filepath: P, _dictionary: &HashMap<String, String>) -> (Self, usize) {
        let path = filepath.as_ref().to_path_buf();
        let mut data_map = HashMap::new();

        if path.exists() {
            if let Ok(file) = File::open(&path) {
                let reader = BufReader::with_capacity(256 * 1024, file);
                if let Ok(raw_map) = serde_json::from_reader::<_, HashMap<String, String>>(reader) {
                    data_map = raw_map;
                }
            }
        }

        let cache = Self {
            filepath: path,
            data: Arc::new(RwLock::new(data_map)),
            dirty: Arc::new(AtomicBool::new(false)),
            changes_count: Arc::new(AtomicUsize::new(0)),
        };

        (cache, 0)
    }

    pub fn make_key(api_code: &str, source_text: &str) -> String {
        format!("{}_{}", api_code, source_text)
    }

    pub fn get(&self, api_code: &str, source_text: &str) -> Option<String> {
        let key = Self::make_key(api_code, source_text);
        self.data.read().get(&key).cloned()
    }

    pub fn set(&self, api_code: &str, source_text: &str, translated: &str) {
        let key = Self::make_key(api_code, source_text);
        {
            let mut writer = self.data.write();
            writer.insert(key, translated.to_string());
        }
        self.dirty.store(true, Ordering::SeqCst);
        let changes = self.changes_count.fetch_add(1, Ordering::SeqCst) + 1;
        if changes % 200 == 0 {
            self.save();
        }
    }

    pub fn len(&self) -> usize {
        self.data.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }

    pub fn clear(&self) {
        {
            let mut writer = self.data.write();
            writer.clear();
        }
        self.dirty.store(true, Ordering::SeqCst);
        self.save();
    }

    pub fn save(&self) {
        if !self.dirty.swap(false, Ordering::SeqCst) {
            return;
        }
        if let Some(parent) = self.filepath.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(file) = File::create(&self.filepath) {
            let writer = BufWriter::with_capacity(256 * 1024, file);
            let reader = self.data.read();
            let _ = serde_json::to_writer_pretty(writer, &*reader);
        }
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = File::create(path).map_err(|e| e.to_string())?;
        let writer = BufWriter::with_capacity(256 * 1024, file);
        let reader = self.data.read();
        serde_json::to_writer_pretty(writer, &*reader).map_err(|e| e.to_string())
    }

    pub fn import_from_json<P: AsRef<Path>>(&self, path: P) -> Result<usize, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::with_capacity(256 * 1024, file);
        let imported: HashMap<String, String> = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
        let count = imported.len();
        {
            let mut writer = self.data.write();
            for (k, v) in imported {
                writer.insert(k, v);
            }
        }
        self.dirty.store(true, Ordering::SeqCst);
        self.save();
        Ok(count)
    }
}
