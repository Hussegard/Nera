// take a command string, run it as a real process, return what happened.
// * on comment lines is for extra clariffcications 
pub mod error;
// Stdio is a standard library type that controls where a process sends its output.
// "piped" means: instead of printing to the terminal, send it to us so we can read it.
// std = standard library, built into Rust, no external dependency needed.
use std::process::Stdio;

// AsyncReadExt adds the read_to_end() method onto async streams.
// Without this import, calling .read_to_end() on a pipe would fail to compile.
// tokio is our async runtime
use tokio::io::AsyncReadExt;
// tokio's version of Command (not std's). The difference:
// std::process::Command BLOCKS — the whole program freezes while waiting (almost like wait())
// tokio::process::Command is ASYNC — it can wait without freezing everything else. (await())
use tokio::process::Command;
 
// "pub use error::*" means: take everything public from error.rs and re-export it.
// So anyone who imports nera-sandbox also automatically gets SandboxError without
// needing a separate import line.
pub use error::*;
// ExecutionResult lives in nera-core. We depend on nera-core in Cargo.toml.
use nera_core::ExecutionResult;

// The one function this file exists to provide.
// pub = other crates can call this. async = this function can pause and wait without blocking the program.
// command: &str = a borrowed string like "echo hello". We read it, we don't own it.
// timeout_secs: u64 = how many seconds before we give up and kill the process.
// -> Result<ExecutionResult, SandboxError> = returns either success or a SandboxError
pub async fn run_local (command: &str, timeout_secs: u64) -> Result<ExecutionResult, SandboxError> {

    // CRITICAL 1:Parse the command string into a program name and its arguments.
    // "echo hello world" is one string. We need to split it into: program = "echo" and arguments = ["hello", "world"]
    // split_whitespace() splits on any whitespace and returns an iterator.
    // collect() turns that iterator into a Vec (a growable list).
    // Vec<&str> means: a list of string slices (borrowed pieces of the original string).
    let parts: Vec<&str> = command.split_whitespace().collect();

    // Safety check: if someone passes an empty string, parts[0] would crash. return Err
    if parts.is_empty() {
        return Err(SandboxError::FailedToStart);
    }

    // parts[0] is the program name. Example: "echo"
    let program = parts[0];
    // parts[1..] is a SLICE of everything after index 0. Example: ["hello", "world"]
    // The & before parts means we borrow the slice instead of copying it.                  *
    // We don't need to own it, just read from it.
    //also errors out since we dont know hpw much this could hold                
    let arguments = &parts[1..];      
    

    
    // CRITICAL 2: Spawn the process.
    // Command::new(program) creates a builder for running "program".                         *
    // .args(arguments) passes the arguments to it.
    // .stdout(Stdio::piped()) means: don't print stdout to the terminal.                     *
    // Instead, give us a handle so we can read it ourselves.
    // .stderr(Stdio::piped()) same thing for stderr (error output).                          *
    //.spawn() actually launches the process. Returns a Result.
    // .map_err(|_| SandboxError::FailedToStart) — if spawn fails, convert the error.
    // spawn() returns std::io::Error, but our function returns SandboxError.
    // map_err converts between them. |_| means "I don't care what the original error        *
    // was, just replace it with FailedToStart."
    // ? at the end means: if this is an Err, stop here and return it to the caller.
    // The caller (eventually nera-server) will decide what to do with the error.
    let mut child = Command:: new(program)
                                    .args(arguments)
                                    .stdout(Stdio:: piped())
                                    .stderr(Stdio::piped())
                                    .spawn().map_err(|_| SandboxError::FailedToStart)?;



    // CRITICAL 3: Take the pipes OUT of child before the race.
    // KEY which solves the ownership problem
    // What is a pipe? When you set stdout to Stdio::piped(), the OS creates a
    // channel between the process and your program. The process writes into one end,
    // you read from the other end. That channel is the "pipe"
    //we take it out BEFORE the race, Because child.wait() (used in the race) requires ownership of child.      *
    // After the race, child is gone. If the stdout pipe was still INSIDE child,
    // we'd lose access to it. By taking the pipes out first, we own them separately
    // and can read them regardless of what happens to child.
    // .take() removes the pipe from child and gives it to us.
    // .ok_or(SandboxError::FailedToStart)? means: if take() returns None
    // (pipe wasn't set up for some reason), return FailedToStart immediately.
    let mut stdout_pipe = child.stdout.take().ok_or(SandboxError::FailedToStart)?;
    let mut stderr_pipe = child.stderr.take().ok_or(SandboxError::FailedToStart)?;



    // CRITICAL 4: Read stdout and stderr in the background while the process runs.
    //tokio::spawn launches a background task — it runs concurrently.
    //We spawn one task for stdout and one for stderr so they read simultaneously.
    //Why do this concurrently? Because if a process writes a lot to stderr,
    // and we waited for stdout first, the stderr buffer could fill up and
    // the process would freeze waiting for us to read it. Running both at once prevents that deadlock.
    //"async move" means: this is an async block, and it TAKES OWNERSHIP (moves)        *
    //of the variables it uses (stdout_pipe, stderr_pipe).
    // Vec::new() creates an empty list of bytes.
    //read_to_end(&mut buf) reads everything from the pipe into buf.
    // .await actually waits for that read to complete (it's async).
    // .map(|_| buf) means: if reading succeeded, return the buffer.
    let stdout_task = tokio::spawn(async move{
        let mut buf = Vec::new();
        stdout_pipe.read_to_end(&mut buf).await.map(|_| buf)
    });
    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stderr_pipe.read_to_end(&mut buf).await.map(|_| buf)
    });

    // Build a Duration object from our timeout number.
    // Duration::from_secs converts a plain u64 like 5 into "5 seconds."
    // tokio needs a Duration, not a raw number, so it knows the unit.
    let duration= std::time::Duration::from_secs(timeout_secs);
    


    // CRITICAL 5: THE RACE.
    // tokio::time::timeout(duration, child.wait()) races two things:
    //  1. The timer (duration)
    //  2. Waiting for the child process to finish (child.wait())
    // child.wait() does NOT take ownership — it just waits.
    // This means we still own child and can kill it if the timer wins.
    // .await actually runs the race.
    // The result is a nested Result:
    //   Err(_)       — timer expired, process still running
    //   Ok(Err(_))   — process finished, but wait() itself failed (OS error)
    //   Ok(Ok(status)) — process finished successfully, status has the exit code
    // "let status =" means on success we extract just the status for use below.
    let status = match  tokio::time::timeout(duration, child.wait()).await {

        // Timer expired. Process is still running. Kill it.
        // child.kill() sends a kill signal to the process.
        // .await waits for the kill to complete.
        // .ok() means "ignore whether kill succeeded" — the process might already be dead.
        // "return" exits the whole function immediately with this error.
        Err(_) => {
            child.kill().await.ok();
            return Err(SandboxError::TimeOut)
        }
        // Process finished in time, but something went wrong at the OS level reading it.
        // Rare, but must be handled
        Ok(Err(_)) => {
            return Err(SandboxError::FailedToStart) //waiting on the child failed, return sandbox error
        }
        // Process finished successfully. Extract the status value for use below.
        // This is the only arm that doesn't return early —
        // "status" becomes the value of the whole match expression.
        Ok(Ok(status)) => status,
    };

    // CRITICAL 6
    // The process finished. Now collect what the background tasks read.
    // stdout_task.await waits for the background reading task to finish.
    // This returns Result<Result<Vec<u8>, io::Error>, JoinError>
    // — another nested Result.
    // First .map_err: if the TASK itself panicked (JoinError), return our error.
    // ? propagates that error up.
    // Second .map_err: if reading the pipe failed (io::Error), return our error.
    // ? propagates that too.
    // After both, stdout_bytes is a plain Vec<u8> — raw bytes of the output.
    let stdout_bytes = stdout_task.await
                                .map_err(|_| SandboxError::FailedToStart)?
                                .map_err(|_| SandboxError::FailedToStart)?;

    let stderr_bytes = stderr_task.await 
                                .map_err(|_| SandboxError::FailedToStart)?
                                .map_err(|_| SandboxError::FailedToStart)?;


    //Critical 7: convert bytes to strings and return OK                                                   
    // Convert raw bytes (Vec<u8>) to a String
    // from_utf8_lossy handles weird/invalid characters gracefully
    // instead of crashing — it replaces them with a placeholder.
    // .to_string() converts from a borrowed Cow<str> to an owned String.
    let stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
    let stderr = String::from_utf8_lossy(&stderr_bytes).to_string();

    // an exit code is where every process returns a number when it finishes
    // 0 means success. Anything else means something went wrong.
    // status.code() returns Option<i32> ,  it's Option because on some platforms
    // or kill signals, there is no exit code.
    // This matches the exit_code field in ExecutionResult which is also Option<i32>.
    let exit_code = status.code();

    // Build and return the ExecutionResult.
    // Ok(...) wraps it in the success variant of Result.
    // duration_ms is None for now since we are not timing execution yet (V2 addition).
    // sandbox_id is None since local processes don't have a container ID.
    // SOME keyword is used since stdout & stderr require an option<> and they are just strings so they must be wrapped
    Ok( ExecutionResult { 
        command: command.to_string(),
        exit_code: exit_code,
        stdout: Some(stdout), 
        stderr: Some(stderr), 
        duration_ms: None, 
        sandbox_id: None, 
    })
}