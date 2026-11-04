use serde::Deserialize;
use std::collections::HashMap;

use crate::models::{MonthWindow, PlantStage, RelationType, Season, Seasonality};
use crate::seasonal::month_from_name;

// ---------- Raw JSON shapes ----------

#[derive(Deserialize, Clone, Debug)]
pub struct RawSource {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub source_type: String,
    pub url: Option<String>,
    pub credibility_tier: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum MonthVal {
    Num(i32),
    Str(String),
}

impl MonthVal {
    fn to_month(&self) -> Option<i32> {
        match self {
            MonthVal::Num(n) => Some(*n),
            MonthVal::Str(s) => month_from_name(s),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawWindow {
    pub start_month: MonthVal,
    pub end_month: MonthVal,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(t) => vec![t],
            OneOrMany::Many(v) => v,
        }
    }
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RawSeasonality {
    #[serde(default)]
    pub sowing: Option<OneOrMany<RawWindow>>,
    #[serde(default)]
    pub harvest: Option<OneOrMany<RawWindow>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawRequirements {
    pub sunlight: Option<String>,
    pub water_requirements: Option<String>,
    pub soil_ph: Option<String>,
    pub soil_type: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawCatalogPlant {
    pub id: String,
    pub name: String,
    pub scientific_name: Option<String>,
    pub family: Option<String>,
    pub plant_type: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub companions: Vec<String>,
    #[serde(default)]
    pub antagonists: Vec<String>,
    pub seasonality: Option<RawSeasonality>,
    pub requirements: Option<RawRequirements>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawKbEntry {
    pub plant_id: Option<String>,
    pub id: Option<String>,
    pub common_name: Option<String>,
    pub scientific_name: Option<String>,
    #[serde(rename = "type")]
    pub entry_type: Option<String>,
    pub family: Option<String>,
    #[serde(default)]
    pub soil_type: Vec<String>,
    pub sunlight: Option<String>,
    pub water_requirements: Option<String>,
    #[serde(default)]
    pub nutrient_preferences: Vec<String>,
    #[serde(default)]
    pub growth_stage: Vec<String>,
    pub seasonality: Option<RawSeasonality>,
    #[serde(default)]
    pub companion_plants: Vec<String>,
    #[serde(default)]
    pub incompatible_plants: Vec<String>,
    #[serde(default)]
    pub common_pests: Vec<String>,
    #[serde(default)]
    pub common_diseases: Vec<String>,
    pub notes: Option<String>,
    pub preferred_ph: Option<String>,
}

// ---------- Output records (consumed by src-tauri for INSERT) ----------

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogRecord {
    pub id: String,
    pub name: String,
    pub scientific_name: Option<String>,
    pub description: Option<String>,
    pub family: Option<String>,
    pub categories: Vec<String>,
    pub life_cycle: String,
    pub growth_habit: Vec<String>,
    pub photosynthesis_type: String,
    pub edible_parts: Vec<String>,
    pub toxic_parts: Vec<String>,
    pub pollination_type: String,
    pub sowing_season: Vec<Season>,
    pub sowing_method: String,
    pub stages: Vec<PlantStage>,
    pub confidence_score: f64,
    pub sunlight: Option<String>,
    pub water_requirements: Option<String>,
    pub soil_type: Vec<String>,
    pub preferred_ph: Option<String>,
    pub common_pests: Vec<String>,
    pub common_diseases: Vec<String>,
    pub nutrient_preferences: Vec<String>,
    pub seasonality: Seasonality,
    pub companions: Vec<String>,
    pub antagonists: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KbRecord {
    pub plant_id: String,
    pub common_name: String,
    pub scientific_name: Option<String>,
    pub entry_type: Option<String>,
    pub family: Option<String>,
    pub soil_type: Vec<String>,
    pub sunlight: Option<String>,
    pub water_requirements: Option<String>,
    pub nutrient_preferences: Vec<String>,
    pub growth_stage: Vec<String>,
    pub seasonality: Seasonality,
    pub companion_plants: Vec<String>,
    pub incompatible_plants: Vec<String>,
    pub common_pests: Vec<String>,
    pub common_diseases: Vec<String>,
    pub notes: Option<String>,
    pub preferred_ph: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationRecord {
    pub plant_id: String,
    pub related_id: String,
    pub relation_type: RelationType,
    pub strength: i64,
    pub source_id: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRecord {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub url: Option<String>,
    pub credibility_tier: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SeedData {
    pub sources: Vec<SourceRecord>,
    pub catalog: Vec<CatalogRecord>,
    pub kb: Vec<KbRecord>,
    pub relations: Vec<RelationRecord>,
}

// ---------- Normalization ----------

pub fn normalize_seasonality(raw: &RawSeasonality) -> Seasonality {
    let windows = |field: &Option<OneOrMany<RawWindow>>| -> Vec<MonthWindow> {
        field
            .as_ref()
            .map(|o| o.clone().into_vec())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|w| {
                Some(MonthWindow {
                    start_month: w.start_month.to_month()?,
                    end_month: w.end_month.to_month()?,
                })
            })
            .collect()
    };
    Seasonality {
        sowing: windows(&raw.sowing),
        harvest: windows(&raw.harvest),
    }
}

// ---------- Stage defaults (ported from generateDefaultStages) ----------

fn stage(id: &str, name: &str, days: i64, water: i64) -> PlantStage {
    PlantStage {
        id: id.to_string(),
        name: name.to_string(),
        duration_days: days,
        water_frequency_days: water,
        image_asset_id: format!("{}_generic", id),
    }
}

pub fn generate_default_stages(category: &str, life_cycle: &str) -> Vec<PlantStage> {
    let cat = category.to_lowercase();
    let is_tree =
        cat.contains("tree") || cat == "fruit" || life_cycle.to_lowercase() == "perennial";
    let is_herb = cat == "herb";

    if is_tree {
        vec![
            stage("germination", "Germination", 30, 3),
            stage("seedling", "Sapling Phase", 365, 7),
            stage("vegetative", "Active Growth", 730, 10),
            stage("flowering", "Bloom Cycle", 45, 5),
            stage("fruiting", "Fruit Set", 90, 4),
            stage("harvest", "Harvest Window", 30, 5),
            stage("dormant", "Dormancy", 120, 14),
        ]
    } else if is_herb {
        vec![
            stage("germination", "Germination", 10, 1),
            stage("seedling", "Establishment", 14, 2),
            stage("vegetative", "Foliage Growth", 60, 3),
            stage("harvest", "Pruning / Harvest", 90, 3),
        ]
    } else {
        vec![
            stage("germination", "Germination", 7, 1),
            stage("seedling", "Seedling Phase", 14, 2),
            stage("vegetative", "Vegetative Growth", 35, 3),
            stage("flowering", "Flowering Stage", 14, 2),
            stage("fruiting", "Fruiting", 30, 3),
            stage("harvest", "Peak Harvest", 21, 2),
        ]
    }
}

// ---------- Synthesis ----------

fn kb_lookup(entries: &[RawKbEntry]) -> HashMap<String, RawKbEntry> {
    entries
        .iter()
        .filter_map(|e| {
            let key = e.plant_id.clone().or_else(|| e.id.clone())?;
            Some((key, e.clone()))
        })
        .collect()
}

fn title_case(id: &str) -> String {
    let mut chars = id.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn synthesize_catalog(raw: &RawCatalogPlant, kb: Option<&RawKbEntry>) -> CatalogRecord {
    let req = raw.requirements.clone().unwrap_or(RawRequirements {
        sunlight: None,
        water_requirements: None,
        soil_ph: None,
        soil_type: None,
    });
    let category = raw
        .plant_type
        .clone()
        .or_else(|| kb.as_ref().and_then(|k| k.entry_type.clone()))
        .unwrap_or_else(|| "vegetable".to_string());
    let life_cycle = "annual".to_string(); // catalog JSON carries no life_cycle; default like the old app

    let stages = match kb {
        Some(k) if !k.growth_stage.is_empty() => k
            .growth_stage
            .iter()
            .map(|id| stage(id, &title_case(id), 20, 3))
            .collect(),
        _ => generate_default_stages(&category, &life_cycle),
    };

    let sunlight = req
        .sunlight
        .clone()
        .or_else(|| kb.as_ref().and_then(|k| k.sunlight.clone()));

    let water = req
        .water_requirements
        .clone()
        .or_else(|| kb.as_ref().and_then(|k| k.water_requirements.clone()));

    let soil_type = kb
        .as_ref()
        .map(|k| k.soil_type.clone())
        .filter(|v| !v.is_empty())
        .or_else(|| req.soil_type.clone().map(|s| vec![s]))
        .unwrap_or_default();

    let preferred_ph = req
        .soil_ph
        .clone()
        .or_else(|| kb.as_ref().and_then(|k| k.preferred_ph.clone()));

    let seasonality = raw
        .seasonality
        .as_ref()
        .map(normalize_seasonality)
        .or_else(|| {
            kb.as_ref()
                .and_then(|k| k.seasonality.as_ref())
                .map(normalize_seasonality)
        })
        .unwrap_or_default();

    CatalogRecord {
        id: raw.id.clone(),
        name: raw.name.clone(),
        scientific_name: raw
            .scientific_name
            .clone()
            .or_else(|| kb.as_ref().and_then(|k| k.scientific_name.clone())),
        description: raw
            .notes
            .clone()
            .or_else(|| kb.as_ref().and_then(|k| k.notes.clone())),
        family: raw
            .family
            .clone()
            .or_else(|| kb.as_ref().and_then(|k| k.family.clone())),
        categories: vec![category],
        life_cycle,
        growth_habit: vec!["bushy".to_string()],
        photosynthesis_type: "C3".to_string(),
        edible_parts: vec![],
        toxic_parts: vec![],
        pollination_type: "insect".to_string(),
        sowing_season: vec![],
        sowing_method: "Direct".to_string(),
        stages,
        confidence_score: 0.95,
        sunlight,
        water_requirements: water,
        soil_type,
        preferred_ph,
        common_pests: kb.as_ref().map(|k| k.common_pests.clone()).unwrap_or_default(),
        common_diseases: kb
            .as_ref()
            .map(|k| k.common_diseases.clone())
            .unwrap_or_default(),
        nutrient_preferences: kb
            .as_ref()
            .map(|k| k.nutrient_preferences.clone())
            .unwrap_or_default(),
        seasonality,
        companions: raw.companions.clone(),
        antagonists: raw.antagonists.clone(),
    }
}

pub fn synthesize(
    sources: &[RawSource],
    catalog: &[RawCatalogPlant],
    kb: &[RawKbEntry],
) -> SeedData {
    let lookup = kb_lookup(kb);

    let source_records: Vec<SourceRecord> = sources
        .iter()
        .map(|s| SourceRecord {
            id: s.id.clone(),
            name: s.name.clone(),
            source_type: s.source_type.clone(),
            url: s.url.clone(),
            credibility_tier: s.credibility_tier.clone(),
        })
        .collect();

    let catalog_records: Vec<CatalogRecord> = catalog
        .iter()
        .map(|raw| synthesize_catalog(raw, lookup.get(&raw.id)))
        .filter(|r| !r.id.is_empty() && !r.name.is_empty() && !r.stages.is_empty())
        .collect();

    let kb_records: Vec<KbRecord> = catalog_records
        .iter()
        .map(|c| KbRecord {
            plant_id: c.id.clone(),
            common_name: c.name.clone(),
            scientific_name: c.scientific_name.clone(),
            entry_type: c.categories.first().cloned(),
            family: c.family.clone(),
            soil_type: c.soil_type.clone(),
            sunlight: c.sunlight.clone(),
            water_requirements: c.water_requirements.clone(),
            nutrient_preferences: c.nutrient_preferences.clone(),
            growth_stage: c.stages.iter().map(|s| s.id.clone()).collect(),
            seasonality: c.seasonality.clone(),
            companion_plants: c.companions.clone(),
            incompatible_plants: c.antagonists.clone(),
            common_pests: c.common_pests.clone(),
            common_diseases: c.common_diseases.clone(),
            notes: c.description.clone(),
            preferred_ph: c.preferred_ph.clone(),
        })
        .collect();

    let mut relations: Vec<RelationRecord> = Vec::new();
    for c in &catalog_records {
        for related in &c.companions {
            relations.push(RelationRecord {
                plant_id: c.id.clone(),
                related_id: related.clone(),
                relation_type: RelationType::Companion,
                strength: 1,
                source_id: None,
                confidence: 0.95,
            });
        }
        for related in &c.antagonists {
            relations.push(RelationRecord {
                plant_id: c.id.clone(),
                related_id: related.clone(),
                relation_type: RelationType::Antagonist,
                strength: 1,
                source_id: None,
                confidence: 0.95,
            });
        }
    }

    SeedData {
        sources: source_records,
        catalog: catalog_records,
        kb: kb_records,
        relations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_catalog() -> RawCatalogPlant {
        RawCatalogPlant {
            id: "plant_apricot".into(),
            name: "Apricot".into(),
            scientific_name: Some("Prunus armeniaca".into()),
            family: Some("Rosaceae".into()),
            plant_type: Some("fruit_tree".into()),
            notes: Some("Early bloomer.".into()),
            companions: vec!["plant_basil".into(), "plant_garlic".into()],
            antagonists: vec!["plant_tomato".into()],
            seasonality: Some(RawSeasonality {
                sowing: Some(OneOrMany::Many(vec![RawWindow {
                    start_month: MonthVal::Num(11),
                    end_month: MonthVal::Num(3),
                }])),
                harvest: None,
            }),
            requirements: Some(RawRequirements {
                sunlight: Some("full_sun".into()),
                water_requirements: None,
                soil_ph: Some("6.0-7.8".into()),
                soil_type: Some("loamy_well_drained".into()),
            }),
        }
    }

    #[test]
    fn normalizes_array_and_single_windows() {
        let raw = raw_catalog();
        let seas = normalize_seasonality(raw.seasonality.as_ref().unwrap());
        assert_eq!(seas.sowing.len(), 1);
        assert_eq!(seas.sowing[0].start_month, 11);
        assert_eq!(seas.sowing[0].end_month, 3);
    }

    #[test]
    fn normalizes_month_names() {
        let raw = RawSeasonality {
            sowing: Some(OneOrMany::One(RawWindow {
                start_month: MonthVal::Str("April".into()),
                end_month: MonthVal::Str("June".into()),
            })),
            harvest: None,
        };
        let seas = normalize_seasonality(&raw);
        assert_eq!(seas.sowing[0].start_month, 4);
        assert_eq!(seas.sowing[0].end_month, 6);
    }

    #[test]
    fn tree_gets_dormant_stage() {
        let stages = generate_default_stages("fruit_tree", "annual");
        assert!(stages.iter().any(|s| s.id == "dormant"));
        assert_eq!(stages.len(), 7);
    }

    #[test]
    fn vegetable_gets_six_stages() {
        let stages = generate_default_stages("vegetable", "annual");
        assert_eq!(stages.len(), 6);
    }

    #[test]
    fn synthesizes_apricot_fields() {
        let rec = synthesize_catalog(&raw_catalog(), None);
        assert_eq!(rec.categories, vec!["fruit_tree".to_string()]);
        assert_eq!(rec.sunlight.as_deref(), Some("full_sun"));
        assert_eq!(rec.preferred_ph.as_deref(), Some("6.0-7.8"));
        assert_eq!(rec.soil_type, vec!["loamy_well_drained".to_string()]);
        assert_eq!(rec.seasonality.sowing.len(), 1);
        assert!(rec.stages.iter().any(|s| s.id == "dormant"));
    }

    #[test]
    fn extracts_companion_and_antagonist_relations() {
        let data = synthesize(&[], &[raw_catalog()], &[]);
        let companions: Vec<_> = data
            .relations
            .iter()
            .filter(|r| r.relation_type == RelationType::Companion)
            .map(|r| r.related_id.clone())
            .collect();
        assert_eq!(
            companions,
            vec!["plant_basil".to_string(), "plant_garlic".to_string()]
        );
        assert_eq!(data.relations.len(), 3); // 2 companions + 1 antagonist
    }
}
