use axum::http::StatusCode;

pub async fn handler() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "404 Not Found")
}
