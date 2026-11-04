use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{CalendarEntry, Hemisphere};

#[tauri::command]
pub fn get_calendar(state: State<AppState>, month: i32, hemisphere: Hemisphere) -> AppResult<Vec<CalendarEntry>> {
    let conn = state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))?;
    db::calendar::get_calendar(&conn, month, hemisphere)
}