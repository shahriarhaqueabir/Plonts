use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{Bed, Garden};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_gardens(state: State<AppState>) -> AppResult<Vec<Garden>> {
    db::gardens::list_gardens(&*lock(&state)?)
}

#[tauri::command]
pub fn create_garden(state: State<AppState>, name: String, garden_type: String) -> AppResult<Garden> {
    db::gardens::create_garden(&*lock(&state)?, &name, &garden_type)
}

#[tauri::command]
pub fn delete_garden(state: State<AppState>, id: String) -> AppResult<()> {
    db::gardens::delete_garden(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn list_beds(state: State<AppState>, garden_id: String) -> AppResult<Vec<Bed>> {
    db::gardens::list_beds(&*lock(&state)?, &garden_id)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_bed(
    state: State<AppState>,
    garden_id: String,
    name: String,
    soil_type: Option<String>,
    sun_exposure: Option<String>,
    width_cm: Option<i64>,
    height_cm: Option<i64>,
    is_container: bool,
) -> AppResult<Bed> {
    db::gardens::create_bed(
        &*lock(&state)?,
        &garden_id,
        &name,
        soil_type.as_deref(),
        sun_exposure.as_deref(),
        width_cm,
        height_cm,
        is_container,
    )
}

#[tauri::command]
pub fn delete_bed(state: State<AppState>, id: String) -> AppResult<()> {
    db::gardens::delete_bed(&*lock(&state)?, &id)
}
