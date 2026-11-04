pub mod commands;
pub mod db;
pub mod error;
pub mod state;
pub mod storage;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let mut conn = db::open(&dir.join("lovelygarden.db"))?;
            db::migrate(&mut conn)?;
            db::seed::ensure_seeded(&mut conn)?;
            db::guides::ensure_guides_seeded(&mut conn)?;
            db::tips::ensure_tips_seeded(&mut conn)?;
            app.manage(AppState {
                db: std::sync::Mutex::new(conn),
                app_data: dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::gardens::list_gardens,
            commands::gardens::create_garden,
            commands::gardens::delete_garden,
            commands::gardens::list_beds,
            commands::gardens::create_bed,
            commands::gardens::delete_bed,
            commands::catalog::search_plants,
            commands::catalog::get_plant_detail,
            commands::catalog::list_categories,
            commands::planted::add_planting,
            commands::planted::list_plantings,
            commands::planted::update_planting_status,
            commands::planted::remove_planting,
            commands::calendar::get_calendar,
            commands::guides::list_guides,
            commands::guides::get_guide,
            commands::guides::save_guide_progress,
            commands::guides::set_guide_bookmark,
            commands::tips::get_daily_tip,
            commands::tips::dismiss_daily_tip,
            commands::tips::snooze_daily_tip,
            commands::journal::list_journal_entries,
            commands::journal::create_journal_entry,
            commands::journal::update_journal_entry,
            commands::journal::delete_journal_entry,
            commands::journal::add_journal_photo,
            commands::journal::delete_journal_photo,
            commands::journal::export_journal_photo,
            commands::journal::add_measurement,
            commands::journal::list_measurements,
            commands::journal::delete_measurement,
            commands::journal::get_plant_timeline,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
