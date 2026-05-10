# Product Phase 7: Multi-Agent Orchestration

Phase 7 adds the first local multi-agent architecture. Agents are permissioned
views over existing capabilities; they do not get unchecked shell access or a
separate bypass around policy.

## Implemented Scope

- `agents.catalog` lists built-in agents and allowed capabilities.
- `agents.plan` creates deterministic delegation plans for goals.
- `agents.orchestrate` executes permitted delegated capabilities through the
  normal policy, audit, and verification path.
- `agents.trace.list` lists replayable orchestration traces.
- Local AI planner mappings for agent catalog, orchestration, delegation, daily
  brief, and resume-workspace prompts.

## Built-In Agents

- `system.agent`: product, desktop, and screen readiness.
- `memory.agent`: session memory, preferences, events, export, and resume plan.
- `file.agent`: file listing, text reads, and semantic search.
- `desktop.agent`: app listing, workspace mode plans, and active context
  snapshots.
- `writer.agent`: safe workspace note creation.

Each agent can call only its listed capabilities. The orchestrator validates
every delegated step before execution.

## Commands

From `product/agent/`:

```bash
cargo run -- run agents.catalog --json
cargo run -- run agents.plan --param "goal=daily brief" --json
cargo run -- run agents.orchestrate --param "goal=daily brief" --confirm --json
cargo run -- run agents.trace.list --json
cargo run -- ai plan "daily brief" --json
```

`agents.orchestrate` is medium-risk because it can trigger delegated
medium-risk capabilities such as `context.snapshot`. It requires confirmation.

## What Is Still Later

- Plugin-provided agents.
- Long-running orchestrator service.
- Parallel delegation runtime.
- Human approval UI per delegated step.
- Rich trace viewer.

## Validation

```bash
cd product/agent
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo run -- run agents.catalog --json
cargo run -- run agents.plan --param "goal=daily brief" --json
cargo run -- run agents.orchestrate --param "goal=daily brief" --confirm --json
cargo run -- run agents.trace.list --json
```
