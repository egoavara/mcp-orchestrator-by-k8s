use crate::assets::STATIC_ASSETS;

pub async fn handler() -> impl axum::response::IntoResponse {
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
}