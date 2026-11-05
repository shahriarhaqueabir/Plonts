use rusqlite::{params, Connection};

use crate::error::AppResult;
use garden_core::fts::build_query;
use garden_core::RetrievedChunk;

pub fn excerpt(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    let slice = &s[..end];
    match slice.rfind(char::is_whitespace) {
        Some(pos) if pos > 0 => format!("{}...", slice[..pos].trim_end()),
        _ => format!("{}...", slice),
    }
}

fn first_step_text(steps_json: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(steps_json).unwrap_or(serde_json::Value::Null);
    if let Some(arr) = v.as_array() {
        if let Some(first) = arr.first() {
            if let Some(body) = first.get("body").and_then(|x| x.as_str()) {
                if !body.trim().is_empty() {
                    return body.to_string();
                }
            }
            if let Some(title) = first.get("title").and_then(|x| x.as_str()) {
                return title.to_string();
            }
        }
    }
    String::new()
}

pub fn retrieve(conn: &Connection, query: &str, limit: usize) -> AppResult<Vec<RetrievedChunk>> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let fts = build_query(q);
    if fts.is_empty() {
        return Ok(Vec::new());
    }
    if limit == 0 {
        return Ok(Vec::new());
    }

    let mut plants: Vec<RetrievedChunk> = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id FROM plant_fts WHERE plant_fts MATCH ?1 ORDER BY rank")?;
        let ids: Vec<String> = stmt
            .query_map(params![fts], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for id in ids {
            if let Ok((name, desc)) = conn.query_row(
                "SELECT name, COALESCE(description, '') FROM catalog WHERE id = ?1",
                params![id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            ) {
                plants.push(RetrievedChunk {
                    id,
                    title: name,
                    kind: "plant".to_string(),
                    excerpt: excerpt(&desc, 160),
                    url: None,
                });
            }
        }
    }

    let mut guides: Vec<RetrievedChunk> = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id FROM guides_fts WHERE guides_fts MATCH ?1 ORDER BY rank")?;
        let ids: Vec<String> = stmt
            .query_map(params![fts], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for id in ids {
            if let Ok((title, steps_json)) = conn.query_row(
                "SELECT title, steps FROM guides WHERE id = ?1",
                params![id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            ) {
                let first = first_step_text(&steps_json);
                guides.push(RetrievedChunk {
                    id,
                    title,
                    kind: "guide".to_string(),
                    excerpt: excerpt(&first, 160),
                    url: None,
                });
            }
        }
    }

    let mut out: Vec<RetrievedChunk> = Vec::with_capacity(limit);
    let mut i = 0usize;
    while out.len() < limit && (i < plants.len() || i < guides.len()) {
        if i < plants.len() && out.len() < limit {
            out.push(plants[i].clone());
        }
        if i < guides.len() && out.len() < limit {
            out.push(guides[i].clone());
        }
        i += 1;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrate(&mut conn).unwrap();
        db::seed::seed(&mut conn).unwrap();
        crate::db::guides::seed_guides(&mut conn).unwrap();
        conn
    }

    #[test]
    fn empty_query_returns_empty() {
        let conn = seeded();
        assert!(retrieve(&conn, "", 6).unwrap().is_empty());
        assert!(retrieve(&conn, "   ", 6).unwrap().is_empty());
    }

    #[test]
    fn tomato_returns_plant_chunk() {
        let conn = seeded();
        let chunks = retrieve(&conn, "tomato", 6).unwrap();
        assert!(chunks.iter().any(|c| c.kind == "plant"), "no plant chunk for tomato");
    }

    #[test]
    fn soil_returns_guide_chunk() {
        let conn = seeded();
        let chunks = retrieve(&conn, "soil", 6).unwrap();
        assert!(chunks.iter().any(|c| c.kind == "guide"), "no guide chunk for soil: {:?}", chunks);
    }

    #[test]
    fn excerpt_trims_to_word_boundary() {
        let s = "hello world this is a long sentence";
        let e = excerpt(s, 12);
        assert!(e.ends_with("..."));
        assert!(e.len() <= 16);
    }

    #[test]
    fn excerpt_short_unchanged() {
        assert_eq!(excerpt("hi", 10), "hi");
    }
}