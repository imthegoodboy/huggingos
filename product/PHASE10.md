# Product Phase 10: Plugin Trust, Packaging, And Approval UI

Phase 10 hardens the manifest plugin SDK before richer integrations are allowed.
It adds package trust metadata, permission review summaries, sandbox
declarations, and rollback metadata while keeping arbitrary plugin code
execution disabled.

## What Works

- Plugin manifests can include `package`, `ui`, and `sandbox` metadata.
- `plugins.package.validate` validates package trust metadata before install.
- `plugins.permission.review` generates user-facing permission and approval
  summaries from a source manifest or installed plugin.
- `plugins.install` returns trust state, approval details, permission summary,
  and rollback metadata.
- `plugins.disable` and `plugins.remove` return rollback metadata.
- Plugin lifecycle and run audit records include `plugin_trust_state`.
- The sample plugin declares signed metadata shape and a disabled sandbox.

## Trust Model

Phase 10 validates package metadata shape only. It does not cryptographically
verify signatures yet. Trust states include:

- `signed_metadata_present_unverified`
- `package_metadata_unsigned`
- `manifest_only_unsigned`

This is intentionally explicit so future UI and packaging work can show the
truth instead of implying verified signatures too early.

## Safety Rules

- Plugin code execution remains disabled.
- Plugin network access remains disabled.
- Plugin install, disable, and remove still require confirmation.
- Permission review is read-only and can run before install.
- Sandboxing metadata must declare `code_execution = disabled` for now.

## Commands

From `product/agent/`:

```bash
cargo run -- run plugins.package.validate --param source=../plugins/hello-assistant --json
cargo run -- run plugins.permission.review --param source=../plugins/hello-assistant --json
cargo run -- run plugins.install --param source=../plugins/hello-assistant --param force=true --confirm --json
```

From the repository root:

```bash
make product-agent-plugin-package-validate
make product-agent-plugin-permission-review
make product-agent-plugin-install
```

## Still Later

- Real cryptographic signature verification. Completed in
  [PHASE11.md](PHASE11.md) for local manifest packages.
- Signed plugin package archives.
- Plugin update channels.
- Desktop overlay approval UI.
- Sandboxed plugin-provided code execution.
