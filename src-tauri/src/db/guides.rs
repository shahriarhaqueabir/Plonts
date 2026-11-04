use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::models::{Guide, GuideFilters, GuideProgress, GuideStep, GuideTopic, SkillLevel};

const GUIDES_JSON: &str = include_str!("../../resources/data/guides.json");
pub const CURRENT_GUIDE_VERSION: i64 = 1;

#[derive(serde::Deserialize)]
struct RawGuide {
    id: String,
    title: String,
    topic: GuideTopic,
    skill_level: SkillLevel,
    time_minutes: i64,
    steps: Vec<GuideStep>,
    #[serde(default)]
    materials: Vec<String>,
    #[serde(default)]
    illustrations: Vec<String>,
    #[serde(default)]
    version: i64,
}

fn json<T: serde::Serialize>(v: &T) -> AppResult<String> {
    Ok(serde_json::to_string(v)?)
}

fn enum_str<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap().trim_matches('"').to_string()
}

fn parse_topic(s: &str) -> GuideTopic {
    serde_json::from_str(&format!("\"{}\"", s)).expect("valid topic")
}

fn parse_skill(s: &str) -> SkillLevel {
    serde_json::from_str(&format!("\"{}\"", s)).expect("valid skill level")
}

fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> T {
    serde_json::from_str(s).expect("valid json field")
}

