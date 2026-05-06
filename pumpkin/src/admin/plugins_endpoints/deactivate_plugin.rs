use crate::admin::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{error, info, warn};

pub async fn deactivate_plugin(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let plugins_dir = std::path::Path::new("plugins");

    for ext in &[".wasm", ".so", ".dll", ".dylib"] {
        let src = plugins_dir.join(format!("{name}{ext}"));
        if src.exists() {
            if let Err(e) = state.server.plugin_manager.unload_plugin(&name).await {
                warn!(%e, name, "admin: plugin not loaded in memory, proceeding with file deactivation");
            }

            let dst = plugins_dir.join(format!("{name}{ext}.deactivated"));
            if let Err(e) = tokio::fs::rename(&src, &dst).await {
                error!(%e, name, "admin: failed to deactivate plugin");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("rename error: {e}")})),
                )
                    .into_response();
            }
            info!(name, "admin: plugin unloaded and deactivated");
            return Json(serde_json::json!({"status": "deactivated", "plugin": name}))
                .into_response();
        }
    }

    warn!(name, "admin: plugin deactivate requested but not found");
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "plugin not found"})),
    )
        .into_response()
}
