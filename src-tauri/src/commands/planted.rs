use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::Planting;

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn add_planting(
    state: State<AppState>,
    bed_id: String,
    catalog_id: String,
    quantity: i64,
    source: String,
    notes: Option<String>,
) -> AppResult<Planting> {
    db::planted::add_planting(&*lock(&state)?, &bed_id, &catalog_id, quantity, &source, notes.as_deref())
}

#[tauri::command]
pub fn list_plantings(state: State<AppState>) -> AppResult<Vec<Planting>> {
    db::planted::list_plantings(&*lock(&state)?)
}

#[tauri::command]
pub fn update_planting_status(
    state: State<AppState>,
    id: String,
    status: String,
    removal_reason: Option<String>,
) -> AppResult<Planting> {
    db::planted::update_status(&*lock(&state)?, &id, &status, removal_reason.as_deref())
}

#[tauri::command]
pub fn remove_planting(state: State<AppState>, id: String) -> AppResult<()> {
    db::planted::remove_planting(&*lock(&state)?, &id)
}