use serde::{Deserialize, Serialize};
use shared::types::{Result, ScriptType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScriptExecution {
    pub script_type: ScriptType,
    pub content: String,
    pub parameters: HashMap<String, String>,
}

impl ScriptExecution {
    pub fn new(script_type: ScriptType, content: String) -> Self {
        Self {
            script_type,
            content,
            parameters: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, key: String, value: String) -> Self {
        self.parameters.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub success: bool,
    pub output: String,
    pub error_output: String,
    pub exit_code: Option<i32>,
}

#[async_trait::async_trait]
pub trait ScriptExecutor: Send + Sync {
    async fn execute(&self, script: &ScriptExecution) -> Result<ScriptResult>;
}
