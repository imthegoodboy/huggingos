# huggingOS AI-Native OS Roadmap

This roadmap turns the AI OS vision into buildable phases for this repository.
The goal is not to fake a chatbot inside a kernel. The main product path is now
a Linux-kernel-based AI operating system layer, because Linux already provides
the real drivers, filesystems, networking, process isolation, permissions, and
application ecosystem needed for a normal working OS.

The existing custom x86 kernel remains valuable, but as a kernel-lab track for
learning and low-level experiments. Product work should not rebuild basic OS
infrastructure from scratch unless there is a clear reason.

## Ground Truth

huggingOS currently has two tracks:

- Product track: a planned Linux-based AI OS distribution/layer. This is the
  path for a normal working OS with app control, networking, storage, GUI, and
  AI services.
- Kernel-lab track: the existing 32-bit x86 hobby kernel with GRUB boot,
  protected mode, VGA text output, PS/2 keyboard input, PIT/RTC, RAMFS, a
  kernel heap, syscalls, logging, and an interactive shell.

Important constraint: external AI APIs, browser control, cloud memory, and
downloaded models require networking, TLS, secret storage, a process model,
userspace services, a GUI/app environment, and permission boundaries. Linux gives
us those foundations now. AI runtimes should connect through controlled
userspace services and capability APIs, not kernel shortcuts.

AI-native does not mean "put an LLM in the kernel." The kernel should provide
isolation, devices, files, networking, timers, graphics, and safe syscalls. AI
logic should run in userspace services that call explicit OS capabilities.

See [ADR 0001](docs/adr/0001-kernel-strategy.md) for the kernel strategy.
See [ARCHITECTURE.md](ARCHITECTURE.md) for the project-wide production
architecture.

## Product Principles

- Every product phase must leave a bootable Linux image or runnable Linux-hosted
  prototype. Kernel-lab phases must leave the hobby ISO bootable in QEMU.
- No dummy feature should be marked complete.
- Kernel changes need a smoke test or in-OS selftest coverage.
- AI actions must run through permissioned capability APIs, not raw arbitrary
  memory or device access.
- Dangerous actions need confirmation, audit logs, and rollback where possible.
- The Linux shell/CLI remains the first product control plane until the desktop
  overlay and app model exist.
- Prefer deterministic local behavior first; add cloud AI only after the secure
  runtime and secret model exist.
- Build the smallest real vertical slice; do not add labels, banners, or command
  names that imply missing behavior is complete.
- Probe or configure hardware and resources instead of assuming fixed addresses,
  host paths, API keys, or emulator-only values.
- Major architecture decisions need a short ADR-style note in docs before the
  implementation becomes large.

## Definition Of Done For Every Product Phase

- A Linux-hosted prototype or bootable Linux image runs the feature.
- Tests or smoke checks prove the executable behavior.
- README or docs describe the new behavior and limitations.
- GitHub issues for the phase are closed or explicitly carried forward.
- No secret, token, local machine path, or fake provider is hardcoded.
- The feature has a failure path that reports what went wrong.

## Definition Of Done For Kernel-Lab Phases

- `make clean all iso` succeeds.
- QEMU boots to an interactive prompt.
- Relevant `selftest` checks pass.
- README or docs describe the new behavior and limitations.
- GitHub issues for the phase are closed or explicitly carried forward.
- No new known crash path is accepted without an issue and mitigation plan.
- No secret, token, local machine path, or fake provider is hardcoded.
- The feature has a failure path that reports what went wrong.

## Non-Hardcoded Implementation Policy

Hardcoding is allowed only for architectural constants that are part of the
platform contract, such as the VGA text memory address or x86 interrupt vector
numbers. Anything environment-specific must be discovered, configured, or passed
through a documented interface.

Do not hardcode:

- API keys, provider names as the only possible provider, tokens, or secrets.
- User-specific paths, drive letters, usernames, or host machine assumptions.
- Fake framebuffer addresses, fake network responses, fake files, or fake memory.
- Test outputs that claim success without executing the feature.
- AI plans that bypass permission, audit, or verification layers.

Every temporary stub must:

- Fail safely.
- State that it is not implemented.
- Avoid modifying state.
- Have a GitHub issue or roadmap entry for the real implementation.

