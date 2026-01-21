# Autonomous AI Implementation Plan

## **🎯 OVERVIEW**

This document outlines the complete plan to transform Bro into a truly autonomous, self-critiquing AI agent using the end-to-end agentic framework provided by the user.

### **Current State Analysis**

**✅ Strengths to Leverage:**
- `InferenceEngine` abstraction (supports Ollama)
- Comprehensive `SafeTool` registry with 12+ validated tools
- `AgentController` with execution limits and verification
- `ToolArgs` and `ToolOutput` structures already implemented
- Existing safety/security infrastructure

**❌ Gaps to Address:**
- No routing/classification system
- Missing structured JSON action format
- No verifier layer for ultra-reliability
- Agent loop not autonomous (requires user interaction)
- No orchestrator coordination

---

## **🏗️ DESIGN SPECIFICATIONS**

### **1. ROUTER CLASSIFICATION SYSTEM**

**New File: `src/infrastructure/src/router.rs`**

```rust
use serde::{Deserialize, Serialize};
use crate::InferenceEngine;
use shared::types::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterRequest {
    pub user_input: String,
    pub has_attachments: bool,
    pub attachment_types: Vec<String>,
    pub recent_context_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterResponse {
    pub route: RouteType,
    pub confidence: f32,
    pub needs_tools: bool,
    pub needs_user_artifact: ArtifactType,
    pub clarifying_question: String,
    pub reason_short: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteType {
    DirectQA,
    RetrievalQA,
    EditText,
    EditCode,
    EditImage,
    EditFile,
    PlanOnly,
    ExecuteTools,
    NeedClarify,
    Refuse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    None,
    Text,
    Code,
    Image,
    File,
    Url,
}

pub struct Router {
    inference_engine: InferenceEngine,
}

impl Router {
    pub fn new(inference_engine: InferenceEngine) -> Self {
        Self { inference_engine }
    }

    pub async fn classify(&self, request: RouterRequest) -> Result<RouterResponse> {
        let prompt = self.build_router_prompt(&request);
        let response = self.inference_engine.generate(&prompt).await?;
        self.parse_router_response(&response)
    }

    fn build_router_prompt(&self, request: &RouterRequest) -> String {
        format!(
            r#"You are a ROUTER for an agentic AI system.

Your job is ONLY to classify user message and output JSON.
You do NOT answer the user.
You do NOT execute tasks.

ALLOWED ROUTES:
DIRECT_QA
RETRIEVAL_QA
EDIT_TEXT
EDIT_CODE
EDIT_IMAGE
EDIT_FILE
PLAN_ONLY
EXECUTE_TOOLS
NEED_CLARIFY
REFUSE

RULES:
- Output MUST be valid JSON only.
- Choose exactly ONE route.
- If user asks to edit something but does NOT include content, choose NEED_CLARIFY.
- If request requires searching, checking files, running commands, or multi-step work, choose EXECUTE_TOOLS.
- If it is a simple explanation that does not need tools, choose DIRECT_QA.
- Ask at most ONE clarifying question.

OUTPUT FORMAT:
{{
  "route": "DIRECT_QA|RETRIEVAL_QA|EDIT_TEXT|EDIT_CODE|EDIT_IMAGE|EDIT_FILE|PLAN_ONLY|EXECUTE_TOOLS|NEED_CLARIFY|REFUSE",
  "confidence": 0.0,
  "needs_tools": true,
  "needs_user_artifact": "none|text|code|image|file|url",
  "clarifying_question": "",
  "reason_short": ""
}}

USER MESSAGE:
{}

CONTEXT:
- has_attachments: {}
- attachment_types: {:?}
- recent_context_summary: {}"#,
            request.user_input,
            request.has_attachments,
            request.attachment_types,
            request.recent_context_summary
        )
    }

    fn parse_router_response(&self, response: &str) -> Result<RouterResponse> {
        // Parse JSON response from router model
        serde_json::from_str(response).map_err(|e| {
            anyhow::anyhow!("Failed to parse router response: {} - Response: {}", e, response)
        })
    }
}
```

### **2. MAIN AGENT LOOP**

