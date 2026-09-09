use crate::auth::{verify_password, CurrentUser};
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::service::Service;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::{header, StatusCode},
    middleware,
    response::{Html, IntoResponse, Json as AxumJson},
    routing::{get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use chrono::Utc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/me", get(me))
        .route("/users", get(list_users).post(create_user))
        .route("/users/:username", patch(update_user_handler).delete(delete_user_handler))
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).patch(patch_agent).delete(delete_agent))
        .route("/customers", get(list_customers).post(create_customer))
        .route("/customers/risk", get(list_customers_risk))
        .route("/customers/:id/risk", get(get_customer_risk))
        .route("/customers/:id", get(get_customer).patch(patch_customer).delete(delete_customer))
        .route("/interactions", get(list_interactions).post(create_interaction))
        .route("/interactions/:id", get(get_interaction))
        .route("/metrics", get(list_metrics).post(create_metric))
        .route("/metrics/:id", patch(update_metric).delete(delete_metric))
        .route("/rubrics", get(list_rubrics).post(create_rubric))
        .route("/scoring/score", post(submit_score))
        .route("/scoring/auto/:interaction_id", post(auto_score_interaction))
        .route("/scoring/:id", get(get_score_by_interaction))
        .route("/scores", get(list_scores))
        .route("/issues", get(list_issues).post(create_issue_handler))
        .route("/issues/:id/resolve", patch(resolve_issue))
        .route("/recommendations", get(list_recommendations))
        .route("/kpis", get(list_kpis).post(create_kpi))
        .route("/kpis/seed", post(seed_kpis))
        .route("/kpis/:id", patch(toggle_kpi).delete(delete_kpi))
        .route("/kpis/measure/:interaction_id", get(measure_interaction))
        .route("/reports/dashboard", get(dashboard))
        .route("/reports/agent/:id", get(agent_report))
        // Coaching Plans (closed-loop QA)
        .route("/coaching/plans", get(list_coaching_plans_handler).post(create_coaching_plan_handler))
        .route("/coaching/plans/:id", get(get_coaching_plan_handler).patch(patch_coaching_plan_handler))
        .route("/coaching/plans/:id/submit", post(submit_coaching_plan_handler))
        .route("/coaching/plans/:id/acknowledge", post(acknowledge_coaching_plan_handler))
        .route("/coaching/plans/:id/close", post(close_coaching_plan_handler))
        .route("/coaching/plans/:id/escalate", post(escalate_coaching_plan_handler))
        .route("/coaching/plans/:id/resume", post(resume_coaching_plan_handler))
        .route("/coaching/summary", get(coaching_summary_handler))
        .route("/coaching/plans/:id/follow-ups", post(record_coaching_follow_up_handler))
                // =================== Calibration Sessions ===================
                .route("/calibration/sessions", get(list_calibration_sessions_handler))
                .route("/calibration/sessions", post(create_calibration_session_handler))
                .route("/calibration/sessions/:id", get(get_calibration_session_handler))
                .route("/calibration/sessions/:id/transition", post(transition_calibration_session_handler))
                .route("/calibration/sessions/:id/score", post(submit_calibration_score_handler))
                .route("/calibration/sessions/:id/my-status", get(calibration_my_status_handler))
                .route("/calibration/sessions/:id/decisions", post(save_calibration_decisions_handler))
                .route("/calibration/rubrics/:rubric_id/history", get(calibration_rubric_history_handler))
                .route("/calibration/summary", get(calibration_summary_handler))
                // =================== Audit Log ===================
                .route("/audit/logs", get(list_audit_logs_handler))
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
                Some("html") => "text/html; charset=utf-8",
                Some("htm") => "text/html; charset=utf-8",
                Some("png") => "image/png",
                Some("svg") => "image/svg+xml",
                Some("json") => "application/json",
                _ => "text/plain",
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

