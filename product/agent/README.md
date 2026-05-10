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
cargo run -- run memory.session.remember --param key=current-goal --param value=phase-six-seven --json
cargo run -- run files.semantic.index --param root=../../docs --confirm --json
cargo run -- run files.semantic.search --param query=capability --json
cargo run -- run agents.catalog --json
cargo run -- run agents.orchestrate --param "goal=daily brief" --confirm --json
cargo run -- run proactive.workflow.detect --json
cargo run -- run proactive.suggest --json
cargo run -- run selfheal.diagnose --param symptom=app_crashed --param target=editor --param simulated=true --json
cargo run -- run timeline.explain --json
cargo run -- run plugins.validate --param source=../plugins/hello-assistant --json
cargo run -- run plugins.package.validate --param source=../plugins/hello-assistant --json
cargo run -- run plugins.permission.review --param source=../plugins/hello-assistant --json
cargo run -- run plugins.install --param source=../plugins/hello-assistant --param force=true --confirm --json
cargo run -- run plugins.catalog --json
cargo run -- run plugins.workflow.plan --param plugin_id=sample.hello --json
cargo run -- run plugins.capability.run --param plugin_id=sample.hello --param capability=hello --json
cargo run -- run plugins.disable --param plugin_id=sample.hello --confirm --json
cargo run -- run plugins.remove --param plugin_id=sample.hello --confirm --json
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

## Phase 6 Memory And Semantic Files

Memory and file search are local capabilities:

- `memory.session.remember` / `memory.session.list` for short-term facts.
- `memory.preference.set` / `memory.preference.list` for preferences.
- `memory.event.list` for audit-derived event history.
- `files.semantic.index` / `files.semantic.search` for opt-in local text search.
- `workspace.resume.plan` for a memory-backed resume plan.
- `memory.export` and `memory.delete` for user control.

The semantic index is `local.token_overlap.v1`; it is not cloud embeddings.

## Phase 7 Multi-Agent Orchestration

Agents are permissioned manifests over existing capabilities:

- `agents.catalog` lists built-in agents.
- `agents.plan` previews deterministic delegation.
- `agents.orchestrate` runs delegated capabilities after confirmation.
- `agents.trace.list` shows replayable local traces.

Agents cannot call capabilities outside their catalog permissions.

## Phase 8 Predictive And Self-Healing OS

Predictive and healing features are suggestion-only capabilities:

- `proactive.workflow.detect` finds repeated audited workflows.
- `proactive.suggest` builds safe proactive recommendations.
- `selfheal.diagnose` diagnoses simulated or reported recoverable failures.
- `timeline.explain` summarizes recent activity from audit, memory, and traces.

These capabilities do not launch apps, restart services, delete files, or run
cleanup. They return recommended next capability steps, which must still pass
policy and confirmation.

## Phase 9 Plugin SDK

Plugins are declarative manifests in this slice:

- `plugins.validate` validates a local manifest.
- `plugins.install` installs a manifest after confirmation.
- `plugins.catalog` lists installed plugin capabilities and workflows.
- `plugins.workflow.plan` previews plugin workflows.
- `plugins.capability.run` runs read-only declarative plugin capabilities.
- `plugins.disable` and `plugins.remove` manage installed plugins after
  confirmation.

The sample plugin lives at `product/plugins/hello-assistant/plugin.json`.
Arbitrary plugin code execution is not enabled yet.

## Phase 10 Plugin Trust

Plugin trust and approval metadata are explicit:

- `plugins.package.validate` validates package metadata shape before install.
- `plugins.permission.review` returns a user-facing permission and approval
  summary.
- Plugin lifecycle actions include rollback metadata.
- Audit records include plugin trust state.

Signature metadata can be present, but signatures are not cryptographically
verified yet. The runtime reports that as `signed_metadata_present_unverified`.

## Safety Model

- Typed capabilities only.
- Policy check before execution.
- JSON Lines audit for every action.
- Obvious secret paths are denied by read-only file capabilities.
- Low-risk note creation is workspace-scoped and uses exclusive file creation.
- Screen/context capabilities redact private active-window data and deny capture
  for private contexts.
- Memory collection is opt-in and deletable.
- Agents delegate only through capability permissions, policy, audit, and traces.
- Predictive and self-healing features are suggestion-first and read-only.
- Plugins are manifest-only and read-only until sandboxing and signing exist.
- Plugin signatures are metadata-only until real verification exists.
- Capabilities fail closed when audit logging is unavailable.
