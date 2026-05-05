use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

use super::AppState;

#[derive(Serialize)]
struct PlayerInfo {
    name: String,
    uuid: String,
    gamemode: String,
    ping: u32,
}

pub async fn players(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let players: Vec<PlayerInfo> = state
        .server
        .get_all_players()
        .iter()
        .map(|p| PlayerInfo {
            name: p.gameprofile.name.clone(),
            uuid: p.gameprofile.id.to_string(),
            gamemode: format!("{:?}", p.gamemode.load()),
            ping: p.ping.load(std::sync::atomic::Ordering::Relaxed),
        })
        .collect();

    Json(players)
}
