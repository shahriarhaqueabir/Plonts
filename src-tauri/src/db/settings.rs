use rusqlite::{params, Connection};

use crate::error::AppResult;
use garden_core::models::Settings;

#[cfg(test)]
use crate::db;

#[cfg(test)]
use garden_core::models::{ExperienceLevel, Hemisphere, Theme, Units};

#[cfg(test)]
fn in_memory() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    db::migrate(&mut conn).unwrap();
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_default_and_persists() {
        let conn = in_memory();
        let s = get(&conn).unwrap();
        assert_eq!(s.id, "local-user");
        assert_eq!(s.hemisphere, Hemisphere::North);
        assert!(!s.first_load_complete);
    }

    #[test]
    fn upsert_round_trips() {
        let conn = in_memory();
        let mut s = get(&conn).unwrap();
        s.hemisphere = Hemisphere::South;
        s.city = Some("Sydney".to_string());
        s.experience_level = ExperienceLevel::Expert;
        s.units = Units::Imperial;
        s.theme = Theme::Light;
        s.first_load_complete = true;
        upsert(&conn, &s).unwrap();
        let got = get(&conn).unwrap();
        assert_eq!(got.hemisphere, Hemisphere::South);
        assert_eq!(got.city.as_deref(), Some("Sydney"));
        assert_eq!(got.experience_level, ExperienceLevel::Expert);
        assert_eq!(got.units, Units::Imperial);
        assert_eq!(got.theme, Theme::Light);
        assert!(got.first_load_complete);
    }
}

fn enum_to_str<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap().trim_matches('"').to_string()
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str) -> T {
    let lower = s.to_ascii_lowercase();
    serde_json::from_str(&format!("\"{}\"", lower)).expect("valid enum value")
}

pub fn get(conn: &Connection) -> AppResult<Settings> {
    conn.execute(
        "INSERT OR IGNORE INTO settings (id) VALUES ('local-user')",
        [],
    )?;
    let (hem, city, exp, units, theme, flc, ver): (
        String,
        Option<String>,
        String,
        String,
        String,
        i64,
        i64,
    ) = conn.query_row(
        "SELECT hemisphere, city, experience_level, units, theme, first_load_complete, data_version
         FROM settings WHERE id = 'local-user'",
        [],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        },
    )?;
    Ok(Settings {
        id: "local-user".to_string(),
        first_load_complete: flc != 0,
        hemisphere: parse_enum(&hem),
        city,
        experience_level: parse_enum(&exp),
        units: parse_enum(&units),
        theme: parse_enum(&theme),
        data_version: ver,
    })
}

pub fn upsert(conn: &Connection, s: &Settings) -> AppResult<Settings> {
    let existing = get(conn)?;
    conn.execute(
        "INSERT INTO settings (id, first_load_complete, hemisphere, city, experience_level, units, theme, data_version)
         VALUES ('local-user', ?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
           first_load_complete = ?1, hemisphere = ?2, city = ?3,
           experience_level = ?4, units = ?5, theme = ?6,
           data_version = ?7, updated_at = datetime('now')",
        params![
            s.first_load_complete as i64,
            enum_to_str(&s.hemisphere),
            s.city,
            enum_to_str(&s.experience_level),
            enum_to_str(&s.units),
            enum_to_str(&s.theme),
            existing.data_version, // preserve bundled data version
        ],
    )?;
    get(conn)
}
