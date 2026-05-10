# ADR 0011: Plugins Start As Declarative Manifests

Date: 2026-05-10

Status: Accepted

## Context

Product Phase 9 needs third-party extension points, but executing arbitrary
plugin code would require sandboxing, signatures, package trust, update policy,
and rollback controls.

## Decision

The first plugin SDK uses versioned JSON manifests with declarative
capabilities, workflows, permissions, and optional agent allowlists. Phase 9
executes only declarative read-only plugin capabilities through the normal
capability registry, policy, verifier, and audit log.

Install, disable, and remove are medium-risk capabilities that require
confirmation. Plugin identity is written into audit records through the
`plugin_identity` field.

## Consequences

- A sample plugin can add a capability and workflow today.
- Plugin lifecycle behavior is testable in CI without unsafe native code.
- Future plugin code execution can build on the manifest contract instead of
  replacing it.
