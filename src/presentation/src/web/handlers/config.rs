//! Configuration handlers

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::web::state::AppState;

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub commands: Vec<CommandInfo>,
    pub workflows: Vec<WorkflowInfo>,
    pub scripts: Vec<ScriptInfo>,
    pub settings: SettingsInfo,
}

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub id: String,
    pub text: String,
    pub action: Value,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<Value>,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ScriptInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub content: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct SettingsInfo {
    pub vosk_model_path: String,
    pub sample_rate: f32,
    #[serde(default)]
    pub audio_device: Option<String>,
    pub web_server_port: u16,
    pub enable_tts: bool,
    #[serde(default)]
    pub enable_webrtc: bool,
    pub tailscale_enabled: bool,
}

pub async fn get_config(State(state): State<AppState>) -> Json<ConfigResponse> {
    let config = state.config.read().await;

    let commands: Vec<CommandInfo> = config
        .power_user
        .commands
        .iter()
        .map(|cmd| CommandInfo {
            id: cmd.id.clone(),
            text: cmd.text.clone(),
            action: serde_json::to_value(&cmd.action).unwrap_or(Value::Null),
            category: cmd.category.clone(),
            enabled: cmd.enabled,
        })
        .collect();

    // Map workflows from config
    let workflows: Vec<WorkflowInfo> = config
        .power_user
        .workflows
        .iter()
        .map(|wf| WorkflowInfo {
            id: wf.id.clone(),
            name: wf.name.clone(),
            description: wf.description.clone(),
            steps: wf
                .steps
                .iter()
                .map(|s| serde_json::to_value(s).unwrap_or(Value::Null))
                .collect(),
            enabled: wf.enabled,
        })
        .collect();

    // Map scripts from config
    let scripts: Vec<ScriptInfo> = config
        .power_user
        .scripts
        .scripts
        .iter()
        .map(|s| ScriptInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            language: format!("{:?}", s.script_type),
            content: s.content.clone(),
            enabled: s.enabled,
        })
        .collect();

    let vosk_settings = config
        .power_user
        .plugins
        .settings
        .get("vosk")
        .cloned()
        .unwrap_or_default();
    let settings = SettingsInfo {
        vosk_model_path: vosk_settings
            .get("model_path")
            .unwrap_or(&"".to_string())
            .clone(),
        sample_rate: vosk_settings
            .get("sample_rate")
            .unwrap_or(&"16000".to_string())
            .parse()
            .unwrap_or(16000.0),
        audio_device: vosk_settings.get("audio_device").cloned(),
        web_server_port: config
            .power_user
            .plugins
            .settings
            .get("web")
            .unwrap_or(&std::collections::HashMap::new())
            .get("server_port")
            .unwrap_or(&"8080".to_string())
            .parse()
            .unwrap_or(8080),
        enable_tts: vosk_settings
            .get("enable_tts")
            .unwrap_or(&"false".to_string())
            == "true",
        enable_webrtc: vosk_settings
            .get("enable_webrtc")
            .unwrap_or(&"false".to_string())
            == "true",
        tailscale_enabled: config
            .power_user
            .plugins
            .settings
            .get("tailscale")
            .unwrap_or(&std::collections::HashMap::new())
            .get("enabled")
            .unwrap_or(&"false".to_string())
            == "true",
    };

    Json(ConfigResponse {
        commands,
        workflows,
        scripts,
        settings,
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub settings: Option<UpdateSettingsRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub vosk_model_path: Option<String>,
    pub sample_rate: Option<f32>,
    pub enable_tts: Option<bool>,
}

pub async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    if let Some(settings) = request.settings {
        let vosk_settings = config
            .power_user
            .plugins
            .settings
            .entry("vosk".to_string())
            .or_insert_with(std::collections::HashMap::new);
        if let Some(path) = settings.vosk_model_path {
            vosk_settings.insert("model_path".to_string(), path);
        }
        if let Some(rate) = settings.sample_rate {
            vosk_settings.insert("sample_rate".to_string(), rate.to_string());
        }
        if let Some(tts) = settings.enable_tts {
            vosk_settings.insert("enable_tts".to_string(), tts.to_string());
        }
    }

    Ok(Json(json!({
        "status": "ok",
        "message": "Configuration updated successfully"
    })))
}