pub async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    // Real connectivity check
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.store.pool)
        .await
        .is_ok();
    tracing::debug!(db_ok, "health check");
    if db_ok {
        ok(json!({ "status": "ok", "database": "connected" }))
    } else {
        ok(json!({ "status": "degraded", "database": "disconnected" }))
    }
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
    tracing::info!(username = %req.username, "login attempt");
    if !verify_password(&req.password, &user.password_hash)? {
        tracing::warn!(username = %req.username, "login failed: wrong password");
        return Err(AppError::Auth("رمز عبور اشتباه است".into()));
    }
    let session = state.sessions.create(&user.username, user.is_admin);
    tracing::info!(username = %user.username, is_admin = user.is_admin, "login successful");
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

// ============ Users ============

pub async fn list_users(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("فقط مدیر سیستم دسترسی دارد".into()));
    }
    let s = Service::new(&state.store);
    Ok(ok(s.list_users().await?))
}

pub async fn create_user(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("فقط مدیر سیستم دسترسی دارد".into()));
    }
    let s = Service::new(&state.store);
    Ok(ok(s.create_user(req).await?))
}

pub async fn update_user_handler(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
    Path(username): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("فقط مدیر سیستم دسترسی دارد".into()));
    }
    let s = Service::new(&state.store);
    Ok(ok(s.update_user(&username, req).await?))
}

pub async fn delete_user_handler(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
    Path(username): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("فقط مدیر سیستم دسترسی دارد".into()));
    }
    let s = Service::new(&state.store);
    s.delete_user(&username).await?;
    Ok(ok(serde_json::json!({ "deleted": username })))
}

// ============ Agents ============

pub async fn list_agents(
    State(state): State<AppState>,
    Query(q): Query<crate::models::ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit: i64 = q.limit.unwrap_or(10).max(1);
    let page: i64 = q.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let (items, total) = state.store.list_agents_paginated(limit, offset).await?;
    let total_pages = if limit > 0 { (total + limit - 1) / limit } else { 1 };
    Ok(ok(json!({
        "items": items,
        "total": total,
        "page": page,
        "limit": limit,
        "total_pages": total_pages
    })))
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

pub async fn list_customers(
    State(state): State<AppState>,
    Query(q): Query<crate::models::ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit: i64 = q.limit.unwrap_or(10).max(1);
    let page: i64 = q.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let (items, total) = state.store.list_customers_paginated(limit, offset).await?;
    let total_pages = if limit > 0 { (total + limit - 1) / limit } else { 1 };
    Ok(ok(json!({
        "items": items,
        "total": total,
        "page": page,
        "limit": limit,
        "total_pages": total_pages
    })))
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

pub async fn list_customers_risk(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.customer_risk_scores().await?))
}

pub async fn get_customer_risk(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    let all = s.customer_risk_scores().await?;
    let found = all
        .into_iter()
        .find(|c| c.customer_id == id)
        .ok_or_else(|| AppError::NotFound("این مشتری یافت نشد یا تعاملی ندارد".into()))?;
    Ok(ok(found))
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
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let limit: i64 = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(10);
    let page: i64 = params.get("page").and_then(|s| s.parse().ok()).unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let (interactions, total) = state.store.list_interactions_paginated(limit, offset).await?;
    let total_pages = if limit > 0 { (total + limit - 1) / limit } else { 1 };
    Ok(ok(json!({
        "items": interactions,
        "total": total,
        "page": page,
        "limit": limit,
        "total_pages": total_pages,
    })))
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
    Extension(user): Extension<Arc<CurrentUser>>,
    Json(req): Json<ScoreRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    let result = s.score_interaction(req).await?;
    let result_id = result.id.clone();
    let details = serde_json::json!({ "interaction_id": result_id });
    let _ = state.store.log_audit(&AuditLog {
        id: format!("audit_{}", Utc::now().timestamp_millis()),
        username: user.username.clone(),
        action: "score_interaction".to_string(),
        resource_type: "score".to_string(),
        resource_id: Some(result_id),
        summary: Some("امتیازدهی تعامل".into()),
        details: Some(details),
        created_at: Utc::now(),
    }).await.unwrap_or_default();
    Ok(ok(result))
}

pub async fn get_score_by_interaction(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // Empty body (success: true, data: null) when not yet scored —
    // this is an expected state for unscored interactions, not an error.
    match state.store.get_score_by_interaction(&id).await? {
        Some(s) => Ok(ok(s)),
        None => Ok(ok(serde_json::Value::Null)),
    }
}

