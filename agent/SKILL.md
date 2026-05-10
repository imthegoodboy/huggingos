---
name: huggingos-builder
description: Use this repo-local skill for all future huggingOS work. It guides AI agents to build real OS features phase-by-phase from PLAN.md, avoid hardcoded or fake behavior, validate in QEMU, and keep GitHub issues/PRs aligned with the roadmap.
---

# huggingOS Builder Skill

Use this skill whenever you modify, review, plan, or publish changes in the
huggingOS repository.

## Mission

Build huggingOS as a real operating system, not a demo shell with fake labels.
The main product path uses the Linux kernel and Linux userspace. The existing
custom x86 kernel remains a kernel-lab track for low-level experiments. Every
change must state which track it belongs to and move that track through a real,
testable capability.

## First Reads

Before changing code, read only what is needed:

- `PLAN.md` for the current phase and acceptance criteria.
- `README.md` for the current user-facing status.
- `UPDATE.md` for recent completed work.
- `docs/adr/0001-kernel-strategy.md` for the Linux product vs kernel-lab split.
- `product/README.md` when working on the Linux product track.
- `agent/TASK_CHECKLIST.md` for the no-drift task checklist.
- `agent/COMMANDS.md` for common build, QEMU, GitHub, and hygiene commands.
- `agent/notes/INDEX.md` for durable lessons from previous agents.
- The active GitHub issue, if one exists.
- The specific kernel modules touched by the task.

## Golden Rules

- Do not hardcode secrets, API keys, tokens, user paths, or machine-specific
  assumptions.
- Do not put product AI features into the custom hobby kernel. Product AI work
  belongs in the Linux product track unless an issue explicitly says otherwise.
- Do not mark a feature complete unless it works in QEMU or has a documented
  reason it cannot be exercised yet.
- Do not fake hardware, networking, AI, persistence, browser automation, or GUI
  support. Stub code must fail safely and be documented as a stub.
- Keep the OS bootable after every phase.
- Prefer small, reviewable PRs tied to GitHub issues.
- Add tests or `selftest` coverage for risky kernel behavior.
- Update docs when behavior or limitations change.
- Capture future-useful gotchas in `agent/notes/` before finishing.
- Preserve user changes in the working tree; never revert unrelated work.

## Track Selection

Before implementation, choose exactly one track:

- Product track: Linux image/rootfs, `product/`, userspace services, CLI,
  desktop integration, AI runtime bridge, capability API, policy, memory,
  automation, plugins, packaging, and CI for the AI OS product.
- Kernel-lab track: `kernel/`, `boot/`, low-level x86 kernel behavior, RAMFS,
  shell, interrupts, drivers, syscalls, QEMU ISO, and lab CI.

If a task asks for a normal working AI OS, choose the product track. If a task
asks about the bootable custom kernel or QEMU ISO, choose the kernel-lab track.

## Phase Workflow

1. Identify the active phase from `PLAN.md`.
2. Identify whether the task is `track:product` or `track:kernel-lab`.
3. Use `gh issue list --label "<track>" --milestone "<phase milestone>"` to
   find the next issue.
4. Create a branch named `codex/<short-task-name>`.
5. Inspect the relevant code before designing the change.
6. Implement the smallest complete vertical slice.
7. Validate with the required commands.
8. Update docs and issue links.
9. Commit, push, and open a PR that references the issue.

Do not jump to advanced AI features before their prerequisites exist. For
example, cloud AI needs networking, TLS, secret storage, and a userspace service
boundary. Browser automation needs a browser/app model. Persistent memory needs
storage and retention controls.

## Validation Checklist

For Linux product changes, run the product-specific smoke/build command from
`product/README.md` or the active issue. If it does not exist yet, the task that
adds it must document and test it.

For kernel-lab code changes, run as much of this list as the task needs:

```bash
make clean all iso
make qemu
```

When automated QEMU smoke testing exists, prefer that over manual QEMU checks.
Until then, boot the ISO and run:

```text
selftest
assist run memory status
```

Also run:

```bash
git diff --check
rg -n "[^\x00-\x7F]" kernel product README.md UPDATE.md PLAN.md agent docs .github
```

If a validation step cannot run, say exactly why in the PR and final summary.

## Knowledge Capture Rule

If you discover something that would help or protect a future agent, write it
down before finishing the task.