**New File: `src/infrastructure/src/autonomous_agent.rs`**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{InferenceEngine, tools::{ToolRegistry, ToolArgs, ToolOutput}};
use super::router::{RouteType, Router};
use shared::types::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub action: ActionType,
    pub route: RouteType,
    pub confidence: f32,
    pub tool: Option<ToolCall>,
    pub ask_user: Option<AskUser>,
    pub finish: Option<FinishAction>,
    pub reason_short: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    ToolCall,
    AskUser,
    Finish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub input: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUser {
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishAction {
    pub answer: String,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub r#type: String,
    pub content: String,
}

pub struct AutonomousAgent {
    inference_engine: InferenceEngine,
    tool_registry: ToolRegistry,
    router: Router,
    current_state: AgentState,
    execution_history: Vec<ExecutionRecord>,
    budgets: AgentBudgets,
}

#[derive(Debug, Clone)]
pub struct AgentState {
    pub user_request: String,
    pub route: RouteType,
    pub available_tools: Vec<String>,
    pub memory_summary: String,
    pub iteration_count: u32,
    pub total_tools_executed: u32,
}

#[derive(Debug, Clone)]
pub struct AgentBudgets {
    pub max_iterations: u32,
    pub max_tools_per_iteration: u32,
    pub max_execution_time_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub iteration: u32,
    pub action: AgentAction,
    pub tool_result: Option<ToolOutput>,
    pub success: bool,
    pub execution_time_ms: u64,
}

impl Default for AgentBudgets {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            max_tools_per_iteration: 3,
            max_execution_time_seconds: 300, // 5 minutes
        }
    }
}

impl AutonomousAgent {
    pub fn new(inference_engine: InferenceEngine, tool_registry: ToolRegistry) -> Self {
        let router = Router::new(inference_engine.clone());
        Self {
            inference_engine,
            tool_registry,
            router,
            current_state: AgentState {
                user_request: String::new(),
                route: RouteType::DirectQA,
                available_tools: tool_registry.list_tools(),
                memory_summary: String::new(),
                iteration_count: 0,
                total_tools_executed: 0,
            },
            execution_history: Vec::new(),
            budgets: AgentBudgets::default(),
        }
    }

    pub async fn execute_autonomous(&mut self, user_request: &str) -> Result<AgentResult> {
        // Initialize state
        self.current_state.user_request = user_request.to_string();
        self.current_state.memory_summary = self.build_memory_summary();

        // Route classification
        let route_request = super::router::RouterRequest {
            user_input: user_request.to_string(),
            has_attachments: false,
            attachment_types: vec![],
            recent_context_summary: self.current_state.memory_summary.clone(),
        };

        let route_response = self.router.classify(route_request).await?;
        self.current_state.route = route_response.route;

        // Main execution loop
        loop {
            if self.should_exit() {
                return Ok(self.build_final_result());
            }

            let action = self.decide_next_action().await?;
            
            match action.action {
                ActionType::ToolCall => {
                    if let Some(tool_call) = action.tool {
                        let result = self.execute_tool(tool_call.clone()).await?;
                        self.record_execution(action, Some(result), true).await;
                    }
                }
                ActionType::AskUser => {
                    if let Some(ask_user) = action.ask_user {
                        return Ok(AgentResult {
                            success: false,
                            final_answer: String::new(),
                            artifacts: vec![],
                            execution_summary: ExecutionSummary {
                                total_iterations: self.current_state.iteration_count,
                                tools_executed: self.current_state.total_tools_executed,
                                execution_time_ms: 0,
                                verification_passes: 0,
                                failures_recovered: 0,
                            },
                            user_input_needed: Some(ask_user.question),
                        });
                    }
                }
                ActionType::Finish => {
                    if let Some(finish) = action.finish {
                        return Ok(AgentResult {
                            success: true,
                            final_answer: finish.answer,
                            artifacts: finish.artifacts,
                            execution_summary: self.build_summary(),
                            user_input_needed: None,
                        });
                    }
                }
            }
        }
    }

    async fn decide_next_action(&self) -> Result<AgentAction> {
        let prompt = self.build_agent_prompt();
        let response = self.inference_engine.generate(&prompt).await?;
        self.parse_agent_action(&response)
    }

