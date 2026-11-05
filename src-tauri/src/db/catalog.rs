use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use garden_core::models::{
    PlantDetail, PlantStage, PlantSummary, Relation, RelationType, SearchFilters, SearchResult,
    Seasonality, SourceCard,
};
use garden_core::fts::build_query;

#[cfg(test)]
use crate::db;

#[cfg(test)]
fn seeded() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    db::seed::seed(&mut conn).unwrap();
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_search_returns_all() {
        let conn = seeded();
        let res = search(&conn, "", &SearchFilters::default(), 200).unwrap();
        assert!(res.len() >= 80);
    }

    #[test]
    fn fts_search_finds_by_name() {
        let conn = seeded();
        let res = search(&conn, "tomato", &SearchFilters::default(), 50).unwrap();
        assert!(res.iter().any(|r| r.plant.name.to_lowercase().contains("tomato")));
    }

    #[test]
    fn filter_by_category() {
        let conn = seeded();
        let f = SearchFilters {
            categories: Some(vec!["herb".to_string()]),
            ..Default::default()
        };
        let res = search(&conn, "", &f, 200).unwrap();
        assert!(!res.is_empty());
        assert!(res.iter().all(|r| r.plant.categories.contains(&"herb".to_string())));
    }

    #[test]
    fn detail_has_relations_and_stages() {
        let conn = seeded();
        let d = get_plant_detail(&conn, "plant_apricot").unwrap();
        assert!(!d.stages.is_empty());
        assert!(!d.companions.is_empty());
        assert!(!d.sources.is_empty());
    }

    #[test]
    fn plant_image_meta_and_override_round_trip() {
        let conn = seeded();
        let meta = get_plant_image_meta(&conn, "plant_tomato").unwrap();
        assert_eq!(meta.asset_id.as_deref(), Some("tomato"));
        assert_eq!(meta.category.as_deref(), Some("vegetable"));
        assert!(meta.override_filename.is_none());

        set_plant_image_override(&conn, "plant_tomato", "abc.jpg").unwrap();
        assert_eq!(get_plant_image_meta(&conn, "plant_tomato").unwrap().override_filename.as_deref(), Some("abc.jpg"));

        let old = set_plant_image_override(&conn, "plant_tomato", "def.png").unwrap();
        assert_eq!(old.as_deref(), Some("abc.jpg"));

        let cleared = clear_plant_image_override(&conn, "plant_tomato").unwrap();
        assert_eq!(cleared.as_deref(), Some("def.png"));
        assert!(get_plant_image_meta(&conn, "plant_tomato").unwrap().override_filename.is_none());
    }

    #[test]
    fn unknown_plant_image_meta_errors() {
        let conn = seeded();
        assert!(get_plant_image_meta(&conn, "plant_nope").is_err());
    }
}

fn parse_json<T: serde::de::DeserializeOwned>(s: &str) -> T {
    serde_json::from_str(s).unwrap_or_else(|_| panic!("invalid json field"))
}

fn parse_opt_json<T: serde::de::DeserializeOwned>(s: Option<&str>) -> T {
    match s {
        Some(v) => parse_json(v),
        None => parse_json("null"),
    }
}

fn summary_from_row(row: &rusqlite::Row) -> rusqlite::Result<PlantSummary> {
    let categories_text: String = row.get(3)?;
    let sowing_text: String = row.get(4)?;
    Ok(PlantSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        scientific_name: row.get(2)?,
        categories: parse_json(&categories_text),
        sowing_season: parse_json(&sowing_text),
        family: row.get(5)?,
        life_cycle: row.get(6)?,
        sunlight: row.get(7)?,
        water_requirements: row.get(8)?,
    })
}

const SUMMARY_SELECT: &str = "SELECT id, name, scientific_name, categories, sowing_season,
    family, life_cycle, sunlight, water_requirements FROM catalog";

