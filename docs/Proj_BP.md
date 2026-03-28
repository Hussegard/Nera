# PROJECT_BLUEPRINT.md
# Nera — Master Project Document
**Version:** 0.1 (Initial Draft)
**Last Updated:** March 2026
**Status:** Pre-build, decisions locked, architecture defined

---

## Table of Contents

1. Project Name & Vision
2. Problem Statement
3. Why This Problem Matters
4. Niche Statement
5. Target Users
6. Non-Goals
7. Core Product Definition
8. Why This Is a Runtime, Not Something Else
9. Architecture Overview
10. V1 Scope — The Four Components (Months 1–4)
11. V2 Scope — Expanding the Platform (Months 5–8)
12. V3 Scope — Production & Fintech Pivot (Months 9–24)
13. Tech Stack & Rationale
14. Major Tradeoffs
15. Competition Landscape
16. Success Criteria
17. Roadmap Summary
18. Open Questions
19. Strategic Notes (Acquisition & Career)

---

---

## 1. Project Name & Vision

**Project Name:** `Nera`

**Tagline:** *The secure execution layer for autonomous coding agents.*

**One-Paragraph Vision:**

Autonomous AI coding agents — systems that write code, run tests, install dependencies, and deploy software — are being deployed into production environments with no meaningful security boundary between the agent and the host machine. Nera is a Rust-based execution layer that sits between any AI coding agent and the underlying system, intercepting every action, enforcing developer-defined policies, and producing a complete immutable audit trail of everything the agent did. Think Docker, but the thing being isolated isn't a service — it's an AI agent that can reason, deceive, and make mistakes with real consequences. The long-term goal is to become the standard runtime infrastructure for safe agent execution, starting with coding agents and expanding into regulated industries including financial services.

---

## 2. Problem Statement

Coding agents today operate with no security boundary between the LLM and the host system.

The current execution model looks like this:

```
LLM output
    ↓
Python script (e.g. LangChain, CrewAI, AutoGen)
    ↓
Shell commands (subprocess, os.system)
    ↓
Filesystem, network, credentials — unrestricted
```

There is nothing in that chain that:
- Validates what the agent is about to do
- Enforces limits on what it is allowed to do
- Records what it actually did
- Stops it from doing something destructive or leaking something sensitive

This is not a theoretical risk. Real consequences include:

- **Prompt injection attacks** — malicious instructions hidden in code comments, files, or web content that redirect the agent's behavior
- **Credential leakage** — agents reading `.env` files, SSH keys, or API tokens and exfiltrating them through network calls
- **Dependency attacks** — agents installing malicious packages from PyPI or npm without verification
- **Repository corruption** — agents modifying or deleting code they were never meant to touch
- **Runaway resource consumption** — agents spinning up infinite loops or consuming all available disk/memory

Every company deploying a coding agent today is accepting these risks silently because there is no standard tool that addresses them.

---

## 3. Why This Problem Matters (Market Context)

- The AI coding agent market is the fastest-growing software category in 2026
- 70%+ of organizations are already piloting AI agents in production workflows
- Security frameworks (OWASP LLM Top 10) now explicitly list agent tool misuse and prompt injection as critical risks
- Research shows that over 50% of malicious prompts succeed against current agent frameworks
- Regulatory pressure on AI in regulated industries (especially financial services) is increasing sharply in 2026
- No standard runtime security layer exists for coding agents — the space is undersolved at the infrastructure layer even while crowded at the application layer

The problem is not awareness. Developers and security teams know this is dangerous. The problem is that there is no good tool to fix it. That is the gap this project fills.

---

## 4. Niche Statement

**Primary niche (V1–V2):** Secure runtime infrastructure for autonomous coding agents

**Secondary niche (V2–V3):** Secure agent execution for regulated industries, starting with financial services / fintech

**What we are not:**
- Not a general AI security platform (too broad, too crowded)
- Not a coding agent ourselves (we are the infrastructure layer underneath them)
- Not an observability/monitoring tool (though we produce data those tools consume)
- Not a replacement for container technology (we use containers as a primitive)

