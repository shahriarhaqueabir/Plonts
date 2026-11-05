use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use garden_core::images::{ImageManifest, PlantImages};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub app_data: PathBuf,
    pub resource_dir: PathBuf,
    pub images: PlantImages,
    pub manifest: ImageManifest,
}
