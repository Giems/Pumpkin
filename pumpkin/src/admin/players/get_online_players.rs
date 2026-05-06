use crate::admin::AppState;
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct OnlinePlayer {
    pub username: String,
    pub uuid: String,
    pub world: String,
    pub ping: u32,
}

pub async fn get_online_players(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let worlds = state.server.worlds.load();
    let mut players = Vec::new();

    for world in worlds.iter() {
        let world_name = world.get_world_name().to_owned();
        for player in world.players.load().iter() {
            players.push(OnlinePlayer {
                username: player.gameprofile.name.clone(),
                uuid: player.gameprofile.id.to_string(),
                world: world_name.clone(),
                ping: player.ping.load(std::sync::atomic::Ordering::Relaxed),
            });
        }
    }

    Json(players)
}
