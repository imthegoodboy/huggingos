# Product Phase 9 Plugin Notes

Date: 2026-05-10

Phase 9 plugins are declarative manifests only. Do not add arbitrary plugin code
execution, network plugin installs, or background plugin daemons until the repo
has sandboxing, signatures, package trust, and rollback rules.

Implemented surfaces:

- `plugins.validate`
- `plugins.install`
- `plugins.catalog`
- `plugins.workflow.plan`
- `plugins.capability.run`
- `plugins.disable`
- `plugins.remove`

Sample plugin:

- `product/plugins/hello-assistant/plugin.json`

Plugin identity is audited through `plugin_identity`. Keep that field present
for future plugin lifecycle and plugin capability actions.
