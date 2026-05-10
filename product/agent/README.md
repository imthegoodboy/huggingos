# huggingOS Agent Runtime

This is the Rust production path for the huggingOS local agent runtime.

The Phase 2 Python CLI remains a reference control surface. New production
agent, daemon, planner, and desktop integration work should start here.

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
cargo run -- run desktop.status --json
cargo run -- run apps.list --json
cargo run -- run browser.open_url --param url=https://example.com --dry-run --json
cargo run -- run workspace.mode.plan --param mode=coding --json
cargo run -- run screen.status --json
cargo run -- run screen.capture --dry-run --json
cargo run -- run context.snapshot --confirm --json
cargo run -- run screen.ocr_image --param path=../../README.md --dry-run --json
cargo run -- ai plan "what is open" --json
cargo run -- ai run "what is open" --confirm --json
cargo run -- ai plan "take a screenshot" --json
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

## Phase 4 Desktop Bridge

Desktop and browser actions are also capabilities:

- `desktop.status` detects graphical-session and backend readiness.
- `apps.list` reads installed `.desktop` entries.
- `apps.launch` launches by safe desktop ID and requires confirmation.
- `browser.open_url` opens HTTP/HTTPS URLs and requires confirmation.
- `workspace.mode.plan` previews mode plans before window management exists.

Headless CI and WSL should use `--dry-run` for mutating desktop actions.

## Phase 5 Screen And Context Engine

Screen and active-context observation are also capabilities:

- `screen.status` reports desktop, capture, OCR, context, clipboard, and privacy
  readiness.
- `screen.capture` captures a screenshot to the safe workspace after
  confirmation and privacy checks.
- `context.snapshot` reports active-window metadata and system context after
  confirmation.
- `screen.ocr_image` runs OCR through `tesseract` after confirmation.

Headless CI and WSL should use `screen.status` plus dry runs. Confirmed capture
requires a supported capture backend and active-context backend so private
windows can be blocked before capture.

## Safety Model

- Typed capabilities only.
- Policy check before execution.
- JSON Lines audit for every action.
- Obvious secret paths are denied by read-only file capabilities.
- Low-risk note creation is workspace-scoped and uses exclusive file creation.
- Screen/context capabilities redact private active-window data and deny capture
  for private contexts.
- Capabilities fail closed when audit logging is unavailable.
