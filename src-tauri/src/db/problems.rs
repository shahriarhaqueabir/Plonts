use rusqlite::{params, Connection};

#[cfg(test)]
use crate::db;
use crate::error::{AppError, AppResult};
use garden_core::models::{Problem, ProblemCause, Symptom};

const SYMPTOMS_JSON: &str = include_str!("../../resources/data/symptoms.json");

#[derive(Clone, serde::Deserialize)]
struct RawCause {
    id: String,
    name: String,
    description: String,
    treatment: String,
    prevention: String,
    organic: bool,
    image_asset_id: Option<String>,
}

fn enum_str<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap().trim_matches('"').to_string()
}

fn parse_symptom(s: &str) -> Symptom {
    serde_json::from_str(&format!("\"{}\"", s)).expect("valid symptom")
}

fn try_parse_symptom(s: &str) -> AppResult<Symptom> {
    serde_json::from_str::<Symptom>(&format!("\"{}\"", s))
        .map_err(|_| AppError::InvalidInput(format!("unknown symptom: {}", s)))
}

fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> T {
    serde_json::from_str(s).expect("valid json field")
}

fn causes_json(symptom: Symptom) -> AppResult<Vec<ProblemCause>> {
    let map: std::collections::BTreeMap<String, Vec<RawCause>> = serde_json::from_str(SYMPTOMS_JSON)?;
    let key = enum_str(&symptom);
    let causes = map.get(&key).cloned().unwrap_or_default();
    Ok(causes
        .into_iter()
        .map(|c| ProblemCause {
            id: c.id,
            name: c.name,
            description: c.description,
            treatment: c.treatment,
            prevention: c.prevention,
            organic: c.organic,
            image_asset_id: c.image_asset_id,
        })
        .collect())
}

