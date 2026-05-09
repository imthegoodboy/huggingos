# huggingOS AI-Native OS Roadmap

This roadmap turns the AI OS vision into buildable phases for this repository.
The goal is not to fake a chatbot inside a kernel. The goal is to grow
huggingOS into a reliable OS foundation, then add an AI-native control layer
that can observe state, plan actions, execute safely, verify results, and keep
memory over time.

## Ground Truth

huggingOS is currently a 32-bit x86 hobby OS with GRUB boot, protected mode,
VGA text output, PS/2 keyboard input, PIT/RTC, RAMFS, a kernel heap, syscalls,
logging, and an interactive shell.

Important constraint: external AI APIs, browser control, cloud memory, and
downloaded models require networking, TLS, secret storage, a process model, and
userspace services. Those do not belong directly in a fragile early kernel. The
right path is to build the OS capability layer first, then connect AI runtimes
through controlled system services.

AI-native does not mean "put an LLM in the kernel." The kernel should provide
isolation, devices, files, networking, timers, graphics, and safe syscalls. AI
logic should run in userspace services that call explicit OS capabilities.

## Product Principles

- Every phase must leave the OS bootable in QEMU.
- No dummy feature should be marked complete.
- Kernel changes need a smoke test or in-OS selftest coverage.
- AI actions must run through permissioned capability APIs, not raw arbitrary
  memory or device access.
- Dangerous actions need confirmation, audit logs, and rollback where possible.
- The shell remains the first control plane until a GUI and app model exist.
- Prefer deterministic local behavior first; add cloud AI only after the secure
  runtime and secret model exist.
- Build the smallest real vertical slice; do not add labels, banners, or command
  names that imply missing behavior is complete.
- Probe or configure hardware and resources instead of assuming fixed addresses,
  host paths, API keys, or emulator-only values.
- Major architecture decisions need a short ADR-style note in docs before the
  implementation becomes large.

## Definition Of Done For Every Phase

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

## Phase 0: Baseline Hardening

Status: complete in PR #2.

Delivered:

- Safe low-memory heap after `kernel_end`.
- Heap split, merge, free, and accounting.
- RAMFS nested path handling, append, recursive delete, and safer writes.
- Shell `echo >` and `echo >>` file redirection.
- Local deterministic `assist` / `ai` command.
- Built-in `selftest`.
- Buffered keyboard input.
- VGA boot dashboard and cursor updates.
- CPU exception panic path.
- Safe VESA stub instead of fake framebuffer writes.
- Updated build scripts and documentation.

## Phase 1: Reliable Kernel And Shell Foundation

Goal: make the current text-mode OS stable enough to build on every day.

Core features:

- QEMU smoke test target that boots the ISO and can run scripted shell commands.
- Serial logging option for automated test output.
- Expanded `selftest` coverage for heap reuse, aligned allocations, RAMFS path
  edge cases, append/write failure behavior, shell redirection, and command
  dispatch.
- Consistent kernel error codes for RAMFS, shell commands, and syscalls.
- Panic diagnostics that show exception number, error code, and relevant
  register values.
- Shell command parser cleanup: quoting, whitespace, redirection, command
  length limits, and clear failures.
- Documentation for the syscall ABI and current kernel memory model.
- CI workflow that builds the kernel and ISO on every PR.
- First ADRs for memory model, syscall ABI, and test strategy.

Acceptance criteria:

- A single command can build the ISO and run a QEMU smoke script.
- `selftest` covers heap, RAMFS, and shell behavior.
- CI rejects build failures.
- The OS stays interactive after common bad inputs.
- No repo script depends on a user-specific local path.

GitHub issue plan:

- Phase 1 epic: reliable kernel and shell foundation.
- Issue: add QEMU smoke test target and serial test output.
- Issue: expand in-kernel `selftest`.
- Issue: harden shell parser and redirection handling.
- Issue: add panic register diagnostics.
- Issue: add CI build for kernel and ISO.

Created GitHub tracking:

