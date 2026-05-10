# ADR 0014: Plugin Approval Starts As A Read-Only Surface

Date: 2026-05-10

Status: Accepted

## Context

Plugin packages can now be cryptographically verified, but a production OS
still needs a user-review boundary before plugins gain more power. Building a
graphical overlay before the underlying trust contract exists would risk a UI
that looks complete while hiding missing safety decisions.

## Decision

Phase 12 adds `plugins.approval.surface`, a read-only capability that returns a
stable `huggingos.plugin.approval.v1` payload for source manifests and installed
plugins. The payload combines identity, verified trust state, permissions,
sandbox posture, update metadata, recent rollback manifests, warnings, blocked
reasons, and available confirmed actions.

The approval surface does not install, disable, remove, update, roll back, or
execute plugin code. Those remain separate audited capabilities with their own
confirmation requirements.

## Consequences

- A future desktop overlay can render one stable contract instead of
  reimplementing trust logic.
- Approval recommendations are based on verified package trust and blocked
  unsafe requests.
- The repo avoids fake GUI claims while still moving toward a real desktop UI.
- Sandboxed plugin code execution, signed archive bundles, update feeds, and
  automatic rollback stay deferred.
