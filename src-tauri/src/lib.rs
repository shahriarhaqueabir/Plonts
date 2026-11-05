pub mod commands;
pub mod db;
pub mod error;
pub mod images;
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
            let resource_dir = app.path().resource_dir().unwrap_or_else(|_| dir.clone());
            let mut conn = db::open(&dir.join("lovelygarden.db"))?;
            db::migrate(&mut conn)?;
            db::seed::ensure_seeded(&mut conn)?;
            db::guides::ensure_guides_seeded(&mut conn)?;
            db::tips::ensure_tips_seeded(&mut conn)?;
            let images = images::load_images()?;
            let manifest = images::load_manifest()?;
            app.manage(AppState {
                db: std::sync::Mutex::new(conn),
                app_data: dir,
                resource_dir,
                images,
                manifest,
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
            commands::problems::diagnose_problem,
            commands::problems::list_problems,
            commands::problems::create_problem,
            commands::problems::resolve_problem,
            commands::problems::delete_problem,
            commands::seeds::list_seeds,
            commands::seeds::add_seed,
            commands::seeds::delete_seed,
            commands::seeds::acknowledge_seed_viability,
            commands::harvests::list_harvests,
            commands::harvests::add_harvest,
            commands::harvests::delete_harvest,
            commands::harvests::yield_summary,
            commands::supplies::list_shopping_items,
            commands::supplies::add_shopping_item,
            commands::supplies::toggle_shopping_item,
            commands::supplies::delete_shopping_item,
            commands::supplies::list_tools,
            commands::supplies::add_tool,
            commands::supplies::mark_tool_maintained,
            commands::supplies::delete_tool
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