Use `agent/notes/TEMPLATE.md` for new notes. Update
`agent/notes/INDEX.md` so the note is discoverable.

Capture:

- Kernel or driver traps.
- Build, QEMU, or CI pitfalls.
- Architecture decisions that are too small for an ADR but important to know.
- Safety rules discovered from bugs.
- Patterns future agents should reuse.

Do not capture:

- Noisy command logs.
- Temporary guesses.
- Personal progress summaries.
- Information already clear in `PLAN.md` or source comments.

Keep notes short, factual, sourced, and actionable.

## Kernel Engineering Rules

- This is freestanding C/ASM. Do not rely on host libc behavior unless the repo
  provides the function in `kernel/lib`.
- Use fixed-width integer types for kernel-visible structures.
- Keep VGA output ASCII-safe unless the renderer explicitly supports more.
- Check pointer, length, and entry bounds before memory writes.
- Do not introduce dynamic allocation before memory is initialized.
- Do not write to guessed device memory. Detect, receive from boot info, or fail
  safely.
- Panic on unrecoverable kernel corruption; return clear errors for user or
  filesystem mistakes.
- Prefer capability-style interfaces over exposing raw internals to future AI
  or userspace layers.

## AI Feature Rules

AI belongs above the kernel behind safe interfaces. In the product track this
means Linux userspace services, not custom-kernel shortcuts:

- Kernel: primitives, isolation, syscalls, device and filesystem support.
- Userspace services: AI runtime bridge, provider selection, memory index,
  workflow engine, agent orchestration, predictive suggestions, and
  self-healing diagnostics.
- Capability API: the only path for AI actions that affect OS state.
- Policy layer: permissions, confirmation, audit logs, and rollback.
- Current production AI path: `product/agent` in Rust, with `local.rules`
  planning and `ai run` execution through capabilities.
- Current desktop path: `desktop.status`, `apps.list`, `apps.launch`,
  `browser.open_url`, and `workspace.mode.plan` in the Rust agent.
- Current screen/context path: `screen.status`, `screen.capture`,
  `context.snapshot`, and `screen.ocr_image` in the Rust agent, with privacy
  redaction and no silent clipboard reads.
- Current memory path: `memory.session.*`, `memory.preference.*`,
  `memory.event.list`, `memory.export`, `memory.delete`,
  `files.semantic.*`, and `workspace.resume.plan` in the Rust agent.
- Current multi-agent path: `agents.catalog`, `agents.plan`,
  `agents.orchestrate`, and `agents.trace.list` in the Rust agent.

Never put model prompts, cloud credentials, provider-specific keys, or network
API calls directly into the custom kernel image.

## What Counts As Real

Real means:

- The feature has an executable code path.
- The user can invoke it from shell, GUI, syscall, or a documented API.
- Failure modes are handled without corrupting kernel state.
- There is a QEMU/manual test or `selftest` coverage.
- Docs describe what works today and what is still planned.

Not real:

- A command that only prints "feature coming soon."
- A framebuffer driver that writes to a guessed address.
- An AI command that claims external reasoning without a provider/runtime.
- A persistent memory feature backed only by RAM.
- Browser control without a browser or app automation layer.

## GitHub Project Management

Use GitHub CLI for project tracking:

```bash
gh issue view <number>
gh issue list --milestone "<milestone>"
gh issue list --label "track:product" --milestone "<milestone>"
gh issue list --label "track:kernel-lab" --milestone "<milestone>"
gh pr create --fill
gh pr view --web
```

PRs should include:

- What changed.
- Why it changed.
- The issue it closes or advances.
- Validation performed.
- Remaining limitations.

Use `.github/PULL_REQUEST_TEMPLATE.md` for PR structure and
`.github/ISSUE_TEMPLATE/phase_task.yml` for new phase issues.

Use draft PRs for incomplete work. Use ready PRs only when validation is done.

## Documentation Rules

- Keep `PLAN.md` as the source of truth for the roadmap.
- Keep `README.md` accurate for users who want to build and run the OS.
- Keep `product/README.md` accurate for Linux product-track setup.
- Keep `UPDATE.md` as completed release notes, not future promises.
- Keep `agent/notes/INDEX.md` updated when adding durable agent notes.
- Update docs in the same PR as behavior changes.
- Use `docs/adr/0000-template.md` for decisions that affect architecture,
  security, process/user boundaries, storage, networking, graphics, AI runtime,
  or capability permissions.
