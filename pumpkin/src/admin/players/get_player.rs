use super::{Position, Rotation};
use crate::admin::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::{Arc, atomic::Ordering};

#[derive(Serialize)]
pub struct PlayerDetail {
    pub username: String,
    pub uuid: String,
    pub world: String,
    pub gamemode: String,
    pub health: f32,
    pub food: u8,
    pub saturation: f32,
    pub experience_level: i32,
    pub experience_progress: f32,
    pub position: Position,
    pub rotation: Rotation,
    pub ping: u32,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_on_ground: bool,
}

pub async fn get_player(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let worlds = state.server.worlds.load();

    for world in worlds.iter() {
        for player in world.players.load().iter() {
            if player.gameprofile.name.eq_ignore_ascii_case(&name) {
                let pos = player.living_entity.entity.pos.load();
                let detail = PlayerDetail {
                    username: player.gameprofile.name.clone(),
                    uuid: player.gameprofile.id.to_string(),
                    world: world.get_world_name().to_owned(),
                    gamemode: format!("{:?}", player.gamemode.load()),
                    health: player.living_entity.health.load(),
                    food: player.hunger_manager.level.load(),
                    saturation: player.hunger_manager.saturation.load(),
                    experience_level: player.experience_level.load(Ordering::Relaxed),
                    experience_progress: player.experience_progress.load(),
                    position: Position {
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                    },
                    rotation: Rotation {
                        yaw: player.living_entity.entity.yaw.load(),
                        pitch: player.living_entity.entity.pitch.load(),
                    },
                    ping: player.ping.load(Ordering::Relaxed),
                    is_sneaking: player.living_entity.entity.sneaking.load(Ordering::Relaxed),
                    is_sprinting: player
                        .living_entity
                        .entity
                        .sprinting
                        .load(Ordering::Relaxed),
                    is_on_ground: player
                        .living_entity
                        .entity
                        .on_ground
                        .load(Ordering::Relaxed),
                };
                return Json(detail).into_response();
            }
        }
    }

    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": format!("player '{name}' not found")})),
    )
        .into_response()
}
