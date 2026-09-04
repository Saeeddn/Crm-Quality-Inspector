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

    // DATABASE_URL must be provided via env var. There is no insecure default —
    // refusing to start is the only safe behavior for a public image.
    let database_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            eprintln!("❌ DATABASE_URL env var is required. Refusing to start.");
            eprintln!("   Set it in .env, your shell, or docker-compose environment.");
            eprintln!("   Example: postgres://crm_quality:STRONG_PASSWORD@db:5432/crm_quality_inspector");
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "DATABASE_URL not set")
        })
        .expect("DATABASE_URL required");
    // Basic sanity check — refuse to start with the legacy example/weak passwords.
    if database_url.contains("PG_USER_REDACTED") || database_url.contains("ssdssd")
        || database_url.contains("admin1234") || database_url.contains("example") {
        eprintln!("❌ DATABASE_URL contains a placeholder/weak password. Refusing to start.");
        eprintln!("   Generate a strong password: openssl rand -base64 24");
        std::process::exit(2);
    }
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
