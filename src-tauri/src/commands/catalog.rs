use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{PlantDetail, SearchFilters, SearchResult};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn search_plants(
    state: State<AppState>,
    query: String,
    filters: Option<SearchFilters>,
    limit: Option<usize>,
) -> AppResult<Vec<SearchResult>> {
    db::catalog::search(&*lock(&state)?, &query, &filters.unwrap_or_default(), limit.unwrap_or(100))
}

#[tauri::command]
pub fn get_plant_detail(state: State<AppState>, id: String) -> AppResult<PlantDetail> {
    db::catalog::get_plant_detail(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn list_categories(state: State<AppState>) -> AppResult<Vec<String>> {
    db::catalog::list_categories(&*lock(&state)?)
}