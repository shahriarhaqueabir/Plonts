use rusqlite::Connection;

use crate::db;
use crate::error::AppResult;
use garden_core::models::{CalendarEntry, Hemisphere, PlantSummary, Season, Seasonality};
use garden_core::seasonal::sowing_eligible;

fn seeded() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    db::migrate(&mut conn).unwrap();
    db::seed::seed(&mut conn).unwrap();
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_returns_entries_with_eligibility() {
        let conn = seeded();
        let entries = get_calendar(&conn, 0, Hemisphere::North).unwrap();
        assert!(entries.len() >= 80);
        assert!(entries.iter().any(|e| e.eligible));
        assert!(entries.iter().any(|e| !e.eligible));
        assert!(!entries.iter().any(|e| e.reason.is_empty()));
    }

    #[test]
    fn hemisphere_changes_eligibility() {
        let conn = seeded();
        let north = get_calendar(&conn, 5, Hemisphere::North).unwrap();
        let south = get_calendar(&conn, 5, Hemisphere::South).unwrap();
        let north_eligible = north.iter().filter(|e| e.eligible).count();
        let south_eligible = south.iter().filter(|e| e.eligible).count();
        assert_ne!(north_eligible, south_eligible);
    }
}

fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> T {
    serde_json::from_str(s).unwrap_or_else(|_| panic!("invalid json field"))
}

pub fn get_calendar(conn: &Connection, month0: i32, hemisphere: Hemisphere) -> AppResult<Vec<CalendarEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, scientific_name, categories, sowing_season, family, life_cycle,
                sunlight, water_requirements, seasonality
         FROM catalog ORDER BY name",
    )?;
    let rows = stmt.query_map([], |r| {
        let categories: String = r.get(3)?;
        let sowing: String = r.get(4)?;
        let seasonality: Option<String> = r.get(9)?;
        let summary = PlantSummary {
            id: r.get(0)?,
            name: r.get(1)?,
            scientific_name: r.get(2)?,
            categories: parse_json(&categories),
            sowing_season: parse_json::<Vec<Season>>(&sowing),
            family: r.get(5)?,
            life_cycle: r.get(6)?,
            sunlight: r.get(7)?,
            water_requirements: r.get(8)?,
        };
        let seas: Option<Seasonality> = seasonality.map(|s| parse_json(&s));
        Ok((summary, seas))
    })?;

    let mut entries = Vec::new();
    for row in rows {
        let (summary, seas) = row?;
        let (eligible, reason) = sowing_eligible(seas.as_ref(), &summary.sowing_season, hemisphere, month0);
        entries.push(CalendarEntry { plant: summary, eligible, reason });
    }

    // Eligible first, then alphabetical within each group.
    entries.sort_by(|a, b| {
        b.eligible
            .cmp(&a.eligible)
            .then_with(|| a.plant.name.cmp(&b.plant.name))
    });

    Ok(entries)
}