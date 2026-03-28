// Any Error related to audit logging
#[derive(Debug, thiserror::Error)]
pub enum AuditError{
    // Any failure when writing to the audit log (file open, write, flush, etc)
    #[error("io error writing audit log: {0}")]
    IoAudit(#[from] std::io::Error),

    // Log entry itself was Invalid (most Likely a Formatting or Serialization issue)
    #[error("The Log Entry is Malformed")]
    InvalidLog,

}