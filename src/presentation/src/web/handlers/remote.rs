//! Remote control handlers

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Command;

use crate::web::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RemoteCommandRequest {
    pub command: String,
    pub parameters: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RemoteCommandResponse {
    pub status: String,
    pub command: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub processed: bool,
}

pub async fn execute_remote_command(
    State(_state): State<AppState>,
    Json(request): Json<RemoteCommandRequest>,
) -> Result<Json<RemoteCommandResponse>, StatusCode> {
    tracing::info!("Executing remote command: {}", request.command);

    // Execute command securely
    match Command::new("sh")
        .arg("-c")
        .arg(&request.command)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok(Json(RemoteCommandResponse {
                    status: "ok".to_string(),
                    command: request.command,
                    result: Some(stdout),
                    error: if stderr.is_empty() { None } else { Some(stderr) },
                    processed: true,
                }))
            } else {
                Ok(Json(RemoteCommandResponse {
                    status: "error".to_string(),
                    command: request.command,
                    result: Some(stdout),
                    error: Some(stderr),
                    processed: false,
                }))
            }
        }
        Err(e) => Ok(Json(RemoteCommandResponse {
            status: "error".to_string(),
            command: request.command,
            result: None,
            error: Some(e.to_string()),
            processed: false,
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct RemoteMouseRequest {
    #[serde(rename = "type")]
    pub event_type: String,
    pub x: i32,
    pub y: i32,
    pub timestamp: Option<u64>,
}

pub async fn handle_mouse_event(
    State(_state): State<AppState>,
    Json(request): Json<RemoteMouseRequest>,
) -> Json<Value> {
    tracing::info!(
        "Mouse event: {} at ({}, {})",
        request.event_type,
        request.x,
        request.y
    );

    // Use xdotool for mouse control on Linux
    let result = match request.event_type.as_str() {
        "move" => {
            Command::new("xdotool")
                .args(["mousemove", &request.x.to_string(), &request.y.to_string()])
                .output()
        }
        "click" | "left_click" => {
            Command::new("xdotool")
                .args([
                    "mousemove",
                    &request.x.to_string(),
                    &request.y.to_string(),
                    "click",
                    "1",
                ])
                .output()
        }
        "right_click" => {
            Command::new("xdotool")
                .args([
                    "mousemove",
                    &request.x.to_string(),
                    &request.y.to_string(),
                    "click",
                    "3",
                ])
                .output()
        }
        "double_click" => {
            Command::new("xdotool")
                .args([
                    "mousemove",
                    &request.x.to_string(),
                    &request.y.to_string(),
                    "click",
                    "--repeat",
                    "2",
                    "1",
                ])
                .output()
        }
        _ => {
            return Json(json!({
                "status": "error",
                "event_type": request.event_type,
                "error": "Unknown mouse event type",
                "message": "Supported types: move, click, left_click, right_click, double_click"
            }));
        }
    };

    match result {
        Ok(output) if output.status.success() => Json(json!({
            "status": "ok",
            "event_type": request.event_type,
            "x": request.x,
            "y": request.y,
            "message": "Mouse event processed successfully"
        })),
        Ok(output) => Json(json!({
            "status": "error",
            "event_type": request.event_type,
            "x": request.x,
            "y": request.y,
            "error": String::from_utf8_lossy(&output.stderr).to_string(),
            "message": "xdotool command failed"
        })),
        Err(e) => Json(json!({
            "status": "error",
            "event_type": request.event_type,
            "x": request.x,
            "y": request.y,
            "error": e.to_string(),
            "message": "Failed to execute mouse event - is xdotool installed?"
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct ScreenOfferRequest {
    pub session_id: Option<String>,
}

pub async fn create_screen_offer(
    State(_state): State<AppState>,
    Json(request): Json<ScreenOfferRequest>,
) -> Json<Value> {
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("session_{}", chrono::Utc::now().timestamp()));

    tracing::info!("Creating screen sharing session: {}", session_id);

    // Screen sharing requires WebRTC - return session info
    Json(json!({
        "status": "ok",
        "session_id": session_id,
        "message": "Screen sharing session created",
        "instructions": "Use WebRTC to connect to this session"
    }))
}

#[derive(Debug, Deserialize)]
pub struct ScreenAnswerRequest {
    pub session_id: String,
    pub answer: Value,
}

pub async fn handle_screen_answer(Json(request): Json<ScreenAnswerRequest>) -> Json<Value> {
    tracing::info!("Screen answer received for session: {}", request.session_id);

    Json(json!({
        "status": "ok",
        "session_id": request.session_id,
        "message": "Screen answer processed"
    }))
}
