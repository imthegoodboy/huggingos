# Product Config

`defaults.toml` contains non-secret product defaults used by the product CLI and
capability layer.
Real API keys, provider tokens, and local machine configuration must not be
committed.

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
