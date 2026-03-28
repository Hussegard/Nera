# Nera — BUILD PLAN V1 (Clean, Chunked)

This is a practical build plan for V1. It removes coaching and focuses only on engineering work. The order follows the original plan, grouped into concrete chunks. Each chunk has small, actionable steps.


## Principles for V1

* Keep scope tight: sandbox + policy + audit + simple API
* Single machine, local-first
* Docker as the only sandbox backend
* Favor correctness over scale (low concurrency is fine)
* Ship a working end-to-end flow early


## Chunk 1 — Repo & Workspace Setup

Goal: create a clean Rust workspace and skeleton services.

Steps:

1. Initialize repo and Rust workspace (`Cargo.toml` with `[workspace]`).
2. Create crates:

   * `nera-core` (types, errors)
   * `nera-sandbox` (execution backends)
   * `nera-policy` (policy parsing + checks)
   * `nera-audit` (logging)
   * `nera-server` (HTTP API)
3. Add dependencies:

   * `tokio`, `axum`, `serde`, `serde_json`, `toml`, `tracing`, `thiserror`, `anyhow`
4. Set up `tracing` with JSON output (stdout for now).
5. Add basic error types in `nera-core`.
6. Create a minimal `main` in `nera-server` with `/health` route.
7. Verify workspace builds and server runs.


## Chunk 2 — Local Process Sandbox (Prototype)

Goal: understand execution control before Docker.

Steps:

1. In `nera-sandbox`, implement `run_command_local(cmd, args, cwd)` using `std::process::Command`.
2. Capture stdout, stderr, exit code.
3. Add timeout support (kill process after N seconds).
4. Restrict working directory to a temp folder (create per request).
5. Block obvious dangerous commands (temporary allowlist).
6. Return a structured result:

   * command, args, exit_code, stdout, stderr, duration
7. Add unit tests for success, failure, timeout.
8. Wire a temporary endpoint `POST /execute-local` to call this.


## Chunk 3 — Docker Sandbox (V1 Backend)

Goal: move execution into containers.

Steps:

1. Choose Docker access method (CLI first for simplicity).
2. Create a base image (e.g., `nera-runner`) with common tools (git, python, node, pytest).
3. Implement `run_command_docker(cmd, args, workspace)`:

   * `docker run --rm` with bind mount to `/workspace`
4. Pass command via `sh -lc` or exec form.
5. Capture logs and exit code.
6. Add resource flags (memory/cpu limits) to container run.
7. Disable network by default (Docker flags) for V1.
8. Clean up containers reliably.

## Chunk 4 — Policy Parsing (TOML)

Goal: load and validate policy files.

Steps:

1. Define policy schema structs in `nera-policy`.
2. Load from `policy.toml` using `toml` crate.
3. Fields for V1:

   * `allowed_commands`, `blocked_commands`, `allowed_dirs`, `network_access`
4. Add validation (no empty rules, conflicting rules).
5. Provide defaults for missing fields.
6. Expose a `Policy` object usable by runtime.
7. Add unit tests for parsing and validation.

## Chunk 5 — Policy Enforcement

Goal: decide allow/deny before execution.

Steps:

1. Implement `check_command(policy, cmd, args, cwd)`.
2. Enforce allowlist/denylist for commands.
3. Validate working directory is within `allowed_dirs`.
4. Enforce network rule (passed to sandbox layer).
5. Return structured decision: allow/deny + reason.
6. Integrate with sandbox call path:

   * if deny → do not execute
   * if allow → call Docker sandbox
7. Add tests for common cases (allowed, denied, edge cases).

## Chunk 6 — Audit Logger (Structured)

Goal: record every action consistently.

Steps:

1. Define audit event struct:

   * timestamp, agent_id, command, args, decision, reason, exit_code, duration
2. Use `tracing` to emit JSON logs.
3. Log both decisions and execution results.
4. Include correlation/request ID per call.
5. Write logs to file (in addition to stdout) for V1.
6. Add helper to format and emit events from all modules.
7. Verify logs are machine-readable and consistent.


## Chunk 7 — HTTP API (Axum)

Goal: expose runtime over HTTP.

Steps:

1. Define `POST /execute` request/response schema.
2. Request includes:

   * `agent_id`, `command`, `args`, `working_dir`
3. In handler:

   * load policy
   * run policy check
   * log decision
   * if allowed → run sandbox
   * log result
4. Return structured response with decision + result.
5. Add `GET /health`.
6. Add basic error handling (map to HTTP codes).
7. Add simple request ID middleware.

## Chunk 8 — End-to-End Integration & Hardening

Goal: make the system usable and stable.

Steps:

1. Run full flow locally with real commands (`pytest`, `git diff`).
2. Add per-request temp workspace and cleanup.
3. Ensure container cleanup on failure/timeouts.
4. Add bounded concurrency (small limit, e.g., 2–4 workers).
5. Add timeouts and cancellation paths end-to-end.
6. Improve error messages and responses.
7. Basic CLI or curl examples for usage.
8. Document how to run (Docker required, policy file location).


## Definition of Done (V1)

* Accepts `POST /execute`
* Enforces policy before execution
* Runs allowed commands in Docker sandbox
* Blocks disallowed actions
* Produces structured audit logs
* Cleans up resources reliably


## Deferred (Not V1)

* Firecracker backend
* Kubernetes / distributed execution
* Advanced policy language (RBAC, approvals)
* Dashboard (TypeScript)
* Log query APIs
* High concurrency / autoscaling

## Notes

* Keep changes small and test each chunk before moving on.
* Do not expand scope during V1.
* Prefer simple, working implementations over perfect abstractions.