// Bulk fetch all scores — used by dashboard charts to avoid N+1 calls
pub async fn list_scores(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let scores = state.store.scan_scores().await?;
    Ok(ok(scores))
}

// ============ Issues ============

pub async fn list_issues(
    State(state): State<AppState>,
    Query(q): Query<crate::models::ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit: i64 = q.limit.unwrap_or(10).max(1);
    let page: i64 = q.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;
    let (items, total) = state.store.list_issues_paginated(
        limit, offset,
        q.severity.as_deref(),
        q.status.as_deref(),
        q.agent_id.as_deref(),
    ).await?;
    let total_pages = if limit > 0 { (total + limit - 1) / limit } else { 1 };
    Ok(ok(json!({
        "items": items,
        "total": total,
        "page": page,
        "limit": limit,
        "total_pages": total_pages
    })))
}

pub async fn create_issue_handler(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
    Json(req): Json<CreateIssueRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    let interaction_id = req.interaction_id.clone();
    let severity = req.severity.clone();
    let category = req.category.clone();
    let issue = s.create_issue(
        interaction_id.clone(),
        req.agent_id,
        severity,
        category,
        req.description,
        if req.status.is_empty() { "باز".into() } else { req.status },
    ).await?;
    let _ = state.store.log_audit(&AuditLog {
        id: format!("audit_{}", Utc::now().timestamp_millis()),
        username: me.username.clone(),
        action: "create_issue".to_string(),
        resource_type: "issue".to_string(),
        resource_id: Some(issue.id.clone()),
        summary: Some(format!("ایجاد ایراد برای تعامل {}", interaction_id)),
        details: Some(serde_json::json!({ "severity": req.severity, "category": req.category })),
        created_at: Utc::now(),
    }).await.unwrap_or_default();
    let _ = me.username;
    Ok(ok(issue))
}

pub async fn resolve_issue(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
    Path(id): Path<String>,
    Json(req): Json<ResolveIssueRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    let issue = s.resolve_issue(&id, req).await?;
    let issue_id = issue.id.clone();
    let issue_status = issue.status.clone();
    let _ = state.store.log_audit(&AuditLog {
        id: format!("audit_{}", Utc::now().timestamp_millis()),
        username: me.username.clone(),
        action: "resolve_issue".to_string(),
        resource_type: "issue".to_string(),
        resource_id: Some(issue_id),
        summary: Some("بستن ایراد".to_string()),
        details: Some(serde_json::json!({ "resolution": issue_status })),
        created_at: Utc::now(),
    }).await.unwrap_or_default();
    Ok(ok(issue))
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

// ============ Metrics ============

pub async fn list_metrics(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.list_metrics().await?))
}

pub async fn create_metric(
    State(state): State<AppState>,
    Json(req): Json<CreateMetricRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.create_metric(req).await?))
}

pub async fn update_metric(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateMetricRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.update_metric(&id, req).await?))
}

pub async fn delete_metric(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    s.delete_metric(&id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}

// =================== KPIs ===================

pub async fn list_kpis(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.list_kpis().await?))
}

pub async fn create_kpi(
    State(state): State<AppState>,
    Json(req): Json<KpiCreateReq>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.create_kpi(req).await?))
}

pub async fn seed_kpis(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.seed_default_kpis().await?))
}

pub async fn toggle_kpi(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ToggleKpiRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.toggle_kpi(&id, req.active).await?))
}

pub async fn delete_kpi(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    s.delete_kpi(&id).await?;
    Ok(ok(serde_json::json!({ "deleted": true })))
}

