//! Comic Reader Backend Server
//!
//! This is the main entry point for the comic reader backend server.
//! It initializes the database, creates all services, and starts the HTTP server.

use std::env;
use std::net::SocketAddr;

use backend::db::{DbConfig, init_db};
use backend::error::AppError;
use backend::router::{AppConfig, AppState, create_router_with_cors};
use backend::services::auth::AuthConfig;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

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

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:comic_reader.db?mode=rwc".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default-secret-change-in-production".to_string());

        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(24);

        let bangumi_api_key = env::var("BANGUMI_API_KEY").ok();

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
                bangumi_api_key,
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
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing first
    init_tracing();

    let config = ServerConfig::from_env();

    info!("Comic Reader Backend starting...");
    debug!(host = %config.host, port = %config.port, database = %config.db.database_url, "Server configuration loaded");

    info!("Initializing database...");
    let pool = init_db(&config.db).await?;
    info!("Database initialized successfully");

    info!("Creating application services...");
    let state = AppState::new(pool, config.app);
    info!("Services created successfully");

    let app = create_router_with_cors(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid address: {}", e)))?;

    info!(%addr, "Starting server");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to bind: {}", e)))?;

    info!("Server is running. Press Ctrl+C to stop.");
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
