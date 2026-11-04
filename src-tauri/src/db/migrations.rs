use rusqlite_migration::{M, Migrations};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            r#"
            CREATE TABLE sources (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                url TEXT,
                credibility_tier TEXT
            );

            CREATE TABLE catalog (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                scientific_name TEXT,
                description TEXT,
                family TEXT,
                categories TEXT NOT NULL DEFAULT '[]',
                life_cycle TEXT,
                growth_habit TEXT NOT NULL DEFAULT '[]',
                photosynthesis_type TEXT,
                edible_parts TEXT NOT NULL DEFAULT '[]',
                toxic_parts TEXT NOT NULL DEFAULT '[]',
                pollination_type TEXT,
                sowing_season TEXT NOT NULL DEFAULT '[]',
                sowing_method TEXT,
                stages TEXT NOT NULL DEFAULT '[]',
                confidence_score REAL,
                sunlight TEXT,
                water_requirements TEXT,
                soil_type TEXT NOT NULL DEFAULT '[]',
                preferred_ph TEXT,
                common_pests TEXT NOT NULL DEFAULT '[]',
                common_diseases TEXT NOT NULL DEFAULT '[]',
                nutrient_preferences TEXT NOT NULL DEFAULT '[]',
                seasonality TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE plant_kb (
                plant_id TEXT PRIMARY KEY,
                common_name TEXT NOT NULL,
                scientific_name TEXT,
                type TEXT,
                family TEXT,
                growth_stage TEXT NOT NULL DEFAULT '[]',
                sowing_season TEXT NOT NULL DEFAULT '[]',
                seasonality TEXT,
                sunlight TEXT,
                water_requirements TEXT,
                soil_type TEXT NOT NULL DEFAULT '[]',
                companion_plants TEXT NOT NULL DEFAULT '[]',
                incompatible_plants TEXT NOT NULL DEFAULT '[]',
                common_pests TEXT NOT NULL DEFAULT '[]',
                common_diseases TEXT NOT NULL DEFAULT '[]',
                nutrient_preferences TEXT NOT NULL DEFAULT '[]',
                notes TEXT,
                preferred_ph TEXT
            );

            CREATE TABLE plant_relations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plant_id TEXT NOT NULL,
                related_id TEXT NOT NULL,
                relation_type TEXT NOT NULL,
                mechanism TEXT NOT NULL DEFAULT '',
                strength INTEGER NOT NULL DEFAULT 1,
                source_id TEXT,
                confidence REAL NOT NULL DEFAULT 0.95
            );
            CREATE INDEX idx_relations_plant ON plant_relations(plant_id);
            CREATE INDEX idx_relations_related ON plant_relations(related_id);

            CREATE TABLE content_packs (
                id TEXT PRIMARY KEY,
                version TEXT NOT NULL,
                installed_at TEXT NOT NULL DEFAULT (datetime('now')),
                path TEXT
            );

            CREATE TABLE gardens (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                garden_type TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE beds (
                id TEXT PRIMARY KEY,
                garden_id TEXT NOT NULL REFERENCES gardens(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                soil_type TEXT,
                sun_exposure TEXT,
                width_cm INTEGER,
                height_cm INTEGER,
                is_container INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_beds_garden ON beds(garden_id);

            CREATE TABLE planted (
                id TEXT PRIMARY KEY,
                bed_id TEXT NOT NULL REFERENCES beds(id) ON DELETE CASCADE,
                catalog_id TEXT NOT NULL REFERENCES catalog(id),
                quantity INTEGER NOT NULL DEFAULT 1,
                planted_at TEXT NOT NULL DEFAULT (datetime('now')),
                source TEXT NOT NULL DEFAULT 'seed',
                notes TEXT,
                status TEXT NOT NULL DEFAULT 'growing',
                removed_at TEXT,
                removal_reason TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_planted_bed ON planted(bed_id);

            CREATE TABLE settings (
                id TEXT PRIMARY KEY,
                first_load_complete INTEGER NOT NULL DEFAULT 0,
                hemisphere TEXT NOT NULL DEFAULT 'North',
                city TEXT,
                experience_level TEXT NOT NULL DEFAULT 'beginner',
                units TEXT NOT NULL DEFAULT 'metric',
                theme TEXT NOT NULL DEFAULT 'dark',
                data_version INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE weather_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                city TEXT NOT NULL,
                latitude REAL,
                longitude REAL,
                payload TEXT NOT NULL,
                fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_weather_city ON weather_cache(city);

            CREATE VIRTUAL TABLE plant_fts USING fts5(
                id UNINDEXED,
                name,
                scientific_name,
                family,
                categories,
                description,
                traits,
                companions,
                antagonists,
                pests,
                diseases
            );
            "#,
        ),
    ])
}
