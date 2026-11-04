use rusqlite::{params, Connection};

#[cfg(test)]
use crate::db;
use crate::error::{AppError, AppResult};
use garden_core::models::Planting;

#[cfg(test)]
fn seeded() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    db::seed::seed(&mut conn).unwrap();
    let g = db::gardens::create_garden(&conn, "Backyard", "raised_bed").unwrap();
    db::gardens::create_bed(&conn, &g.id, "Bed 1", None, None, None, None, false).unwrap();
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_list_planting() {
        let conn = seeded();
        let bed = db::gardens::list_beds(&conn, &db::gardens::list_gardens(&conn).unwrap()[0].id)
            .unwrap()
            .remove(0);
        let p = add_planting(&conn, &bed.id, "plant_tomato", 3, "seed", Some("notes")).unwrap();
        assert_eq!(p.plant_name, "Tomato");
        assert_eq!(p.quantity, 3);
        assert_eq!(list_plantings(&conn).unwrap().len(), 1);
    }

    #[test]
    fn update_status_and_reason() {
        let conn = seeded();
        let bed = db::gardens::list_beds(&conn, &db::gardens::list_gardens(&conn).unwrap()[0].id)
            .unwrap()
            .remove(0);
        let p = add_planting(&conn, &bed.id, "plant_basil", 1, "seed", None).unwrap();
        let updated = update_status(&conn, &p.id, "removed", Some("slug damage")).unwrap();
        assert_eq!(updated.status, "removed");
        assert_eq!(updated.removal_reason.as_deref(), Some("slug damage"));
        assert!(updated.removed_at.is_some());
    }
}

const PLANTING_SELECT: &str = "SELECT p.id, p.bed_id, b.name, p.catalog_id, c.name, p.quantity,
    p.planted_at, p.source, p.notes, p.status, p.removed_at, p.removal_reason
    FROM planted p
    JOIN beds b ON b.id = p.bed_id
    JOIN catalog c ON c.id = p.catalog_id";

fn row_to_planting(row: &rusqlite::Row) -> rusqlite::Result<Planting> {
    Ok(Planting {
        id: row.get(0)?,
        bed_id: row.get(1)?,
        bed_name: row.get(2)?,
        catalog_id: row.get(3)?,
        plant_name: row.get(4)?,
        quantity: row.get(5)?,
        planted_at: row.get(6)?,
        source: row.get(7)?,
        notes: row.get(8)?,
        status: row.get(9)?,
        removed_at: row.get(10)?,
        removal_reason: row.get(11)?,
    })
}

pub fn add_planting(
    conn: &Connection,
    bed_id: &str,
    catalog_id: &str,
    quantity: i64,
    source: &str,
    notes: Option<&str>,
) -> AppResult<Planting> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO planted (id, bed_id, catalog_id, quantity, source, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, bed_id, catalog_id, quantity, source, notes],
    )?;
    get_planting(conn, &id)
}

pub fn get_planting(conn: &Connection, id: &str) -> AppResult<Planting> {
    conn.query_row(
        &format!("{} WHERE p.id = ?1", PLANTING_SELECT),
        params![id],
        row_to_planting,
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn list_plantings(conn: &Connection) -> AppResult<Vec<Planting>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY p.planted_at DESC", PLANTING_SELECT))?;
    let rows = stmt.query_map([], row_to_planting)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn update_status(
    conn: &Connection,
    id: &str,
    status: &str,
    removal_reason: Option<&str>,
) -> AppResult<Planting> {
    let removed_at = if status == "removed" || status == "dead" {
        Some(chrono::Utc::now().to_rfc3339())
    } else {
        None
    };
    conn.execute(
        "UPDATE planted SET status = ?1, removal_reason = ?2, removed_at = COALESCE(?3, removed_at),
         updated_at = datetime('now') WHERE id = ?4",
        params![status, removal_reason, removed_at, id],
    )?;
    get_planting(conn, id)
}

pub fn remove_planting(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM planted WHERE id = ?1", params![id])?;
    Ok(())
}