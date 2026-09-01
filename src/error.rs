use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("bcrypt: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),

    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("auth: {0}")]
    Auth(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) | AppError::Auth(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({"success": false, "error": self.to_string()}).to_string();
        (self.status(), [("content-type", "application/json")], body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
