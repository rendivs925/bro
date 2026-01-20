use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared::types::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInput {
    pub command: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginOutput {
    pub success: bool,
    pub result: serde_json::Value,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub author: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCapability {
    CommandProvider,
    DataProcessor,
    Integration,
    Automation,
}

#[async_trait]
pub trait VoicePlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    async fn execute(&self, input: PluginInput) -> Result<PluginOutput>;
    fn has_capability(&self, capability: &PluginCapability) -> bool;
}

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn VoicePlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn VoicePlugin>) {
        let name = plugin.metadata().name.clone();
        self.plugins.insert(name, plugin);
    }

    pub fn get_plugin(&self, name: &str) -> Option<&dyn VoicePlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }

    pub fn get_available_plugins(&self) -> Vec<PluginMetadata> {
        self.list_plugins()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
