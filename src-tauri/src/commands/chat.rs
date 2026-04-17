use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;
use futures_util::StreamExt;
use surrealdb::Notification;

use crate::db::AppState;
use crate::error::{into_err, AppError};
use crate::models::{Message, Room};

/// Wrapper emitted to the frontend for each LIVE query notification.
/// Includes the action type so the frontend can distinguish create/update/delete.
#[derive(serde::Serialize)]
struct LiveMessageEvent<'a> {
    action: String,
    data: &'a Message,
}

/// Create a new chat room.
#[tauri::command]
pub async fn create_room(
    state: State<'_, AppState>,
    name: String,
) -> Result<Room, String> {
    let mut result: Vec<Room> = state
        .db
        .query("CREATE room SET name = $name, created = time::now()")
        .bind(("name", name))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::NotFound("room after create".into())))
}

/// Fetch all rooms.
#[tauri::command]
pub async fn get_rooms(state: State<'_, AppState>) -> Result<Vec<Room>, String> {
    let result: Vec<Room> = state
        .db
        .query("SELECT * FROM room ORDER BY created DESC")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    Ok(result)
}

/// Send a message to a room.
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    room_id: String,
    body: String,
) -> Result<Message, String> {
    let mut result: Vec<Message> = state
        .db
        .query(
            "CREATE message SET
                room            = type::record('room', $room_id),
                author          = $auth,
                author_username = $auth.username,
                body            = $body,
                created         = time::now()",
        )
        .bind(("room_id", room_id))
        .bind(("body", body))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::NotFound("message after create".into())))
}

/// Fetch all messages in a room, oldest first.
#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    room_id: String,
) -> Result<Vec<Message>, String> {
    let result: Vec<Message> = state
        .db
        .query("SELECT * FROM message WHERE room = type::record('room', $room_id) ORDER BY created ASC")
        .bind(("room_id", room_id))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    Ok(result)
}

/// Delete a message by its ID string (e.g. "message:abc123").
#[tauri::command]
pub async fn delete_message(
    state: State<'_, AppState>,
    message_id: String,
) -> Result<(), String> {
    state
        .db
        .query("DELETE type::record($id) WHERE author = $auth")
        .bind(("id", message_id))
        .await
        .map_err(into_err)?;

    Ok(())
}

/// Start a LIVE query for new messages in a room.
/// Spawns a background tokio task that emits "chat:message" Tauri events.
///
/// Returns a local subscription UUID — pass it to `unsubscribe_room` on cleanup.
/// Aborting the JoinHandle drops the stream, which closes the LIVE query automatically.
#[tauri::command]
pub async fn subscribe_room(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    room_id: String,
) -> Result<String, String> {
    let db = state.db.clone();

    let mut stream = db
        .query("LIVE SELECT * FROM message WHERE room = type::record('room', $room_id)")
        .bind(("room_id", room_id))
        .await
        .map_err(into_err)?
        .stream::<Notification<Message>>(0)
        .map_err(into_err)?;

    let sub_id = Uuid::new_v4();

    let handle = tokio::spawn(async move {
        while let Some(Ok(notification)) = stream.next().await {
            let _ = app_handle.emit("chat:message", &LiveMessageEvent {
                action: format!("{:?}", notification.action),
                data: &notification.data,
            });
        }
    });

    state.subscriptions.lock().unwrap().insert(sub_id, handle);

    Ok(sub_id.to_string())
}

/// Stop a LIVE query subscription.
/// Aborts the background task — dropping the stream closes the LIVE query.
#[tauri::command]
pub async fn unsubscribe_room(
    state: State<'_, AppState>,
    sub_id: String,
) -> Result<(), String> {
    let uuid = sub_id
        .parse::<Uuid>()
        .map_err(|e| into_err(AppError::Subscription(e.to_string())))?;

    if let Some(handle) = state.subscriptions.lock().unwrap().remove(&uuid) {
        handle.abort();
    }

    Ok(())
}
