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
}

/// There are only two possible answers for the policy decision: allowed, or denied with a reason. 
/// allow carries nothing, deny carries a reason string for why the decision was denied 
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String},
}

/// The audit logger needs an event to record, What does a complete audit record need? A unique ID so you can find it later,
/// a timestamp, which agent made the request, what command was requested, what the policy decided, why if it was denied, 
/// and what the exit code was if it ran, and duration?
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    pub id: String, // uuid
    pub time_stamp: String, //ISO
    pub agent_id: String,
    pub command: String,
    pub decision: PolicyDecision, // return what the decision was, if denied then it would include the reasoning in enum
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u64>, 
}