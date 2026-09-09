use std::sync::Mutex;
use std::net::SocketAddr;
use crm_qi::{build_app, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, filter};

#[tokio::main]
async fn main() {
    // Force flush stderr before anything else, so we see startup output
    // even if the binary panics or exits silently.
    eprintln!("[boot] starting CRM Quality Inspector");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    // Load .env file (if present) into process env vars.
    let _ = dotenvy::dotenv();
    eprintln!("[boot] dotenv loaded");
    let _ = std::io::Write::flush(&mut std::io::stderr());

    // Initialize tracing — write to both stdout and a log file on disk.
    let log_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let env_filter = format!("crm_qi={}", log_level);
    let env_filter = tracing_subscriber::EnvFilter::try_new(env_filter)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    // Log file path from env var, defaults to /var/log/crm-qi/ when running in Docker
    let log_file = std::env::var("LOG_FILE")
        .unwrap_or_else(|_| "/var/log/crm-qi/crm-quality-inspector.log".to_string());

    // Open log file with append mode, force flush after each write for Docker log visibility
    let logfile = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
        .expect("failed to open log file");
    let logfile = Mutex::new(logfile);

    tracing_subscriber::registry()
        .with(env_filter.clone())
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_writer(logfile)
                .with_ansi(false),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_writer(std::io::stderr)
                .with_ansi(cfg!(not(test))),
        )
        .init();

    eprintln!("[boot] tracing initialized");
    let _ = std::io::Write::flush(&mut std::io::stderr());
    tracing::info!(LOG_FILE = %log_file, "Logging initialized");

    // DATABASE_URL must be provided via env var.
    let database_url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| {
            eprintln!("❌ DATABASE_URL env var is required. Refusing to start.");
            eprintln!("   Set it in .env, your shell, or docker-compose environment.");
            eprintln!("   Example: postgres://crm_quality:***@db:5432/crm_quality_inspector");
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "DATABASE_URL not set")
        })
        .expect("DATABASE_URL required");

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