pub fn search(
    conn: &Connection,
    query: &str,
    filters: &SearchFilters,
    limit: usize,
) -> AppResult<Vec<SearchResult>> {
    let q = query.trim();
    let mut results: Vec<SearchResult> = if q.is_empty() {
        let mut stmt = conn.prepare(&format!("{} ORDER BY name", SUMMARY_SELECT))?;
        let rows = stmt.query_map([], |r| {
            Ok(SearchResult {
                plant: summary_from_row(r)?,
                score: 0.0,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    } else {
        let fts = build_query(q);
        let mut stmt = conn.prepare(
            "SELECT id, rank FROM plant_fts WHERE plant_fts MATCH ?1 ORDER BY rank LIMIT ?2",
        )?;
        let hits = stmt.query_map(params![fts, limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
        })?;
        let mut out = Vec::new();
        for hit in hits {
            let (id, rank) = hit?;
            let summary: PlantSummary = conn.query_row(
                &format!("{} WHERE id = ?1", SUMMARY_SELECT),
                [&id],
                summary_from_row,
            )?;
            out.push(SearchResult {
                plant: summary,
                score: -rank,
            });
        }
        out
    };

    if let Some(cats) = &filters.categories {
        results.retain(|r| cats.iter().any(|c| r.plant.categories.contains(c)));
    }
    if let Some(sun) = &filters.sunlight {
        results.retain(|r| r.plant.sunlight.as_deref() == Some(sun.as_str()));
    }
    if let Some(lc) = &filters.life_cycle {
        results.retain(|r| r.plant.life_cycle.as_deref() == Some(lc.as_str()));
    }

    results.truncate(limit);
    Ok(results)
}

pub fn list_categories(conn: &Connection) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT value FROM catalog, json_each(catalog.categories) ORDER BY value")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn list_sources(conn: &Connection) -> AppResult<Vec<SourceCard>> {
    let mut stmt = conn.prepare("SELECT id, name, url, credibility_tier FROM sources ORDER BY name")?;
    let rows = stmt.query_map([], |r| {
        Ok(SourceCard {
            id: r.get(0)?,
            name: r.get(1)?,
            url: r.get(2)?,
            credibility_tier: r.get(3)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn relations_for(conn: &Connection, plant_id: &str) -> AppResult<Vec<Relation>> {
    let mut stmt = conn.prepare(
        "SELECT pr.plant_id, pr.related_id, c.name, pr.relation_type, pr.strength, pr.source_id, pr.confidence
         FROM plant_relations pr
         JOIN catalog c ON c.id = pr.related_id
         WHERE pr.plant_id = ?1
         ORDER BY pr.relation_type, c.name",
    )?;
    let rows = stmt.query_map(params![plant_id], |r| {
        let rel_type: String = r.get(3)?;
        Ok(Relation {
            plant_id: r.get(0)?,
            related_id: r.get(1)?,
            related_name: r.get(2)?,
            relation_type: if rel_type == "antagonist" {
                RelationType::Antagonist
            } else {
                RelationType::Companion
            },
            strength: r.get(4)?,
            source_id: r.get(5)?,
            confidence: r.get(6)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_plant_detail(conn: &Connection, id: &str) -> AppResult<PlantDetail> {
    let row = conn
        .query_row(
            "SELECT id, name, scientific_name, description, family, categories, life_cycle,
                    growth_habit, photosynthesis_type, edible_parts, toxic_parts, pollination_type,
                    sowing_method, stages, seasonality, sunlight, water_requirements, soil_type,
                    preferred_ph, common_pests, common_diseases, nutrient_preferences, confidence_score
             FROM catalog WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, String>(9)?,
                    r.get::<_, String>(10)?,
                    r.get::<_, Option<String>>(11)?,
                    r.get::<_, Option<String>>(12)?,
                    r.get::<_, String>(13)?,
                    r.get::<_, Option<String>>(14)?,
                    r.get::<_, Option<String>>(15)?,
                    r.get::<_, Option<String>>(16)?,
                    r.get::<_, String>(17)?,
                    r.get::<_, Option<String>>(18)?,
                    r.get::<_, String>(19)?,
                    r.get::<_, String>(20)?,
                    r.get::<_, String>(21)?,
                    r.get::<_, Option<f64>>(22)?,
                ))
            },
        )
        .map_err(|_| AppError::NotFound(id.to_string()))?;

    let (
        pid, name, scientific_name, description, family, categories, life_cycle,
        growth_habit, photosynthesis_type, edible_parts, toxic_parts, pollination_type,
        sowing_method, stages, seasonality, sunlight, water_requirements, soil_type,
        preferred_ph, common_pests, common_diseases, nutrient_preferences, confidence_score,
    ) = row;

    let relations = relations_for(conn, &pid)?;
    let companions: Vec<Relation> = relations
        .iter()
        .filter(|r| r.relation_type == RelationType::Companion)
        .cloned()
        .collect();
    let antagonists: Vec<Relation> = relations
        .iter()
        .filter(|r| r.relation_type == RelationType::Antagonist)
        .cloned()
        .collect();
    let sources = list_sources(conn)?;

    Ok(PlantDetail {
        id: pid,
        name,
        scientific_name,
        description,
        family,
        categories: parse_json(&categories),
        life_cycle,
        growth_habit: parse_json(&growth_habit),
        photosynthesis_type,
        edible_parts: parse_json(&edible_parts),
        toxic_parts: parse_json(&toxic_parts),
        pollination_type,
        sowing_method,
        stages: parse_json::<Vec<PlantStage>>(&stages),
        seasonality: parse_opt_json::<Seasonality>(seasonality.as_deref()),
        sunlight,
        water_requirements,
        soil_type: parse_json(&soil_type),
        preferred_ph,
        common_pests: parse_json(&common_pests),
        common_diseases: parse_json(&common_diseases),
        nutrient_preferences: parse_json(&nutrient_preferences),
        companions,
        antagonists,
        sources,
        confidence_score,
    })
}

pub struct PlantImageMeta {
    pub asset_id: Option<String>,
    pub image_path: Option<String>,
    pub license: Option<String>,
    pub attribution: Option<String>,
    pub category: Option<String>,
    pub override_filename: Option<String>,
}

pub fn get_plant_image_meta(conn: &Connection, catalog_id: &str) -> AppResult<PlantImageMeta> {
    let row = conn
        .query_row(
            "SELECT image_asset_id, image_path, image_license, image_attribution, categories
             FROM catalog WHERE id = ?1",
            params![catalog_id],
            |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|_| AppError::NotFound(catalog_id.to_string()))?;
    let (asset_id, image_path, license, attribution, categories) = row;
    let categories: Vec<String> = parse_json(&categories);
    let category = categories.first().cloned();
    let override_filename = conn
        .query_row(
            "SELECT path FROM plant_image_overrides WHERE catalog_id = ?1",
            params![catalog_id],
            |r| r.get::<_, String>(0),
        )
        .ok();
    Ok(PlantImageMeta { asset_id, image_path, license, attribution, category, override_filename })
}

/// Upsert an override; returns the previous override filename (if any) for cleanup.
pub fn set_plant_image_override(conn: &Connection, catalog_id: &str, filename: &str) -> AppResult<Option<String>> {
    let previous = conn
        .query_row("SELECT path FROM plant_image_overrides WHERE catalog_id = ?1", params![catalog_id], |r| r.get::<_, String>(0))
        .ok();
    conn.execute(
        "INSERT INTO plant_image_overrides (catalog_id, path) VALUES (?1, ?2)
         ON CONFLICT(catalog_id) DO UPDATE SET path = ?2, created_at = datetime('now')",
        params![catalog_id, filename],
    )?;
    Ok(previous)
}

/// Remove an override; returns the removed filename (if any) for cleanup.
pub fn clear_plant_image_override(conn: &Connection, catalog_id: &str) -> AppResult<Option<String>> {
    let previous = conn
        .query_row("SELECT path FROM plant_image_overrides WHERE catalog_id = ?1", params![catalog_id], |r| r.get::<_, String>(0))
        .ok();
    conn.execute("DELETE FROM plant_image_overrides WHERE catalog_id = ?1", params![catalog_id])?;
    Ok(previous)
}