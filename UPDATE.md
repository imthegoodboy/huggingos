# huggingOS Update Log

## Planning: Linux Product Track

The main product roadmap now uses the Linux kernel and Linux userspace for the
AI-native OS path. The existing custom x86 kernel remains as the kernel-lab
track for low-level experiments and QEMU validation.

Completed planning updates:

- Added ADR 0001 for the Linux-kernel product strategy.
- Added `product/README.md` as the product-track entry point.
- Split roadmap execution into Product and Kernel-Lab tracks.
- Added GitHub tracking labels and Product Phase 1 milestone/issues.
- Updated agent rules, commands, checklist, and notes so future agents do not
  add product AI features to the custom kernel by mistake.
- Added Product Phase 1 kickoff guidance, product-aware issue/PR templates, and
  local secret/runtime ignore rules before implementation starts.

## Product Phase 1 Foundation

Completed the first Linux-hosted product slice:

- Added ADR 0002 for the Ubuntu LTS hosted prototype base strategy.
- Added a Python standard-library-only `huggingos` product CLI.
- Added non-secret product config defaults and local override rules.
- Added the initial product service boundary documentation.
- Added product smoke tests and a Product Phase 1 CI workflow.
- Added product `make` targets for status, doctor, and smoke validation.

## Pre-Phase 2 Architecture Audit

Readiness checks found that the Phase 1 direction is correct, but executable
scripts needed LF line endings enforced for WSL/Linux execution from Windows
checkouts.

Completed before Phase 2:

- Added `.gitattributes` for Linux-facing scripts, source, Makefiles, TOML, and
  workflows.
- Added ADR 0003 for the capability control plane.
- Added `product/architecture.md` with the intended agent control architecture.
- Added Product Phase 2 GitHub milestone and issues for schema, registry,
  policy, audit, first capabilities, and tests.

## Root Architecture Document

Added `ARCHITECTURE.md` as the project-wide north-star architecture for the
Linux-based AI OS layer, including component boundaries, agent control flow,
technology choices, phase roadmap, and production-readiness gates.

## v2.0 Production-Readiness Pass

This pass turns the repo into a cleanly building, QEMU-verified hobby OS baseline
with fewer dummy paths and safer kernel behavior.

### Critical Issues Fixed

- Replaced the unsafe heap base at `0xC0000000`, which was not mapped in the flat
  kernel, with a low-memory heap that starts after the linked kernel image.
- Added heap block metadata, splitting, coalescing, `kfree`, and memory usage
  accounting.
- Disabled fake paging claims in code paths that still run as a flat kernel.
- Reworked RAMFS path handling for absolute paths, nested paths, `.`, `..`,
  create/read/write/append/delete/rename/copy behavior, and recursive deletion.
- Fixed shell file redirection so `echo text > file` and `echo text >> file` write
  through RAMFS correctly.
- Added buffered keyboard input so fast keystrokes are not lost between IRQs.
- Made CPU exceptions panic instead of silently returning after faults.
- Made the VESA driver a safe stub instead of writing to a bogus framebuffer.
- Removed UTF-8 boot art from VGA text output to avoid CP437 mojibake.
- Added VGA hardware cursor updates and a clean boot dashboard.

### New User-Facing Features

- `selftest` command for in-OS heap/RAMFS/path smoke checks.
- `assist` and `ai` commands for local natural-language command suggestions.
- Improved command discovery through `help` and `which`.
- Better boot status UI and clearer post-boot prompt.
- Appending shell redirection with `>>`.

### Verified

- `make clean all` completes successfully.
- `make iso` creates `huggingOs.iso`.
- QEMU boots the ISO with `-vga std`.
- In-kernel `selftest` passes in QEMU.

### Still Not Implemented

- Persistent disk filesystem.
- Network stack, DNS, TLS, browser, or external AI API calls.
- User-mode process isolation and scheduler.
- Real VESA framebuffer GUI.

These are now explicit roadmap items instead of being represented as complete.
