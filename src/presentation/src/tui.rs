use std::io::{self, stdout, Stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::cli::{Cli, CliApp};
use clap::Parser;
use infrastructure::config::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentType {
    Question,
    Command,
    FileRead,
    FileEdit,
    FileSearch,
    MultiStep,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub intent_type: IntentType,
    pub description: String,
    pub confidence: f32,
    pub details: Option<String>,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [domain::session::Message],
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    message: domain::session::Message,
}

/// Agent execution phase
#[derive(Debug, Clone, PartialEq)]
pub enum AgentPhase {
    Idle,
    ClassifyingIntent,
    Planning,
    AwaitingApproval,
    Executing {
        current_step: usize,
        total_steps: usize,
    },
    Complete,
    Error,
}

/// Agent status information
#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub phase: AgentPhase,
    pub confidence: Option<f32>,
    pub current_goal: Option<String>,
    pub execution_time: Option<std::time::Duration>,
    pub tools_used: Vec<String>,
    pub memory_usage: Option<u64>,
    pub error_message: Option<String>,
}

/// TUI application state
pub struct TuiApp {
    cli_app: CliApp,
    config: Config,
    current_mode: TuiMode,
    input_buffer: String,
    cursor_position: usize,
    status_message: String,
    show_overlay: Option<Overlay>,
    session_list: Vec<String>,
    current_session: Option<String>,
    command_history: Vec<String>,
    history_index: Option<usize>,
    tui_mode: Option<String>, // Current TUI mode (plan, build, run, chat, etc.)
    pending_action: Option<PendingAction>, // Action confirmed but not yet executed

    // Agent loop state
    agent_status: AgentStatus,
    current_plan: Option<domain::models::AgentResponse>,
    execution_progress: Vec<String>, // Step-by-step execution log
    scroll_offset: usize,            // For scrolling through large content
}

/// TUI runner that manages the terminal
pub struct TuiRunner {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app: TuiApp,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuiMode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug, Clone)]
pub enum Overlay {
    Sessions,
    Tools,
    Context,
    Palette,
    Confirmation {
        message: String,
        default_yes: bool,
        action: PendingAction,
    },
    Response {
        title: String,
        content: String,
        scroll_offset: usize,
    },
    Thinking {
        message: String,
        step: usize,
    },
}

#[derive(Debug, Clone)]
pub enum PendingAction {
    Command(String),
    FileRead(String),
    FileEdit(String),
    FileSearch(String),
    MultiStep(String),
    Unknown(String),
}

impl TuiApp {
    /// Start an agent workflow with the given goal
    pub async fn start_agent_workflow(&mut self, goal: String) -> Result<()> {
        // Reset agent state
        self.agent_status = AgentStatus {
            phase: AgentPhase::ClassifyingIntent,
            confidence: None,
            current_goal: Some(goal.clone()),
            execution_time: None,
            tools_used: Vec::new(),
            memory_usage: None,
            error_message: None,
        };
        self.current_plan = None;
        self.execution_progress.clear();
        self.scroll_offset = 0;

        // Here we would trigger the actual agent service call
        // For now, simulate the workflow progression
        self.simulate_agent_workflow(goal).await?;

        Ok(())
    }

    /// Simulate agent workflow for demonstration (replace with real agent calls)
    async fn simulate_agent_workflow(&mut self, goal: String) -> Result<()> {
        use tokio::time::{sleep, Duration};

        // Phase 1: Classifying intent
        sleep(Duration::from_millis(500)).await;
        self.agent_status.confidence = Some(0.94);

        // Phase 2: Planning
        self.agent_status.phase = AgentPhase::Planning;
        sleep(Duration::from_millis(1500)).await;

        // Create a mock plan
        let mock_plan = domain::models::AgentResponse {
            reasoning: vec![
                "Analyzed project structure and dependencies".to_string(),
                "Identified 2 files that need modification".to_string(),
                "Planned 4 sequential steps with safety checks".to_string(),
            ],
            tool_calls: vec![
                domain::models::ToolCall {
                    id: "1".to_string(),
                    name: "FileRead".to_string(),
                    parameters: [("path".to_string(), serde_json::json!("auth.rs"))].into(),
                    reasoning: "Reading current authentication implementation".to_string(),
                },
                domain::models::ToolCall {
                    id: "2".to_string(),
                    name: "FileEdit".to_string(),
                    parameters: [("path".to_string(), serde_json::json!("auth.rs"))].into(),
                    reasoning: "Refactoring to use dependency injection".to_string(),
                },
            ],
            tool_results: vec![],
            final_response: "Plan created successfully".to_string(),
            confidence: 0.87,
        };
        self.current_plan = Some(mock_plan);

        // Phase 3: Awaiting approval
        self.agent_status.phase = AgentPhase::AwaitingApproval;

        Ok(())
    }

    /// Execute the approved plan
    pub async fn execute_approved_plan(&mut self) -> Result<()> {
        if let Some(plan) = &self.current_plan {
            self.agent_status.phase = AgentPhase::Executing {
                current_step: 1,
                total_steps: plan.tool_calls.len(),
            };
            self.execution_progress.clear();

            // Simulate execution steps
            for (i, tool_call) in plan.tool_calls.iter().enumerate() {
                let step_num = i + 1;
                self.agent_status.phase = AgentPhase::Executing {
                    current_step: step_num,
                    total_steps: plan.tool_calls.len(),
                };

                self.execution_progress
                    .push(format!("Step {}: {}", step_num, tool_call.reasoning));

                // Simulate execution time
                use tokio::time::{sleep, Duration};
                sleep(Duration::from_millis(1000)).await;
            }

            // Mark as complete
            self.agent_status.phase = AgentPhase::Complete;
            self.agent_status.execution_time = Some(std::time::Duration::from_secs(6));
            self.agent_status.tools_used =
                plan.tool_calls.iter().map(|tc| tc.name.clone()).collect();
            self.agent_status.memory_usage = Some(89 * 1024 * 1024); // 89MB
        }

        Ok(())
    }

    /// Create a new TUI application state
    pub fn new(_cli: Cli) -> Result<Self> {
        let cli_app = CliApp::new();
        let config = Config::load(); // safe_mode=true, cache_enabled=true, copy=false

        // Initialize CLI app with the parsed CLI args
        // Note: We'll handle the TUI-specific logic separately

        Ok(Self {
            cli_app,
            config,
            current_mode: TuiMode::Insert,
            input_buffer: String::new(),
            cursor_position: 0,
            status_message:
                "INSERT - Type your command, press Enter to execute, Esc for normal mode"
                    .to_string(),
            show_overlay: None,
            session_list: vec!["default".to_string()],
            current_session: Some("default".to_string()),
            command_history: Vec::new(),
            history_index: None,
            tui_mode: None,
            pending_action: None,
            // Initialize agent state
            agent_status: AgentStatus {
                phase: AgentPhase::Idle,
                confidence: None,
                current_goal: None,
                execution_time: None,
                tools_used: Vec::new(),
                memory_usage: None,
                error_message: None,
            },
            current_plan: None,
            execution_progress: Vec::new(),
            scroll_offset: 0,
        })
    }
}

impl TuiRunner {
    /// Create a new TUI runner with terminal
    pub fn new(cli: Cli) -> Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;
        let app = TuiApp::new(cli)?;

