use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared::types::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub action: WorkflowAction,
    pub inputs: HashMap<String, WorkflowInput>,
    pub outputs: Vec<String>,
    pub on_error: ErrorHandling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowAction {
    ExecuteCommand {
        command: String,
        args: Vec<String>,
    },
    CallService {
        service: String,
        method: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    TransformData {
        transformation: String,
    },
    Conditional {
        condition: String,
        then_step: String,
        else_step: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowInput {
    Static(serde_json::Value),
    Variable(String),
    StepOutput { step_id: String, output_key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorHandling {
    Continue,
    Stop,
    Retry { max_attempts: u32, delay_ms: u64 },
    AlternativeStep(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    pub success: bool,
    pub outputs: HashMap<String, serde_json::Value>,
    pub errors: Vec<String>,
    pub execution_time_ms: u64,
}

#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    async fn execute_workflow(&self, workflow: &Workflow) -> Result<WorkflowExecutionResult>;
    async fn validate_workflow(&self, workflow: &Workflow) -> Result<Vec<String>>; // returns validation errors
    async fn get_workflow_status(&self, execution_id: &str) -> Result<WorkflowExecutionState>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowExecutionState {
    Pending,
    Running,
    Completed(WorkflowExecutionResult),
    Failed(Vec<String>),
}
