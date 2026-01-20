# Bro: Voice-Powered AI CLI Unification Plan

## Overview

This document outlines the comprehensive plan to unify `vibespeak` and `vibe_cli` into a single monorepo called "bro" - a voice-powered AI coding assistant that enables hands-free development from anywhere.

## Vision

Create a unified tool that allows users to work without being physically at their desk, using voice commands to control AI-assisted coding while streaming their desktop screen to mobile devices.

## Architecture

### Unified Structure
```
bro/
├── src/                    # Unified codebase (formerly vibe_cli)
│   ├── domain/            # Business logic entities
│   ├── application/       # Use cases and orchestration
│   ├── infrastructure/    # External integrations (voice, AI, etc.)
│   ├── presentation/      # Interfaces (CLI, TUI, Web)
│   └── shared/            # Common utilities
├── docs/                  # Documentation
├── models/                # AI/ML models
├── config/                # Configuration files
└── tests/                 # Comprehensive tests
```

### Component Integration

#### From vibespeak:
- **Voice Recognition**: Vosk-based offline speech recognition
- **TTS**: Piper neural voice synthesis
- **Browser Automation**: Chromium-based web control
- **Web Interface**: Leptos WASM frontend
- **Screen Sharing**: WebRTC-based desktop streaming

#### From vibe_cli:
- **AI Assistant**: Ollama-powered local LLM inference
- **RAG System**: Retrieval-Augmented Generation
- **Agent Services**: Multi-step task planning
- **Safety Sandbox**: Command execution isolation
- **CLI/TUI**: Terminal interfaces

## Implementation Phases

### Phase 1: Structural Unification (Week 1)
1. **Rename vibe_cli to src**
   - Move `/vibe_cli/` → `/src/`
   - Update all import paths
   - Update workspace configuration

2. **Merge Cargo.toml configurations**
   - Combine dependencies from both projects
   - Resolve version conflicts
   - Update binary name to "bro"

3. **Establish unified project structure**
   - Create docs/ directory
   - Merge configuration systems
   - Update README and documentation

### Phase 2: Voice Integration (Week 2-3)

#### Wake Word System ("bro ...")
- **Background Listening**: Low-power Vosk model for wake word detection
- **Activation**: Switch to full recognition on "bro" detection
- **Timeout**: Automatic deactivation after command completion
- **Configuration**: Customizable wake phrases

#### Voice Input Pipeline
- **Speech Recognition**: Integrate VoskAdapter into infrastructure layer
- **Intent Classification**: Route voice commands to appropriate modes
- **Error Handling**: Restart listening session on recognition failure
- **Multi-source**: Support both server (desktop) and client (mobile) microphones

#### Voice-to-Code Generation
- **Reuse Existing**: Leverage vibe_cli's agent and RAG services
- **Context Awareness**: Project-specific code generation
- **Multi-language**: Support Rust, Python, Bash, JavaScript, TypeScript
- **Safety**: Apply existing sandbox and confirmation systems

### Phase 3: Mobile Web Interface (Week 4-5)

#### WebRTC Screen Streaming
- **Server-side Capture**: Record desktop where bro runs
- **Client Streaming**: Live video feed to mobile browsers
- **No Permissions**: Server captures screen, no client browser permissions needed
- **Performance**: Optimized streaming for mobile networks

#### Touch Controls
- **Gesture Mapping**: Mobile touches → desktop actions
- **Voice Integration**: Client microphone → server processing
- **Feedback**: Visual and audio confirmation of actions

#### Mobile-Optimized UI
- **Responsive Design**: Touch-friendly interface
- **Voice Status**: Real-time feedback on voice recognition
- **Command History**: Access to recent voice commands
- **Quick Actions**: Common commands as touch buttons

### Phase 4: Enhanced AI Capabilities (Week 6-7)

#### Zero-Cost AI Workflow
- **Computer Vision**: OCR for screen content analysis
- **Playwright Integration**: Browser automation for ChatGPT access
- **Response Streaming**: Real-time AI output parsing
- **Fallback System**: Local LLM primary, ChatGPT backup

#### Advanced Voice Commands
- **Conversational AI**: Natural language task description
- **Multi-step Planning**: Complex workflow generation
- **Code Analysis**: Voice-driven code review and explanation
- **System Administration**: Voice-controlled server management

### Phase 5: CLI Enhancement (Week 8-9)

#### Voice CLI Modes
```bash
# Voice-powered modes
bro --voice --chat          # Voice conversations with AI
bro --voice --agent         # Voice-driven multi-step tasks
bro --voice --rag           # Voice codebase queries
bro --voice --code          # Direct voice-to-code generation

# Pure voice interface
bro --voice-only            # No text input, all voice
bro --continuous-listening  # Always listening with wake word

# Mobile interface
bro --web                   # Start mobile web interface
bro --web --voice           # Web interface with voice integration
```

