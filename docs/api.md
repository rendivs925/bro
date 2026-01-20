# Bro: Development Guide

This guide covers the development setup, architecture details, and contribution guidelines for the unified "bro" project.

## 🏗️ Project Structure

After unification, the bro monorepo follows this structure:

```
bro/
├── src/                    # Main Rust codebase (formerly vibe_cli)
│   ├── domain/            # Business logic layer
│   │   ├── entities/      # Core business objects
│   │   ├── services/      # Domain services
│   │   └── policies/      # Business rules
│   ├── application/       # Use case layer
│   │   ├── voice_to_code.rs    # Voice code generation
│   │   ├── mobile_control.rs   # Mobile interface logic
│   │   ├── agent_service.rs    # AI agent orchestration
│   │   └── rag_service.rs      # RAG system
│   ├── infrastructure/    # External integrations
│   │   ├── adapters/      # External system adapters
│   │   │   ├── vosk_adapter.rs     # Speech recognition
│   │   │   ├── tts_adapter.rs      # Text-to-speech
│   │   │   ├── ollama_client.rs    # Local AI inference
│   │   │   └── webrtc_streamer.rs  # Screen streaming
│   │   ├── repositories/  # Data persistence
│   │   └── config/        # Configuration management
│   ├── presentation/      # User interface layer
│   │   ├── cli/          # Command-line interface
│   │   ├── tui/          # Terminal user interface
│   │   └── web/          # Mobile web interface
│   └── shared/           # Common utilities
│       ├── security/     # Sandbox and validation
│       ├── performance/  # Caching and optimization
│       └── utils/        # Common helpers
├── docs/                 # Documentation
├── models/               # AI/ML model files
├── config/               # Configuration files
├── tests/                # Test suites
└── tools/                # Development tools
```

## 🚀 Development Setup

### Prerequisites

- **Rust**: 1.70+ with Cargo
- **CMake**: 3.13+ (for Piper TTS)
- **Python**: 3.8+ (for some build tools)
- **Node.js**: 16+ (for web interface development)
- **Ollama**: For local AI inference
- **System Audio**: ALSA/PulseAudio for voice I/O

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/rendivs925/bro.git
cd bro

# Install system dependencies
make install-deps

# Download AI models and setup
make setup-models

# Build the project
make build

# Run tests
make test

# Start development server
make dev
```

### Detailed Setup

#### 1. System Dependencies

**Arch Linux:**
```bash
sudo pacman -S rust cmake gcc alsa-utils pulseaudio \
               python python-pip nodejs npm
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install rustc cargo cmake build-essential \
                 libasound2-dev pulseaudio-utils \
                 python3 python3-pip nodejs npm
```

**macOS:**
```bash
brew install rust cmake alsa-utils pulseaudio \
            python node
```

#### 2. Download Models

```bash
# Create models directory
mkdir -p models

# Download Vosk speech recognition model
cd models
wget https://alphacephei.com/vosk/models/vosk-model-en-us-0.22-lgraph.zip
unzip vosk-model-en-us-0.22-lgraph.zip

# Download Piper TTS models
# (Handled by make setup-models)
```

#### 3. Setup Ollama

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Start Ollama service
ollama serve

# Download required models
ollama pull qwen2.5:3b
ollama pull llama2:7b  # Optional fallback
```

### Development Workflow

#### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check compilation
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt
```

#### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_voice_recognition

# Run with output
cargo test -- --nocapture

# Performance benchmarks
cargo bench
```

#### Development Server
```bash
# Start with voice interface
cargo run -- --voice --web

# Start CLI only
cargo run -- --chat

# Debug mode
RUST_LOG=debug cargo run -- --voice
```

## 🏛️ Architecture Deep Dive

### Clean Architecture Layers

#### Domain Layer
Contains business logic independent of external concerns:

```rust
// src/domain/entities/voice_command.rs
#[derive(Debug, Clone)]
pub struct VoiceCommand {
    pub id: Uuid,
    pub text: String,
    pub intent: Intent,
    pub confidence: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum Intent {
    CodeGeneration(String),
    CommandExecution(String),
    CodeAnalysis(String),
    SystemQuery(String),
}
```

#### Application Layer
Orchestrates use cases using domain entities:

