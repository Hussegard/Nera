// nera-sandbox/src/tests.rs
// Step 1.1 test suite — validates run_local() behavior before the SandboxBackend trait is introduced.
// Tests are grouped: the three required cases from v1.md first, then edge cases covering real attack surfaces.
// All tests are async because run_local() is async.

#[cfg(test)]
mod tests {
    use crate::run_local;
    use crate::SandboxError;

    // -------------------------------------------------------------------------
    // THE THREE REQUIRED TESTS (v1.md, Step 1.1)
    // -------------------------------------------------------------------------

    // Happy path. Proves spawn, pipe capture, and result construction all work together.
    // Uses contains() not == because echo appends \n and behavior can vary slightly by platform.
    #[tokio::test]
    async fn success_echo_returns_zero_and_captures_stdout() {
        let result = run_local("echo hello", 5).await.expect("echo should not fail");

        assert_eq!(result.exit_code, Some(0), "echo must exit with 0");
        assert!(
            result.stdout.as_deref().unwrap_or("").contains("hello"),
            "stdout must contain 'hello', got: {:?}",
            result.stdout
        );
    }

    // "Command ran but reported failure" — distinct from "command couldn't start."
    // We don't hardcode the exit code because ls failure codes vary by distro (e.g. 1 on Linux, 2 on macOS).
    // We just assert it's non-zero and that stderr has content, which is the meaningful contract.
    #[tokio::test]
    async fn failure_ls_nonexistent_returns_nonzero_and_captures_stderr() {
        let result = run_local("ls /this_path_does_not_exist_nera", 5)
            .await
            .expect("ls should start even if the path doesn't exist");

        assert_ne!(
            result.exit_code,
            Some(0),
            "ls on a nonexistent path must not exit 0"
        );
        assert!(
            !result.stderr.as_deref().unwrap_or("").is_empty(),
            "stderr must contain an error message from ls"
        );
    }

    // Timeout enforcement. If child.kill() didn't work, this test would hang for 30 seconds
    // and eventually time out the test runner — making the failure impossible to miss.
    // Fast completion IS the proof that the kill path executed correctly.
    #[tokio::test]
    async fn timeout_kills_long_running_process() {
        let result = run_local("sleep 30", 1).await;

        assert!(
            matches!(result, Err(SandboxError::TimeOut)),
            "sleep 30 with a 1-second timeout must return SandboxError::TimeOut, got: {:?}",
            result
        );
    }

    // -------------------------------------------------------------------------
    // EDGE CASES — each covers a real attack surface or a downstream contract
    // -------------------------------------------------------------------------

    // A malformed API call could produce an empty command string.
    // run_local() must reject it cleanly rather than panicking on parts[0].
    #[tokio::test]
    async fn empty_command_is_rejected() {
        let result = run_local("", 5).await;

        assert!(
            matches!(result, Err(SandboxError::FailedToStart)),
            "empty command must return FailedToStart, got: {:?}",
            result
        );
    }

    // Whitespace-only strings pass split_whitespace() as empty — same danger as "".
    // This catches the subtler variant that a pure emptiness check would miss.
    #[tokio::test]
    async fn whitespace_only_command_is_rejected() {
        let result = run_local("   \t\n  ", 5).await;

        assert!(
            matches!(result, Err(SandboxError::FailedToStart)),
            "whitespace-only command must return FailedToStart, got: {:?}",
            result
        );
    }

    // An AI agent can hallucinate a program name that doesn't exist on the system.
    // spawn() will fail at the OS level. We must convert that to FailedToStart cleanly —
    // not a panic, not an unhandled io::Error surfacing to the caller.
    #[tokio::test]
    async fn nonexistent_program_returns_failed_to_start() {
        let result = run_local("this_program_absolutely_does_not_exist_nera_v1", 5).await;

        assert!(
            matches!(result, Err(SandboxError::FailedToStart)),
            "nonexistent program must return FailedToStart, got: {:?}",
            result
        );
    }

    // The command field in ExecutionResult feeds directly into nera-audit's tamper-evident chain.
    // If the string is modified in any way (trimmed, reconstructed from parts, etc.),
    // the audit log records something different from what was submitted — a correctness violation.
    #[tokio::test]
    async fn result_preserves_original_command_string() {
        let input = "echo   preserving   this   string";
        let result = run_local(input, 5).await.expect("echo should succeed");

        assert_eq!(
            result.command, input,
            "command field must be the exact original input string, byte-for-byte"
        );
    }

    // Validates that concurrent pipe reading works correctly.
    // If pipes were read sequentially and stderr filled its OS buffer first,
    // the process would block waiting for us to read stderr while we waited for stdout —
    // a classic deadlock. Running both via tokio::spawn prevents this.
    // This test forces output to both streams and verifies both are captured independently.
    #[tokio::test]
    async fn stdout_and_stderr_captured_independently() {
        // sh -c lets us run a small shell script as a single command string.
        // We write to stdout first, then stderr, to create the potential for ordering issues.
        let result = run_local("sh -c 'echo to_stdout; echo to_stderr >&2'", 5)
            .await
            .expect("sh should be available and succeed");

        assert!(
            result.stdout.as_deref().unwrap_or("").contains("to_stdout"),
            "stdout must contain 'to_stdout', got: {:?}",
            result.stdout
        );
        assert!(
            result.stderr.as_deref().unwrap_or("").contains("to_stderr"),
            "stderr must contain 'to_stderr', got: {:?}",
            result.stderr
        );
    }

    // `true` is a Unix command that exits 0 and produces no output.
    // stdout and stderr must be Some("") — not None.
    // Downstream code (nera-audit, nera-server) depends on these fields being Some
    // to know the pipes were set up correctly. None would signal a pipe failure.
    #[tokio::test]
    async fn silent_command_returns_some_empty_not_none() {
        let result = run_local("true", 5).await.expect("true must succeed");

        assert_eq!(
            result.stdout,
            Some(String::new()),
            "stdout must be Some(\"\") for a silent command, not None"
        );
        assert_eq!(
            result.stderr,
            Some(String::new()),
            "stderr must be Some(\"\") for a silent command, not None"
        );
    }

    // Local processes have no container — sandbox_id must always be None.
    // If it were ever Some(...), audit events would falsely indicate the command
    // ran inside a Docker container, corrupting the audit trail.
    #[tokio::test]
    async fn local_process_has_no_sandbox_id() {
        let result = run_local("echo audit_check", 5).await.expect("echo must succeed");

        assert_eq!(
            result.sandbox_id, None,
            "local process must never have a sandbox_id"
        );
    }
}