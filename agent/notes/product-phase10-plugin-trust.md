# Product Phase 10 Plugin Trust Notes

Date: 2026-05-10

Phase 10 validates plugin package trust metadata shape but does not verify
signatures cryptographically. Keep the wording honest:

- `signed_metadata_present_unverified` means signature fields are present.
- It does not mean the signature has been verified.
- Plugin code execution remains disabled.

Phase 11 supersedes the signature-warning part of this note for current code:
local plugin packages now verify as `signature_verified` before install. The
Phase 10 warning still matters when reading old PRs, old docs, or old installed
state.

Implemented surfaces:

- `plugins.package.validate`
- `plugins.permission.review`
- Trust state in plugin lifecycle/run audit records.
- Rollback metadata for install, disable, and remove.

Future agents should add sandboxing, approval UI, and rollback execution before
allowing plugin-provided code, daemons, network access, or filesystem writes.
