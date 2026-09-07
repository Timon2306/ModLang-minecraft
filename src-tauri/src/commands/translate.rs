use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::core::config::AppConfig;
use crate::state::job_runner::{run_translation_job, AppState};

#[tauri::command]
pub async fn start_translation_cmd(
    app_handle: AppHandle,
    state: State<'_, Arc<AppState>>,
    config: Option<AppConfig>,
) -> Result<(), String> {
    if *state.is_running.read() {
        return Err("Перевод уже выполняется".to_string());
    }

    if let Some(cfg) = config {
        *state.config.write() = cfg.clone();
        let _ = cfg.save(crate::core::constants::SETTINGS_FILE);
    }

    let state_clone = state.inner().clone();
    tokio::spawn(async move {
        let _ = run_translation_job(app_handle, state_clone).await;
    });

    Ok(())
}

#[tauri::command]
pub fn pause_translation_cmd(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    *state.pause_flag.write() = true;
    Ok(())
}

#[tauri::command]
pub fn resume_translation_cmd(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    *state.pause_flag.write() = false;
    Ok(())
}

#[tauri::command]
pub fn stop_translation_cmd(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.cancel_token.read().cancel();
    *state.pause_flag.write() = false;
    *state.is_running.write() = false;
    Ok(())
}

#[tauri::command]
pub fn is_translation_running_cmd(state: State<'_, Arc<AppState>>) -> bool {
    *state.is_running.read()
}

#[tauri::command]
pub fn open_resourcepack_folder_cmd(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let cfg = state.config.read().clone();
    let path = std::path::Path::new(&cfg.mc_dir).join("resourcepacks");
    let _ = std::fs::create_dir_all(&path);
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(&path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path).spawn();
    }
    Ok(())
}
