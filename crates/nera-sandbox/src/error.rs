// Errors that occur during sandboxed execution (The Docker layer)
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("Execution Timed Out")]
    TimeOut,

    //Bad image, Invalid command, or Docker Issue
    #[error("The Container Failed to Start")]
    FailedToStart,

    //Any error returned by the Docker API
    //#[from]` allows automatic conversion when using `?` on Docker calls.
    #[error("Docker is Not Available {0}")]
    DockerError(#[from] bollard::errors::Error),

    //container may have ran successfully but we failed to read output
    #[error("The Container Output Could Not be Read")]
    CantReadContainerOutput,
}
