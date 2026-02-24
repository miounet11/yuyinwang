use crate::core::{error::Result, types::*};
use crate::services::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings> {
    Ok(state.settings.lock().clone())
}

#[tauri::command]
pub fn update_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<()> {
    state.save_settings(settings)?;
    Ok(())
}