## Architecture Records

Create short design notes under `docs/adr/` when a phase chooses a direction
that will be hard to reverse. Examples:

- Boot protocol and bootloader assumptions.
- Memory model and paging strategy.
- Executable format and syscall ABI.
- Filesystem and block-device strategy.
- Networking stack boundary.
- AI runtime provider and secret-storage model.
- Capability permission model.

## Track A: Linux Product Roadmap

This is the main path for building the AI-native OS.

## Phase 0: Product Direction

Status: complete in PR #20.

Delivered:

- Decision to use Linux as the production kernel.
- Custom x86 kernel kept as the kernel-lab track.
- Agent guardrails and durable notes.
- ADR template and kernel strategy ADR.

## Phase 1: Linux Product Foundation

Status: complete.

Goal: create a bootable/runnable Linux-based huggingOS foundation that future AI
services can build on.

Core features:

- Choose base approach: Debian/Ubuntu live image, Buildroot, or Yocto. Done in
  [ADR 0002](docs/adr/0002-linux-base-strategy.md).
- Create `product/` tree for Linux image config, services, and packaging. Done
  for CLI, config, distro, policy, services, and tests.
- Produce first bootable Linux image or runnable dev rootfs. Done as a runnable
  Ubuntu LTS hosted prototype; full image is deferred by ADR 0002.
- Add a `huggingos` CLI entrypoint for local AI OS commands. Done with Phase 1
  `status`, `doctor`, and `config` commands.
- Add service layout for future daemon work. Done with `product/services`.
- Add non-secret runtime config layout. Done with `product/config`.
- Add smoke check for image/prototype startup. Done with product tests.
- Add CI for docs and the initial product build/prototype checks. Done with the
  Product Phase 1 workflow.

Acceptance criteria:

- A fresh checkout can run the Linux product prototype or build the first image
  using documented commands.
- No command depends on a user-specific local path.
- The `huggingos` CLI runs a real command and reports clear errors.
- Runtime config exists, but no secrets are committed.
- CI validates the product foundation.

GitHub issue plan:

