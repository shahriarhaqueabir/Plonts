use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::storage;
use garden_core::llm::{ChatConversation, ChatMessage, ContextFlags, Task};
use garden_core::llm::split_plan_into_tasks;

fn lock(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| AppError::InvalidInput("db lock poisoned".into()))
}

#[tauri::command]
pub fn list_conversations(state: State<AppState>) -> AppResult<Vec<ChatConversation>> {
    db::chat::list_conversations(&*lock(&state)?)
}

#[tauri::command]
pub fn create_conversation(
    state: State<AppState>,
    title: String,
    flags: ContextFlags,
) -> AppResult<ChatConversation> {
    db::chat::create_conversation(&*lock(&state)?, &title, &flags)
}

#[tauri::command]
pub fn update_conversation_flags(
    state: State<AppState>,
    id: String,
    flags: ContextFlags,
) -> AppResult<ChatConversation> {
    db::chat::update_conversation_flags(&*lock(&state)?, &id, &flags)
}

#[tauri::command]
pub fn delete_conversation(state: State<AppState>, id: String) -> AppResult<()> {
    let names = db::chat::delete_conversation(&*lock(&state)?, &id)?;
    for n in names {
        let _ = storage::delete_photo_file(&state.app_data, &n);
    }
    Ok(())
}

#[tauri::command]
pub fn clear_conversation(state: State<AppState>, id: String) -> AppResult<()> {
    let names = db::chat::clear_conversation(&*lock(&state)?, &id)?;
    for n in names {
        let _ = storage::delete_photo_file(&state.app_data, &n);
    }
    Ok(())
}

#[tauri::command]
pub fn clear_all_conversations(state: State<AppState>) -> AppResult<()> {
    let names = db::chat::clear_all_conversations(&*lock(&state)?)?;
    for n in names {
        let _ = storage::delete_photo_file(&state.app_data, &n);
    }
    Ok(())
}

#[tauri::command]
pub fn list_messages(state: State<AppState>, conversation_id: String) -> AppResult<Vec<ChatMessage>> {
    db::chat::list_messages(&*lock(&state)?, &state.app_data, &conversation_id)
}

#[tauri::command]
pub fn flag_message(
    state: State<AppState>,
    message_id: String,
    note: Option<String>,
) -> AppResult<ChatMessage> {
    db::chat::flag_message(&*lock(&state)?, &state.app_data, &message_id, note.as_deref())
}

#[tauri::command]
pub fn create_tasks_from_text(state: State<AppState>, text: String) -> AppResult<Vec<Task>> {
    let titles = split_plan_into_tasks(&text);
    let conn = lock(&state)?;
    let mut out = Vec::new();
    for t in titles {
        out.push(db::tasks::create_task(&*conn, &t, None, None)?);
    }
    Ok(out)
}

#[tauri::command]
pub fn create_journal_from_text(state: State<AppState>, text: String) -> AppResult<garden_core::models::JournalEntry> {
    let conn = lock(&state)?;
    db::journal::create_entry(&*conn, &state.app_data, None, None, &text, &[])
}

#[tauri::command]
pub fn create_layout_from_text(state: State<AppState>, text: String) -> AppResult<garden_core::models::Layout> {
    let conn = lock(&state)?;
    let gardens = db::gardens::list_gardens(&*conn)?;
    let garden = gardens
        .first()
        .ok_or_else(|| AppError::InvalidInput("create a garden first".into()))?;
    let name = format!("Coach draft - {}", text.chars().take(40).collect::<String>());
    db::layouts::create_layout(&*conn, &garden.id, &name)
}