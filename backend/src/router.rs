//! Application router configuration.
//!
//! This module provides the router configuration for the Axum web server.

use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, TraceLayer},
};
use tracing::Level;

#[cfg(feature = "dev")]
use tower_http::{LatencyUnit, trace::DefaultOnResponse};

use crate::handlers::{
    apikey, auth, content, filesystem, komga, library, progress, scan_queue, static_files,
};
use crate::middlewares::auth_middleware;
use crate::state::AppState;

/// Create the application router with all routes configured.
///
/// This function separates routes into public and protected groups:
/// - Public routes: /api/auth/login (no authentication required)
/// - Protected routes: All other routes (require authentication via middleware)
///
/// # Arguments
/// * `state` - The application state containing all services
///
/// # Returns
/// A configured Axum router with all API endpoints
///
/// # Requirements
/// - 4.1: Allow applying middleware to specific route groups
/// - 4.2: Process requests without authentication for routes without middleware
/// - 4.3: Support nesting routers with and without authentication
pub fn create_router(state: AppState) -> Router {
    // Public routes - no authentication required
    let public_routes = Router::new().route("/api/auth/login", post(auth::login));

    // Komga compatibility routes - no authentication for now
    let komga_routes = Router::new()
        .route("/komga/api/v1/series", get(komga::get_series_list))
        .route("/komga/api/v1/series/{seriesId}", get(komga::get_series))
        .route(
            "/komga/api/v1/series/{seriesId}/thumbnail",
            get(komga::get_series_thumbnail),
        )
        .route(
            "/komga/api/v1/series/{seriesId}/books",
            get(komga::get_books),
        )
        .route("/komga/api/v1/books/{bookId}", get(komga::get_book))
        .route(
            "/komga/api/v1/books/{bookId}/thumbnail",
            get(komga::get_book_thumbnail),
        )
        .route(
            "/komga/api/v1/books/{bookId}/pages",
            get(komga::get_page_list),
        )
        .route(
            "/komga/api/v1/books/{bookId}/pages/{pageNumber}",
            get(komga::get_page),
        )
        .route("/komga/api/v1/libraries", get(komga::get_libraries))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Protected routes - authentication required
    let protected_routes = Router::new()
        // Auth routes (except login)
        .route("/api/auth/me", get(auth::get_me).put(auth::update_me))
        .route("/api/auth/password", put(auth::update_password))
        // Library routes
        .route("/api/libraries", get(library::list).post(library::create))
        .route(
            "/api/libraries/{library_id}",
            get(library::get)
                .put(library::update)
                .delete(library::delete),
        )
        .route(
            "/api/libraries/{library_id}/paths",
            get(library::list_paths).post(library::add_path),
        )
        .route(
            "/api/libraries/{library_id}/paths/{path_id}",
            delete(library::remove_path),
        )
        .route("/api/libraries/{library_id}/contents", get(content::list))
        .route(
            "/api/libraries/{library_id}/scan",
            post(scan_queue::submit_scan),
        )
        .route("/api/libraries/{library_id}/search", get(content::search))
        // Scan queue routes
        .route("/api/scan-tasks", get(scan_queue::list_tasks))
        .route(
            "/api/scan-tasks/{task_id}",
            get(scan_queue::get_task).delete(scan_queue::cancel_task),
        )
        // Content routes
        .route(
            "/api/contents/{content_id}",
            get(content::get)
                .put(content::update)
                .delete(content::delete),
        )
        .route(
            "/api/contents/{content_id}/thumbnail",
            get(content::get_thumbnail),
        )
        .route(
            "/api/contents/{content_id}/chapters",
            get(content::list_chapters),
        )
        .route(
            "/api/contents/{content_id}/progress",
            get(progress::get_content_progress),
        )
        .route(
            "/api/contents/{content_id}/chapters/{chapter_id}/pages/{page}",
            get(content::get_page),
        )
        .route(
            "/api/contents/{content_id}/chapters/{chapter_id}/text",
            get(content::get_chapter_text),
        )
        // Progress routes
        .route("/api/progress/recent", get(progress::get_recent_progress))
        // Chapter progress routes
        .route(
            "/api/chapters/{chapter_id}/progress",
            get(progress::get_chapter_progress).put(progress::update_chapter_progress),
        )
        // API Key routes
        .route(
            "/api/api-keys",
            get(apikey::list_api_keys).post(apikey::create_api_key),
        )
        .route("/api/api-keys/{id}", delete(apikey::delete_api_key))
        // Filesystem routes
        .route("/api/filesystem", get(filesystem::list_directories))
        // Apply authentication middleware to all protected routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Merge public and protected routers
    let api_router = Router::new()
        .merge(public_routes)
        .merge(komga_routes)
        .merge(protected_routes);

    let router = Router::new()
        .merge(api_router)
        .route("/", get(static_files::serve_index))
        .route("/{*path}", get(static_files::serve_static))
        .with_state(state);

    // Add OpenAPI routes when dev feature is enabled
    add_openapi_routes(router)
}

/// Add OpenAPI documentation routes (Swagger UI and OpenAPI JSON endpoint).
#[cfg(feature = "dev")]
fn add_openapi_routes(router: Router) -> Router {
    use utoipa::OpenApi;
    use utoipa_swagger_ui::SwaggerUi;

    use crate::openapi::ApiDoc;

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

/// No-op when dev feature is disabled.
#[cfg(not(feature = "dev"))]
fn add_openapi_routes(router: Router) -> Router {
    router
}

/// Create the router with Layers middleware configured.
///
/// # Arguments
/// * `state` - The application state containing all services
///
/// # Returns
/// A configured Axum router with Layers enabled
pub fn create_router_with_layers(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = create_router(state).layer(cors);

    // Add tracing layer with request logging always enabled
    // Response logging only when dev feature is enabled
    add_tracing_layer(router)
}

/// Add tracing layer with request logging always enabled, response logging only in dev mode.
#[cfg(feature = "dev")]
fn add_tracing_layer(router: Router) -> Router {
    use tower_http::trace::DefaultOnFailure;

    let tracing = TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::INFO)
                .include_headers(false),
        )
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_failure(DefaultOnFailure::new().level(Level::WARN))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Millis),
        );

    router.layer(tracing)
}

/// Add tracing layer with request logging only (no response logging in production).
#[cfg(not(feature = "dev"))]
fn add_tracing_layer(router: Router) -> Router {
    use tower_http::{LatencyUnit, trace::DefaultOnFailure};

    let tracing = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_failure(
            DefaultOnFailure::new()
                .level(Level::WARN)
                .latency_unit(LatencyUnit::Millis),
        )
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::ERROR)
                .include_headers(false),
        );

    router.layer(tracing)
}