pub async fn measure_interaction(
    State(state): State<AppState>,
    Path(interaction_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    Ok(ok(s.auto_score(&interaction_id).await?))
}

pub async fn auto_score_interaction(
    State(state): State<AppState>,
    Path(interaction_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let s = Service::new(&state.store);
    let score = s.auto_score_and_save(&interaction_id).await?;
    Ok(ok(score))
}

// ============ Coaching Plans (Closed-Loop QA) ============

#[derive(Deserialize)]
pub struct CoachingListQuery {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

pub async fn list_coaching_plans_handler(
    State(state): State<AppState>,
    Query(q): Query<CoachingListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = q.limit.unwrap_or(25).max(1).min(200);
    let offset = q.offset.unwrap_or(0).max(0);
    let (items, total) = state
        .store
        .list_coaching_plans(
            q.agent_id.as_deref(),
            q.status.as_deref(),
            limit,
            offset,
        )
        .await?;
    let total_pages = if limit > 0 { (total + limit - 1) / limit } else { 0 };
    Ok(ok(json!({
        "items": items,
        "total": total,
        "page": if limit > 0 { offset / limit + 1 } else { 1 },
        "limit": limit,
        "total_pages": total_pages,
    })))
}

pub async fn create_coaching_plan_handler(
    Extension(me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Json(req): Json<CoachingPlanCreate>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    let id = state.store.next_id("coaching_plans_id_seq").await?;
    let plan = CoachingPlan {
        id,
        agent_id: req.agent_id,
        interaction_id: req.interaction_id,
        created_by: me.username.clone(),
        created_at: chrono::Utc::now(),
        coaching_theme: req.coaching_theme,
        behavior_gap: req.behavior_gap,
        evidence: req.evidence,
        root_cause: req.root_cause,
        customer_impact: req.customer_impact,
        practice_activity: req.practice_activity,
        success_metric: req.success_metric,
        follow_up_due_at: req.follow_up_due_at,
        follow_up_review_count: req.follow_up_review_count,
        status: "draft".into(),
        acknowledged_at: None,
        acknowledged_note: None,
        closed_at: None,
        closed_outcome: None,
        escalated_at: None,
    };
    state.store.create_coaching_plan(&plan).await?;
    Ok(ok(json!({ "id": plan.id, "status": plan.status })))
}

pub async fn get_coaching_plan_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let plan = state
        .store
        .get_coaching_plan(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("coaching_plan".to_string()))?;
    let follow_ups = state.store.list_coaching_follow_ups(&id).await?;
    Ok(ok(json!({ "plan": plan, "follow_ups": follow_ups })))
}

pub async fn patch_coaching_plan_handler(
    Extension(me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(patch): Json<CoachingPlanPatch>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    state.store.patch_coaching_plan(&id, &patch).await?;
    Ok(ok(json!({ "id": id, "status": "patched" })))
}

pub async fn submit_coaching_plan_handler(
    Extension(_me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    state.store.transition_coaching_plan(&id, "submit", None, None).await?;
    Ok(ok(json!({ "id": id, "status": "pending_acknowledgement" })))
}

pub async fn acknowledge_coaching_plan_handler(
    Extension(me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AcknowledgeRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let plan = state
        .store
        .get_coaching_plan(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("coaching_plan".to_string()))?;
    // Agents can acknowledge only their own plan; admins can do any.
    if !me.is_admin && plan.agent_id != me.username {
        return Err(AppError::Forbidden("not your plan".into()));
    }
    state
        .store
        .transition_coaching_plan(&id, "acknowledge", None, req.note.as_deref())
        .await?;
    let _ = state.store.log_audit(&AuditLog {
        id: format!("audit_{}", Utc::now().timestamp_millis()),
        username: me.username.clone(),
        action: "acknowledge_coaching".to_string(),
        resource_type: "coaching_plan".to_string(),
        resource_id: Some(id.clone()),
        summary: Some(format!("تأیید برنامه آموزشی")),
        details: Some(serde_json::json!({ "agent_id": plan.agent_id })),
        created_at: Utc::now(),
    }).await.unwrap_or_default();
    Ok(ok(json!({ "id": id, "status": "acknowledged" })))
}

pub async fn close_coaching_plan_handler(
    Extension(me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CloseRequest>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    let plan_id = id.clone();
    state
        .store
        .transition_coaching_plan(&id, "close", Some(&req.outcome), None)
        .await?;
    let _ = state.store.log_audit(&AuditLog {
        id: format!("audit_{}", Utc::now().timestamp_millis()),
        username: me.username.clone(),
        action: "close_coaching".to_string(),
        resource_type: "coaching_plan".to_string(),
        resource_id: Some(plan_id.clone()),
        summary: Some(format!("بستن برنامه آموزشی با نتیجه {}", req.outcome)),
        details: Some(serde_json::json!({ "outcome": req.outcome })),
        created_at: Utc::now(),
    }).await.unwrap_or_default();
    Ok(ok(json!({ "id": plan_id, "status": "closed", "outcome": req.outcome })))
}

pub async fn escalate_coaching_plan_handler(
    Extension(me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    state.store.transition_coaching_plan(&id, "escalate", None, None).await?;
    Ok(ok(json!({ "id": id, "status": "escalated" })))
}

pub async fn resume_coaching_plan_handler(
    Extension(me): Extension<Arc<CurrentUser>>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    state.store.transition_coaching_plan(&id, "resume", None, None).await?;
    Ok(ok(json!({ "id": id, "status": "in_progress" })))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FollowUpCreate {
    pub interaction_id: String,
    #[serde(default)]
    pub criterion_scores: std::collections::HashMap<String, f64>,
    pub overall_score: f64,
    pub success: bool,
}

pub async fn record_coaching_follow_up_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<FollowUpCreate>,
) -> AppResult<Json<serde_json::Value>> {
    let fu = crate::models::CoachingFollowUp {
        id: state.store.next_id("coaching_follow_ups_id_seq").await?,
        plan_id: id,
        interaction_id: req.interaction_id,
        measured_at: chrono::Utc::now(),
        criterion_scores: req.criterion_scores,
        overall_score: req.overall_score,
        success: req.success,
    };
    state.store.record_coaching_follow_up(&fu).await?;
    Ok(ok(json!({ "id": fu.id, "status": "recorded" })))
}

pub async fn coaching_summary_handler(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let summary = state.store.get_coaching_summary().await?;
    Ok(ok(summary))
}

// ============ Calibration Sessions ============

#[derive(Deserialize)]
pub struct CalibrationListQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub rubric_id: Option<String>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

pub async fn list_calibration_sessions_handler(
    State(state): State<AppState>,
    Query(q): Query<CalibrationListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = q.limit.unwrap_or(25).max(1).min(200);
    let offset = q.offset.unwrap_or(0).max(0);
    let (items, total) = state.store
        .list_calibration_sessions(q.status.as_deref(), q.rubric_id.as_deref(), limit, offset)
        .await?;
    Ok(ok(json!({ "items": items, "total": total, "offset": offset, "limit": limit })))
}

pub async fn create_calibration_session_handler(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<CurrentUser>>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    if !user.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    let session = CalibrationSession {
        id: state.store.next_id("calibration_sessions_id_seq").await?,
        name: body["name"].as_str().unwrap_or("").to_string(),
        status: "draft".to_string(),
        rubric_id: body["rubric_id"].as_str().unwrap_or("").to_string(),
        reviewer_usernames: body["reviewer_usernames"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).map(String::from).collect())
            .unwrap_or_default(),
        sample_interaction_ids: body["sample_interaction_ids"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).map(String::from).collect())
            .unwrap_or_default(),
        target_agreement_rate: body["target_agreement_rate"].as_f64(),
        min_reviewers_per_interaction: body["min_reviewers_per_interaction"].as_u64().unwrap_or(2) as u32,
        deadline_at: body["deadline_at"].as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| AppError::BadRequest("deadline_at is required".into()))?,
        meeting_started_at: None,
        facilitator_id: Some(user.username.clone()),
        agreement_rate: None,
        variance_per_criterion: None,
        created_at: Utc::now(),
    };
    state.store.create_calibration_session(&session).await?;
    Ok(ok(session))
}

pub async fn get_calibration_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let session = state.store.get_calibration_session(&session_id).await?
        .ok_or_else(|| AppError::NotFound("calibration_session".to_string()))?;
    Ok(ok(session))
}

pub async fn transition_calibration_session_handler(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<CurrentUser>>,
    Path(session_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    if !user.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    let to_status = body["to_status"].as_str().ok_or_else(|| AppError::BadRequest("to_status required".into()))?;
    let mut session = state.store.get_calibration_session(&session_id).await?
        .ok_or_else(|| AppError::NotFound("calibration_session".to_string()))?;
    match to_status {
        "start" => { if session.status != "draft" { return Err(AppError::BadRequest("only draft can start".into())); } session.status = "scoring".to_string(); }
        "begin-meeting" => { if session.status != "scoring" { return Err(AppError::BadRequest("must be scoring".into())); } session.status = "in_session".to_string(); session.meeting_started_at = Some(Utc::now()); }
        "complete" => {
            if session.status != "in_session" { return Err(AppError::BadRequest("must be in_session".into())); }
            let (rate, variance) = state.store.compute_calibration_summary(&session_id).await?;
            session.agreement_rate = rate;
            session.variance_per_criterion = variance;
            session.status = "completed".to_string();
        }
        "cancel" => { session.status = "cancelled".to_string(); }
        _ => return Err(AppError::BadRequest(format!("invalid status: {}", to_status))),
    }
    state.store.create_calibration_session(&session).await?; // update via upsert
    Ok(ok(session))
}

pub async fn submit_calibration_score_handler(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<CurrentUser>>,
    Path(session_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    let session = state.store.get_calibration_session(&session_id).await?
        .ok_or_else(|| AppError::NotFound("calibration_session".to_string()))?;
    if !session.reviewer_usernames.contains(&user.username) {
        return Err(AppError::Forbidden("not a reviewer".into()));
    }
    let interaction_id = body["interaction_id"].as_str().ok_or_else(|| AppError::BadRequest("interaction_id required".into()))?;
    let criterion_scores: serde_json::Map<String, serde_json::Value> = body["criterion_scores"]
        .as_object().cloned().ok_or_else(|| AppError::BadRequest("criterion_scores required".into()))?;
    let overall_score: f64 = body["overall_score"].as_f64().ok_or_else(|| AppError::BadRequest("overall_score required".into()))?;
    let notes = body["notes"].as_str().map(String::from);
    let score = CalibrationScore {
        id: format!("{}-{}-{}", session_id, interaction_id, user.username),
        session_id: session_id.clone(),
        interaction_id: interaction_id.to_string(),
        reviewer: user.username.clone(),
        submitted_at: Utc::now(),
        criterion_scores: criterion_scores.into(),
        overall_score,
        notes,
    };
    state.store.submit_calibration_score(&score).await?;
    let _ = state.store.log_audit(&AuditLog {
        id: format!("audit_{}", Utc::now().timestamp_millis()),
        username: user.username.clone(),
        action: "submit_calibration_score".to_string(),
        resource_type: "calibration_score".to_string(),
        resource_id: Some(score.id),
        summary: Some(format!("امتیاز کالیبراسیون برای تعامل {}", interaction_id)),
        details: Some(serde_json::json!({ "session_id": session_id, "overall_score": overall_score })),
        created_at: Utc::now(),
    }).await.unwrap_or_default();
    Ok(ok(serde_json::json!({ "submitted": true })))
}

pub async fn calibration_my_status_handler(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<CurrentUser>>,
    Path(session_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let (submitted, total) = state.store.get_my_submission_status(&session_id, &user.username).await?;
    Ok(ok(serde_json::json!({ "submitted_count": submitted, "total_expected": total })))
}

pub async fn save_calibration_decisions_handler(
    State(state): State<AppState>,
    Extension(user): Extension<Arc<CurrentUser>>,
    Path(session_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    if !user.is_admin {
        return Err(AppError::Forbidden("admin only".into()));
    }
    let decisions: Vec<serde_json::Value> = body["decisions"].as_array()
        .ok_or_else(|| AppError::BadRequest("decisions array required".into()))?
        .clone();
    for d in decisions.iter() {
        let decision = CalibrationDecision {
            id: format!("{}-{}-{}", session_id, d["criterion_id"], Utc::now().timestamp()),
            session_id: session_id.clone(),
            criterion_id: d["criterion_id"].as_str().ok_or_else(|| AppError::BadRequest("criterion_id required".into()))?.to_string(),
            agreed_interpretation: d["agreed_interpretation"].as_str().ok_or_else(|| AppError::BadRequest("agreed_interpretation required".into()))?.to_string(),
            example_interaction_id: d["example_interaction_id"].as_str().map(String::from),
            rubric_edit_proposed: d["rubric_edit_proposed"].as_str().map(String::from),
            created_at: Utc::now(),
        };
        state.store.save_calibration_decisions(&decision).await?;
    }
    Ok(ok(serde_json::json!({ "saved": decisions.len() })))
}

pub async fn calibration_rubric_history_handler(
    State(state): State<AppState>,
    Path(rubric_id): Path<String>,
    Query(q): Query<CalibrationListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = q.limit.unwrap_or(25).max(1).min(200);
    let offset = q.offset.unwrap_or(0).max(0);
    let (decisions, total) = state.store.get_rubric_decision_history(&rubric_id, limit, offset).await?;
    Ok(ok(json!({ "decisions": decisions, "total": total })))
}

pub async fn calibration_summary_handler(
    State(state): State<AppState>,
    Query(q): Query<CalibrationListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let sessions = state.store.list_calibration_sessions(q.status.as_deref(), None, 100, 0).await?.0;
    let mut trend = Vec::new();
    for s in sessions {
        if s.status == "completed" && s.agreement_rate.is_some() {
            trend.push(serde_json::json!({
                "date": s.created_at.format("%Y-%m-%d").to_string(),
                "agreement_rate": s.agreement_rate,
                "variance": s.variance_per_criterion
            }));
        }
    }
    trend.sort_by(|a, b| b["date"].as_str().cmp(&a["date"].as_str()));
    Ok(ok(serde_json::json!({ "trend": trend })))
}

// =================== AUDIT LOG ===================

/// ترجمه action enum به رشته قابل خواندن فارسی
fn audit_action_label(action: &str) -> &str {
    match action {
        "create_interaction" => "ایجاد تعامل",
        "score_interaction" => "امتیازدهی",
        "create_issue" => "ایجاد ایراد",
        "resolve_issue" => "بستن ایراد",
        "create_agent" => "ایجاد کارشناس",
        "update_agent" => "ویرایش کارشناس",
        "delete_agent" => "حذف کارشناس",
        "create_customer" => "ایجاد مشتری",
        "update_customer" => "ویرایش مشتری",
        "delete_customer" => "حذف مشتری",
        "create_rubric" => "ایجاد روبریک",
        "update_rubric" => "ویرایش روبریک",
        "create_coaching_plan" => "ایجاد برنامه آموزشی",
        "update_coaching_plan" => "ویرایش برنامه آموزشی",
        "acknowledge_coaching" => "تأیید برنامه آموزشی",
        "close_coaching" => "بستن برنامه آموزشی",
        "create_calibration_session" => "ایجاد جلسه کالیبراسیون",
        "update_calibration_session" => "ویرایش جلسه کالیبراسیون",
        "submit_calibration_score" => "ثبت امتیاز کالیبراسیون",
        "create_user" => "ایجاد کاربر",
        "update_user" => "ویرایش کاربر",
        "delete_user" => "حذف کاربر",
        "login" => "ورود",
        _ => action,
    }
}

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub action: Option<String>,
    pub username: Option<String>,
    pub resource_type: Option<String>,
}

pub async fn list_audit_logs_handler(
    State(state): State<AppState>,
    Extension(me): Extension<Arc<CurrentUser>>,
    Query(q): Query<AuditLogQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if !me.is_admin {
        return Err(AppError::Forbidden("فقط مدیر سیستم دسترسی دارد".into()));
    }
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(50).min(200).max(1);
    let offset = (page - 1) * limit;
    let (items, total) = state.store.list_audit_logs(
        limit, offset,
        q.action.as_deref(),
        q.username.as_deref(),
        q.resource_type.as_deref(),
    ).await?;
    let items: Vec<serde_json::Value> = items.into_iter().map(|a| {
        serde_json::json!({
            "id": a.id,
            "username": a.username,
            "action": a.action,
            "action_label": audit_action_label(&a.action),
            "resource_type": a.resource_type,
            "resource_id": a.resource_id,
            "summary": a.summary,
            "details": a.details,
            "created_at": a.created_at.to_rfc3339(),
        })
    }).collect();
    Ok(ok(json!({
        "items": items,
        "total": total,
        "page": page,
        "limit": limit,
        "total_pages": if limit > 0 { (total + limit - 1) / limit } else { 1 },
    })))
}
