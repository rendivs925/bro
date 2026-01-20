# Bro: HTTP API Reference

This document describes the REST API endpoints provided by bro for programmatic access and integration.

## Base URL

```
http://localhost:8080/api
```

## Authentication

Currently, bro runs locally and doesn't require authentication. All endpoints are accessible on the local machine.

## Response Format

All API responses follow this structure:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2024-01-20T10:30:00Z"
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VOICE_RECOGNITION_FAILED",
    "message": "Failed to recognize speech",
    "details": { ... }
  },
  "timestamp": "2024-01-20T10:30:00Z"
}
```

## Voice API

### POST /api/voice/command

Process a voice command directly.

**Request:**
```json
{
  "text": "generate a rust hello world function",
  "confidence": 0.95,
  "language": "en-US"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "command_id": "cmd_123456",
    "intent": "code_generation",
    "response": {
      "type": "code",
      "language": "rust",
      "content": "fn main() {\n    println!(\"Hello, World!\");\n}",
      "explanation": "Generated a simple Rust hello world function"
    },
    "execution_time_ms": 1250
  }
}
```

### GET /api/voice/status

Get current voice recognition status.

**Response:**
```json
{
  "success": true,
  "data": {
    "is_listening": true,
    "wake_word_active": true,
    "current_session": "session_789",
    "last_command": {
      "text": "check compilation errors",
      "timestamp": "2024-01-20T10:29:45Z",
      "status": "completed"
    }
  }
}
```

### POST /api/voice/wake

Manually trigger wake word activation.

**Request:**
```json
{
  "source": "api",
  "duration_ms": 5000
}
```

### WebSocket /api/voice/stream

Real-time voice command streaming.

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:8080/api/voice/stream');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'recognition_result') {
    console.log('Recognized:', data.text);
  }
};
```

**Messages:**
```json
{
  "type": "recognition_result",
  "text": "generate fibonacci function",
  "confidence": 0.92,
  "is_final": true
}
```

## AI Assistant API

### POST /api/ai/generate

Generate code or text using AI.

**Request:**
```json
{
  "prompt": "create a python function to calculate factorial",
  "type": "code",
  "language": "python",
  "context": {
    "project_type": "web_app",
    "framework": "flask"
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "content": "def factorial(n):\n    if n == 0:\n        return 1\n    return n * factorial(n - 1)",
    "language": "python",
    "confidence": 0.89,
    "tokens_used": 156,
    "generation_time_ms": 850
  }
}
```

### POST /api/ai/chat

Interactive chat with AI assistant.

**Request:**
```json
{
  "message": "explain rust ownership",
  "conversation_id": "conv_123",
  "include_context": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "response": "Rust ownership is a system that ensures memory safety...",
    "conversation_id": "conv_123",
    "sources": [
      {
        "file": "src/main.rs",
        "line": 42,
        "relevance": 0.95
      }
    ]
  }
}
```

### POST /api/ai/analyze

Analyze code for issues or improvements.

**Request:**
```json
{
  "code": "fn main() { println!(\"Hello\"); }",
  "language": "rust",
  "analysis_type": "all"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "issues": [
      {
        "type": "style",
        "severity": "warning",
        "message": "Missing main function documentation",
        "line": 1,
        "suggestion": "Add /// documentation comment"
      }
    ],
    "metrics": {
      "complexity": 1,
      "maintainability": 85,
      "test_coverage": 0
    }
  }
}
```

## Command Execution API

### POST /api/commands/execute

Execute a shell command with safety checks.

**Request:**
```json
{
  "command": "cargo check",
  "working_directory": "/home/user/project",
  "timeout_seconds": 30,
  "require_confirmation": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "command_id": "exec_456",
    "stdout": "Checking project...\nAll checks passed!",
    "stderr": "",
    "exit_code": 0,
    "execution_time_ms": 2450,
    "confirmed": true
  }
}
```

### GET /api/commands/history

Get command execution history.

