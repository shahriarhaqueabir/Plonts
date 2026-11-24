use std::path::Path;

use garden_core::images::{ImageManifest, PlantImages};
use garden_core::models::{Credit, ImageAttribution, PlantImage};

use crate::error::AppResult;

const PLANT_IMAGES_JSON: &str = include_str!("../resources/data/plant-images.json");
const MANIFEST_JSON: &str = include_str!("../resources/images/plants/manifest.json");
const CREDITS_JSON: &str = include_str!("../resources/images/plants/credits.json");

pub fn load_images() -> AppResult<PlantImages> {
    let raw: garden_core::images::RawPlantImages = serde_json::from_str(PLANT_IMAGES_JSON)?;
    Ok(raw.into())
}

pub fn load_manifest() -> AppResult<ImageManifest> {
    let raw: garden_core::images::RawManifest = serde_json::from_str(MANIFEST_JSON)?;
    Ok(raw.into())
}

pub fn list_credits() -> AppResult<Vec<Credit>> {
    Ok(serde_json::from_str(CREDITS_JSON)?)
}

/// Build an `asset://localhost/` URL for a bundled resource at `rel_path`
/// relative to the image bundle root (`resources/images/plants/`).
pub fn bundled_url(resource_dir: &Path, rel_path: &str) -> String {
    let abs = resource_dir
        .join("resources")
        .join("images")
        .join("plants")
        .join(rel_path);
    format!("asset://localhost/{}", abs.to_string_lossy().replace('\\', "/"))
}

pub fn generic_url(resource_dir: &Path, images: &PlantImages, manifest: &ImageManifest) -> String {
    manifest
        .assets
        .get(&images.generic_asset_id)
        .map(|info| bundled_url(resource_dir, &info.path))
        .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_plant_image(
    app_data: &Path,
    resource_dir: &Path,
    images: &PlantImages,
    manifest: &ImageManifest,
    catalog_id: &str,
    asset_id: Option<&str>,
    image_path: Option<&str>,
    license_text: Option<&str>,
    attribution_text: Option<&str>,
    category: Option<&str>,
    override_filename: Option<&str>,
) -> PlantImage {
    let override_url = override_filename.map(|f| crate::storage::plant_image_url(app_data, f));

    let species_url = asset_id
        .and_then(|id| manifest.assets.get(id))
        .map(|info| bundled_url(resource_dir, &info.path))
        .or_else(|| image_path.map(|p| bundled_url(resource_dir, p)));

    let category_url = category
        .and_then(|c| images.categories.get(c))
        .and_then(|id| manifest.assets.get(id))
        .map(|info| bundled_url(resource_dir, &info.path));

    let generic = generic_url(resource_dir, images, manifest);

    let attribution = asset_id
        .and_then(|id| manifest.assets.get(id))
        .map(|info| ImageAttribution {
            license: info.license.clone(),
            attribution: info.attribution.clone(),
        })
        .or_else(|| match (license_text, attribution_text) {
            (Some(l), Some(a)) => Some(ImageAttribution {
                license: l.to_string(),
                attribution: a.to_string(),
            }),
            _ => None,
        });

    PlantImage {
        catalog_id: catalog_id.to_string(),
        override_url,
        species_url,
        category_url,
        generic_url: generic,
        attribution,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn bundled_url_uses_resource_dir() {
        let url = bundled_url(Path::new("C:/app/res"), "categories/herb.svg");
        assert!(url.starts_with("asset://localhost/C:/app/res/resources/images/plants/categories/herb.svg"));
    }

    #[test]
    fn resolve_orders_override_then_species() {
        let images = PlantImages {
            species: HashMap::new(),
            categories: [("vegetable".to_string(), "cat_vegetable".to_string())].into_iter().collect(),
            generic_asset_id: "generic_leaf".to_string(),
        };
        let manifest = ImageManifest {
            assets: [
                ("tomato".to_string(), crate_garden_asset("vegetable/tomato.svg")),
                ("cat_vegetable".to_string(), crate_garden_asset("categories/vegetable.svg")),
                ("generic_leaf".to_string(), crate_garden_asset("generic/leaf.svg")),
            ].into_iter().collect(),
        };
        let pi = resolve_plant_image(
            Path::new("C:/app/data"),
            Path::new("C:/app/res"),
            &images,
            &manifest,
            "plant_tomato",
            Some("tomato"),
            None,
            None,
            None,
            Some("vegetable"),
            Some("user.jpg"),
        );
        assert!(pi.override_url.as_deref().unwrap().ends_with("plant_images/user.jpg"));
        assert!(pi.species_url.as_deref().unwrap().ends_with("vegetable/tomato.svg"));
        assert!(pi.category_url.as_deref().unwrap().ends_with("categories/vegetable.svg"));
        assert!(pi.generic_url.ends_with("generic/leaf.svg"));
        assert!(pi.attribution.is_some());
    }

    fn crate_garden_asset(path: &str) -> garden_core::images::AssetInfo {
        garden_core::images::AssetInfo {
            path: path.to_string(),
            license: "CC0".to_string(),
            attribution: "Twemoji".to_string(),
            source_url: None,
        }
    }
}