**The one-sentence pitch:**
"Nera is the execution layer that makes AI coding agents safe to run in production — every action intercepted, every policy enforced, every decision logged."

---

---

## 5. Target Users

### V1 Primary User: Individual Developer / Researcher

- Building or experimenting with AI coding agents (using LangChain, AutoGen, CrewAI, Claude Code, Devin API, etc.)
- Frustrated by agents doing unexpected things to their filesystem or environment
- Technically sophisticated — comfortable with CLI tools, config files, Rust/Python
- Will try an open source tool if setup takes under 10 minutes
- Does not need a dashboard, does need good docs and a clear README
- **Where they live:** GitHub, Hacker News, r/LocalLLaMA, AI engineering Discord servers

### V2 Primary User: Engineering Team at a Startup

- Small team (5–50 engineers) building internal tooling with AI agents
- Has had at least one incident where an agent did something unexpected
- Wants policy enforcement and audit trails but doesn't have budget for enterprise security vendors
- Will pay for a managed/hosted version if it saves them building this themselves
- **Where they live:** Y Combinator companies, Series A/B startups, AI-first engineering teams

### V3 Primary User: Fintech / Bank Engineering or Security Team

- Deploying AI agents for internal workflows (compliance checks, code review, report generation)
- Subject to regulatory requirements — needs provable audit trails, access controls, data isolation
- Has a security team that will evaluate the product seriously
- Has budget — enterprise contracts in the $50K–$500K/year range are realistic
- Has a long procurement cycle — 6–18 months from first conversation to signed contract
- **Where they live:** Referrals, conferences (FinovateFall, Money20/20), fintech engineering blogs

---

## 6. Non-Goals

These are explicitly out of scope and should never creep in during V1 or V2:

- **We do not build coding agents.** We are the runtime they run on.
- **We do not build a UI dashboard in V1.** A CLI and config files are sufficient. Dashboards come when users ask for them.
- **We do not build distributed execution in V1 or V2.** Single-machine runtime only until real users need multi-node.
- **We do not target generic enterprise security buyers in V1.** Individual developers first.
- **We do not support Windows in V1.** Linux only. macOS support if it's easy. Windows adds enormous complexity for sandboxing primitives.
- **We do not attempt SOC 2 compliance in V1 or V2.** This is a V3 / startup phase concern.
- **We do not build our own container runtime.** We use Docker. We are not reinventing Docker.
- **We do not build a policy language compiler.** TOML/YAML policy files are sufficient for V1–V2.

---

## 7. Core Product Definition

Nera is a **local-first, open-source, Rust-based execution daemon** that wraps any shell command or tool call made by an AI agent and enforces a security policy before allowing execution.

The agent does not call the system directly. The agent calls the runtime. The runtime decides.

```
AI Agent (any framework)
        ↓
     [ Nera ]   ← this is what we build
        ↓
   Docker Sandbox
        ↓
  Host System (protected)
```

Every interaction flows through the runtime. No exceptions. No escape hatches.

**The three guarantees Nera provides:**

1. **Enforcement** — agents can only do what their policy file allows. Everything else is blocked.
2. **Isolation** — agent execution happens in a sandboxed environment, not the host.
3. **Auditability** — every action is logged immutably with full context. You can always reconstruct exactly what happened.

These three guarantees are the product. Everything else is implementation detail.

---

## 8. Why This Is a Runtime, Not Something Else

This question matters because it defines the architecture.

**Why not a proxy?** A proxy sits in the network path. We sit in the execution path. Network proxies miss filesystem operations, process spawning, and local tool calls. We intercept all of them.

**Why not a monitoring tool?** Monitoring observes and alerts after the fact. We enforce before execution. The difference is blocking a destructive action vs. notifying you it happened.

**Why not an agent framework?** Frameworks (LangChain, AutoGen) define how agents think and plan. We define how agents are allowed to act. These are orthogonal layers. Any agent framework should be able to use our runtime without modification.

**Why not a container?** Docker isolates a process. We add policy enforcement, identity management, and audit logging on top of isolation. A container is a primitive we use. A runtime is a contract we enforce.

