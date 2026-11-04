use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::Seed;

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state.db.lock().map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_seeds(state: State<AppState>) -> AppResult<Vec<Seed>> {
    db::seeds::list_seeds(&*lock(&state)?)
}

#[tauri::command]
pub fn add_seed(
    state: State<AppState>,
    catalog_id: Option<String>,
    variety: String,
    source: String,
    year: i64,
    quantity: i64,
    germination_notes: Option<String>,
) -> AppResult<Seed> {
    db::seeds::add_seed(&*lock(&state)?, catalog_id.as_deref(), &variety, &source, year, quantity, germination_notes.as_deref())
}

#[tauri::command]
pub fn delete_seed(state: State<AppState>, id: String) -> AppResult<()> {
    db::seeds::delete_seed(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn acknowledge_seed_viability(state: State<AppState>, id: String) -> AppResult<Seed> {
    db::seeds::acknowledge_viability(&*lock(&state)?, &id)
}
