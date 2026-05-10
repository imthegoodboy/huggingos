# huggingOS Architecture

This document is the north-star architecture for huggingOS. It defines what we
are building, how the system should be shaped, and how agents gain broad control
without turning the computer into an unsafe pile of shell commands.

## What huggingOS Is

huggingOS is a Linux-based AI operating system layer.

Linux provides the real operating system foundation:

- Kernel, drivers, filesystems, networking, process isolation, permissions, and
  desktop integration.
- Mature package and update infrastructure.
- Real app, browser, service, and security surfaces.

huggingOS provides the intelligent control layer above Linux:

- Natural-language command center.
- Capability API for files, apps, shell, browser, settings, screen, memory, and
  workflows.
- Policy engine for permissions, confirmations, and risk control.
- Agent runtime for planning, execution, verification, and self-correction.
- Audit log and observability for everything agents do.
- Future desktop overlay, memory, workflow automation, plugin SDK, and local +
  cloud AI runtime.

The goal is not to replace Linux. The goal is to make Linux feel like an
AI-native OS where the user expresses intent and huggingOS safely executes.

## Core Principle

Agents get full control through typed capabilities, not arbitrary unchecked
access.

```text
User intent or agent plan
  -> planner
  -> capability registry
  -> policy decision
  -> executor
  -> verifier
  -> audit log
  -> result
```

This is the key architecture. It lets agents become powerful over time while
keeping every action visible, testable, reversible where possible, and bound by
policy.

## Product Tracks

```text
Track A: Product OS
  Linux kernel + Ubuntu/Debian userspace
  huggingOS CLI, services, UI, capability APIs, agents, memory, plugins

Track B: Kernel Lab
  Existing custom x86 kernel
  QEMU boot, RAMFS, shell, interrupts, heap, low-level experiments
```

The product track is the main path. The kernel-lab track is useful for learning
and low-level experiments, but product AI features belong in Linux userspace.

## Target Production Architecture

```text
+------------------------------------------------------------------+
| User Interfaces                                                   |
| CLI, hotkey command center, desktop overlay, voice, future GUI     |
+-------------------------------+----------------------------------+
                                |
+-------------------------------v----------------------------------+
| Intent And Planning Layer                                         |
| local rules, LLM bridge, task planner, verifier, recovery loop     |
+-------------------------------+----------------------------------+
                                |
+-------------------------------v----------------------------------+
| Capability Control Plane                                          |
| registry, schemas, risk levels, policy, executor, verifier, audit  |
+-------------------------------+----------------------------------+
                                |
+-------------------------------v----------------------------------+
| OS Integration Layer                                              |
| files, apps, shell, browser, desktop portals, D-Bus, systemd user  |
+-------------------------------+----------------------------------+
                                |
+-------------------------------v----------------------------------+
| Linux Base                                                        |
| kernel, drivers, namespaces, permissions, filesystems, networking  |
+------------------------------------------------------------------+
```

## Main Components

### 1. Product CLI

Current reference entrypoint: `product/cli/huggingos.py`.
Production runtime entrypoint: `product/agent/` (`huggingos-agent`, Rust).

Responsibilities:

- Developer and power-user control plane.
- Status, doctor, config inspection.
- Phase 2 capability listing and execution.
- Rust agent runtime parity path.
- CI-friendly smoke validation.

Rules:

- Must run without root for normal commands.
- Must use standard library first unless a dependency clearly pays for itself.
- Must show honest errors for missing features.

### 2. Local User Service

Future entrypoint: Rust `huggingosd` as a user-level service.

Responsibilities:

- Own long-running state, audit logs, memory indexes, subscriptions, and local
  automation triggers.
- Serve local IPC to CLI, GUI, and agent runtime.
- Keep privileged work separate from normal user work.

Recommended path:

- Start in-process in Phase 2.
- Move the production runtime to Rust in `product/agent/`.
- Move to a systemd user service after the schemas stabilize.
- Use D-Bus or local HTTP/Unix socket only when the service boundary is needed.

### 3. Capability Registry

The registry is the operating surface agents can see.

Each capability needs:

- Stable name and version.
- Owner and description.
- Risk level.
- Permission requirements.
- Input and output schemas.
- Executor.
- Verifier.
- Audit fields.
- Rollback metadata when possible.

Initial capability families:

- `product.status`
- `fs.list`
- `fs.read_text`
- `notes.create`
- `audit.list`

Later capability families:

- `app.launch`
- `window.arrange`
- `browser.open`
- `browser.click`
- `screen.capture`
- `context.snapshot`
- `screen.ocr_image`
- `settings.set`
- `workflow.run`

