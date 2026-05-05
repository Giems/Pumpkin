#![cfg(feature = "admin_panel")]

mod auth;
mod command;
mod players;
mod plugins;
mod status;

use crate::{admin::auth::bearer_auth, server::Server};
use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
use pumpkin_config::AdminPanelConfig;
use std::sync::Arc;
use tower::util::option_layer;
use tower_http::cors::CorsLayer;

pub async fn start(config: AdminPanelConfig, server: Arc<Server>) {
    let state = Arc::new(AppState {
        server,
        token: config.token.clone(),
    });

    let auth = (!state.token.is_empty())
        .then(|| middleware::from_fn_with_state(state.clone(), bearer_auth));

    let app = Router::new()
        .route("/api/status", get(status::status))
        .route("/api/players", get(players::players))
        .route("/api/plugins", get(plugins::list_plugins))
        .route("/api/plugins", post(plugins::upload_plugin))
        .route("/api/plugins/:name", delete(plugins::delete_plugin))
        .route("/api/command", post(command::run_command))
        .layer(option_layer(auth))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(config.address).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Admin panel failed to bind {}: {e}", config.address);
            return;
        }
    };

    tracing::info!("Admin panel listening on http://{}", config.address);
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Admin panel error: {e}");
    }
}

pub(super) struct AppState {
    pub server: Arc<Server>,
    pub token: String,
}
