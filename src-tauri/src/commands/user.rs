use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::db::{SURREAL_ACCESS, SURREAL_DB, SURREAL_NS};
use crate::error::{into_err, AppError};
use crate::models::{Contact, User};
use crate::AppState;

const SESSION_STORE: &str = "session.json";
const TOKEN_KEY: &str = "token";
const MIN_PASSWORD_LEN: usize = 8;
const MAX_USERNAME_LEN: usize = 32;
const MAX_EMAIL_LEN: usize = 254;

fn validate_email(email: &str) -> Result<(), String> {
    let email = email.trim();
    if email.is_empty() || email.len() > MAX_EMAIL_LEN || !email.contains('@') {
        return Err(AppError::Auth("enter a valid email address".into()).to_string());
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(AppError::Auth(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters"
        ))
        .to_string());
    }
    Ok(())
}

fn validate_username(username: &str) -> Result<(), String> {
    let username = username.trim();
    if username.is_empty() || username.chars().count() > MAX_USERNAME_LEN {
        return Err(
            AppError::Auth(format!("username must be 1-{MAX_USERNAME_LEN} characters")).to_string(),
        );
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(
            AppError::Auth("username can use letters, numbers, _, -, and .".into()).to_string(),
        );
    }
    Ok(())
}

fn validate_avatar(avatar: &Option<String>) -> Result<(), String> {
    if let Some(avatar) = avatar {
        let avatar = avatar.trim();
        if !avatar.is_empty() && !(avatar.starts_with("https://") || avatar.starts_with("http://"))
        {
            return Err(
                AppError::Auth("avatar must be a valid http or https URL".into()).to_string(),
            );
        }
    }
    Ok(())
}

/// Create a new user account via SurrealDB Record Auth SIGNUP.
/// Returns the created User record. Persists the JWT token to disk.
#[tauri::command]
pub async fn signup(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    email: String,
    username: String,
    password: String,
) -> Result<User, String> {
    validate_email(&email)?;
    validate_username(&username)?;
    validate_password(&password)?;

    let credentials = surrealdb::opt::auth::Record {
        access: SURREAL_ACCESS.to_string(),
        namespace: SURREAL_NS.to_string(),
        database: SURREAL_DB.to_string(),
        params: serde_json::json!({
            "email": email.trim(),
            "username": username.trim(),
            "password": password,
        }),
    };
    let token = state.db.signup(credentials).await.map_err(into_err)?;
    let token_str = token.access.into_insecure_token();
    save_token(&app_handle, &token_str)?;

    let mut result: Vec<User> = state
        .db
        .query("SELECT * FROM $auth")
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result
        .pop()
        .ok_or_else(|| into_err(AppError::Auth("signup succeeded but $auth not set".into())))
}

/// Authenticate an existing user via SurrealDB Record Auth SIGNIN.
/// Returns the JWT token string. Persists the token to disk.
#[tauri::command]
pub async fn signin(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    email: String,
    password: String,
) -> Result<String, String> {
    validate_email(&email)?;
    validate_password(&password)?;

    let credentials = surrealdb::opt::auth::Record {
        access: SURREAL_ACCESS.to_string(),
        namespace: SURREAL_NS.to_string(),
        database: SURREAL_DB.to_string(),
        params: serde_json::json!({
            "email": email.trim(),
            "password": password,
        }),
    };
    let token_str = state
        .db
        .signin(credentials)
        .await
        .map_err(into_err)?
        .access
        .into_insecure_token();
    save_token(&app_handle, &token_str)?;
    Ok(token_str)
}

/// Clear the current session. Invalidates the token in state and removes it from disk.
#[tauri::command]
pub async fn signout(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    state.db.invalidate().await.map_err(into_err)?;
    clear_token(&app_handle)?;
    Ok(())
}

