# Product Phase 11: Plugin Signature Verification

Phase 11 turns plugin package trust from honest metadata into an enforced local
cryptographic check. Plugin manifests can still only provide declarative,
read-only behavior, but install now requires a verified package signature.

## What Works

- `plugins.package.validate` verifies the sample package digest and signature.
- `plugins.install` requires verified package trust before writing an installed
  plugin manifest.
- The local signed package format is `huggingos.plugin.package.v1`.
- The signature algorithm is `ed25519-canonical-json-sha256-v1`.
- Package update metadata can declare `local`, `stable`, `beta`, or `dev`
  channels, but `auto_update` must stay false.
- Install, disable, and remove write rollback manifests under the local state
  directory.
- Tampered plugin manifests fail closed with a digest or signature error.

## Signed Package Format

The current package is the plugin manifest itself, not an executable archive.
The verifier builds canonical JSON from the manifest after clearing
`package.sha256` and removing `package.signature`, then:

1. Computes SHA-256 over the canonical JSON.
2. Compares it with `package.sha256`.
3. Verifies the Ed25519 signature over the same canonical JSON.
4. Reports `signature_verified` only when both checks pass.

This keeps the first implementation small and testable while preserving a clean
path to future archive formats.

## Safety Rules

- Plugin code execution remains disabled.
- Plugin network access remains disabled.
- Unsigned or tampered packages cannot be installed.
- Plugin install, disable, and remove require confirmation.
- Rollback manifests are local recovery records, not automatic undo.
- Auto-update is rejected until update approvals and rollback execution exist.

## Commands

From `product/agent/`:

```bash
cargo run -- run plugins.package.validate --param source=../plugins/hello-assistant --json
cargo run -- run plugins.install --param source=../plugins/hello-assistant --param force=true --confirm --json
cargo run -- run plugins.permission.review --param source=../plugins/hello-assistant --json
```

From the repository root:

```bash
make product-agent-plugin-package-validate
make product-agent-plugin-install
make product-agent-plugin-permission-review
```

## Still Later

- Signed plugin archive bundles beyond manifest-only packages.
- Desktop approval UI data contract for trust, permissions, and updates.
  Completed in [PHASE12.md](PHASE12.md) as a read-only approval surface.
- Sandboxed plugin-provided code execution.
- Trusted marketplace/update feeds.
- Automatic rollback execution.
