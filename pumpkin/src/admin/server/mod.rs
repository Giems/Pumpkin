mod get_machine_info;
mod get_server_status;
mod get_worlds_info;

use crate::admin::AppState;
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn server_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/get_server_status",
            get(get_server_status::get_server_status),
        )
        .route("/get_machine_info", get(get_machine_info::get_machine_info))
        .route("/get_worlds_info", get(get_worlds_info::get_worlds_info))
}
