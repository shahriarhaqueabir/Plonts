use std::path::Path;

use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::models::{JournalEntry, JournalFilters, JournalPhoto, Measurement, MeasurementKind, PlantTimeline};

fn enum_str<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap().trim_matches('"').to_string()
}

fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> T {
    serde_json::from_str(s).expect("valid json field")
}

fn parse_kind(s: &str) -> MeasurementKind {
    serde_json::from_str(&format!("\"{}\"", s)).expect("valid measurement kind")
}

const ENTRY_SELECT: &str = "SELECT e.id, e.plant_id, e.bed_id, c.name AS plant_name, b.name AS bed_name,
    e.text, e.tags, e.created_at, e.updated_at
    FROM journal_entries e
    LEFT JOIN catalog c ON c.id = e.plant_id
    LEFT JOIN beds b ON b.id = e.bed_id";

fn entry_from_row(conn: &Connection, photos_base: &Path, row: &rusqlite::Row) -> rusqlite::Result<JournalEntry> {
    let id: String = row.get(0)?;
    let tags: String = row.get(6)?;
    Ok(JournalEntry {
        id: id.clone(),
        plant_id: row.get(1)?,
        bed_id: row.get(2)?,
        plant_name: row.get(3)?,
        bed_name: row.get(4)?,
        text: row.get(5)?,
        tags: parse_json(&tags),
        photos: list_photos(conn, photos_base, &id).unwrap_or_default(),
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

pub fn list_entries(
    conn: &Connection,
    photos_base: &Path,
    filters: &JournalFilters,
) -> AppResult<Vec<JournalEntry>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY e.created_at DESC", ENTRY_SELECT))?;
    let rows = stmt.query_map([], |r| entry_from_row(conn, photos_base, r))?;
    let mut entries: Vec<JournalEntry> = rows.collect::<Result<Vec<_>, _>>()?;
    if let Some(q) = filters.query.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let q = q.to_lowercase();
        entries.retain(|e| {
            e.text.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
        });
    }
    if let Some(pid) = &filters.plant_id {
        entries.retain(|e| e.plant_id.as_deref() == Some(pid.as_str()));
    }
    if let Some(tag) = &filters.tag {
        entries.retain(|e| e.tags.iter().any(|t| t == tag));
    }
    Ok(entries)
}

pub fn get_entry(conn: &Connection, photos_base: &Path, id: &str) -> AppResult<JournalEntry> {
    conn.query_row(&format!("{} WHERE e.id = ?1", ENTRY_SELECT), params![id], |r| {
        entry_from_row(conn, photos_base, r)
    })
    .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn create_entry(
    conn: &Connection,
    photos_base: &Path,
    plant_id: Option<&str>,
    bed_id: Option<&str>,
    text: &str,
    tags: &[String],
) -> AppResult<JournalEntry> {
    if text.trim().is_empty() {
        return Err(AppError::InvalidInput("journal text is required".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO journal_entries (id, plant_id, bed_id, text, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, plant_id, bed_id, text, serde_json::to_string(tags)?],
    )?;
    get_entry(conn, photos_base, &id)
}

pub fn update_entry(
    conn: &Connection,
    photos_base: &Path,
    id: &str,
    plant_id: Option<&str>,
    bed_id: Option<&str>,
    text: &str,
    tags: &[String],
) -> AppResult<JournalEntry> {
    if text.trim().is_empty() {
        return Err(AppError::InvalidInput("journal text is required".into()));
    }
    conn.execute(
        "UPDATE journal_entries SET plant_id = ?2, bed_id = ?3, text = ?4, tags = ?5,
         updated_at = datetime('now') WHERE id = ?1",
        params![id, plant_id, bed_id, text, serde_json::to_string(tags)?],
    )?;
    get_entry(conn, photos_base, id)
}

/// Deletes an entry (cascading its photo rows) and returns the photo filenames
/// so the caller can delete the backing files from disk.
pub fn delete_entry(conn: &Connection, id: &str) -> AppResult<Vec<String>> {
    let filenames = photo_filenames_for_entry(conn, id)?;
    conn.execute("DELETE FROM journal_entries WHERE id = ?1", params![id])?;
    Ok(filenames)
}

fn photo_filenames_for_entry(conn: &Connection, entry_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM journal_photos WHERE entry_id = ?1")?;
    let rows = stmt.query_map(params![entry_id], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn insert_photo(
    conn: &Connection,
    photos_base: &Path,
    entry_id: &str,
    filename: &str,
) -> AppResult<JournalPhoto> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO journal_photos (id, entry_id, path) VALUES (?1, ?2, ?3)",
        params![id, entry_id, filename],
    )?;
    get_photo(conn, photos_base, &id)
}

pub fn get_photo(conn: &Connection, photos_base: &Path, id: &str) -> AppResult<JournalPhoto> {
    let (entry_id, path, created_at): (String, String, String) = conn
        .query_row(
            "SELECT entry_id, path, created_at FROM journal_photos WHERE id = ?1",
            params![id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
        )
        .map_err(|_| AppError::NotFound(id.to_string()))?;
    Ok(JournalPhoto {
        id: id.to_string(),
        entry_id,
        url: crate::storage::asset_url(photos_base, &path),
        path,
        created_at,
    })
}

pub fn list_photos(conn: &Connection, photos_base: &Path, entry_id: &str) -> AppResult<Vec<JournalPhoto>> {
    let mut stmt = conn.prepare(
        "SELECT id, entry_id, path, created_at FROM journal_photos WHERE entry_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt.query_map(params![entry_id], |r| {
        let path: String = r.get(2)?;
        Ok(JournalPhoto {
            id: r.get(0)?,
            entry_id: r.get(1)?,
            path: path.clone(),
            url: crate::storage::asset_url(photos_base, &path),
            created_at: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_photo(conn: &Connection, id: &str) -> AppResult<String> {
    let filename: String = conn
        .query_row("SELECT path FROM journal_photos WHERE id = ?1", params![id], |r| r.get(0))
        .map_err(|_| AppError::NotFound(id.to_string()))?;
    conn.execute("DELETE FROM journal_photos WHERE id = ?1", params![id])?;
    Ok(filename)
}

const MEASUREMENT_SELECT: &str = "SELECT m.id, m.plant_id, COALESCE(c.name, m.plant_id), m.kind,
    m.value, m.unit, m.recorded_at
    FROM measurements m LEFT JOIN catalog c ON c.id = m.plant_id";

fn measurement_from_row(row: &rusqlite::Row) -> rusqlite::Result<Measurement> {
    let kind: String = row.get(3)?;
    Ok(Measurement {
        id: row.get(0)?,
        plant_id: row.get(1)?,
        plant_name: row.get(2)?,
        kind: parse_kind(&kind),
        value: row.get(4)?,
        unit: row.get(5)?,
        recorded_at: row.get(6)?,
    })
}

pub fn add_measurement(
    conn: &Connection,
    plant_id: &str,
    kind: MeasurementKind,
    value: f64,
    unit: &str,
) -> AppResult<Measurement> {
    if !value.is_finite() || value <= 0.0 {
        return Err(AppError::InvalidInput("measurement value must be positive".into()));
    }
    if unit.trim().is_empty() {
        return Err(AppError::InvalidInput("measurement unit is required".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO measurements (id, plant_id, kind, value, unit) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, plant_id, enum_str(&kind), value, unit],
    )?;
    get_measurement(conn, &id)
}

pub fn get_measurement(conn: &Connection, id: &str) -> AppResult<Measurement> {
    conn.query_row(
        &format!("{} WHERE m.id = ?1", MEASUREMENT_SELECT),
        params![id],
        measurement_from_row,
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn list_measurements(conn: &Connection, plant_id: &str) -> AppResult<Vec<Measurement>> {
    let mut stmt = conn.prepare(&format!(
        "{} WHERE m.plant_id = ?1 ORDER BY m.recorded_at DESC",
        MEASUREMENT_SELECT
    ))?;
    let rows = stmt.query_map(params![plant_id], measurement_from_row)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn delete_measurement(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM measurements WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_timeline(conn: &Connection, photos_base: &Path, plant_id: &str) -> AppResult<PlantTimeline> {
    let plant_name: String = conn
        .query_row("SELECT name FROM catalog WHERE id = ?1", params![plant_id], |r| r.get(0))
        .unwrap_or_else(|_| plant_id.to_string());
    let measurements = list_measurements(conn, plant_id)?;
    let filters = JournalFilters { plant_id: Some(plant_id.to_string()), ..Default::default() };
    let entries = list_entries(conn, photos_base, &filters)?;
    Ok(PlantTimeline {
        plant_id: plant_id.to_string(),
        plant_name,
        measurements,
        entries,
    })
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
        db::seed::seed(&mut conn).unwrap();
        let base = std::env::temp_dir().join(format!("lovelygarden-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).unwrap();
        (conn, base)
    }

    #[test]
    fn create_and_list_entries() {
        let (conn, base) = seeded();
        create_entry(&conn, &base, None, None, "Watered everything", &[]).unwrap();
        assert_eq!(list_entries(&conn, &base, &JournalFilters::default()).unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn entry_resolves_plant_and_bed_names() {
        let (conn, base) = seeded();
        let g = db::gardens::create_garden(&conn, "Backyard", "raised_bed").unwrap();
        let bed = db::gardens::create_bed(&conn, &g.id, "Bed 1", None, None, None, None, false).unwrap();
        let e = create_entry(
            &conn,
            &base,
            Some("plant_tomato"),
            Some(&bed.id),
            "Tomatoes look great",
            &vec!["tomato".to_string()],
        )
        .unwrap();
        assert_eq!(e.plant_name.as_deref(), Some("Tomato"));
        assert_eq!(e.bed_name.as_deref(), Some("Bed 1"));
        assert_eq!(e.tags, vec!["tomato".to_string()]);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn empty_text_rejected() {
        let (conn, base) = seeded();
        assert!(create_entry(&conn, &base, None, None, "   ", &[]).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn update_entry_changes_text() {
        let (conn, base) = seeded();
        let e = create_entry(&conn, &base, None, None, "before", &[]).unwrap();
        let u = update_entry(&conn, &base, &e.id, None, None, "after", &[]).unwrap();
        assert_eq!(u.text, "after");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn delete_entry_returns_photo_filenames() {
        let (conn, base) = seeded();
        let e = create_entry(&conn, &base, None, None, "hello", &[]).unwrap();
        insert_photo(&conn, &base, &e.id, "x.jpg").unwrap();
        insert_photo(&conn, &base, &e.id, "y.jpg").unwrap();
        let names = delete_entry(&conn, &e.id).unwrap();
        assert_eq!(names.len(), 2);
        assert!(list_entries(&conn, &base, &JournalFilters::default()).unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn photo_insert_and_delete() {
        let (conn, base) = seeded();
        let e = create_entry(&conn, &base, None, None, "hello", &[]).unwrap();
        let p = insert_photo(&conn, &base, &e.id, "abc.jpg").unwrap();
        assert!(p.url.starts_with("asset://localhost/"));
        assert_eq!(delete_photo(&conn, &p.id).unwrap(), "abc.jpg");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn measurement_add_and_list() {
        let (conn, base) = seeded();
        add_measurement(&conn, "plant_tomato", MeasurementKind::Height, 42.5, "cm").unwrap();
        let list = list_measurements(&conn, "plant_tomato").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].plant_name, "Tomato");
        assert_eq!(list[0].value, 42.5);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn bad_measurement_rejected() {
        let (conn, base) = seeded();
        assert!(add_measurement(&conn, "plant_tomato", MeasurementKind::Height, -1.0, "cm").is_err());
        assert!(add_measurement(&conn, "plant_tomato", MeasurementKind::Yield, 5.0, "").is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn search_filters_by_text_and_tag() {
        let (conn, base) = seeded();
        create_entry(&conn, &base, None, None, "watered the bed", &vec!["watering".to_string()]).unwrap();
        create_entry(&conn, &base, None, None, "harvested basil", &[]).unwrap();
        let by_text = list_entries(&conn, &base, &JournalFilters { query: Some("basil".into()), ..Default::default() }).unwrap();
        assert_eq!(by_text.len(), 1);
        let by_tag = list_entries(&conn, &base, &JournalFilters { tag: Some("watering".into()), ..Default::default() }).unwrap();
        assert_eq!(by_tag.len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn timeline_groups_measurements_and_entries() {
        let (conn, base) = seeded();
        create_entry(&conn, &base, Some("plant_tomato"), None, "planted", &[]).unwrap();
        add_measurement(&conn, "plant_tomato", MeasurementKind::Height, 10.0, "cm").unwrap();
        let t = get_timeline(&conn, &base, "plant_tomato").unwrap();
        assert_eq!(t.plant_name, "Tomato");
        assert_eq!(t.measurements.len(), 1);
        assert_eq!(t.entries.len(), 1);
        let _ = std::fs::remove_dir_all(&base);
    }
}