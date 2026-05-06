mod execute_command;
mod list_commands;

use crate::admin::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

pub fn command_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/list_commands", get(list_commands::list_commands))
        .route("/execute_command", post(execute_command::execute_command))
}
