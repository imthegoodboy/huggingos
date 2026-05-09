# huggingOS Update Log

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