**The analogy that explains it:**
- The OS kernel is a runtime for processes — processes don't access hardware directly, they call the kernel
- The JVM is a runtime for Java — bytecode doesn't run on bare metal, it runs in the VM
- **Nera is a runtime for AI agents** — agents don't touch the system directly, they operate through the runtime

This framing is important for how you explain the project to developers, investors, and potential acquirers. It positions the project correctly in the infrastructure stack.

---

## 9. Architecture Overview

### The Execution Flow (V1)

```
Agent sends tool request (HTTP to Agent API)
            ↓
    Agent API receives request
            ↓
    Policy Engine evaluates:
      - Is this command allowed?
      - Is this directory accessible?
      - Is this network call permitted?
            ↓
      [BLOCKED] → returns error + logs rejection
            ↓
      [ALLOWED] → passes to Execution Sandbox
            ↓
    Execution Sandbox:
      - Spawns Docker container
      - Sets resource limits
      - Executes command in isolation
      - Captures stdout/stderr
      - Enforces timeout
            ↓
    Audit Logger writes immutable record:
      - timestamp
      - agent_id
      - command
      - policy decision (allow/block)
      - result (exit code, output)
      - files touched
            ↓
    Agent API returns result to agent
```

### Component Interaction Diagram (V1)

```
┌─────────────────────────────────────────────┐
│              Nera Daemon            │
│                                              │
│  ┌──────────┐    ┌──────────────────────┐   │
│  │ Agent    │───▶│   Policy Engine      │   │
│  │ API      │    │  (reads policy.toml) │   │
│  │ (Axum)   │    └──────────┬───────────┘   │
│  └──────────┘               │               │
│                    allow    │    block       │
│                    ┌────────┘                │
│                    ▼                         │
│  ┌─────────────────────────┐                 │
│  │   Execution Sandbox     │                 │
│  │   (Docker container)    │                 │
│  └────────────┬────────────┘                 │
│               │                              │
│               ▼                              │
│  ┌─────────────────────────┐                 │
│  │     Audit Logger        │                 │
│  │  (append-only log file) │                 │
│  └─────────────────────────┘                 │
│                                              │
└─────────────────────────────────────────────┘
```

### V2 Architecture Addition

```
V1 components +
┌─────────────────────────────────────────────┐
│  ┌──────────────┐  ┌────────────────────┐   │
│  │ Tool Gateway │  │ Resource Controller│   │
│  │ (git/pip/npm)│  │ (CPU/mem/disk)     │   │
│  └──────────────┘  └────────────────────┘   │
│  ┌──────────────┐                           │
│  │Identity Mgr  │                           │
│  │(agent tokens)│                           │
│  └──────────────┘                           │
└─────────────────────────────────────────────┘
```

### V3 Architecture Addition

```
V1 + V2 components +
┌─────────────────────────────────────────────┐
│  ┌──────────────┐  ┌────────────────────┐   │
│  │Memory        │  │Workflow Controller │   │
│  │Isolation     │  │(multi-step agents) │   │
│  └──────────────┘  └────────────────────┘   │
│  ┌──────────────────────────────────────┐   │
│  │  Distributed Execution Layer         │   │
│  │  (control plane, runtime nodes,      │   │
│  │   agent scheduler)                   │   │
│  └──────────────────────────────────────┘   │
│  ┌──────────────────────────────────────┐   │
│  │  Firecracker Micro-VM support        │   │
│  │  (stronger isolation for fintech)    │   │
│  └──────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

---

---

## 10. V1 Scope — The Four Components (Months 1–4)

V1 has one goal: **a working, trustworthy runtime that a developer can point their coding agent at and immediately get sandbox enforcement + audit logging.**

Nothing else. No dashboard. No cloud. No distributed execution.

---

### Component 1: Execution Sandbox

**What it is:** The layer that takes a command approved by the policy engine and executes it inside a Docker container with full isolation from the host system.

**What it does:**
- Accepts a command (string) and execution context (working directory, environment variables, agent ID)
- Spins up a Docker container configured for isolation
- Executes the command inside the container
- Enforces a configurable timeout (default: 30 seconds)
- Captures stdout, stderr, and exit code
- Tears down the container after execution
- Returns the result to the caller

**What it does NOT do in V1:**
- Resource limiting (CPU/memory) — that's V2
- Persistent container environments — each execution is ephemeral
- Firecracker isolation — Docker only in V1

**Why it matters:** Without this, agents run directly on the host. This component is the physical boundary between the agent and the system.

**V1 definition of done:**
- [ ] Can execute any shell command inside a Docker container
- [ ] Enforces timeout, kills container if exceeded
- [ ] Returns stdout/stderr/exit code to caller
- [ ] Container is fully torn down after each execution
- [ ] No host filesystem is mounted by default

**Key Rust crates:** `bollard` (Docker API), `tokio` (async runtime)

---

### Component 2: Policy Engine

**What it is:** The decision layer. Before any command executes, the policy engine reads the agent's policy file and decides: allow or block.

**What it does:**
- Reads a `policy.toml` file at startup
- Evaluates every incoming command against the policy rules
- Returns `Allow` or `Deny` with a reason
- Supports rules for: allowed commands, allowed directories, allowed network destinations, blocked patterns

**Example `policy.toml`:**

```toml
[agent.code-review-bot]
allowed_commands = ["grep", "cat", "git diff", "pytest"]
blocked_commands = ["rm", "curl", "wget", "ssh"]
allowed_dirs = ["/workspace/repo"]
blocked_dirs = ["/etc", "/home", "~/.ssh"]
network_access = false
max_execution_time_seconds = 60

