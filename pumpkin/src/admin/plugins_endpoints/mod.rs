mod deactivate_plugin;
mod get_loaded_plugins;
mod upload_plugin;

use crate::admin::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

pub fn plugin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/get_loaded_plugins",
            get(get_loaded_plugins::get_loaded_plugins),
        )
        .route("/upload_plugin", post(upload_plugin::upload_plugin))
        .route(
            "/deactivate_plugin/{name}",
            post(deactivate_plugin::deactivate_plugin),
        )
}
