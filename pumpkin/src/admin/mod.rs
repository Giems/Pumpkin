#![cfg(feature = "admin_panel")]

mod auth;
mod commands;
mod players;
mod plugins_endpoints;
mod server;

use crate::{admin::auth::bearer_auth, server::Server};
use axum::{Router, middleware};
use pumpkin_config::AdminPanelConfig;
use std::sync::Arc;
use tower::util::option_layer;
use tower_http::cors::CorsLayer;

pub async fn start(config: AdminPanelConfig, srv: Arc<Server>) {
    let state = Arc::new(AppState {
        server: srv,
        token: config.token.clone(),
    });

    let auth = (!state.token.is_empty())
        .then(|| middleware::from_fn_with_state(state.clone(), bearer_auth));

    let app = Router::new()
        .nest("/api/server", server::server_router())
        .nest("/api/players", players::player_router())
        .nest("/api/plugins", plugins_endpoints::plugin_router())
        .nest("/api/commands", commands::command_router())
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
