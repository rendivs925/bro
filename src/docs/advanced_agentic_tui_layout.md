# Advanced Agentic TUI Layout Design

## Overview
This document outlines the comprehensive TUI layout for the most advanced agentic AI tool in Rust, building upon the existing "bro" codebase. The design focuses on a minimalist, vim-friendly interface that supports the full agentic workflow: goal → plan → execute → observe → repeat.

## Core Architecture

### Existing Components Leveraged
- **TUI Framework**: Ratatui with crossterm for terminal control
- **Agent Services**: Existing agent_service.rs with streaming planning and execution
- **Memory Systems**: Semantic memory, RAG, session storage
- **Tool Registry**: Infrastructure tools with validation and security
- **Agent Control**: Execution bounds, verification, and safety

### New Components to Add
1. **Agent Orchestrator**: Deterministic loop with planner-executor separation
2. **Enhanced TUI App**: Agent loop integration with real-time feedback
3. **Memory Dashboard**: Inspect and manage agent memory layers
4. **Execution Traces**: Debug and replay capabilities

## Layout Structure

### Base Layout
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ bro v0.1.0 - Agentic AI Assistant                     [SESSION: name] [MODE] │
├─────────────────────────────────────────────────────────────────────────────┤
│ [MODE] Status message - key hints                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                            Main Content Area                                │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Footer: Agent status | Progress | Resources | Actions                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Mode-Specific Layouts

#### 1. Ready/Idle State
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ bro v0.1.0 - Agentic AI Assistant                     [SESSION: main]        │
├─────────────────────────────────────────────────────────────────────────────┤
│ [NORMAL] Ready - Type 'i' to input goal, ':' for commands                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│                                                                             │
│                            [AGENT STATUS: IDLE]                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Agent: Idle | Session: main | Memory: 45MB | Tools: 12                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 2. Goal Input Phase
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ bro v0.1.0 - Agentic AI Assistant                     [SESSION: refactor]    │
├─────────────────────────────────────────────────────────────────────────────┤
│ [INSERT] Type your goal, press Enter to submit, Esc for normal mode         │
├─────────────────────────────────────────────────────────────────────────────┤
│ Refactor the authentication middleware in auth.rs to use dependency         │
│ injection instead of global state. Ensure all tests pass and maintain      │
│ API compatibility.                                                         │
│                                                                             │
│ [AGENT STATUS: CLASSIFYING INTENT...]                                      │
│                                                                             │
│                                                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ Intent: MultiStep | Confidence: 0.94 | Tools: FileRead, FileEdit, TestRun  │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 3. Planning Phase
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ bro v0.1.0 - Agentic AI Assistant                     [SESSION: refactor]    │
├─────────────────────────────────────────────────────────────────────────────┤
│ [NORMAL] Plan ready - Press 'y' to execute, 'e' to edit, 'q' to cancel      │
├─────────────────────────────────────────────────────────────────────────────┤
│ EXECUTION PLAN: Refactor auth middleware (4 steps, ~8 min)                 │
│                                                                             │
│ 1. [FileRead] Analyze current auth.rs structure and dependencies           │
│    Risk: InfoOnly | Time: 30s                                              │
│                                                                             │
│ 2. [FileEdit] Extract global state into injectable dependencies            │
│    Risk: SystemChanges | Time: 3min | Rollback: git reset                   │
│                                                                             │
│ 3. [FileEdit] Update middleware constructor and usage patterns             │
│    Risk: SystemChanges | Time: 3min | Rollback: git reset                   │
│                                                                             │
│ 4. [Command] Run test suite to verify changes                              │
│    Risk: SafeOperations | Time: 1min                                        │
│                                                                             │
│ [AGENT STATUS: AWAITING APPROVAL] [y/n/e/q]                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│ Disk Impact: ~2MB | Network: No | Safety: Medium (system changes)          │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 4. Execution Phase
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ bro v0.1.0 - Agentic AI Assistant                     [SESSION: refactor]    │
├─────────────────────────────────────────────────────────────────────────────┤
│ [NORMAL] Executing step 2/4 - Press 'p' for pause, 'q' for abort            │
├─────────────────────────────────────────────────────────────────────────────┤
│ STEP 2: [FileEdit] Extract global state into injectable dependencies       │
│                                                                             │
│ [EXECUTING] AI is generating code changes...                               │
│                                                                             │
│ ┌─ auth.rs ──────────────────────────────────────────────────────────────┐  │
│ │ // Before:                                                             │  │
│ │ static AUTH_CONFIG: OnceCell<AuthConfig> = OnceCell::new();            │  │
│ │                                                                       │  │
│ │ // After:                                                              │  │
│ │ #[derive(Clone)]                                                       │  │
│ │ pub struct AuthMiddleware {                                            │  │
│ │     config: Arc<AuthConfig>,                                           │  │
│ │ }                                                                       │  │
│ └─────────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│ [AGENT STATUS: EXECUTING] [Step 2/4] [Time: 45s/3min]                      │
├─────────────────────────────────────────────────────────────────────────────┤
│ Confidence: 0.87 | Tools Used: FileRead(1), CodeAnalysis(2) | Memory: 45MB │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 5. Results Phase
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ bro v0.1.0 - Agentic AI Assistant                     [SESSION: refactor]    │
├─────────────────────────────────────────────────────────────────────────────┤
│ [NORMAL] Execution complete - Press 'r' to review, 'n' for new task         │
├─────────────────────────────────────────────────────────────────────────────┤
│ ✅ REFACTORING COMPLETE                                                      │
│                                                                             │
│ SUMMARY:                                                                    │
│ • 4/4 steps completed successfully                                          │
│ • 2 files modified, 147 lines changed                                       │
│ • All tests passing (23/23)                                                 │
│ • Git commit created: "feat: refactor auth middleware to use DI"           │
│                                                                             │
│ NEXT ACTIONS:                                                               │
│ • Run integration tests?                                                    │
│ • Update documentation?                                                     │
│ • Deploy to staging?                                                        │
│                                                                             │
│ [AGENT STATUS: COMPLETE] [r/n/:suggest/:undo]                               │
├─────────────────────────────────────────────────────────────────────────────┤
│ Session: refactor | Duration: 6m 32s | Cost: $0.12 | Memory: 89MB          │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Key UI Components

