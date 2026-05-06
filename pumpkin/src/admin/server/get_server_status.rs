use crate::admin::AppState;
use axum::{Json, extract::State, response::IntoResponse};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct ServerStatus {
    pub version: String,
    pub protocol: u32,
    pub tps: f64,
    pub mspt: f64,
    pub online_players: usize,
    pub max_players: u32,
    pub difficulty: String,
    pub tick_count: i32,
}

pub async fn get_server_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let server = &state.server;
    Json(ServerStatus {
        version: CURRENT_MC_VERSION.to_string(),
        protocol: CURRENT_MC_VERSION.protocol_version() as u32,
        tps: server.get_tps(),
        mspt: server.get_mspt(),
        online_players: server.get_player_count(),
        max_players: server.basic_config.max_players,
        difficulty: format!("{:?}", server.get_difficulty()),
        tick_count: server.tick_count.load(std::sync::atomic::Ordering::Relaxed),
    })
}
