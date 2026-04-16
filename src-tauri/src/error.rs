use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(#[from] surrealdb::Error),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Subscription error: {0}")]
    Subscription(String),
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

/// Convert any error that's Into<AppError> into the String Tauri commands require.
/// Usage: `.map_err(into_err)`
pub fn into_err<E: Into<AppError>>(e: E) -> String {
    e.into().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_converts_to_string() {
        let e = AppError::Auth("bad credentials".to_string());
        let s: String = into_err(e);
        assert_eq!(s, "Auth error: bad credentials");
    }

    #[test]
    fn not_found_error_converts_to_string() {
        let e = AppError::NotFound("room".to_string());
        let s: String = into_err(e);
        assert_eq!(s, "Not found: room");
    }
}