### 4. Policy Engine

The policy engine decides whether an action can run.

Decisions:

- `allow`
- `deny`
- `confirm`
- `dry_run_only`

Risk model:

- `read`: observes local state only.
- `low`: creates or changes constrained user-owned state.
- `medium`: opens apps, modifies files, or runs constrained commands.
- `high`: deletes, overwrites, changes settings, uses network, touches secrets,
  captures screen, controls browser, or runs broad shell commands.

Default stance:

- Read actions can run if scoped and auditable.
- Low-risk actions need a safe workspace or explicit target.
- Medium-risk actions need clear policy and user-visible result.
- High-risk actions require confirmation or are denied until implemented safely.

### 5. Audit And Observability

Every capability decision and execution must be recorded.

Audit records should include:

- Action id.
- Actor.
- Capability.
- Input summary.
- Policy decision.
- Start and finish times.
- Status.
- Verification result.
- Error summary.
- Rollback reference if available.

Use structured JSON Lines locally. Add OpenTelemetry later for traces, metrics,
and logs when services become long-running.

### 6. AI Runtime

The AI runtime must be provider-agnostic.

Provider types:

- Local deterministic planner.
- Local model runtime.
- Cloud model runtime.

Rules:

- No API keys in source.
- No provider-specific logic in the kernel.
- No model can directly mutate OS state.
- Models propose plans; the capability control plane executes approved actions.
- Provider failures must not break local commands.

MCP can be supported later as an adapter layer for external tools, but internal
OS control should stay behind huggingOS capabilities so policy and audit remain
consistent.

Phase 3 implements the first production AI runtime bridge in Rust:

- `local.rules` is the deterministic offline provider.
- `ai plan` maps supported natural-language intents into typed capability
  calls.
- `ai run` executes only through the capability engine, policy, audit, and
  verifier.
- `secrets status` reports provider readiness without exposing values.
- Cloud and local-model providers are declared for selection/status/failure
  handling, but are not executable until real adapters are added.

### 7. Desktop Integration

Use Linux desktop-native APIs before brittle UI scraping.

Preferred order:

1. App or service API.
2. D-Bus/systemd/user service integration.
3. XDG Desktop Portal for permissioned desktop actions.
4. Browser automation backend for browser workflows.
5. Accessibility tree and screen understanding.
6. Vision/OCR fallback only when semantic APIs are unavailable.

Screen capture and audio/video flows should use desktop portals and PipeWire
where the desktop supports them.

Phase 4 implements the first desktop-control bridge in Rust:

- `desktop.status` detects graphical-session and backend readiness.
- `apps.list` reads installed `.desktop` entries.
- `apps.launch` launches a safe desktop ID through `gio` or `gtk-launch` after
  confirmation.
- `browser.open_url` opens HTTP/HTTPS URLs through `xdg-open` or `gio open`
  after confirmation.
- `workspace.mode.plan` previews workspace modes before full window management
  exists.

Phase 5 implements the first screen/context bridge in Rust:

- `screen.status` detects capture, OCR, active-context, clipboard, and privacy
  readiness.
- `screen.capture` captures screenshots through discovered Linux backends after
  confirmation and active-context privacy checks.
- `context.snapshot` reports active-window and desktop context with privacy
  redaction.
- `screen.ocr_image` runs OCR through `tesseract` for approved local image paths.
- Clipboard content, accessibility trees, browser tab context, and portal-based
  capture remain later.

### 8. Memory System

Memory must be inspectable and deletable.

Memory layers:

- Session memory: current task state.
- Preference memory: user settings and style.
- Event memory: audited actions and workflow history.
- Semantic file memory: embeddings over user-approved files.
- Private mode: no collection beyond active task.

Rules:

- User can inspect, export, and delete memory.
- Retention is explicit.
- Sensitive apps and folders can be excluded.
- Memory cannot bypass permissions.

### 9. Plugin And Skill System

Plugins should extend capabilities, UI surfaces, agents, and workflows.

Plugin manifest fields:

- Id, name, version.
- Provided capabilities.
- Required permissions.
- Risk declarations.
- Entry points.
- Config schema.
- Audit identity.

Plugins must never silently gain broad OS control. They register capabilities,
then policy decides what can run.

## Recommended Technology Choices

Use now:

- Python standard library for Phase 1 and Phase 2 reference behavior.
- Rust for the production agent runtime and future `huggingosd`.
- TOML for local config.
- JSON/JSON Lines for action and audit records.
- GitHub Actions on Ubuntu for CI.
- Ubuntu LTS hosted prototype as the current product base.

