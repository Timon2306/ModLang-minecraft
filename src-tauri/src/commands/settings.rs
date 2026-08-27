use std::sync::Arc;
use tauri::State;

use crate::core::config::AppConfig;
use crate::core::constants::SETTINGS_FILE;
use crate::state::job_runner::AppState;

#[tauri::command]
pub fn get_settings_cmd(state: State<'_, Arc<AppState>>) -> AppConfig {
    state.config.read().clone()
}

#[tauri::command]
pub fn save_settings_cmd(
    new_config: AppConfig,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    new_config.save(SETTINGS_FILE)?;
    *state.config.write() = new_config;
    Ok(())
}
