# ADR 0005: AI Runtime Bridge Uses Capability Plans

Date: 2026-05-10

Status: Accepted

## Context

Product Phase 3 needs AI planning without creating a second, unsafe automation
surface. Phase 2 already provides typed capabilities, policy, audit, and
verification. The AI layer should use that surface instead of directly clicking,
running shell commands, or mutating files.

## Decision

Build the production AI runtime bridge in the Rust agent under
`product/agent/`.

The runtime exposes:

- `ai status` for provider and offline-mode readiness.
- `ai plan` for prompt-to-capability planning.
- `ai run` for executing plans through the capability engine.
- `secrets status` for redacted provider readiness checks.

The first executable provider is `local.rules`, a deterministic offline planner.
Cloud and local-model providers are declared in status output but are not
executable until real adapters, budgets, retries, consent, and secret storage are
implemented.

## Consequences

- AI actions inherit policy, audit, and verification by construction.
- Offline operation works without API keys or cloud services.
- Unknown prompts fail visibly instead of pretending to act.
- Future providers must emit capability plans; they must not mutate the OS
  directly.