pub fn seed_guides(conn: &mut Connection) -> AppResult<()> {
    let guides: Vec<RawGuide> = serde_json::from_str(GUIDES_JSON)?;
    let tx = conn.transaction()?;
    for g in &guides {
        tx.execute(
            "INSERT OR REPLACE INTO guides (id, title, topic, skill_level, time_minutes, steps, materials, illustrations, version)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                g.id,
                g.title,
                enum_str(&g.topic),
                enum_str(&g.skill_level),
                g.time_minutes,
                json(&g.steps)?,
                json(&g.materials)?,
                json(&g.illustrations)?,
                g.version,
            ],
        )?;
    }
    tx.execute(
        "INSERT OR REPLACE INTO content_packs (id, version) VALUES ('guides', ?1)",
        params![CURRENT_GUIDE_VERSION],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn ensure_guides_seeded(conn: &mut Connection) -> AppResult<()> {
    let version: i64 = conn
        .query_row(
            "SELECT COALESCE((SELECT version FROM content_packs WHERE id='guides'), 0)",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if version < CURRENT_GUIDE_VERSION {
        seed_guides(conn)?;
    }
    Ok(())
}

const GUIDE_SELECT: &str = "SELECT g.id, g.title, g.topic, g.skill_level, g.time_minutes,
    g.steps, g.materials, g.illustrations, g.version,
    (b.guide_id IS NOT NULL) AS bookmarked, p.current_step, p.completed, p.updated_at
    FROM guides g
    LEFT JOIN guide_bookmarks b ON b.guide_id = g.id
    LEFT JOIN guide_progress p ON p.guide_id = g.id";

fn guide_from_row(row: &rusqlite::Row) -> rusqlite::Result<Guide> {
    let steps: String = row.get(5)?;
    let materials: String = row.get(6)?;
    let illustrations: String = row.get(7)?;
    let topic: String = row.get(2)?;
    let skill: String = row.get(3)?;
    let bookmarked: i64 = row.get(9)?;
    let progress = match row.get::<_, Option<i64>>(10)? {
        Some(cur) => Some(GuideProgress {
            guide_id: row.get(0)?,
            current_step: cur,
            completed: row.get::<_, i64>(11)? != 0,
            updated_at: row.get::<_, Option<String>>(12)?.unwrap_or_default(),
        }),
        None => None,
    };
    Ok(Guide {
        id: row.get(0)?,
        title: row.get(1)?,
        topic: parse_topic(&topic),
        skill_level: parse_skill(&skill),
        time_minutes: row.get(4)?,
        steps: parse_json(&steps),
        materials: parse_json(&materials),
        illustrations: parse_json(&illustrations),
        version: row.get(8)?,
        bookmarked: bookmarked != 0,
        progress,
    })
}

pub fn list_guides(conn: &Connection, filters: &GuideFilters) -> AppResult<Vec<Guide>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY g.title", GUIDE_SELECT))?;
    let rows = stmt.query_map([], guide_from_row)?;
    let mut guides: Vec<Guide> = rows.collect::<Result<Vec<_>, _>>()?;
    if let Some(t) = &filters.topic {
        guides.retain(|g| &g.topic == t);
    }
    if let Some(s) = &filters.skill_level {
        guides.retain(|g| &g.skill_level == s);
    }
    if let Some(max) = filters.time_max_minutes {
        guides.retain(|g| g.time_minutes <= max);
    }
    Ok(guides)
}

pub fn get_guide(conn: &Connection, id: &str) -> AppResult<Guide> {
    conn.query_row(&format!("{} WHERE g.id = ?1", GUIDE_SELECT), params![id], guide_from_row)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn save_progress(
    conn: &Connection,
    guide_id: &str,
    current_step: i64,
    completed: bool,
) -> AppResult<GuideProgress> {
    conn.execute(
        "INSERT INTO guide_progress (guide_id, current_step, completed, updated_at)
         VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(guide_id) DO UPDATE SET current_step = ?2, completed = ?3, updated_at = datetime('now')",
        params![guide_id, current_step, completed as i64],
    )?;
    get_progress(conn, guide_id)
}

pub fn get_progress(conn: &Connection, guide_id: &str) -> AppResult<GuideProgress> {
    conn.query_row(
        "SELECT guide_id, current_step, completed, updated_at FROM guide_progress WHERE guide_id = ?1",
        params![guide_id],
        |r| {
            Ok(GuideProgress {
                guide_id: r.get(0)?,
                current_step: r.get(1)?,
                completed: r.get::<_, i64>(2)? != 0,
                updated_at: r.get(3)?,
            })
        },
    )
    .map_err(|_| AppError::NotFound(guide_id.to_string()))
}

pub fn set_bookmark(conn: &Connection, guide_id: &str, bookmarked: bool) -> AppResult<Guide> {
    if bookmarked {
        conn.execute("INSERT OR IGNORE INTO guide_bookmarks (guide_id) VALUES (?1)", params![guide_id])?;
    } else {
        conn.execute("DELETE FROM guide_bookmarks WHERE guide_id = ?1", params![guide_id])?;
    }
    get_guide(conn, guide_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrate(&mut conn).unwrap();
        seed_guides(&mut conn).unwrap();
        conn
    }

    #[test]
    fn seeds_six_guides() {
        let conn = seeded();
        assert_eq!(list_guides(&conn, &GuideFilters::default()).unwrap().len(), 6);
    }

    #[test]
    fn get_guide_has_steps_and_materials() {
        let conn = seeded();
        let g = get_guide(&conn, "guide-soil-basics").unwrap();
        assert_eq!(g.topic, GuideTopic::Soil);
        assert!(!g.steps.is_empty());
        assert!(!g.materials.is_empty());
        assert!(!g.bookmarked);
        assert!(g.progress.is_none());
    }

    #[test]
    fn filters_by_topic() {
        let conn = seeded();
        let f = GuideFilters { topic: Some(GuideTopic::Soil), ..Default::default() };
        let r = list_guides(&conn, &f).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].id, "guide-soil-basics");
    }

    #[test]
    fn filters_by_time() {
        let conn = seeded();
        let f = GuideFilters { time_max_minutes: Some(15), ..Default::default() };
        let r = list_guides(&conn, &f).unwrap();
        assert!(r.iter().all(|g| g.time_minutes <= 15));
    }

    #[test]
    fn progress_round_trips() {
        let conn = seeded();
        let p = save_progress(&conn, "guide-soil-basics", 2, false).unwrap();
        assert_eq!(p.current_step, 2);
        assert!(!p.completed);
        let p2 = save_progress(&conn, "guide-soil-basics", 3, true).unwrap();
        assert_eq!(p2.current_step, 3);
        assert!(p2.completed);
        let g = get_guide(&conn, "guide-soil-basics").unwrap();
        assert_eq!(g.progress.unwrap().current_step, 3);
    }

    #[test]
    fn bookmark_toggles() {
        let conn = seeded();
        assert!(set_bookmark(&conn, "guide-soil-basics", true).unwrap().bookmarked);
        assert!(!set_bookmark(&conn, "guide-soil-basics", false).unwrap().bookmarked);
    }

    #[test]
    fn reseed_keeps_progress() {
        let mut conn = seeded();
        save_progress(&conn, "guide-soil-basics", 1, false).unwrap();
        seed_guides(&mut conn).unwrap();
        assert!(get_guide(&conn, "guide-soil-basics").unwrap().progress.is_some());
    }
}
