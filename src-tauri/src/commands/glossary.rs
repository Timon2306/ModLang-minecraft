use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tauri::State;

use crate::core::constants::DICT_FILE;
use crate::state::job_runner::AppState;

#[tauri::command]
pub fn get_glossary_cmd(state: State<'_, Arc<AppState>>) -> HashMap<String, String> {
    state.dictionary.read().clone()
}

#[tauri::command]
pub fn save_glossary_cmd(
    new_dict: HashMap<String, String>,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let json_str = serde_json::to_string_pretty(&new_dict).map_err(|e| e.to_string())?;
    fs::write(DICT_FILE, json_str).map_err(|e| e.to_string())?;
    *state.dictionary.write() = new_dict;
    Ok(())
}