        Ok(Self { terminal, app })
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), EnterAlternateScreen, Show)?;
        self.terminal.clear()?;

        // Main event loop
        loop {
            // Draw the UI
            let app = &self.app;
            self.terminal.draw(move |f| Self::draw_ui(f, app))?;

            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.app.current_mode {
                        TuiMode::Normal => {
                            if self.handle_normal_mode(key).await? {
                                break; // Exit application
                            }
                        }
                        TuiMode::Insert => {
                            self.handle_insert_mode(key).await?;
                        }
                        TuiMode::Command => {
                            if self.handle_command_mode(key).await? {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Cleanup
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, Show)?;

        Ok(())
    }

    /// Handle normal mode key events (vim-style)
    async fn handle_normal_mode(&mut self, key: event::KeyEvent) -> Result<bool> {
        // Handle agent approval actions in awaiting approval phase
        if let AgentPhase::AwaitingApproval = self.app.agent_status.phase {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    // Execute the approved plan
                    self.app.show_overlay = None;
                    self.app.execute_approved_plan().await?;
                    return Ok(false);
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    // Edit the plan (placeholder - would open editor)
                    self.app.show_overlay = Some(Overlay::Response {
                        title: "Plan Editing".to_string(),
                        content: "Plan editing not yet implemented. Press any key to continue."
                            .to_string(),
                        scroll_offset: 0,
                    });
                    return Ok(false);
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    // Cancel the plan
                    self.app.agent_status.phase = AgentPhase::Idle;
                    self.app.current_plan = None;
                    self.app.show_overlay = None;
                    self.app.status_message = "Plan cancelled".to_string();
                    return Ok(false);
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    // Show plan details
                    if let Some(plan) = &self.app.current_plan {
                        let details = format!(
                            "Plan Details:\n• {} steps\n• Confidence: {:.1}%\n• Tools: {}\n\nReasoning:\n{}",
                            plan.tool_calls.len(),
                            plan.confidence * 100.0,
                            plan.tool_calls.iter().map(|tc| tc.name.as_str()).collect::<Vec<_>>().join(", "),
                            plan.reasoning.join("\n")
                        );
                        self.app.show_overlay = Some(Overlay::Response {
                            title: "Plan Details".to_string(),
                            content: details,
                            scroll_offset: 0,
                        });
                    }
                    return Ok(false);
                }
                _ => {} // Continue with normal key handling
            }
        }

        // Handle completion phase actions
        if let AgentPhase::Complete = self.app.agent_status.phase {
            match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    // Review results
                    let summary = if let Some(plan) = &self.app.current_plan {
                        format!(
                            "Execution Summary:\n\nGoal: {}\n\nCompleted {} steps\nTools Used: {}\nExecution Time: {:?}\nConfidence: {:.1}%\n\nNext actions available:\n• Run tests\n• Update documentation\n• Deploy changes",
                            self.app.agent_status.current_goal.as_deref().unwrap_or("Unknown"),
                            plan.tool_calls.len(),
                            self.app.agent_status.tools_used.join(", "),
                            self.app.agent_status.execution_time.unwrap_or_default(),
                            plan.confidence * 100.0
                        )
                    } else {
                        "No execution data available".to_string()
                    };
                    self.app.show_overlay = Some(Overlay::Response {
                        title: "Execution Review".to_string(),
                        content: summary,
                        scroll_offset: 0,
                    });
                    return Ok(false);
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    // Start new task
                    self.app.agent_status.phase = AgentPhase::Idle;
                    self.app.current_plan = None;
                    self.app.execution_progress.clear();
                    self.app.current_mode = TuiMode::Insert;
                    self.app.status_message = "Ready for new goal".to_string();
                    return Ok(false);
                }
                _ => {} // Continue with normal key handling
            }
        }

        match key.code {
            // Quit commands
            KeyCode::Char('q') => return Ok(true), // Quit
            KeyCode::Char('Z') if key.modifiers.contains(KeyModifiers::SHIFT) => return Ok(true), // ZZ

            // Mode switching
            KeyCode::Char('i') => {
                self.app.current_mode = TuiMode::Insert;
                self.app.status_message = "INSERT".to_string();
            }
            KeyCode::Char('I') => {
                // Insert at beginning of line
                self.app.current_mode = TuiMode::Insert;
                self.app.cursor_position = 0;
                self.app.status_message = "INSERT".to_string();
            }
            KeyCode::Char('a') => {
                // Append after cursor
                self.app.current_mode = TuiMode::Insert;
                self.app.status_message = "INSERT".to_string();
            }
            KeyCode::Char('A') => {
                // Append at end of line
                self.app.current_mode = TuiMode::Insert;
                self.app.cursor_position = self.app.input_buffer.len();
                self.app.status_message = "INSERT".to_string();
            }
            KeyCode::Char('O') => {
                // Open new line above (like vim 'O')
                self.app.input_buffer.insert(0, '\n');
                self.app.cursor_position = 1;
                self.app.current_mode = TuiMode::Insert;
                self.app.status_message = "INSERT".to_string();
            }
            KeyCode::Char(':') => {
                self.app.current_mode = TuiMode::Command;
                self.app.input_buffer.clear();
                self.app.cursor_position = 0;
                self.app.status_message = "COMMAND".to_string();
            }

            // Vim-style navigation (hjkl)
            KeyCode::Char('h') => {
                if self.app.cursor_position > 0 {
                    self.app.cursor_position -= 1;
                }
            }
            KeyCode::Char('l') => {
                if self.app.cursor_position < self.app.input_buffer.len() {
                    self.app.cursor_position += 1;
                }
            }
            KeyCode::Char('w') => {
                // Move to next word
                let rest = &self.app.input_buffer[self.app.cursor_position..];
                if let Some(word_end) = rest.find(|c: char| c.is_whitespace()) {
                    self.app.cursor_position += word_end + 1;
                    // Skip additional whitespace
                    while self.app.cursor_position < self.app.input_buffer.len()
                        && self
                            .app
                            .input_buffer
                            .chars()
                            .nth(self.app.cursor_position)
                            .unwrap()
                            .is_whitespace()
                    {
                        self.app.cursor_position += 1;
                    }
                } else {
                    self.app.cursor_position = self.app.input_buffer.len();
                }
            }
            KeyCode::Char('b') => {
                // Move to previous word
                if self.app.cursor_position > 0 {
                    let mut pos = self.app.cursor_position - 1;
                    // Skip current whitespace
                    while pos > 0
                        && self
                            .app
                            .input_buffer
                            .chars()
                            .nth(pos)
                            .unwrap()
                            .is_whitespace()
                    {
                        pos -= 1;
                    }
                    // Find word start
                    while pos > 0
                        && !self
                            .app
                            .input_buffer
                            .chars()
                            .nth(pos - 1)
                            .unwrap()
                            .is_whitespace()
                    {
                        pos -= 1;
                    }
                    self.app.cursor_position = pos;
                }
            }
            KeyCode::Char('0') => {
                self.app.cursor_position = 0; // Beginning of line
            }
            KeyCode::Char('$') => {
                self.app.cursor_position = self.app.input_buffer.len(); // End of line
            }

            // Vim-style editing
            KeyCode::Char('x') => {
                // Delete character under cursor
                if self.app.cursor_position < self.app.input_buffer.len() {
                    self.app.input_buffer.remove(self.app.cursor_position);
                }
            }
            KeyCode::Char('X') => {
                // Delete character before cursor
                if self.app.cursor_position > 0 {
                    self.app.cursor_position -= 1;
                    self.app.input_buffer.remove(self.app.cursor_position);
                }
            }
            KeyCode::Char('d') => {
                // Delete line (dd)
                self.app.input_buffer.clear();
                self.app.cursor_position = 0;
            }

            // Overlays and special functions (Ctrl+key)
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+O: Show context overlay
                self.app.show_overlay = Some(Overlay::Context);
                self.app.status_message = "CONTEXT".to_string();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+S: Show sessions overlay
                self.app.show_overlay = Some(Overlay::Sessions);
                self.app.status_message = "SESSIONS".to_string();
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+P: Show command palette
                self.app.show_overlay = Some(Overlay::Palette);
                self.app.status_message = "PALETTE".to_string();
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+K: Show tools overlay
                self.app.show_overlay = Some(Overlay::Tools);
                self.app.status_message = "TOOLS".to_string();
            }

            // Regular 'o' key (must come after Ctrl+O to avoid unreachable pattern)
            KeyCode::Char('o') => {
                // Open new line below (like vim 'o')
                self.app.input_buffer.push('\n');
                self.app.cursor_position = self.app.input_buffer.len();
                self.app.current_mode = TuiMode::Insert;
                self.app.status_message = "INSERT".to_string();
            }

            // History navigation (bash-style)
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+P: Previous command
                self.navigate_history(true);
            }
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+N: Next command
                self.navigate_history(false);
            }
            KeyCode::Up => {
                // Arrow up: Previous command
                self.navigate_history(true);
            }
            KeyCode::Down => {
                // Arrow down: Next command
                self.navigate_history(false);
            }

            KeyCode::Esc => {
                // Dismiss overlay
                self.app.show_overlay = None;
                self.app.status_message = "Ready".to_string();
            }
            KeyCode::Char(c) => {
                // Handle overlay-specific keybindings
                if let Some(overlay) = &self.app.show_overlay {
                    match overlay {
                        Overlay::Sessions => {
                            self.handle_sessions_overlay_key(c);
                        }
                        Overlay::Tools => {
                            self.handle_tools_overlay_key(c);
                        }
                        Overlay::Context => {
                            self.handle_context_overlay_key(c);
                        }
                        Overlay::Palette => {
                            self.handle_palette_overlay_key(c);
                        }
                        Overlay::Confirmation { .. } => {
                            self.handle_confirmation_overlay_key(c);
                        }
                        Overlay::Response { .. } => {
                            self.handle_response_overlay_key(c);
                        }
                        Overlay::Thinking { .. } => {
                            // Thinking overlay doesn't handle input
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// Navigate command history
    fn navigate_history(&mut self, previous: bool) {
        if self.app.command_history.is_empty() {
            return;
        }

        let current_index = self
            .app
            .history_index
            .unwrap_or(self.app.command_history.len());

        let new_index = if previous {
            if current_index > 0 {
                current_index - 1
            } else {
                current_index
            }
        } else {
            if current_index < self.app.command_history.len() {
                current_index + 1
            } else {
                current_index
            }
        };

        if new_index < self.app.command_history.len() {
            self.app.input_buffer = self.app.command_history[new_index].clone();
            self.app.history_index = Some(new_index);
        } else {
            // At the end, clear the buffer
            self.app.input_buffer.clear();
            self.app.history_index = None;
        }
        self.app.cursor_position = self.app.input_buffer.len();
    }

    /// Handle sessions overlay key events
    fn handle_sessions_overlay_key(&mut self, key: char) {
        match key {
            '1'..='9' => {
                // Select session by number
                let index = (key as u8 - b'1') as usize;
                if index < self.app.session_list.len() {
                    let session_name = self.app.session_list[index].clone();
                    self.app.current_session = Some(session_name.clone());
                    self.app.show_overlay = None;
                    self.app.status_message = format!("Switched to session: {}", session_name);
                }
            }
            'n' => {
                // Create new session
                self.app.show_overlay = None;
                self.app.status_message =
                    "New session: type name and press Enter (not implemented yet)".to_string();
            }
            'd' => {
                // Delete current session (if not default)
                if let Some(current) = self.app.current_session.clone() {
                    if current != "default_session" {
                        self.app.session_list.retain(|s| s != &current);
                        self.app.current_session = Some("default_session".to_string());
                        self.app.show_overlay = None;
                        self.app.status_message = format!("Deleted session: {}", current);
                    } else {
                        self.app.status_message = "Cannot delete default session".to_string();
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle tools overlay key events
    fn handle_tools_overlay_key(&mut self, key: char) {
        match key {
            '1' => {
                self.app.show_overlay = None;
                let _ = self.switch_tui_mode("plan");
            }
            '2' => {
                self.app.show_overlay = None;
                let _ = self.switch_tui_mode("build");
            }
            '3' => {
                self.app.show_overlay = None;
                let _ = self.switch_tui_mode("run");
            }
            '4' => {
                self.app.show_overlay = None;
                let _ = self.switch_tui_mode("chat");
            }
            '5' => {
                self.app.show_overlay = None;
                let _ = self.switch_tui_mode("rag");
            }
            _ => {}
        }
    }

    /// Handle context overlay key events
    fn handle_context_overlay_key(&mut self, key: char) {
        match key {
            '1' => {
                self.app.show_overlay = None;
                self.app.status_message = "Showing current directory files".to_string();
            }
            '2' => {
                self.app.show_overlay = None;
                self.app.status_message = "Showing open files".to_string();
            }
            '3' => {
                self.app.show_overlay = None;
                self.app.status_message = "Showing git status".to_string();
            }
            '4' => {
                self.app.show_overlay = None;
                self.app.status_message = "Showing recent commands".to_string();
            }
            '5' => {
                self.app.show_overlay = None;
                self.app.status_message = "Showing project structure".to_string();
            }
            _ => {}
        }
    }

    /// Handle palette overlay key events
    fn handle_palette_overlay_key(&mut self, key: char) {
        match key {
            '1' => {
                self.app.show_overlay = None;
                self.app.status_message = "Quit command executed".to_string();
                // In a real implementation, this would trigger quit
            }
            '2' => {
                self.app.show_overlay = None;
                self.app.status_message =
                    "Help: i=insert, :q=quit, hjkl=navigate, Ctrl+P/N=history".to_string();
            }
            '3' => {
                self.app.show_overlay = None;
                self.app.status_message =
                    "Session switch: use :session <name>. Current: default_session ".to_string();
            }
            '4' => {
                self.app.show_overlay = None;
                self.app.input_buffer.clear();
                self.app.cursor_position = 0;
                self.app.status_message = "Buffer cleared".to_string();
            }
            '5' => {
                self.app.show_overlay = None;
                self.app.status_message = format!(
                    "Status: Mode={:?}, Session={:?}, History={} cmds",
                    self.app.current_mode,
                    self.app.current_session,
                    self.app.command_history.len()
                );
            }
            '6' => {
                self.app.show_overlay = None;
                self.app.status_message = "Session saved. Goodbye!".to_string();
                // In a real implementation, this would save and quit
            }
            '7' => {
                self.app.show_overlay = None;
                self.app.status_message =
                    "Mode switch: use :mode <plan|build|run|chat>. Current: normal ".to_string();
            }
            '8' => {
                self.app.show_overlay = None;
                let history_preview: Vec<String> = self
                    .app
                    .command_history
                    .iter()
                    .rev()
                    .take(5)
                    .map(|cmd| format!("  {}", cmd))
                    .collect();
                self.app.status_message =
                    format!("Recent history:\n{}", history_preview.join("\n"));
            }
            _ => {}
        }
    }

    /// Handle confirmation overlay key events
    fn handle_confirmation_overlay_key(&mut self, key: char) {
        if let Some(Overlay::Confirmation {
            default_yes,
            action,
            ..
        }) = &self.app.show_overlay.clone()
        {
            let result = match key {
                'y' | 'Y' => Some(true),
                'n' | 'N' => Some(false),
                '\n' | '\r' => Some(*default_yes), // Enter uses default
                _ => None,
            };

            if let Some(confirmed) = result {
                // Close the overlay
                self.app.show_overlay = None;

                if confirmed {
                    self.app.status_message = "Action confirmed - executing...".to_string();

                    // Execute the action based on type
                    match action {
                        PendingAction::Command(cmd) => {
                            // For commands, we'll let the normal flow handle it
                            self.app.input_buffer = cmd.clone();
                            // The command will be executed by the normal Enter key handling
                        }
                        PendingAction::FileRead(cmd) => {
                            // Show thinking overlay first
                            self.app.show_overlay = Some(Overlay::Thinking {
                                message: "Reading files...".to_string(),
                                step: 0,
                            });
                            // Store the command to be executed
                            self.app.input_buffer = cmd.clone();
                        }
                        PendingAction::FileEdit(cmd) => {
                            self.app.show_overlay = Some(Overlay::Thinking {
                                message: "Editing files...".to_string(),
                                step: 0,
                            });
                            self.app.input_buffer = cmd.clone();
                        }
                        PendingAction::FileSearch(cmd) => {
                            self.app.show_overlay = Some(Overlay::Thinking {
                                message: "Searching files...".to_string(),
                                step: 0,
                            });
                            self.app.input_buffer = cmd.clone();
                        }
                        PendingAction::MultiStep(cmd) => {
                            self.app.show_overlay = Some(Overlay::Thinking {
                                message: "Planning execution...".to_string(),
                                step: 0,
                            });
                            self.app.input_buffer = cmd.clone();
                        }
                        PendingAction::Unknown(cmd) => {
                            self.app.input_buffer = cmd.clone();
                        }
                    }
                } else {
                    self.app.status_message = "Action cancelled".to_string();
                }
            }
        }
    }

    /// Handle response overlay key events
    fn handle_response_overlay_key(&mut self, key: char) {
        if let Some(Overlay::Response {
            scroll_offset,
            content,
            ..
        }) = &mut self.app.show_overlay
        {
            let lines: Vec<&str> = content.lines().collect();
            let max_visible_lines = 20; // Approximate visible lines in overlay

            match key {
                'j' | 'J' => {
                    if *scroll_offset < lines.len().saturating_sub(max_visible_lines) {
                        *scroll_offset += 1;
                    }
                }
                'k' | 'K' => {
                    if *scroll_offset > 0 {
                        *scroll_offset = scroll_offset.saturating_sub(1);
                    }
                }
                'g' => *scroll_offset = 0,
                'G' => *scroll_offset = lines.len().saturating_sub(max_visible_lines),
                'q' | 'Q' | '\x1b' => {
                    // ESC or q to quit
                    self.app.show_overlay = None;
                    self.app.status_message = "Ready".to_string();
                }
                _ => {}
            }
        }
    }

    /// Handle insert mode key events
    async fn handle_insert_mode(&mut self, key: event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.app.current_mode = TuiMode::Normal;
                self.app.status_message = "Ready".to_string();
            }
            KeyCode::Enter => {
                // Execute the command
                let command = self.app.input_buffer.clone();
                self.execute_command(&command).await?;
                self.app.input_buffer.clear();
                self.app.cursor_position = 0;
                self.app.current_mode = TuiMode::Normal;
                self.app.status_message = "Ready".to_string();
            }
            KeyCode::Tab => {
                // Auto-completion (placeholder)
                self.app.status_message = "Tab completion not yet implemented".to_string();
            }
            KeyCode::Backspace => {
                if self.app.cursor_position > 0 {
                    self.app.input_buffer.remove(self.app.cursor_position - 1);
                    self.app.cursor_position -= 1;
                }
            }
            KeyCode::Delete => {
                if self.app.cursor_position < self.app.input_buffer.len() {
                    self.app.input_buffer.remove(self.app.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.app.cursor_position > 0 {
                    self.app.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.app.cursor_position < self.app.input_buffer.len() {
                    self.app.cursor_position += 1;
                }
            }
            KeyCode::Char(c) => {
                self.app.input_buffer.insert(self.app.cursor_position, c);
                self.app.cursor_position += 1;
                // Update status to show current input
                self.app.status_message = format!("Typing: {} chars", self.app.input_buffer.len());
            }
            _ => {
                // Debug: show what key was pressed
                self.app.status_message = format!("Unknown key: {:?}", key.code);
            }
        }
        Ok(false)
    }

    /// Handle command mode key events
    async fn handle_command_mode(&mut self, key: event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.app.current_mode = TuiMode::Normal;
                self.app.status_message = "Ready".to_string();
            }
            KeyCode::Enter => {
                let command = self.app.input_buffer.clone();
                if self.execute_vim_command(&command).await? {
                    return Ok(true); // Quit command
                }
                self.app.current_mode = TuiMode::Normal;
                self.app.status_message = "Ready".to_string();
            }
            KeyCode::Backspace => {
                if self.app.cursor_position > 0 {
                    self.app.input_buffer.remove(self.app.cursor_position - 1);
                    self.app.cursor_position -= 1;
                }
            }
            KeyCode::Char(c) => {
                self.app.input_buffer.insert(self.app.cursor_position, c);
                self.app.cursor_position += 1;
            }
            _ => {}
        }
        Ok(false)
    }

    /// Execute a command from the input buffer
    async fn execute_command(&mut self, command: &str) -> Result<()> {
        if !command.trim().is_empty() {
            // Add to history
            self.app.command_history.push(command.to_string());
            if self.app.command_history.len() > 100 {
                self.app.command_history.remove(0); // Keep only last 100 commands
            }
            self.app.history_index = None;
        }

        // Clear input buffer
        self.app.input_buffer.clear();
        self.app.cursor_position = 0;

        // Classify intent
        let intent = self.classify_intent(command).await?;

        let result = match intent.intent_type {
            IntentType::Question => {
                let response = self.handle_question(command).await;
                match response {
                    Ok(content) => {
                        // Show response in overlay
                        self.app.show_overlay = Some(Overlay::Response {
                            title: "AI Response".to_string(),
                            content,
                            scroll_offset: 0,
                        });
                        Ok::<String, anyhow::Error>("Response displayed in overlay".to_string())
                    }
                    Err(e) => Ok(format!("Error: {}", e)),
                }
            }
            IntentType::Command => {
                // Show confirmation overlay
                self.app.show_overlay = Some(Overlay::Confirmation {
                    message: format!("Execute command: {}", intent.description),
                    default_yes: true,
                    action: PendingAction::Command(command.to_string()),
                });
                Ok("Waiting for confirmation...".to_string())
            }
            IntentType::FileRead => {
                // Show confirmation overlay
                self.app.show_overlay = Some(Overlay::Confirmation {
                    message: format!("Read files: {}", intent.description),
                    default_yes: true,
                    action: PendingAction::FileRead(command.to_string()),
                });
                Ok("Waiting for confirmation...".to_string())
            }
            IntentType::FileEdit => {
                // Show confirmation overlay
                self.app.show_overlay = Some(Overlay::Confirmation {
                    message: format!("Edit files: {}", intent.description),
                    default_yes: false, // File edits need more caution
                    action: PendingAction::FileEdit(command.to_string()),
                });
                Ok("Waiting for confirmation...".to_string())
            }
            IntentType::FileSearch => {
                // Show confirmation overlay
                self.app.show_overlay = Some(Overlay::Confirmation {
                    message: format!("Search files: {}", intent.description),
                    default_yes: true,
                    action: PendingAction::FileSearch(command.to_string()),
                });
                Ok("Waiting for confirmation...".to_string())
            }
            IntentType::MultiStep => {
                // Show confirmation overlay
                self.app.show_overlay = Some(Overlay::Confirmation {
                    message: format!("Execute complex plan: {}", intent.description),
                    default_yes: false, // Complex operations need confirmation
                    action: PendingAction::MultiStep(command.to_string()),
                });
                Ok("Waiting for confirmation...".to_string())
            }
            IntentType::Unknown => {
                // Show confirmation overlay for unknown commands
                self.app.show_overlay = Some(Overlay::Confirmation {
                    message: format!("Execute unknown command: {}", command),
                    default_yes: false, // Unknown commands need caution
                    action: PendingAction::Unknown(command.to_string()),
                });
                Ok("Waiting for confirmation...".to_string())
            }
        };

        match result {
            Ok(output) => {
                self.app.status_message = format!("[OK] {}", output);
            }
            Err(e) => {
                self.app.status_message = format!("[ERR] Error: {}", e);
            }
        }

        Ok(())
    }

    /// Execute a shell command
    async fn execute_shell_command(&mut self, command: &str) -> Result<String> {
        use tokio::process::Command;

        let output = Command::new("sh").arg("-c").arg(command).output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!("Success: {}", stdout.trim()))
        } else {
            Err(anyhow::anyhow!("Command failed: {}", stderr.trim()))
        }
    }

    /// Execute a plan mode command
    async fn execute_plan_mode(&mut self, goal: &str) -> Result<String> {
        // Create a temporary CLI instance for plan mode
        let mut cli = Cli {
            plan: true,
            args: vec![goal.to_string()],
            ..Default::default()
        };

        // Set session if active
        if let Some(session) = &self.app.current_session {
            cli.session = Some(session.clone());
        }

        // Call CliApp's plan handler
        match self.app.cli_app.handle_plan_mode(goal).await {
            Ok(_) => Ok(format!("Plan created for: '{}'", goal)),
            Err(e) => Err(anyhow::anyhow!("Plan mode failed: {}", e)),
        }
    }

    /// Execute a build mode command
    async fn execute_build_mode(&mut self, goal: &str) -> Result<String> {
        // Create a temporary CLI instance for build mode
        let mut cli = Cli {
            build: true,
            args: vec![goal.to_string()],
            ..Default::default()
        };

        // Set session if active
        if let Some(session) = &self.app.current_session {
            cli.session = Some(session.clone());
        }

        // Call CliApp's build handler
        match self
            .app
            .cli_app
            .handle_build(goal, false, false, false)
            .await
        {
            Ok(_) => Ok(format!("Build completed for: '{}'", goal)),
            Err(e) => Err(anyhow::anyhow!("Build mode failed: {}", e)),
        }
    }

    /// Execute a run mode command
    async fn execute_run_mode(&mut self, goal: &str) -> Result<String> {
        // Create a temporary CLI instance for run mode
        let mut cli = Cli {
            run: true,
            args: vec![goal.to_string()],
            ..Default::default()
        };

        // Set session if active
        if let Some(session) = &self.app.current_session {
            cli.session = Some(session.clone());
        }

        // Call CliApp's agent handler (run mode)
        match self.app.cli_app.handle_agent(goal).await {
            Ok(_) => Ok(format!("Run completed for: '{}'", goal)),
            Err(e) => Err(anyhow::anyhow!("Run mode failed: {}", e)),
        }
    }

    /// Execute a chat mode command
    async fn execute_chat_mode(&mut self, message: &str) -> Result<String> {
        // For now, just indicate chat mode - full chat integration would need more work
        Ok(format!("Chat mode: '{}'", message))
    }

    /// Execute a RAG mode command
    async fn execute_rag_mode(&mut self, query: &str) -> Result<String> {
        // Create a temporary CLI instance for RAG mode
        let mut cli = Cli {
            rag: true,
            args: vec![query.to_string()],
            ..Default::default()
        };

        // Set session if active
        if let Some(session) = &self.app.current_session {
            cli.session = Some(session.clone());
        }

        // Call CliApp's RAG handler
        match self.app.cli_app.handle_rag(query, false).await {
            Ok(_) => Ok(format!("RAG query completed: '{}'", query)),
            Err(e) => Err(anyhow::anyhow!("RAG mode failed: {}", e)),
        }
    }

    /// Switch TUI mode
    fn switch_tui_mode(&mut self, mode: &str) -> Result<String> {
        match mode {
            "plan" => {
                self.app.tui_mode = Some("plan".to_string());
                self.app.status_message = "Switched to PLAN mode".to_string();
                Ok("Switched to PLAN mode".to_string())
            }
            "build" => {
                self.app.tui_mode = Some("build".to_string());
                self.app.status_message = "Switched to BUILD mode".to_string();
                Ok("Switched to BUILD mode".to_string())
            }
            "run" => {
                self.app.tui_mode = Some("run".to_string());
                self.app.status_message = "Switched to RUN mode".to_string();
                Ok("Switched to RUN mode".to_string())
            }
            "chat" => {
                self.app.tui_mode = Some("chat".to_string());
                self.app.status_message = "Switched to CHAT mode".to_string();
                Ok("Switched to CHAT mode".to_string())
            }
            "rag" => {
                self.app.tui_mode = Some("rag".to_string());
                self.app.status_message = "Switched to RAG mode".to_string();
                Ok("Switched to RAG mode".to_string())
            }
            _ => {
                self.app.tui_mode = None;
                self.app.status_message =
                    format!("Unknown mode: {}. Switched to normal mode.", mode);
                Ok(format!("Unknown mode: {}. Switched to normal mode.", mode))
            }
        }
    }

    /// Handle question intent - answer directly without commands
    async fn handle_question(&mut self, question: &str) -> Result<String> {
        // For questions, we need to generate an answer using AI
        let client = reqwest::Client::new();

        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "/home/user".to_string());

        let msgs = vec![domain::session::Message {
            role: "system".into(),
            content: "You are a helpful AI assistant. Answer questions concisely and directly. Do not generate commands.".into(),
        }, domain::session::Message {
            role: "user".into(),
            content: format!("Current directory: {}. Question: {}", cwd, question),
        }];

        let req = ChatRequest {
            model: &self.app.config.ollama_model,
            messages: &msgs,
            stream: false,
        };

        let resp = client
            .post(&self.app.config.ollama_base_url)
            .json(&req)
            .send()
            .await?;

        let raw = resp.text().await?;

        // Handle streaming response (NDJSON)
        let lines: Vec<&str> = raw.lines().collect();
        for line in lines.into_iter().rev() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<ChatResponse>(line) {
                if v.message.role == "assistant" {
                    return Ok(v.message.content.trim().to_string());
                }
            }
        }

        // JSON parse first (non-streaming)
        if let Ok(v) = serde_json::from_str::<ChatResponse>(&raw) {
            return Ok(v.message.content.trim().to_string());
        }

        Ok("Could not generate answer".to_string())
    }

    /// Handle file read intent
    async fn handle_file_read(&mut self, request: &str) -> Result<String> {
        // Use AI to determine what files to read
        let client = reqwest::Client::new();

        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "/home/user".to_string());

        let msgs = vec![domain::session::Message {
            role: "user".into(),
            content: format!(
                "Based on this request: '{}'\nCurrent directory: {}\n\nSuggest specific files to read. Return only a JSON array of file paths, no other text.",
                request, cwd
            ),
        }];

        let req = ChatRequest {
            model: &self.app.config.ollama_model,
            messages: &msgs,
            stream: false,
        };

        let resp = client
            .post(&self.app.config.ollama_base_url)
            .json(&req)
            .send()
            .await?;

        let raw = resp.text().await?;

        // Parse JSON array of file paths
        if let Ok(file_paths) = serde_json::from_str::<Vec<String>>(&raw) {
            let mut results = Vec::new();
            for path in file_paths {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        results.push(format!("=== {} ===\n{}", path, content));
                    }
                    Err(e) => {
                        results.push(format!("Error reading {}: {}", path, e));
                    }
                }
            }
            Ok(results.join("\n\n"))
        } else {
            Ok(format!(
                "Could not determine files to read for: {}",
                request
            ))
        }
    }

    /// Handle file edit intent
    async fn handle_file_edit(&mut self, request: &str) -> Result<String> {
        // Use AI to determine what file edits to make
        let client = reqwest::Client::new();

        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "/home/user".to_string());

        let msgs = vec![domain::session::Message {
            role: "user".into(),
            content: format!(
                "Based on this edit request: '{}'\nCurrent directory: {}\n\nGenerate a JSON object with file edits. Format:\n{{
  \"edits\": [
    {{
      \"file_path\": \"path/to/file\",
      \"old_string\": \"exact text to replace\",
      \"new_string\": \"replacement text\"
    }}
  ]
}}\n\nOnly output valid JSON, no explanation.",
                request, cwd
            ),
        }];

        let req = ChatRequest {
            model: &self.app.config.ollama_model,
            messages: &msgs,
            stream: false,
        };

        let resp = client
            .post(&self.app.config.ollama_base_url)
            .json(&req)
            .send()
            .await?;

        let raw = resp.text().await?;

        // Parse JSON response
        #[derive(Deserialize)]
        struct EditSpec {
            file_path: String,
            old_string: String,
            new_string: String,
        }

        #[derive(Deserialize)]
        struct EditPlan {
            edits: Vec<EditSpec>,
        }

        if let Ok(plan) = serde_json::from_str::<EditPlan>(&raw) {
            let mut results = Vec::new();

            for edit in plan.edits {
                // Read the file first
                match std::fs::read_to_string(&edit.file_path) {
                    Ok(content) => {
                        if content.contains(&edit.old_string) {
                            // Apply the edit
                            let new_content = content.replace(&edit.old_string, &edit.new_string);
                            match std::fs::write(&edit.file_path, &new_content) {
                                Ok(_) => {
                                    results.push(format!(
                                        "Edited {}: replaced '{}' with '{}'",
                                        edit.file_path, edit.old_string, edit.new_string
                                    ));
                                }
                                Err(e) => {
                                    results
                                        .push(format!("Failed to write {}: {}", edit.file_path, e));
                                }
                            }
                        } else {
                            results.push(format!(
                                "Could not find '{}' in {}",
                                edit.old_string, edit.file_path
                            ));
                        }
                    }
                    Err(e) => {
                        results.push(format!("Could not read {}: {}", edit.file_path, e));
                    }
                }
            }

            Ok(results.join("\n"))
        } else {
            Ok(format!("Could not parse edit plan for: {}", request))
        }
    }

    /// Handle file search intent
    async fn handle_file_search(&mut self, request: &str) -> Result<String> {
        // Use AI to determine search pattern
        let client = reqwest::Client::new();

        let msgs = vec![domain::session::Message {
            role: "user".into(),
            content: format!(
                "Convert this search request to a grep command: '{}'\nReturn only the grep command, no explanation.",
                request
            ),
        }];

        let req = ChatRequest {
            model: &self.app.config.ollama_model,
            messages: &msgs,
            stream: false,
        };

        let resp = client
            .post(&self.app.config.ollama_base_url)
            .json(&req)
            .send()
            .await?;

        let raw = resp.text().await?;

        // Extract command and execute
        let command = raw.trim().trim_matches('"');
        if command.starts_with("grep") {
            self.execute_shell_command(command).await
        } else {
            Ok(format!(
                "Could not generate search command for: {}",
                request
            ))
        }
    }

    /// Classify user intent from prompt
    async fn classify_intent(&self, prompt: &str) -> Result<Intent> {
        let client = reqwest::Client::new();

        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "/home/user".to_string());

        let system = r#"You are an AI assistant that classifies user prompts into specific intent categories.

