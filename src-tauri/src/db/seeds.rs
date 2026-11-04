use chrono::Datelike;
use rusqlite::{params, Connection};

#[cfg(test)]
use crate::db;
use crate::error::{AppError, AppResult};
use garden_core::models::Seed;

#[cfg(test)]
fn seeded() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    db::seed::seed(&mut conn).unwrap();
    conn
}

const SEED_SELECT: &str = "SELECT s.id, s.catalog_id, c.name AS plant_name, s.variety, s.source,
    s.year, s.quantity, s.germination_notes, s.viability_warned
    FROM seeds s
    LEFT JOIN catalog c ON c.id = s.catalog_id";

fn row_to_seed(row: &rusqlite::Row) -> rusqlite::Result<Seed> {
    let year: i64 = row.get(5)?;
    let current_year = chrono::Utc::now().year() as i64;
    Ok(Seed {
        id: row.get(0)?,
        catalog_id: row.get(1)?,
        plant_name: row.get(2)?,
        variety: row.get(3)?,
        source: row.get(4)?,
        year,
        quantity: row.get(6)?,
        germination_notes: row.get(7)?,
        viability_warned: row.get::<_, i64>(8)? != 0,
        viability: garden_core::inventory::viability_status(year, current_year),
    })
}

pub fn add_seed(
    conn: &Connection,
    catalog_id: Option<&str>,
    variety: &str,
    source: &str,
    year: i64,
    quantity: i64,
    germination_notes: Option<&str>,
) -> AppResult<Seed> {
    if variety.trim().is_empty() {
        return Err(AppError::InvalidInput("seed variety is required".into()));
    }
    if source.trim().is_empty() {
        return Err(AppError::InvalidInput("seed source is required".into()));
    }
    if year < 1900 || year > chrono::Utc::now().year() as i64 + 1 {
        return Err(AppError::InvalidInput("seed year is out of range".into()));
    }
    if quantity < 0 {
        return Err(AppError::InvalidInput("quantity must be non-negative".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO seeds (id, catalog_id, variety, source, year, quantity, germination_notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, catalog_id, variety, source, year, quantity, germination_notes],
    )?;
    get_seed(conn, &id)
}

pub fn get_seed(conn: &Connection, id: &str) -> AppResult<Seed> {
    conn.query_row(&format!("{} WHERE s.id = ?1", SEED_SELECT), params![id], row_to_seed)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn list_seeds(conn: &Connection) -> AppResult<Vec<Seed>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY s.created_at DESC", SEED_SELECT))?;
    let rows = stmt.query_map([], row_to_seed)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_seed(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM seeds WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn acknowledge_viability(conn: &Connection, id: &str) -> AppResult<Seed> {
    conn.execute("UPDATE seeds SET viability_warned = 1 WHERE id = ?1", params![id])?;
    get_seed(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use garden_core::models::Viability;

    fn this_year() -> i64 {
        chrono::Utc::now().year() as i64
    }

    #[test]
    fn add_and_list_seed() {
        let conn = seeded();
        let s = add_seed(&conn, Some("plant_tomato"), "Cherokee Purple", "Local shop", this_year(), 3, Some("great germ")).unwrap();
        assert_eq!(s.plant_name.as_deref(), Some("Tomato"));
        assert_eq!(s.quantity, 3);
        assert_eq!(list_seeds(&conn).unwrap().len(), 1);
    }

    #[test]
    fn viability_is_derived() {
        let conn = seeded();
        let fresh = add_seed(&conn, None, "Fresh", "x", this_year(), 1, None).unwrap();
        assert_eq!(fresh.viability, Viability::Fresh);
        let expired = add_seed(&conn, None, "Old", "x", this_year() - 5, 1, None).unwrap();
        assert_eq!(expired.viability, Viability::Expired);
        assert!(!expired.viability_warned);
        let acked = acknowledge_viability(&conn, &expired.id).unwrap();
        assert!(acked.viability_warned);
    }

    #[test]
    fn invalid_inputs_rejected() {
        let conn = seeded();
        assert!(add_seed(&conn, None, "  ", "x", this_year(), 1, None).is_err());
        assert!(add_seed(&conn, None, "Var", "x", 1000, 1, None).is_err());
        assert!(add_seed(&conn, None, "Var", "x", this_year(), -1, None).is_err());
    }

    #[test]
    fn delete_removes() {
        let conn = seeded();
        let s = add_seed(&conn, None, "Var", "x", this_year(), 1, None).unwrap();
        delete_seed(&conn, &s.id).unwrap();
        assert!(list_seeds(&conn).unwrap().is_empty());
    }
}