/// Ranked causes for a symptom, optionally narrowed by a plant's known pests/diseases.
pub fn diagnose(conn: &Connection, symptom: &str, plant_id: Option<&str>) -> AppResult<Vec<ProblemCause>> {
    let symptom = try_parse_symptom(symptom)?;
    let causes = causes_json(symptom)?;
    match plant_id {
        Some(pid) => {
            let (pests, diseases): (String, String) = conn
                .query_row(
                    "SELECT common_pests, common_diseases FROM catalog WHERE id = ?1",
                    params![pid],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .map_err(|_| AppError::NotFound(pid.to_string()))?;
            Ok(garden_core::problems::rank_causes(
                causes,
                &parse_json::<Vec<String>>(&pests),
                &parse_json::<Vec<String>>(&diseases),
            ))
        }
        None => Ok(causes),
    }
}

const PROBLEM_SELECT: &str = "SELECT p.id, p.planting_id, c.name AS planting_name, p.symptom,
    p.affected_part, p.spread, p.cause, p.treatment, p.prevention, p.follow_up_at,
    p.resolved, p.resolution_note, p.created_at, p.resolved_at
    FROM problems p
    LEFT JOIN planted pl ON pl.id = p.planting_id
    LEFT JOIN catalog c ON c.id = pl.catalog_id";

fn row_to_problem(row: &rusqlite::Row) -> rusqlite::Result<Problem> {
    let symptom: String = row.get(3)?;
    Ok(Problem {
        id: row.get(0)?,
        planting_id: row.get(1)?,
        planting_name: row.get(2)?,
        symptom: parse_symptom(&symptom),
        affected_part: row.get(4)?,
        spread: row.get(5)?,
        cause: row.get(6)?,
        treatment: row.get(7)?,
        prevention: row.get(8)?,
        follow_up_at: row.get(9)?,
        resolved: row.get::<_, i64>(10)? != 0,
        resolution_note: row.get(11)?,
        created_at: row.get(12)?,
        resolved_at: row.get(13)?,
    })
}

pub fn get_problem(conn: &Connection, id: &str) -> AppResult<Problem> {
    conn.query_row(&format!("{} WHERE p.id = ?1", PROBLEM_SELECT), params![id], row_to_problem)
        .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn list_problems(conn: &Connection) -> AppResult<Vec<Problem>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY p.resolved ASC, p.created_at DESC", PROBLEM_SELECT))?;
    let rows = stmt.query_map([], row_to_problem)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

#[allow(clippy::too_many_arguments)]
pub fn create_problem(
    conn: &Connection,
    planting_id: Option<&str>,
    symptom: &str,
    affected_part: Option<&str>,
    spread: Option<&str>,
    cause: &str,
    treatment: &str,
    prevention: &str,
    follow_up_at: Option<&str>,
) -> AppResult<Problem> {
    let symptom = try_parse_symptom(symptom)?;
    if cause.trim().is_empty() || treatment.trim().is_empty() || prevention.trim().is_empty() {
        return Err(AppError::InvalidInput("cause, treatment and prevention are required".into()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO problems (id, planting_id, symptom, affected_part, spread, cause, treatment, prevention, follow_up_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, planting_id, enum_str(&symptom), affected_part, spread, cause, treatment, prevention, follow_up_at],
    )?;
    get_problem(conn, &id)
}

pub fn resolve_problem(conn: &Connection, id: &str, resolution_note: Option<&str>) -> AppResult<Problem> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE problems SET resolved = 1, resolution_note = ?2, resolved_at = ?3 WHERE id = ?1",
        params![id, resolution_note, now],
    )?;
    get_problem(conn, id)
}

pub fn delete_problem(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM problems WHERE id = ?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrate(&mut conn).unwrap();
        db::seed::seed(&mut conn).unwrap();
        conn
    }

    fn planting_id(conn: &Connection) -> String {
        let g = db::gardens::create_garden(conn, "Backyard", "raised_bed").unwrap();
        let bed = db::gardens::create_bed(conn, &g.id, "Bed 1", None, None, None, None, false).unwrap();
        db::planted::add_planting(conn, &bed.id, "plant_tomato", 1, "seed", None).unwrap().id
    }

    #[test]
    fn diagnose_returns_causes() {
        let conn = seeded();
        let causes = diagnose(&conn, "yellow_leaves", None).unwrap();
        assert!(causes.len() >= 3);
        assert!(causes.iter().all(|c| !c.name.is_empty() && !c.treatment.is_empty()));
    }

    #[test]
    fn diagnose_rejects_unknown_symptom() {
        let conn = seeded();
        assert!(diagnose(&conn, "purple_leaves", None).is_err());
    }

    #[test]
    fn create_resolve_delete_problem() {
        let conn = seeded();
        let pid = planting_id(&conn);
        let p = create_problem(
            &conn,
            Some(&pid),
            "holes",
            Some("leaves"),
            Some("spreading"),
            "Slugs and snails",
            "Hand-pick at dusk",
            "Water in the morning",
            Some("2026-07-01"),
        )
        .unwrap();
        assert_eq!(p.planting_name.as_deref(), Some("Tomato"));
        assert!(!p.resolved);
        assert_eq!(p.symptom, Symptom::Holes);
        let resolved = resolve_problem(&conn, &p.id, Some("Beer traps worked")).unwrap();
        assert!(resolved.resolved);
        assert_eq!(resolved.resolution_note.as_deref(), Some("Beer traps worked"));
        assert!(resolved.resolved_at.is_some());
        delete_problem(&conn, &p.id).unwrap();
        assert!(list_problems(&conn).unwrap().is_empty());
    }

    #[test]
    fn create_rejects_blank_cause() {
        let conn = seeded();
        assert!(create_problem(&conn, None, "spots", None, None, "  ", "treat", "prev", None).is_err());
    }

    #[test]
    fn unresolved_problems_list_first() {
        let conn = seeded();
        let a = create_problem(&conn, None, "wilting", None, None, "Underwatering", "Soak", "Mulch", None).unwrap();
        let b = create_problem(&conn, None, "spots", None, None, "Fungal leaf spot", "Remove leaves", "Water at base", None).unwrap();
        resolve_problem(&conn, &a.id, Some("done")).unwrap();
        let list = list_problems(&conn).unwrap();
        assert_eq!(list[0].id, b.id);
        assert!(list[1].resolved);
    }
}