    fn build_agent_prompt(&self) -> String {
        format!(
            r#"You are an AGENT running inside a deterministic Rust orchestrator.

You DO NOT control execution.
You ONLY propose ONE next action at a time.

The orchestrator will:
- give you current state
- execute tools safely
- return tool results
- enforce budgets and policies

━━━━━━━━━━ RULES ━━━━━━━━━━
1. Output MUST be valid JSON and NOTHING else.
2. Choose exactly ONE action:
   - TOOL_CALL
   - ASK_USER
   - FINISH
3. NEVER hallucinate tool results.
4. Use tools ONLY when required.
5. If input is missing or ambiguous, ask ONE clarifying question.
6. Respect tool allowlists and budgets.
7. Be concise and correct.

━━━━━━━━━━ ACTION SCHEMA ━━━━━━━━━━
{{
  "action": "TOOL_CALL|ASK_USER|FINISH",
  "route": "{:?}",
  "confidence": 0.0,

  "tool": {{
    "name": "string",
    "input": {{}}
  }},

  "ask_user": {{
    "question": "string"
  }},

  "finish": {{
    "answer": "string",
    "artifacts": [
      {{ "type": "string", "content": "string" }}
    ]
  }},

  "reason_short": "short explanation"
}}

━━━━━━━━━━ CURRENT STATE ━━━━━━━━━━
User request:
{}

Route:
{:?}

Available tools:
{:?}

Memory summary:
{}

Budgets:
- max_iterations: {}
- current_iteration: {}
- max_tools_per_iteration: {}
- total_tools_executed: {}"#,
            self.current_state.route,
            self.current_state.user_request,
            self.current_state.route,
            self.current_state.available_tools,
            self.current_state.memory_summary,
            self.budgets.max_iterations,
            self.current_state.iteration_count,
            self.budgets.max_tools_per_iteration,
            self.current_state.total_tools_executed
        )
    }

    fn parse_agent_action(&self, response: &str) -> Result<AgentAction> {
        serde_json::from_str(response).map_err(|e| {
            anyhow::anyhow!("Failed to parse agent action: {} - Response: {}", e, response)
        })
    }

