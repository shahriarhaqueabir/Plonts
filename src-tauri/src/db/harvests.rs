use chrono::Datelike;
use rusqlite::{params, Connection};

#[cfg(test)]
use crate::db;
use crate::error::{AppError, AppResult};
use garden_core::models::{Harvest, YieldRow};
use garden_core::seasonal::{season_for_month0, season_name};

#[cfg(test)]
fn seeded() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    db::seed::seed(&mut conn).unwrap();
    conn
}

const HARVEST_SELECT: &str = "SELECT h.id, h.planting_id, h.catalog_id,
    COALESCE(c.name, pc.name) AS plant_name, h.date, h.weight_g, h.count, h.notes
    FROM harvests h
    LEFT JOIN catalog c ON c.id = h.catalog_id
    LEFT JOIN planted pl ON pl.id = h.planting_id
    LEFT JOIN catalog pc ON pc.id = pl.catalog_id";

fn row_to_harvest(row: &rusqlite::Row) -> rusqlite::Result<Harvest> {
    Ok(Harvest {
        id: row.get(0)?,
        planting_id: row.get(1)?,
        catalog_id: row.get(2)?,
        plant_name: row.get(3)?,
        date: row.get(4)?,
        weight_g: row.get(5)?,
        count: row.get(6)?,
        notes: row.get(7)?,
    })
}

