use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

use crate::core::constants::{get_languages, get_pack_formats, LanguageInfo, PackFormat};
use crate::core::scanner::analyzer::{analyze_modpack, AnalysisResult};
use crate::core::scanner::estimator::estimate_total_strings;
use crate::state::job_runner::AppState;

#[tauri::command]
pub fn get_languages_cmd() -> Vec<LanguageInfo> {
    get_languages()
}

#[tauri::command]
pub fn get_pack_formats_cmd() -> std::collections::HashMap<String, PackFormat> {
    get_pack_formats()
}

#[tauri::command]
pub async fn analyze_modpack_cmd(
    state: State<'_, Arc<AppState>>,
) -> Result<AnalysisResult, String> {
    let config = state.config.read().clone();
    let mc_dir = PathBuf::from(&config.mc_dir);
    if !mc_dir.exists() {
        return Err("Папка Minecraft не найдена".to_string());
    }

    let target_lang = get_languages()
        .into_iter()
        .find(|l| l.label == config.target_language)
        .ok_or("Целевой язык не найден")?;

    let res = tokio::task::spawn_blocking(move || {
        analyze_modpack(
            &mc_dir,
            &target_lang.regex_pattern,
            config.translate_mods,
            config.translate_books,
            config.translate_quests,
            config.smart_glue,
        )
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(res)
}

#[tauri::command]
pub async fn estimate_strings_cmd(
    state: State<'_, Arc<AppState>>,
) -> Result<usize, String> {
    let config = state.config.read().clone();
    let mc_dir = PathBuf::from(&config.mc_dir);
    if !mc_dir.exists() {
        return Err("Папка Minecraft не найдена".to_string());
    }

    let target_lang = get_languages()
        .into_iter()
        .find(|l| l.label == config.target_language)
        .ok_or("Целевой язык не найден")?;

    let count = tokio::task::spawn_blocking(move || {
        estimate_total_strings(
            &mc_dir,
            &target_lang.regex_pattern,
            &config.process_mode,
            config.translate_mods,
            config.translate_books,
            config.translate_quests,
            config.smart_glue,
        )
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(count)
}
