//! Comic Reader Backend Server
//!
//! This is the main entry point for the comic reader backend server.
//! It initializes the database, creates all services, and starts the HTTP server.

use std::env;
use std::net::SocketAddr;

use argon2::password_hash::rand_core::{OsRng, RngCore};
use backend::db::{DbConfig, init_db};
use backend::error::AppError;
use backend::router::create_router_with_layers;
use backend::services::auth::AuthConfig;
use backend::state::{AppConfig, AppState};
use tokio::signal;
use tracing::{debug, info, warn};
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn generate_random_secret_hex(byte_len: usize) -> String {
    // Use OS CSPRNG. This is suitable for secrets.
    let mut bytes = vec![0u8; byte_len];
    OsRng.fill_bytes(&mut bytes);

    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Server configuration loaded from environment variables.
struct ServerConfig {
    host: String,
    port: u16,
    db: DbConfig,
    app: AppConfig,
}

impl ServerConfig {
    fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:ryuri.db?mode=rwc".to_string());

        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => {
                let secret = generate_random_secret_hex(32);
                warn!(
                    "JWT_SECRET is not set (or is empty); using a random secret from OS RNG. \
                    Tokens will be invalid after restart. Set JWT_SECRET to make it persistent."
                );
                secret
            }
        };

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(24);

        Self {
            host,
            port,
            db: DbConfig {
                database_url,
                max_connections: 5,
            },
            app: AppConfig {
                auth: AuthConfig {
                    jwt_secret,
                    jwt_expiration_hours,
                },
            },
        }
    }
}

/// Initialize the tracing subscriber with env-filter support.
///
/// Log levels can be controlled via the RUST_LOG environment variable:
/// - RUST_LOG=debug - Detailed debug information
/// - RUST_LOG=info - Normal operation information (default)
/// - RUST_LOG=warn - Warnings and errors only
/// - RUST_LOG=backend=debug,sqlx=warn - Module-level control
fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_timer(ChronoLocal::new(String::from("%x %X")))
                .compact(),
        )
        .init();
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing first
    init_tracing();

    let config = ServerConfig::from_env();

    info!("Ryuri starting...");
    debug!(host = %config.host, port = %config.port, database = %config.db.database_url, "Server configuration loaded");

    info!("Initializing database...");
    let pool = init_db(&config.db).await?;
    info!("Database initialized successfully");

    info!("Creating application services...");
    let state = AppState::new(pool, config.app);
    info!("Services created successfully");

    // Start the scan queue worker to process submitted scan tasks
    info!("Starting scan queue worker...");
    state.scan_queue_service.start_worker().await;

    let app = create_router_with_layers(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid address: {}", e)))?;

    info!(%addr, "Starting server");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to bind: {}", e)))?;

    info!("Server is running. Press Ctrl+C to stop.");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