[agent.deployment-bot]
allowed_commands = ["git", "docker build", "npm install"]
blocked_commands = ["rm -rf", "dd", "mkfs"]
allowed_dirs = ["/workspace"]
network_access = true
allowed_hosts = ["github.com", "registry.npmjs.org"]
max_execution_time_seconds = 300
requires_approval = ["docker push", "kubectl apply"]
```

**Why it matters:** Without the policy engine, the sandbox is just a container. Anyone could execute anything inside it. The policy engine is what makes the runtime an enforcer, not just an isolator. This is the component that has the most direct business value — it's what enterprises will customize and pay for.

**V1 definition of done:**
- [ ] Reads and parses TOML policy file at startup
- [ ] Evaluates command against allowed/blocked lists
- [ ] Evaluates directory access against allowed/blocked paths
- [ ] Returns structured decision (Allow/Deny + reason)
- [ ] Reloads policy file on change without daemon restart
- [ ] Policy violations are logged with full context

**Key Rust crates:** `toml` (parsing), `serde` (deserialization)

---

### Component 3: Audit Logger

**What it is:** The immutable record of everything that happened. Every command attempted, every policy decision made, every execution result — written to an append-only log.

**What it does:**
- Writes a structured log entry for every event passing through the runtime
- Log entries are JSON-formatted for easy parsing
- Log file is append-only — entries are never modified or deleted
- Each entry includes: timestamp, agent_id, command, policy_decision, result, duration, files_touched
- Writes to local disk in V1 (remote log shipping in V3)

**Example log entry:**

```json
{
  "timestamp": "2026-03-22T14:32:01.442Z",
  "agent_id": "code-review-bot",
  "session_id": "sess_a1b2c3",
  "event_type": "command_execution",
  "command": "pytest tests/",
  "policy_decision": "allow",
  "sandbox_id": "sandbox_x9y8z7",
  "exit_code": 0,
  "duration_ms": 4821,
  "stdout_lines": 47,
  "stderr_lines": 0,
  "files_read": ["/workspace/repo/tests/test_api.py"],
  "files_written": [],
  "network_calls": []
}
```

**Why it matters:** The audit log is what makes the runtime trustworthy to enterprises and regulators. "We can prove exactly what your AI agent did" is a direct answer to the compliance requirements of V3 customers. Even in V1 for individual developers, the audit log is what lets you debug agent behavior and prove it didn't do something it shouldn't have.

**V1 definition of done:**
- [ ] Writes structured JSON log entry for every runtime event
- [ ] Append-only — no update or delete operations
- [ ] Includes all required fields (see example above)
- [ ] Log entries are written atomically (no partial/corrupt entries)
- [ ] Log rotation when file exceeds configurable size limit
- [ ] CLI command to read/filter logs (`Nera logs --agent code-review-bot`)

**Key Rust crates:** `tracing`, `tracing-subscriber`, `serde_json`

---

### Component 4: Agent API

**What it is:** The HTTP interface that AI agents use to communicate with the runtime. Instead of making direct system calls, agents make HTTP requests to the runtime daemon, which enforces policy and executes in the sandbox.

**What it does:**
- Runs as a local HTTP server (default port 7777)
- Accepts `POST /execute` with command and context
- Returns execution result (stdout, stderr, exit code, duration)
- Accepts `GET /health` — is the runtime running?
- Accepts `GET /policy` — what is the current policy for this agent?
- Accepts `GET /logs` — query recent audit log entries
- Authentication in V1: simple API key in header

**Example request/response:**

```
POST /execute
{
  "agent_id": "code-review-bot",
  "command": "pytest tests/",
  "working_dir": "/workspace/repo",
  "timeout_seconds": 60
}