#### Enhanced Features
- **Dictation Mode**: Voice typing anywhere (editors, browsers, terminals)
- **Session Management**: Voice command history and favorites
- **Performance Monitoring**: Voice-activated system diagnostics

### Phase 6: Testing & Optimization (Week 10-11)

#### Voice Recognition Testing
- **Accuracy**: Test wake word and command recognition
- **Noise Handling**: Performance in various environments
- **Multi-accent**: Support for different speech patterns
- **Language Support**: English primary, extensible to others

#### Mobile Interface Testing
- **Cross-device**: iOS Safari, Android Chrome, desktop browsers
- **Network Conditions**: Performance on 3G/4G/5G
- **Touch Latency**: Optimize for real-time feel
- **Battery Impact**: Minimize mobile battery drain

#### AI Integration Testing
- **Response Quality**: Code generation accuracy and relevance
- **Safety Validation**: Confirm all voice commands pass security checks
- **Performance**: Sub-second response times for voice commands
- **Context Awareness**: Accurate project-specific code generation

### Phase 7: Documentation & Release (Week 12)

#### User Documentation
- **Getting Started**: Installation and basic voice setup
- **Voice Commands**: Complete command reference
- **Mobile Interface**: Web interface usage guide
- **Troubleshooting**: Common issues and solutions

#### Developer Documentation
- **Architecture**: Clean architecture explanation
- **Contributing**: Development setup and contribution guidelines
- **API Reference**: Internal API documentation
- **Security**: Security model and best practices

## Technical Specifications

### Voice Processing
- **Wake Word**: "bro ..." with configurable alternatives
- **Recognition**: Vosk offline models for privacy
- **TTS**: Piper neural voices for natural feedback
- **Sampling**: 16kHz mono for optimal recognition
- **Latency**: <500ms wake word detection, <2s command processing

### AI Capabilities
- **Local LLM**: Qwen 2.5 3B via Ollama
- **RAG System**: Vector-based codebase understanding
- **Safety**: Sandbox execution with resource limits
- **Caching**: Ultra-fast response caching
- **Memory**: Long-term conversation context

### Mobile Interface
- **WebRTC**: Screen streaming without browser permissions
- **Responsive**: Mobile-first design with touch optimization
- **Real-time**: Sub-100ms latency for controls
- **Cross-platform**: iOS, Android, desktop browser support

### Security & Privacy
- **Local Processing**: All voice recognition offline
- **No Data Transmission**: Audio stays on device
- **Sandbox Execution**: Isolated command running
- **Confirmation System**: User approval for all actions
- **Secrets Detection**: Prevent sensitive data exposure

## Success Metrics

### User Experience
- **Activation Time**: <1 second from wake word to ready
- **Recognition Accuracy**: >95% for clear speech
- **Task Completion**: >80% of voice commands successful on first attempt
- **Mobile Latency**: <200ms touch-to-action delay

### Technical Performance
- **Memory Usage**: <300MB base memory footprint
- **CPU Usage**: <5% during idle listening, <20% during active processing
- **Battery Impact**: <10% additional drain on mobile devices
- **Network Usage**: <50KB/min during screen streaming

### AI Quality
- **Code Generation**: >90% syntactically correct generated code
- **Context Accuracy**: >85% of generated code matches project conventions
- **Safety Compliance**: 100% of commands pass security validation
- **Response Time**: <3 seconds for typical voice-to-code requests

## Risk Mitigation

### Technical Risks
- **Voice Recognition**: Offline fallback, clear error messages, restart on failure
- **WebRTC Complexity**: Simplified architecture, extensive testing
- **AI Integration**: Local-first approach, graceful degradation
- **Mobile Compatibility**: Progressive enhancement, broad testing

### User Experience Risks
- **Learning Curve**: Intuitive wake word, helpful error messages
- **False Activations**: Configurable sensitivity, easy deactivation
- **Privacy Concerns**: Local processing, transparent data handling
- **Performance Issues**: Optimization focus, resource monitoring

## Conclusion

The unified "bro" tool will revolutionize hands-free coding by combining the best of voice automation and AI assistance. Users can work from anywhere - fishing, working out, traveling - while maintaining full control over their development environment through natural voice commands and mobile screen streaming.

This plan ensures a smooth transition from two separate tools to one cohesive, powerful system that maintains all existing capabilities while adding transformative voice and mobile features.</content>
<parameter name="filePath">/home/rendi/projects/bro/docs/unification-plan.md