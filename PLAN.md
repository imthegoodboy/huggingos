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

Status: architecture ready; implementation not started.

Goal: expose safe local OS actions that AI can later call.

Core features:

- Capability registry for files, apps, shell commands, windows, browser, system
  state, notifications, and settings. Start with read-only and low-risk local
  capabilities.
- Structured action schema: intent, parameters, risk level, permissions, result.
- Confirmation policy for destructive/sensitive actions.
- Audit log for every automated action.
- Reversible file operations where possible.
- Local deterministic planner for simple commands before LLM integration.

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

- CLI can execute real structured actions like list files, open app, create
  note, or run shell command through the capability layer.
- Dangerous actions require confirmation.
- Audit logs show what happened, when, and why.

## Phase 3: AI Runtime Bridge And Secrets

Goal: connect AI planning to OS capabilities without hardcoded keys or kernel
shortcuts.

Core features:

- Provider abstraction for local rules, local model runtimes, and cloud models.
- Secret loading from OS keyring or encrypted runtime config.
- Prompt/action schema mapped to the capability API.
- Planner, executor, verifier loop.
- Offline mode that keeps local control working without cloud providers.
- Provider failure handling and user-visible error reporting.

Acceptance criteria:

- API keys are never committed or baked into images.
- The AI bridge can produce a plan and execute approved capability calls.
- The verifier checks observable results before reporting success.
- Provider failures do not break local OS control.

## Phase 4: Desktop Overlay And App Control

Goal: make the AI OS feel integrated with the Linux desktop.

Core features:

- Global hotkey command center.
- Desktop overlay/sidebar.
- App launching and workspace arrangement through desktop APIs.
- Browser automation through a real browser automation layer.
- Notification policy controls.
- Workspace modes: coding, study, deep work, gaming, travel.

Acceptance criteria:

- User can invoke the command center from a hotkey or CLI.
- AI can open and arrange real apps through permissioned capabilities.
- Browser actions use a real browser automation backend, not fake responses.
- Every action is logged and reversible when possible.

## Phase 5: Screen Understanding And Context Engine

Goal: let the OS understand visible state and active work.

Core features:

- Screenshot capture through desktop APIs.
- OCR pipeline.
- Accessibility tree integration.
- Active app/window/control state.
- Clipboard, file, browser tab, and system context snapshots.
- Privacy controls for observed apps and screen regions.

Acceptance criteria:

- AI command center can answer "what is open?" from real OS state.
- Screen capture and OCR are permissioned and logged.
- Private apps/regions are not observed.

## Phase 6: Memory And Semantic Files

Goal: add useful memory and semantic file search without unsafe collection.

Core features:

- Short-term session memory.
- User preference store.
- Event store for app/file/workflow history.
- Semantic file index with embeddings.
- User controls for inspect, edit, export, and delete memory.
- Retention rules and private mode.

Acceptance criteria:

- "Resume my last workspace" can restore known local state.
- User can inspect and delete remembered facts.
- Memory collection is documented, permissioned, and testable.

## Phase 7: Multi-Agent Orchestration

Goal: split intelligence into focused agents coordinated by an orchestrator.

Core features:

- Agent manifest format.
- Capability permissions per agent.
- System, file, app, browser, coding, security, productivity agents.
- Orchestrator that plans, delegates, verifies, and reports.
- Agent logs and replayable traces.

Acceptance criteria:

- Orchestrator can delegate at least one workflow to two separate agents.
- Agents cannot call capabilities outside their permission scope.
- User can inspect what each agent did.

## Phase 8: Predictive And Self-Healing OS

Goal: move from reactive commands to useful proactive help.

Core features:

- Detect repeated workflows and suggest automations.
- Monitor crashes, failed services, memory pressure, and slow operations.
- Recommend cleanup or optimization actions.
- Safe auto-fix rules for low-risk actions.
- Explain-what-happened timeline.

Acceptance criteria:

- OS can detect and summarize at least one repeated workflow.
- OS can detect a simulated app failure and recommend a recovery action.
- Proactive actions are never destructive without confirmation.

## Phase 9: Plugin SDK And Ecosystem

Goal: let new apps and agents integrate cleanly.

Core features:

- Plugin manifest and permission model.
- SDK for capability providers.
- SDK for app UI metadata.
- SDK for agents and workflows.
- Versioned API contracts and compatibility tests.

Acceptance criteria:

- A sample third-party plugin can add one capability and one workflow.
- Plugin install, disable, and remove paths work.
- Permission prompts and audit logs include plugin identity.

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

## Next Sprint: Product Phase 2

Start here next after Product Phase 1 is merged:

1. Create a Product Phase 2 milestone and issues.
2. Define the capability action schema and risk levels.
3. Add the first local capability registry.
4. Add audit log structure for product actions.
5. Wire the CLI through the capability executor for safe read-only actions.

Product Phase 1 gives later AI features a real Linux OS foundation to stand on.
Phase 2 should now add the permissioned capability layer before any app control
or AI provider integration.

## Things We Will Not Fake

- No hardcoded API keys in source code.
- No fake network-backed AI until networking or a secure host bridge exists.
- No fake browser automation until a browser/app model exists.
- No "persistent memory" until storage and retention controls exist.
- No "full OS control" until actions pass through permissioned capability APIs.
- No hardcoded host paths, local usernames, API keys, fake device addresses, or
  success messages that hide missing implementation.
