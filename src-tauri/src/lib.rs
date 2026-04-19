use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tauri::Manager;
use tokio::task::JoinHandle;
use uuid::Uuid;

mod commands;
mod db;
mod error;
mod models;

use db::{init_db, SURREAL_DB, SURREAL_NS, SURREAL_URL};
use models::Message;

pub struct AppState {
    pub db: Arc<Surreal<Client>>,
    /// In-process message cache keyed by room_id string. Arc so the live-event
    /// task in subscribe_room can hold a reference without borrowing AppState.
    pub msg_cache: Arc<Mutex<HashMap<String, Vec<Message>>>>,
    /// LRU order of cached room IDs (front = most recent). Evicts beyond 5.
    pub cache_order: Arc<Mutex<Vec<String>>>,
    /// std::sync::Mutex is intentional: guards are never held across .await points.
    pub subscriptions: Mutex<HashMap<Uuid, JoinHandle<()>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let surreal = init_db(
                    SURREAL_URL.as_str(),
                    SURREAL_NS.as_str(),
                    SURREAL_DB.as_str(),
                )
                .await
                .expect("Failed to connect to SurrealDB");

                let state = AppState {
                    db: Arc::new(surreal),
                    msg_cache: Arc::new(Mutex::new(HashMap::new())),
                    cache_order: Arc::new(Mutex::new(Vec::new())),
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
            commands::user::restore_session,
            commands::user::update_profile,
            commands::user::search_users,
            commands::user::get_contacts,
            commands::user::add_contact,
            commands::chat::create_room,
            commands::chat::get_rooms,
            commands::chat::invite_to_room,
            commands::chat::get_or_create_direct_room,
            commands::chat::send_message,
            commands::chat::get_messages,
            commands::chat::get_cached_messages,
            commands::chat::delete_message,
            commands::chat::edit_message,
            commands::chat::toggle_reaction,
            commands::chat::mark_room_read,
            commands::chat::subscribe_room,
            commands::chat::unsubscribe_room,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
