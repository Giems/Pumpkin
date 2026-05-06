use crate::admin::AppState;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{error, info, warn};

pub async fn upload_plugin(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let filename = match field.file_name() {
            Some(n) => n.to_owned(),
            None => continue,
        };

        if !filename.ends_with(".wasm")
            && !filename.ends_with(".so")
            && !filename.ends_with(".dll")
            && !filename.ends_with(".dylib")
        {
            warn!(
                filename,
                "admin: plugin upload rejected — unsupported format"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "unsupported plugin format"})),
            )
                .into_response();
        }

        let data: Bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                error!(%e, filename, "admin: failed to read plugin upload");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": format!("read error: {e}")})),
                )
                    .into_response();
            }
        };

        let dest = std::path::Path::new("plugins").join(&filename);
        if let Err(e) = tokio::fs::write(&dest, &data).await {
            error!(%e, filename, "admin: failed to write plugin to disk");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("write error: {e}")})),
            )
                .into_response();
        }

        info!(filename, bytes = data.len(), "admin: plugin uploaded");
        return Json(serde_json::json!({
            "status": "uploaded",
            "filename": filename,
            "bytes": data.len()
        }))
        .into_response();
    }

    warn!("admin: plugin upload request had no file field");
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "no file field in multipart"})),
    )
        .into_response()
}
