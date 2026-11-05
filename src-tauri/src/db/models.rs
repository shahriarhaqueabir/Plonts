use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::{ModelInfo, ModelKind};

fn enum_str(v: &ModelKind) -> String {
    serde_json::to_string(v).unwrap().trim_matches('"').to_string()
}

fn parse_kind(s: &str) -> ModelKind {
    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or_default()
}

const MODEL_SELECT: &str = "SELECT id, name, path, size_bytes, kind, created_at, last_used_at FROM models";

fn row_to_model(row: &rusqlite::Row) -> rusqlite::Result<ModelInfo> {
    let kind: String = row.get(4)?;
    Ok(ModelInfo {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        size_bytes: row.get(3)?,
        kind: parse_kind(&kind),
        created_at: row.get(5)?,
        last_used_at: row.get(6)?,
    })
}

pub fn list_models(conn: &Connection) -> AppResult<Vec<ModelInfo>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY created_at DESC, rowid DESC", MODEL_SELECT))?;
    let rows = stmt.query_map([], row_to_model)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_model(conn: &Connection, id: &str) -> AppResult<ModelInfo> {
    conn.query_row(&format!("{} WHERE id = ?1", MODEL_SELECT), params![id], row_to_model)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn create_model(
    conn: &Connection,
    name: &str,
    path: &str,
    size_bytes: i64,
    kind: ModelKind,
) -> AppResult<ModelInfo> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO models (id, name, path, size_bytes, kind) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, path, size_bytes, enum_str(&kind)],
    )?;
    get_model(conn, &id)
}

pub fn delete_model(conn: &Connection, id: &str) -> AppResult<ModelInfo> {
    let info = get_model(conn, id)?;
    conn.execute("DELETE FROM models WHERE id = ?1", params![id])?;
    Ok(info)
}

pub fn touch_used(conn: &Connection, id: &str) -> AppResult<()> {
    let n = conn.execute(
        "UPDATE models SET last_used_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrate(&mut conn).unwrap();
        conn
    }

    #[test]
    fn create_and_list_models() {
        let conn = seeded();
        let m = create_model(&conn, "Qwen", "/tmp/q.gguf", 1024, ModelKind::Text).unwrap();
        assert_eq!(m.name, "Qwen");
        assert_eq!(m.path, "/tmp/q.gguf");
        assert_eq!(m.size_bytes, 1024);
        assert_eq!(m.kind, ModelKind::Text);
        assert!(m.last_used_at.is_none());
        assert_eq!(list_models(&conn).unwrap().len(), 1);
    }

    #[test]
    fn get_model_returns_row() {
        let conn = seeded();
        let m = create_model(&conn, "Vision", "/v.gguf", 2048, ModelKind::Vision).unwrap();
        let g = get_model(&conn, &m.id).unwrap();
        assert_eq!(g, m);
    }

    #[test]
    fn get_missing_returns_not_found() {
        let conn = seeded();
        assert!(get_model(&conn, "nope").is_err());
    }

    #[test]
    fn delete_model_returns_deleted() {
        let conn = seeded();
        let m = create_model(&conn, "X", "/x.gguf", 10, ModelKind::Text).unwrap();
        let deleted = delete_model(&conn, &m.id).unwrap();
        assert_eq!(deleted, m);
        assert!(list_models(&conn).unwrap().is_empty());
        assert!(delete_model(&conn, &m.id).is_err());
    }

    #[test]
    fn touch_used_sets_last_used() {
        let conn = seeded();
        let m = create_model(&conn, "X", "/x.gguf", 10, ModelKind::Text).unwrap();
        assert!(m.last_used_at.is_none());
        touch_used(&conn, &m.id).unwrap();
        let g = get_model(&conn, &m.id).unwrap();
        assert!(g.last_used_at.is_some());
    }

    #[test]
    fn touch_missing_returns_not_found() {
        let conn = seeded();
        assert!(touch_used(&conn, "nope").is_err());
    }

    #[test]
    fn list_orders_by_created_desc() {
        let conn = seeded();
        let a = create_model(&conn, "A", "/a.gguf", 1, ModelKind::Text).unwrap();
        let b = create_model(&conn, "B", "/b.gguf", 2, ModelKind::Text).unwrap();
        let list = list_models(&conn).unwrap();
        assert_eq!(list.len(), 2);
        // most recently created first
        assert_eq!(list[0].id, b.id);
        assert_eq!(list[1].id, a.id);
    }
}
