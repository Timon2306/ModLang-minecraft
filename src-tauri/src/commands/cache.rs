use std::sync::Arc;
use tauri::State;

use crate::state::job_runner::AppState;

#[tauri::command]
pub fn get_cache_stats_cmd(state: State<'_, Arc<AppState>>) -> usize {
    state.cache.len()
}

#[tauri::command]
pub fn clear_cache_cmd(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.cache.clear();
    Ok(())
}

#[tauri::command]
pub fn export_cache_cmd(
    export_path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.cache.export_to_json(export_path)
}

#[tauri::command]
pub fn import_cache_cmd(
    import_path: String,
    state: State<'_, Arc<AppState>>,
) -> Result<usize, String> {
    state.cache.import_from_json(import_path)
}
