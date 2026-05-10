# huggingOS Product Track

This directory is for the Linux-kernel-based huggingOS product.

The product track is the path toward a normal working AI-native OS:

- Linux kernel and drivers.
- Linux userspace services.
- Real filesystems, networking, process isolation, and desktop APIs.
- AI command center and capability API.
- Secure runtime config and secret handling.
- Desktop overlay, app control, memory, and agents.

The existing `kernel/` directory remains the custom kernel-lab track. Do not add
product AI features to the hobby kernel unless an issue explicitly says to work
on the kernel-lab track.

For the Product Phase 1 kickoff sequence, see [PHASE1.md](PHASE1.md).

## Planned Structure

```text
product/
  README.md              Product-track entry point
  distro/                Base image, package, and rootfs definitions
  services/              huggingOS daemons and local APIs
  cli/                   huggingos command-line entrypoint
  ui/                    Desktop overlay and control center
  policy/                Permissions, confirmations, audit, and rollback rules
  tests/                 Product smoke tests
```

These folders should be created when the matching implementation issue starts.
Do not fill them with fake placeholders.

## Phase 1 Target

Phase 1 should produce the smallest real Linux product foundation:

- Pick the base image strategy.
- Add reproducible dev/build commands.
- Add a real `huggingos` CLI.
- Add runtime config layout with no committed secrets.
- Add smoke tests and CI.

## Phase 1 Commands

From the repository root on Linux or WSL:

```bash
python3 product/cli/huggingos.py status
python3 product/cli/huggingos.py doctor
python3 -m unittest discover -s product/tests -p "test_*.py"
```

Or with `make`:

```bash
make product-status
make product-doctor
make product-smoke
```

From inside `product/`:

```bash
make status
make doctor
make smoke
```

The CLI is intentionally small in Phase 1. It reports real product and host
state, validates the local product foundation, and reads non-secret config. It
does not automate apps, call AI providers, or change OS state yet.

## Local Files

Local runtime files, secrets, and machine-specific config are intentionally not
tracked. Use documented config examples when they exist, and keep real provider
keys in the OS keyring or local ignored files only.

## Rules

- No hardcoded API keys or local machine paths.
- No fake AI provider responses.
- No fake browser automation.
- No root-only behavior unless the action truly needs root and explains why.
- Every OS action should move toward the capability API and audit model.
