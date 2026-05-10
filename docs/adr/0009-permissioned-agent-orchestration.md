# ADR 0009: Agents Delegate Through Existing Capabilities

Date: 2026-05-10

Status: Accepted

## Context

Product Phase 7 needs multiple agents, but separate agents must not become a
new way to bypass policy, confirmations, or audit logs.

## Decision

Represent built-in agents as manifests with explicit allowed capabilities.
The orchestrator creates deterministic delegation plans and executes each step
through the existing capability registry, policy engine, verifier, and audit
log. Agent traces are stored locally as JSON Lines.

## Consequences

- Agents are useful immediately without a daemon or cloud model.
- Per-agent permissions are testable.
- Orchestration can delegate to multiple agents while preserving auditability.
- Plugin agents, parallel execution, and richer approval UI remain later work.
