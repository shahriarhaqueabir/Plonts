use std::path::Path;

use chrono::Datelike;
use tauri::State;

use crate::db;
use crate::error::{AppError, AppResult};
use crate::llm;
use crate::state::AppState;
use crate::storage;
use garden_core::llm::{ChatResponse, ContextFlags, ModelInfo, ModelKind};
use garden_core::models::Settings;

fn lock_db(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, rusqlite::Connection>> {
    state
        .db
        .lock()
        .map_err(|_| AppError::InvalidInput("db lock poisoned".into()))
}

fn lock_llm(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, Option<llm::LoadedModel>>> {
    state
        .llm
        .lock()
        .map_err(|_| AppError::InvalidInput("llm lock poisoned".into()))
}

#[tauri::command]
pub async fn load_model(state: State<'_, AppState>, id: String) -> AppResult<ModelInfo> {
    let info = {
        let conn = lock_db(&state)?;
        db::models::get_model(&*conn, &id)?
    };
    let loaded = llm::load_gguf(&info.id, &info.path, info.kind).await?;
    {
        let mut guard = lock_llm(&state)?;
        *guard = Some(loaded);
    }
    {
        let conn = lock_db(&state)?;
        let _ = db::models::touch_used(&*conn, &id);
    }
    Ok(info)
}

#[tauri::command]
pub fn unload_model(state: State<AppState>) -> AppResult<()> {
    let mut guard = lock_llm(&state)?;
    *guard = None;
    Ok(())
}

#[tauri::command]
pub fn get_model_status(state: State<AppState>) -> AppResult<Option<ModelInfo>> {
    let loaded_id = {
        let guard = lock_llm(&state)?;
        guard.as_ref().map(|m| m.model_id.clone())
    };
    match loaded_id {
        Some(id) => {
            let conn = lock_db(&state)?;
            Ok(Some(db::models::get_model(&*conn, &id)?))
        }
        None => Ok(None),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn generate_chat(
    state: State<'_, AppState>,
    conversation_id: String,
    content: String,
    flags: ContextFlags,
    attachment_source_path: Option<String>,
    mime: Option<String>,
) -> AppResult<ChatResponse> {
    // 1. Persist user message.
    let (conv_id, attachment_filename) = {
        let conn = lock_db(&state)?;
        // get or create conversation
        let conv = match db::chat::get_conversation(&*conn, &conversation_id) {
            Ok(c) => c,
            Err(_) => {
                let title = content.chars().take(40).collect::<String>();
                let title = if title.trim().is_empty() {
                    "New chat".to_string()
                } else {
                    title
                };
                db::chat::create_conversation(&*conn, &title, &flags)?
            }
        };
        // update flags
        let _ = db::chat::update_conversation_flags(&*conn, &conv.id, &flags);
        let msg = db::chat::insert_message(&*conn, &state.app_data, &conv.id, garden_core::llm::MessageRole::User, &content)?;
        let mut fname = None;
        if let Some(src) = &attachment_source_path {
            let filename = storage::save_photo_file(&state.app_data, Path::new(src))?;
            db::chat::insert_attachment(&*conn, &state.app_data, &msg.id, &filename, mime.as_deref())?;
            fname = Some(filename);
        }
        (conv.id, fname)
    };

    // 2. Gather context (db) in short locked scope.
    let (garden_ctx, zone_ctx, tasks_ctx, history) = {
        let conn = lock_db(&state)?;
        // settings
        let settings = db::settings::get(&*conn).unwrap_or(Settings::default());
        // zone context
        let zone = if flags.zone {
            Some(format!(
                "hemisphere: {:?}, city: {}, experience: {:?}",
                settings.hemisphere,
                settings.city.clone().unwrap_or_else(|| "unknown".to_string()),
                settings.experience_level
            ))
        } else {
            None
        };
        // garden context
        let garden = if flags.garden {
            let mut s = String::new();
            let gardens = db::gardens::list_gardens(&*conn).unwrap_or_default();
            for g in &gardens {
                s.push_str(&format!("Garden: {} ({})\n", g.name, g.garden_type));
                if let Ok(beds) = db::gardens::list_beds(&*conn, &g.id) {
                    for b in beds {
                        s.push_str(&format!(
                            "  Bed: {} ({:?}x{:?} cm)\n",
                            b.name, b.width_cm, b.height_cm
                        ));
                    }
                }
            }
            let plantings = db::planted::list_plantings(&*conn).unwrap_or_default();
            if !plantings.is_empty() {
                s.push_str("Plantings:\n");
                for p in plantings {
                    s.push_str(&format!(
                        "  - {} x{} in bed {} ({})\n",
                        p.plant_name, p.quantity, p.bed_name, p.status
                    ));
                }
            }
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        } else {
            None
        };
        // tasks context
        let tasks = if flags.tasks {
            // calendar eligible this month
            let month0 = (chrono::Utc::now().month() as i32) - 1;
            let entries = db::calendar::get_calendar(&*conn, month0, settings.hemisphere).unwrap_or_default();
            let eligible: Vec<String> = entries
                .iter()
                .filter(|e| e.eligible)
                .map(|e| e.plant.name.clone())
                .collect();
            // also open tasks
            let open_tasks = db::tasks::list_tasks(&*conn).unwrap_or_default();
            let mut s = String::new();
            if !eligible.is_empty() {
                s.push_str("Eligible to sow this month: ");
                s.push_str(&eligible.join(", "));
                s.push('\n');
            }
            if !open_tasks.is_empty() {
                s.push_str("Open tasks:\n");
                for t in open_tasks {
                    if t.status == garden_core::llm::TaskStatus::Open {
                        s.push_str(&format!("- {}\n", t.title));
                    }
                }
            }
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        } else {
            None
        };
        // history: all messages except the last (the one we just inserted)
        let msgs = db::chat::list_messages(&*conn, &state.app_data, &conv_id).unwrap_or_default();
        let mut hist: Vec<(garden_core::llm::MessageRole, String)> = Vec::new();
        let n = msgs.len();
        for (i, m) in msgs.iter().enumerate() {
            if i == n - 1 {
                break;
            }
            hist.push((m.role, m.content.clone()));
        }
        (garden, zone, tasks, hist)
    };

    // 3. Retrieve.
    let chunks = {
        let conn = lock_db(&state)?;
        db::retrieval::retrieve(&*conn, &content, llm::MAX_RETRIEVED).unwrap_or_default()
    };
    let sources = llm::chunks_to_sources(&chunks);
    let grounded = !sources.is_empty();

    // 4. Build prompt.
    let system = llm::build_system_prompt(
        garden_ctx.as_deref(),
        zone_ctx.as_deref(),
        tasks_ctx.as_deref(),
        &chunks,
    );

    // 5. Run inference.
    let (model_arc, kind) = {
        let guard = lock_llm(&state)?;
        match guard.as_ref() {
            Some(lm) => (lm.model.clone(), lm.kind),
            None => {
                return Err(AppError::InvalidInput(
                    "no model loaded - import and load one in the Models tab".into(),
                ))
            }
        }
    };

    let generated = if kind == ModelKind::Vision {
        if let Some(fname) = &attachment_filename {
            let path = storage::photos_dir(&state.app_data).join(fname);
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    llm::generate_with_image(&*model_arc, &system, &content, img, llm::MAX_ANSWER_TOKENS).await?
                } else {
                    llm::generate_text(&*model_arc, &system, &history, &content, llm::MAX_ANSWER_TOKENS).await?
                }
            } else {
                llm::generate_text(&*model_arc, &system, &history, &content, llm::MAX_ANSWER_TOKENS).await?
            }
        } else {
            llm::generate_text(&*model_arc, &system, &history, &content, llm::MAX_ANSWER_TOKENS).await?
        }
    } else {
        llm::generate_text(&*model_arc, &system, &history, &content, llm::MAX_ANSWER_TOKENS).await?
    };

    // 6. Persist assistant message.
    let assistant_msg = {
        let conn = lock_db(&state)?;
        db::chat::insert_assistant_message(
            &*conn,
            &state.app_data,
            &conv_id,
            &generated.content,
            &sources,
            grounded,
        )?
    };
    // touch used model
    if let Some(id) = {
        let g = lock_llm(&state)?;
        g.as_ref().map(|m| m.model_id.clone())
    } {
        let conn = lock_db(&state)?;
        let _ = db::models::touch_used(&*conn, &id);
    }

    Ok(ChatResponse {
        message: assistant_msg,
        sources,
        grounded,
        prompt_tok_per_sec: generated.prompt_tok_per_sec,
        compl_tok_per_sec: generated.compl_tok_per_sec,
    })
}

