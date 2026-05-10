# ADR 0008: Memory And Semantic Files Stay Local And User-Controlled

Date: 2026-05-10

Status: Accepted

## Context

Product Phase 6 needs memory and semantic file search, but silent collection or
fake cloud-backed memory would break the trust model established by the
capability control plane.

## Decision

Implement memory as local Rust agent capabilities:

- session facts in JSON Lines
- preferences in a local JSON object
- event memory derived from the existing audit log
- opt-in file indexing by user-provided root
- local token-overlap semantic search
- explicit export and delete capabilities

Do not claim embeddings until an embedding provider, retention model, deletion
model, and storage backend are implemented. The Phase 6 index reports its engine
as `local.token_overlap.v1`.

## Consequences

- Memory works offline and without secrets.
- Users can inspect and delete remembered data.
- File indexing requires confirmation and skips hidden/secret-like paths.
- Search quality is useful but lexical; richer semantic embeddings remain
  future work.
