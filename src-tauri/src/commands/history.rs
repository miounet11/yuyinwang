use crate::core::{error::Result, types::*};
use crate::services::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_history(state: State<'_, AppState>, limit: Option<usize>) -> Result<Vec<TranscriptionEntry>> {
    state.database.get_history(limit.unwrap_or(100))
}

#[tauri::command]
pub fn search_history(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<TranscriptionEntry>> {
    state.database.search(&query, limit.unwrap_or(50))
}

#[tauri::command]
pub fn delete_entry(state: State<'_, AppState>, id: String) -> Result<()> {
    state.database.delete(&id)
}
