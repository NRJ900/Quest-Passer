use tauri::Manager;

pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_log::Builder::default().build())
    .setup(|app| {
      #[cfg(debug_assertions)]
      {
        let window = app.get_webview_window("main").unwrap();
        // window.open_devtools(); 
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        commands::create_dummy_game,
        commands::start_game_process,
        commands::stop_process,
        commands::fetch_game_list
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
