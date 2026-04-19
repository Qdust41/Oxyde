use std::collections::HashMap;

use futures_util::StreamExt;
use surrealdb::types::{RecordId, RecordIdKey};
use surrealdb::Notification;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::db::AppState;
use crate::error::{into_err, AppError};
use crate::models::{Message, MessageReaction, MessageReactionSummary, Room, User};

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 100;
const MAX_MESSAGE_LEN: usize = 4000;
const MAX_ROOM_NAME_LEN: usize = 80;

/// Wrapper emitted to the frontend for each LIVE query notification.
/// Includes the action type so the frontend can distinguish create/update/delete.
#[derive(serde::Serialize)]
struct LiveMessageEvent<'a> {
    action: String,
    data: &'a Message,
}

fn validate_room_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Auth("room name is required".into()).to_string());
    }
    if trimmed.chars().count() > MAX_ROOM_NAME_LEN {
        return Err(AppError::Auth(format!(
            "room name must be {MAX_ROOM_NAME_LEN} characters or less"
        ))
        .to_string());
    }
    Ok(())
}

fn validate_message_body(body: &str) -> Result<(), String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(AppError::Auth("message cannot be empty".into()).to_string());
    }
    if trimmed.chars().count() > MAX_MESSAGE_LEN {
        return Err(AppError::Auth(format!(
            "message must be {MAX_MESSAGE_LEN} characters or less"
        ))
        .to_string());
    }
    Ok(())
}

fn record_key_string(id: &RecordId) -> String {
    match &id.key {
        RecordIdKey::String(value) => value.clone(),
        RecordIdKey::Number(value) => value.to_string(),
        RecordIdKey::Uuid(value) => value.to_string(),
        other => format!("{other:?}"),
    }
}

fn user_record_key(user_id: &str) -> String {
    let user_id = user_id.trim();
    if let Some((table, key)) = user_id.split_once(':') {
        if table == "user" {
            return format!("user:{key}");
        }
    }

    format!("user:{user_id}")
}

fn user_id_key(user_id: &str) -> String {
    let user_id = user_id.trim();
    if let Some((table, key)) = user_id.split_once(':') {
        if table == "user" {
            return key.to_string();
        }
    }

    user_id.to_string()
}

fn direct_room_key(current_user: &RecordId, target_user_id: &str) -> String {
    let mut participants = [
        user_record_key(&record_key_string(current_user)),
        user_record_key(target_user_id),
    ];
    participants.sort();
    participants.join("|")
}

