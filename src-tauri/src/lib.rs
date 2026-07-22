mod commands;

use std::sync::Mutex;

use tauri::Manager;

pub struct AppState {
  pub db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let data_dir = app.path().app_data_dir()?;
      std::fs::create_dir_all(&data_dir)?;
      let db = mifi_core::open(data_dir.join("mifi.sqlite3"))
        .expect("database should open and migrate cleanly");
      app.manage(AppState { db: Mutex::new(db) });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::list_accounts,
      commands::list_transactions,
      commands::get_overview,
      commands::list_categories,
      commands::get_category_detail,
      commands::get_budget_overview,
      commands::set_budget_target
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
