# ADR 0003: Capability Control Plane

Status: accepted

Date: 2026-05-10

## Context

Product Phase 2 is the first phase where huggingOS starts to control the host
through AI-ready actions. The user wants agents to have broad control over apps,
files, shell commands, browser workflows, settings, and later the desktop. That
control must be real, fast, auditable, and safe.

Direct arbitrary shell execution would be quick to implement, but it would make
permissions, rollback, verification, and user trust hard to add later. A better
foundation is a capability control plane: every agent action becomes a typed
request that passes through policy, execution, verification, and audit.

## Decision

Build Product Phase 2 around a local capability control plane.

Required flow:

```text
User or agent intent
  -> planner
  -> capability registry
  -> policy decision
  -> executor
  -> verifier
  -> audit log
  -> result
```

The Phase 2 implementation should start in-process inside the CLI for speed and
simplicity. The same interfaces should be shaped so they can move behind a local
user service in a later phase without changing the action schema.

## Core Contracts

Capability:

- Stable name, version, owner, description, risk level, and permissions.
- Input schema and result schema.
- Executor that performs one bounded action.
- Verifier that checks observable result state where practical.
- Rollback metadata when the action can be reversed.

Action request:

- `action_id`
- `capability`
- `params`
- `actor`
- `reason`
- `requested_at`
- `dry_run`

Action result:

- `action_id`
- `status`
- `started_at`
- `finished_at`
- `summary`
- `data`
- `error`
- `verification`
- `audit_ref`

Risk levels:

- `read`: observes state only.
- `low`: creates or changes non-sensitive user-owned state.
- `medium`: modifies files, launches apps, or runs constrained commands.
- `high`: deletes, overwrites, changes settings, uses network, touches secrets,
  or runs broad shell commands.

## Phase 2 Scope

Start with read-only and low-risk local capabilities:

- Product status.
- List a directory with path checks.
- Read a small text file with size limits.
- Create a note in a configured workspace directory.
- Show audit log entries.

Do not start with:

- Unrestricted shell execution.
- Browser automation.
- Secret access.
- System settings changes.
- App automation that depends on desktop APIs not yet modeled.

## Consequences

Easier:

- Agents can gain broad OS control by composing explicit capabilities.
- Every action has permissions, audit, and later rollback.
- The CLI, service, UI, and AI runtime can share one action schema.
- Tests can validate actions without requiring a full desktop session.

Harder:

- Initial development is more structured than direct scripts.
- Every capability needs metadata, policy, tests, and honest failure modes.
- Some "do anything" behavior waits until the capability exists.

## Validation

Product Phase 2 is ready only when:

- A CLI command can list registered capabilities.
- Read-only capabilities execute through the registry and policy engine.
- Low-risk write capability requires a configured safe workspace.
- Every action writes an audit record.
- Tests cover allow, deny, dry-run, execution, verification, and audit paths.