#[derive(Debug, Serialize)]
pub struct TailscaleStatus {
    pub enabled: bool,
    pub connected: bool,
    pub hostname: Option<String>,
    pub port: u16,
    pub error: Option<String>,
}

pub async fn get_tailscale_status(State(state): State<AppState>) -> Json<TailscaleStatus> {
    let config = state.config.read().await;

    let tailscale_settings = config
        .power_user
        .plugins
        .settings
        .get("tailscale")
        .cloned()
        .unwrap_or_default();
    let web_settings = config
        .power_user
        .plugins
        .settings
        .get("web")
        .cloned()
        .unwrap_or_default();
    Json(TailscaleStatus {
        enabled: tailscale_settings
            .get("enabled")
            .unwrap_or(&"false".to_string())
            == "true",
        connected: tailscale_settings
            .get("enabled")
            .unwrap_or(&"false".to_string())
            == "true",
        hostname: tailscale_settings.get("hostname").cloned(),
        port: web_settings
            .get("server_port")
            .unwrap_or(&"8080".to_string())
            .parse()
            .unwrap_or(8080),
        error: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateTailscaleRequest {
    pub enabled: bool,
    pub hostname: Option<String>,
    pub port: Option<u16>,
}

pub async fn update_tailscale_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateTailscaleRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let tailscale_settings = config
        .power_user
        .plugins
        .settings
        .entry("tailscale".to_string())
        .or_insert_with(std::collections::HashMap::new);
    tailscale_settings.insert("enabled".to_string(), request.enabled.to_string());
    if let Some(hostname) = request.hostname {
        tailscale_settings.insert("hostname".to_string(), hostname);
    }
    let web_settings = config
        .power_user
        .plugins
        .settings
        .entry("web".to_string())
        .or_insert_with(std::collections::HashMap::new);
    if let Some(port) = request.port {
        web_settings.insert("server_port".to_string(), port.to_string());
    }

    Ok(Json(json!({
        "status": "ok",
        "message": "Tailscale configuration updated"
    })))
}

// ============= COMMAND CRUD =============

#[derive(Debug, Deserialize)]
pub struct CreateCommandRequest {
    pub text: String,
    pub action: Value,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommandRequest {
    pub text: Option<String>,
    pub action: Option<Value>,
    pub category: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn create_command(
    State(state): State<AppState>,
    Json(request): Json<CreateCommandRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    // Parse the action from JSON
    let action: domain::entities::voice_command::CommandAction =
        serde_json::from_value(request.action.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let command = domain::entities::voice_command::VoiceCommand {
        id: uuid::Uuid::new_v4().to_string(),
        text: request.text,
        action,
        category: request.category,
        enabled: true,
        confidence: 0.0,
        metadata: domain::entities::voice_command::CommandMetadata::default(),
    };

    let id = command.id.clone();
    config.power_user.commands.push(command);

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "id": id,
        "message": "Command created successfully"
    })))
}

pub async fn update_command(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateCommandRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let command = config
        .power_user
        .commands
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(text) = request.text {
        command.text = text;
    }
    if let Some(action) = request.action {
        command.action = serde_json::from_value(action).map_err(|_| StatusCode::BAD_REQUEST)?;
    }
    if let Some(category) = request.category {
        command.category = category;
    }
    if let Some(enabled) = request.enabled {
        command.enabled = enabled;
    }

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "message": "Command updated successfully"
    })))
}

pub async fn delete_command(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let initial_len = config.power_user.commands.len();
    config.power_user.commands.retain(|c| c.id != id);

    if config.power_user.commands.len() == initial_len {
        return Err(StatusCode::NOT_FOUND);
    }

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "message": "Command deleted successfully"
    })))
}

pub async fn list_commands(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    Ok(Json(json!({
        "status": "ok",
        "commands": config.power_user.commands
    })))
}

