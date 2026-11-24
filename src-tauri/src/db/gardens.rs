use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::models::{Bed, Garden};

#[cfg(test)]
use crate::db;

#[cfg(test)]
fn in_memory() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list_gardens() {
        let conn = in_memory();
        let g = create_garden(&conn, "Backyard", "raised_bed").unwrap();
        assert_eq!(g.name, "Backyard");
        assert_eq!(list_gardens(&conn).unwrap().len(), 1);
    }

    #[test]
    fn beds_belong_to_garden_and_cascade() {
        let conn = in_memory();
        let g = create_garden(&conn, "Backyard", "raised_bed").unwrap();
        let b = create_bed(
            &conn,
            &g.id,
            "Bed 1",
            Some("loam"),
            Some("full_sun"),
            Some(120),
            Some(90),
            false,
        )
        .unwrap();
        assert_eq!(b.garden_id, g.id);
        assert_eq!(list_beds(&conn, &g.id).unwrap().len(), 1);
        delete_garden(&conn, &g.id).unwrap();
        // cascade deletes the bed
        assert_eq!(list_beds(&conn, &g.id).unwrap().len(), 0);
    }
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn list_gardens(conn: &Connection) -> AppResult<Vec<Garden>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, garden_type, created_at FROM gardens ORDER BY created_at",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(Garden {
            id: r.get(0)?,
            name: r.get(1)?,
            garden_type: r.get(2)?,
            created_at: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn create_garden(conn: &Connection, name: &str, garden_type: &str) -> AppResult<Garden> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput("garden name is required".into()));
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO gardens (id, name, garden_type) VALUES (?1, ?2, ?3)",
        params![id, name, garden_type],
    )?;
    get_garden(conn, &id)
}

pub fn get_garden(conn: &Connection, id: &str) -> AppResult<Garden> {
    conn.query_row(
        "SELECT id, name, garden_type, created_at FROM gardens WHERE id = ?1",
        params![id],
        |r| {
            Ok(Garden {
                id: r.get(0)?,
                name: r.get(1)?,
                garden_type: r.get(2)?,
                created_at: r.get(3)?,
            })
        },
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn delete_garden(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM gardens WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn list_beds(conn: &Connection, garden_id: &str) -> AppResult<Vec<Bed>> {
    let mut stmt = conn.prepare(
        "SELECT id, garden_id, name, soil_type, sun_exposure, width_cm, height_cm, is_container
         FROM beds WHERE garden_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt.query_map(params![garden_id], |r| {
        Ok(Bed {
            id: r.get(0)?,
            garden_id: r.get(1)?,
            name: r.get(2)?,
            soil_type: r.get(3)?,
            sun_exposure: r.get(4)?,
            width_cm: r.get(5)?,
            height_cm: r.get(6)?,
            is_container: r.get::<_, i64>(7)? != 0,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[allow(clippy::too_many_arguments)]
pub fn create_bed(
    conn: &Connection,
    garden_id: &str,
    name: &str,
    soil_type: Option<&str>,
    sun_exposure: Option<&str>,
    width_cm: Option<i64>,
    height_cm: Option<i64>,
    is_container: bool,
) -> AppResult<Bed> {
    let id = new_id();
    conn.execute(
        "INSERT INTO beds (id, garden_id, name, soil_type, sun_exposure, width_cm, height_cm, is_container)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            id, garden_id, name, soil_type, sun_exposure, width_cm, height_cm,
            is_container as i64
        ],
    )?;
    get_bed(conn, &id)
}

pub fn get_bed(conn: &Connection, id: &str) -> AppResult<Bed> {
    conn.query_row(
        "SELECT id, garden_id, name, soil_type, sun_exposure, width_cm, height_cm, is_container
         FROM beds WHERE id = ?1",
        params![id],
        |r| {
            Ok(Bed {
                id: r.get(0)?,
                garden_id: r.get(1)?,
                name: r.get(2)?,
                soil_type: r.get(3)?,
                sun_exposure: r.get(4)?,
                width_cm: r.get(5)?,
                height_cm: r.get(6)?,
                is_container: r.get::<_, i64>(7)? != 0,
            })
        },
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn delete_bed(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM beds WHERE id = ?1", params![id])?;
    Ok(())
}