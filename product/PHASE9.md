# Product Phase 9: Plugin SDK And Ecosystem

Phase 9 adds the first plugin contract for huggingOS. Plugins are declarative
manifests in this slice: they can declare read-only capabilities, workflows, and
agent allowlists, but they cannot execute arbitrary native code.

## What Works

- `plugins.validate` validates a local `plugin.json` manifest.
- `plugins.install` installs a plugin manifest into the local state directory
  after confirmation.
- `plugins.catalog` lists installed plugins, capabilities, workflows, agents,
  permissions, and enabled state.
- `plugins.workflow.plan` returns a plugin-provided workflow plan without
  executing it.
- `plugins.capability.run` runs a declarative read-only plugin capability.
- `plugins.disable` disables a plugin without deleting its manifest.
- `plugins.remove` removes an installed plugin from local state after
  confirmation.
- Audit records include `plugin_identity` for plugin capability runs and plugin
  lifecycle actions.
- `product/plugins/hello-assistant/plugin.json` is the sample third-party
  plugin used by tests and CI.

## Safety Rules

- Phase 9 plugins are manifest-only.
- Plugin capabilities must be declarative and read-only.
- Plugin install, disable, and remove are medium-risk and require confirmation.
- Plugin workflow planning does not execute steps.
- Plugin capabilities run through the same capability policy, verifier, and
  audit log as built-in capabilities.
- Native plugin code, network plugin downloads, and plugin-provided daemons are
  deferred until sandboxing and signature checks exist.

## Commands

From `product/agent/`:

```bash
cargo run -- run plugins.validate --param source=../plugins/hello-assistant --json
cargo run -- run plugins.install --param source=../plugins/hello-assistant --param force=true --confirm --json
cargo run -- run plugins.catalog --json
cargo run -- run plugins.workflow.plan --param plugin_id=sample.hello --json
cargo run -- run plugins.capability.run --param plugin_id=sample.hello --param capability=hello --json
cargo run -- run plugins.disable --param plugin_id=sample.hello --confirm --json
cargo run -- run plugins.remove --param plugin_id=sample.hello --confirm --json
cargo run -- ai plan "list plugins" --json
```

From the repository root:

```bash
make product-agent-plugin-validate
make product-agent-plugin-install
make product-agent-plugin-catalog
make product-agent-plugin-workflow
make product-agent-plugin-run
make product-agent-plugin-disable
make product-agent-plugin-remove
```

## Still Later

- Signed plugin packages.
- Sandboxed plugin code execution.
- Plugin download/update registry.
- Plugin UI metadata surfaces.
- Plugin-provided long-running agents and services.
