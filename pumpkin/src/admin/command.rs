use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::command::CommandSender;

use super::AppState;

#[derive(Deserialize)]
pub struct CommandBody {
    pub command: String,
}

#[derive(Serialize)]
struct CommandResponse {
    output: Vec<String>,
}

pub async fn run_command(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CommandBody>,
) -> impl IntoResponse {
    let output = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
    let server = state.server.clone();
    let cmd = body.command.clone();

    let command_source = CommandSender::Rcon(output.clone())
        .into_source(&server)
        .await;

    tokio::spawn(async move {
        server
            .command_dispatcher
            .read()
            .await
            .handle_command(&command_source, &cmd)
            .await;
    })
    .await
    .ok();

    let lines = output.lock().await.clone();
    Json(CommandResponse { output: lines })
}