/// Attempt to restore a previous session from the persisted token on disk.
/// Authenticates the DB connection with the stored JWT.
/// Returns the authenticated User on success, or an error if no token exists
/// or the token is expired/invalid (in which case the stored token is also cleared).
#[tauri::command]
pub async fn restore_session(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<User, String> {
    let token_str = load_token(&app_handle)?
        .ok_or_else(|| AppError::Auth("no saved session".into()).to_string())?;

    match state
        .db
        .authenticate(surrealdb::opt::auth::Token::from(token_str.clone()))
        .await
    {
        Ok(_) => {
            let mut result: Vec<User> = state
                .db
                .query("SELECT * FROM $auth")
                .await
                .map_err(into_err)?
                .take(0)
                .map_err(into_err)?;

            result.pop().ok_or_else(|| {
                into_err(AppError::Auth("session restored but $auth not set".into()))
            })
        }
        Err(_) => {
            let _ = clear_token(&app_handle);
            Err(AppError::Auth("session expired, please sign in again".into()).to_string())
        }
    }
}

/// Fetch the currently authenticated user record.
#[tauri::command]
pub async fn get_me(state: State<'_, AppState>) -> Result<User, String> {
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

/// Update mutable profile fields. Only provided fields are changed.
#[tauri::command]
pub async fn update_profile(
    state: State<'_, AppState>,
    username: Option<String>,
    avatar: Option<String>,
) -> Result<User, String> {
    if let Some(username) = &username {
        validate_username(username)?;
    }
    validate_avatar(&avatar)?;

    let mut result: Vec<User> = state
        .db
        .query(
            "UPDATE $auth SET
                username = $username ?? username,
                avatar   = $avatar   ?? avatar
             RETURN AFTER",
        )
        .bind(("username", username.map(|s| s.trim().to_string())))
        .bind((
            "avatar",
            avatar
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        ))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result
        .pop()
        .ok_or_else(|| into_err(AppError::NotFound("user".into())))
}

/// Search users by username. Returns safe profile fields only.
#[tauri::command]
pub async fn search_users(state: State<'_, AppState>, query: String) -> Result<Vec<User>, String> {
    let query = query.trim();
    if query.chars().count() < 2 {
        return Ok(Vec::new());
    }

    let result: Vec<User> = state
        .db
        .query(
            "SELECT id, username, email, avatar, created FROM user
             WHERE id != $auth AND string::lowercase(username) CONTAINS string::lowercase($query)
             ORDER BY username
             LIMIT 10",
        )
        .bind(("query", query.to_string()))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    Ok(result)
}

/// Return the contacts list for the current user.
#[tauri::command]
pub async fn get_contacts(state: State<'_, AppState>) -> Result<Vec<User>, String> {
    let result: Vec<User> = state
        .db
        .query(
            "SELECT * FROM user
             WHERE id IN (SELECT VALUE target FROM contact WHERE owner = $auth)
             ORDER BY username",
        )
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    Ok(result)
}

/// Add a user to the current user's contact list.
#[tauri::command]
pub async fn add_contact(state: State<'_, AppState>, user_id: String) -> Result<Contact, String> {
    let mut result: Vec<Contact> = state
        .db
        .query("CREATE contact SET owner = $auth, target = type::record('user', $uid)")
        .bind(("uid", user_id))
        .await
        .map_err(into_err)?
        .take(0)
        .map_err(into_err)?;

    result
        .pop()
        .ok_or_else(|| into_err(AppError::NotFound("contact after create".into())))
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn save_token(app: &AppHandle, token: &str) -> Result<(), String> {
    let store = app.store(SESSION_STORE).map_err(|e| e.to_string())?;
    store.set(TOKEN_KEY, serde_json::json!(token));
    store.save().map_err(|e| e.to_string())
}

fn load_token(app: &AppHandle) -> Result<Option<String>, String> {
    let store = app.store(SESSION_STORE).map_err(|e| e.to_string())?;
    Ok(store
        .get(TOKEN_KEY)
        .and_then(|v| v.as_str().map(String::from)))
}

fn clear_token(app: &AppHandle) -> Result<(), String> {
    let store = app.store(SESSION_STORE).map_err(|e| e.to_string())?;
    store.delete(TOKEN_KEY);
    store.save().map_err(|e| e.to_string())
}
