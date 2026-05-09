# huggingOS

huggingOS is a small bootable 32-bit x86 operating system built from scratch for
OS development practice. It boots with GRUB, enters protected mode, initializes
core kernel services, and provides an interactive VGA text-mode shell with a
heap-backed in-memory file system.

This repository is now set up as a working, testable hobby OS rather than a
mock UI. It is not a replacement for a desktop operating system yet: there is no
network stack, persistent disk driver, browser engine, or user-mode process
isolation. The current focus is a stable kernel base that can be extended safely.

## Current Status

- Bootable Multiboot ISO via GRUB.
- 32-bit x86 protected-mode kernel.
- GDT, IDT, PIC IRQ handling, CPU exception panic handling, and syscall entry.
- VGA text terminal with clean ASCII-safe boot UI and hardware cursor updates.
- PS/2 keyboard driver with buffered input.
- PIT timer and RTC clock support.
- Low-memory first-fit heap allocator with `kmalloc`/`kfree` reuse.
- RAMFS with nested paths, create/read/write/append/delete/rename/copy support.
- Interactive shell with Unix-like utilities, logs, environment variables, aliases,
  history, and file redirection.
- Built-in `selftest` command for kernel/RAMFS/heap smoke checks.
- Built-in `assist` / `ai` command for smart command suggestions and quick actions.

For the full AI-native OS roadmap, see [PLAN.md](PLAN.md).

## Quick Start

On Windows, the easiest path is WSL with QEMU:

```bash
wsl sudo apt update
wsl sudo apt install -y build-essential gcc-multilib nasm grub-pc-bin grub-common xorriso qemu-system-x86
wsl make clean all iso
wsl make qemu
```

From a Linux shell inside the repo:

```bash
make clean all iso
make qemu
```

The generated bootable image is `huggingOs.iso`.

## Verify The OS

After boot, run:

```text
selftest
```

Expected result:

```text
huggingOS selftest
[PASS] create absolute directory
[PASS] create nested file
[PASS] write file through heap
[PASS] append file through heap
[PASS] read file contents
[PASS] resolve nested absolute path
[PASS] delete directory tree
Selftest complete: all checks passed.
```

Useful smoke-test commands:

```text
help
info
mem
mkdir /notes
echo "hello from huggingOS" > /notes/readme.txt
echo "second line" >> /notes/readme.txt
cat /notes/readme.txt
assist list files
assist run memory status
dmesg
```

## Shell Commands

System:

```text
help selftest clear info version reboot shutdown whoami uname exit dmesg log
```

Date and time:

```text
date clock calendar timer uptime sleep
```

File system:

```text
ls mkdir cd pwd cat touch rm mv cp find df du
```

Text processing:

```text
grep wc head tail sort
```

Utilities:

```text
echo assist ai calc color banner about history mem env export alias unalias
test true false basename dirname which
```

Fun:

```text
moti joke fortune
```

## Smart Assistant

`assist` is intentionally local and deterministic. It maps natural-language
requests to built-in shell commands:

```text
assist memory
assist run list files
assist create file notes.txt
```

External AI API calls are not enabled yet because huggingOS does not currently
include a network stack, TLS, DNS, or persistent secret storage. The repo is ready
for those pieces to be added in later kernel milestones without pretending that
they exist today.

## Build Targets

```bash
make all      # build build/kernel.bin
make iso      # build huggingOs.iso
make qemu     # boot the ISO in QEMU
make clean    # remove generated artifacts
make help     # show build help
```

## Project Layout

```text
boot/grub/grub.cfg          GRUB menu configuration
kernel/boot.asm             Multiboot entry point and stack setup
kernel/kernel.c             Kernel initialization and main loop
kernel/gdt.*                Global Descriptor Table setup
kernel/interrupts.*         IDT, ISR, IRQ, and syscall interrupt entry
kernel/memory/              Heap and memory accounting
kernel/drivers/             VGA, keyboard, PIT, RTC, and VESA stubs
kernel/fs/                  RAMFS implementation
kernel/terminal/            Terminal wrapper and command shell
kernel/sys/                 Kernel logging
kernel/syscalls/            Syscall dispatcher
kernel/lib/                 Freestanding libc-style helpers
Makefile                    Main build system
```

## Current Limitations

- Single kernel address space; no user mode or process scheduler yet.
- RAMFS is in-memory only and is cleared on reboot.
- VESA is a safe stub; graphics are currently VGA text mode.
- No networking, TCP/IP, DNS, TLS, browser, or external AI API integration yet.
- No persistent disk filesystem driver yet.

## Troubleshooting

If `make iso` cannot find GRUB tools, install `grub-pc-bin`, `grub-common`, and
`xorriso`.

If QEMU shows a black screen, rebuild from scratch:

```bash
make clean all iso
qemu-system-i386 -cdrom huggingOs.iso -m 128M -vga std
```

If keyboard input seems stuck, click/focus the VM window. For automated testing,
QEMU monitor `sendkey` commands work with the interactive shell.

## Development Notes

The current v2.0 work fixed the most serious stability issue: heap allocations
previously targeted an unmapped high-half address. The heap now starts after the
kernel image in low memory, supports split/merge reuse, and is bounded by the
reported memory size. RAMFS and shell file redirection now use that allocator
instead of relying on dummy or unsafe behavior.

## References

- [OSDev Wiki](https://wiki.osdev.org/)
- [GNU GRUB Multiboot](https://www.gnu.org/software/grub/manual/multiboot/)
- [Intel Software Developer Manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
