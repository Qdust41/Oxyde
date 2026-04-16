use std::collections::HashMap;
use std::sync::{Arc, Mutex, LazyLock};
use std::env;

use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::Surreal;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::error::AppError;

pub static SURREAL_URL: LazyLock<String> = LazyLock::new(|| {
    env::var("SURREAL_URL").unwrap_or_else(|_| "localhost:8000".to_string())
});
pub static SURREAL_NS: LazyLock<String> = LazyLock::new(|| {
    env::var("SURREAL_NS").unwrap_or_else(|_| "dev".to_string())
});
pub static SURREAL_DB: LazyLock<String> = LazyLock::new(|| {
    env::var("SURREAL_DB").unwrap_or_else(|_| "oxyde".to_string())
});
pub static SURREAL_ACCESS: LazyLock<String> = LazyLock::new(|| {
    env::var("SURREAL_ACCESS").unwrap_or_else(|_| "account".to_string())
});

pub struct AppState {
    /// Long-lived authenticated WebSocket connection to SurrealDB.
    pub db: Arc<Surreal<Client>>,
    /// JWT token from Record Auth signin. Used to re-authenticate on reconnect.
    /// std::sync::Mutex is intentional: lock is acquired and released before any .await.
    pub token: Mutex<Option<String>>,
    /// Active LIVE query tasks keyed by their SurrealDB LIVE query UUID.
    /// Abort a handle + KILL the query to clean up.
    /// std::sync::Mutex is intentional: guards are never held across .await points.
    /// If a future command needs to lock across .await, switch to tokio::sync::Mutex.
    pub subscriptions: Mutex<HashMap<Uuid, JoinHandle<()>>>,
}

/// Connect to SurrealDB over WebSocket and select namespace/database.
/// Call once at app startup before managing state.
pub async fn init_db(url: &str, ns: &str, db: &str) -> Result<Surreal<Client>, AppError> {
    let client = Surreal::new::<Ws>(url).await?;
    client.use_ns(ns).use_db(db).await?;
    Ok(client)
}
