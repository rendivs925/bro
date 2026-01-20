use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared::types::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSession {
    pub session_id: String,
    pub browser_type: BrowserType,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserAction {
    pub action_type: BrowserActionType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserActionType {
    Navigate { url: String },
    Click { selector: String },
    Type { selector: String, text: String },
    Screenshot,
    GetText { selector: String },
    Wait { milliseconds: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub screenshot: Option<String>, // base64 encoded
}

#[async_trait]
pub trait BrowserAutomationService: Send + Sync {
    async fn create_session(&self, browser_type: BrowserType) -> Result<BrowserSession>;
    async fn execute_action(
        &self,
        session_id: &str,
        action: BrowserAction,
    ) -> Result<BrowserResult>;
    async fn close_session(&self, session_id: &str) -> Result<()>;
}
