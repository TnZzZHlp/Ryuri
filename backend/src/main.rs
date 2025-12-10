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

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let config = ServerConfig::from_env();

    println!("Comic Reader Backend starting...");
    println!("  Host: {}", config.host);
    println!("  Port: {}", config.port);
    println!("  Database: {}", config.db.database_url);

    println!("Initializing database...");
    let pool = init_db(&config.db).await?;
    println!("Database initialized successfully.");

    println!("Creating application services...");
    let state = AppState::new(pool, config.app);
    println!("Services created successfully.");

    let app = create_router_with_cors(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| AppError::Internal(format!("Invalid address: {}", e)))?;

    println!("Starting server on http://{}...", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to bind: {}", e)))?;

    println!("Server is running. Press Ctrl+C to stop.");
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}
