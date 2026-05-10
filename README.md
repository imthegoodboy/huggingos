# huggingOS

huggingOS is now planned as a Linux-kernel-based AI operating system layer. The
goal is a normal working OS experience with real networking, filesystems,
process isolation, desktop integration, app control, memory, and AI automation.

This repository has two tracks:

- Product track: the main path under `product/`, built on the Linux kernel and
  Linux userspace services.
- Kernel-lab track: the existing bootable 32-bit x86 hobby kernel under
  `kernel/`, kept for low-level OS experiments and learning.

The current product work has completed the Linux foundation, the first
capability control plane, the first Rust AI planning bridge, the first
permissioned Linux desktop-control slice, the first screen/context observation
engine, local memory/semantic file search, and permissioned multi-agent
orchestration, plus the first predictive/self-healing suggestion layer. The
first plugin SDK, trust-metadata slice, cryptographically verified local plugin
package path, and first plugin approval surface are also present for
manifest-based third-party extensions. The custom kernel is already a working
QEMU-bootable lab OS, but it is not the production AI OS path.
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
- Phase 3 Rust AI bridge exists under `product/agent/` for deterministic local
  planning, redacted provider readiness, and plan-execute-verify execution.
- Phase 4 desktop capabilities exist for desktop readiness, app listing,
  confirmed app launch, confirmed browser URL opening, and workspace mode
  planning.
- Phase 5 screen/context capabilities exist for screen readiness, permissioned
  screenshot capture, active-context snapshots, OCR image reads, and privacy
  redaction.
- Phase 6 memory capabilities exist for session facts, preferences, audit-event
  history, opt-in semantic file indexing/search, export/delete, and resume
  planning.
- Phase 7 agent capabilities exist for a built-in agent catalog, delegation
  plans, confirmed orchestration, and trace listing.
- Phase 8 predictive/self-healing capabilities exist for repeated workflow
  detection, proactive suggestions, recoverable failure diagnosis, and recent
  activity timelines.
- Phase 9 plugin capabilities exist for manifest validation, install, catalog,
  workflow planning, read-only plugin capability runs, disable, and remove.
- Phase 10 plugin trust capabilities exist for package metadata validation,
  permission review, sandbox declarations, rollback metadata, and audited trust
  state.
- Phase 11 plugin signature capabilities exist for local signed package
  verification, install-time signature enforcement, package update metadata,
  persisted rollback manifests, and tamper rejection.
- Phase 12 plugin approval surface capabilities exist for desktop-ready review
  payloads covering trust, permissions, sandbox, update, rollback, and confirmed
  next actions.
- Product readiness audit capability exists for machine-readable production
  gates over capabilities, audit scoping, plugin trust, approval surfaces, docs,
  smoke targets, and known deferred work.
- Current capabilities include `product.status`, `fs.list`, `fs.read_text`,
  `notes.create`, `audit.list`, `desktop.status`, `apps.list`, `apps.launch`,
  `browser.open_url`, `workspace.mode.plan`, `screen.status`,
  `screen.capture`, `context.snapshot`, `screen.ocr_image`,
  `memory.session.remember`, `memory.session.list`, `memory.preference.set`,
  `memory.preference.list`, `memory.delete`, `memory.export`,
  `memory.event.list`, `files.semantic.index`, `files.semantic.search`,
  `workspace.resume.plan`, `agents.catalog`, `agents.plan`,
  `agents.orchestrate`, `agents.trace.list`, `proactive.workflow.detect`,
  `proactive.suggest`, `selfheal.diagnose`, `timeline.explain`,
  `plugins.validate`, `plugins.install`, `plugins.catalog`,
  `plugins.workflow.plan`, `plugins.capability.run`, `plugins.disable`, and
  `plugins.remove`, plus `plugins.package.validate` and
  `plugins.permission.review`, `plugins.approval.surface`, and
  `product.readiness.audit`.
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
cd product/agent && cargo run -- run product.status --json
cd product/agent && cargo run -- ai run "show product status" --json
cd product/agent && cargo run -- run desktop.status --json
cd product/agent && cargo run -- run browser.open_url --param url=https://example.com --dry-run --json
cd product/agent && cargo run -- run screen.status --json
cd product/agent && cargo run -- run screen.capture --dry-run --json
cd product/agent && cargo run -- run context.snapshot --confirm --json
cd product/agent && cargo run -- ai run "what is open" --confirm --json
cd product/agent && cargo run -- run memory.session.remember --param key=current-goal --param value=phase-six-seven --json
cd product/agent && cargo run -- run files.semantic.index --param root=../../docs --confirm --json
cd product/agent && cargo run -- run agents.orchestrate --param "goal=daily brief" --confirm --json
cd product/agent && cargo run -- run plugins.approval.surface --param source=../plugins/hello-assistant --json
cd product/agent && cargo run -- run product.readiness.audit --json
python3 -m unittest discover -s product/tests -p "test_*.py"
```

Or use the make targets:

```bash
make product-status
make product-capabilities
make product-run-status
make product-agent-ai-run
make product-agent-desktop-status
make product-agent-browser-dry-run
make product-agent-screen-status
make product-agent-screen-capture-dry-run
make product-agent-context-snapshot
make product-agent-memory-remember
make product-agent-semantic-index
make product-agent-agents-orchestrate
make product-agent-plugin-approval-surface
make product-agent-readiness-audit
make product-agent-smoke
make product-smoke
```

The Rust product agent now plans simple local AI intents and executes them
through registry, policy, verification, and audit. It also has real desktop
session/app/browser contracts, permissioned screen/context observation,
user-controlled local memory, opt-in file search, permissioned multi-agent
orchestration, signed local plugin package verification, desktop-ready plugin
approval payloads, and a machine-readable readiness audit gate. Cloud AI
provider execution, browser DOM automation, global hotkeys, rendered overlays,
accessibility-tree extraction, and window arrangement remain intentionally
deferred until their backends and permission models exist.

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
- No cloud AI provider execution is implemented yet.
- No XDG Desktop Portal/PipeWire capture path, accessibility tree, desktop
  overlay, browser DOM automation, browser tab context, window arrangement,
  cloud embeddings, arbitrary plugin code execution, plugin daemons, or
  long-running orchestrator daemon is implemented yet.

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

The next product work should start after Product Phase 12 by adding sandbox
architecture, signed archive bundles, and richer signed plugin package update
flows.

## References

- [OSDev Wiki](https://wiki.osdev.org/)
- [GNU GRUB Multiboot](https://www.gnu.org/software/grub/manual/multiboot/)
- [Intel Software Developer Manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
