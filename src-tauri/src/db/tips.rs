use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::models::DailyTip;

const TIPS_JSON: &str = include_str!("../../resources/data/daily-tips.json");
pub const CURRENT_TIPS_VERSION: i64 = 1;

#[derive(serde::Deserialize)]
struct RawTip {
    id: String,
    title: String,
    body: String,
}

pub fn seed_tips(conn: &mut Connection) -> AppResult<()> {
    let tips: Vec<RawTip> = serde_json::from_str(TIPS_JSON)?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM daily_tips", [])?;
    for t in &tips {
        tx.execute(
            "INSERT INTO daily_tips (id, title, body) VALUES (?1, ?2, ?3)",
            params![t.id, t.title, t.body],
        )?;
    }
    tx.execute(
        "INSERT OR REPLACE INTO content_packs (id, version) VALUES ('tips', ?1)",
        params![CURRENT_TIPS_VERSION],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn ensure_tips_seeded(conn: &mut Connection) -> AppResult<()> {
    let version: i64 = conn
        .query_row(
            "SELECT COALESCE((SELECT version FROM content_packs WHERE id='tips'), 0)",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if version < CURRENT_TIPS_VERSION {
        seed_tips(conn)?;
    }
    Ok(())
}

fn tip_from_row(row: &rusqlite::Row) -> rusqlite::Result<DailyTip> {
    Ok(DailyTip {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        dismiss_until: row.get(3)?,
        snoozed_until: row.get(4)?,
    })
}

pub fn get_tip(conn: &Connection, id: &str) -> AppResult<DailyTip> {
    conn.query_row(
        "SELECT id, title, body, dismiss_until, snoozed_until FROM daily_tips WHERE id = ?1",
        params![id],
        tip_from_row,
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

/// Returns today's eligible tip (not dismissed/snoozed), rotated by day, or None.
pub fn get_daily_tip(conn: &Connection, now_rfc3339: &str) -> AppResult<Option<DailyTip>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, body, dismiss_until, snoozed_until FROM daily_tips
         WHERE (dismiss_until IS NULL OR dismiss_until <= ?1)
           AND (snoozed_until IS NULL OR snoozed_until <= ?1)
         ORDER BY id",
    )?;
    let rows = stmt.query_map(params![now_rfc3339], tip_from_row)?;
    let eligible: Vec<DailyTip> = rows.collect::<Result<Vec<_>, _>>()?;
    if eligible.is_empty() {
        return Ok(None);
    }
    let day = garden_core::tips::days_from_rfc3339(now_rfc3339);
    let idx = garden_core::tips::daily_tip_index(day, eligible.len());
    Ok(Some(eligible[idx].clone()))
}

pub fn dismiss_tip(conn: &Connection, id: &str, days: i64) -> AppResult<DailyTip> {
    let until = (chrono::Utc::now() + chrono::Duration::days(days)).to_rfc3339();
    conn.execute("UPDATE daily_tips SET dismiss_until = ?1 WHERE id = ?2", params![until, id])?;
    get_tip(conn, id)
}

pub fn snooze_tip(conn: &Connection, id: &str, hours: i64) -> AppResult<DailyTip> {
    let until = (chrono::Utc::now() + chrono::Duration::hours(hours)).to_rfc3339();
    conn.execute("UPDATE daily_tips SET snoozed_until = ?1 WHERE id = ?2", params![until, id])?;
    get_tip(conn, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrate(&mut conn).unwrap();
        seed_tips(&mut conn).unwrap();
        conn
    }

    fn list_tip_ids(conn: &Connection) -> Vec<String> {
        let mut stmt = conn.prepare("SELECT id FROM daily_tips").unwrap();
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).unwrap();
        rows.collect::<Result<Vec<_>, _>>().unwrap()
    }

    #[test]
    fn seeds_six_tips() {
        let conn = seeded();
        assert_eq!(list_tip_ids(&conn).len(), 6);
    }

    #[test]
    fn same_day_returns_same_tip() {
        let conn = seeded();
        let a = get_daily_tip(&conn, "2026-06-15T12:00:00+00:00").unwrap().unwrap();
        let b = get_daily_tip(&conn, "2026-06-15T23:59:00+00:00").unwrap().unwrap();
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn dismiss_hides_tip() {
        let conn = seeded();
        let now = "2026-06-15T12:00:00+00:00";
        let tip = get_daily_tip(&conn, now).unwrap().unwrap();
        dismiss_tip(&conn, &tip.id, 30).unwrap();
        let next = get_daily_tip(&conn, now).unwrap();
        assert!(next.is_none() || next.unwrap().id != tip.id);
    }

    #[test]
    fn snooze_hides_tip() {
        let conn = seeded();
        let now = "2026-06-15T12:00:00+00:00";
        let tip = get_daily_tip(&conn, now).unwrap().unwrap();
        snooze_tip(&conn, &tip.id, 8).unwrap();
        let next = get_daily_tip(&conn, now).unwrap();
        assert!(next.is_none() || next.unwrap().id != tip.id);
    }

    #[test]
    fn no_tip_when_all_dismissed() {
        let conn = seeded();
        let ids = list_tip_ids(&conn);
        for id in ids {
            dismiss_tip(&conn, &id, 1).unwrap();
        }
        assert!(get_daily_tip(&conn, "2026-06-15T12:00:00+00:00").unwrap().is_none());
    }
}
