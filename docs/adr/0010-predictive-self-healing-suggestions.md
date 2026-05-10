# ADR 0010: Predictive Help Stays Suggestion-First

Date: 2026-05-10

Status: Accepted

## Context

Product Phase 8 needs proactive and self-healing behavior, but silent OS actions
would be risky before rollback, daemon controls, resource telemetry, and UI
approval flows are mature.

## Decision

Phase 8 predictive and self-healing behavior is implemented as read-only
capabilities that inspect local audit history and return recommended next
actions. The recommendations reference existing capabilities, but those
capabilities must still pass their own policy, audit, and confirmation checks.

Repeated workflow detection uses the local audit log. Failure diagnosis supports
simulated and user-reported symptoms so CI can verify behavior without touching
real services.

## Consequences

- huggingOS can now explain repeated workflows and recommend recoveries.
- No background fixer runs without the user choosing an action.
- Future daemon and UI work can reuse the same recommendation contract.
