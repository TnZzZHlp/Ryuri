use axum::{
    body::Body,
    extract::Path,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../frontend/dist"]
struct Assets;

pub async fn serve_index() -> impl IntoResponse {
    serve_asset("index.html")
}

pub async fn serve_static(Path(path): Path<String>) -> impl IntoResponse {
    serve_asset(&path)
}

fn serve_asset(path: &str) -> Response {
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                )],
                Body::from(content.data),
            )
                .into_response()
        }
        None => {
            // If the path starts with "api/", return 404 to avoid serving index.html for API errors
            if path.starts_with("api/") {
                return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
            }

            // SPA Fallback: If file not found, serve index.html
            if let Some(index) = Assets::get("index.html") {
                (
                    [(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))],
                    Body::from(index.data),
                )
                    .into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}
