use js_sys::Promise;
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke")]
    fn invoke_raw(cmd: &str, args: JsValue) -> Promise;
}

/// Invoke a Tauri command with serialized args, returning a typed result.
pub async fn call<T: DeserializeOwned, A: Serialize>(cmd: &str, args: &A) -> Result<T, String> {
    let args_js = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let promise = invoke_raw(cmd, args_js);
    let js = JsFuture::from(promise)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "invoke failed".to_string()))?;
    serde_wasm_bindgen::from_value(js).map_err(|e| e.to_string())
}

/// Invoke a Tauri command that takes no arguments.
pub async fn call0<T: DeserializeOwned>(cmd: &str) -> Result<T, String> {
    let promise = invoke_raw(cmd, JsValue::UNDEFINED);
    let js = JsFuture::from(promise)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "invoke failed".to_string()))?;
    serde_wasm_bindgen::from_value(js).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct DialogFilter {
    name: String,
    extensions: Vec<String>,
}

#[derive(serde::Serialize)]
struct OpenDialogArgs {
    multiple: bool,
    directory: bool,
    filters: Vec<DialogFilter>,
}

#[derive(serde::Serialize)]
struct SaveDialogArgs {
    filters: Vec<DialogFilter>,
}

fn image_filters() -> Vec<DialogFilter> {
    vec![DialogFilter {
        name: "Images".to_string(),
        extensions: vec!["png", "jpg", "jpeg", "webp", "gif"]
            .into_iter()
            .map(String::from)
            .collect(),
    }]
}

/// Open a native file picker via tauri-plugin-dialog; returns the selected path, or None.
pub async fn pick_file() -> Result<Option<String>, String> {
    let args = OpenDialogArgs {
        multiple: false,
        directory: false,
        filters: image_filters(),
    };
    call::<Option<String>, _>("plugin:dialog|open", &args).await
}