async fn current_user(state: &State<'_, AppState>) -> Result<User, String> {
    let mut result: Vec<User> = state
        .db
        .query("SELECT * FROM $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result
        .pop()
        .ok_or_else(|| into_err(AppError::Auth("not authenticated".into())))
}

async fn hydrate_reactions(
    state: &State<'_, AppState>,
    user: &User,
    messages: &mut [Message],
) -> Result<(), String> {
    for message in messages {
        let reactions: Vec<MessageReaction> = state
            .db
            .query("SELECT * FROM message_reaction WHERE message = $message")
            .bind(("message", message.id.clone()))
            .await
            .map_err(into_err)?
            .take(0)
            .map_err(into_err)?;

        let mut grouped: HashMap<String, MessageReactionSummary> = HashMap::new();
        for reaction in reactions {
            let entry = grouped
                .entry(reaction.emoji.clone())
                .or_insert(MessageReactionSummary {
                    emoji: reaction.emoji,
                    count: 0,
                    reacted_by_me: false,
                });
            entry.count += 1;
            if reaction.user == user.id {
                entry.reacted_by_me = true;
            }
        }

        let mut summaries: Vec<MessageReactionSummary> = grouped.into_values().collect();
        summaries.sort_by(|a, b| a.emoji.cmp(&b.emoji));
        message.reactions = Some(summaries);
    }

    Ok(())
}

async fn hydrate_direct_rooms(
    state: &State<'_, AppState>,
    rooms: &mut [Room],
) -> Result<(), String> {
    for room in rooms.iter_mut().filter(|room| room.kind == "direct") {
        let mut users: Vec<User> = state
            .db
            .query(
                "SELECT * FROM user
                 WHERE id IN (
                    SELECT VALUE user FROM room_member
                    WHERE room = $room AND user != $auth
                 )
                 LIMIT 1",
            )
            .bind(("room", room.id.clone()))
            .await
            .map_err(into_err)?
            .take(0)
            .map_err(into_err)?;

        room.other_user = users.pop();
    }

    Ok(())
}

fn dedupe_direct_rooms(rooms: Vec<Room>) -> Vec<Room> {
    let mut seen_direct_users = HashMap::new();
    let mut deduped = Vec::with_capacity(rooms.len());

    for room in rooms {
        if room.kind == "direct" {
            if let Some(other_user) = &room.other_user {
                let key = user_record_key(&record_key_string(&other_user.id));
                if seen_direct_users.insert(key, ()).is_some() {
                    continue;
                }
            }
        }

        deduped.push(room);
    }

    deduped
}

/// Create a new chat room and add the creator as owner.
#[tauri::command]
pub async fn create_room(
    state: State<'_, AppState>,
    name: String,
    kind: Option<String>,
) -> Result<Room, String> {
    validate_room_name(&name)?;
    let room_kind = kind.unwrap_or_else(|| "public".to_string());
    if !matches!(room_kind.as_str(), "public" | "private") {
        return Err(AppError::Auth("room kind must be public or private".into()).to_string());
    }

    let mut result: Vec<Room> = state
        .db
        .query(
            "CREATE room SET
                name       = $name,
                kind       = $kind,
                created_by = $auth,
                created    = time::now(),
                updated    = time::now()",
        )
        .bind(("name", name.trim().to_string()))
        .bind(("kind", room_kind))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    let room = result
        .pop()
        .ok_or_else(|| into_err(AppError::NotFound("room after create".into())))?;

    state
        .db
        .query(
            "CREATE room_member SET
                room         = $room,
                user         = $auth,
                role         = 'owner',
                joined       = time::now(),
                last_read_at = time::now(),
                muted        = false",
        )
        .bind(("room", room.id.clone()))
        .await
        .map_err(into_err)?;

    Ok(room)
}

/// Fetch public rooms and rooms the current user belongs to.
#[tauri::command]
pub async fn get_rooms(state: State<'_, AppState>) -> Result<Vec<Room>, String> {
    let mut result: Vec<Room> = state
        .db
        .query(
            "SELECT * FROM room
             WHERE kind = 'public' OR id IN (SELECT VALUE room FROM room_member WHERE user = $auth)
             ORDER BY updated DESC, created DESC",
        )
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    hydrate_direct_rooms(&state, &mut result).await?;
    Ok(dedupe_direct_rooms(result))
}

/// Add a user to a room. Room owners can invite others.
#[tauri::command]
pub async fn invite_to_room(
    state: State<'_, AppState>,
    room_id: String,
    user_id: String,
) -> Result<(), String> {
    state
        .db
        .query(
            "CREATE room_member SET
                room         = type::record('room', $room_id),
                user         = type::record('user', $user_id),
                role         = 'member',
                joined       = time::now(),
                muted        = false",
        )
        .bind(("room_id", room_id))
        .bind(("user_id", user_id))
        .await
        .map_err(into_err)?;

    Ok(())
}

/// Return an existing direct room for two users or create it.
#[tauri::command]
pub async fn get_or_create_direct_room(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<Room, String> {
    let me = current_user(&state).await?;
    let target_user_id = user_id_key(&user_id);
    let current_user_key = record_key_string(&me.id);
    if user_record_key(&current_user_key) == user_record_key(&target_user_id) {
        return Err(
            AppError::Auth("cannot start a direct message with yourself".into()).to_string(),
        );
    }

    let direct_key = direct_room_key(&me.id, &target_user_id);

    let mut existing: Vec<Room> = state
        .db
        .query("SELECT * FROM room WHERE kind = 'direct' AND direct_key = $direct_key LIMIT 1")
        .bind(("direct_key", direct_key.clone()))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    if let Some(mut room) = existing.pop() {
        hydrate_direct_rooms(&state, std::slice::from_mut(&mut room)).await?;
        return Ok(room);
    }

    let mut existing_by_members: Vec<Room> = state
        .db
        .query(
            "SELECT * FROM room
             WHERE kind = 'direct'
               AND id IN (SELECT VALUE room FROM room_member WHERE user = $auth)
               AND id IN (SELECT VALUE room FROM room_member WHERE user = type::record('user', $user_id))
             ORDER BY updated DESC, created DESC
             LIMIT 1",
        )
        .bind(("user_id", target_user_id.clone()))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    if let Some(mut room) = existing_by_members.pop() {
        hydrate_direct_rooms(&state, std::slice::from_mut(&mut room)).await?;
        return Ok(room);
    }

    let mut created: Vec<Room> = state
        .db
        .query(
            "CREATE room SET
                name       = NONE,
                kind       = 'direct',
                direct_key = $direct_key,
                created_by = $auth,
                created    = time::now(),
                updated    = time::now()",
        )
        .bind(("direct_key", direct_key))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    let room = created
        .pop()
        .ok_or_else(|| into_err(AppError::NotFound("direct room after create".into())))?;

    state
        .db
        .query(
            "CREATE room_member SET room = $room, user = $auth, role = 'owner', joined = time::now(), last_read_at = time::now(), muted = false;
             CREATE room_member SET room = $room, user = type::record('user', $user_id), role = 'member', joined = time::now(), muted = false;",
        )
        .bind(("room", room.id.clone()))
        .bind(("user_id", target_user_id))
        .await
        .map_err(into_err)?;

    let mut room = room;
    hydrate_direct_rooms(&state, std::slice::from_mut(&mut room)).await?;
    Ok(room)
}

/// Send a message to a room.
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    room_id: String,
    body: String,
    reply_to: Option<String>,
) -> Result<Message, String> {
    validate_message_body(&body)?;

    let query = if reply_to.is_some() {
        "CREATE message SET
                room            = type::record('room', $room_id),
                author          = $auth,
                author_username = $auth.username,
                body            = $body,
                reply_to        = type::record('message', $reply_to),
                deleted         = false,
                created         = time::now();
             UPDATE type::record('room', $room_id) SET updated = time::now();"
    } else {
        "CREATE message SET
                room            = type::record('room', $room_id),
                author          = $auth,
                author_username = $auth.username,
                body            = $body,
                deleted         = false,
                created         = time::now();
             UPDATE type::record('room', $room_id) SET updated = time::now();"
    };

    let mut response = state
        .db
        .query(query)
        .bind(("room_id", room_id))
        .bind(("body", body.trim().to_string()));

    if let Some(reply_to) = reply_to {
        response = response.bind(("reply_to", reply_to));
    }

    let mut result: Vec<Message> = response
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result
        .pop()
        .ok_or_else(|| into_err(AppError::NotFound("message after create".into())))
}

/// Fetch a bounded page of messages in a room, oldest first.
#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    room_id: String,
    before: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<Message>, String> {
    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let query = if before.is_some() {
        "SELECT * FROM message
         WHERE room = type::record('room', $room_id) AND created < <datetime>$before
         ORDER BY created DESC
         LIMIT $limit"
    } else {
        "SELECT * FROM message
         WHERE room = type::record('room', $room_id)
         ORDER BY created DESC
         LIMIT $limit"
    };

    let mut response = state
        .db
        .query(query)
        .bind(("room_id", room_id))
        .bind(("limit", limit));

    if let Some(before) = before {
        response = response.bind(("before", before));
    }

    let mut result: Vec<Message> = response
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.reverse();
    let user = current_user(&state).await?;
    hydrate_reactions(&state, &user, &mut result).await?;
    Ok(result)
}

/// Soft-delete a message by its ID string.
#[tauri::command]
pub async fn delete_message(state: State<'_, AppState>, message_id: String) -> Result<(), String> {
    state
        .db
        .query("UPDATE type::record($id) SET deleted = true, body = '', updated = time::now() WHERE author = $auth")
        .bind(("id", message_id))
        .await
        .map_err(into_err)?;

    Ok(())
}

/// Edit the current user's message.
#[tauri::command]
pub async fn edit_message(
    state: State<'_, AppState>,
    message_id: String,
    body: String,
) -> Result<Message, String> {
    validate_message_body(&body)?;
    let mut result: Vec<Message> = state
        .db
        .query("UPDATE type::record($id) SET body = $body, updated = time::now() WHERE author = $auth RETURN AFTER")
        .bind(("id", message_id))
        .bind(("body", body.trim().to_string()))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result
        .pop()
        .ok_or_else(|| into_err(AppError::NotFound("message".into())))
}

/// Toggle one emoji reaction for the current user.
#[tauri::command]
pub async fn toggle_reaction(
    state: State<'_, AppState>,
    message_id: String,
    emoji: String,
) -> Result<(), String> {
    let emoji = emoji.trim();
    if emoji.is_empty() || emoji.chars().count() > 16 {
        return Err(AppError::Auth("invalid reaction".into()).to_string());
    }

    let existing: Vec<MessageReaction> = state
        .db
        .query("SELECT * FROM message_reaction WHERE message = type::record($message_id) AND user = $auth AND emoji = $emoji")
        .bind(("message_id", message_id.clone()))
        .bind(("emoji", emoji.to_string()))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    if existing.is_empty() {
        state
            .db
            .query("CREATE message_reaction SET message = type::record($message_id), user = $auth, emoji = $emoji, created = time::now()")
            .bind(("message_id", message_id))
            .bind(("emoji", emoji.to_string()))
            .await
            .map_err(into_err)?;
    } else {
        state
            .db
            .query("DELETE message_reaction WHERE message = type::record($message_id) AND user = $auth AND emoji = $emoji")
            .bind(("message_id", message_id))
            .bind(("emoji", emoji.to_string()))
            .await
            .map_err(into_err)?;
    }

    Ok(())
}

/// Mark the room read for the current user.
#[tauri::command]
pub async fn mark_room_read(state: State<'_, AppState>, room_id: String) -> Result<(), String> {
    state
        .db
        .query("UPDATE room_member SET last_read_at = time::now() WHERE room = type::record('room', $room_id) AND user = $auth")
        .bind(("room_id", room_id))
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
            let _ = app_handle.emit(
                "chat:message",
                &LiveMessageEvent {
                    action: format!("{:?}", notification.action),
                    data: &notification.data,
                },
            );
        }
    });

    state.subscriptions.lock().unwrap().insert(sub_id, handle);

    Ok(sub_id.to_string())
}

/// Stop a LIVE query subscription.
/// Aborts the background task — dropping the stream closes the LIVE query.
#[tauri::command]
pub async fn unsubscribe_room(state: State<'_, AppState>, sub_id: String) -> Result<(), String> {
    let uuid = sub_id
        .parse::<Uuid>()
        .map_err(|e| into_err(AppError::Subscription(e.to_string())))?;

    if let Some(handle) = state.subscriptions.lock().unwrap().remove(&uuid) {
        handle.abort();
    }

    Ok(())
}
