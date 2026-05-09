# Agent Command Reference

Use these commands from the repository root unless a task says otherwise.

## Build

```bash
make clean all iso
```

Windows/WSL:

```bash
wsl make clean all iso
```

## Run

```bash
make qemu
```

Manual QEMU:

```bash
qemu-system-i386 -cdrom huggingOs.iso -m 128M -vga std
```

## Smoke Commands Inside huggingOS

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
gh issue list --milestone "Phase 1: Reliable Kernel And Shell Foundation"
gh pr create --fill
gh pr view --json url,state,mergeStateStatus,statusCheckRollup
```

## Hygiene

```bash
git diff --check
rg -n "[^\x00-\x7F]" kernel README.md UPDATE.md PLAN.md agent .github docs
git status -sb
```

## Rule

Do not treat these commands as magic. If a command fails, inspect the error and
fix the cause. Do not silence failures or replace real checks with printed
success messages.