```rust
// src/application/voice_to_code.rs
pub struct VoiceToCodeService {
    ai_client: Arc<dyn AiClient>,
    code_validator: Arc<dyn CodeValidator>,
    project_analyzer: Arc<dyn ProjectAnalyzer>,
}

impl VoiceToCodeService {
    pub async fn generate_code(&self, voice_command: VoiceCommand) -> Result<CodeGeneration> {
        // Analyze project context
        let context = self.project_analyzer.analyze().await?;

        // Generate code with AI
        let prompt = self.build_prompt(&voice_command, &context);
        let code = self.ai_client.generate_code(prompt).await?;

        // Validate generated code
        self.code_validator.validate(&code)?;

        Ok(CodeGeneration { code, context })
    }
}
```

#### Infrastructure Layer
Handles external integrations:

```rust
// src/infrastructure/adapters/vosk_adapter.rs
pub struct VoskAdapter {
    model: vosk::Model,
    recognizer: Mutex<vosk::Recognizer>,
}

impl VoskAdapter {
    pub async fn recognize(&self, audio: AudioSample) -> Result<VoiceCommand> {
        let mut recognizer = self.recognizer.lock().await;

        // Process audio chunks
        for chunk in audio.chunks(4000) {
            recognizer.accept_waveform(chunk)?;
        }

        // Extract final result
        let result = recognizer.final_result()?;
        let text = result.text.trim().to_string();

        Ok(VoiceCommand {
            text,
            confidence: result.confidence,
            timestamp: Utc::now(),
            ..Default::default()
        })
    }
}
```

### Voice Processing Pipeline

1. **Audio Capture**: Microphone → AudioSample
2. **Wake Word Detection**: Background Vosk model monitors for "bro"
3. **Speech Recognition**: Full Vosk model transcribes speech
4. **Intent Classification**: NLP classifies command type
5. **AI Processing**: Route to appropriate AI service
6. **Code Generation**: Generate context-aware code
7. **Safety Validation**: Sandbox and security checks
8. **Execution**: Run commands with confirmation

### WebRTC Integration

```rust
// src/infrastructure/adapters/webrtc_streamer.rs
pub struct WebRTCStreamer {
    peer_connection: RTCPeerConnection,
    screen_track: VideoTrack,
}

impl WebRTCStreamer {
    pub async fn start_stream(&self) -> Result<String> {
        // Capture desktop screen
        let screen_capture = self.capture_desktop()?;

        // Create WebRTC peer connection
        let offer = self.create_offer().await?;

        // Return SDP offer for client
        Ok(offer.to_string())
    }

    pub async fn handle_answer(&self, answer: String) -> Result<()> {
        // Process client answer
        let remote_description = RTCSessionDescription::from_string(answer)?;
        self.peer_connection.set_remote_description(remote_description).await?;
        Ok(())
    }
}
```

## 🔧 Development Guidelines

### Code Style
- Follow Rust idioms and conventions
- Use `cargo fmt` and `cargo clippy`
- Write comprehensive documentation
- Include unit tests for all public functions

### Error Handling
```rust
// Use thiserror for custom errors
#[derive(Debug, thiserror::Error)]
pub enum BroError {
    #[error("Voice recognition failed: {0}")]
    VoiceRecognition(String),

    #[error("AI service unavailable: {0}")]
    AiUnavailable(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

// Use anyhow for generic errors
pub async fn process_command(command: String) -> Result<CommandResult> {
    let voice_command = self.recognize_speech(command)
        .await
        .context("Failed to recognize speech")?;

    let ai_response = self.generate_response(voice_command)
        .await
        .context("Failed to generate AI response")?;

    Ok(CommandResult { response: ai_response })
}
```

### Testing Strategy

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_voice_recognition() {
        let adapter = VoskAdapter::new("test_model").await.unwrap();

        // Mock audio data
        let audio = create_test_audio("hello world");

        let result = adapter.recognize(audio).await.unwrap();
        assert_eq!(result.text, "hello world");
        assert!(result.confidence > 0.8);
    }
}
```

#### Integration Tests
```rust
#[tokio::test]
async fn test_full_voice_pipeline() {
    let voice_service = VoiceProcessingService::new().await;

    // Simulate voice input
    let command = voice_service.process("generate fibonacci function").await.unwrap();

    // Verify code generation
    assert!(command.code.contains("fn fibonacci"));
    assert!(command.code.contains("n: u32"));
}
```

#### Performance Benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_voice_recognition(c: &mut Criterion) {
    c.bench_function("voice_recognition", |b| {
        b.iter(|| {
            let audio = create_benchmark_audio();
            black_box(recognize_speech(audio));
        })
    });
}
```

