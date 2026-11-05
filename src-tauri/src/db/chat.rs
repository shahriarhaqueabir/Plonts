use std::path::Path;

use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::storage;
use garden_core::llm::{ChatAttachment, ChatConversation, ChatMessage, ChatSource, ContextFlags, MessageRole};

fn json<T: serde::Serialize>(v: &T) -> AppResult<String> {
    Ok(serde_json::to_string(v)?)
}

fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> T {
    serde_json::from_str(s).expect("valid json field")
}

fn role_str(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
    }
}

fn parse_role(s: &str) -> MessageRole {
    match s {
        "assistant" => MessageRole::Assistant,
        _ => MessageRole::User,
    }
}

const CONV_SELECT: &str = "SELECT id, title, context_flags_json, created_at, updated_at FROM chat_conversations";

fn conv_from_row(row: &rusqlite::Row) -> rusqlite::Result<ChatConversation> {
    let flags: String = row.get(2)?;
    Ok(ChatConversation {
        id: row.get(0)?,
        title: row.get(1)?,
        context_flags: parse_json(&flags),
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

const MESSAGE_SELECT: &str =
    "SELECT id, conversation_id, role, content, sources_json, grounded, flagged, flag_note, created_at FROM chat_messages";

fn message_from_row(
    conn: &Connection,
    photos_base: &Path,
    row: &rusqlite::Row,
) -> rusqlite::Result<ChatMessage> {
    let id: String = row.get(0)?;
    let role: String = row.get(2)?;
    let sources_json: String = row.get(4)?;
    let grounded: i64 = row.get(5)?;
    let flagged: i64 = row.get(6)?;
    let attachments = list_attachments(conn, photos_base, &id).unwrap_or_default();
    Ok(ChatMessage {
        id: id.clone(),
        conversation_id: row.get(1)?,
        role: parse_role(&role),
        content: row.get(3)?,
        sources: parse_json(&sources_json),
        grounded: grounded != 0,
        flagged: flagged != 0,
        flag_note: row.get(7)?,
        attachments,
        created_at: row.get(8)?,
    })
}

pub fn get_conversation(conn: &Connection, id: &str) -> AppResult<ChatConversation> {
    conn.query_row(&format!("{} WHERE id = ?1", CONV_SELECT), params![id], conv_from_row)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

fn get_message(conn: &Connection, photos_base: &Path, id: &str) -> AppResult<ChatMessage> {
    conn.query_row(
        &format!("{} WHERE id = ?1", MESSAGE_SELECT),
        params![id],
        |r| message_from_row(conn, photos_base, r),
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

fn get_attachment(conn: &Connection, photos_base: &Path, id: &str) -> AppResult<ChatAttachment> {
    let (message_id, path, mime, created_at): (String, String, Option<String>, String) = conn
        .query_row(
            "SELECT message_id, path, mime, created_at FROM chat_attachments WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|_| AppError::NotFound(id.to_string()))?;
    Ok(ChatAttachment {
        id: id.to_string(),
        message_id,
        url: storage::asset_url(photos_base, &path),
        path,
        mime,
        created_at,
    })
}

pub fn list_conversations(conn: &Connection) -> AppResult<Vec<ChatConversation>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY updated_at DESC, rowid DESC", CONV_SELECT))?;
    let rows = stmt.query_map([], conv_from_row)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_conversation(
    conn: &Connection,
    title: &str,
    flags: &ContextFlags,
) -> AppResult<ChatConversation> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO chat_conversations (id, title, context_flags_json) VALUES (?1, ?2, ?3)",
        params![id, title, json(flags)?],
    )?;
    get_conversation(conn, &id)
}

pub fn update_conversation_flags(
    conn: &Connection,
    id: &str,
    flags: &ContextFlags,
) -> AppResult<ChatConversation> {
    let n = conn.execute(
        "UPDATE chat_conversations SET context_flags_json = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![id, json(flags)?],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    get_conversation(conn, id)
}

pub fn rename_conversation(conn: &Connection, id: &str, title: &str) -> AppResult<ChatConversation> {
    let n = conn.execute(
        "UPDATE chat_conversations SET title = ?2 WHERE id = ?1",
        params![id, title],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    get_conversation(conn, id)
}

pub fn delete_conversation(conn: &Connection, id: &str) -> AppResult<Vec<String>> {
    // collect attachment filenames before cascade delete
    let mut stmt = conn.prepare(
        "SELECT a.path FROM chat_attachments a JOIN chat_messages m ON m.id = a.message_id WHERE m.conversation_id = ?1",
    )?;
    let names: Vec<String> = stmt
        .query_map(params![id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    let n = conn.execute("DELETE FROM chat_conversations WHERE id = ?1", params![id])?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    Ok(names)
}

pub fn clear_conversation(conn: &Connection, id: &str) -> AppResult<Vec<String>> {
    // ensure conversation exists
    let _ = get_conversation(conn, id)?;
    let mut stmt = conn.prepare(
        "SELECT a.path FROM chat_attachments a JOIN chat_messages m ON m.id = a.message_id WHERE m.conversation_id = ?1",
    )?;
    let names: Vec<String> = stmt
        .query_map(params![id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    conn.execute("DELETE FROM chat_messages WHERE conversation_id = ?1", params![id])?;
    Ok(names)
}

pub fn clear_all_conversations(conn: &Connection) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM chat_attachments")?;
    let names: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    conn.execute("DELETE FROM chat_conversations", [])?;
    Ok(names)
}

pub fn list_messages(
    conn: &Connection,
    photos_base: &Path,
    conversation_id: &str,
) -> AppResult<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(&format!(
        "{} WHERE conversation_id = ?1 ORDER BY created_at, rowid",
        MESSAGE_SELECT
    ))?;
    let rows = stmt.query_map(params![conversation_id], |r| {
        message_from_row(conn, photos_base, r)
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn insert_message(
    conn: &Connection,
    photos_base: &Path,
    conversation_id: &str,
    role: MessageRole,
    content: &str,
) -> AppResult<ChatMessage> {
    // ensure conversation exists
    let _ = get_conversation(conn, conversation_id)?;
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO chat_messages (id, conversation_id, role, content) VALUES (?1, ?2, ?3, ?4)",
        params![id, conversation_id, role_str(role), content],
    )?;
    get_message(conn, photos_base, &id)
}

pub fn insert_assistant_message(
    conn: &Connection,
    photos_base: &Path,
    conversation_id: &str,
    content: &str,
    sources: &[ChatSource],
    grounded: bool,
) -> AppResult<ChatMessage> {
    let _ = get_conversation(conn, conversation_id)?;
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO chat_messages (id, conversation_id, role, content, sources_json, grounded) VALUES (?1, ?2, 'assistant', ?3, ?4, ?5)",
        params![id, conversation_id, content, serde_json::to_string(sources)?, grounded as i64],
    )?;
    get_message(conn, photos_base, &id)
}

pub fn flag_message(
    conn: &Connection,
    photos_base: &Path,
    id: &str,
    note: Option<&str>,
) -> AppResult<ChatMessage> {
    let n = conn.execute(
        "UPDATE chat_messages SET flagged = 1, flag_note = ?2 WHERE id = ?1",
        params![id, note],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    get_message(conn, photos_base, id)
}

pub fn insert_attachment(
    conn: &Connection,
    photos_base: &Path,
    message_id: &str,
    filename: &str,
    mime: Option<&str>,
) -> AppResult<ChatAttachment> {
    // ensure message exists
    let _ = get_message(conn, photos_base, message_id)?;
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO chat_attachments (id, message_id, path, mime) VALUES (?1, ?2, ?3, ?4)",
        params![id, message_id, filename, mime],
    )?;
    get_attachment(conn, photos_base, &id)
}

pub fn list_attachments(
    conn: &Connection,
    photos_base: &Path,
    message_id: &str,
) -> AppResult<Vec<ChatAttachment>> {
    let mut stmt = conn.prepare(
        "SELECT id, message_id, path, mime, created_at FROM chat_attachments WHERE message_id = ?1 ORDER BY created_at, rowid",
    )?;
    let rows = stmt.query_map(params![message_id], |r| {
        let id: String = r.get(0)?;
        let path: String = r.get(2)?;
        Ok(ChatAttachment {
            id,
            message_id: r.get(1)?,
            path: path.clone(),
            mime: r.get(3)?,
            url: storage::asset_url(photos_base, &path),
            created_at: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::path::PathBuf;

    fn seeded() -> (Connection, PathBuf) {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrate(&mut conn).unwrap();
        let base = std::env::temp_dir().join(format!("lovelygarden-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();
        (conn, base)
    }

    #[test]
    fn create_and_list_conversations() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "Chat 1", &ContextFlags::default()).unwrap();
        assert_eq!(c.title, "Chat 1");
        assert!(c.context_flags.garden && c.context_flags.zone && c.context_flags.tasks);
        assert_eq!(list_conversations(&conn).unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn get_conversation_returns_row() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let g = get_conversation(&conn, &c.id).unwrap();
        assert_eq!(g, c);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn get_missing_returns_not_found() {
        let (conn, base) = seeded();
        assert!(get_conversation(&conn, "nope").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn update_flags_bumps_and_persists() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let flags = ContextFlags { garden: false, zone: true, tasks: false };
        let u = update_conversation_flags(&conn, &c.id, &flags).unwrap();
        assert_eq!(u.context_flags, flags);
        assert!(u.updated_at >= c.updated_at);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn update_flags_missing_returns_not_found() {
        let (conn, base) = seeded();
        assert!(update_conversation_flags(&conn, "nope", &ContextFlags::default()).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn rename_conversation_changes_title() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "Old", &ContextFlags::default()).unwrap();
        let r = rename_conversation(&conn, &c.id, "New").unwrap();
        assert_eq!(r.title, "New");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn rename_missing_returns_not_found() {
        let (conn, base) = seeded();
        assert!(rename_conversation(&conn, "nope", "x").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn insert_and_list_messages() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let m1 = insert_message(&conn, &base, &c.id, MessageRole::User, "hi").unwrap();
        assert_eq!(m1.role, MessageRole::User);
        assert_eq!(m1.content, "hi");
        assert!(m1.attachments.is_empty());
        let m2 = insert_assistant_message(&conn, &base, &c.id, "hello", &[], false).unwrap();
        assert_eq!(m2.role, MessageRole::Assistant);
        assert!(!m2.grounded);
        let msgs = list_messages(&conn, &base, &c.id).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].id, m1.id);
        assert_eq!(msgs[1].id, m2.id);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn insert_message_missing_conv_errors() {
        let (conn, base) = seeded();
        assert!(insert_message(&conn, &base, "nope", MessageRole::User, "hi").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn assistant_message_stores_sources_and_grounded() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let src = ChatSource { id: "p1".into(), title: "Tomato".into(), kind: "plant".into(), url: None };
        let m = insert_assistant_message(&conn, &base, &c.id, "answer", &[src.clone()], true).unwrap();
        assert!(m.grounded);
        assert_eq!(m.sources, vec![src]);
        let msgs = list_messages(&conn, &base, &c.id).unwrap();
        assert_eq!(msgs[0].sources.len(), 1);
        assert!(msgs[0].grounded);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn flag_message_sets_flagged_and_note() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let m = insert_message(&conn, &base, &c.id, MessageRole::Assistant, "bad").unwrap();
        assert!(!m.flagged);
        let f = flag_message(&conn, &base, &m.id, Some("wrong")).unwrap();
        assert!(f.flagged);
        assert_eq!(f.flag_note.as_deref(), Some("wrong"));
        let msgs = list_messages(&conn, &base, &c.id).unwrap();
        assert!(msgs[0].flagged);
        assert_eq!(msgs[0].flag_note.as_deref(), Some("wrong"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn flag_message_missing_returns_not_found() {
        let (conn, base) = seeded();
        assert!(flag_message(&conn, &base, "nope", None).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn attachment_insert_and_list() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let m = insert_message(&conn, &base, &c.id, MessageRole::User, "photo").unwrap();
        let a = insert_attachment(&conn, &base, &m.id, "abc.jpg", Some("image/jpeg")).unwrap();
        assert_eq!(a.path, "abc.jpg");
        assert_eq!(a.mime.as_deref(), Some("image/jpeg"));
        assert!(a.url.starts_with("asset://localhost/"));
        let atts = list_attachments(&conn, &base, &m.id).unwrap();
        assert_eq!(atts.len(), 1);
        // message resolves attachments
        let msgs = list_messages(&conn, &base, &c.id).unwrap();
        assert_eq!(msgs[0].attachments.len(), 1);
        assert_eq!(msgs[0].attachments[0].id, a.id);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn attachment_missing_message_errors() {
        let (conn, base) = seeded();
        assert!(insert_attachment(&conn, &base, "nope", "x.jpg", None).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn delete_conversation_returns_attachment_filenames() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let m = insert_message(&conn, &base, &c.id, MessageRole::User, "photo").unwrap();
        insert_attachment(&conn, &base, &m.id, "a.jpg", None).unwrap();
        insert_attachment(&conn, &base, &m.id, "b.jpg", None).unwrap();
        let names = delete_conversation(&conn, &c.id).unwrap();
        assert_eq!(names.len(), 2);
        assert!(list_conversations(&conn).unwrap().is_empty());
        assert!(list_messages(&conn, &base, &c.id).is_err() || list_messages(&conn, &base, &c.id).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn delete_conversation_missing_returns_not_found() {
        let (conn, base) = seeded();
        assert!(delete_conversation(&conn, "nope").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn clear_conversation_keeps_conv_and_returns_names() {
        let (conn, base) = seeded();
        let c = create_conversation(&conn, "T", &ContextFlags::default()).unwrap();
        let m = insert_message(&conn, &base, &c.id, MessageRole::User, "photo").unwrap();
        insert_attachment(&conn, &base, &m.id, "a.jpg", None).unwrap();
        let names = clear_conversation(&conn, &c.id).unwrap();
        assert_eq!(names, vec!["a.jpg".to_string()]);
        assert_eq!(list_conversations(&conn).unwrap().len(), 1);
        assert!(list_messages(&conn, &base, &c.id).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn clear_conversation_missing_returns_not_found() {
        let (conn, base) = seeded();
        assert!(clear_conversation(&conn, "nope").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn clear_all_conversations_returns_all_names() {
        let (conn, base) = seeded();
        let c1 = create_conversation(&conn, "A", &ContextFlags::default()).unwrap();
        let c2 = create_conversation(&conn, "B", &ContextFlags::default()).unwrap();
        let m1 = insert_message(&conn, &base, &c1.id, MessageRole::User, "x").unwrap();
        insert_attachment(&conn, &base, &m1.id, "x.jpg", None).unwrap();
        let m2 = insert_message(&conn, &base, &c2.id, MessageRole::User, "y").unwrap();
        insert_attachment(&conn, &base, &m2.id, "y.jpg", None).unwrap();
        let names = clear_all_conversations(&conn).unwrap();
        assert_eq!(names.len(), 2);
        assert!(list_conversations(&conn).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn list_conversations_orders_by_updated_desc() {
        let (conn, base) = seeded();
        let a = create_conversation(&conn, "A", &ContextFlags::default()).unwrap();
        let b = create_conversation(&conn, "B", &ContextFlags::default()).unwrap();
        let list = list_conversations(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, b.id);
        assert_eq!(list[1].id, a.id);
        let _ = std::fs::remove_dir_all(&base);
    }
}
