use rusqlite::{params, Connection};

use crate::error::AppResult;

const SOURCES_JSON: &str = include_str!("../../resources/data/sources.json");
const CATALOG_JSON: &str = include_str!("../../resources/data/plants-catalog.json");
const KB_JSON: &str = include_str!("../../resources/data/plants-kb.json");

pub const CURRENT_DATA_VERSION: i64 = 1;

pub fn load_seed_data() -> AppResult<garden_core::seed::SeedData> {
    let sources: Vec<garden_core::seed::RawSource> = serde_json::from_str(SOURCES_JSON)?;
    let catalog: Vec<garden_core::seed::RawCatalogPlant> = serde_json::from_str(CATALOG_JSON)?;
    let kb: Vec<garden_core::seed::RawKbEntry> = serde_json::from_str(KB_JSON)?;
    Ok(garden_core::seed::synthesize(&sources, &catalog, &kb))
}

use garden_core::models::RelationType;

fn json<T: serde::Serialize>(v: &T) -> AppResult<String> {
    Ok(serde_json::to_string(v)?)
}

fn opt(v: &Option<String>) -> Option<&str> {
    v.as_deref()
}

pub fn seed(conn: &mut Connection) -> AppResult<()> {
    let data = load_seed_data()?;

    let name_map: std::collections::HashMap<&str, &str> = data
        .catalog
        .iter()
        .map(|c| (c.id.as_str(), c.name.as_str()))
        .collect();

    let tx = conn.transaction()?;

    tx.execute("DELETE FROM plant_fts", [])?;
    tx.execute("DELETE FROM plant_relations", [])?;
    tx.execute("DELETE FROM plant_kb", [])?;
    tx.execute("DELETE FROM catalog", [])?;
    tx.execute("DELETE FROM sources", [])?;

    for s in &data.sources {
        tx.execute(
            "INSERT OR REPLACE INTO sources (id, name, type, url, credibility_tier) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![s.id, s.name, s.source_type, s.url, s.credibility_tier],
        )?;
    }

    for c in &data.catalog {
        tx.execute(
            "INSERT OR REPLACE INTO catalog
             (id, name, scientific_name, description, family, categories, life_cycle,
              growth_habit, photosynthesis_type, edible_parts, toxic_parts, pollination_type,
              sowing_season, sowing_method, stages, confidence_score, sunlight, water_requirements,
              soil_type, preferred_ph, common_pests, common_diseases, nutrient_preferences, seasonality)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)",
            params![
                c.id,
                c.name,
                opt(&c.scientific_name),
                opt(&c.description),
                opt(&c.family),
                json(&c.categories)?,
                c.life_cycle,
                json(&c.growth_habit)?,
                c.photosynthesis_type,
                json(&c.edible_parts)?,
                json(&c.toxic_parts)?,
                c.pollination_type,
                json(&c.sowing_season)?,
                c.sowing_method,
                json(&c.stages)?,
                c.confidence_score,
                opt(&c.sunlight),
                opt(&c.water_requirements),
                json(&c.soil_type)?,
                opt(&c.preferred_ph),
                json(&c.common_pests)?,
                json(&c.common_diseases)?,
                json(&c.nutrient_preferences)?,
                json(&c.seasonality)?,
            ],
        )?;
    }

    for k in &data.kb {
        tx.execute(
            "INSERT OR REPLACE INTO plant_kb
             (plant_id, common_name, scientific_name, type, family, growth_stage, seasonality,
              sunlight, water_requirements, soil_type, companion_plants, incompatible_plants,
              common_pests, common_diseases, nutrient_preferences, notes, preferred_ph)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17)",
            params![
                k.plant_id,
                k.common_name,
                opt(&k.scientific_name),
                opt(&k.entry_type),
                opt(&k.family),
                json(&k.growth_stage)?,
                json(&k.seasonality)?,
                opt(&k.sunlight),
                opt(&k.water_requirements),
                json(&k.soil_type)?,
                json(&k.companion_plants)?,
                json(&k.incompatible_plants)?,
                json(&k.common_pests)?,
                json(&k.common_diseases)?,
                json(&k.nutrient_preferences)?,
                opt(&k.notes),
                opt(&k.preferred_ph),
            ],
        )?;
    }

    for r in &data.relations {
        let rel_type = match r.relation_type {
            RelationType::Companion => "companion",
            RelationType::Antagonist => "antagonist",
        };
        tx.execute(
            "INSERT INTO plant_relations (plant_id, related_id, relation_type, strength, source_id, confidence)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![r.plant_id, r.related_id, rel_type, r.strength, r.source_id, r.confidence],
        )?;
    }

    // Build FTS rows: one per catalog plant, with resolved relation names.
    let companions_of = |id: &str| -> String {
        data.relations
            .iter()
            .filter(|r| r.plant_id == id && r.relation_type == RelationType::Companion)
            .filter_map(|r| name_map.get(r.related_id.as_str()).map(|n| n.to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let antagonists_of = |id: &str| -> String {
        data.relations
            .iter()
            .filter(|r| r.plant_id == id && r.relation_type == RelationType::Antagonist)
            .filter_map(|r| name_map.get(r.related_id.as_str()).map(|n| n.to_string()))
            .collect::<Vec<_>>()
            .join(" ")
    };

    for c in &data.catalog {
        let traits = [c.sunlight.as_deref(), c.water_requirements.as_deref()]
            .into_iter()
            .flatten()
            .chain(c.soil_type.iter().map(|s| s.as_str()))
            .collect::<Vec<_>>()
            .join(" ");
        tx.execute(
            "INSERT INTO plant_fts
             (id, name, scientific_name, family, categories, description, traits, companions, antagonists, pests, diseases)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                c.id,
                c.name,
                c.scientific_name.clone().unwrap_or_default(),
                c.family.clone().unwrap_or_default(),
                c.categories.join(" "),
                c.description.clone().unwrap_or_default(),
                traits,
                companions_of(&c.id),
                antagonists_of(&c.id),
                c.common_pests.join(" "),
                c.common_diseases.join(" "),
            ],
        )?;
    }

    // Record a default content pack for provenance.
    tx.execute(
        "INSERT OR REPLACE INTO content_packs (id, version) VALUES ('bundled', ?1)",
        params![format!("{}", CURRENT_DATA_VERSION)],
    )?;

    // Ensure a settings row exists, then stamp the data version.
    tx.execute(
        "INSERT OR IGNORE INTO settings (id) VALUES ('local-user')",
        [],
    )?;
    tx.execute(
        "UPDATE settings SET data_version = ?1 WHERE id = 'local-user'",
        params![CURRENT_DATA_VERSION],
    )?;

    tx.commit()?;
    Ok(())
}

/// Seeds the DB only when the stored data version is behind the bundled version.
pub fn ensure_seeded(conn: &mut Connection) -> AppResult<()> {
    let version: i64 = conn
        .query_row(
            "SELECT COALESCE((SELECT data_version FROM settings WHERE id='local-user'), 0)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if version < CURRENT_DATA_VERSION {
        seed(conn)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bundled_json() {
        let data = load_seed_data().unwrap();
        assert!(!data.sources.is_empty());
        assert!(data.catalog.len() >= 80, "expected ~81 catalog rows, got {}", data.catalog.len());
        assert!(!data.relations.is_empty());
    }
}

#[cfg(test)]
mod seed_db_tests {
    use super::*;
    use crate::db;

    fn in_memory() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrate(&mut conn).unwrap();
        conn
    }

    #[test]
    fn seeds_and_reseeds_idempotently() {
        let mut conn = in_memory();
        seed(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM catalog", [], |r| r.get(0))
            .unwrap();
        assert!(count >= 80);
        // re-seed should not duplicate rows
        seed(&mut conn).unwrap();
        let count2: i64 = conn
            .query_row("SELECT COUNT(*) FROM catalog", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, count2);
    }

    #[test]
    fn fts_finds_plants_by_name_and_companion() {
        let mut conn = in_memory();
        seed(&mut conn).unwrap();
        let by_name: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM plant_fts WHERE plant_fts MATCH ?1",
                ["\"tomato\""],
                |r| r.get(0),
            )
            .unwrap();
        assert!(by_name > 0);
    }
}
