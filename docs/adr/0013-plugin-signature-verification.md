# ADR 0013: Plugin Packages Must Verify Before Install

Date: 2026-05-10

Status: Accepted

## Context

Phase 10 made plugin trust metadata visible, but metadata alone is not enough to
support a plugin ecosystem. The runtime needs a real cryptographic boundary
before plugin power grows beyond declarative read-only responses.

## Decision

Phase 11 requires local plugin packages to verify before install. The first
package format is `huggingos.plugin.package.v1`, where the manifest is the
package payload. The verifier canonicalizes the manifest by clearing
`package.sha256` and omitting `package.signature`, computes SHA-256 over that
canonical JSON, and verifies an Ed25519 signature using
`ed25519-canonical-json-sha256-v1`.

`plugins.package.validate` and `plugins.install` require `signature_verified`.
Manifest inspection can still report unverified trust states, but install fails
closed for unsigned or tampered packages. Update metadata is allowed only with
`auto_update = false`, and rollback manifests are written for lifecycle
operations.

## Consequences

- Plugin install no longer depends on trust claims alone.
- The sample plugin proves the signed-package path end to end.
- Future archive formats can reuse the digest/signature policy while replacing
  the payload canonicalization.
- Plugin code execution remains blocked until sandboxing and approval UI exist.
