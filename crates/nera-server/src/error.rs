//Most HTTP-level errors are handled by Axum itself, so this is intentionally minimal (likely subject to change as Project grows)
#[derive(Debug, thiserror::Error)]
pub enum ServerError{
    #[error("The Port is Already in Use")]
    PortInUse,

    #[error("Incoming Request has Invalid JSON")]
    InvalidJson,

    #[error("An Agent ID is Missing from Request")]
    IdMissing,
}