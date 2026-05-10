# ADR 0012: Plugin Trust Metadata Comes Before Code Execution

Date: 2026-05-10

Status: Accepted

## Context

Phase 9 introduced declarative plugin manifests. The next risk is allowing
third-party code too early, before signing, package trust, sandboxing, approval
UI, and rollback behavior exist.

## Decision

Phase 10 adds package trust metadata, permission review summaries, sandbox
declarations, and rollback metadata while keeping plugin code execution
disabled.

Signature fields are accepted as metadata, but cryptographic verification is
not claimed yet. The runtime reports trust states explicitly, such as
`signed_metadata_present_unverified`.

## Consequences

- Users and future UI surfaces can inspect plugin permissions before install.
- Lifecycle audit records include plugin trust state.
- Plugin code execution remains blocked until a later sandboxed and verified
  package model exists.
