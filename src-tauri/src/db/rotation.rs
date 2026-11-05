use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::models::RotationPlan;

#[cfg(test)]
use crate::db;

#[cfg(test)]
fn in_memory() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    conn
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn list_rotation_plans(conn: &Connection, garden_id: &str) -> AppResult<Vec<RotationPlan>> {
    let mut stmt = conn.prepare(
        "SELECT id, garden_id, season, plant_family, notes, created_at
         FROM rotation_plans WHERE garden_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt.query_map(params![garden_id], |r| {
        Ok(RotationPlan {
            id: r.get(0)?,
            garden_id: r.get(1)?,
            season: r.get(2)?,
            plant_family: r.get(3)?,
            notes: r.get(4)?,
            created_at: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_rotation_plan(
    conn: &Connection,
    garden_id: &str,
    season: &str,
    plant_family: &str,
    notes: Option<&str>,
) -> AppResult<RotationPlan> {
    if plant_family.trim().is_empty() {
        return Err(AppError::InvalidInput("plant family is required".into()));
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO rotation_plans (id, garden_id, season, plant_family, notes) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, garden_id, season, plant_family, notes],
    )?;
    conn.query_row(
        "SELECT id, garden_id, season, plant_family, notes, created_at FROM rotation_plans WHERE id = ?1",
        params![id],
        |r| {
            Ok(RotationPlan {
                id: r.get(0)?,
                garden_id: r.get(1)?,
                season: r.get(2)?,
                plant_family: r.get(3)?,
                notes: r.get(4)?,
                created_at: r.get(5)?,
            })
        },
    )
    .map_err(|_| AppError::NotFound(id))
}

pub fn delete_rotation_plan(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM rotation_plans WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn rotation_crud_roundtrip() {
        let conn = in_memory();
        let g = db::gardens::create_garden(&conn, "G", "raised_bed").unwrap();
        let p = create_rotation_plan(&conn, &g.id, "Spring", "Solanaceae", None).unwrap();
        assert_eq!(p.season, "Spring");
        assert_eq!(p.plant_family, "Solanaceae");
        assert_eq!(list_rotation_plans(&conn, &g.id).unwrap().len(), 1);
        delete_rotation_plan(&conn, &p.id).unwrap();
        assert!(list_rotation_plans(&conn, &g.id).unwrap().is_empty());
    }

    #[test]
    fn rotation_requires_family() {
        let conn = in_memory();
        let g = db::gardens::create_garden(&conn, "G", "raised_bed").unwrap();
        assert!(create_rotation_plan(&conn, &g.id, "Spring", "  ", None).is_err());
    }
}
