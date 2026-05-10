# Product Config

`defaults.toml` contains non-secret product defaults used by the product CLI and
capability layer.
Real API keys, provider tokens, and local machine configuration must not be
committed.

Phase 5 also reads non-secret privacy defaults from this config. Privacy markers
are policy hints, not secrets. Keep user-specific exclusions in ignored local
override files when they reveal private app names, folder names, or workflows.

Phase 6 through Phase 12 store local memory, semantic indexes, agent traces,
predictive/self-healing audit-derived state, installed plugin manifests, plugin
trust/lifecycle state, and plugin rollback manifests under the configured state
directory. Those runtime files are ignored and should not be committed.

Allowed tracked files:

- Non-secret defaults.
- Example files such as `.env.example`.
- Schemas or docs that describe how config should be loaded.

Ignored local files:

- `.env` and `.env.*`
- `product/runtime/`
- `product/secrets/`
- `product/config/*.local.*`
- Config filenames containing `secret`, `token`, or `key`

The CLI supports these local override environment variables:

- `HUGGINGOS_CONFIG_FILE`: load an alternate non-secret config file.
- `HUGGINGOS_STATE_DIR`: move local runtime state and the audit log.
- `HUGGINGOS_WORKSPACE_DIR`: constrain low-risk workspace writes.

Keep override files outside Git or in ignored local paths.
