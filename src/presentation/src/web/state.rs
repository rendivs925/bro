//! Application state for the Axum server

use application::voice_command_processor::VoiceCommandProcessor;
use infrastructure::config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state for all handlers
#[derive(Clone)]
pub struct AppState {
    pub voice_processor: Option<Arc<VoiceCommandProcessor>>,
    pub config: Arc<RwLock<Config>>,
}

impl AppState {
    pub fn new(voice_processor: Option<Arc<VoiceCommandProcessor>>, config: Config) -> Self {
        Self {
            voice_processor,
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Create a minimal state without voice processor (for testing or fallback)
    pub fn minimal(config: Config) -> Self {
        // Create a minimal voice processor - this is a placeholder
        // In production, this should be properly initialized
        Self {
            voice_processor: None,
            config: Arc::new(RwLock::new(config)),
        }
    }
}
