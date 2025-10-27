use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Duration;
use mime_guess::{mime, Mime};
use include_dir::{File};

use crate::assets::STATIC_ASSETS;

pub async fn handler(Path(path): Path<String>) -> impl IntoResponse {
    let serve_file =
        |file: &File, mime_type: Option<Mime>, cache: Duration, code: Option<StatusCode>| {
            Response::builder()
                .status(code.unwrap_or(StatusCode::OK))
                .header(
                    header::CONTENT_TYPE,
                    mime_type.unwrap_or(mime::TEXT_PLAIN).to_string(),
                )
                .header(
                    header::CACHE_CONTROL,
                    format!("max-age={}", cache.as_seconds_f32()),
                )
                .body(Body::from(file.contents().to_owned()))
                .unwrap()
        };
    let path = path.trim_start_matches('/');
    if let Some(file) = STATIC_ASSETS.get_file(path) {
        let mime_type = mime_guess::from_path(file.path()).first();
        serve_file(file, mime_type, Duration::days(30), None)
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(header::CONTENT_TYPE, mime::TEXT_PLAIN.to_string())
            .body(Body::from("404 Not Found"))
            .unwrap()
    }
}