pub async fn get_command(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    let command = config
        .power_user
        .commands
        .iter()
        .find(|c| c.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "status": "ok",
        "command": command
    })))
}

// ============= WORKFLOW CRUD =============

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    pub trigger: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn create_workflow(
    State(state): State<AppState>,
    Json(request): Json<CreateWorkflowRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let workflow = domain::entities::workflow::Workflow {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        trigger: domain::entities::workflow::WorkflowTrigger::Manual,
        steps: Vec::new(),
        variables: std::collections::HashMap::new(),
        error_handling: domain::entities::workflow::ErrorStrategy::Stop,
        enabled: true,
    };

    let id = workflow.id.clone();
    config.power_user.workflows.push(workflow);

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "id": id,
        "message": "Workflow created successfully"
    })))
}

pub async fn update_workflow(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateWorkflowRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let workflow = config
        .power_user
        .workflows
        .iter_mut()
        .find(|w| w.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = request.name {
        workflow.name = name;
    }
    if let Some(description) = request.description {
        workflow.description = description;
    }
    if let Some(enabled) = request.enabled {
        workflow.enabled = enabled;
    }

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "message": "Workflow updated successfully"
    })))
}

pub async fn delete_workflow(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let initial_len = config.power_user.workflows.len();
    config.power_user.workflows.retain(|w| w.id != id);

    if config.power_user.workflows.len() == initial_len {
        return Err(StatusCode::NOT_FOUND);
    }

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "message": "Workflow deleted successfully"
    })))
}

pub async fn list_workflows(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    Ok(Json(json!({
        "status": "ok",
        "workflows": config.power_user.workflows
    })))
}

pub async fn get_workflow(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    let workflow = config
        .power_user
        .workflows
        .iter()
        .find(|w| w.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "status": "ok",
        "workflow": workflow
    })))
}

// ============= SCRIPT CRUD =============

#[derive(Debug, Deserialize)]
pub struct CreateScriptRequest {
    pub name: String,
    pub language: String,
    pub content: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScriptRequest {
    pub name: Option<String>,
    pub content: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn create_script(
    State(state): State<AppState>,
    Json(request): Json<CreateScriptRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let script_type = match request.language.to_lowercase().as_str() {
        "bash" | "shell" => shared::types::ScriptType::Bash,
        "python" | "py" => shared::types::ScriptType::Python,
        "javascript" | "js" => shared::types::ScriptType::JavaScript,
        "ruby" | "rb" => shared::types::ScriptType::Ruby,
        "powershell" | "ps" => shared::types::ScriptType::PowerShell,
        _ => shared::types::ScriptType::Bash,
    };

    let script = infrastructure::config::Script {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        script_type,
        content: request.content,
        description: request.description.unwrap_or_default(),
        enabled: true,
    };

    let id = script.id.clone();
    config.power_user.scripts.scripts.push(script);

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "id": id,
        "message": "Script created successfully"
    })))
}

pub async fn update_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateScriptRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let script = config
        .power_user
        .scripts
        .scripts
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = request.name {
        script.name = name;
    }
    if let Some(content) = request.content {
        script.content = content;
    }
    if let Some(enabled) = request.enabled {
        script.enabled = enabled;
    }

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "message": "Script updated successfully"
    })))
}

pub async fn delete_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let initial_len = config.power_user.scripts.scripts.len();
    config.power_user.scripts.scripts.retain(|s| s.id != id);

    if config.power_user.scripts.scripts.len() == initial_len {
        return Err(StatusCode::NOT_FOUND);
    }

    // Save to file
    let _ = config
        .power_user
        .save_to_file(&std::path::PathBuf::from("config/system.json"));

    Ok(Json(json!({
        "status": "ok",
        "message": "Script deleted successfully"
    })))
}

pub async fn list_scripts(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    Ok(Json(json!({
        "status": "ok",
        "scripts": config.power_user.scripts.scripts
    })))
}

pub async fn get_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    let script = config
        .power_user
        .scripts
        .scripts
        .iter()
        .find(|s| s.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "status": "ok",
        "script": script
    })))
}
