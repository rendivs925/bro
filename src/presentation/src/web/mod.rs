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

use application::voice_command_processor::VoiceCommandProcessor;
use infrastructure::config::Config;
use anyhow::Result;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct AxumServer {
    state: AppState,
}

impl AxumServer {
    pub fn new(
        voice_processor: Arc<VoiceCommandProcessor>,
        config: Config,
    ) -> Self {
        Self {
            state: AppState::new(voice_processor, config),
        }
    }

    pub async fn run(self, port: u16) -> Result<()> {
        let config = self.state.config.read().await;

        let addr = if config.settings.tailscale_enabled {
            if let Some(ref bind_addr) = config.settings.web_server_bind {
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
    bind_addr.parse().map_err(|_| {
        anyhow::anyhow!("Invalid bind address: {}", bind_addr)
    })
}
