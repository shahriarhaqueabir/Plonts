use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Hemisphere {
    #[default]
    North,
    South,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExperienceLevel {
    #[default]
    Beginner,
    Learning,
    Confident,
    Expert,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Units {
    #[default]
    Metric,
    Imperial,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Companion,
    Antagonist,
}

/// A single growth stage in a plant's lifecycle (ported from PlantStage).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlantStage {
    pub id: String,
    pub name: String,
    pub duration_days: i64,
    pub water_frequency_days: i64,
    pub image_asset_id: String,
}

/// A month window with 1-based inclusive months (e.g. start=10, end=3 = wraps the year).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct MonthWindow {
    pub start_month: i32,
    pub end_month: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Seasonality {
    pub sowing: Vec<MonthWindow>,
    pub harvest: Vec<MonthWindow>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Settings {
    pub id: String,
    pub first_load_complete: bool,
    pub hemisphere: Hemisphere,
    pub city: Option<String>,
    pub experience_level: ExperienceLevel,
    pub units: Units,
    pub theme: Theme,
    pub data_version: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            id: "local-user".to_string(),
            first_load_complete: false,
            hemisphere: Hemisphere::North,
            city: None,
            experience_level: ExperienceLevel::Beginner,
            units: Units::Metric,
            theme: Theme::Dark,
            data_version: 0,
        }
    }
}

/// Compact plant row for list/search results.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlantSummary {
    pub id: String,
    pub name: String,
    pub scientific_name: Option<String>,
    pub family: Option<String>,
    pub categories: Vec<String>,
    pub life_cycle: Option<String>,
    pub sunlight: Option<String>,
    pub water_requirements: Option<String>,
    pub sowing_season: Vec<Season>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SourceCard {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
    pub credibility_tier: Option<String>,
}

/// Full plant detail (catalog + KB merged + resolved relations).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlantDetail {
    pub id: String,
    pub name: String,
    pub scientific_name: Option<String>,
    pub description: Option<String>,
    pub family: Option<String>,
    pub categories: Vec<String>,
    pub life_cycle: Option<String>,
    pub growth_habit: Vec<String>,
    pub photosynthesis_type: Option<String>,
    pub edible_parts: Vec<String>,
    pub toxic_parts: Vec<String>,
    pub pollination_type: Option<String>,
    pub sowing_method: Option<String>,
    pub stages: Vec<PlantStage>,
    pub seasonality: Seasonality,
    pub sunlight: Option<String>,
    pub water_requirements: Option<String>,
    pub soil_type: Vec<String>,
    pub preferred_ph: Option<String>,
    pub common_pests: Vec<String>,
    pub common_diseases: Vec<String>,
    pub nutrient_preferences: Vec<String>,
    pub companions: Vec<Relation>,
    pub antagonists: Vec<Relation>,
    pub sources: Vec<SourceCard>,
    pub confidence_score: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Relation {
    pub plant_id: String,
    pub related_id: String,
    pub related_name: String,
    pub relation_type: RelationType,
    pub strength: i64,
    pub source_id: Option<String>,
    pub confidence: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Garden {
    pub id: String,
    pub name: String,
    pub garden_type: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Bed {
    pub id: String,
    pub garden_id: String,
    pub name: String,
    pub soil_type: Option<String>,
    pub sun_exposure: Option<String>,
    pub width_cm: Option<i64>,
    pub height_cm: Option<i64>,
    pub is_container: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Planting {
    pub id: String,
    pub bed_id: String,
    pub bed_name: String,
    pub catalog_id: String,
    pub plant_name: String,
    pub quantity: i64,
    pub planted_at: String,
    pub source: String,
    pub notes: Option<String>,
    pub status: String,
    pub removed_at: Option<String>,
    pub removal_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SearchFilters {
    pub categories: Option<Vec<String>>,
    pub sunlight: Option<String>,
    pub life_cycle: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SearchResult {
    pub plant: PlantSummary,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CalendarEntry {
    pub plant: PlantSummary,
    pub eligible: bool,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WeatherSnapshot {
    pub city: String,
    pub temperature_c: Option<f64>,
    pub condition: Option<String>,
    pub fetched_at: Option<String>,
    pub cached: bool,
}

// ---------- Guides ----------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuideTopic {
    Soil,
    Composting,
    Pruning,
    Propagation,
    Pests,
    Seasonal,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SkillLevel {
    #[default]
    Beginner,
    Intermediate,
    Advanced,
}

/// A single step in a guide, with a checklist of tools/materials/actions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuideStep {
    pub id: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub checklist: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GuideProgress {
    pub guide_id: String,
    pub current_step: i64,
    pub completed: bool,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Guide {
    pub id: String,
    pub title: String,
    pub topic: GuideTopic,
    pub skill_level: SkillLevel,
    pub time_minutes: i64,
    pub steps: Vec<GuideStep>,
    pub materials: Vec<String>,
    pub illustrations: Vec<String>,
    pub version: i64,
    pub bookmarked: bool,
    pub progress: Option<GuideProgress>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GuideFilters {
    pub topic: Option<GuideTopic>,
    pub skill_level: Option<SkillLevel>,
    pub time_max_minutes: Option<i64>,
}

// ---------- Daily tips ----------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DailyTip {
    pub id: String,
    pub title: String,
    pub body: String,
    pub dismiss_until: Option<String>,
    pub snoozed_until: Option<String>,
}

// ---------- Journal ----------

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementKind {
    Height,
    Yield,
    Flowers,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JournalPhoto {
    pub id: String,
    pub entry_id: String,
    pub path: String,
    pub url: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Measurement {
    pub id: String,
    pub plant_id: String,
    pub plant_name: String,
    pub kind: MeasurementKind,
    pub value: f64,
    pub unit: String,
    pub recorded_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JournalEntry {
    pub id: String,
    pub plant_id: Option<String>,
    pub bed_id: Option<String>,
    pub plant_name: Option<String>,
    pub bed_name: Option<String>,
    pub text: String,
    pub tags: Vec<String>,
    pub photos: Vec<JournalPhoto>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct JournalFilters {
    pub query: Option<String>,
    pub plant_id: Option<String>,
    pub tag: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlantTimeline {
    pub plant_id: String,
    pub plant_name: String,
    pub measurements: Vec<Measurement>,
    pub entries: Vec<JournalEntry>,
}
