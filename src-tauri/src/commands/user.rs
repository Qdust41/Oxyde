use tauri::State;

use crate::db::{AppState, SURREAL_ACCESS, SURREAL_DB, SURREAL_NS};
use crate::error::{into_err, AppError};
use crate::models::{Contact, User};

/// Create a new user account via SurrealDB Record Auth SIGNUP.
/// Returns the created User record.
#[tauri::command]
pub async fn signup(
    state: State<'_, AppState>,
    email: String,
    username: String,
    password: String,
) -> Result<User, String> {
    let credentials = surrealdb::opt::auth::Record {
        access: SURREAL_ACCESS.to_string(),
        namespace: SURREAL_NS.to_string(),
        database: SURREAL_DB.to_string(),
        params: serde_json::json!({
            "email": email,
            "username": username,
            "password": password,
        }),
    };
    // into_insecure_token() returns the raw JWT String directly (3.x API).
    let token = state.db.signup(credentials).await.map_err(into_err)?;
    *state.token.lock().unwrap() = Some(token.access.into_insecure_token());

    let mut result: Vec<User> = state
        .db
        .query("SELECT * FROM $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::Auth("signup succeeded but $auth not set".into())))
}

/// Authenticate an existing user via SurrealDB Record Auth SIGNIN.
/// Returns the JWT token string.
#[tauri::command]
pub async fn signin(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<String, String> {
    let credentials = surrealdb::opt::auth::Record {
        access: SURREAL_ACCESS.to_string(),
        namespace: SURREAL_NS.to_string(),
        database: SURREAL_DB.to_string(),
        params: serde_json::json!({
            "email": email,
            "password": password,
        }),
    };
    let token_str = state.db.signin(credentials).await.map_err(into_err)?.access.into_insecure_token();
    *state.token.lock().unwrap() = Some(token_str.clone());
    Ok(token_str)
}

/// Clear the current session. Invalidates the token in state.
#[tauri::command]
pub async fn signout(state: State<'_, AppState>) -> Result<(), String> {
    state.db.invalidate().await.map_err(into_err)?;
    *state.token.lock().unwrap() = None;
    Ok(())
}

/// Fetch the currently authenticated user record.
/// Relies on the DB connection being authenticated (token set via signin/signup).
#[tauri::command]
pub async fn get_me(state: State<'_, AppState>) -> Result<User, String> {
    let mut result: Vec<User> = state
        .db
        .query("SELECT * FROM $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::Auth("not authenticated".into())))
}

/// Update mutable profile fields. Only provided fields are changed.
#[tauri::command]
pub async fn update_profile(
    state: State<'_, AppState>,
    username: Option<String>,
    avatar: Option<String>,
) -> Result<User, String> {
    let mut result: Vec<User> = state
        .db
        .query(
            "UPDATE $auth SET
                username = $username ?? username,
                avatar   = $avatar   ?? avatar
             RETURN AFTER",
        )
        .bind(("username", username))
        .bind(("avatar", avatar))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::NotFound("user".into())))
}

/// Return the contacts list for the current user.
/// Contacts are `contact` records where `owner = $auth`.
#[tauri::command]
pub async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let result: Vec<User> = state
        .db
        .query("SELECT target.* FROM contact WHERE owner = $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    Ok(result)
}

/// Add a user to the current user's contact list. Stub — returns the Contact record.
#[tauri::command]
pub async fn add_contact(
    state: State<'_, AppState>,
    user_id: String,
) -> Result<Contact, String> {
    let mut result: Vec<Contact> = state
        .db
        .query("CREATE contact SET owner = $auth, target = type::record('user', $uid)")
        .bind(("uid", user_id))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result.pop().ok_or_else(|| into_err(AppError::NotFound("contact after create".into())))
}
