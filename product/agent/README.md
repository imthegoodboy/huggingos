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
cargo test
```

## Safety Model

- Typed capabilities only.
- Policy check before execution.
- JSON Lines audit for every action.
- Obvious secret paths are denied by read-only file capabilities.
- Low-risk note creation is workspace-scoped and uses exclusive file creation.
- Capabilities fail closed when audit logging is unavailable.