→ 200 OK
{
  "status": "completed",
  "exit_code": 0,
  "stdout": "47 passed in 4.82s",
  "stderr": "",
  "duration_ms": 4821,
  "sandbox_id": "sandbox_x9y8z7",
  "audit_id": "log_entry_uuid"
}

→ 403 Forbidden (policy violation)
{
  "status": "blocked",
  "reason": "command 'rm -rf /workspace' matches blocked pattern 'rm -rf'",
  "policy_rule": "blocked_commands[2]",
  "audit_id": "log_entry_uuid"
}
```

**Why it matters:** This is the integration point. It's how an agent using LangChain, AutoGen, or any other framework actually uses the runtime. Making this API clean, well-documented, and easy to integrate with is the difference between a project developers adopt and one they ignore. The Python SDK (V2) wraps this API so developers don't have to write HTTP calls manually.

**V1 definition of done:**
- [ ] HTTP server starts reliably on configurable port
- [ ] `POST /execute` correctly routes through policy engine and sandbox
- [ ] `GET /health` returns runtime status
- [ ] `GET /policy` returns current effective policy for agent ID
- [ ] `GET /logs` returns recent entries with basic filtering
- [ ] API key authentication for all endpoints
- [ ] Meaningful error messages for policy violations and sandbox failures
- [ ] Request/response logged to audit log

**Key Rust crates:** `axum` (HTTP framework), `tokio` (async), `serde_json`

---

---

## 11. V2 Scope — Expanding the Platform (Months 5–8)

V2 goal: **make the runtime useful for small engineering teams, not just individual developers.**

This means adding the features that a team of 5–20 engineers deploying multiple agents needs, and shipping the Python SDK that makes integration trivial.

---

### Component 5: Tool Gateway

**What it is:** A layer that adds semantic understanding of specific developer tools, so the policy engine can enforce rules at the tool level, not just the command level.

**What it does:**
- Understands git, pip, npm, docker, pytest as first-class concepts
- Allows policies like "git clone is allowed from github.com only" rather than "git is allowed"
- Intercepts package installation and validates against allowlist
- Blocks known-malicious package names (typosquatting detection)

**Why it matters:** Raw command allow/block lists are brittle. `pip install requests` and `pip install reqests` look similar but one is a supply chain attack. The tool gateway adds semantic understanding that a command allow-list can't.

---

### Component 6: Resource Controller

**What it is:** Enforces CPU, memory, disk, and network bandwidth limits on agent execution.

**What it does:**
- Sets Docker container resource limits per policy
- Kills executions that exceed limits
- Reports resource usage in audit log
- Prevents runaway agents from consuming all host resources

**Why it matters:** An agent writing an infinite loop or downloading a 50GB dataset shouldn't take down the developer's machine or a production server.

---

### Component 7: Identity Manager

**What it is:** Manages agent identities so the runtime knows which agent is making each request and can enforce per-agent policies.

**What it does:**
- Issues agent tokens at registration
- Validates token on every request
- Associates all audit log entries with a specific agent identity
- Supports multiple simultaneous agents with different policies
- Allows revoking an agent's access without restarting the runtime

**Why it matters:** In V1, all agents use the same policy. In V2, different agents have different permissions. A code review bot and a deployment bot should not have the same access level.

**Also in V2:**
- Python SDK (wraps the Agent API for easy integration with AI frameworks)
- Improved CLI tooling
- Policy validation tooling (lint your policy file before deploying)

---

---

## 12. V3 Scope — Production & Fintech Pivot (Months 9–24)

V3 goal: **make the runtime viable for regulated industries, specifically fintech/banking.**

This is where the business becomes real. Individual developers don't pay much. Fintechs pay a lot.

---

### Component 8: Memory Isolation

**What it is:** Prevents agent sessions from contaminating each other's context or persisting malicious injected context across sessions.

**Why it matters in fintech:** A prompt injection attack that poisons one agent's memory should not affect other agents or persist into future sessions. In a banking context, this is the difference between an isolated incident and a systemic compromise.

---

### Component 9: Workflow Controller

**What it is:** Manages multi-step agent workflows where some steps require human approval before proceeding.

**What it does:**
- Defines approval gates in the policy file
- Pauses workflow execution at defined checkpoints
- Sends approval request (webhook, email, or Slack in V3)
- Resumes or cancels based on human response
- Full audit trail of who approved what and when

**Why it matters in fintech:** "Deploy to production requires a human to approve" is a compliance requirement, not a nice-to-have. The workflow controller makes this enforceable and auditable.

---

### Component 10: Firecracker Micro-VM Support

**What it is:** Replaces or supplements Docker containers with Firecracker micro-VMs for execution environments that require stronger isolation.

**Why it matters in fintech:** Banks need stronger isolation guarantees than Docker provides. Firecracker is what AWS Lambda uses for isolation. It provides VM-level isolation at near-container performance. This upgrade makes the runtime credible in enterprise security reviews.

---

### Also in V3:
- Distributed execution across multiple runtime nodes
- Compliance-formatted audit reports (PDF exports, regulatory templates)
- SSO/SAML authentication for enterprise teams
- RBAC (role-based access control)
- Remote log shipping to SIEM systems
- TypeScript control plane dashboard
- Cloud-hosted managed runtime option

---

---

## 13. Tech Stack & Rationale

### Rust — Runtime Core

**Used for:** All four V1 components and all V2/V3 runtime components

**Why Rust specifically:**
- Memory safety without garbage collection — critical when executing untrusted agent code
- A memory safety bug in a security boundary is itself a security vulnerability
- Excellent concurrency model for handling multiple simultaneous agent requests
- Strong ecosystem for systems programming (`tokio`, `axum`, `bollard`)
- Performance: the runtime adds minimal overhead to agent execution
- Growing relevance in infrastructure engineering — strong signal on a resume

**The honest caveat:** Rust has a steep learning curve. Learning Rust while building this project is the plan, not a problem. Build complexity grows alongside Rust competence through deliberate component sequencing.

---

### Python — SDK Layer (V2)

**Used for:** Client library that wraps the Agent API

**Why Python:**
- Every major AI agent framework (LangChain, AutoGen, CrewAI) is Python
- If the SDK isn't Python, adoption will be near zero
- Python is not used in the runtime core — only in the integration layer

---

### TypeScript — Control Plane Dashboard (V3)

**Used for:** Web dashboard for policy management, audit log exploration, team access control

**Why TypeScript:**
- Standard for web dashboards
- Deprioritized until V3 — developers don't need a GUI, enterprises do
- A CLI serves all V1/V2 needs

---

### Docker — Sandbox Primitive (V1–V2)

**Why Docker for V1:**
- Mature, well-documented, battle-tested
- Easy to run locally and in CI
- Large ecosystem — answers to every problem exist
- Sufficient isolation for developer use cases

**Why Firecracker in V3:**
- VM-level isolation required for banking/fintech security reviews
- Powers AWS Lambda — production-proven at scale
- Docker becomes the "fast path," Firecracker becomes the "secure path"

---

### TOML/YAML — Policy Format

**Why TOML:**
- Human-readable and writable
- Git-trackable (policies treated as code)
- Simple enough that a developer can write a policy in 10 minutes
- Rust has excellent TOML support via `serde` + `toml` crates

---

---

## 14. Major Tradeoffs

**Open source core vs. proprietary:**
Chose open source core. Tradeoff: competitors can study the code. Benefit: developer trust and adoption that cannot be bought. Infrastructure companies that try to sell to developers without open source almost always lose. The proprietary layer is the enterprise features (compliance reporting, SSO, Firecracker isolation, SLA support).

**Local-first vs. cloud-first:**
Chose local-first. Tradeoff: harder to monetize in V1 since there's no billing surface. Benefit: zero infrastructure cost, zero latency, zero trust concerns for security-conscious users. Developers will try something they can run locally in 5 minutes. They will not try something that requires a cloud account and payment info before showing value.

**Docker vs. Firecracker from day one:**
Chose Docker first. Tradeoff: weaker isolation than Firecracker. Benefit: dramatically simpler implementation, faster to ship, sufficient for V1 users. Firecracker is saved for V3 where the stronger isolation is a selling point, not just an engineering nicety.

**TOML policies vs. a custom policy language:**
Chose TOML. Tradeoff: less expressive than a Rego-style policy language (like OPA). Benefit: every developer can read and write a TOML policy immediately with no learning curve. Expressive policy language is a V3 feature if users demand it.

**Four V1 components vs. the full 12-component architecture:**
Chose four components. Tradeoff: V1 is not feature-complete. Benefit: V1 ships. The best architecture in the world is worthless if it never ships. The four components chosen for V1 deliver the three core guarantees (enforcement, isolation, auditability) without the complexity that kills solo projects.

---

---

## 15. Competition Landscape

### Direct Competitors (runtime / infrastructure layer)

| Company | What they do | Threat level |
|---|---|---|
| Invariant Labs | Agent behavior monitoring | Medium — observability focus, not enforcement |
| Portkey | LLM gateway with guardrails | Low — API layer, not execution layer |
| LangSmith / LangFuse | Agent observability | Low — they log, we enforce |
| Nemo Guardrails (NVIDIA) | Input/output filtering | Low — different layer |

### Adjacent Threats (could expand into our space)

| Company | Why they're a threat |
|---|---|
| Palo Alto Networks | Acquired Protect AI, building AI security suite |
| Check Point | Acquired Lakera in April 2025 |
| Datadog | Already doing agent observability, could add enforcement |
| AWS / Azure / GCP | Could build native agent runtimes into their AI services |

### Why the space is still winnable

1. All current solutions focus on **observability** (what happened) not **enforcement** (what is allowed)
2. No existing tool is built specifically for the **execution layer** — the space between the agent and the system
3. The incumbents acquiring security startups are not building this — they're acquiring monitoring tools
4. A focused open source runtime with strong developer adoption is a different kind of asset than another monitoring dashboard
5. The window to establish developer mindshare is approximately 18–24 months before cloud providers build this natively

---

---

## 16. Success Criteria

### V1 Success (Month 4)

- [ ] Working runtime: developer can install, configure, and run it in under 15 minutes
- [ ] Passes own security tests: a prompt injection attempt that tries to read `~/.ssh/id_rsa` is blocked and logged
- [ ] First 10 external GitHub stars (signal that at least a few people found it interesting)
- [ ] At least one developer not involved in building it uses it and reports feedback
- [ ] Clean, well-written README that explains the project in under 5 minutes

### V2 Success (Month 8)

- [ ] Python SDK published to PyPI with documentation
- [ ] 100+ GitHub stars
- [ ] Integration example working with at least two agent frameworks (LangChain + one other)
- [ ] First piece of public writing about the project (blog post, LinkedIn article, Hacker News Show HN)
- [ ] At least 5 developers actively using it (usage reported through voluntary telemetry or direct contact)

### V3 Success (Month 18–24)

- [ ] At least one fintech company using it in any capacity (pilot, design partner, or paid)
- [ ] 500+ GitHub stars
- [ ] Acquisition conversation has occurred with at least one strategic buyer
- [ ] Product is technically credible enough to pass a security review at a bank
- [ ] Revenue or a funding conversation has started

---

---

## 17. Roadmap Summary

```
Month 1    Learn Rust fundamentals (chapters 1–6 of The Rust Book)
           Build: Process sandbox prototype (the core primitive)
           LinkedIn account live, GitHub repo created

