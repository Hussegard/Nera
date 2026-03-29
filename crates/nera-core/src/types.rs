/// Execution result is a struct thats going to hold all aspects of the executed move, including: 
/// What command was executed? Did it succeed? What does it print to screen? What errors/warnings did it print? Duration? 
/// some are options due to the dependant nature of some of the feilds (whether execution succeeds or not)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionResult{
    pub command: String,
    pub exit_code: Option <i32>, // status number a process returns after it executes which can give us further insight
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub duration_ms: Option<u64>, 
    pub sandbox_id: Option<String>
}

/// There are only two possible answers for the policy decision: allowed, or denied with a reason. 
/// allow carries nothing, deny carries a reason string for why the decision was denied 
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
    RequiresApproval { reason: String },  // (for V3) structurally present now
}

/// much ot be studied on the audit structure/skeleton, will come back later with more 
/// in depth comments explaining every facet of this 
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    pub event_id: String,              // was "id" — rename for clarity
    pub timestamp: String,             // was "time_stamp" — rename for consistency
    pub event_type: String,            // "command_execution", "policy_violation", "sandbox_error"
    pub request_id: String,            // UUID correlating all events from one API request
    pub session_id: Option<String>,    // correlates events within an agent session
    pub agent_id: String,
    pub command: String,
    pub policy_decision: PolicyDecision,
    pub policy_reason: String,         // human-readable reason for the decision
    pub matched_rule: Option<String>,  // which rule triggered the decision
    pub sandbox_id: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>,
    pub stdout_lines: Option<u64>,     // count, not content (content can be huge)
    pub stderr_lines: Option<u64>,
    pub prev_hash: String,             // SHA-256 of previous event — tamper-evident chain
    pub event_hash: Option<String>,    // SHA-256 of this event (computed after serialization)
}