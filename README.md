# huggingOS

huggingOS is now planned as a Linux-kernel-based AI operating system layer. The
goal is a normal working OS experience with real networking, filesystems,
process isolation, desktop integration, app control, memory, and AI automation.

This repository has two tracks:

- Product track: the main path under `product/`, built on the Linux kernel and
  Linux userspace services.
- Kernel-lab track: the existing bootable 32-bit x86 hobby kernel under
  `kernel/`, kept for low-level OS experiments and learning.

The current product work has completed the Linux foundation and the first
capability control plane. The custom kernel is already a working QEMU-bootable
lab OS, but it is not the production AI OS path.
For the full roadmap, see [PLAN.md](PLAN.md). For the kernel decision, see
[docs/adr/0001-kernel-strategy.md](docs/adr/0001-kernel-strategy.md).
For the full system architecture, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Current Status

Product track:

- Linux product direction selected.
- `product/README.md` is the entry point for the Linux OS layer.
- Runtime config, CLI, service, packaging, and smoke-test structure exists.
- Phase 2 capability layer exists for typed, policy-checked, audited local
  actions.
- Current capabilities: `product.status`, `fs.list`, `fs.read_text`,
  `notes.create`, and `audit.list`.
- No committed API keys, provider secrets, or fake AI integrations.

Kernel-lab track:

- Bootable Multiboot ISO via GRUB.
- 32-bit x86 protected-mode kernel.
- GDT, IDT, PIC IRQ handling, CPU exception panic handling, and syscall entry.
- VGA text terminal with clean ASCII-safe boot UI and hardware cursor updates.
- PS/2 keyboard driver with buffered input.
- PIT timer and RTC clock support.
- Low-memory first-fit heap allocator with `kmalloc`/`kfree` reuse.
- RAMFS with nested paths, create/read/write/append/delete/rename/copy support.
- Interactive shell with Unix-like utilities, logs, environment variables,
  aliases, history, and file redirection.
- Built-in `selftest` command for kernel/RAMFS/heap smoke checks.
- Built-in `assist` / `ai` command for local deterministic command suggestions.

Future AI builders should follow [agent/SKILL.md](agent/SKILL.md) and
[agent/TASK_CHECKLIST.md](agent/TASK_CHECKLIST.md). Durable gotchas and build
lessons live in [agent/notes/INDEX.md](agent/notes/INDEX.md).

## Product Track

Start with:

```bash
python3 product/cli/huggingos.py status
python3 product/cli/huggingos.py capabilities
python3 product/cli/huggingos.py run product.status
python3 -m unittest discover -s product/tests -p "test_*.py"
```

Or use the make targets:

```bash
make product-status
make product-capabilities
make product-run-status
make product-smoke
```

The product CLI now executes safe local capabilities through registry, policy,
verification, and audit. AI providers, browser control, and desktop app control
are intentionally deferred until the secret and desktop integration phases.

## Kernel-Lab Quick Start

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

The generated bootable lab image is `huggingOs.iso`.

## Verify The Kernel Lab

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

The kernel-lab `assist` command is intentionally local and deterministic. It maps
natural-language requests to built-in shell commands:

```text
assist memory
assist run list files
assist create file notes.txt
```

External AI API calls belong in the Linux product track after networking,
secret storage, provider configuration, policy, and audit logging exist. Do not
add API keys or cloud calls directly to the custom kernel.

## Build Targets

Kernel-lab targets:

```bash
make all      # build build/kernel.bin
make iso      # build huggingOs.iso
make qemu     # boot the ISO in QEMU
make clean    # remove generated artifacts
make help     # show build help
```

## Project Layout

```text
product/                    Linux product track entry point
docs/adr/                   Architecture decision records
agent/                      Agent rules, commands, checklist, and notes
boot/grub/grub.cfg          GRUB menu configuration for kernel lab
kernel/boot.asm             Multiboot entry point and stack setup
kernel/kernel.c             Kernel-lab initialization and main loop
kernel/gdt.*                Global Descriptor Table setup
kernel/interrupts.*         IDT, ISR, IRQ, and syscall interrupt entry
kernel/memory/              Heap and memory accounting
kernel/drivers/             VGA, keyboard, PIT, RTC, and VESA stubs
kernel/fs/                  RAMFS implementation
kernel/terminal/            Terminal wrapper and command shell
kernel/sys/                 Kernel logging
kernel/syscalls/            Syscall dispatcher
kernel/lib/                 Freestanding libc-style helpers
Makefile                    Kernel-lab build system
```

## Current Limitations

Product track:

- No Linux image/rootfs build is implemented yet.
- No long-running `huggingosd` daemon is implemented yet.
- No AI provider integration is implemented yet.
- No desktop overlay, browser automation, app/window control, or screen capture
  is implemented yet.

Kernel-lab track:

- Single kernel address space; no user mode or process scheduler yet.
- RAMFS is in-memory only and is cleared on reboot.
- VESA is a safe stub; graphics are currently VGA text mode.
- No networking, TCP/IP, DNS, TLS, browser, or external AI API integration.
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

The v2.0 kernel-lab work fixed the most serious stability issue: heap
allocations previously targeted an unmapped high-half address. The heap now
starts after the linked kernel image in low memory, supports split/merge reuse,
and is bounded by the reported memory size. RAMFS and shell file redirection now
use that allocator instead of relying on dummy or unsafe behavior.

The next product work should start in Product Phase 3, connecting an AI runtime
and secure secret loading to the Phase 2 capability layer without adding
advanced AI features to the custom kernel.

## References

- [OSDev Wiki](https://wiki.osdev.org/)
- [GNU GRUB Multiboot](https://www.gnu.org/software/grub/manual/multiboot/)
- [Intel Software Developer Manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
