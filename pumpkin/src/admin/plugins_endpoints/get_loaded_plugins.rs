use crate::admin::AppState;
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub state: String,
}

pub async fn get_loaded_plugins(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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
