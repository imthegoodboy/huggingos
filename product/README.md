# huggingOS Product Track

This directory is for the Linux-kernel-based huggingOS product.

The product track is the path toward a normal working AI-native OS:

- Linux kernel and drivers.
- Linux userspace services.
- Real filesystems, networking, process isolation, and desktop APIs.
- AI command center and capability API.
- Secure runtime config and secret handling.
- Desktop overlay, app control, memory, and agents.

The existing `kernel/` directory remains the custom kernel-lab track. Do not add
product AI features to the hobby kernel unless an issue explicitly says to work
on the kernel-lab track.

For the Product Phase 1 kickoff sequence, see [PHASE1.md](PHASE1.md).
For the Product Phase 2 capability layer, see [PHASE2.md](PHASE2.md).
For the Product Phase 3 AI runtime bridge, see [PHASE3.md](PHASE3.md).
For the Product Phase 4 desktop control slice, see [PHASE4.md](PHASE4.md).
For the Product Phase 5 screen/context engine, see [PHASE5.md](PHASE5.md).
For the Product Phase 6 memory layer, see [PHASE6.md](PHASE6.md).
For the Product Phase 7 agent orchestration layer, see [PHASE7.md](PHASE7.md).
For the Product Phase 8 predictive layer, see [PHASE8.md](PHASE8.md).
For the Product Phase 9 plugin SDK, see [PHASE9.md](PHASE9.md).
For the Product Phase 10 plugin trust layer, see [PHASE10.md](PHASE10.md).
For the Product Phase 11 plugin signature layer, see [PHASE11.md](PHASE11.md).
For the Product Phase 12 plugin approval surface, see [PHASE12.md](PHASE12.md).
For the executable production gate, see
[PRODUCTION_READINESS.md](PRODUCTION_READINESS.md).
For the product architecture, see [architecture.md](architecture.md).

## Planned Structure

```text
product/
  README.md              Product-track entry point
  huggingos_core/        Capability, policy, audit, and config library
  agent/                 Rust production agent runtime
  distro/                Base image, package, and rootfs definitions
  services/              huggingOS daemons and local APIs
  cli/                   huggingos command-line entrypoint
  ui/                    Desktop overlay and control center
  policy/                Permissions, confirmations, audit, and rollback rules
  plugins/               Sample and future third-party plugin manifests
  tests/                 Product smoke tests
```

These folders should be created when the matching implementation issue starts.
Do not fill them with fake placeholders.

## Current Product Slice

The product track currently provides:

- Ubuntu LTS hosted prototype strategy.
- Reproducible dev and smoke commands.
- A real `huggingos` CLI.
- Runtime config layout with no committed secrets.
- Product smoke tests and CI.
- Phase 2 in-process capability control plane.
- Rust production agent runtime started under `product/agent/`.
- First safe capabilities for product status, file listing, small text reads,
  safe workspace note creation, and audit listing.
- Phase 3 Rust AI bridge with deterministic local planning, redacted provider
  secret readiness, and plan-execute-verify execution through capabilities.
- Phase 4 desktop capabilities for session status, app listing, confirmed app
  launch, confirmed browser URL opening, and workspace mode planning.
- Phase 5 screen/context capabilities for readiness, permissioned screenshot
  capture, active-context snapshots, OCR image reads, and privacy redaction.
- Phase 6 memory capabilities for session facts, preferences, audit-derived
  events, opt-in semantic file indexing/search, export/delete, and resume
  planning.
- Phase 7 agent capabilities for catalog, delegation plans, confirmed
  orchestration, and trace listing.
- Phase 8 predictive/self-healing capabilities for repeated workflow
  detection, proactive suggestions, failure diagnosis, and activity timelines.
- Phase 9 plugin capabilities for manifest validation, install, catalog,
  workflow planning, read-only capability runs, disable, and remove.
- Phase 10 plugin trust capabilities for package validation, permission review,
  sandbox declarations, rollback metadata, and audited trust state.
