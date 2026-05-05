use std::sync::Arc;

use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;

use super::AppState;

#[derive(Serialize)]
struct PluginInfo {
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
    state: String,
}

pub async fn list_plugins(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metadata = state.server.plugin_manager.loaded_plugins().await;
    let mut infos = Vec::with_capacity(metadata.len());

    for meta in metadata {
        let plugin_state = state
            .server
            .plugin_manager
            .get_plugin_state(&meta.name)
            .await
            .map(|s| format!("{s:?}"))
            .unwrap_or_else(|| "Unknown".into());

        infos.push(PluginInfo {
            name: meta.name,
            version: meta.version,
            authors: meta.authors,
            description: meta.description,
            state: plugin_state,
        });
    }

    Json(infos).into_response()
}

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
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "unsupported plugin format"})),
            )
                .into_response();
        }

        let data: Bytes = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": format!("read error: {e}")})),
                )
                    .into_response();
            }
        };

        let dest = std::path::Path::new("plugins").join(&filename);
        if let Err(e) = tokio::fs::write(&dest, &data).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("write error: {e}")})),
            )
                .into_response();
        }

        return Json(serde_json::json!({
            "status": "uploaded",
            "filename": filename,
            "bytes": data.len()
        }))
        .into_response();
    }

    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "no file field in multipart"})),
    )
        .into_response()
}

pub async fn delete_plugin(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let plugins_dir = std::path::Path::new("plugins");

    for ext in &[".wasm", ".so", ".dll", ".dylib"] {
        let src = plugins_dir.join(format!("{name}{ext}"));
        if src.exists() {
            let dst = plugins_dir.join(format!("{name}{ext}.deactivated"));
            if let Err(e) = tokio::fs::rename(&src, &dst).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("rename error: {e}")})),
                )
                    .into_response();
            }
            return Json(serde_json::json!({"status": "deactivated", "plugin": name}))
                .into_response();
        }
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "plugin not found"})),
    )
        .into_response()
}
