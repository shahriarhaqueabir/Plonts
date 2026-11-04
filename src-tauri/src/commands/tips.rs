use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::DailyTip;

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn get_daily_tip(state: State<AppState>) -> AppResult<Option<DailyTip>> {
    let now = chrono::Utc::now().to_rfc3339();
    db::tips::get_daily_tip(&*lock(&state)?, &now)
}

#[tauri::command]
pub fn dismiss_daily_tip(state: State<AppState>, id: String, days: i64) -> AppResult<DailyTip> {
    db::tips::dismiss_tip(&*lock(&state)?, &id, days)
}

#[tauri::command]
pub fn snooze_daily_tip(state: State<AppState>, id: String, hours: i64) -> AppResult<DailyTip> {
    db::tips::snooze_tip(&*lock(&state)?, &id, hours)
}
