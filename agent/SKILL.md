---
name: huggingos-builder
description: Use this repo-local skill for all future huggingOS work. It guides AI agents to build real OS features phase-by-phase from PLAN.md, avoid hardcoded or fake behavior, validate in QEMU, and keep GitHub issues/PRs aligned with the roadmap.
---

# huggingOS Builder Skill

Use this skill whenever you modify, review, plan, or publish changes in the
huggingOS repository.

## Mission

Build huggingOS as a real operating system, not a demo shell with fake labels.
Every change must move the project toward a bootable, testable OS with honest
capabilities, clear limitations, and a path to AI-native control through safe
system APIs.

## First Reads

Before changing code, read only what is needed:

- `PLAN.md` for the current phase and acceptance criteria.
- `README.md` for the current user-facing status.
- `UPDATE.md` for recent completed work.
- `agent/TASK_CHECKLIST.md` for the no-drift task checklist.
- The active GitHub issue, if one exists.
- The specific kernel modules touched by the task.

## Golden Rules

- Do not hardcode secrets, API keys, tokens, user paths, or machine-specific
  assumptions.
- Do not mark a feature complete unless it works in QEMU or has a documented
  reason it cannot be exercised yet.
- Do not fake hardware, networking, AI, persistence, browser automation, or GUI
  support. Stub code must fail safely and be documented as a stub.
- Keep the OS bootable after every phase.
- Prefer small, reviewable PRs tied to GitHub issues.
- Add tests or `selftest` coverage for risky kernel behavior.
- Update docs when behavior or limitations change.
- Preserve user changes in the working tree; never revert unrelated work.

## Phase Workflow

1. Identify the active phase from `PLAN.md`.
2. Use `gh issue list --milestone "<phase milestone>"` to find the next issue.
3. Create a branch named `codex/<short-task-name>`.
4. Inspect the relevant code before designing the change.
5. Implement the smallest complete vertical slice.
6. Validate with the required commands.
7. Update docs and issue links.
8. Commit, push, and open a PR that references the issue.

Do not jump to advanced AI features before their prerequisites exist. For
example, cloud AI needs networking, TLS, secret storage, and a userspace service
boundary. Browser automation needs a browser/app model. Persistent memory needs
storage and retention controls.

## Validation Checklist

For code changes, run as much of this list as the task needs:

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
rg -n "[^\x00-\x7F]" kernel README.md UPDATE.md PLAN.md agent
```

If a validation step cannot run, say exactly why in the PR and final summary.

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

AI belongs above the kernel behind safe interfaces:

- Kernel: primitives, isolation, syscalls, device and filesystem support.
- Userspace services: AI runtime bridge, provider selection, memory index,
  workflow engine, and agent orchestration.
- Capability API: the only path for AI actions that affect OS state.
- Policy layer: permissions, confirmation, audit logs, and rollback.

Never put model prompts, cloud credentials, provider-specific keys, or network
API calls directly into the kernel image.

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
- Keep `UPDATE.md` as completed release notes, not future promises.
- Update docs in the same PR as behavior changes.
- Use `docs/adr/0000-template.md` for decisions that affect architecture,
  security, process/user boundaries, storage, networking, graphics, AI runtime,
  or capability permissions.
