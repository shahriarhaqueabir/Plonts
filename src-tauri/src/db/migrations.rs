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
        M::up(
            r#"
            CREATE TABLE guides (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                topic TEXT NOT NULL,
                skill_level TEXT NOT NULL,
                time_minutes INTEGER NOT NULL DEFAULT 15,
                steps TEXT NOT NULL DEFAULT '[]',
                materials TEXT NOT NULL DEFAULT '[]',
                illustrations TEXT NOT NULL DEFAULT '[]',
                version INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE guide_progress (
                guide_id TEXT PRIMARY KEY,
                current_step INTEGER NOT NULL DEFAULT 0,
                completed INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE guide_bookmarks (
                guide_id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE journal_entries (
                id TEXT PRIMARY KEY,
                plant_id TEXT,
                bed_id TEXT,
                text TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_journal_plant ON journal_entries(plant_id);
            CREATE INDEX idx_journal_bed ON journal_entries(bed_id);

            CREATE TABLE journal_photos (
                id TEXT PRIMARY KEY,
                entry_id TEXT NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
                path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_photos_entry ON journal_photos(entry_id);

            CREATE TABLE measurements (
                id TEXT PRIMARY KEY,
                plant_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                value REAL NOT NULL,
                unit TEXT NOT NULL,
                recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_measurements_plant ON measurements(plant_id);

            CREATE TABLE daily_tips (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                dismiss_until TEXT,
                snoozed_until TEXT
            );
            "#,
        ),
        M::up(
            r#"
            CREATE TABLE problems (
                id TEXT PRIMARY KEY,
                planting_id TEXT,
                symptom TEXT NOT NULL,
                affected_part TEXT,
                spread TEXT,
                cause TEXT NOT NULL,
                treatment TEXT NOT NULL,
                prevention TEXT NOT NULL,
                follow_up_at TEXT,
                resolved INTEGER NOT NULL DEFAULT 0,
                resolution_note TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                resolved_at TEXT
            );
            CREATE INDEX idx_problems_planting ON problems(planting_id);

            CREATE TABLE seeds (
                id TEXT PRIMARY KEY,
                catalog_id TEXT,
                variety TEXT NOT NULL,
                source TEXT NOT NULL,
                year INTEGER NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                germination_notes TEXT,
                viability_warned INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_seeds_catalog ON seeds(catalog_id);

            CREATE TABLE harvests (
                id TEXT PRIMARY KEY,
                planting_id TEXT,
                catalog_id TEXT,
                date TEXT NOT NULL,
                weight_g REAL,
                count INTEGER,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_harvests_planting ON harvests(planting_id);
            CREATE INDEX idx_harvests_catalog ON harvests(catalog_id);

            CREATE TABLE shopping_items (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT 'other',
                quantity TEXT,
                checked INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE tools (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                last_maintained TEXT,
                next_maintenance TEXT,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        ),
        M::up(
            r#"
            ALTER TABLE catalog ADD COLUMN image_asset_id TEXT;
            ALTER TABLE catalog ADD COLUMN image_path TEXT;
            ALTER TABLE catalog ADD COLUMN image_license TEXT;
            ALTER TABLE catalog ADD COLUMN image_attribution TEXT;

            CREATE TABLE plant_image_overrides (
                catalog_id TEXT PRIMARY KEY REFERENCES catalog(id) ON DELETE CASCADE,
                path TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        ),
        M::up(
            r#"
            CREATE TABLE layouts (
                id TEXT PRIMARY KEY,
                garden_id TEXT NOT NULL REFERENCES gardens(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                canvas_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_layouts_garden ON layouts(garden_id);

            CREATE TABLE layout_plants (
                id TEXT PRIMARY KEY,
                layout_id TEXT NOT NULL REFERENCES layouts(id) ON DELETE CASCADE,
                catalog_id TEXT NOT NULL REFERENCES catalog(id),
                bed_id TEXT,
                x REAL NOT NULL,
                y REAL NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_layout_plants_layout ON layout_plants(layout_id);

            CREATE TABLE rotation_plans (
                id TEXT PRIMARY KEY,
                garden_id TEXT NOT NULL REFERENCES gardens(id) ON DELETE CASCADE,
                season TEXT NOT NULL,
                plant_family TEXT NOT NULL,
                notes TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX idx_rotation_garden ON rotation_plans(garden_id);
            "#,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn migration_four_adds_image_columns_and_override_table() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations().to_latest(&mut conn).unwrap();

        let cols: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA table_info(catalog)").unwrap();
            let rows = stmt.query_map([], |r| r.get::<_, String>(1)).unwrap();
            rows.collect::<Result<_, _>>().unwrap()
        };
        for c in ["image_asset_id", "image_path", "image_license", "image_attribution"] {
            assert!(cols.iter().any(|x| x == c), "missing catalog column {c}");
        }

        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='plant_image_overrides'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn migration_five_adds_layout_tables() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations().to_latest(&mut conn).unwrap();

        for table in ["layouts", "layout_plants", "rotation_plans"] {
            let n: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "missing table {table}");
        }
    }
}
