use std::fs;
use std::sync::Arc;

pub mod commands;
pub mod core;
pub mod state;

use crate::commands::cache::*;
use crate::commands::glossary::*;
use crate::commands::scan::*;
use crate::commands::settings::*;
use crate::commands::translate::*;
use crate::core::cache::TranslationCache;
use crate::core::config::AppConfig;
use crate::core::constants::{CACHE_FILE_STD, DICT_FILE, SETTINGS_FILE};
use crate::state::job_runner::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load(SETTINGS_FILE);

    let dictionary = if std::path::Path::new(DICT_FILE).exists() {
        if let Ok(content) = fs::read_to_string(DICT_FILE) {
            serde_json::from_str(&content).unwrap_or_else(|_| crate::core::constants::get_default_dictionary())
        } else {
            crate::core::constants::get_default_dictionary()
        }
    } else {
        let default_dict = crate::core::constants::get_default_dictionary();
        if let Ok(json_str) = serde_json::to_string_pretty(&default_dict) {
            let _ = fs::write(DICT_FILE, json_str);
        }
        default_dict
    };

    let (cache, _polished) = TranslationCache::new(CACHE_FILE_STD, &dictionary);
    let app_state = Arc::new(AppState::new(config, cache, dictionary));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_languages_cmd,
            get_pack_formats_cmd,
            analyze_modpack_cmd,
            estimate_strings_cmd,
            start_translation_cmd,
            pause_translation_cmd,
            resume_translation_cmd,
            stop_translation_cmd,
            is_translation_running_cmd,
            open_resourcepack_folder_cmd,
            get_settings_cmd,
            save_settings_cmd,
            get_glossary_cmd,
            save_glossary_cmd,
            get_cache_stats_cmd,
            clear_cache_cmd,
            export_cache_cmd,
            import_cache_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