Analyze the user's prompt and respond with a JSON object containing:
- intent_type: One of "Question", "Command", "FileRead", "FileEdit", "FileSearch", "MultiStep", "Unknown"
- description: Brief description of what the user wants
- confidence: Number between 0.0 and 1.0 indicating confidence
- details: Optional additional details (can be null)

Intent categories:
- Question: User is asking for information, explanation, or advice (answer directly)
- Command: User wants to execute a single command or operation
- FileRead: User wants to read/view file contents
- FileEdit: User wants to modify or create files
- FileSearch: User wants to search for files or content within files
- MultiStep: Complex task requiring multiple steps or commands
- Unknown: Cannot determine intent

Examples:
"what is the capital of France?" -> Question
"list files in current directory" -> Command
"show me the contents of main.rs" -> FileRead
"add error handling to this function" -> FileEdit
"find all TODO comments" -> FileSearch
"set up a new React project" -> MultiStep

Output ONLY valid JSON, no other text."#;

        let msgs = vec![
            domain::session::Message {
                role: "system".into(),
                content: system.into(),
            },
            domain::session::Message {
                role: "user".into(),
                content: format!("Current directory: {}\n\nPrompt: {}", cwd, prompt),
            },
        ];

        let req = ChatRequest {
            model: &self.app.config.ollama_model,
            messages: &msgs,
            stream: false,
        };

        let resp = client
            .post(&self.app.config.ollama_base_url)
            .json(&req)
            .send()
            .await?;

        let raw = resp.text().await?;

        // Try to parse JSON directly
        if let Ok(intent) = serde_json::from_str::<Intent>(&raw) {
            return Ok(intent);
        }

        // Try to extract JSON from response
        if let Some(json) = raw.find('{').and_then(|start| {
            raw[start..]
                .find('}')
                .map(|end| &raw[start..start + end + 1])
        }) {
            if let Ok(intent) = serde_json::from_str::<Intent>(json) {
                return Ok(intent);
            }
        }

        // Fallback to Unknown intent
        Ok(Intent {
            intent_type: IntentType::Unknown,
            description: "Could not classify intent".to_string(),
            confidence: 0.0,
            details: Some(raw),
        })
    }

    /// Execute a vision mode command using ChatGPT browser automation
    async fn execute_vision_command(&mut self, goal: &str) -> Result<String> {
        use infrastructure::chatgpt_browser::ChatGPTBrowser;

        // Initialize browser automation
        let browser = match ChatGPTBrowser::new() {
            Ok(browser) => {
                // Ensure Docker image is available
                if let Err(e) = browser.ensure_docker_image() {
                    return Ok(format!("Vision mode setup failed: {}. Install Docker and run: docker pull mcr.microsoft.com/playwright:v1.40.0-jammy", e));
                }
                browser
            }
            Err(e) => {
                return Ok(format!(
                    "Vision mode not available: {}. Install Docker for cross-platform support.",
                    e
                ))
            }
        };

        // Check if ChatGPT is accessible
        match browser.is_chatgpt_available() {
            Ok(true) => {
                // Proceed with structured query
                let mut browser = browser; // Make mutable for context gathering
                match browser.query_with_context(goal).await {
                    Ok(result) if result.success => {
                        Ok(format!("🤖 AI Response: {}", result.response))
                    }
                    Ok(result) => {
                        Ok(format!("❌ Vision query failed: {}", result.error_message.unwrap_or("Unknown error".to_string())))
                    }
                    Err(e) => Ok(format!("❌ Vision query error: {}. Make sure you're logged into ChatGPT in a browser.", e)),
                }
            }
            _ => Ok("❌ ChatGPT not accessible. Please ensure you're logged into https://chat.openai.com in a browser.".to_string()),
        }
    }

    /// Execute a vim-style command
    async fn execute_vim_command(&mut self, command: &str) -> Result<bool> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        let cmd = parts.get(0).unwrap_or(&"");

        match *cmd {
            "q" | "quit" => {
                self.app.status_message = "Goodbye!".to_string();
                return Ok(true);
            }
            "q!" => {
                self.app.status_message = "Force quit!".to_string();
                return Ok(true);
            }
            "w" | "write" => {
                self.app.status_message = "Session saved".to_string();
            }
            "wq" => {
                self.app.status_message = "Session saved. Goodbye!".to_string();
                return Ok(true);
            }
            "x" => {
                self.app.status_message = "Session saved. Goodbye!".to_string();
                return Ok(true);
            }
            "h" | "help" => {
                self.app.status_message =
                    "Help: i=insert, :q=quit, :w=save, hjkl=navigate".to_string();
            }
            "session" => {
                if let Some(name) = parts.get(1) {
                    self.app.current_session = Some(name.to_string());
                    self.app.status_message = format!("Switched to session: {}", name);
                } else {
                    self.app.status_message =
                        format!("Usage: :session <name>. Current: {}", "default_session");
                }
            }
            "mode" => {
                if let Some(mode) = parts.get(1) {
                    match *mode {
                        "plan" => self.app.status_message = "Switched to PLAN mode ".to_string(),
                        "build" => self.app.status_message = "Switched to BUILD mode ".to_string(),
                        "run" => self.app.status_message = "Switched to RUN mode ".to_string(),
                        "chat" => self.app.status_message = "Switched to CHAT mode ".to_string(),
                        _ => self.app.status_message = format!("Unknown mode: {}", mode),
                    }
                } else {
                    self.app.status_message =
                        format!("Usage: :mode <plan|build|run|chat>. Current: {}", "normal");
                }
            }
            "clear" => {
                self.app.input_buffer.clear();
                self.app.cursor_position = 0;
                self.app.status_message = format!("Buffer {}", "cleared");
            }
            "status" => {
                self.app.status_message = format!(
                    "Mode: {:?}, Session: {:?}, Buffer: {} {}",
                    self.app.current_mode,
                    self.app.current_session,
                    self.app.input_buffer.len(),
                    "chars"
                );
            }
            _ => {
                self.app.status_message = format!(
                    "Unknown command: {}. Type :help for {}",
                    command, "commands"
                );
            }
        }
        Ok(false)
    }

    /// Draw the TUI interface
    fn draw_ui(f: &mut Frame, app: &TuiApp) {
        let size = f.size();

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(size);

        // Draw header
        Self::draw_header(f, chunks[0], app);
        Self::draw_main_content(f, chunks[1], app);
        Self::draw_status_bar(f, chunks[2], app);

        // Draw overlay if present
        if let Some(overlay) = &app.show_overlay {
            Self::draw_overlay(f, overlay.clone(), app);
        }
    }

    /// Draw the header section
    fn draw_header(f: &mut Frame, area: Rect, app: &TuiApp) {
        let header_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title row
                Constraint::Length(1), // Status row
                Constraint::Length(1), // Spacer
            ])
            .split(area);

        let title_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
            ])
            .split(header_chunks[0]);

        let status_row = header_chunks[1];

        // Title row
        let title = Paragraph::new("bro v0.1.0 - Agentic AI Assistant")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Left);
        f.render_widget(title, title_row[0]);

        // Session and Agent Status
        let session = app.current_session.as_deref().unwrap_or("no session");
        let agent_phase = match app.agent_status.phase {
            AgentPhase::Idle => "",
            AgentPhase::ClassifyingIntent => "[CLASSIFYING]",
            AgentPhase::Planning => "[PLANNING]",
            AgentPhase::AwaitingApproval => "[AWAITING APPROVAL]",
            AgentPhase::Executing {
                current_step,
                total_steps,
            } => &format!("[EXECUTING {}/{}]", current_step, total_steps),
            AgentPhase::Complete => "[COMPLETE]",
            AgentPhase::Error => "[ERROR]",
        };
        let session_info = Paragraph::new(format!("[SESSION: {}] {}", session, agent_phase))
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(session_info, title_row[1]);

        // Mode indicator
        let mode_text = match app.current_mode {
            TuiMode::Normal => {
                if let Some(tui_mode) = &app.tui_mode {
                    format!("NORMAL ({})", tui_mode.to_uppercase())
                } else {
                    "NORMAL".to_string()
                }
            }
            TuiMode::Insert => {
                if let Some(tui_mode) = &app.tui_mode {
                    format!("INSERT ({})", tui_mode.to_uppercase())
                } else {
                    "INSERT".to_string()
                }
            }
            TuiMode::Command => {
                if let Some(tui_mode) = &app.tui_mode {
                    format!("COMMAND ({})", tui_mode.to_uppercase())
                } else {
                    "COMMAND".to_string()
                }
            }
        };
        let mode = Paragraph::new(format!("[{}]", mode_text))
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Right);
        f.render_widget(mode, title_row[2]);

        // Status message row
        let status_message = match app.agent_status.phase {
            AgentPhase::Idle => {
                if app.current_mode == TuiMode::Insert {
                    "Type your goal, press Enter to submit, Esc for normal mode".to_string()
                } else {
                    "Ready - Type 'i' to input goal, ':' for commands".to_string()
                }
            }
            AgentPhase::ClassifyingIntent => {
                "AI is analyzing your goal and classifying intent...".to_string()
            }
            AgentPhase::Planning => "AI is creating an execution plan...".to_string(),
            AgentPhase::AwaitingApproval => {
                "Plan ready - Press 'y' to execute, 'e' to edit, 'q' to cancel".to_string()
            }
            AgentPhase::Executing {
                current_step,
                total_steps,
            } => {
                format!(
                    "Executing step {}/{} - Press 'p' for pause, 'q' for abort",
                    current_step, total_steps
                )
            }
            AgentPhase::Complete => {
                "Execution complete - Press 'r' to review, 'n' for new task".to_string()
            }
            AgentPhase::Error => {
                format!(
                    "Error occurred - {}",
                    app.agent_status
                        .error_message
                        .as_deref()
                        .unwrap_or("Unknown error")
                )
            }
        };
        let status = Paragraph::new(status_message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left);
        f.render_widget(status, status_row);
    }

    /// Draw the main content area
    fn draw_main_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        match app.agent_status.phase {
            AgentPhase::Idle => Self::draw_idle_content(f, area, app),
            AgentPhase::ClassifyingIntent => Self::draw_classifying_content(f, area, app),
            AgentPhase::Planning => Self::draw_planning_content(f, area, app),
            AgentPhase::AwaitingApproval => Self::draw_approval_content(f, area, app),
            AgentPhase::Executing { .. } => Self::draw_execution_content(f, area, app),
            AgentPhase::Complete => Self::draw_complete_content(f, area, app),
            AgentPhase::Error => Self::draw_error_content(f, area, app),
        }
    }

    /// Draw idle state (ready for input)
    fn draw_idle_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Input area
                Constraint::Length(3), // History area
            ])
            .split(area);

        // Input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Goal Input ({})", app.input_buffer.len()));

        let input_text = if app.input_buffer.is_empty() {
            "Type your goal here (e.g., 'Refactor auth middleware to use dependency injection')..."
        } else {
            &app.input_buffer
        };
        let input_paragraph = Paragraph::new(input_text).block(input_block.clone());
        f.render_widget(input_paragraph, content_chunks[0]);

        // Cursor rendering for input
        if app.current_mode == TuiMode::Insert {
            let input_area = content_chunks[0];
            let inner_area = input_block.inner(input_area);
            let cursor_x = inner_area.x + app.cursor_position as u16;
            let cursor_y = inner_area.y;
            f.set_cursor(cursor_x, cursor_y);
        }

        // History area (show last 3 commands)
        let history_block = Block::default().borders(Borders::ALL).title("Recent Goals");
        let history_text = app
            .command_history
            .iter()
            .rev()
            .take(3)
            .map(|cmd| Line::from(format!("  {}", cmd)))
            .collect::<Vec<_>>();
        let history_paragraph = Paragraph::new(history_text).block(history_block);
        f.render_widget(history_paragraph, content_chunks[1]);
    }

    /// Draw intent classification state
    fn draw_classifying_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("🤖 AI Intent Classification");

        let content = vec![
            Line::from(""),
            Line::from(format!(
                "Goal: {}",
                app.agent_status
                    .current_goal
                    .as_deref()
                    .unwrap_or("Unknown")
            )),
            Line::from(""),
            Line::from("🔄 Analyzing intent and determining action type..."),
            Line::from(""),
            Line::from("This may take a few seconds..."),
        ];

        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    /// Draw planning state
    fn draw_planning_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("📋 AI Planning Phase");

        let content = vec![
            Line::from(""),
            Line::from(format!(
                "Goal: {}",
                app.agent_status
                    .current_goal
                    .as_deref()
                    .unwrap_or("Unknown")
            )),
            Line::from(""),
            Line::from("🔄 AI is creating a detailed execution plan..."),
            Line::from("   • Analyzing project structure"),
            Line::from("   • Determining required tools"),
            Line::from("   • Assessing risks and dependencies"),
            Line::from("   • Generating step-by-step actions"),
            Line::from(""),
            Line::from("This may take 10-30 seconds..."),
        ];

        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    /// Draw approval waiting state
    fn draw_approval_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Plan summary
                Constraint::Min(1),    // Plan details
                Constraint::Length(3), // Action hints
            ])
            .split(area);

        // Plan summary
        let summary_block = Block::default()
            .borders(Borders::ALL)
            .title("📋 Execution Plan Ready");

        let plan_summary = if let Some(plan) = &app.current_plan {
            format!(
                "Plan: {} steps, ~{} min, Risk: Medium",
                plan.tool_calls.len(),
                plan.tool_calls.len() * 2
            ) // Rough estimate
        } else {
            "Plan details loading...".to_string()
        };

        let summary_content = vec![
            Line::from(format!(
                "Goal: {}",
                app.agent_status
                    .current_goal
                    .as_deref()
                    .unwrap_or("Unknown")
            )),
            Line::from(plan_summary),
            Line::from(""),
            Line::from("Ready for execution - awaiting your approval"),
        ];

        let summary_paragraph = Paragraph::new(summary_content).block(summary_block);
        f.render_widget(summary_paragraph, content_chunks[0]);

        // Plan details (placeholder for now)
        let details_block = Block::default().borders(Borders::ALL).title("Plan Steps");

        let details_content = vec![
            Line::from("Step 1: [FileRead] Analyze current structure"),
            Line::from("Step 2: [FileEdit] Make required changes"),
            Line::from("Step 3: [Command] Run tests"),
            Line::from("... (more steps available)"),
        ];

        let details_paragraph = Paragraph::new(details_content).block(details_block);
        f.render_widget(details_paragraph, content_chunks[1]);

        // Action hints
        let hints_block = Block::default().borders(Borders::ALL);
        let hints_content = vec![Line::from(
            "Actions: [y] Execute  [e] Edit Plan  [q] Cancel  [d] Show Details",
        )];
        let hints_paragraph = Paragraph::new(hints_content).block(hints_block);
        f.render_widget(hints_paragraph, content_chunks[2]);
    }

    /// Draw execution state
    fn draw_execution_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Current step
                Constraint::Min(1),    // Progress/details
                Constraint::Length(2), // Controls
            ])
            .split(area);

        // Current step
        let step_block = Block::default()
            .borders(Borders::ALL)
            .title("⚡ Current Execution");

        let (current_step, total_steps) = match app.agent_status.phase {
            AgentPhase::Executing {
                current_step,
                total_steps,
            } => (current_step, total_steps),
            _ => (0, 0),
        };

        let step_content = vec![
            Line::from(format!("Step {}/{}", current_step, total_steps)),
            Line::from("AI is executing the current step..."),
            Line::from(""),
            Line::from("This may take several seconds..."),
        ];

        let step_paragraph = Paragraph::new(step_content).block(step_block);
        f.render_widget(step_paragraph, content_chunks[0]);

        // Progress details
        let progress_block = Block::default().borders(Borders::ALL).title("Progress Log");

        let progress_content: Vec<Line> = app
            .execution_progress
            .iter()
            .rev()
            .take(10)
            .map(|line| Line::from(format!("  {}", line)))
            .collect();

        let progress_paragraph = Paragraph::new(progress_content).block(progress_block);
        f.render_widget(progress_paragraph, content_chunks[1]);

        // Controls
        let controls_block = Block::default().borders(Borders::ALL);
        let controls_content = vec![Line::from("Controls: [p] Pause  [q] Abort  [s] Status")];
        let controls_paragraph = Paragraph::new(controls_content).block(controls_block);
        f.render_widget(controls_paragraph, content_chunks[2]);
    }

    /// Draw completion state
    fn draw_complete_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Results summary
                Constraint::Min(1),    // Details
                Constraint::Length(3), // Next actions
            ])
            .split(area);

        // Results summary
        let results_block = Block::default()
            .borders(Borders::ALL)
            .title("✅ Execution Complete");

        let results_content = vec![
            Line::from("SUCCESS: All steps completed successfully"),
            Line::from(""),
            Line::from(format!(
                "Goal: {}",
                app.agent_status
                    .current_goal
                    .as_deref()
                    .unwrap_or("Unknown")
            )),
            Line::from(format!(
                "Duration: {:?}",
                app.agent_status.execution_time.unwrap_or_default()
            )),
            Line::from(format!("Tools Used: {}", app.agent_status.tools_used.len())),
        ];

        let results_paragraph = Paragraph::new(results_content)
            .block(results_block)
            .style(Style::default().fg(Color::Green));
        f.render_widget(results_paragraph, content_chunks[0]);

        // Details (placeholder)
        let details_block = Block::default()
            .borders(Borders::ALL)
            .title("Completion Details");

        let details_content = vec![
            Line::from("• Files modified: 2"),
            Line::from("• Tests passed: 23/23"),
            Line::from("• Git commit created"),
            Line::from("• Documentation updated"),
        ];

        let details_paragraph = Paragraph::new(details_content).block(details_block);
        f.render_widget(details_paragraph, content_chunks[1]);

        // Next actions
        let actions_block = Block::default().borders(Borders::ALL).title("Next Actions");

        let actions_content = vec![Line::from(
            "[r] Review changes  [n] New task  [s] Save session  [q] Quit",
        )];

        let actions_paragraph = Paragraph::new(actions_content).block(actions_block);
        f.render_widget(actions_paragraph, content_chunks[2]);
    }

    /// Draw error state
    fn draw_error_content(f: &mut Frame, area: Rect, app: &TuiApp) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("❌ Execution Error")
            .style(Style::default().fg(Color::Red));

        let error_message = app
            .agent_status
            .error_message
            .as_deref()
            .unwrap_or("An unknown error occurred during execution");

        let content = vec![
            Line::from(""),
            Line::from("An error occurred during execution:"),
            Line::from(""),
            Line::from(error_message),
            Line::from(""),
            Line::from("Actions: [r] Retry  [e] Edit goal  [q] Quit"),
        ];

        let paragraph = Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    /// Draw the status bar with agent metrics
    fn draw_status_bar(f: &mut Frame, area: Rect, app: &TuiApp) {
        let status_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Agent status
                Constraint::Percentage(20), // Performance metrics
                Constraint::Percentage(20), // Resource usage
                Constraint::Percentage(20), // Tools info
                Constraint::Percentage(15), // Actions
            ])
            .split(area);

        // Agent status
        let agent_status = match app.agent_status.phase {
            AgentPhase::Idle => "Agent: Idle".to_string(),
            AgentPhase::ClassifyingIntent => "Agent: Classifying".to_string(),
            AgentPhase::Planning => "Agent: Planning".to_string(),
            AgentPhase::AwaitingApproval => "Agent: Ready".to_string(),
            AgentPhase::Executing {
                current_step,
                total_steps,
            } => format!("Agent: Exec {}/{}", current_step, total_steps),
            AgentPhase::Complete => "Agent: Complete".to_string(),
            AgentPhase::Error => "Agent: Error".to_string(),
        };
        let agent_widget = Paragraph::new(agent_status)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left);
        f.render_widget(agent_widget, status_chunks[0]);

        // Performance metrics
        let confidence = app
            .agent_status
            .confidence
            .map(|c| format!("Conf: {:.0}%", c * 100.0))
            .unwrap_or_else(|| "Conf: N/A".to_string());
        let duration = app
            .agent_status
            .execution_time
            .map(|d| format!("Time: {:.1}s", d.as_secs_f32()))
            .unwrap_or_else(|| "Time: N/A".to_string());
        let perf_text = format!("{} | {}", confidence, duration);
        let perf_widget = Paragraph::new(perf_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(perf_widget, status_chunks[1]);

        // Resource usage
        let memory = app
            .agent_status
            .memory_usage
            .map(|m| format!("Mem: {}MB", m / 1024 / 1024))
            .unwrap_or_else(|| "Mem: N/A".to_string());
        let tools_count = format!("Tools: {}", app.agent_status.tools_used.len());
        let resource_text = format!("{} | {}", memory, tools_count);
        let resource_widget = Paragraph::new(resource_text)
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center);
        f.render_widget(resource_widget, status_chunks[2]);

        // Tools info (available tools count)
        let tools_text = "Tools: 12 avail";
        let tools_widget = Paragraph::new(tools_text)
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        f.render_widget(tools_widget, status_chunks[3]);

        // Action hints (contextual)
        let actions_text = match app.agent_status.phase {
            AgentPhase::Idle => "Actions: i=goal, :=cmd",
            AgentPhase::AwaitingApproval => "Actions: y=exec, e=edit, q=cancel",
            AgentPhase::Executing { .. } => "Actions: p=pause, q=abort",
            AgentPhase::Complete => "Actions: r=review, n=new",
            AgentPhase::Error => "Actions: r=retry, e=edit",
            _ => "Actions: ...",
        };
        let actions_widget = Paragraph::new(actions_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Right);
        f.render_widget(actions_widget, status_chunks[4]);
    }

    /// Draw overlay windows
    fn draw_overlay(f: &mut Frame, overlay: Overlay, app: &TuiApp) {
        let area = Self::centered_rect(60, 40, f.size());
        f.render_widget(Clear, area);

        match overlay {
            Overlay::Sessions => Self::draw_sessions_overlay(f, area, app),
            Overlay::Tools => Self::draw_tools_overlay(f, area, app),
            Overlay::Context => Self::draw_context_overlay(f, area, app),
            Overlay::Palette => Self::draw_palette_overlay(f, area, app),
            Overlay::Confirmation {
                message,
                default_yes,
                ..
            } => Self::draw_confirmation_overlay(f, area, &message, default_yes),
            Overlay::Response {
                title,
                content,
                scroll_offset,
            } => Self::draw_response_overlay(f, area, &title, &content, scroll_offset),
            Overlay::Thinking { message, step } => {
                Self::draw_thinking_overlay(f, area, &message, step)
            }
        }
    }

    /// Draw sessions overlay
    fn draw_sessions_overlay(f: &mut Frame, area: Rect, app: &TuiApp) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with actions
                Constraint::Min(1),    // Session list
                Constraint::Length(2), // Footer hints
            ])
            .split(area);

        // Header with session actions
        let header_block = Block::default()
            .title("Session Manager")
            .borders(Borders::ALL);

        let header_text = vec![Line::from(vec![
            Span::styled("Actions: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                "n",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("ew session, ", Style::default().fg(Color::Gray)),
            Span::styled(
                "d",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("elete, ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" switch", Style::default().fg(Color::Gray)),
        ])];

        let header = Paragraph::new(header_text)
            .block(header_block)
            .wrap(Wrap { trim: true });
        f.render_widget(header, chunks[0]);

        // Session list
        let list_block = Block::default().borders(Borders::ALL);

        let items: Vec<ListItem> = app
            .session_list
            .iter()
            .enumerate()
            .map(|(_i, session)| {
                let mut style = Style::default();
                let mut prefix = "  ";

                if Some(session) == app.current_session.as_ref() {
                    style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
                    prefix = "● ";
                }

                // Show session metadata (placeholder for now)
                let display_name = if session == "default_session" {
                    format!("{}{} (default)", prefix, session)
                } else {
                    format!("{}{}", prefix, session)
                };

                ListItem::new(display_name).style(style)
            })
            .collect();

        let list = List::new(items).block(list_block);
        f.render_widget(list, chunks[1]);

        // Footer hints
        let footer_text = vec![Line::from(vec![
            Span::styled(
                "Use number keys to select, ",
                Style::default().fg(Color::Gray),
            ),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close", Style::default().fg(Color::Gray)),
        ])];

        let footer = Paragraph::new(footer_text).alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }

    /// Draw tools overlay
    fn draw_tools_overlay(f: &mut Frame, area: Rect, _app: &TuiApp) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(1),    // Tools list
                Constraint::Length(2), // Footer
            ])
            .split(area);

        // Header
        let header_block = Block::default()
            .title("Available Tools")
            .borders(Borders::ALL);

        let header = Paragraph::new("Choose a tool to switch modes:")
            .block(header_block)
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Tools list
        let list_block = Block::default().borders(Borders::ALL);

        let tools = vec![
            (
                "1",
                "Plan Mode",
                "Create execution plans without running commands",
            ),
            (
                "2",
                "Build Mode",
                "Safe code modifications with AI assistance",
            ),
            ("3", "Run Mode", "Execute multi-step command sequences"),
            ("4", "Chat Mode", "Interactive conversation with AI"),
            ("5", "RAG Mode", "Query codebase with context retrieval"),
        ];

        let items: Vec<ListItem> = tools
            .iter()
            .map(|(num, name, desc)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} {}", num, name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" - ", Style::default().fg(Color::Gray)),
                    Span::styled(*desc, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items).block(list_block);
        f.render_widget(list, chunks[1]);

        // Footer
        let footer = Paragraph::new("Press number key to select, Esc to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(footer, chunks[2]);
    }

    /// Draw context overlay
    fn draw_context_overlay(f: &mut Frame, area: Rect, _app: &TuiApp) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(1),    // Context list
                Constraint::Length(2), // Footer
            ])
            .split(area);

        // Header
        let header_block = Block::default()
            .title("Project Context")
            .borders(Borders::ALL);

        let header = Paragraph::new("Explore project state and history:")
            .block(header_block)
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Context list
        let list_block = Block::default().borders(Borders::ALL);

        let context_items = vec![
            ("1", "Directory Files", "List files in current directory"),
            ("2", "Git Status", "Show uncommitted changes"),
            ("3", "Recent Commands", "Command history for this session"),
            ("4", "Project Structure", "Show directory tree"),
            ("5", "Configuration", "Show current settings"),
        ];

        let items: Vec<ListItem> = context_items
            .iter()
            .map(|(num, name, desc)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} {}", num, name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" - ", Style::default().fg(Color::Gray)),
                    Span::styled(*desc, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items).block(list_block);
        f.render_widget(list, chunks[1]);

        // Footer
        let footer = Paragraph::new("Press number key to select, Esc to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(footer, chunks[2]);
    }

    /// Draw command palette overlay
    fn draw_palette_overlay(f: &mut Frame, area: Rect, _app: &TuiApp) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(1),    // Commands list
                Constraint::Length(2), // Footer
            ])
            .split(area);

        // Header
        let header_block = Block::default()
            .title("Command Palette")
            .borders(Borders::ALL);

        let header = Paragraph::new("Quick commands and actions:")
            .block(header_block)
            .alignment(Alignment::Center);
        f.render_widget(header, chunks[0]);

        // Commands list
        let list_block = Block::default().borders(Borders::ALL);

        let commands = vec![
            ("1", ":quit", "Exit the application"),
            ("2", ":help", "Show help and keybindings"),
            ("3", ":session <name>", "Switch to named session"),
            ("4", ":clear", "Clear command buffer"),
            ("5", ":status", "Show current status"),
            ("6", ":wq", "Save session and quit"),
            ("7", ":mode <type>", "Switch mode (plan/build/run/chat)"),
            ("8", ":history", "Show full command history"),
        ];

        let items: Vec<ListItem> = commands
            .iter()
            .map(|(num, cmd, desc)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} {}", num, cmd),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" - ", Style::default().fg(Color::Gray)),
                    Span::styled(*desc, Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items).block(list_block);
        f.render_widget(list, chunks[1]);

        // Footer
        let footer = Paragraph::new("Press number key to execute, Esc to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(footer, chunks[2]);
    }

    /// Draw response overlay with scrollable content
    fn draw_response_overlay(
        f: &mut Frame,
        area: Rect,
        title: &str,
        content: &str,
        scroll_offset: usize,
    ) {
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        let lines: Vec<Line> = content
            .lines()
            .skip(scroll_offset)
            .take(area.height.saturating_sub(4) as usize) // Account for borders and title
            .map(|line| Line::from(line))
            .collect();

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);

        // Show scroll indicators
        if scroll_offset > 0 {
            let up_indicator = Paragraph::new("↑")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(
                up_indicator,
                Rect::new(area.x + area.width - 1, area.y + 1, 1, 1),
            );
        }

        let total_lines = content.lines().count();
        let visible_lines = area.height.saturating_sub(4) as usize;
        if scroll_offset + visible_lines < total_lines {
            let down_indicator = Paragraph::new("↓")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(
                down_indicator,
                Rect::new(area.x + area.width - 1, area.y + area.height - 2, 1, 1),
            );
        }
    }

    /// Draw thinking/progress overlay
    fn draw_thinking_overlay(f: &mut Frame, area: Rect, message: &str, step: usize) {
        let block = Block::default()
            .title("AI Processing")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner = spinner_chars[step % spinner_chars.len()];

        let text = vec![
            Line::from(vec![
                Span::styled(spinner, Style::default().fg(Color::Cyan)),
                Span::styled(" ", Style::default()),
                Span::styled(message, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Please wait...",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }
    /// Draw confirmation overlay
    fn draw_confirmation_overlay(f: &mut Frame, area: Rect, message: &str, default_yes: bool) {
        let block = Block::default()
            .title("Confirm Action")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let default_text = format!(" for {}", if default_yes { "Yes" } else { "No" });

        let text = vec![
            Line::from(message),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Y",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" for Yes, ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "N",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" for No, or ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&default_text, Style::default().fg(Color::Gray)),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    /// Helper function to create a centered rectangle
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
