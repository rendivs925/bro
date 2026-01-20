//! Axum-based web server for the Vibespeak voice automation system
//!
//! This module provides a modular, clean architecture for the web API:
//! - `state` - Application state management
//! - `routes` - Route definitions
//! - `handlers` - Request handlers organized by feature
//! - `extractors` - Custom extractors for request parsing

pub mod handlers;
pub mod routes;
pub mod state;

use anyhow::Result;
use application::voice_command_processor::VoiceCommandProcessor;
use infrastructure::config::Config;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct AxumServer {
    state: AppState,
}

impl AxumServer {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>, config: Config) -> Self {
        Self {
            state: AppState::new(Some(voice_processor), config),
        }
    }

    pub async fn run(self, port: u16) -> Result<()> {
        let config = self.state.config.read().await;

        let tailscale_enabled = config
            .power_user
            .plugins
            .settings
            .get("tailscale")
            .unwrap_or(&std::collections::HashMap::new())
            .get("enabled")
            .unwrap_or(&"false".to_string())
            == "true";
        let web_settings = config
            .power_user
            .plugins
            .settings
            .get("web")
            .cloned()
            .unwrap_or_default();
        let addr = if tailscale_enabled {
            if let Some(bind_addr) = web_settings.get("server_bind") {
                parse_bind_address(bind_addr)?
            } else {
                SocketAddr::from(([127, 0, 0, 1], port))
            }
        } else {
            SocketAddr::from(([127, 0, 0, 1], port))
        };

        drop(config);

        let app = routes::create_router(self.state);

        tracing::info!("Starting Axum server on {}", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind: {}", e))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }
}

fn parse_bind_address(bind_addr: &str) -> Result<SocketAddr> {
    bind_addr
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid bind address: {}", bind_addr))
}
