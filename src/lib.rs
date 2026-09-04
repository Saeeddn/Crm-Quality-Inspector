pub mod models;



pub mod error;
pub mod store;
pub mod auth;
pub mod service;
pub mod api;
pub mod openapi;

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
        // Admin credentials from env vars. There is NO default password for security:
        // if neither ADMIN_USERNAME nor ADMIN_PASSWORD is set, the server refuses to start.
        // The first start creates the admin user from these env vars; subsequent starts
        // leave the user alone (use the API to change the password, which then diverges
        // from the env var on purpose).
        let admin_user = std::env::var("ADMIN_USERNAME")
            .map_err(|_| error::AppError::Config(
                "ADMIN_USERNAME env var is required. Set it in .env or your shell.".into()
            ))?;
        let admin_pass = std::env::var("ADMIN_PASSWORD")
            .map_err(|_| error::AppError::Config(
                "ADMIN_PASSWORD env var is required. Set it in .env or your shell.".into()
            ))?;
        if admin_user.trim().is_empty() || admin_pass.len() < 8 {
            return Err(error::AppError::Config(
                "ADMIN_USERNAME and ADMIN_PASSWORD must be set; password ≥ 8 chars".into()
            ));
        }
        // Only seed admin if no users exist (first run). After that, the UI owns
        // user management — changing ADMIN_PASSWORD in .env will NOT overwrite
        // a password that was changed via the API.
        if self.store.list_users().await?.is_empty() {
            self.store.ensure_admin(&admin_user, &admin_pass).await?;
        } else {
            eprintln!("✓ Users already exist; ADMIN_USERNAME/ADMIN_PASSWORD env vars are ignored.");
            eprintln!("  Use the web UI (Users tab) to change passwords.");
        }
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
        // OpenAPI / Swagger UI
        .route("/openapi.json", axum::routing::get(openapi::openapi_json))
        .route("/swagger-ui", axum::routing::get(openapi::swagger_ui))
        .route("/docs", axum::routing::get(openapi::swagger_ui)) // alias
        .nest("/api", api::router())
        .layer(axum::middleware::from_fn_with_state(state.clone(), crate::auth::auth_middleware_inner))
        .layer(CorsLayer::permissive())
        .with_state(state)
}









