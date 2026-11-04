use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{Guide, GuideFilters, GuideProgress};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_guides(state: State<AppState>, filters: Option<GuideFilters>) -> AppResult<Vec<Guide>> {
    db::guides::list_guides(&*lock(&state)?, &filters.unwrap_or_default())
}

#[tauri::command]
pub fn get_guide(state: State<AppState>, id: String) -> AppResult<Guide> {
    db::guides::get_guide(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn save_guide_progress(
    state: State<AppState>,
    guide_id: String,
    current_step: i64,
    completed: bool,
) -> AppResult<GuideProgress> {
    db::guides::save_progress(&*lock(&state)?, &guide_id, current_step, completed)
}

#[tauri::command]
pub fn set_guide_bookmark(state: State<AppState>, guide_id: String, bookmarked: bool) -> AppResult<Guide> {
    db::guides::set_bookmark(&*lock(&state)?, &guide_id, bookmarked)
}
