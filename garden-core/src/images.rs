use std::collections::HashMap;

use serde::Deserialize;

use crate::seed::CatalogRecord;

// ---------- Raw JSON shapes (parsed by src-tauri from bundled files) ----------

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RawSpeciesImage {
    pub plant_id: String,
    pub asset_id: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RawCategoryImage {
    pub category: String,
    pub asset_id: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RawPlantImages {
    #[serde(default)]
    pub species: Vec<RawSpeciesImage>,
    #[serde(default)]
    pub categories: Vec<RawCategoryImage>,
    #[serde(default)]
    pub generic_asset_id: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawAsset {
    pub path: String,
    pub license: String,
    pub attribution: String,
    #[serde(default)]
    pub source_url: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RawManifest {
    #[serde(default)]
    pub data_version: i64,
    #[serde(default)]
    pub assets: HashMap<String, RawAsset>,
}

// ---------- Resolved structures ----------

#[derive(Clone, Debug, Default)]
pub struct PlantImages {
    pub species: HashMap<String, String>,
    pub categories: HashMap<String, String>,
    pub generic_asset_id: String,
}

impl From<RawPlantImages> for PlantImages {
    fn from(raw: RawPlantImages) -> Self {
        Self {
            species: raw.species.into_iter().map(|s| (s.plant_id, s.asset_id)).collect(),
            categories: raw.categories.into_iter().map(|c| (c.category, c.asset_id)).collect(),
            generic_asset_id: raw.generic_asset_id.unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssetInfo {
    pub path: String,
    pub license: String,
    pub attribution: String,
    pub source_url: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ImageManifest {
    pub assets: HashMap<String, AssetInfo>,
}

impl From<RawManifest> for ImageManifest {
    fn from(raw: RawManifest) -> Self {
        Self {
            assets: raw.assets.into_iter().map(|(k, a)| (k, AssetInfo {
                path: a.path,
                license: a.license,
                attribution: a.attribution,
                source_url: a.source_url,
            })).collect(),
        }
    }
}

/// Fill species image fields on each catalog record that has a mapped asset.
pub fn assign_images(catalog: &mut [CatalogRecord], images: &PlantImages, manifest: &ImageManifest) {
    for rec in catalog.iter_mut() {
        if let Some(asset_id) = images.species.get(&rec.id) {
            if let Some(info) = manifest.assets.get(asset_id) {
                rec.image_asset_id = Some(asset_id.clone());
                rec.image_path = Some(info.path.clone());
                rec.image_license = Some(info.license.clone());
                rec.image_attribution = Some(info.attribution.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog_record(id: &str) -> CatalogRecord {
        CatalogRecord {
            id: id.to_string(),
            name: id.to_string(),
            scientific_name: None,
            description: None,
            family: None,
            categories: vec![],
            life_cycle: "annual".to_string(),
            growth_habit: vec![],
            photosynthesis_type: "C3".to_string(),
            edible_parts: vec![],
            toxic_parts: vec![],
            pollination_type: "insect".to_string(),
            sowing_season: vec![],
            sowing_method: "Direct".to_string(),
            stages: vec![],
            confidence_score: 1.0,
            sunlight: None,
            water_requirements: None,
            soil_type: vec![],
            preferred_ph: None,
            common_pests: vec![],
            common_diseases: vec![],
            nutrient_preferences: vec![],
            seasonality: crate::models::Seasonality::default(),
            companions: vec![],
            antagonists: vec![],
            image_asset_id: None,
            image_path: None,
            image_license: None,
            image_attribution: None,
        }
    }

    #[test]
    fn assign_images_fills_mapped_species() {
        let images = PlantImages {
            species: [("plant_tomato".to_string(), "tomato".to_string())].into_iter().collect(),
            categories: HashMap::new(),
            generic_asset_id: "generic_leaf".to_string(),
        };
        let manifest = ImageManifest {
            assets: [("tomato".to_string(), AssetInfo {
                path: "vegetable/tomato.svg".to_string(),
                license: "CC0".to_string(),
                attribution: "svgrepo".to_string(),
                source_url: Some("https://example.com/tomato.svg".to_string()),
            })].into_iter().collect(),
        };
        let mut catalog = vec![catalog_record("plant_tomato"), catalog_record("plant_basil")];
        assign_images(&mut catalog, &images, &manifest);

        let tomato = &catalog[0];
        assert_eq!(tomato.image_asset_id.as_deref(), Some("tomato"));
        assert_eq!(tomato.image_path.as_deref(), Some("vegetable/tomato.svg"));
        assert_eq!(tomato.image_license.as_deref(), Some("CC0"));
        assert_eq!(tomato.image_attribution.as_deref(), Some("svgrepo"));

        let basil = &catalog[1];
        assert!(basil.image_asset_id.is_none());
        assert!(basil.image_path.is_none());
    }

    #[test]
    fn unknown_asset_id_leaves_fields_none() {
        let images = PlantImages {
            species: [("plant_tomato".to_string(), "missing".to_string())].into_iter().collect(),
            categories: HashMap::new(),
            generic_asset_id: String::new(),
        };
        let manifest = ImageManifest::default();
        let mut catalog = vec![catalog_record("plant_tomato")];
        assign_images(&mut catalog, &images, &manifest);
        assert!(catalog[0].image_asset_id.is_none());
    }

    #[test]
    fn raw_plant_images_convert_to_lookup() {
        let raw = RawPlantImages {
            species: vec![RawSpeciesImage { plant_id: "plant_tomato".into(), asset_id: "tomato".into() }],
            categories: vec![RawCategoryImage { category: "vegetable".into(), asset_id: "cat_vegetable".into() }],
            generic_asset_id: Some("generic_leaf".into()),
        };
        let p: PlantImages = raw.into();
        assert_eq!(p.species.get("plant_tomato"), Some(&"tomato".to_string()));
        assert_eq!(p.categories.get("vegetable"), Some(&"cat_vegetable".to_string()));
        assert_eq!(p.generic_asset_id, "generic_leaf");
    }
}
