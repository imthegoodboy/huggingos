# Agent Command Reference

Use these commands from the repository root unless a task says otherwise.

## Product Track

Use these for Linux product work:

```bash
gh issue list --label "track:product"
gh issue list --label "track:product" --milestone "Product Phase 1: Linux Product Foundation"
```

Product build and smoke commands live in `product/README.md` and the active
issue. Use the Rust agent commands there for Phase 2+ product work.

Current product checks:

```bash
make product-agent-smoke
make product-agent-ai-run
make product-agent-desktop-status
make product-agent-screen-status
make product-agent-screen-capture-dry-run
make product-agent-memory-remember
make product-agent-semantic-index
make product-agent-agents-orchestrate
make product-agent-workflow-detect
make product-agent-selfheal-diagnose
make product-agent-plugin-install
make product-agent-plugin-package-validate
make product-agent-plugin-permission-review
make product-agent-plugin-run
make product-agent-plugin-remove
make product-smoke
```

Before starting Product Phase 1:

```bash
git diff --check
rg -n "[^\x00-\x7F]" product README.md PLAN.md agent .github docs
```

## Kernel-Lab Build

```bash
make clean all iso
```

Windows/WSL:

```bash
wsl make clean all iso
```

## Kernel-Lab Run

```bash
make qemu
```

Manual QEMU:

```bash
qemu-system-i386 -cdrom huggingOs.iso -m 128M -vga std
```

## Smoke Commands Inside Kernel-Lab huggingOS

```text
selftest
assist run memory status
mem
dmesg
```

## GitHub Project Commands

```bash
gh issue list --state open
gh issue view <issue-number>
gh issue list --label "track:product"
gh issue list --label "track:kernel-lab"
gh issue list --milestone "Product Phase 1: Linux Product Foundation"
gh issue list --milestone "Product Phase 5: Screen And Context Engine"
gh issue list --milestone "Phase 1: Reliable Kernel And Shell Foundation"
gh issue list --label "track:product" --search "Phase 6"
gh issue list --label "track:product" --search "Phase 7"
gh issue list --label "track:product" --search "Phase 8"
gh issue list --label "track:product" --search "Phase 9"
gh issue list --label "track:product" --search "Phase 10"
gh pr create --fill
gh pr view --json url,state,mergeStateStatus,statusCheckRollup
```

## Hygiene

```bash
git diff --check
rg -n "[^\x00-\x7F]" kernel product README.md UPDATE.md PLAN.md agent .github docs
git ls-files | rg "(^|/)(\\.env(\\..*)?|.*(secret|token).*|.*(api[-_]?key|private[-_]?key).*)$" | rg -v "(^|/)\\.env\\.example$"
git status -sb
```

## Rule

Do not treat these commands as magic. If a command fails, inspect the error and
fix the cause. Do not silence failures or replace real checks with printed
success messages.
