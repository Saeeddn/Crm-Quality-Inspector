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
        //
        // Env vars are BOOTSTRAP ONLY. They are read once on the very first start to
        // create the initial admin user. After that, the web UI owns user management
        // and changing ADMIN_PASSWORD in .env has NO EFFECT on existing users.
        //
        // Why bootstrap-only (not UPSERT): if we synced .env → DB on every restart,
        // any password an operator changed via the UI would silently revert on the
        // next deploy/restart. That's a security regression disguised as convenience.
        let admin_user = std::env::var("ADMIN_USERNAME")
            .map_err(|_| error::AppError::Config(
                "ADMIN_USERNAME env var is required. Set it in .env or your shell.".into()
            ))?;
        let admin_pass = std::env::var("ADMIN_PASSWORD")
            .map_err(|_| error::AppError::Config(
                "ADMIN_PASSWORD env var is required. Set it in .env or your shell.".into()
            ))?;
        if admin_user.trim().is_empty() {
            return Err(error::AppError::Config(
                "ADMIN_USERNAME env var must be set and non-empty".into()
            ));
        }
        if admin_pass.len() < 12 {
            return Err(error::AppError::Config(
                "ADMIN_PASSWORD must be at least 12 characters (use `openssl rand -base64 18`)".into()
            ));
        }
        // Reject well-known weak passwords so a misconfigured .env doesn't ship a
        // guessable admin login to production.
        let lower = admin_pass.to_ascii_lowercase();
        for weak in &["admin", "password", "123456", "admin1234", "letmein", "qwerty"] {
            if lower.contains(weak) {
                return Err(error::AppError::Config(format!(
                    "ADMIN_PASSWORD contains the weak substring '{}'. Pick a strong random password.",
                    weak
                )));
            }
        }
        // Bootstrap-only: seed admin only on the very first start.
        // After that, ADMIN_USERNAME/ADMIN_PASSWORD env vars are ignored — use the
        // web UI (Users tab) to change passwords.
        if self.store.list_users().await?.is_empty() {
            self.store.ensure_admin(&admin_user, &admin_pass).await?;
            eprintln!("✓ Seeded initial admin user '{}' from ADMIN_USERNAME env var.", admin_user);
            eprintln!("  On subsequent restarts, ADMIN_PASSWORD env var is ignored —");
            eprintln!("  use the web UI Users tab to change passwords.");
        } else {
            eprintln!("✓ Users already exist; ADMIN_USERNAME/ADMIN_PASSWORD env vars are ignored.");
            eprintln!("  Use the web UI Users tab to change passwords.");
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









