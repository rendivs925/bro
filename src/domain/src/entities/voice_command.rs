use anyhow::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type CommandId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub id: CommandId,
    pub text: String,
    pub confidence: f64,
    pub category: String,
    pub action: CommandAction,
    pub enabled: bool,
    pub metadata: CommandMetadata,
}

impl VoiceCommand {
    pub fn new(text: String, action: CommandAction) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            confidence: 0.0,
            category: "general".to_string(),
            action,
            enabled: true,
            metadata: CommandMetadata::default(),
        }
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.category = category;
        self
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.text.trim().is_empty() {
            return Err(anyhow::anyhow!("Command text cannot be empty"));
        }
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(anyhow::anyhow!("Confidence must be between 0.0 and 1.0"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandAction {
    ShellCommand(String),
    Workflow(String),        // Reference to workflow ID
    Script(String),          // Reference to script ID
    IntegrationCall(String), // API call to integration
    Composite(Vec<String>),  // Multiple shell commands
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Vec<String>,
}

impl Default for CommandMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            usage_count: 0,
            last_used: None,
            tags: Vec::new(),
        }
    }
}
