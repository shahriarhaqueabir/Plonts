use rusqlite::{params, Connection};

#[cfg(test)]
use crate::db;
use crate::error::{AppError, AppResult};
use garden_core::models::{ShoppingItem, Tool};

#[cfg(test)]
fn seeded() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    db::seed::seed(&mut conn).unwrap();
    conn
}

fn today_iso() -> String {
    chrono::Utc::now().date_naive().to_string()
}

// ---------- Shopping items ----------

const SHOPPING_SELECT: &str = "SELECT id, name, category, quantity, checked FROM shopping_items";

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<ShoppingItem> {
    Ok(ShoppingItem {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        quantity: row.get(3)?,
        checked: row.get::<_, i64>(4)? != 0,
    })
}

pub fn list_shopping_items(conn: &Connection) -> AppResult<Vec<ShoppingItem>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY checked ASC, created_at ASC", SHOPPING_SELECT))?;
    let rows = stmt.query_map([], row_to_item)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_shopping_item(conn: &Connection, id: &str) -> AppResult<ShoppingItem> {
    conn.query_row(&format!("{} WHERE id = ?1", SHOPPING_SELECT), params![id], row_to_item)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn add_shopping_item(
    conn: &Connection,
    name: &str,
    category: &str,
    quantity: Option<&str>,
) -> AppResult<ShoppingItem> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput("item name is required".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO shopping_items (id, name, category, quantity) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, category, quantity],
    )?;
    get_shopping_item(conn, &id)
}

pub fn toggle_shopping_item(conn: &Connection, id: &str) -> AppResult<ShoppingItem> {
    conn.execute("UPDATE shopping_items SET checked = 1 - checked WHERE id = ?1", params![id])?;
    get_shopping_item(conn, id)
}

pub fn delete_shopping_item(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM shopping_items WHERE id = ?1", params![id])?;
    Ok(())
}

// ---------- Tools ----------

const TOOL_SELECT: &str = "SELECT id, name, last_maintained, next_maintenance, notes FROM tools";

fn row_to_tool(row: &rusqlite::Row) -> rusqlite::Result<Tool> {
    let next_maintenance: Option<String> = row.get(3)?;
    let maintenance_due = garden_core::inventory::is_due(next_maintenance.as_deref(), &today_iso());
    Ok(Tool {
        id: row.get(0)?,
        name: row.get(1)?,
        last_maintained: row.get(2)?,
        next_maintenance,
        notes: row.get(4)?,
        maintenance_due,
    })
}

pub fn list_tools(conn: &Connection) -> AppResult<Vec<Tool>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY name", TOOL_SELECT))?;
    let rows = stmt.query_map([], row_to_tool)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_tool(conn: &Connection, id: &str) -> AppResult<Tool> {
    conn.query_row(&format!("{} WHERE id = ?1", TOOL_SELECT), params![id], row_to_tool)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn add_tool(
    conn: &Connection,
    name: &str,
    next_maintenance: Option<&str>,
    notes: Option<&str>,
) -> AppResult<Tool> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput("tool name is required".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tools (id, name, next_maintenance, notes) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, next_maintenance, notes],
    )?;
    get_tool(conn, &id)
}

pub fn mark_tool_maintained(conn: &Connection, id: &str, next_maintenance: Option<&str>) -> AppResult<Tool> {
    let today = today_iso();
    conn.execute(
        "UPDATE tools SET last_maintained = ?2, next_maintenance = ?3 WHERE id = ?1",
        params![id, today, next_maintenance],
    )?;
    get_tool(conn, id)
}

pub fn delete_tool(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM tools WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shopping_add_list_toggle_delete() {
        let conn = seeded();
        let item = add_shopping_item(&conn, "Tomato seeds", "seeds", Some("2 packets")).unwrap();
        assert!(!item.checked);
        assert_eq!(item.category, "seeds");
        assert_eq!(list_shopping_items(&conn).unwrap().len(), 1);
        let toggled = toggle_shopping_item(&conn, &item.id).unwrap();
        assert!(toggled.checked);
        delete_shopping_item(&conn, &item.id).unwrap();
        assert!(list_shopping_items(&conn).unwrap().is_empty());
    }

    #[test]
    fn shopping_empty_name_rejected() {
        let conn = seeded();
        assert!(add_shopping_item(&conn, "  ", "tools", None).is_err());
    }

    #[test]
    fn tool_add_due_and_maintain() {
        let conn = seeded();
        let tool = add_tool(&conn, "Secateurs", Some("2000-01-01"), None).unwrap();
        assert!(tool.maintenance_due);
        let future = add_tool(&conn, "Trowel", Some("2999-01-01"), None).unwrap();
        assert!(!future.maintenance_due);
        let maintained = mark_tool_maintained(&conn, &tool.id, Some("2999-12-31")).unwrap();
        assert!(!maintained.maintenance_due);
        assert!(maintained.last_maintained.is_some());
    }

    #[test]
    fn tool_empty_name_rejected() {
        let conn = seeded();
        assert!(add_tool(&conn, "", None, None).is_err());
    }

    #[test]
    fn tool_delete_removes() {
        let conn = seeded();
        let t = add_tool(&conn, "Hoe", None, None).unwrap();
        delete_tool(&conn, &t.id).unwrap();
        assert!(list_tools(&conn).unwrap().is_empty());
    }
}