use crate::{admin::AppState, command::CommandSender};
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
}

pub async fn list_commands(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let dispatcher = state.server.command_dispatcher.read().await;
    let source = CommandSender::Console.into_source(&state.server).await;
    let commands = dispatcher.get_all_permitted_commands_usage(&source).await;

    let infos: Vec<CommandInfo> = commands
        .into_iter()
        .map(|(name, (desc, usage))| CommandInfo {
            name: name.to_owned(),
            description: desc.to_owned(),
            usage: usage.to_string(),
        })
        .collect();

    Json(infos)
}
