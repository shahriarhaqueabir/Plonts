use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{Problem, ProblemCause};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state.db.lock().map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn diagnose_problem(state: State<AppState>, symptom: String, plant_id: Option<String>) -> AppResult<Vec<ProblemCause>> {
    db::problems::diagnose(&*lock(&state)?, &symptom, plant_id.as_deref())
}

#[tauri::command]
pub fn list_problems(state: State<AppState>) -> AppResult<Vec<Problem>> {
    db::problems::list_problems(&*lock(&state)?)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_problem(
    state: State<AppState>,
    planting_id: Option<String>,
    symptom: String,
    affected_part: Option<String>,
    spread: Option<String>,
    cause: String,
    treatment: String,
    prevention: String,
    follow_up_at: Option<String>,
) -> AppResult<Problem> {
    db::problems::create_problem(
        &*lock(&state)?,
        planting_id.as_deref(),
        &symptom,
        affected_part.as_deref(),
        spread.as_deref(),
        &cause,
        &treatment,
        &prevention,
        follow_up_at.as_deref(),
    )
}

#[tauri::command]
pub fn resolve_problem(state: State<AppState>, id: String, resolution_note: Option<String>) -> AppResult<Problem> {
    db::problems::resolve_problem(&*lock(&state)?, &id, resolution_note.as_deref())
}

#[tauri::command]
pub fn delete_problem(state: State<AppState>, id: String) -> AppResult<()> {
    db::problems::delete_problem(&*lock(&state)?, &id)
}
