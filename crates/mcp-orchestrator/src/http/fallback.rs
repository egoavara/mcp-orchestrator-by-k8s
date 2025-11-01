use axum::{
    body::Body,
    http::{Request, Response},
    response::IntoResponse,
};

use crate::assets::STATIC_ASSETS;

pub async fn handler(req: Request<Body>) -> Response<Body> {
    match req.uri().path() {
        f if f.starts_with("/.well-known") => not_found(),
        f if f.starts_with("/oauth") => not_found(),
        f if f.starts_with("/statics") => not_found(),
        f if f.starts_with("/mcp") => not_found(),
        _ => index_file(),
    }
}

fn index_file() -> Response<Body> {
    STATIC_ASSETS
        .get_file("index.html")
        .map(|file| {
            (
                axum::http::StatusCode::OK,
                [("Content-Type", "text/html")],
                file.contents(),
            )
        })
        .unwrap_or((
            axum::http::StatusCode::NOT_FOUND,
            [("Content-Type", "text/plain")],
            "404 Not Found".as_bytes(),
        ))
        .into_response()
}

fn not_found() -> Response<Body> {
    (
        axum::http::StatusCode::NOT_FOUND,
        [("Content-Type", "text/plain")],
        "404 Not Found".as_bytes(),
    )
        .into_response()
}
