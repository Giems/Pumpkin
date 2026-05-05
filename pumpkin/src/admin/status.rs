use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use serde::Serialize;

use super::AppState;

#[derive(Serialize)]
struct StatusResponse {
    version: String,
    protocol: u32,
    tps: f64,
    mspt: f64,
    online_players: usize,
    max_players: u32,
}

pub async fn status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let server = &state.server;
    Json(StatusResponse {
        version: CURRENT_MC_VERSION.to_string(),
        protocol: CURRENT_MC_VERSION.protocol_version() as u32,
        tps: server.get_tps(),
        mspt: server.get_mspt(),
        online_players: server.get_player_count(),
        max_players: server.basic_config.max_players,
    })
}
