# Product Phase 6: Memory And Semantic Files

Phase 6 adds local, user-controlled memory to the Rust agent. It is not cloud
memory and it does not silently collect files. Every write goes through a typed
capability, policy, verification, and audit.

## Implemented Scope

- `memory.session.remember` stores short-term session facts.
- `memory.session.list` inspects session facts with optional query/limit.
- `memory.preference.set` stores local user preferences.
- `memory.preference.list` inspects preferences.
- `memory.delete` deletes session memory, preferences, semantic indexes, traces,
  or all local memory after confirmation.
- `memory.export` exports inspectable local memory state.
- `memory.event.list` derives recent event memory from the audit log.
- `files.semantic.index` builds an opt-in local token index over approved text
  roots.
- `files.semantic.search` searches that local index.
- `workspace.resume.plan` builds a plan from recent audit events, session
  memory, preferences, and index presence.

## Commands

From `product/agent/`:

```bash
cargo run -- run memory.session.remember --param key=current-goal --param value=phase-six-seven --json
cargo run -- run memory.session.list --json
cargo run -- run memory.preference.set --param key=theme --param value=dark --json
cargo run -- run memory.preference.list --json
cargo run -- run memory.event.list --json
cargo run -- run files.semantic.index --param root=../../docs --confirm --json
cargo run -- run files.semantic.search --param query=capability --json
cargo run -- run workspace.resume.plan --json
cargo run -- run memory.export --json
cargo run -- run memory.delete --param scope=session --confirm --json
```

## Safety Model

- Memory is local under the configured state directory.
- File indexing is opt-in by root path and requires confirmation.
- Hidden files and obvious secret paths such as `.env`, `.env.local`, `.ssh`,
  credentials, tokens, and private keys are skipped or denied.
- Secret-like memory/preference keys such as API keys, tokens, passwords, and
  credentials are denied.
- The semantic index uses deterministic local token overlap. It does not claim
  cloud embeddings or external vector search.
- Delete/export/list paths are explicit capabilities so users can inspect and
  remove remembered data.

## What Is Still Later

- Real embedding providers.
- SQLite/vector extension storage.
- Per-folder retention policies.
- Private mode toggle in a long-running daemon.
- UI for memory inspection and deletion.

## Validation

```bash
cd product/agent
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo run -- run memory.session.remember --param key=current-goal --param value=phase-six-seven --json
cargo run -- run files.semantic.index --param root=../../docs --confirm --json
cargo run -- run files.semantic.search --param query=capability --json
cargo run -- run workspace.resume.plan --json
```
