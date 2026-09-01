use crate::auth::{verify_password, CurrentUser};
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::service::Service;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, StatusCode},
    middleware,
    response::{Html, IntoResponse},
    routing::{get, patch, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/me", get(me))
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).patch(patch_agent).delete(delete_agent))
        .route("/customers", get(list_customers).post(create_customer))
        .route("/customers/:id", get(get_customer).patch(patch_customer).delete(delete_customer))
        .route("/interactions", get(list_interactions).post(create_interaction))
        .route("/interactions/:id", get(get_interaction))
        .route("/rubrics", get(list_rubrics).post(create_rubric))
        .route("/scoring/score", post(submit_score))
        .route("/scoring/:id", get(get_score_by_interaction))
        .route("/issues", get(list_issues))
        .route("/issues/:id/resolve", patch(resolve_issue))
        .route("/recommendations", get(list_recommendations))
        .route("/reports/dashboard", get(dashboard))
        .route("/reports/agent/:id", get(agent_report))
}

pub async fn serve_index() -> impl IntoResponse {
    let html = include_str!("../static/index.html");
    Html(html.to_string())
}

pub async fn serve_static(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let safe = path.replace("..", "");
    match tokio::fs::read(format!("static/{safe}")).await {
        Ok(bytes) => {
            let ct = match safe.rsplit('.').next() {
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };
            (StatusCode::OK, [(header::CONTENT_TYPE, ct)], bytes).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

fn ok<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": data }))
}

// ============ Health & Auth ============

pub async fn health() -> Json<serde_json::Value> {
    ok(json!({ "status": "ok" }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = state
        .store
        .get_user(&req.username)
        .await?
        .ok_or_else(|| AppError::Auth("کاربر یافت نشد".into()))?;
    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Auth("رمز عبور اشتباه است".into()));
    }
    let session = state.sessions.create(&user.username, user.is_admin);
    Ok(ok(json!({
        "username": user.username,
        "is_admin": user.is_admin,
        "token": session.token,
        "expires_at": session.expires_at,
    })))
}

pub async fn register(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Auth("فقط مدیر می‌تواند کاربر ایجاد کند".into()));
    }
    if req.username.trim().is_empty() || req.password.len() < 8 {
        return Err(AppError::Validation(
            "نام کاربری نباید خالی و رمز نباید کمتر از ۸ کاراکتر باشد".into(),
        ));
    }
    if state.store.user_exists(&req.username).await? {
        return Err(AppError::Validation("کاربر تکراری است".into()));
    }
    let user = User {
        username: req.username,
        password_hash: crate::auth::hash_password(&req.password)?,
        is_admin: req.is_admin,
        created_at: chrono::Utc::now(),
    };
    state.store.put_user(&user).await?;
    Ok(ok(json!({ "username": user.username })))
}

pub async fn me(Extension(me): Extension<Arc<CurrentUser>>) -> Json<serde_json::Value> {
    ok(json!({ "username": me.username, "is_admin": me.is_admin }))
}

// ============ Agents ============

pub async fn list_agents(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(ok(state.store.list_agents().await?))
}

pub async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let a = state
        .store
        .get_agent(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("کارشناس یافت نشد".into()))?;
    Ok(ok(a))
}

pub async fn create_agent(
    State(state): State<AppState>,
    Json(req): Json<CreateAgentRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.create_agent(req).await?))
}

pub async fn patch_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAgentRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.update_agent(&id, req).await?))
}

pub async fn delete_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    s.delete_agent(&id).await?;
    Ok(ok(json!({ "deleted": true })))
}

// ============ Customers ============

pub async fn list_customers(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(ok(state.store.list_customers().await?))
}

pub async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let c = state
        .store
        .get_customer(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("مشتری یافت نشد".into()))?;
    Ok(ok(c))
}

pub async fn create_customer(
    State(state): State<AppState>,
    Json(req): Json<CreateCustomerRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.create_customer(req).await?))
}

pub async fn patch_customer(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCustomerRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.update_customer(&id, req).await?))
}

pub async fn delete_customer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    s.delete_customer(&id).await?;
    Ok(ok(json!({ "deleted": true })))
}

// ============ Interactions ============

pub async fn list_interactions(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(ok(state.store.list_interactions().await?))
}

pub async fn get_interaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let i = state
        .store
        .get_interaction(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("تعامل یافت نشد".into()))?;
    Ok(ok(i))
}

pub async fn create_interaction(
    State(state): State<AppState>,
    Json(req): Json<CreateInteractionRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.create_interaction(req).await?))
}

// ============ Rubrics ============

pub async fn list_rubrics(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    Ok(ok(state.store.list_rubrics().await?))
}

pub async fn create_rubric(
    State(state): State<AppState>,
    Json(req): Json<CreateRubricRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.create_rubric(req).await?))
}

// ============ Scoring ============

pub async fn submit_score(
    State(state): State<AppState>,
    Json(req): Json<ScoreRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.score_interaction(req).await?))
}

pub async fn get_score_by_interaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    match state.store.get_score_by_interaction(&id).await? {
        Some(s) => Ok(ok(s)),
        None => Err(AppError::NotFound("ارزیابی یافت نشد".into())),
    }
}

// ============ Issues ============

pub async fn list_issues(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let mut v = state.store.list_issues().await?;
    if let Some(sev) = &q.severity { v.retain(|x| &x.severity == sev); }
    if let Some(st) = &q.status { v.retain(|x| &x.status == st); }
    if let Some(aid) = &q.agent_id { v.retain(|x| &x.agent_id == aid); }
    Ok(ok(v))
}

pub async fn resolve_issue(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ResolveIssueRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.resolve_issue(&id, req).await?))
}

// ============ Recommendations ============

pub async fn list_recommendations(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.compute_recommendations().await?))
}

// ============ Dashboard ============

pub async fn dashboard(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.dashboard().await?))
}

pub async fn agent_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.agent_report(&id).await?))
}
