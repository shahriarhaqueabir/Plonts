use std::path::Path;

use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{
    JournalEntry, JournalFilters, JournalPhoto, Measurement, MeasurementKind, PlantTimeline,
};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_journal_entries(
    state: State<AppState>,
    filters: Option<JournalFilters>,
) -> AppResult<Vec<JournalEntry>> {
    let app_data = state.app_data.clone();
    db::journal::list_entries(&*lock(&state)?, &app_data, &filters.unwrap_or_default())
}

#[tauri::command]
pub fn create_journal_entry(
    state: State<AppState>,
    plant_id: Option<String>,
    bed_id: Option<String>,
    text: String,
    tags: Vec<String>,
) -> AppResult<JournalEntry> {
    let app_data = state.app_data.clone();
    db::journal::create_entry(
        &*lock(&state)?,
        &app_data,
        plant_id.as_deref(),
        bed_id.as_deref(),
        &text,
        &tags,
    )
}

#[tauri::command]
pub fn update_journal_entry(
    state: State<AppState>,
    id: String,
    plant_id: Option<String>,
    bed_id: Option<String>,
    text: String,
    tags: Vec<String>,
) -> AppResult<JournalEntry> {
    let app_data = state.app_data.clone();
    db::journal::update_entry(
        &*lock(&state)?,
        &app_data,
        &id,
        plant_id.as_deref(),
        bed_id.as_deref(),
        &text,
        &tags,
    )
}

#[tauri::command]
pub fn delete_journal_entry(state: State<AppState>, id: String) -> AppResult<()> {
    let filenames = {
        let conn = lock(&state)?;
        db::journal::delete_entry(&conn, &id)?
    };
    for filename in filenames {
        crate::storage::delete_photo_file(&state.app_data, &filename)?;
    }
    Ok(())
}

#[tauri::command]
pub fn add_journal_photo(state: State<AppState>, entry_id: String, source_path: String) -> AppResult<JournalPhoto> {
    let filename = crate::storage::save_photo_file(&state.app_data, Path::new(&source_path))?;
    let app_data = state.app_data.clone();
    db::journal::insert_photo(&*lock(&state)?, &app_data, &entry_id, &filename)
}

#[tauri::command]
pub fn delete_journal_photo(state: State<AppState>, photo_id: String) -> AppResult<()> {
    let filename = {
        let conn = lock(&state)?;
        db::journal::delete_photo(&conn, &photo_id)?
    };
    crate::storage::delete_photo_file(&state.app_data, &filename)?;
    Ok(())
}

#[tauri::command]
pub fn export_journal_photo(state: State<AppState>, photo_id: String, dest_path: String) -> AppResult<()> {
    let filename = {
        let app_data = state.app_data.clone();
        let conn = lock(&state)?;
        db::journal::get_photo(&conn, &app_data, &photo_id)?.path
    };
    crate::storage::copy_photo_out(&state.app_data, &filename, Path::new(&dest_path))?;
    Ok(())
}

#[tauri::command]
pub fn add_measurement(
    state: State<AppState>,
    plant_id: String,
    kind: MeasurementKind,
    value: f64,
    unit: String,
) -> AppResult<Measurement> {
    db::journal::add_measurement(&*lock(&state)?, &plant_id, kind, value, &unit)
}

#[tauri::command]
pub fn list_measurements(state: State<AppState>, plant_id: String) -> AppResult<Vec<Measurement>> {
    db::journal::list_measurements(&*lock(&state)?, &plant_id)
}

#[tauri::command]
pub fn delete_measurement(state: State<AppState>, id: String) -> AppResult<()> {
    db::journal::delete_measurement(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn get_plant_timeline(state: State<AppState>, plant_id: String) -> AppResult<PlantTimeline> {
    let app_data = state.app_data.clone();
    db::journal::get_timeline(&*lock(&state)?, &app_data, &plant_id)
}
