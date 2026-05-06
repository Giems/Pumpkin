use crate::{admin::AppState, command::CommandSender};
use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize)]
pub struct ExecuteCommandBody {
    pub command: String,
}

#[derive(Serialize)]
struct ExecuteCommandResponse {
    output: Vec<String>,
}

pub async fn execute_command(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ExecuteCommandBody>,
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
    info!(
        command = body.command,
        lines = lines.len(),
        "admin: command executed"
    );
    Json(ExecuteCommandResponse { output: lines })
}