/// Open a native save dialog via tauri-plugin-dialog; returns the chosen path, or None.
pub async fn pick_save_path() -> Result<Option<String>, String> {
    let args = SaveDialogArgs { filters: image_filters() };
    call::<Option<String>, _>("plugin:dialog|save", &args).await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PlantImageArgs {
    catalog_id: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SetOverrideArgs {
    catalog_id: String,
    source_path: String,
}

pub async fn get_plant_image(catalog_id: &str) -> Result<garden_core::models::PlantImage, String> {
    call("get_plant_image", &PlantImageArgs { catalog_id: catalog_id.to_string() }).await
}

pub async fn get_generic_image() -> Result<String, String> {
    call0("get_generic_image").await
}

pub async fn set_plant_image_override(catalog_id: &str, source_path: &str) -> Result<garden_core::models::PlantImage, String> {
    call(
        "set_plant_image_override",
        &SetOverrideArgs { catalog_id: catalog_id.to_string(), source_path: source_path.to_string() },
    )
    .await
}

pub async fn clear_plant_image_override(catalog_id: &str) -> Result<garden_core::models::PlantImage, String> {
    call("clear_plant_image_override", &PlantImageArgs { catalog_id: catalog_id.to_string() }).await
}

pub async fn list_credits() -> Result<Vec<garden_core::models::Credit>, String> {
    call0("list_credits").await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveExportArgs {
    path: String,
    data_base64: String,
}

/// Send a base64 PNG to the backend to be written to `path`.
pub async fn save_export(path: &str, data_base64: &str) -> Result<(), String> {
    call(
        "save_export",
        &SaveExportArgs { path: path.to_string(), data_base64: data_base64.to_string() },
    )
    .await
}

fn png_filters() -> Vec<DialogFilter> {
    vec![DialogFilter {
        name: "PNG image".to_string(),
        extensions: vec!["png".to_string()],
    }]
}

/// Open a native save dialog filtered to PNG; returns the chosen path, or None.
pub async fn pick_png_save_path() -> Result<Option<String>, String> {
    let args = SaveDialogArgs { filters: png_filters() };
    call::<Option<String>, _>("plugin:dialog|save", &args).await
}

fn gguf_filters() -> Vec<DialogFilter> {
    vec![DialogFilter {
        name: "GGUF model".to_string(),
        extensions: vec!["gguf".to_string()],
    }]
}

/// Open a native file picker filtered to GGUF; returns the selected path, or None.
pub async fn pick_gguf() -> Result<Option<String>, String> {
    let args = OpenDialogArgs {
        multiple: false,
        directory: false,
        filters: gguf_filters(),
    };
    call::<Option<String>, _>("plugin:dialog|open", &args).await
}

// ---- Coach / LLM wrappers ----

pub async fn list_models() -> Result<Vec<garden_core::llm::ModelInfo>, String> {
    call0("list_models").await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportModelArgs {
    source_path: String,
    name: String,
    kind: garden_core::llm::ModelKind,
}

pub async fn import_model(
    source_path: &str,
    name: &str,
    kind: garden_core::llm::ModelKind,
) -> Result<garden_core::llm::ModelInfo, String> {
    call(
        "import_model",
        &ImportModelArgs {
            source_path: source_path.to_string(),
            name: name.to_string(),
            kind,
        },
    )
    .await
}

#[derive(serde::Serialize)]
struct DeleteModelArgs {
    id: String,
}

pub async fn delete_model(id: &str) -> Result<(), String> {
    call("delete_model", &DeleteModelArgs { id: id.to_string() }).await
}

pub async fn load_model(id: &str) -> Result<garden_core::llm::ModelInfo, String> {
    call("load_model", &DeleteModelArgs { id: id.to_string() }).await
}

pub async fn unload_model() -> Result<(), String> {
    call0("unload_model").await
}

pub async fn get_model_status() -> Result<Option<garden_core::llm::ModelInfo>, String> {
    call0("get_model_status").await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateChatArgs {
    conversation_id: String,
    content: String,
    flags: garden_core::llm::ContextFlags,
    attachment_source_path: Option<String>,
    mime: Option<String>,
}

pub async fn generate_chat(
    conversation_id: &str,
    content: &str,
    flags: &garden_core::llm::ContextFlags,
    attachment_source_path: Option<&str>,
    mime: Option<&str>,
) -> Result<garden_core::llm::ChatResponse, String> {
    call(
        "generate_chat",
        &GenerateChatArgs {
            conversation_id: conversation_id.to_string(),
            content: content.to_string(),
            flags: flags.clone(),
            attachment_source_path: attachment_source_path.map(String::from),
            mime: mime.map(String::from),
        },
    )
    .await
}

pub async fn list_conversations() -> Result<Vec<garden_core::llm::ChatConversation>, String> {
    call0("list_conversations").await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateConversationArgs {
    title: String,
    flags: garden_core::llm::ContextFlags,
}

pub async fn create_conversation(
    title: &str,
    flags: &garden_core::llm::ContextFlags,
) -> Result<garden_core::llm::ChatConversation, String> {
    call(
        "create_conversation",
        &CreateConversationArgs {
            title: title.to_string(),
            flags: flags.clone(),
        },
    )
    .await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateConversationFlagsArgs {
    id: String,
    flags: garden_core::llm::ContextFlags,
}

pub async fn update_conversation_flags(
    id: &str,
    flags: &garden_core::llm::ContextFlags,
) -> Result<garden_core::llm::ChatConversation, String> {
    call(
        "update_conversation_flags",
        &UpdateConversationFlagsArgs {
            id: id.to_string(),
            flags: flags.clone(),
        },
    )
    .await
}

pub async fn delete_conversation(id: &str) -> Result<(), String> {
    call("delete_conversation", &DeleteModelArgs { id: id.to_string() }).await
}

pub async fn clear_conversation(id: &str) -> Result<(), String> {
    call("clear_conversation", &DeleteModelArgs { id: id.to_string() }).await
}

pub async fn clear_all_conversations() -> Result<(), String> {
    call0("clear_all_conversations").await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ListMessagesArgs {
    conversation_id: String,
}

pub async fn list_messages(
    conversation_id: &str,
) -> Result<Vec<garden_core::llm::ChatMessage>, String> {
    call(
        "list_messages",
        &ListMessagesArgs {
            conversation_id: conversation_id.to_string(),
        },
    )
    .await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FlagMessageArgs {
    message_id: String,
    note: Option<String>,
}

pub async fn flag_message(
    message_id: &str,
    note: Option<&str>,
) -> Result<garden_core::llm::ChatMessage, String> {
    call(
        "flag_message",
        &FlagMessageArgs {
            message_id: message_id.to_string(),
            note: note.map(String::from),
        },
    )
    .await
}

#[derive(serde::Serialize)]
struct TextArgs {
    text: String,
}

pub async fn create_tasks_from_text(text: &str) -> Result<Vec<garden_core::llm::Task>, String> {
    call("create_tasks_from_text", &TextArgs { text: text.to_string() }).await
}

pub async fn create_journal_from_text(text: &str) -> Result<garden_core::models::JournalEntry, String> {
    call("create_journal_from_text", &TextArgs { text: text.to_string() }).await
}

pub async fn create_layout_from_text(text: &str) -> Result<garden_core::models::Layout, String> {
    call("create_layout_from_text", &TextArgs { text: text.to_string() }).await
}

pub async fn list_tasks() -> Result<Vec<garden_core::llm::Task>, String> {
    call0("list_tasks").await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateTaskArgs {
    title: String,
    body: Option<String>,
    due_at: Option<String>,
}

pub async fn create_task(
    title: &str,
    body: Option<&str>,
    due_at: Option<&str>,
) -> Result<garden_core::llm::Task, String> {
    call(
        "create_task",
        &CreateTaskArgs {
            title: title.to_string(),
            body: body.map(String::from),
            due_at: due_at.map(String::from),
        },
    )
    .await
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SetTaskStatusArgs {
    id: String,
    status: garden_core::llm::TaskStatus,
}

pub async fn set_task_status(
    id: &str,
    status: garden_core::llm::TaskStatus,
) -> Result<garden_core::llm::Task, String> {
    call(
        "set_task_status",
        &SetTaskStatusArgs {
            id: id.to_string(),
            status,
        },
    )
    .await
}

pub async fn delete_task(id: &str) -> Result<(), String> {
    call("delete_task", &DeleteModelArgs { id: id.to_string() }).await
}