use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tauri::Manager;

mod commands;
mod db;
mod error;
mod models;

use db::{init_db, AppState, SURREAL_DB, SURREAL_NS, SURREAL_URL};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let surreal = init_db(SURREAL_URL.as_str(), SURREAL_NS.as_str(), SURREAL_DB.as_str())
                    .await
                    .expect("Failed to connect to SurrealDB");

                let state = AppState {
                    db: Arc::new(surreal),
                    token: Mutex::new(None),
                    subscriptions: Mutex::new(HashMap::new()),
                };

                app_handle.manage(state);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::user::signup,
            commands::user::signin,
            commands::user::signout,
            commands::user::get_me,
            commands::user::update_profile,
            commands::user::get_contacts,
            commands::user::add_contact,
            commands::chat::create_room,
            commands::chat::get_rooms,
            commands::chat::send_message,
            commands::chat::get_messages,
            commands::chat::delete_message,
            commands::chat::subscribe_room,
            commands::chat::unsubscribe_room,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