Month 2    Build: Policy engine (TOML reader + command evaluator)
           First integration: sandbox + policy working together
           First LinkedIn post about what you're building

Month 3    Build: Audit logger (structured JSON, append-only)
           All three components integrated and tested
           Write basic tests for each component

Month 4    Build: Agent API (Axum HTTP server)
           V1 complete: all four components working end-to-end
           Write README + quick start guide
           Post demo to LinkedIn + GitHub

Month 5    Python SDK prototype
           Begin engaging with AI developer communities
           Reach out to 5 developers for feedback

Month 6    Tool Gateway prototype
           First blog post / Show HN post
           Identity Manager design

Month 7    Resource Controller + Identity Manager
           Python SDK published to PyPI
           Growing developer feedback loop

Month 8    V2 complete
           First fintech outreach conversations
           Consider Y Combinator application

Month 9–12 V3 design based on what users actually need
           Fintech design partner conversations
           Firecracker isolation implementation

Month 13–24 Production hardening
            First paying customer
            Acquisition conversations if traction exists
```


---

## 18. Open Questions

These are not blocking decisions but will need to be resolved:

1. **Project name:** `Nera` A real name matters for GitHub discoverability and brand. When to decide: 

2. **License:** MIT vs. Apache 2.0 for the open source core. BUSL (Business Source License) for enterprise features (what HashiCorp used). When to decide: before first GitHub push.

3. **Telemetry:** Should V1 collect anonymous usage telemetry (commands executed, policies loaded, not command content)? Helps understand adoption. Controversial in security tools. Decision needed before first public release.

4. **Policy language evolution:** TOML is V1. Will users need something more expressive (conditional logic, variables, inheritance)? Monitor what users ask for before investing.

5. **MCP integration:** Model Context Protocol (Anthropic's standard for agent tools) is gaining adoption. Should the Agent API speak MCP natively? Could dramatically increase integration surface. Evaluate in V2.

6. **When to start fintech conversations:** The recommendation is month 5-6 — when V1 is working and you have something to show. Not before. Not after month 8.

7. **Solo vs. co-founder:** This project benefits enormously from a systems engineering co-founder or a business co-founder with fintech connections. Worth thinking about after V1 ships.

---

## 19. Strategic Notes

### The Two-Phase Strategy

**Phase 1 (coding agents):** Build credibility, real product, developer adoption. The technical foundation.

**Phase 2 (fintech pivot):** Apply the same architecture to a market that pays 10–100x more and has regulatory urgency. The business model.

The key insight: the core architecture does not change between phases. The sandbox, policy engine, audit logger, and agent API are equally valuable in both contexts. Phase 2 adds compliance formatting, stronger isolation (Firecracker), and memory isolation for PII. It does not rebuild the foundation.

### The Acquisition Logic

Strategic buyers are: Palo Alto Networks, Datadog, ServiceNow, AWS, Azure, any major bank's technology arm, any fintech infrastructure company.

What makes this acquirable:
- Open source community and GitHub reputation (distribution)
- Genuine technical differentiation at the execution layer (technology)
- Design partner relationships with fintech companies (revenue signal)
- A small tight team that built something real (people)

The acquisition does not require massive revenue. It requires being the obvious solution to a problem a large company has and being easier to acquire than to build.

### The Resume/Career Value

Regardless of acquisition outcome, completing V1 of this project demonstrates:
- Systems programming in Rust
- Security engineering (sandboxing, policy enforcement)
- Infrastructure design (runtime architecture)
- API design (Agent API)
- Product thinking (understanding who the user is and what they need)

This profile is directly hireable at: Cloudflare, Datadog, Fly.io, Modal, any AI infrastructure startup, any fintech engineering team. Completing this project changes your career trajectory irrespective of whether it becomes a startup.

---

*This document is a living artifact. Update it when major decisions change. Do not update it when implementation details change — those belong in code and comments.*

*Next document: `BUILD_PLAN_V1.md`*
