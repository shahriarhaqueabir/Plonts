use tauri::State;

use crate::db;
use crate::error::AppResult;
use crate::state::AppState;
use garden_core::models::{ShoppingItem, Tool};

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state.db.lock().map_err(|_| crate::error::AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_shopping_items(state: State<AppState>) -> AppResult<Vec<ShoppingItem>> {
    db::supplies::list_shopping_items(&*lock(&state)?)
}

#[tauri::command]
pub fn add_shopping_item(state: State<AppState>, name: String, category: String, quantity: Option<String>) -> AppResult<ShoppingItem> {
    db::supplies::add_shopping_item(&*lock(&state)?, &name, &category, quantity.as_deref())
}

#[tauri::command]
pub fn toggle_shopping_item(state: State<AppState>, id: String) -> AppResult<ShoppingItem> {
    db::supplies::toggle_shopping_item(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn delete_shopping_item(state: State<AppState>, id: String) -> AppResult<()> {
    db::supplies::delete_shopping_item(&*lock(&state)?, &id)
}

#[tauri::command]
pub fn list_tools(state: State<AppState>) -> AppResult<Vec<Tool>> {
    db::supplies::list_tools(&*lock(&state)?)
}

#[tauri::command]
pub fn add_tool(state: State<AppState>, name: String, next_maintenance: Option<String>, notes: Option<String>) -> AppResult<Tool> {
    db::supplies::add_tool(&*lock(&state)?, &name, next_maintenance.as_deref(), notes.as_deref())
}

#[tauri::command]
pub fn mark_tool_maintained(state: State<AppState>, id: String, next_maintenance: Option<String>) -> AppResult<Tool> {
    db::supplies::mark_tool_maintained(&*lock(&state)?, &id, next_maintenance.as_deref())
}

#[tauri::command]
pub fn delete_tool(state: State<AppState>, id: String) -> AppResult<()> {
    db::supplies::delete_tool(&*lock(&state)?, &id)
}