- Milestone: [Product Phase 1: Linux Product Foundation](https://github.com/imthegoodboy/huggingos/milestone/2)
- Epic: [#13 Product Phase 1 Linux product foundation](https://github.com/imthegoodboy/huggingos/issues/13)
- [#14 Choose Linux base image strategy](https://github.com/imthegoodboy/huggingos/issues/14)
- [#15 Create product tree and dev/build commands](https://github.com/imthegoodboy/huggingos/issues/15)
- [#17 Add first huggingos CLI](https://github.com/imthegoodboy/huggingos/issues/17)
- [#18 Add runtime config layout and no-secret policy](https://github.com/imthegoodboy/huggingos/issues/18)
- [#19 Add product smoke test and CI](https://github.com/imthegoodboy/huggingos/issues/19)

## Phase 2: Capability API And Local Automation

Status: complete.

Goal: expose safe local OS actions that AI can later call.

Core features:

- Capability registry for read-only and low-risk local capabilities. Done with
  `product.status`, `fs.list`, `fs.read_text`, `notes.create`, and
  `audit.list`.
- Structured action schema: intent, parameters, risk level, permissions, result.
  Done in `product/huggingos_core/models.py`.
- Production agent runtime language selected. Done in
  [ADR 0004](docs/adr/0004-agent-runtime-language.md), with the first Rust crate
  under `product/agent/`.
- Confirmation policy for destructive/sensitive actions. Done with allow, deny,
  confirm, and dry-run decisions.
- Audit log for every automated action. Done with append-only JSON Lines audit
  records.
- Reversible file operations where possible. Started with constrained
  workspace-only note creation that refuses overwrites.
- Local deterministic planner for simple commands before LLM integration.
  Deferred to Product Phase 3, after the capability contract is stable.

Architecture:

- ADR: [0003 Capability Control Plane](docs/adr/0003-capability-control-plane.md)
- Product overview: [product/architecture.md](product/architecture.md)

GitHub issue plan:

- Milestone: [Product Phase 2: Capability API And Local Automation](https://github.com/imthegoodboy/huggingos/milestone/3)
- Epic: [#31 Product Phase 2 capability API and local automation](https://github.com/imthegoodboy/huggingos/issues/31)
- [#23 Define capability action schema and risk levels](https://github.com/imthegoodboy/huggingos/issues/23)
- [#24 Add local capability registry and CLI listing](https://github.com/imthegoodboy/huggingos/issues/24)
- [#26 Add policy engine for capability execution](https://github.com/imthegoodboy/huggingos/issues/26)
- [#27 Add append-only audit log for capability actions](https://github.com/imthegoodboy/huggingos/issues/27)
- [#30 Add first safe local capabilities](https://github.com/imthegoodboy/huggingos/issues/30)
- [#28 Expand product smoke tests for capability layer](https://github.com/imthegoodboy/huggingos/issues/28)

Acceptance criteria:

- CLI can list capabilities.
- Read-only actions execute through registry, policy, executor, verifier, and
  audit.
- Low-risk note creation uses a configured safe workspace.
- Dangerous action classes require confirmation or are denied by default.
- Audit logs show what happened, when, and why.

## Phase 3: AI Runtime Bridge And Secrets

Goal: connect AI planning to OS capabilities without hardcoded keys or kernel
shortcuts.

Core features:

- Provider abstraction for local rules, local model runtimes, and cloud models.
  Done in the Rust agent runtime status model.
- Secret readiness checks without printing or storing provider secrets. Done
  through redacted environment/keyring-ready boundaries.
- Prompt/action schema mapped to the capability API. Done for deterministic
  local intents.
- Planner, executor, verifier loop. Done through `ai plan` and `ai run`.
- Offline mode that keeps local control working without cloud providers. Done
  through the `local.rules` provider.
- Provider failure handling and user-visible error reporting. Done for declared
  but unavailable local-model and cloud adapters.

Acceptance criteria:

- API keys are never committed or baked into images.
- The AI bridge can produce a plan and execute approved capability calls.
- The verifier checks observable results before reporting success.
- Provider failures do not break local OS control.

Tracking:

- Milestone: [Product Phase 3: AI Runtime Bridge And Secrets](https://github.com/imthegoodboy/huggingos/milestone/4)
- Epic: [#39 Product Phase 3 Epic: AI runtime bridge and secrets](https://github.com/imthegoodboy/huggingos/issues/39)
- Provider interface: [#43](https://github.com/imthegoodboy/huggingos/issues/43)
- Secret checks: [#38](https://github.com/imthegoodboy/huggingos/issues/38)
- Local planner: [#45](https://github.com/imthegoodboy/huggingos/issues/45)
- Plan-execute-verify: [#40](https://github.com/imthegoodboy/huggingos/issues/40)
- Smoke tests and docs: [#37](https://github.com/imthegoodboy/huggingos/issues/37)

## Phase 4: Desktop Overlay And App Control

Goal: make the AI OS feel integrated with the Linux desktop.

Core features:

- CLI command center through the Rust AI planner. Done through `ai plan` and
  `ai run` mappings for desktop intents.
- Desktop/session readiness detection. Done through `desktop.status`.
- App listing and launch through desktop APIs. Done for `.desktop` registry
  listing and confirmed `gio`/`gtk-launch` app launch.
- Browser opening through a real desktop backend. Done for confirmed HTTP/HTTPS
  URL opening via `xdg-open`/`gio open`; DOM automation remains later.
- Workspace modes: coding, study, deep work, gaming, travel. Done as
  inspectable `workspace.mode.plan` previews.
- Global hotkey command center, graphical overlay/sidebar, window arrangement,
  notification policy controls. Deferred until `huggingosd` desktop service
  work.

Acceptance criteria:

- User can invoke the command center from a CLI.
- AI can plan app/browser desktop actions through permissioned capabilities.
- Browser URL opening uses a real desktop backend, not fake responses.
- Every desktop action is logged and reversible status is explicit where
  rollback is not possible.

Tracking:

- Milestone: [Product Phase 4: Desktop Command Center And App Control](https://github.com/imthegoodboy/huggingos/milestone/5)
- Epic: [#56 Product Phase 4 Epic: Desktop command center and app control](https://github.com/imthegoodboy/huggingos/issues/56)
- Desktop status: [#57](https://github.com/imthegoodboy/huggingos/issues/57)
- App listing/launch: [#54](https://github.com/imthegoodboy/huggingos/issues/54)
- Browser URL open: [#58](https://github.com/imthegoodboy/huggingos/issues/58)
- Workspace mode planning: [#53](https://github.com/imthegoodboy/huggingos/issues/53)
- Smoke tests and docs: [#51](https://github.com/imthegoodboy/huggingos/issues/51)

## Phase 5: Screen Understanding And Context Engine

Status: complete.

Goal: let the OS understand visible state and active work.

Core features:

- Screenshot capture through desktop APIs. Done for discovered Linux capture
  backends with confirmation, audit, safe workspace output, and active-context
  privacy checks. XDG Desktop Portal/PipeWire capture remains later.
- OCR pipeline. Done for provided local image paths through `tesseract` when it
  is installed.
- Accessibility tree integration. Deferred until the desktop service and richer
  permission model exist.
- Active app/window/control state. Done for active app/window metadata through a
  discovered context backend. Fine-grained control state remains later.
- Clipboard, file, browser tab, and system context snapshots. Done for desktop
  status, active-window metadata, screen readiness, privacy status, and
  clipboard readiness. Clipboard contents, browser tab context, and semantic
  file context remain later.
- Privacy controls for observed apps and screen regions. Done for app/window
  marker redaction and capture denial on private active contexts. Region-level
  redaction remains later.

Acceptance criteria:

- AI command center can answer "what is open?" from real OS state through
  `context.snapshot`.
- Screen capture and OCR are permissioned and logged.
- Private apps/windows are redacted and capture-denied. Region-level privacy is
  explicitly carried forward.

Tracking:

- Milestone: [Product Phase 5: Screen And Context Engine](https://github.com/imthegoodboy/huggingos/milestone/6)
- Epic: [#65 Product Phase 5 Epic: Screen and context engine](https://github.com/imthegoodboy/huggingos/issues/65)
- Screen readiness: [#62](https://github.com/imthegoodboy/huggingos/issues/62)
- Screen capture: [#66](https://github.com/imthegoodboy/huggingos/issues/66)
- Active context snapshot: [#61](https://github.com/imthegoodboy/huggingos/issues/61)
- Privacy exclusions: [#67](https://github.com/imthegoodboy/huggingos/issues/67)
- Smoke tests and docs: [#68](https://github.com/imthegoodboy/huggingos/issues/68)

## Phase 6: Memory And Semantic Files

Status: complete.

Goal: add useful memory and semantic file search without unsafe collection.

Core features:

- Short-term session memory. Done through `memory.session.remember` and
  `memory.session.list`.
- User preference store. Done through `memory.preference.set` and
  `memory.preference.list`.
- Event store for app/file/workflow history. Started through
  `memory.event.list`, which derives event memory from the audit log.
- Semantic file index with embeddings. Done as a confirmed opt-in local token
  index through `files.semantic.index` and `files.semantic.search`; embeddings
  remain later because provider, retention, and deletion controls must mature
  first.
- User controls for inspect, edit, export, and delete memory. Done through
  list/set/export/delete capabilities.
- Retention rules and private mode. Partially done through explicit delete and
  secret-path exclusions; daemon-level retention/private mode remains later.

Acceptance criteria:

- "Resume my last workspace" can build a memory-backed plan from known local
  state through `workspace.resume.plan`.
- User can inspect and delete remembered facts.
- Memory collection is documented, permissioned, and testable.

Tracking:

- Epic: [#72 Product Phase 6 Epic: Memory and semantic files](https://github.com/imthegoodboy/huggingos/issues/72)

## Phase 7: Multi-Agent Orchestration

Status: complete.

Goal: split intelligence into focused agents coordinated by an orchestrator.

Core features:

- Agent manifest format. Done through the built-in `agents.catalog`.
- Capability permissions per agent. Done with explicit per-agent allowlists.
- System, file, app, browser, coding, security, productivity agents. Started
  with system, memory, file, desktop, and writer agents. Browser/coding/security
  specialist agents remain later when those capability surfaces exist.
- Orchestrator that plans, delegates, verifies, and reports. Done through
  `agents.plan` and confirmed `agents.orchestrate`.
- Agent logs and replayable traces. Done through local JSONL traces and
  `agents.trace.list`.

Acceptance criteria:

- Orchestrator can delegate at least one workflow to two separate agents.
- Agents cannot call capabilities outside their permission scope.
- User can inspect what each agent did.

Tracking:

- Epic: [#73 Product Phase 7 Epic: Multi-agent orchestration](https://github.com/imthegoodboy/huggingos/issues/73)

## Phase 8: Predictive And Self-Healing OS

Goal: move from reactive commands to useful proactive help.

Status: complete for the first suggestion-first slice.

Core features:

- Detect repeated workflows and suggest automations. Done through
  `proactive.workflow.detect` over audited capability events.
- Monitor crashes, failed services, memory pressure, and slow operations. Done
  for simulated or user-reported symptoms through `selfheal.diagnose`.
- Recommend cleanup or optimization actions. Done as suggestion-only outputs
  through `proactive.suggest`.
- Safe auto-fix rules for low-risk actions. Deferred until rollback, daemon
  controls, and approval UI exist.
- Explain-what-happened timeline. Done through `timeline.explain`.

Acceptance criteria:

- OS can detect and summarize at least one repeated workflow. Done and covered
  by Rust tests.
- OS can detect a simulated app failure and recommend a recovery action. Done
  and covered by Rust tests.
- Proactive actions are never destructive without confirmation. Done by making
  Phase 8 capabilities read-only and suggestion-first.

Tracking:

- Epic: [#75 Product Phase 8 Epic: Predictive and self-healing OS](https://github.com/imthegoodboy/huggingos/issues/75)

## Phase 9: Plugin SDK And Ecosystem

Goal: let new apps and agents integrate cleanly.

Status: complete for the first declarative manifest slice.

Core features:

- Plugin manifest and permission model. Done with
  `huggingos.plugin.v1` manifests and plugin permissions.
- SDK for capability providers. Done for declarative read-only plugin
  capabilities through `plugins.capability.run`.
- SDK for app UI metadata. Deferred until the desktop overlay exists.
- SDK for agents and workflows. Done for manifest-declared workflows and agent
  allowlists; plugin-native agent execution remains later.
- Versioned API contracts and compatibility tests. Done with manifest
  validation and Rust tests.

Acceptance criteria:

- A sample third-party plugin can add one capability and one workflow. Done with
  `product/plugins/hello-assistant/plugin.json`.
- Plugin install, disable, and remove paths work. Done and covered by Rust tests
  and CI smoke commands.
- Permission prompts and audit logs include plugin identity. Done through
  `plugin_identity` in plugin lifecycle and run audit records.

Tracking:

- Epic: [#77 Product Phase 9 Epic: Plugin SDK and ecosystem](https://github.com/imthegoodboy/huggingos/issues/77)

## Phase 10: Plugin Trust, Packaging, And Approval UI

Goal: make the plugin ecosystem safe enough for richer integrations.

Status: complete for the first trust-metadata slice.

Core features:

- Signed plugin package metadata. Done as validated signature metadata fields
  in Phase 10; cryptographic verification is completed in Phase 11.
- Install-time permission review. Done through `plugins.permission.review`.
- Disable/remove rollback metadata. Done in plugin lifecycle results.
- Sandbox design for plugin-provided code execution. Done as manifest sandbox
  declarations with code execution disabled.
- UI metadata contract for future desktop overlay approval flows. Done through
  `ui.display_name`, `ui.approval_summary`, and approval summaries.

Acceptance criteria:

- A plugin package can be validated before install. Done through
  `plugins.package.validate`.
- User-facing permission summaries can be generated from a manifest. Done
  through `plugins.permission.review`.
- Plugin lifecycle audit records include package trust state. Done through
  `plugin_trust_state`.
- Arbitrary plugin code remains disabled until sandboxing exists. Done through
  sandbox validation and runtime policy.

Tracking:

- Epic: [#79 Product Phase 10 Epic: Plugin trust packaging and approval UI](https://github.com/imthegoodboy/huggingos/issues/79)

## Phase 11: Plugin Signature Verification

Goal: make local plugin package trust cryptographically verifiable before
expanding plugin power.

Status: complete for the first signed manifest package slice.

Core features:

- Real cryptographic signature verification for plugin packages. Done with
  SHA-256 plus Ed25519 verification over canonical manifest JSON.
- Local signed package archive format. Done as
  `huggingos.plugin.package.v1`, a signed manifest package format that can
  evolve into archive bundles later.
- Package update channel metadata and rollback manifests. Done with `update`
  metadata, disabled auto-update, and persisted rollback records under local
  state.
- Desktop approval UI surfaces for plugin trust and permissions. Deferred to
  Product Phase 12; Phase 11 exposes the data contracts and CLI responses.
- Plugin-provided code remains disabled until sandboxing exists. Done through
  sandbox validation and runtime policy.

Acceptance criteria:

- A plugin package can be cryptographically validated before install. Done
  through `plugins.package.validate` returning `signature_verified`.
- Tampered plugin manifests fail closed. Done with Rust regression coverage.
- Plugin install requires verified package trust. Done through
  `plugins.install`.
- Rollback manifests are persisted for plugin install, disable, and remove.
  Done under the configured state directory.

Tracking:

- Epic: [#81 Product Phase 11 Epic: Plugin signature verification](https://github.com/imthegoodboy/huggingos/issues/81)

## Phase 12: Plugin Approval Surface

Goal: make verified plugin trust decisions renderable before adding more plugin
power.

Status: complete for the first desktop-ready approval contract slice.

Core features:

- Desktop approval UI surfaces for plugin trust, permissions, sandbox
  declarations, and update metadata. Done as the read-only
  `plugins.approval.surface` JSON control surface.
- Surface-level action review. Done with confirmed next-action payloads for
  install, disable, and remove.
- Rollback visibility. Done with recent rollback manifest summaries and clear
  automatic-rollback limitations.
- Sandbox boundary design for plugin-provided code execution. Deferred to
  Product Phase 13; Phase 12 keeps plugin code disabled.
- Signed archive bundles and trusted update feeds. Deferred to Product Phase
  13.

Acceptance criteria:

- A source plugin approval surface can be generated without installing the
  plugin. Done through `plugins.approval.surface --param source=...`.
- An installed plugin approval surface can show lifecycle actions and rollback
  records. Done through `plugins.approval.surface --param plugin_id=...`.
- The local planner can map plugin approval prompts. Done through
  `local.rules`.
- The approval surface does not mutate plugin state. Done as a read-only
  capability.

Tracking:

- Epic: [#83 Product Phase 12 Epic: Plugin approval surface readiness](https://github.com/imthegoodboy/huggingos/issues/83)

## Product Readiness Audit Gate

Goal: make production-readiness claims executable and repeatable.

Status: complete for the current product surface.

Core features:

- Machine-readable readiness audit. Done through `product.readiness.audit`.
- Capability registration checks. Done for the current product-critical
  capability set.
- Audit path scoping check. Done against the configured state directory.
- Plugin trust and approval checks. Done against the signed sample plugin and
  `plugins.approval.surface`.
- Documentation and smoke target checks. Done against the current repo files.
- Known deferred work reporting. Done in the readiness payload.

Acceptance criteria:

- `product.readiness.audit` returns `huggingos.product.readiness.v1`. Done.
- The current repo is not blocked by the readiness gate. Done in Rust tests and
  smoke commands.
- Deferred work remains visible instead of hidden behind a green status. Done.

Tracking:

- Issue: [#85 Product Readiness Audit: executable production gate](https://github.com/imthegoodboy/huggingos/issues/85)

## Track B: Kernel-Lab Roadmap

This track keeps the existing custom x86 kernel useful without blocking the
Linux product path.

## Kernel-Lab Phase 1: Reliable Kernel And Shell Foundation

Goal: make the current text-mode hobby OS stable enough for experiments.

Core features:

- QEMU smoke test target that boots the ISO and can run scripted shell commands.
- Serial logging option for automated test output.
- Expanded `selftest` coverage for heap reuse, aligned allocations, RAMFS path
  edge cases, append/write failure behavior, shell redirection, and command
  dispatch.
- Panic diagnostics that show exception number, error code, and registers.
- Shell parser cleanup.
- CI workflow for the hobby kernel ISO.
- ADRs for memory model, syscall ABI, and test strategy.

Acceptance criteria:

- A single command can build the ISO and run a QEMU smoke script.
- `selftest` covers heap, RAMFS, and shell behavior.
- CI rejects build failures.
- The OS stays interactive after common bad inputs.

Existing GitHub tracking for this lab track:

- Milestone: [Phase 1: Reliable Kernel And Shell Foundation](https://github.com/imthegoodboy/huggingos/milestone/1)
- Epic: [#9 Phase 1 reliable kernel and shell foundation](https://github.com/imthegoodboy/huggingos/issues/9)
- [#3 Add QEMU smoke test target and serial output](https://github.com/imthegoodboy/huggingos/issues/3)
- [#4 Expand in-kernel selftest coverage](https://github.com/imthegoodboy/huggingos/issues/4)
- [#5 Harden shell parser and redirection handling](https://github.com/imthegoodboy/huggingos/issues/5)
- [#6 Add panic register diagnostics](https://github.com/imthegoodboy/huggingos/issues/6)
- [#7 Add CI build for kernel and ISO](https://github.com/imthegoodboy/huggingos/issues/7)
- [#8 Document syscall ABI and memory model](https://github.com/imthegoodboy/huggingos/issues/8)

## Project Management Plan

Use GitHub for execution:

- One milestone per phase.
- One epic issue per phase.
- Implementation issues under the active phase only.
- Use `track:product` for Linux product work and `track:kernel-lab` for the
  custom hobby kernel.
- Labels:
  - `phase:1`, `phase:2`, etc.
  - `track:product`, `track:kernel-lab`.
  - `type:epic`, `type:feature`, `type:test`, `type:docs`, `type:security`.
  - `area:kernel`, `area:shell`, `area:fs`, `area:drivers`, `area:ai`,
    `area:gui`, `area:automation`, `area:infra`, `area:net`,
    `area:security`, `area:product`, `area:distro`, `area:cli`,
    `area:policy`, `area:service`.
- PRs should mention the issue they close.
- Each phase ends with a release note in `UPDATE.md`.
- Future agents should follow [agent/SKILL.md](agent/SKILL.md).

## Agent Working Kit

These files keep AI builders focused and repeatable:

- [agent/SKILL.md](agent/SKILL.md): repo-local build rules for AI agents.
- [agent/COMMANDS.md](agent/COMMANDS.md): common build, QEMU, GitHub, and
  hygiene commands.
- [agent/TASK_CHECKLIST.md](agent/TASK_CHECKLIST.md): no-drift checklist for
  each task.
- [agent/notes/INDEX.md](agent/notes/INDEX.md): durable notes and warnings from
  previous agents.
- [.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md): PR gate
  for validation and no-fake behavior.
- [.github/ISSUE_TEMPLATE/phase_task.yml](.github/ISSUE_TEMPLATE/phase_task.yml):
  issue template for phase tasks.
- [docs/adr/0000-template.md](docs/adr/0000-template.md): architecture decision
  template.

Agents should treat this kit as guardrails, not as a substitute for reading the
actual source code.

Future-useful discoveries should be captured in `agent/notes/` using
`agent/notes/TEMPLATE.md`. Keep those notes concise and evidence-based.

## Next Sprint: Product Phase 13

Start here next after Product Phase 12 is merged:

1. Design the sandbox boundary for future plugin-provided code execution.
2. Add a signed archive bundle format beyond manifest-only packages.
3. Add trusted update feed metadata and manual update approval flow.
4. Add rendered desktop overlay/control-center screens that consume
   `huggingos.plugin.approval.v1`.
5. Keep plugin-provided code disabled until the sandbox is implemented,
   tested, and audited.

Product Phase 12 gives plugin manifests a reviewable approval contract. Phase
13 should add sandbox and archive architecture before expanding plugin power.

## Things We Will Not Fake

- No hardcoded API keys in source code.
- No fake network-backed AI until networking or a secure host bridge exists.
- No fake browser automation until a browser/app model exists.
- No expanded persistent memory beyond explicit local state until retention and
  private-mode controls exist.
- No "full OS control" until actions pass through permissioned capability APIs.
- No hardcoded host paths, local usernames, API keys, fake device addresses, or
  success messages that hide missing implementation.
