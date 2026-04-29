use axum::body::Body;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
pub struct FrontendAssets;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    serve_asset(path)
}

pub fn serve_asset(path: &str) -> Response {
    match FrontendAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.to_vec()))
                .unwrap_or_else(|_| internal_error())
        }
        None => {
            // SPA fallback: return index.html for any unrecognised path.
            match FrontendAssets::get("index.html") {
                Some(index) => Response::builder()
                    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(Body::from(index.data.to_vec()))
                    .unwrap_or_else(|_| internal_error()),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("frontend not built"))
                    .unwrap_or_else(|_| internal_error()),
            }
        }
    }
}

fn internal_error() -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap()
}