**Query Parameters:**
- `limit` (optional): Number of commands to return (default: 50)
- `offset` (optional): Pagination offset (default: 0)
- `status` (optional): Filter by status (pending, running, completed, failed)

**Response:**
```json
{
  "success": true,
  "data": {
    "commands": [
      {
        "id": "exec_456",
        "command": "cargo check",
        "status": "completed",
        "exit_code": 0,
        "timestamp": "2024-01-20T10:30:00Z",
        "duration_ms": 2450
      }
    ],
    "total": 150,
    "limit": 50,
    "offset": 0
  }
}
```

### POST /api/commands/cancel

Cancel a running command.

**Request:**
```json
{
  "command_id": "exec_456"
}
```

## File System API

### GET /api/files/list

List files in a directory.

**Query Parameters:**
- `path`: Directory path
- `recursive` (optional): Include subdirectories (default: false)
- `include_hidden` (optional): Include hidden files (default: false)

**Response:**
```json
{
  "success": true,
  "data": {
    "path": "/home/user/project",
    "files": [
      {
        "name": "Cargo.toml",
        "type": "file",
        "size": 1024,
        "modified": "2024-01-20T09:00:00Z",
        "permissions": "644"
      },
      {
        "name": "src",
        "type": "directory",
        "size": 4096,
        "modified": "2024-01-20T10:00:00Z",
        "permissions": "755"
      }
    ]
  }
}
```

### GET /api/files/read

Read file contents.

**Query Parameters:**
- `path`: File path
- `encoding` (optional): Text encoding (default: utf-8)
- `max_lines` (optional): Maximum lines to return

**Response:**
```json
{
  "success": true,
  "data": {
    "path": "/home/user/project/src/main.rs",
    "content": "fn main() {\n    println!(\"Hello, World!\");\n}",
    "encoding": "utf-8",
    "size": 45,
    "modified": "2024-01-20T10:00:00Z"
  }
}
```

### POST /api/files/write

Write content to a file.

**Request:**
```json
{
  "path": "/home/user/project/src/main.rs",
  "content": "fn main() {\n    println!(\"Hello, World!\");\n}",
  "encoding": "utf-8",
  "create_directories": true
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "path": "/home/user/project/src/main.rs",
    "size": 45,
    "modified": "2024-01-20T10:30:00Z"
  }
}
```

## Project Analysis API

### GET /api/project/analyze

Analyze project structure and dependencies.

**Query Parameters:**
- `path` (optional): Project root path (default: current directory)

**Response:**
```json
{
  "success": true,
  "data": {
    "language": "rust",
    "framework": "axum",
    "dependencies": [
      { "name": "tokio", "version": "1.0", "type": "runtime" },
      { "name": "serde", "version": "1.0", "type": "serialization" }
    ],
    "structure": {
      "src/": {
        "main.rs": { "type": "entry_point", "lines": 150 },
        "lib.rs": { "type": "library", "lines": 200 }
      }
    },
    "metrics": {
      "total_files": 15,
      "total_lines": 2500,
      "test_coverage": 85
    }
  }
}
```

### GET /api/project/search

Search codebase using RAG.

**Query Parameters:**
- `query`: Search query
- `type` (optional): Search type (code, docs, all)
- `limit` (optional): Max results (default: 10)

**Response:**
```json
{
  "success": true,
  "data": {
    "query": "authentication middleware",
    "results": [
      {
        "file": "src/middleware/auth.rs",
        "line": 15,
        "content": "pub struct AuthMiddleware {\n    jwt_secret: String,\n}",
        "relevance": 0.95,
        "context": "JWT authentication middleware implementation"
      }
    ],
    "total_results": 5
  }
}
```

## System API

### GET /api/system/status

Get system status and resource usage.

