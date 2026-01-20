//! Domain services traits for voice processing

use crate::entities::recognition_session::{AudioSample, RecognitionResult};
use crate::entities::voice_command::VoiceCommand;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Speech recognition service trait
#[async_trait]
pub trait SpeechRecognitionService: Send + Sync {
    /// Recognize speech from audio sample
    async fn recognize(&self, audio: AudioSample) -> anyhow::Result<RecognitionResult>;

    /// Check if the service is ready
    async fn is_ready(&self) -> bool {
        true // Default implementation
    }
}

/// Text-to-speech service trait
#[async_trait]
pub trait TextToSpeechService: Send + Sync {
    /// Convert text to speech and return audio data
    async fn synthesize(&self, text: &str, voice: Option<&str>) -> anyhow::Result<Vec<i16>>;

    /// Get available voices
    async fn get_available_voices(&self) -> anyhow::Result<Vec<String>>;

    /// Speak text directly (convenience method)
    async fn speak(&self, text: &str) -> anyhow::Result<()> {
        let _audio = self.synthesize(text, None).await?;
        // In a real implementation, this would play the audio
        Ok(())
    }
}

/// Voice command processing service trait
#[async_trait]
pub trait VoiceCommandService: Send + Sync {
    /// Process a voice command
    async fn process_command(&self, command: &str) -> anyhow::Result<VoiceCommand>;

    /// Get available commands
    async fn get_available_commands(&self) -> anyhow::Result<Vec<String>>;
}

/// Command context for execution
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub previous_commands: Vec<String>,
    pub environment: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

/// Interpreted command with parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpretedCommand {
    pub action: CommandAction,
    pub confidence: f32,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Command actions that can be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandAction {
    Execute(String),     // Execute a command
    Workflow(String),    // Execute a workflow
    Script(String),      // Execute a script
    Integration(String), // Execute an integration
}

/// Plugin-related types
pub mod plugin {
    use super::*;

    /// Plugin context for execution
    #[derive(Debug, Clone)]
    pub struct PluginContext {
        pub command: String,
        pub parameters: HashMap<String, serde_json::Value>,
        pub user_context: HashMap<String, String>,
    }

    /// Plugin capabilities
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum PluginCapability {
        CommandProvider,
        DataProcessor,
        Integration,
        Automation,
    }

    /// Voice plugin trait
    #[async_trait]
    pub trait VoicePlugin: Send + Sync {
        fn metadata(&self) -> PluginMetadata;
        async fn execute(&self, input: PluginInput) -> anyhow::Result<PluginOutput>;
        fn has_capability(&self, capability: &PluginCapability) -> bool;
    }

    /// Plugin input
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PluginInput {
        pub command: String,
        pub parameters: HashMap<String, serde_json::Value>,
        pub context: HashMap<String, serde_json::Value>,
    }

    /// Plugin output
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PluginOutput {
        pub success: bool,
        pub result: serde_json::Value,
        pub error_message: Option<String>,
    }

    /// Plugin metadata
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PluginMetadata {
        pub name: String,
        pub version: String,
        pub description: String,
        pub capabilities: Vec<PluginCapability>,
        pub author: String,
    }
}
