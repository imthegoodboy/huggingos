# Agent Task Checklist

Use this checklist before, during, and after every huggingOS task.

## Before Coding

- [ ] Read the active GitHub issue.
- [ ] Read the matching phase in `PLAN.md`.
- [ ] Confirm the task belongs to the active phase or explain why it must happen now.
- [ ] Inspect the relevant source files before designing the change.
- [ ] Identify the real user-visible or kernel-visible behavior being added.
- [ ] Identify the validation command that will prove it works.

## No-Drift Gate

Stop and rethink if the task starts to become:

- A fake command that only prints success.
- A UI label for a feature that does not exist.
- A hardcoded path, token, API key, fake device address, or emulator-only shortcut.
- A cloud AI feature before networking, TLS, secret storage, and userspace service
  boundaries exist.
- A browser automation feature before a browser/app automation layer exists.
- A large refactor unrelated to the issue.

## Implementation Gate

- [ ] Keep the change as a small vertical slice.
- [ ] Fail safely for unsupported hardware, invalid input, and allocation failure.
- [ ] Do not corrupt shell, RAMFS, heap, or kernel state on bad input.
- [ ] Add or update `selftest`, QEMU smoke coverage, or docs for the behavior.
- [ ] Update `README.md`, `UPDATE.md`, `PLAN.md`, or ADR docs only when they are
  directly affected.

## Validation Gate

For code changes:

- [ ] `make clean all iso`
- [ ] QEMU boot check
- [ ] `selftest`
- [ ] Feature-specific manual or scripted test
- [ ] `git diff --check`

For docs-only changes:

- [ ] `git diff --check`
- [ ] Links and issue numbers are correct.

For all changes:

- [ ] No hardcoded secrets or user-specific paths.
- [ ] Remaining limitations are stated honestly.
- [ ] PR references the issue it advances or closes.

## Knowledge Capture Gate

- [ ] If this task revealed a future-useful gotcha, note, or warning, add or
  update a file in `agent/notes/`.
- [ ] If a note was added, update `agent/notes/INDEX.md`.
- [ ] If no note was needed, say so in the PR or final summary.
