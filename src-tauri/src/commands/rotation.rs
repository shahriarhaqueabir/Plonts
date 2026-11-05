use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::RotationPlan;

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_rotation_plans(state: State<AppState>, garden_id: String) -> AppResult<Vec<RotationPlan>> {
    db::rotation::list_rotation_plans(&*lock(&state)?, &garden_id)
}

#[tauri::command]
pub fn create_rotation_plan(
    state: State<AppState>,
    garden_id: String,
    season: String,
    plant_family: String,
    notes: Option<String>,
) -> AppResult<RotationPlan> {
    db::rotation::create_rotation_plan(&*lock(&state)?, &garden_id, &season, &plant_family, notes.as_deref())
}

#[tauri::command]
pub fn delete_rotation_plan(state: State<AppState>, id: String) -> AppResult<()> {
    db::rotation::delete_rotation_plan(&*lock(&state)?, &id)
}