### Status Bar
- **Mode**: [NORMAL] [INSERT] [COMMAND]
- **Session**: Current session name and status
- **Agent Status**: IDLE | PLANNING | EXECUTING | COMPLETE | ERROR
- **Quick Actions**: Contextual key hints

### Main Content Area
- **Dynamic Layout**: Adapts based on current phase
- **Scrollable**: Large plans/outputs with navigation
- **Syntax Highlighting**: Code diffs and outputs
- **Progress Indicators**: Visual progress bars for long operations

### Footer Bar
- **Agent Metrics**: Confidence scores, tool usage, memory
- **Resource Usage**: CPU, memory, network, disk
- **Action Hints**: Available commands based on context

## Overlays

### Context Overlay (Ctrl+O)
```
┌─ Context ──────────────────────────────────────────────────────────────────┐
│ Project Structure                                                          │
│ ├── src/                                                                   │
│ │   ├── main.rs                                                            │
│ │   ├── auth.rs              [MODIFIED]                                    │
│ │   └── user.rs                                                            │
│ ├── tests/                                                                 │
│ │   └── auth_tests.rs        [MODIFIED]                                    │
│ └── Cargo.toml                                                             │
│                                                                            │
│ Recent Files                                                               │
│ • auth.rs - 5 min ago                                                     │
│ • user.rs - 2 hours ago                                                   │
│ • main.rs - 1 day ago                                                     │
│                                                                            │
│ Git Status                                                                 │
│ M auth.rs                                                                  │
│ M auth_tests.rs                                                            │
│                                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Sessions Overlay (Ctrl+S)
```
┌─ Sessions ─────────────────────────────────────────────────────────────────┐
│ Active Session: refactor                                                   │
│                                                                            │
│ Available Sessions                                                         │
│ ▶ refactor - "Refactor auth middleware" (6m 32s ago)                      │
│   feature-api - "Add user profile API" (2 days ago)                       │
│   bugfix-validation - "Fix email validation" (1 week ago)                 │
│                                                                            │
│ Session Actions                                                            │
│ [n] Create new session                                                     │
│ [d] Delete current session                                                 │
│ [s] Switch session                                                         │
│                                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Tools Overlay (Ctrl+K)
```
┌─ Tools ────────────────────────────────────────────────────────────────────┐
│ Available Tools                                                            │
│ ▶ [FileRead] Read and analyze files                                       │
│   [FileEdit] Modify files with precision                                  │
│   [Command] Execute shell commands                                        │
│   [WebSearch] Search the web for information                              │
│   [TestRun] Execute test suites                                           │
│   [GitCommit] Create git commits                                          │
│                                                                            │
│ Recent Tool Usage                                                          │
│ • FileRead (auth.rs) - 2 min ago                                          │
│ • FileEdit (auth.rs) - 1 min ago                                          │
│ • TestRun (auth_tests) - 30s ago                                          │
│                                                                            │
│ Tool Statistics                                                            │
│ Total Calls: 47 | Success Rate: 95% | Avg Response: 1.2s                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Vim-Friendly Keybindings

### Normal Mode
- **Navigation**: h/j/k/l (no conflicts with Ctrl+hjkl)
- **Mode Switching**: i (insert), : (command)
- **Actions**: y (yes), n (no), e (edit), q (quit)
- **Overlays**: Ctrl+O (context), Ctrl+S (sessions), Ctrl+P (palette), Ctrl+K (tools)

### Insert Mode
- **Submit**: Enter
- **Cancel**: Esc
- **Edit**: Standard text editing keys

### Command Mode
- **Execute**: Enter
- **Cancel**: Esc
- **Complete**: Tab

## Implementation Phases

### Phase 1: Base Layout & State Management
- [ ] Extend TuiApp with agent loop state
- [ ] Add agent status tracking
- [ ] Implement mode-specific layouts
- [ ] Create base overlay system

### Phase 2: Planning Integration
- [ ] Integrate with agent_service planning
- [ ] Add plan visualization
- [ ] Implement approval workflow
- [ ] Add plan editing capabilities

### Phase 3: Execution Integration
- [ ] Real-time execution feedback
- [ ] Progress indicators
- [ ] Error handling and recovery
- [ ] Pause/resume functionality

### Phase 4: Results & Actions
- [ ] Results summary display
- [ ] Next action suggestions
- [ ] Undo/rollback integration
- [ ] Session persistence

### Phase 5: Advanced Features
- [ ] Memory inspection overlay
- [ ] Execution trace replay
- [ ] Multi-agent coordination UI
- [ ] Performance metrics dashboard

## Real-World Use Cases Covered

1. **Code Refactoring**: Multi-step file modifications with testing
2. **Bug Fixing**: Error detection, analysis, and autonomous fixes
3. **Feature Implementation**: Complex multi-file feature development
4. **System Administration**: Command execution with safety checks
5. **Research Tasks**: Web search, analysis, and synthesis

This design provides a complete, production-ready interface for advanced agentic AI workflows while maintaining the minimalist, vim-friendly philosophy.</content>
<parameter name="filePath">docs/advanced_agentic_tui_layout.md