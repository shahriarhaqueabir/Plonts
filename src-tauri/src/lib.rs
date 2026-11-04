pub mod commands;
pub mod db;
pub mod error;
pub mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let mut conn = db::open(&dir.join("lovelygarden.db"))?;
            db::migrate(&mut conn)?;
            app.manage(AppState {
                db: std::sync::Mutex::new(conn),
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