- Phase 11 plugin signature capabilities for Ed25519 package verification,
  install-time verified-signature enforcement, update channel metadata,
  persisted rollback manifests, and tamper rejection.
- Phase 12 plugin approval surface capabilities for desktop-ready trust,
  permission, sandbox, update, rollback, and action-review payloads.
- Product readiness audit capability for a machine-readable gate over the
  current working product surface and known deferred work.

## Product Commands

From the repository root on Linux or WSL:

```bash
python3 product/cli/huggingos.py status
python3 product/cli/huggingos.py doctor
python3 product/cli/huggingos.py capabilities
python3 product/cli/huggingos.py run product.status
cd product/agent && cargo run -- status --json
cd product/agent && cargo run -- run product.status --json
cd product/agent && cargo run -- ai status --json
cd product/agent && cargo run -- ai plan "show product status" --json
cd product/agent && cargo run -- ai run "show product status" --json
cd product/agent && cargo run -- run product.readiness.audit --json
cd product/agent && cargo run -- secrets status --json
cd product/agent && cargo run -- run desktop.status --json
cd product/agent && cargo run -- run apps.list --json
cd product/agent && cargo run -- run browser.open_url --param url=https://example.com --dry-run --json
cd product/agent && cargo run -- run workspace.mode.plan --param mode=coding --json
cd product/agent && cargo run -- run screen.status --json
cd product/agent && cargo run -- run screen.capture --dry-run --json
cd product/agent && cargo run -- run context.snapshot --confirm --json
cd product/agent && cargo run -- run screen.ocr_image --param path=../../README.md --dry-run --json
cd product/agent && cargo run -- ai run "what is open" --confirm --json
cd product/agent && cargo run -- run memory.session.remember --param key=current-goal --param value=phase-six-seven --json
cd product/agent && cargo run -- run memory.preference.set --param key=theme --param value=dark --json
cd product/agent && cargo run -- run files.semantic.index --param root=../../docs --confirm --json
cd product/agent && cargo run -- run files.semantic.search --param query=capability --json
cd product/agent && cargo run -- run agents.orchestrate --param "goal=daily brief" --confirm --json
cd product/agent && cargo run -- run proactive.workflow.detect --json
cd product/agent && cargo run -- run proactive.suggest --json
cd product/agent && cargo run -- run selfheal.diagnose --param symptom=app_crashed --param target=editor --param simulated=true --json
cd product/agent && cargo run -- run timeline.explain --json
cd product/agent && cargo run -- run plugins.validate --param source=../plugins/hello-assistant --json
cd product/agent && cargo run -- run plugins.package.validate --param source=../plugins/hello-assistant --json
cd product/agent && cargo run -- run plugins.permission.review --param source=../plugins/hello-assistant --json
cd product/agent && cargo run -- run plugins.approval.surface --param source=../plugins/hello-assistant --json
cd product/agent && cargo run -- run plugins.install --param source=../plugins/hello-assistant --param force=true --confirm --json
cd product/agent && cargo run -- run plugins.catalog --json
cd product/agent && cargo run -- run plugins.workflow.plan --param plugin_id=sample.hello --json
cd product/agent && cargo run -- run plugins.capability.run --param plugin_id=sample.hello --param capability=hello --json
cd product/agent && cargo run -- run plugins.disable --param plugin_id=sample.hello --confirm --json
cd product/agent && cargo run -- run plugins.remove --param plugin_id=sample.hello --confirm --json
python3 -m unittest discover -s product/tests -p "test_*.py"
```

Or with `make`:

