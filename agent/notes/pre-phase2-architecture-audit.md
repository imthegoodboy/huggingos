# Pre-Phase 2 Architecture Audit

Date: 2026-05-10

Area: product, architecture, tooling

Related:

- Files: `.gitattributes`, `product/architecture.md`
- ADR: `docs/adr/0003-capability-control-plane.md`

## Finding

Phase 1 is the right direction: Linux is the base OS and huggingOS is the AI
control layer above it. The main issue found before Phase 2 was operational:
Windows checkout line endings broke direct execution of `product/cli/huggingos.py`
under WSL because the shebang became CRLF.

## Why It Matters

Phase 2 will add more Linux-facing scripts and product entrypoints. If line
endings are not controlled, commands can pass through `python3 script.py` but
fail when invoked as executables.

## Rule For Future Agents

- Keep Linux-facing scripts and Makefiles LF-only.
- Run both `wsl python3 product/cli/huggingos.py doctor` and
  `wsl ./product/cli/huggingos.py doctor` when changing CLI entrypoints.
- Build broad agent control through the capability control plane, not arbitrary
  shell execution.

## Evidence / Validation

The failure mode was:

```text
/usr/bin/env: 'python3\r': No such file or directory
```

`.gitattributes` now enforces LF for scripts, source, Makefiles, TOML, and
workflow files.
