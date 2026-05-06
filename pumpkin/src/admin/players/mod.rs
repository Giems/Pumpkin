mod get_online_players;
mod get_player;

use crate::admin::AppState;
use axum::{Router, routing::get};
use serde::Serialize;
use std::sync::Arc;

pub fn player_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/get_online_players",
            get(get_online_players::get_online_players),
        )
        .route("/get_player/{name}", get(get_player::get_player))
}

#[derive(Serialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Serialize)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}