```bash
make product-status
make product-doctor
make product-capabilities
make product-run-status
make product-agent-ai-status
make product-agent-ai-plan
make product-agent-ai-run
make product-agent-readiness-audit
make product-agent-secrets
make product-agent-desktop-status
make product-agent-apps-list
make product-agent-browser-dry-run
make product-agent-workspace-plan
make product-agent-screen-status
make product-agent-screen-capture-dry-run
make product-agent-context-snapshot
make product-agent-ocr-dry-run
make product-agent-memory-remember
make product-agent-memory-list
make product-agent-preference-set
make product-agent-semantic-index
make product-agent-semantic-search
make product-agent-resume-plan
make product-agent-agents-catalog
make product-agent-agents-plan
make product-agent-agents-orchestrate
make product-agent-agents-trace-list
make product-agent-workflow-detect
make product-agent-proactive-suggest
make product-agent-selfheal-diagnose
make product-agent-timeline-explain
make product-agent-plugin-validate
make product-agent-plugin-package-validate
make product-agent-plugin-permission-review
make product-agent-plugin-approval-surface
make product-agent-plugin-install
make product-agent-plugin-catalog
make product-agent-plugin-workflow
make product-agent-plugin-run
make product-agent-plugin-disable
make product-agent-plugin-remove
make product-agent-smoke
make product-smoke
```

From inside `product/`:

```bash
make status
make doctor
make capabilities
make run-status
make agent-ai-status
make agent-ai-plan
make agent-ai-run
make agent-readiness-audit
make agent-secrets
make agent-desktop-status
make agent-apps-list
make agent-browser-dry-run
make agent-workspace-plan
make agent-screen-status
make agent-screen-capture-dry-run
make agent-context-snapshot
make agent-ocr-dry-run
make agent-memory-remember
make agent-memory-list
make agent-preference-set
make agent-semantic-index
make agent-semantic-search
make agent-resume-plan
make agent-agents-catalog
make agent-agents-plan
make agent-agents-orchestrate
make agent-agents-trace-list
make agent-workflow-detect
make agent-proactive-suggest
make agent-selfheal-diagnose
make agent-timeline-explain
make agent-plugin-validate
make agent-plugin-package-validate
make agent-plugin-permission-review
make agent-plugin-approval-surface
make agent-plugin-install
make agent-plugin-catalog
make agent-plugin-workflow
make agent-plugin-run
make agent-plugin-disable
make agent-plugin-remove
make agent-smoke
make smoke
```

## Capability Examples

All real automated actions should go through the capability control plane:

```bash
python3 product/cli/huggingos.py capabilities
python3 product/cli/huggingos.py run fs.list --param path=.
python3 product/cli/huggingos.py run fs.read_text --param path=product/README.md
python3 product/cli/huggingos.py run notes.create --param title=PhaseTwo --param content="real note"
python3 product/cli/huggingos.py run audit.list --param limit=10
```

Use `HUGGINGOS_STATE_DIR` to move local audit/runtime state, and
`HUGGINGOS_WORKSPACE_DIR` to constrain low-risk workspace writes. The default
audit log is JSON Lines at the product state path.

The Rust agent can now plan simple natural-language intents through the offline
`local.rules` provider and execute those plans through the capability control
plane. It can detect desktop readiness, list desktop apps, perform confirmed
app/browser launch commands from a real graphical Linux session, report
screen/context readiness, capture screenshots through discovered Linux backends,
snapshot active-window context, OCR local images when `tesseract` exists, store
user-controlled memory, search opt-in text file indexes, and delegate work
through permissioned built-in agents. It can also install and run declarative
read-only plugin manifests with explicit permission review, cryptographically
verified local package signatures, desktop-ready approval surfaces, and a
machine-readable product readiness audit. It still does not call cloud AI
providers, automate browser DOMs, launch arbitrary shell commands, read
clipboard contents by default, extract accessibility trees, execute arbitrary
plugin code, render a desktop overlay, run plugin daemons, auto-update plugins,
or arrange windows.

The Rust agent is the production path for AI planning, future daemon work, and
desktop integration. The Python CLI remains a reference control surface for the
Phase 2 capability model.

## Local Files

Local runtime files, secrets, and machine-specific config are intentionally not
tracked. Use documented config examples when they exist, and keep real provider
keys in the OS keyring or local ignored files only.

## Rules

- No hardcoded API keys or local machine paths.
- No fake AI provider responses.
- No fake browser automation.
- No root-only behavior unless the action truly needs root and explains why.
- Every OS action should move toward the capability API and audit model.
