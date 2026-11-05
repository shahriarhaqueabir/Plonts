use std::path::Path;

use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::storage;
use garden_core::llm::{ModelInfo, ModelKind};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_models(state: State<AppState>) -> AppResult<Vec<ModelInfo>> {
    db::models::list_models(&*lock(&state)?)
}

#[tauri::command]
pub fn import_model(
    state: State<AppState>,
    source_path: String,
    name: String,
    kind: ModelKind,
) -> AppResult<ModelInfo> {
    let src = Path::new(&source_path);
    let filename = storage::save_model_file(&state.app_data, src)?;
    let abs = storage::models_dir(&state.app_data).join(&filename);
    let size_bytes = std::fs::metadata(&abs).map(|m| m.len() as i64).unwrap_or(0);
    let path_str = abs.to_string_lossy().to_string();
    db::models::create_model(&*lock(&state)?, &name, &path_str, size_bytes, kind)
}

#[tauri::command]
pub fn delete_model(state: State<AppState>, id: String) -> AppResult<()> {
    let info = db::models::delete_model(&*lock(&state)?, &id)?;
    let filename = Path::new(&info.path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if !filename.is_empty() {
        let _ = storage::delete_model_file(&state.app_data, filename);
    }
    Ok(())
}