Use in Phase 2 through Phase 5:

- Dataclasses or typed models for action contracts.
- Rust structs/enums for production action contracts.
- systemd user service for `huggingosd` once daemon state exists.
- D-Bus for Linux service/app integration where appropriate.
- `xdotool`, `grim`, `gnome-screenshot`, `spectacle`, `scrot`, ImageMagick
  `import`, and `tesseract` as discovered optional host backends for the first
  screen/context slice.
- SQLite for audit indexes, event history, and local metadata when JSON Lines is
  no longer enough.
- OS keyring/libsecret for provider secrets.

Use later:

- XDG Desktop Portal for permissioned desktop integration.
- PipeWire for screen/audio/video capture paths.
- OpenTelemetry for service traces, metrics, and logs.
- eBPF only for advanced observability/security after the user-space control
  plane is stable.
- MCP adapter for external AI tools, not as the internal permission boundary.
- Vector database or SQLite vector extension for semantic memory after retention
  and deletion rules exist.

Avoid for now:

- Copying the Linux kernel source into this repo.
- Full custom kernel work for product features.
- Unrestricted shell execution.
- Browser automation before browser permissions and audit exist.
- Cloud AI before secrets, policy, audit, and local fallback exist.
- A full ISO/live image before the product services are useful enough to ship.

## Phase Roadmap

### Phase 1: Product Foundation

Complete.

- Ubuntu LTS hosted prototype.
- Real CLI.
- Non-secret config.
- Product tests.
- CI.
- Service/policy/distro boundaries.

### Phase 2: Capability API And Local Automation

Complete.

- Action schema.
- Capability registry.
- Policy engine.
- Audit log.
- First safe local capabilities.
- CLI execution through the control plane.
- Rust production agent crate started.

### Phase 3: AI Runtime And Secrets

Complete in the Rust production agent:

- Provider abstraction.
- Deterministic local planner.
- Redacted secret readiness checks.
- Plan-execute-verify loop.
- Offline mode.
- Safe unavailable-provider errors.

### Phase 4: Desktop Command Center And App Control

Complete for the first capability-backed desktop slice:

- CLI command center through Rust `ai plan` and `ai run`.
- Desktop readiness detection.
- `.desktop` app registry listing.
- Confirmed app launch.
- Confirmed browser URL opening through a real desktop backend.
- Workspace mode previews.

Still later:

- Global hotkey command center.
- Graphical overlay/sidebar.
- Window/workspace arrangement.
- Browser DOM automation.
- Notification policy controls.
- Desktop portal integration for capture and richer window context.

### Phase 5: Screen And Context Engine

Complete for the first capability-backed observation slice:

- Screen readiness and backend discovery.
- Permissioned screen capture with active-context privacy checks.
- Active app/window context snapshots.
- OCR over provided local images through `tesseract`.
- Privacy markers and redaction.

Still later:

- XDG Desktop Portal/PipeWire capture.
- Accessibility tree extraction.
- Browser tab and DOM context.
- Clipboard content reads with explicit consent.
- Region-level redaction.

### Phase 6: Memory And Semantic Files

- User-controlled memory.
- Semantic file search.
- Retention and deletion controls.

### Phase 7: Multi-Agent Orchestration

- Agent manifests.
- Per-agent permissions.
- Orchestrator.
- Replayable traces.

### Phase 8: Predictive And Self-Healing OS

- Repeated workflow detection.
- Safe proactive suggestions.
- Crash/service monitoring.

### Phase 9: Plugin SDK

- Plugin manifest.
- Capability provider SDK.
- Agent/workflow SDK.

## Production-Readiness Gates

No phase is complete until:

- It has executable behavior.
- Tests or smoke checks prove it.
- Docs describe what works and what does not.
- Secrets are not committed.
- Dangerous actions require confirmation or are denied.
- Every agent action is auditable.
- The system fails safely.

## Sources Checked

- Ubuntu release cycle: https://ubuntu.com/about/release-cycle
- Debian releases: https://www.debian.org/releases/index
- Buildroot manual: https://buildroot.org/downloads/manual/manual.html
- Yocto Project overview: https://www.yoctoproject.org/about/project-overview/
- Model Context Protocol specification: https://modelcontextprotocol.io/specification/2025-06-18/server/tools
- XDG Desktop Portal documentation: https://flatpak.github.io/xdg-desktop-portal/docs/api-reference
- PipeWire project: https://pipewire.org/
- OpenTelemetry documentation: https://opentelemetry.io/docs/
- eBPF documentation: https://docs.ebpf.io/
