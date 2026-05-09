# Phase 0 Baseline Lessons

Date: 2026-05-10

Area: kernel, fs, drivers, ai

Related:

- PR: https://github.com/imthegoodboy/huggingos/pull/2
- Files: `kernel/memory/heap.c`, `kernel/fs/ramfs.c`,
  `kernel/drivers/vesa.c`, `kernel/drivers/vga.c`,
  `kernel/terminal/shell.c`

## Finding

The first production-readiness pass fixed several places where the OS looked
more complete than it really was.

Important lessons:

- The heap must stay in mapped low memory until paging/high-half mapping is real.
- RAMFS writes must allocate replacement storage before freeing existing file
  contents.
- VESA must not write to guessed framebuffer addresses. It should fail safely
  until bootloader-provided framebuffer info or a real mode switch exists.
- VGA text output should stay ASCII-safe unless the renderer supports more.
- The local `assist` command is deterministic command mapping, not external AI.

## Why It Matters

Future agents may be tempted to make the OS look advanced by adding labels,
fake responses, or guessed hardware addresses. That creates fragile code and
misleads the roadmap.

## Rule For Future Agents

- Build real executable paths.
- Keep stubs safe and honest.
- Never claim networking, external AI, browser automation, persistence, or GUI
  support until those layers exist and pass validation.
- Prefer a small working primitive over a large fake feature.

## Evidence / Validation

The Phase 0 PR passed:

- `wsl make clean all`
- `wsl make iso`
- QEMU boot smoke test
- In-OS `selftest`
- In-OS `assist run memory status`
