use crate::admin::AppState;
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct WorldInfo {
    pub name: String,
    pub dimension: String,
    pub player_count: usize,
    pub entity_count: usize,
    pub loaded_chunks: usize,
    pub time_of_day: i64,
    pub world_age: i64,
    pub is_raining: bool,
    pub is_thundering: bool,
    pub spawn: SpawnPoint,
    pub seed: i64,
    pub difficulty: String,
}

#[derive(Serialize)]
pub struct SpawnPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub async fn get_worlds_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let worlds = state.server.worlds.load();
    let level_data = state.server.level_info.load();

    let mut infos = Vec::with_capacity(worlds.len());

    for world in worlds.iter() {
        let time = world.level_time.lock().await;
        let active_chunks = world.active_chunks.load().len();
        let player_count = world.players.load().len();
        let entity_count = world.entities.load().len();

        infos.push(WorldInfo {
            name: world.get_world_name().to_owned(),
            dimension: format!("{:?}", world.dimension),
            player_count,
            entity_count,
            loaded_chunks: active_chunks,
            time_of_day: time.time_of_day,
            world_age: time.world_age,
            is_raining: world.is_raining().await,
            is_thundering: world.is_thundering().await,
            spawn: SpawnPoint {
                x: level_data.spawn_x,
                y: level_data.spawn_y,
                z: level_data.spawn_z,
            },
            seed: level_data.world_gen_settings.seed,
            difficulty: format!("{:?}", level_data.difficulty),
        });
    }

    Json(infos)
}
