pub mod models;



pub mod error;
pub mod store;
pub mod auth;
pub mod service;
pub mod api;

use std::sync::Arc;
use axum::Router;
use tower_http::cors::CorsLayer;
use crate::store::Store;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Store>,
    pub sessions: Arc<auth::SessionStore>,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, error::AppError> {
        let store = Store::connect(database_url).await?;
        let sessions = Arc::new(auth::SessionStore::new());
        let s = Self { store: Arc::new(store), sessions };
        s.seed_defaults().await?;
        Ok(s)
    }

    async fn seed_defaults(&self) -> Result<(), error::AppError> {
        self.store.ensure_admin("admin", "ADMIN_PASS_REDACTED").await?;
        self.store.ensure_default_rubric().await?;
        if std::env::var("FORCE_SEED").ok().as_deref() == Some("1") {
            // Wipe business data before re-seeding demo. Users and rubrics
            // are preserved (admin user + default rubric).
            sqlx::query(
                "TRUNCATE TABLE scores, issues, interactions, kpis, customers, agents RESTART IDENTITY CASCADE"
            )
            .execute(&self.store.pool)
            .await
            .ok();
        }
        self.store.seed_demo_data().await?;
        self.store.seed_scores_and_issues().await?;
        Ok(())
    }
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/", axum::routing::get(api::serve_index))
        .route("/index.html", axum::routing::get(api::serve_index))
        .route("/static/*path", axum::routing::get(api::serve_static))
        .nest("/api", api::router())
        .layer(axum::middleware::from_fn_with_state(state.clone(), crate::auth::auth_middleware_inner))
        .layer(CorsLayer::permissive())
        .with_state(state)
}







