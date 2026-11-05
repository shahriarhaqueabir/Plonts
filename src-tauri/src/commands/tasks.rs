use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use garden_core::llm::{Task, TaskStatus};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_tasks(state: State<AppState>) -> AppResult<Vec<Task>> {
    db::tasks::list_tasks(&*lock(&state)?)
}

#[tauri::command]
pub fn create_task(
    state: State<AppState>,
    title: String,
    body: Option<String>,
    due_at: Option<String>,
) -> AppResult<Task> {
    db::tasks::create_task(&*lock(&state)?, &title, body.as_deref(), due_at.as_deref())
}

#[tauri::command]
pub fn set_task_status(state: State<AppState>, id: String, status: TaskStatus) -> AppResult<Task> {
    db::tasks::set_task_status(&*lock(&state)?, &id, status)
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: String) -> AppResult<()> {
    db::tasks::delete_task(&*lock(&state)?, &id)
}