- Milestone: [Phase 1: Reliable Kernel And Shell Foundation](https://github.com/imthegoodboy/huggingos/milestone/1)
- Epic: [#9 Phase 1 reliable kernel and shell foundation](https://github.com/imthegoodboy/huggingos/issues/9)
- [#3 Add QEMU smoke test target and serial output](https://github.com/imthegoodboy/huggingos/issues/3)
- [#4 Expand in-kernel selftest coverage](https://github.com/imthegoodboy/huggingos/issues/4)
- [#5 Harden shell parser and redirection handling](https://github.com/imthegoodboy/huggingos/issues/5)
- [#6 Add panic register diagnostics](https://github.com/imthegoodboy/huggingos/issues/6)
- [#7 Add CI build for kernel and ISO](https://github.com/imthegoodboy/huggingos/issues/7)
- [#8 Document syscall ABI and memory model](https://github.com/imthegoodboy/huggingos/issues/8)

## Phase 2: Process And Userspace Foundation

Goal: support real apps instead of only built-in kernel shell commands.

Core features:

- Process table and task states.
- Cooperative scheduler first, then timer-driven preemption.
- Kernel/user privilege boundary.
- Page directory setup for kernel and user memory.
- ELF loader or simpler first user program format.
- File descriptors for stdin, stdout, stderr, and filesystem objects.
- Syscalls for process lifecycle, file I/O, time, memory, and logging.
- `exec`, `wait`, `exit`, and process error reporting.
- Init process and userspace shell migration path.

Acceptance criteria:

- At least two user tasks can run without corrupting kernel memory.
- A userspace hello-world app runs through the syscall layer.
- A crashing userspace app cannot crash the whole kernel.

## Phase 3: Persistent Storage And VFS

Goal: files survive reboot and the OS has a real storage abstraction.

Core features:

- VFS layer over RAMFS and disk-backed filesystems.
- Initrd support for bundled user programs and config.
- ATA PIO or virtio block driver, depending on emulator target.
- Partition and disk image discovery for the supported boot target.
- Block cache with explicit flush behavior.
- FAT12/FAT16 first, then consider FAT32.
- File permissions metadata, timestamps, and basic stat calls.
- Safe write path with flush and corruption checks.

Acceptance criteria:

- OS can read files from a boot image or disk image.
- OS can write a test file and read it after reboot in the chosen disk format.
- RAMFS and disk filesystem share the same VFS command surface.

## Phase 4: Device Model, Networking, And Secure Config

Goal: add normal OS device discovery, real network foundations, and a safe way
to store configuration before any cloud AI work.

Core features:

- Device manager for discovered hardware and drivers.
- PCI discovery if the emulator/device target needs it.
- Network driver target selection, such as e1000, rtl8139, or virtio-net.
- Ethernet, ARP, IPv4, ICMP, UDP, and TCP roadmap with incremental delivery.
- DNS resolver and basic HTTP client in userspace after sockets exist.
- TLS strategy documented before sending secrets over the network.
- Secure configuration store for user settings.
- Secret storage design that keeps API keys out of source, docs, and kernel
  images.
- Optional host bridge for development only, clearly marked and permissioned.

Acceptance criteria:

- OS can discover at least one supported emulated device without hardcoded host
  assumptions.
- OS can perform a real network smoke test for the implemented protocol level,
  such as ARP/ICMP before TCP exists.
- Secrets are loaded from a documented runtime store, never from hardcoded source
  constants.
- Networking failures report actionable errors instead of pretending success.

## Phase 5: Graphics, Mouse, And Windowing

Goal: move from text-mode shell to a real graphical desktop base.

Core features:

- Real VESA or framebuffer mode initialization through bootloader-provided info.
- 2D drawing primitives, fonts, double buffering, and damage regions.
- PS/2 mouse driver.
- Window manager with focus, move, resize, close, and keyboard routing.
- Basic UI toolkit: labels, buttons, menus, text fields, panels, and dialogs.
- System overlay surface reserved for the future AI command center.

Acceptance criteria:

- GUI boots reliably in QEMU.
- Mouse and keyboard control at least two windows.
- Text-mode fallback remains available for recovery.

## Phase 6: OS Capability And Automation API

Goal: expose safe, auditable OS actions that an AI can call.

Core features:

- Capability registry for files, windows, apps, settings, shell, and system info.
- Structured action format: intent, parameters, risk level, permissions, result.
- Confirmation prompts for destructive or sensitive actions.
- Audit log for every automated action.
- Rollback hooks for reversible file and settings operations.
- Automation runner for simple event rules.

Acceptance criteria:

- A local command can execute structured actions like "open app", "create file",
  "move window", or "clean folder" through the same capability API.
- Destructive actions require confirmation.
- Audit logs show what happened, when, and why.

## Phase 7: Local AI Runtime Bridge

Goal: connect AI planning to OS capabilities without putting model logic inside
the kernel.

Core features:

- AI service process boundary.
- Provider abstraction for local deterministic rules, local models, and later
  cloud models.
- Secure secret storage design before any API key support.
- Prompt/action schema that maps user goals to capability calls.
- Planner, executor, verifier loop.
- Failure recovery: retry, ask user, or stop safely.
- Offline mode that still works without cloud providers.

Acceptance criteria:

- Text command center can convert a goal into a plan.
- Plan steps execute only through capability APIs.
- The verifier checks observable results before reporting success.
- API keys are never hardcoded into the repo or kernel image.
- Provider failure does not break local OS control.

## Phase 8: Screen Understanding And Context Engine

Goal: let the OS understand visible state and active work.

Core features:

- Screenshot capture from framebuffer.
- OCR pipeline in userspace.
- Accessibility-like semantic UI tree for native apps.
- Active app, active window, focused control, clipboard, file, and system state.
- Context snapshot format that the planner can consume.
- Privacy controls for what screen regions can be observed.

Acceptance criteria:

- AI command center can answer "what is open?" from OS state.
- Native apps expose machine-readable UI metadata.
- Screen capture and OCR are permissioned and logged.

## Phase 9: Memory System

Goal: make the OS remember useful context without becoming unsafe or creepy.

Core features:

- Short-term session memory.
- Long-term user preference store.
- Event store for app/file/workflow history.
- Semantic memory index for files and notes.
- User controls for view, edit, export, and delete memory.
- Retention rules and private mode.

Acceptance criteria:

- "Resume my last workspace" can restore known local state.
- User can inspect and delete remembered facts.
- Memory collection is documented and permissioned.

## Phase 10: Multi-Agent Orchestration

Goal: split intelligence into focused agents coordinated by an orchestrator.

Agents:

- System agent: OS settings, hardware state, performance.
- File agent: search, organize, summarize, and transform files.
- App agent: launch and control native apps.
- Browser agent: future web automation after networking and browser support.
- Coding agent: edit, build, test, and debug projects.
- Security agent: permissions, suspicious activity, and policy enforcement.
- Productivity agent: calendar, tasks, workflows, and reminders.

Core features:

- Agent manifest format.
- Capability permissions per agent.
- Orchestrator that plans, delegates, verifies, and reports.
- Agent logs and replayable traces.

Acceptance criteria:

- Orchestrator can delegate at least one workflow to two separate agents.
- Agents cannot call capabilities outside their permission scope.
- User can inspect what each agent did.

## Phase 11: AI Desktop Overlay And Workspace Modes

Goal: make the AI feel like part of the OS, not a separate app.

Core features:

- Global hotkey command center.
- Floating overlay on top of windows.
- Contextual suggestions.
- Workspace modes: coding, study, deep work, gaming, travel.
- Rules for notifications, app layout, resource priority, and shortcuts.
- "Resume my day" workflow.

Acceptance criteria:

- User can switch workspace modes through text or UI.
- Mode changes are visible, reversible, and logged.
- Overlay can inspect context and call approved capabilities.

## Phase 12: Predictive And Self-Healing OS

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

## Phase 13: Plugin SDK And Ecosystem

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

## Project Management Plan

Use GitHub for execution:

- One milestone per phase.
- One epic issue per phase.
- Implementation issues under the active phase only.
- Labels:
  - `phase:1`, `phase:2`, etc.
  - `type:epic`, `type:feature`, `type:test`, `type:docs`, `type:security`.
  - `area:kernel`, `area:shell`, `area:fs`, `area:drivers`, `area:ai`,
    `area:gui`, `area:automation`, `area:infra`, `area:net`,
    `area:security`.
- PRs should mention the issue they close.
- Each phase ends with a release note in `UPDATE.md`.
- Future agents should follow [agent/SKILL.md](agent/SKILL.md).

## Agent Working Kit

These files keep AI builders focused and repeatable:

- [agent/SKILL.md](agent/SKILL.md): repo-local build rules for AI agents.
- [agent/TASK_CHECKLIST.md](agent/TASK_CHECKLIST.md): no-drift checklist for
  each task.
- [.github/PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md): PR gate
  for validation and no-fake behavior.
- [.github/ISSUE_TEMPLATE/phase_task.yml](.github/ISSUE_TEMPLATE/phase_task.yml):
  issue template for phase tasks.
- [docs/adr/0000-template.md](docs/adr/0000-template.md): architecture decision
  template.

Agents should treat this kit as guardrails, not as a substitute for reading the
actual source code.

## Immediate Sprint: Phase 1

Start here next:

1. Add QEMU smoke automation and serial output.
2. Expand `selftest`.
3. Harden shell parser/redirection.
4. Add panic register diagnostics.
5. Add CI build.

Phase 1 is intentionally boring and important. Once this is solid, the later AI
features have something real to stand on.

## Things We Will Not Fake

- No hardcoded API keys in source code.
- No fake network-backed AI until networking or a secure host bridge exists.
- No fake browser automation until a browser/app model exists.
- No "persistent memory" until storage and retention controls exist.
- No "full OS control" until actions pass through permissioned capability APIs.
- No hardcoded host paths, local usernames, API keys, fake device addresses, or
  success messages that hide missing implementation.
