# Product Phase 12: Plugin Approval Surface

Phase 12 makes plugin trust decisions renderable by a future desktop control
center without pretending the graphical overlay exists yet.

## What Works

- `plugins.approval.surface` returns a stable
  `huggingos.plugin.approval.v1` payload.
- The surface works for source manifests before install and installed plugins
  after install.
- The payload includes identity, verified trust state, permissions, sandbox
  posture, update metadata, recent rollback manifests, warnings, blocked
  reasons, and confirmed next actions.
- The local planner maps plugin approval prompts to the read-only capability.
- The plugin agent can inspect approval surfaces through its allowlist.

## Safety Rules

- Approval surfaces are read-only.
- Install, disable, remove, and run remain separate audited capabilities.
- Plugin code execution remains disabled.
- Plugin auto-update remains disabled.
- Rollback manifests are visible but automatic rollback execution is not
  implemented.
- The desktop overlay renderer is not implemented yet; this phase provides the
  data contract it should consume.

## Commands

From `product/agent/`:

```bash
cargo run -- run plugins.approval.surface --param source=../plugins/hello-assistant --json
cargo run -- ai plan "plugin approval sample.hello" --json
```

From the repository root:

```bash
make product-agent-plugin-approval-surface
```

## Still Later

- Rendered desktop approval screens.
- Sandboxed plugin-provided code execution.
- Signed plugin archive bundles.
- Trusted plugin update feeds.
- Automatic rollback execution.
