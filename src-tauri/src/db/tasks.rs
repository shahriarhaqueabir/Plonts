use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::{Task, TaskStatus};

fn enum_str(v: &TaskStatus) -> String {
    serde_json::to_string(v).unwrap().trim_matches('"').to_string()
}

fn parse_status(s: &str) -> TaskStatus {
    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or_default()
}

const TASK_SELECT: &str = "SELECT id, title, body, status, due_at, created_at, updated_at FROM tasks";

fn row_to_task(row: &rusqlite::Row) -> rusqlite::Result<Task> {
    let status: String = row.get(3)?;
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        status: parse_status(&status),
        due_at: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

pub fn list_tasks(conn: &Connection) -> AppResult<Vec<Task>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY created_at DESC, rowid DESC", TASK_SELECT))?;
    let rows = stmt.query_map([], row_to_task)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_task(conn: &Connection, id: &str) -> AppResult<Task> {
    conn.query_row(&format!("{} WHERE id = ?1", TASK_SELECT), params![id], row_to_task)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn create_task(
    conn: &Connection,
    title: &str,
    body: Option<&str>,
    due_at: Option<&str>,
) -> AppResult<Task> {
    if title.trim().is_empty() {
        return Err(AppError::InvalidInput("title must not be empty".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tasks (id, title, body, due_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, title, body, due_at],
    )?;
    get_task(conn, &id)
}

pub fn set_task_status(conn: &Connection, id: &str, status: TaskStatus) -> AppResult<Task> {
    let n = conn.execute(
        "UPDATE tasks SET status = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![id, enum_str(&status)],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    get_task(conn, id)
}

pub fn delete_task(conn: &Connection, id: &str) -> AppResult<()> {
    let n = conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
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
    fn create_and_list_tasks() {
        let conn = seeded();
        let t = create_task(&conn, "Water tomatoes", Some("daily"), Some("2026-07-01")).unwrap();
        assert_eq!(t.title, "Water tomatoes");
        assert_eq!(t.body.as_deref(), Some("daily"));
        assert_eq!(t.due_at.as_deref(), Some("2026-07-01"));
        assert_eq!(t.status, TaskStatus::Open);
        assert_eq!(list_tasks(&conn).unwrap().len(), 1);
    }

    #[test]
    fn create_rejects_empty_title() {
        let conn = seeded();
        assert!(create_task(&conn, "  ", None, None).is_err());
        assert!(create_task(&conn, "", None, None).is_err());
    }

    #[test]
    fn get_task_returns_row() {
        let conn = seeded();
        let t = create_task(&conn, "Prune", None, None).unwrap();
        let g = get_task(&conn, &t.id).unwrap();
        assert_eq!(g, t);
    }

    #[test]
    fn get_missing_returns_not_found() {
        let conn = seeded();
        assert!(get_task(&conn, "nope").is_err());
    }

    #[test]
    fn set_task_status_updates() {
        let conn = seeded();
        let t = create_task(&conn, "Weed", None, None).unwrap();
        let done = set_task_status(&conn, &t.id, TaskStatus::Done).unwrap();
        assert_eq!(done.status, TaskStatus::Done);
        assert!(done.updated_at >= t.updated_at);
        let reopened = set_task_status(&conn, &t.id, TaskStatus::Open).unwrap();
        assert_eq!(reopened.status, TaskStatus::Open);
    }

    #[test]
    fn set_status_missing_returns_not_found() {
        let conn = seeded();
        assert!(set_task_status(&conn, "nope", TaskStatus::Done).is_err());
    }

    #[test]
    fn delete_task_removes_row() {
        let conn = seeded();
        let t = create_task(&conn, "X", None, None).unwrap();
        delete_task(&conn, &t.id).unwrap();
        assert!(list_tasks(&conn).unwrap().is_empty());
        assert!(delete_task(&conn, &t.id).is_err());
    }

    #[test]
    fn list_orders_by_created_desc() {
        let conn = seeded();
        let a = create_task(&conn, "A", None, None).unwrap();
        let b = create_task(&conn, "B", None, None).unwrap();
        let list = list_tasks(&conn).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id, b.id);
        assert_eq!(list[1].id, a.id);
    }
}
