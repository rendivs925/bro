//! Application state for the Axum server

use application::voice_command_processor::VoiceCommandProcessor;
use infrastructure::config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub voice_processor: Arc<VoiceCommandProcessor>,
    pub config: Arc<RwLock<Config>>,
}

impl AppState {
    pub fn new(
        voice_processor: Arc<VoiceCommandProcessor>,
        config: Config,
    ) -> Self {
        Self {
            voice_processor,
            config: Arc::new(RwLock::new(config)),
        }
    }
}
