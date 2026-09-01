use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("auth: {0}")]
    Auth(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) | AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "not_found",
            AppError::BadRequest(_) | AppError::Validation(_) => "bad_request",
            AppError::Auth(_) => "unauthorized",
            AppError::Forbidden(_) => "forbidden",
            AppError::Conflict(_) => "conflict",
            _ => "internal",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = json!({
            "success": false,
            "error_code": self.code(),
            "error": self.to_string(),
        }).to_string();
        (self.status(), [("content-type", "application/json")], body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