pub fn add_harvest(
    conn: &Connection,
    planting_id: Option<&str>,
    catalog_id: Option<&str>,
    date: &str,
    weight_g: Option<f64>,
    count: Option<i64>,
    notes: Option<&str>,
) -> AppResult<Harvest> {
    if chrono::NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").is_err() {
        return Err(AppError::InvalidInput("harvest date must be YYYY-MM-DD".into()));
    }
    if weight_g.map(|w| !w.is_finite() || w < 0.0).unwrap_or(false) {
        return Err(AppError::InvalidInput("weight must be non-negative".into()));
    }
    if count.map(|c| c < 0).unwrap_or(false) {
        return Err(AppError::InvalidInput("count must be non-negative".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO harvests (id, planting_id, catalog_id, date, weight_g, count, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, planting_id, catalog_id, date.trim(), weight_g, count, notes],
    )?;
    get_harvest(conn, &id)
}

pub fn get_harvest(conn: &Connection, id: &str) -> AppResult<Harvest> {
    conn.query_row(&format!("{} WHERE h.id = ?1", HARVEST_SELECT), params![id], row_to_harvest)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn list_harvests(conn: &Connection) -> AppResult<Vec<Harvest>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY h.date DESC", HARVEST_SELECT))?;
    let rows = stmt.query_map([], row_to_harvest)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_harvest(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM harvests WHERE id = ?1", params![id])?;
    Ok(())
}

fn sum_f64(acc: &mut Option<f64>, v: Option<f64>) {
    if let Some(x) = v {
        *acc = Some(acc.unwrap_or(0.0) + x);
    }
}

fn sum_i64(acc: &mut Option<i64>, v: Option<i64>) {
    if let Some(x) = v {
        *acc = Some(acc.unwrap_or(0) + x);
    }
}

pub fn yield_summary(conn: &Connection, group_by: &str) -> AppResult<Vec<YieldRow>> {
    match group_by {
        "plant" => yield_by_plant(conn),
        "bed" => yield_by_bed(conn),
        "season" => yield_by_season(conn),
        other => Err(AppError::InvalidInput(format!("unknown group_by: {}", other))),
    }
}

fn yield_by_plant(conn: &Connection) -> AppResult<Vec<YieldRow>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(h.catalog_id, pl.catalog_id, 'unknown') AS key,
                COALESCE(c.name, pc.name, 'Unknown plant') AS label,
                SUM(h.weight_g), SUM(h.count), COUNT(*)
         FROM harvests h
         LEFT JOIN catalog c ON c.id = h.catalog_id
         LEFT JOIN planted pl ON pl.id = h.planting_id
         LEFT JOIN catalog pc ON pc.id = pl.catalog_id
         GROUP BY key ORDER BY label",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(YieldRow {
            key: r.get(0)?,
            label: r.get(1)?,
            total_weight_g: r.get(2)?,
            total_count: r.get(3)?,
            harvest_count: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn yield_by_bed(conn: &Connection) -> AppResult<Vec<YieldRow>> {
    let mut stmt = conn.prepare(
        "SELECT COALESCE(b.id, 'no-bed') AS key,
                COALESCE(b.name, 'No bed') AS label,
                SUM(h.weight_g), SUM(h.count), COUNT(*)
         FROM harvests h
         LEFT JOIN planted pl ON pl.id = h.planting_id
         LEFT JOIN beds b ON b.id = pl.bed_id
         GROUP BY key ORDER BY label",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(YieldRow {
            key: r.get(0)?,
            label: r.get(1)?,
            total_weight_g: r.get(2)?,
            total_count: r.get(3)?,
            harvest_count: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn yield_by_season(conn: &Connection) -> AppResult<Vec<YieldRow>> {
    let harvests = list_harvests(conn)?;
    let mut acc: std::collections::BTreeMap<String, (Option<f64>, Option<i64>, i64)> = std::collections::BTreeMap::new();
    for h in &harvests {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&h.date, "%Y-%m-%d") {
            let label = season_name(season_for_month0(d.month0() as i32));
            let e = acc.entry(label.to_string()).or_insert((None, None, 0));
            sum_f64(&mut e.0, h.weight_g);
            sum_i64(&mut e.1, h.count);
            e.2 += 1;
        }
    }
    let order = ["Spring", "Summer", "Autumn", "Winter"];
    let mut rows: Vec<YieldRow> = acc
        .into_iter()
        .map(|(label, (w, c, n))| YieldRow {
            key: label.clone(),
            label,
            total_weight_g: w,
            total_count: c,
            harvest_count: n,
        })
        .collect();
    rows.sort_by_key(|r| order.iter().position(|s| *s == r.label).unwrap_or(usize::MAX));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_list_harvest() {
        let conn = seeded();
        let h = add_harvest(&conn, None, Some("plant_tomato"), "2026-07-15", Some(1200.0), Some(6), Some("great")).unwrap();
        assert_eq!(h.plant_name.as_deref(), Some("Tomato"));
        assert_eq!(list_harvests(&conn).unwrap().len(), 1);
    }

    #[test]
    fn invalid_date_rejected() {
        let conn = seeded();
        assert!(add_harvest(&conn, None, None, "not-a-date", None, None, None).is_err());
        assert!(add_harvest(&conn, None, None, "2026-07-15", Some(-1.0), None, None).is_err());
    }

    #[test]
    fn yield_by_plant_sums() {
        let conn = seeded();
        add_harvest(&conn, None, Some("plant_tomato"), "2026-07-01", Some(1000.0), Some(5), None).unwrap();
        add_harvest(&conn, None, Some("plant_tomato"), "2026-07-02", Some(500.0), Some(3), None).unwrap();
        add_harvest(&conn, None, Some("plant_basil"), "2026-07-03", Some(50.0), None, None).unwrap();
        let rows = yield_summary(&conn, "plant").unwrap();
        let tomato = rows.iter().find(|r| r.label == "Tomato").unwrap();
        assert_eq!(tomato.total_weight_g, Some(1500.0));
        assert_eq!(tomato.total_count, Some(8));
        assert_eq!(tomato.harvest_count, 2);
    }

    #[test]
    fn yield_by_season_groups_month() {
        let conn = seeded();
        add_harvest(&conn, None, Some("plant_tomato"), "2026-03-10", Some(100.0), None, None).unwrap();
        add_harvest(&conn, None, Some("plant_tomato"), "2026-07-10", Some(200.0), None, None).unwrap();
        let rows = yield_summary(&conn, "season").unwrap();
        assert!(rows.iter().any(|r| r.label == "Spring"));
        assert!(rows.iter().any(|r| r.label == "Summer"));
        let spring = rows.iter().find(|r| r.label == "Spring").unwrap();
        assert_eq!(spring.total_weight_g, Some(100.0));
    }

    #[test]
    fn unknown_group_rejected() {
        let conn = seeded();
        assert!(yield_summary(&conn, "nope").is_err());
    }
}
