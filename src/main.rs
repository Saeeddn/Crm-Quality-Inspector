use std::net::SocketAddr;
use crm_qi::{build_app, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Force flush stderr before anything else, so we see startup output
    // even if the binary panics or exits silently.
    eprintln!("[boot] starting CRM Quality Inspector");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    // Load .env file (if present) into process env vars.
    // Variables already set in the actual environment take precedence over .env.
    // Safe to call if the file doesn't exist (e.g. in production with real env vars).
    let _ = dotenvy::dotenv();
    eprintln!("[boot] dotenv loaded");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    // Initialize tracing — write to stderr, line-buffered so it shows up in docker logs
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false).with_writer(std::io::stderr))
        .init();
    eprintln!("[boot] tracing initialized");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    // DATABASE_URL must be provided via env var in production.
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        eprintln!("⚠️  DATABASE_URL not set; using insecure localhost default (DO NOT use in production)");
        "postgres://PG_USER_REDACTED:***@127.0.0.1:5432/crm_quality_inspector".to_string()
    });
    eprintln!("[boot] DATABASE_URL configured, connecting...");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    let state = AppState::new(&database_url).await.expect("failed to init state");
    eprintln!("[boot] database connected, seeding defaults...");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    let app = build_app(state);

    // SERVER_ADDR env var lets us bind to 0.0.0.0 inside Docker / behind a reverse proxy.
    // Default to 0.0.0.0:3000 so a release build is reachable both locally and in containers
    // without requiring every operator to remember to set the env var.
    let addr: SocketAddr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("SERVER_ADDR must be a valid socket address like 0.0.0.0:3000");

    eprintln!("\n  CRM Quality Inspector");
    eprintln!("  listening on http://{}\n", addr);
    let _ = std::io::Write::flush(&mut std::io::stderr());

    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind failed");
    eprintln!("[boot] bound, serving...");
    let _ = std::io::Write::flush(&mut std::io::stderr());
    axum::serve(listener, app).await.expect("server failed");
}
