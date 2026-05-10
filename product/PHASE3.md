# Product Phase 3: AI Runtime Bridge And Secrets

Phase 3 connects natural-language intent to the real capability control plane in
the Rust production agent.

This is intentionally not a fake chatbot layer. The local AI bridge can plan and
execute only typed capabilities that already pass policy, audit, and verifier
checks.

## Implemented Scope

- Provider-neutral AI runtime status model.
- Deterministic offline planner through `local.rules`.
- Redacted AI secret readiness checks.
- Plan-execute-verify flow through the existing Rust capability engine.
- Structured JSON output for automation and CI.
- Safe provider failure behavior for unavailable cloud/local-model adapters.

## Commands

From `product/agent/`:

```bash
cargo run -- ai status --json
cargo run -- ai plan "show product status" --json
cargo run -- ai run "show product status" --json
cargo run -- secrets status --json
```

Supported local planning intents:

- Product, system, or agent status -> `product.status`
- List files or directories -> `fs.list`
- Read a small text file -> `fs.read_text`
- Create a safe workspace note -> `notes.create`
- Show recent audit records -> `audit.list`

Unknown prompts return a non-executable plan instead of pretending success.

## Secret Model

The agent reports whether configured provider readiness signals exist, but never
prints secret values.

Default environment names:

- `HUGGINGOS_OPENAI_API_KEY`
- `HUGGINGOS_ANTHROPIC_API_KEY`
- `HUGGINGOS_LOCAL_MODEL_ENDPOINT`

Cloud providers are declared but not executable in this build. That is deliberate:
outbound model calls should be added only after provider clients, retry behavior,
budget controls, and user consent policies are implemented.

## Safety Rules

- Models and planners propose capability calls; they do not mutate the OS
  directly.
- Every executed or denied action still goes through policy and audit.
- File reads still deny obvious secret paths such as `.env`, `.ssh`, tokens, and
  private keys.
- Offline local control must continue working when cloud providers are missing
  or disabled.

## Validation

```bash
cd product/agent
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo run -- ai status --json
cargo run -- ai plan "show product status" --json
cargo run -- ai run "show product status" --json
cargo run -- secrets status --json
```
