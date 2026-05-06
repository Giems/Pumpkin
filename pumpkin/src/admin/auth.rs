use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use tracing::warn;

use super::AppState;

pub async fn bearer_auth(
    State(state): State<Arc<AppState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if state.token.is_empty() {
        return next.run(request).await;
    }

    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match token {
        Some(t) if t == state.token => next.run(request).await,
        Some(_) => {
            warn!(uri = %request.uri(), "admin: rejected request — invalid token");
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Invalid admin token"))
                .unwrap()
        }
        None => {
            warn!(uri = %request.uri(), "admin: rejected request — missing Authorization header");
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Missing Authorization header"))
                .unwrap()
        }
    }
}
