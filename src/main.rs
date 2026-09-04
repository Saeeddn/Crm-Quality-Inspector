use std::net::SocketAddr;
use crm_qi::{build_app, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env file (if present) into process env vars.
    // Variables already set in the actual environment take precedence over .env.
    // Safe to call if the file doesn't exist (e.g. in production with real env vars).
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // DATABASE_URL must be provided via env var in production.
    // Format: postgres://user:pass@host:port/dbname
    // For local dev, put DATABASE_URL in .env (never commit .env).
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("⚠️  DATABASE_URL not set; using insecure localhost default (DO NOT use in production)");
        "postgres://PG_USER_REDACTED:***@127.0.0.1:5432/crm_quality_inspector".to_string()
    });
    let state = AppState::new(&database_url).await.expect("failed to init state");

    let app = build_app(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("\n  CRM Quality Inspector");
    println!("  http://{}/\n", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}
