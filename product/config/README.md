# Product Config

`defaults.toml` contains non-secret product defaults used by the Phase 1 CLI.
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

The CLI supports `HUGGINGOS_CONFIG_FILE` for testing or local overrides. Keep
override files outside Git or in ignored local paths.
