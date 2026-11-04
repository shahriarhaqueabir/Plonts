use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{Harvest, YieldRow};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state.db.lock().map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_harvests(state: State<AppState>) -> AppResult<Vec<Harvest>> {
    db::harvests::list_harvests(&*lock(&state)?)
}

#[tauri::command]
pub fn add_harvest(
    state: State<AppState>,
    planting_id: Option<String>,
    catalog_id: Option<String>,
    date: String,
    weight_g: Option<f64>,
    count: Option<i64>,
    notes: Option<String>,
) -> AppResult<Harvest> {
    db::harvests::add_harvest(&*lock(&state)?, planting_id.as_deref(), catalog_id.as_deref(), &date, weight_g, count, notes.as_deref())
}

#[tauri::command]
pub fn delete_harvest(state: State<AppState>, id: String) -> AppResult<()> {
    db::harvests::delete_harvest(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn yield_summary(state: State<AppState>, group_by: String) -> AppResult<Vec<YieldRow>> {
    db::harvests::yield_summary(&*lock(&state)?, &group_by)
}
