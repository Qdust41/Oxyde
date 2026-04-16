use std::collections::HashMap;
use std::sync::{Arc, Mutex, LazyLock};
use std::env;

use surrealdb::engine::remote::ws::{Client, Ws, Wss};
use surrealdb::Surreal;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::error::AppError;

// This should set the env variable correctly both during compile time and runtime (for development).
pub static SURREAL_URL: LazyLock<String> = LazyLock::new(|| {
    option_env!("SURREAL_URL")
        .map(str::to_string)
        .unwrap_or_else(|| env::var("SURREAL_URL")
            .unwrap_or_else(|_| "ws://localhost:8000".to_string()))
});
pub static SURREAL_NS: LazyLock<String> = LazyLock::new(|| {
    option_env!("SURREAL_NS")
        .map(str::to_string)
        .unwrap_or_else(|| env::var("SURREAL_NS")
            .unwrap_or_else(|_| "dev".to_string()))
});
pub static SURREAL_DB: LazyLock<String> = LazyLock::new(|| {
    option_env!("SURREAL_DB")
        .map(str::to_string)
        .unwrap_or_else(|| env::var("SURREAL_DB")
            .unwrap_or_else(|_| "oxyde".to_string()))
});
pub static SURREAL_ACCESS: LazyLock<String> = LazyLock::new(|| {
    option_env!("SURREAL_ACCESS")
        .map(str::to_string)
        .unwrap_or_else(|| env::var("SURREAL_ACCESS")
            .unwrap_or_else(|_| "account".to_string()))
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
/// URL may include protocol prefix: `ws://`, `wss://`, `http://`, or `https://`.
/// `wss://` and `https://` use TLS; others use plain WebSocket.
/// Call once at app startup before managing state.
pub async fn init_db(url: &str, ns: &str, db: &str) -> Result<Surreal<Client>, AppError> {
    let client = if url.starts_with("wss://") || url.starts_with("https://") {
        let host = url
            .strip_prefix("wss://")
            .or_else(|| url.strip_prefix("https://"))
            .unwrap_or(url);
        Surreal::new::<Wss>(host).await?
    } else {
        let host = url
            .strip_prefix("ws://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);
        Surreal::new::<Ws>(host).await?
    };
    client.use_ns(ns).use_db(db).await?;
    Ok(client)
}
