# Agent Task Checklist

Use this checklist before, during, and after every huggingOS task.

## Before Coding

- [ ] Read the active GitHub issue.
- [ ] Read the matching phase in `PLAN.md`.
- [ ] Decide the track: Linux product or kernel lab.
- [ ] Confirm the task belongs to the active phase or explain why it must happen now.
- [ ] For product work, read `product/README.md` and the kernel strategy ADR.
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
- A product AI feature being added to the custom hobby kernel instead of the
  Linux product track.
- A browser automation feature before a browser/app automation layer exists.
- Silent screen, clipboard, or OCR observation that does not pass through
  policy, confirmation, privacy rules, and audit.
- Silent memory collection, semantic indexing outside an opt-in root, or agents
  executing capabilities outside their allowlist.
- Proactive or self-healing behavior that silently launches apps, restarts
  services, deletes files, or performs cleanup without confirmation and
  rollback design.
- Plugin behavior that executes arbitrary native code, downloads packages, or
  starts background services without sandboxing, signatures, and rollback.
- Plugin trust UI or docs that imply signature verification before real
  cryptographic verification exists.
- A large refactor unrelated to the issue.

## Implementation Gate

- [ ] Keep the change as a small vertical slice.
- [ ] Fail safely for unsupported hardware, invalid input, and allocation failure.
- [ ] Do not corrupt shell, RAMFS, heap, or kernel state on bad input.
- [ ] Add or update `selftest`, QEMU smoke coverage, or docs for the behavior.
- [ ] Update `README.md`, `UPDATE.md`, `PLAN.md`, or ADR docs only when they are
  directly affected.

## Validation Gate

For product code changes:

- [ ] Product command or smoke test from `product/README.md` or the active issue.
- [ ] Missing dependency path is documented and fails clearly.
- [ ] No product runtime, secret, generated image, or local config file is staged.
- [ ] Product docs are updated.

For kernel-lab code changes:

- [ ] `make clean all iso`
- [ ] QEMU boot check
- [ ] `selftest`
- [ ] Feature-specific manual or scripted test

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