### Security Considerations

#### Input Validation
```rust
// src/shared/security/input_validator.rs
pub struct InputValidator {
    dangerous_patterns: Vec<Regex>,
}

impl InputValidator {
    pub fn validate(&self, input: &str) -> Result<()> {
        for pattern in &self.dangerous_patterns {
            if pattern.is_match(input) {
                return Err(BroError::SecurityViolation(
                    format!("Dangerous pattern detected: {}", pattern.as_str())
                ));
            }
        }
        Ok(())
    }
}
```

#### Sandbox Execution
```rust
// src/infrastructure/sandbox.rs
pub struct CommandSandbox {
    allowed_commands: HashSet<String>,
    resource_limits: ResourceLimits,
}

impl CommandSandbox {
    pub async fn execute(&self, command: &str) -> Result<CommandOutput> {
        // Validate command
        self.validate_command(command)?;

        // Set resource limits
        let limits = self.create_limits()?;

        // Execute in sandbox
        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(command)
            .limits(limits)
            .output()
            .await?;

        Ok(CommandOutput::from(output))
    }
}
```

## 🚀 Deployment

### Release Build
```bash
# Create optimized release
cargo build --release

# Strip debug symbols
strip target/release/bro

# Create distribution package
make package
```

### Docker Deployment
```dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libasound2-dev \
    pulseaudio-utils \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bro /usr/local/bin/
COPY --from=builder /app/models /app/models

CMD ["bro", "--voice", "--web"]
```

### System Service
```systemd
# /etc/systemd/system/bro.service
[Unit]
Description=Bro Voice-Powered AI Assistant
After=network.target ollama.service

[Service]
Type=simple
User=bro
ExecStart=/usr/local/bin/bro --voice --web
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

## 🔍 Debugging

### Voice Recognition Debugging
```bash
# Enable debug logging
RUST_LOG=vosk=debug,bro=debug cargo run -- --voice

# Test microphone input
cargo run -- --test-mic

# Save audio for analysis
cargo run -- --record-audio test.wav
```

### AI Service Debugging
```bash
# Test Ollama connection
curl http://localhost:11434/api/tags

# Debug AI prompts
RUST_LOG=ollama=debug cargo run -- --chat

# Test RAG indexing
cargo run -- --rag-index /path/to/project
```

### WebRTC Debugging
```bash
# Test screen capture
cargo run -- --test-screen

# Debug WebRTC connection
RUST_LOG=webrtc=debug cargo run -- --web

# Check browser console for client-side errors
```

## 📊 Monitoring & Observability

### Metrics Collection
```rust
// src/shared/performance/metrics.rs
pub struct MetricsCollector {
    voice_recognition_time: Histogram,
    ai_response_time: Histogram,
    command_execution_time: Histogram,
}

impl MetricsCollector {
    pub fn record_voice_recognition(&self, duration: Duration) {
        self.voice_recognition_time.observe(duration.as_millis() as f64);
    }
}
```

### Health Checks
```rust
// src/presentation/web/health.rs
pub async fn health_check(State(state): State<AppState>) -> Json<HealthStatus> {
    let voice_health = state.voice_service.health_check().await;
    let ai_health = state.ai_service.health_check().await;
    let db_health = state.database.health_check().await;

    Json(HealthStatus {
        status: if voice_health && ai_health && db_health { "healthy" } else { "degraded" },
        voice: voice_health,
        ai: ai_health,
        database: db_health,
    })
}
```

## 🤝 Contributing

### Pull Request Process
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Review Guidelines
- All PRs require review from at least one maintainer
- Tests must pass and coverage maintained
- Documentation updated for user-facing changes
- Security review for any new dependencies

### Issue Reporting
- Use issue templates for bug reports and feature requests
- Include system information and reproduction steps
- Attach logs with `RUST_LOG=debug` when reporting bugs

This development guide provides the foundation for contributing to bro. The unified architecture combines the best of voice automation and AI assistance into a powerful, user-friendly system.</content>
<parameter name="filePath">/home/rendi/projects/bro/docs/development.md