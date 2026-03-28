# Nera

Nera is a secure execution runtime for autonomous coding agents.

It provides a controlled environment where AI agents can run commands safely, with clear limits, isolation, and full logging.


## Overview

AI coding agents today typically run like this:

* LLM generates output
* A script executes that output
* The script calls system commands
* The commands run directly on your machine

This creates real risks:

* access to sensitive files (API keys, SSH keys, configs)
* installation of unsafe dependencies
* unintended code changes or deletions
* unrestricted network calls

There is no standard layer that enforces control before execution.

Nera introduces that missing layer.


## What Nera Does

Nera sits between the agent and the system.

* The agent sends a request to Nera
* Nera checks whether the action is allowed
* If allowed, the action runs in an isolated environment
* Every action is recorded

Core guarantees:

* enforcement

  * agents can only perform actions defined in a policy

* isolation

  * all execution happens in a sandbox, not on the host

* auditability

  * every action is logged with full context


## Example

If an agent attempts:

* deleting system files → blocked
* running tests → allowed and executed in a sandbox


## Current Scope (V1)

The initial version focuses on a minimal, working runtime for coding agents.

Components:

* execution sandbox

  * runs commands inside Docker containers

* policy engine

  * defines what is allowed or blocked
  * uses a simple config file (TOML)

* audit logger

  * records all actions and decisions

* agent API

  * HTTP interface used by agents


## Example Policy

[agent.default]
allowed_commands = ["pytest", "git diff"]
blocked_commands = ["rm", "curl", "wget"]
allowed_dirs = ["/workspace"]
network_access = false


## What Nera Is Not

* not an AI agent framework
* not a monitoring-only tool
* not a replacement for Docker
* not an operating system

Nera is a runtime layer that works with existing tools.


## Tech Stack

* Rust

  * runtime core, policy engine, sandbox control

* Python (planned)

  * SDK for integrating with agent frameworks

* TypeScript (future)

  * control dashboard for teams

* Docker

  * sandbox environment for execution


## Project Status

Early development.

Current focus:

* command interception
* policy enforcement
* sandbox execution
* audit logging


## Documentation

Full project design and architecture:

* docs/PROJECT_BLUEPRINT.md


## Goal

Make it possible to run autonomous agents on real systems without giving them full control.


## License

