# huggingOS Agent Runtime

This is the Rust production path for the huggingOS local agent runtime.

The Phase 2 Python CLI remains a reference implementation while the Rust runtime
reaches parity. New production agent, daemon, planner, and desktop integration
work should start here.

## Commands

```bash
cargo run -- status --json
cargo run -- capabilities --json
cargo run -- run product.status --json
cargo run -- run fs.list --param path=.. --json
cargo run -- run notes.create --param title=RustAgent --dry-run --json
cargo run -- ai status --json
cargo run -- ai plan "show product status" --json
cargo run -- ai run "show product status" --json
cargo run -- secrets status --json
cargo test
```

## Phase 3 AI Bridge

The Rust agent owns the production AI bridge.

- `local.rules` is the current offline provider.
- Natural-language prompts become typed capability plans.
- `ai run` executes those plans only through policy, audit, and verification.
- Secret readiness is reported as present/missing and never prints values.
- Cloud/local-model providers are declared for status and failure handling, but
  they are not executable until real provider adapters are added.

## Safety Model

- Typed capabilities only.
- Policy check before execution.
- JSON Lines audit for every action.
- Obvious secret paths are denied by read-only file capabilities.
- Low-risk note creation is workspace-scoped and uses exclusive file creation.
- Capabilities fail closed when audit logging is unavailable.
