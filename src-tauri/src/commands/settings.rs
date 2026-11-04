use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::Settings;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> AppResult<Settings> {
    let conn = state.db.lock().map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))?;
    db::settings::get(&conn)
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, settings: Settings) -> AppResult<Settings> {
    let conn = state.db.lock().map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))?;
    db::settings::upsert(&conn, &settings)
}