**Response:**
```json
{
  "success": true,
  "data": {
    "version": "1.0.0",
    "uptime_seconds": 3600,
    "memory": {
      "used_mb": 245,
      "total_mb": 8192,
      "percentage": 3
    },
    "cpu": {
      "usage_percentage": 5.2,
      "cores": 8
    },
    "disk": {
      "used_gb": 45,
      "total_gb": 256,
      "percentage": 18
    },
    "services": {
      "voice_recognition": "active",
      "ai_assistant": "active",
      "web_interface": "active"
    }
  }
}
```

### GET /api/system/health

Health check endpoint.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "checks": {
      "voice_service": { "status": "pass", "response_time_ms": 5 },
      "ai_service": { "status": "pass", "response_time_ms": 50 },
      "file_system": { "status": "pass", "response_time_ms": 2 },
      "database": { "status": "pass", "response_time_ms": 10 }
    }
  }
}
```

### POST /api/system/restart

Restart bro services.

**Request:**
```json
{
  "services": ["voice", "ai", "web"],
  "reason": "configuration_update"
}
```

### GET /api/system/logs

Get application logs.

**Query Parameters:**
- `level` (optional): Log level filter (debug, info, warn, error)
- `since` (optional): ISO 8601 timestamp
- `limit` (optional): Max log entries (default: 100)

**Response:**
```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "timestamp": "2024-01-20T10:30:00Z",
        "level": "info",
        "message": "Voice command processed successfully",
        "service": "voice_processor",
        "metadata": {
          "command_id": "cmd_123",
          "confidence": 0.92
        }
      }
    ]
  }
}
```

## Configuration API

### GET /api/config

Get current configuration.

**Response:**
```json
{
  "success": true,
  "data": {
    "voice": {
      "wake_word": "bro",
      "model_path": "model/vosk-model-en-us",
      "sample_rate": 16000,
      "sensitivity": 0.8
    },
    "ai": {
      "provider": "ollama",
      "model": "qwen2.5:3b",
      "temperature": 0.7,
      "max_tokens": 2048
    },
    "web": {
      "port": 8080,
      "host": "127.0.0.1",
      "cors_origins": ["http://localhost:3000"]
    },
    "security": {
      "sandbox_enabled": true,
      "confirmation_required": true,
      "secrets_detection": true
    }
  }
}
```

### POST /api/config/update

Update configuration (requires restart).

**Request:**
```json
{
  "voice": {
    "wake_word": "computer"
  },
  "ai": {
    "temperature": 0.5
  }
}
```

## WebRTC API

### POST /api/webrtc/offer

Initiate WebRTC connection for screen streaming.

**Request:**
```json
{
  "client_type": "mobile",
  "capabilities": {
    "audio": true,
    "video": true,
    "data_channel": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "session_id": "webrtc_789",
    "offer": {
      "type": "offer",
      "sdp": "v=0\r\no=- ..."
    },
    "ice_servers": [
      { "urls": "stun:stun.l.google.com:19302" }
    ]
  }
}
```

### POST /api/webrtc/answer

Complete WebRTC handshake.

**Request:**
```json
{
  "session_id": "webrtc_789",
  "answer": {
    "type": "answer",
    "sdp": "..."
  }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| VOICE_RECOGNITION_FAILED | Speech recognition failed |
| AI_SERVICE_UNAVAILABLE | AI model not responding |
| COMMAND_EXECUTION_FAILED | Shell command failed |
| FILE_NOT_FOUND | Requested file doesn't exist |
| PERMISSION_DENIED | Insufficient permissions |
| SECURITY_VIOLATION | Command violates security policy |
| INVALID_REQUEST | Malformed request data |
| SERVICE_OVERLOADED | Too many concurrent requests |

## Rate Limits

- Voice commands: 10 per minute
- AI generation: 20 per minute
- File operations: 100 per minute
- System queries: Unlimited

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 9
X-RateLimit-Reset: 1642684800
```

This API provides comprehensive programmatic access to bro's voice-powered AI capabilities, enabling integration with other tools and automation workflows.</content>
<parameter name="filePath">/home/rendi/projects/bro/docs/api.md