    async fn execute_tool(&mut self, tool_call: ToolCall) -> Result<ToolOutput> {
        let tool_name = &tool_call.name;
        let tool = self.tool_registry.get_tool(tool_name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

        let args = ToolArgs {
            parameters: tool_call.input,
            timeout: None,
            working_directory: None,
        };

        tool.execute(args).await
    }

    fn should_exit(&self) -> bool {
        self.current_state.iteration_count >= self.budgets.max_iterations ||
        self.current_state.total_tools_executed >= self.budgets.max_tools_per_iteration * self.budgets.max_iterations
    }

    fn record_execution(&mut self, action: AgentAction, tool_result: Option<ToolOutput>, success: bool) {
        self.current_state.iteration_count += 1;
        if action.tool.is_some() {
            self.current_state.total_tools_executed += 1;
        }

        self.execution_history.push(ExecutionRecord {
            iteration: self.current_state.iteration_count,
            action,
            tool_result,
            success,
            execution_time_ms: 0, // Should be measured
        });
    }

    fn build_final_result(&self) -> AgentResult {
        AgentResult {
            success: false,
            final_answer: "Budget exceeded or max iterations reached".to_string(),
            artifacts: vec![],
            execution_summary: self.build_summary(),
            user_input_needed: None,
        }
    }

    fn build_memory_summary(&self) -> String {
        format!(
            "Executed {} iterations, used {} tools, current route: {:?}",
            self.current_state.iteration_count,
            self.current_state.total_tools_executed,
            self.current_state.route
        )
    }

    fn build_summary(&self) -> ExecutionSummary {
        ExecutionSummary {
            total_iterations: self.current_state.iteration_count,
            tools_executed: self.current_state.total_tools_executed,
            execution_time_ms: 0, // Should be accumulated
            verification_passes: 0,
            failures_recovered: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub success: bool,
    pub final_answer: String,
    pub artifacts: Vec<Artifact>,
    pub execution_summary: ExecutionSummary,
    pub user_input_needed: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub total_iterations: u32,
    pub tools_executed: u32,
    pub execution_time_ms: u64,
    pub verification_passes: u32,
    pub failures_recovered: u32,
}
```

### **3. VERIFIER SYSTEM**

**New File: `src/infrastructure/src/verifier.rs`**

```rust
use serde::{Deserialize, Serialize};
use crate::{InferenceEngine, tools::ToolOutput};
use shared::types::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub user_input: String,
    pub agent_draft: String,
    pub tool_results: Vec<ToolOutput>,
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResponse {
    pub status: VerificationStatus,
    pub missing: Vec<String>,
    pub next_action: Option<VerificationNextAction>,
    pub final_answer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationStatus {
    Approve,
    NeedsTool,
    NeedsUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationNextAction {
    pub action: ActionType,
    pub tool: Option<ToolCall>,
    pub question: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    ToolCall,
    AskUser,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub input: std::collections::HashMap<String, String>,
}

pub struct Verifier {
    inference_engine: InferenceEngine,
}

impl Verifier {
    pub fn new(inference_engine: InferenceEngine) -> Self {
        Self { inference_engine }
    }

    pub async fn verify(&self, request: VerificationRequest) -> Result<VerificationResponse> {
        let prompt = self.build_verification_prompt(&request);
        let response = self.inference_engine.generate(&prompt).await?;
        self.parse_verification_response(&response)
    }

    fn build_verification_prompt(&self, request: &VerificationRequest) -> String {
        format!(
            r#"You are a VERIFIER.

Your job is to check if agent's answer satisfies user request.
You do NOT add new information unless fully supported by evidence.

RULES:
- Output JSON only.
- If information depends on tools, ensure tool results exist.
- If something is missing, request exactly ONE next action.
- If everything is correct, approve and provide final answer.

OUTPUT FORMAT:
{{
  "status": "APPROVE|NEEDS_TOOL|NEEDS_USER",
  "missing": ["string"],
  "next_action": {{
    "action": "TOOL_CALL|ASK_USER|NONE",
    "tool": {{ "name": "string", "input": {{}} }},
    "question": "string"
  }},
  "final_answer": "string"
}}

USER REQUEST:
{}

AGENT ANSWER:
{}

TOOL RESULTS:
{:?}

SUCCESS CRITERIA:
{:?}"#,
            request.user_input,
            request.agent_draft,
            request.tool_results,
            request.success_criteria
        )
    }

    fn parse_verification_response(&self, response: &str) -> Result<VerificationResponse> {
        serde_json::from_str(response).map_err(|e| {
            anyhow::anyhow!("Failed to parse verification response: {} - Response: {}", e, response)
        })
    }
}
```

### **4. ORCHESTRATOR LAYER**

**New File: `src/infrastructure/src/orchestrator.rs`**

```rust
use crate::{
    InferenceEngine,
    tools::{ToolRegistry, ToolOutput},
    router::{Router, RouterRequest},
    autonomous_agent::{AutonomousAgent, AgentResult},
    verifier::{Verifier, VerificationRequest, VerificationResponse},
};
use shared::types::Result;
use std::collections::HashMap;

pub struct Orchestrator {
    router: Router,
    agent: AutonomousAgent,
    verifier: Verifier,
    tool_registry: ToolRegistry,
    execution_context: ExecutionContext,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub user_request: String,
    pub attachments: Vec<Attachment>,
    pub recent_context: Vec<Message>,
    pub policies: SecurityPolicy,
    pub budgets: AgentBudgets,
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub r#type: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub allow_network_access: bool,
    pub allow_file_system: bool,
    pub allow_system_commands: bool,
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub final_answer: String,
    pub artifacts: Vec<Artifact>,
    pub execution_summary: ExecutionSummary,
}

impl Orchestrator {
    pub fn new(inference_engine: InferenceEngine) -> Self {
        let tool_registry = ToolRegistry::new();
        let router = Router::new(inference_engine.clone());
        let agent = AutonomousAgent::new(inference_engine.clone(), tool_registry.clone());
        let verifier = Verifier::new(inference_engine);

        Self {
            router,
            agent,
            verifier,
            tool_registry,
            execution_context: ExecutionContext {
                user_request: String::new(),
                attachments: vec![],
                recent_context: vec![],
                policies: SecurityPolicy {
                    allow_network_access: true,
                    allow_file_system: true,
                    allow_system_commands: false,
                },
                budgets: AgentBudgets::default(),
            },
        }
    }

    pub async fn execute_request(&mut self, user_request: &str) -> Result<ExecutionResult> {
        // Initialize execution context
        self.execution_context.user_request = user_request.to_string();

        // Execute autonomous agent
        let agent_result = self.agent.execute_autonomous(user_request).await?;

        // If agent finished successfully, verify the result
        if agent_result.success && !agent_result.final_answer.is_empty() {
            match self.verify_result(&agent_result).await? {
                Some(verified_result) => verified_result,
                None => Ok(self.convert_agent_result(agent_result)),
            }
        } else {
            Ok(self.convert_agent_result(agent_result))
        }
    }

    async fn verify_result(&mut self, agent_result: &AgentResult) -> Result<Option<ExecutionResult>> {
        let verification_request = VerificationRequest {
            user_input: self.execution_context.user_request.clone(),
            agent_draft: agent_result.final_answer.clone(),
            tool_results: vec![], // Should be extracted from agent execution history
            success_criteria: vec![
                "Answer addresses user request".to_string(),
                "Evidence supports claims".to_string(),
                "No hallucinations detected".to_string(),
            ],
        };

        let verification_response = self.verifier.verify(verification_request).await?;

        match verification_response.status {
            super::verifier::VerificationStatus::Approve => {
                if let Some(final_answer) = verification_response.final_answer {
                    return Ok(Some(ExecutionResult {
                        success: true,
                        final_answer,
                        artifacts: agent_result.artifacts.clone(),
                        execution_summary: agent_result.execution_summary.clone(),
                    }));
                }
            }
            super::verifier::VerificationStatus::NeedsTool => {
                // Execute additional tool and re-run agent
                if let Some(next_action) = verification_response.next_action {
                    if let Some(tool) = next_action.tool {
                        let _tool_result = self.execute_tool_directly(tool).await?;
                        // Re-run agent with new tool result
                        let new_result = self.agent.execute_autonomous(&self.execution_context.user_request).await?;
                        return Ok(Some(self.convert_agent_result(new_result)));
                    }
                }
            }
            super::verifier::VerificationStatus::NeedsUser => {
                // Return user question
                if let Some(next_action) = verification_response.next_action {
                    if let Some(question) = next_action.question {
                        return Ok(Some(ExecutionResult {
                            success: false,
                            final_answer: format!("User input required: {}", question),
                            artifacts: vec![],
                            execution_summary: agent_result.execution_summary.clone(),
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn execute_tool_directly(&mut self, tool_call: super::autonomous_agent::ToolCall) -> Result<ToolOutput> {
        let tool = self.tool_registry.get_tool(&tool_call.name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_call.name))?;

        let args = crate::tools::ToolArgs {
            parameters: tool_call.input,
            timeout: None,
            working_directory: None,
        };

        tool.execute(args).await
    }

    fn convert_agent_result(&self, agent_result: AgentResult) -> ExecutionResult {
        ExecutionResult {
            success: agent_result.success,
            final_answer: agent_result.final_answer,
            artifacts: agent_result.artifacts,
            execution_summary: agent_result.execution_summary,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub r#type: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub total_iterations: u32,
    pub tools_executed: u32,
    pub execution_time_ms: u64,
    pub verification_passes: u32,
    pub failures_recovered: u32,
}
```

---

## **🚀 IMPLEMENTATION PHASES**

### **Phase 1: Core Router System (Priority 1)**
1. **Create `router.rs`** with classification logic
2. **Integrate with existing `InferenceEngine`**
3. **Add routing tests** for all 9 route types
4. **Connect to existing `input_classifier`** (extend functionality)

**Files to modify/create:**
- `src/infrastructure/src/router.rs` (NEW)
- `src/infrastructure/src/lib.rs` (add `pub mod router`)

### **Phase 2: Autonomous Agent Loop (Priority 1)**
1. **Create `autonomous_agent.rs`** with JSON action schema
2. **Replace manual agent loops** in `agent_service.rs`
3. **Integrate with existing `AgentController`**
4. **Add self-critique and budget enforcement**

**Files to modify/create:**
- `src/infrastructure/src/autonomous_agent.rs` (NEW)
- `src/infrastructure/src/lib.rs` (add module)
- `src/application/src/agent_service.rs` (MODIFY to use new autonomous loop)

### **Phase 3: Ultra-Reliable Verifier (Priority 1)**
1. **Create `verifier.rs`** with evidence checking
2. **Add verification prompts** for each route type
3. **Integrate with tool result validation**
4. **Add learning from verification failures**

**Files to modify/create:**
- `src/infrastructure/src/verifier.rs` (NEW)
- `src/infrastructure/src/lib.rs` (add module)

### **Phase 4: Orchestrator Coordination (Priority 2)**
1. **Create `orchestrator.rs`** as main coordinator
2. **Replace current CLI flow** with orchestrated execution
3. **Add comprehensive logging and observability**
4. **Implement graceful budget exhaustion handling**

**Files to modify/create:**
- `src/infrastructure/src/orchestrator.rs` (NEW)
- `src/presentation/src/cli.rs` (MODIFY to use orchestrator)
- `src/src/main.rs` (SIMPLIFY from 11+ flags to 3-4)

### **Phase 5: Integration & Testing (Priority 2)**
1. **Update tool registry** to match new action schema
2. **Extend existing safety system** for autonomous decisions
3. **Add comprehensive test suite** for full workflow
4. **Performance optimization** and resource monitoring

---

## **🎬 END-STATE VISION**

### **User Experience Transformation**

#### **BEFORE (Current Bro):**
```bash
$ bro --agent --rag "implement user authentication"

❓ Multiple conflicting flags detected
❓ Please specify: --chat, --agent, or --rag?
❓ Manual confirmation required for each step
❓ 5+ minute setup time with multiple services
```

#### **AFTER (Autonomous Bro):**
```bash
$ bro "implement user authentication"

✅ Auto-detecting: MULTI-STEP TASK
✅ Route: EXECUTE_TOOLS (confidence: 0.94)
✅ Autonomous mode: Active (10 iteration budget)
✅ Verification: Ultra-reliable mode enabled
```

### **Expected Outcomes**

#### **Before Implementation:**
- 11+ confusing CLI flags
- Manual agent execution
- No self-critique or verification
- Over-engineered but not autonomous

#### **After Implementation:**
- **3-4 essential CLI flags** with auto-detection
- **Fully autonomous agent loop** with self-critique
- **Ultra-reliable verification** preventing hallucinations
- **Predictable JSON actions** for Rust orchestration
- **Budget-aware execution** preventing infinite loops
- **Evidence-based decisions** with verifiable outputs

#### **Performance Targets:**
- **Autonomous success rate**: >95%
- **Verification accuracy**: >99%  
- **False positive reduction**: 90% fewer
- **Agent loop convergence**: <5 iterations average
- **User intervention needed**: <5% of tasks

### **Key Differentiators vs Other Systems:**

| Feature | Current Bro | ChatGPT | Claude | Autonomous Bro |
|---------|-------------|----------|--------|----------------|
| Local Processing | ✅ | ❌ | ❌ | ✅ |
| Code Execution | ✅ | ❌ | ❌ | ✅ |
| File System Access | ✅ | ❌ | ❌ | ✅ |
| Self-Critique | Basic | Good | Good | Ultra-reliable |
| Budget Control | Basic | ❌ | ❌ | ✅ |
| Privacy | ✅ | ❌ | ❌ | ✅ |
| Autonomous Execution | ❌ | ❌ | ❌ | ✅ |

---

## **🔧 INTEGRATION POINTS**

### **Leveraging Existing Infrastructure:**
- **`InferenceEngine`**: Direct use in all new components
- **`SafeTool` enum**: Map 1:1 to new `ToolCall` schema  
- **`ToolArgs/ToolOutput`**: Use as-is for compatibility
- **`AgentController`**: Extend with new autonomous features
- **Safety system**: Add policy-based autonomous decisions

### **Migration Strategy:**
1. **Gradual replacement** - keep old system working
2. **Feature flag controlled rollout** using existing `feature_flags.rs`
3. **Backward compatibility** during transition
4. **Comprehensive testing** before full switch

---

## **📋 NEXT STEPS**

### **Immediate Actions:**
1. **Review and approve this plan** - Any modifications needed?
2. **Create new infrastructure modules** in order: Router → Agent → Verifier → Orchestrator
3. **Update existing modules** to use new autonomous system
4. **Comprehensive testing** at each phase
5. **Performance optimization** and monitoring

### **File Creation Order:**
1. `src/infrastructure/src/router.rs`
2. `src/infrastructure/src/autonomous_agent.rs`
3. `src/infrastructure/src/verifier.rs`
4. `src/infrastructure/src/orchestrator.rs`
5. Update `src/infrastructure/src/lib.rs`
6. Update `src/application/src/agent_service.rs`
7. Update `src/presentation/src/cli.rs`
8. Update `src/src/main.rs`

This plan transforms Bro from a feature-bloated CLI into a truly autonomous, self-critiquing AI agent that can reliably decide and execute tasks independently while maintaining safety through verification.