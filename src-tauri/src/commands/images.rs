use std::path::Path;

use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use garden_core::models::{Credit, PlantImage};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| AppError::InvalidInput("db lock poisoned".into()))
}

fn resolve(state: &AppState, catalog_id: &str) -> AppResult<PlantImage> {
    let meta = db::catalog::get_plant_image_meta(&*lock(state)?, catalog_id)?;
    Ok(crate::images::resolve_plant_image(
        &state.app_data,
        &state.resource_dir,
        &state.images,
        &state.manifest,
        catalog_id,
        meta.asset_id.as_deref(),
        meta.image_path.as_deref(),
        meta.license.as_deref(),
        meta.attribution.as_deref(),
        meta.category.as_deref(),
        meta.override_filename.as_deref(),
    ))
}

#[tauri::command]
pub fn get_plant_image(state: State<AppState>, catalog_id: String) -> AppResult<PlantImage> {
    resolve(&state, &catalog_id)
}

#[tauri::command]
pub fn get_generic_image(state: State<AppState>) -> AppResult<String> {
    Ok(crate::images::generic_url(&state.resource_dir, &state.images, &state.manifest))
}

#[tauri::command]
pub fn set_plant_image_override(
    state: State<AppState>,
    catalog_id: String,
    source_path: String,
) -> AppResult<PlantImage> {
    let filename = crate::storage::save_plant_image(&state.app_data, Path::new(&source_path))?;
    let previous = {
        let conn = lock(&state)?;
        db::catalog::set_plant_image_override(&conn, &catalog_id, &filename)?
    };
    if let Some(old) = previous {
        crate::storage::delete_plant_image(&state.app_data, &old)?;
    }
    resolve(&state, &catalog_id)
}

#[tauri::command]
pub fn clear_plant_image_override(state: State<AppState>, catalog_id: String) -> AppResult<PlantImage> {
    let previous = {
        let conn = lock(&state)?;
        db::catalog::clear_plant_image_override(&conn, &catalog_id)?
    };
    if let Some(old) = previous {
        crate::storage::delete_plant_image(&state.app_data, &old)?;
    }
    resolve(&state, &catalog_id)
}

#[tauri::command]
pub fn list_credits() -> AppResult<Vec<Credit>> {
    crate::images::list_credits()
}