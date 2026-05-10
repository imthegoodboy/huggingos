# Product Phase 11 Plugin Signature Notes

## Context

Product Phase 11 makes plugin package trust cryptographically enforced for the
local manifest package format.

## What Future Agents Should Know

- `plugins.package.validate` and `plugins.install` require
  `signature_verified` for `huggingos.plugin.package.v1`.
- The digest and Ed25519 signature cover canonical manifest JSON with
  `package.sha256` cleared and `package.signature` removed.
- Editing the sample plugin manifest requires regenerating both `package.sha256`
  and `package.signature`.
- `update.auto_update` must remain false until an approval flow exists.
- Rollback manifests are persisted recovery records only; do not claim automatic
  rollback until execution exists.
- Plugin code execution is still disabled even for verified packages.

## Evidence

- Source: `product/agent/src/main.rs`
- Sample package: `product/plugins/hello-assistant/plugin.json`
- Tests: `cargo test` in `product/agent`
