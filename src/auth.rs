use crate::error::{AppError, AppResult};
use crate::models::Session;
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
const SESSION_TTL_HOURS: i64 = 12;

pub struct SessionStore {
    map: Mutex<HashMap<String, Session>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self { map: Mutex::new(HashMap::new()) }
    }

    pub fn create(&self, username: &str, is_admin: bool) -> Session {
        let mut buf = [0u8; 32];
        rand::thread_rng().fill(&mut buf);
        let token = hex::encode(buf);
        let now = Utc::now();
        let session = Session {
            token: token.clone(),
            username: username.into(),
            is_admin,
            created_at: now,
            expires_at: now + Duration::hours(SESSION_TTL_HOURS),
        };
        self.map.lock().unwrap().insert(token, session.clone());
        session
    }

    pub fn validate(&self, token: &str) -> Option<Session> {
        let map = self.map.lock().unwrap();
        map.get(token).and_then(|s| {
            if s.expires_at > Utc::now() {
                Some(s.clone())
            } else {
                None
            }
        })
    }
}

#[derive(Clone)]
pub struct CurrentUser {
    pub username: String,
    pub is_admin: bool,
}

pub async fn auth_middleware_inner(
    State(state): State<crate::AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = req.uri().path().to_string();
    // Allow public endpoints
    if path == "/api/auth/login" || path == "/api/health" || !path.starts_with("/api/") {
        return Ok(next.run(req).await);
    }
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());
    let token = match token {
        Some(t) => t,
        None => return Err(AppError::Auth("missing bearer token".into())),
    };
    let session = state
        .sessions
        .validate(&token)
        .ok_or_else(|| AppError::Auth("invalid or expired session".into()))?;
    req.extensions_mut().insert(CurrentUser {
        username: session.username,
        is_admin: session.is_admin,
    });
    Ok(next.run(req).await)
}

pub fn hash_password(password: &str) -> AppResult<String> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    Ok(bcrypt::verify(password, hash)?)
}
