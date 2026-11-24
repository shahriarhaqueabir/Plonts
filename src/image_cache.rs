use std::collections::HashMap;

use leptos::prelude::*;
use garden_core::models::PlantImage;

/// A small in-memory cache for resolved plant images, shared via context.
#[derive(Clone, Default)]
pub struct PlantImageCache {
    pub inner: RwSignal<HashMap<String, PlantImage>>,
}

impl PlantImageCache {
    pub fn new() -> Self {
        Self { inner: RwSignal::new(HashMap::new()) }
    }

    pub fn get(&self, id: &str) -> Option<PlantImage> {
        self.inner.get().get(id).cloned()
    }

    pub fn set(&self, id: String, img: PlantImage) {
        self.inner.update(|m| { m.insert(id, img); });
    }
}

pub fn provide_image_cache() {
    provide_context(PlantImageCache::new());
}

pub fn use_image_cache() -> PlantImageCache {
    use_context::<PlantImageCache>().unwrap_or_default()
}