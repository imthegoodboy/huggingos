# Product Phase 1 Foundation

Date: 2026-05-10

Area: product, cli, config, ci

Related:

- Files: `product/cli/huggingos.py`, `product/config/defaults.toml`
- Files: `product/tests/test_cli.py`, `.github/workflows/product-phase1.yml`
- ADR: `docs/adr/0002-linux-base-strategy.md`

## Finding

Product Phase 1 uses an Ubuntu LTS hosted prototype instead of a full bootable
image. The first real product behavior is a Python standard-library-only CLI
with `status`, `doctor`, and `config` commands.

## Why It Matters

This gives the product track executable Linux userspace behavior without
pretending app control, cloud AI, browser automation, or a full image already
exists.

## Rule For Future Agents

- Run `make product-smoke` or `python3 -m unittest discover -s product/tests -p
  "test_*.py"` before changing the product CLI.
- Keep the CLI usable without root.
- Keep provider secrets out of config defaults and examples.
- Add new OS-changing behavior through a capability API, not directly inside
  ad hoc CLI command bodies.

## Evidence / Validation

Validated on Ubuntu 24.04.3 LTS under WSL2 with:

- `wsl python3 product/cli/huggingos.py status --json`
- `wsl python3 product/cli/huggingos.py doctor --json`
- `wsl make product-smoke`
- `wsl make -C product smoke`
- `wsl make clean all iso product-smoke`
