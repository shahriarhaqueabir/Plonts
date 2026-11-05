use std::path::Path;

use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use garden_core::models::{Layout, LayoutDiff, LayoutPlant, LayoutWarning};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_layouts(state: State<AppState>, garden_id: String) -> AppResult<Vec<Layout>> {
    db::layouts::list_layouts(&*lock(&state)?, &garden_id)
}

#[tauri::command]
pub fn get_layout(state: State<AppState>, id: String) -> AppResult<Layout> {
    db::layouts::get_layout(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn create_layout(state: State<AppState>, garden_id: String, name: String) -> AppResult<Layout> {
    db::layouts::create_layout(&*lock(&state)?, &garden_id, &name)
}

#[tauri::command]
pub fn save_layout(state: State<AppState>, id: String, canvas_json: String) -> AppResult<Layout> {
    db::layouts::save_layout(&*lock(&state)?, &id, &canvas_json)
}

#[tauri::command]
pub fn delete_layout(state: State<AppState>, id: String) -> AppResult<()> {
    db::layouts::delete_layout(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn duplicate_layout(state: State<AppState>, id: String) -> AppResult<Layout> {
    db::layouts::duplicate_layout(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn compare_layouts(state: State<AppState>, a_id: String, b_id: String) -> AppResult<LayoutDiff> {
    db::layouts::compare_layouts(&*lock(&state)?, &a_id, &b_id)
}

#[tauri::command]
pub fn list_layout_plants(state: State<AppState>, layout_id: String) -> AppResult<Vec<LayoutPlant>> {
    db::layouts::list_layout_plants(&*lock(&state)?, &layout_id)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn add_layout_plant(
    state: State<AppState>,
    layout_id: String,
    catalog_id: String,
    bed_id: Option<String>,
    x: f64,
    y: f64,
    quantity: i64,
) -> AppResult<LayoutPlant> {
    db::layouts::add_layout_plant(&*lock(&state)?, &layout_id, &catalog_id, bed_id.as_deref(), x, y, quantity)
}

#[tauri::command]
pub fn update_layout_plant(
    state: State<AppState>,
    id: String,
    bed_id: Option<String>,
    x: f64,
    y: f64,
    quantity: i64,
) -> AppResult<LayoutPlant> {
    db::layouts::update_layout_plant(&*lock(&state)?, &id, bed_id.as_deref(), x, y, quantity)
}

#[tauri::command]
pub fn delete_layout_plant(state: State<AppState>, id: String) -> AppResult<()> {
    db::layouts::delete_layout_plant(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn analyze_layout(state: State<AppState>, layout_id: String) -> AppResult<Vec<LayoutWarning>> {
    db::layouts::analyze_layout(&*lock(&state)?, &layout_id)
}

/// Write a base64-encoded PNG export to a user-chosen path.
#[tauri::command]
pub fn save_export(path: String, data_base64: String) -> AppResult<()> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64.trim())
        .map_err(|e| AppError::InvalidInput(format!("invalid base64: {e}")))?;
    std::fs::write(Path::new(&path), bytes)?;
    Ok(())
}
