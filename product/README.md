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
For the Product Phase 2 capability layer, see [PHASE2.md](PHASE2.md).
For the product architecture, see [architecture.md](architecture.md).

## Planned Structure

```text
product/
  README.md              Product-track entry point
  huggingos_core/        Capability, policy, audit, and config library
  agent/                 Rust production agent runtime
  distro/                Base image, package, and rootfs definitions
  services/              huggingOS daemons and local APIs
  cli/                   huggingos command-line entrypoint
  ui/                    Desktop overlay and control center
  policy/                Permissions, confirmations, audit, and rollback rules
  tests/                 Product smoke tests
```

These folders should be created when the matching implementation issue starts.
Do not fill them with fake placeholders.

## Current Product Slice

The product track currently provides:

- Ubuntu LTS hosted prototype strategy.
- Reproducible dev and smoke commands.
- A real `huggingos` CLI.
- Runtime config layout with no committed secrets.
- Product smoke tests and CI.
- Phase 2 in-process capability control plane.
- Rust production agent runtime started under `product/agent/`.
- First safe capabilities for product status, file listing, small text reads,
  safe workspace note creation, and audit listing.

## Product Commands

From the repository root on Linux or WSL:

```bash
python3 product/cli/huggingos.py status
python3 product/cli/huggingos.py doctor
python3 product/cli/huggingos.py capabilities
python3 product/cli/huggingos.py run product.status
cd product/agent && cargo run -- status --json
cd product/agent && cargo run -- run product.status --json
python3 -m unittest discover -s product/tests -p "test_*.py"
```

Or with `make`:

```bash
make product-status
make product-doctor
make product-capabilities
make product-run-status
make product-agent-smoke
make product-smoke
```

From inside `product/`:

```bash
make status
make doctor
make capabilities
make run-status
make agent-smoke
make smoke
```

## Capability Examples

All real automated actions should go through the capability control plane:

```bash
python3 product/cli/huggingos.py capabilities
python3 product/cli/huggingos.py run fs.list --param path=.
python3 product/cli/huggingos.py run fs.read_text --param path=product/README.md
python3 product/cli/huggingos.py run notes.create --param title=PhaseTwo --param content="real note"
python3 product/cli/huggingos.py run audit.list --param limit=10
```

Use `HUGGINGOS_STATE_DIR` to move local audit/runtime state, and
`HUGGINGOS_WORKSPACE_DIR` to constrain low-risk workspace writes. The default
audit log is JSON Lines at the product state path.

The CLI still does not call AI providers, control browsers, launch arbitrary
shell commands, or manage desktop apps. Those come after the safe capability,
secret, and desktop integration phases.

The Rust agent currently mirrors the Phase 2 capability safety model and is the
production path for the future daemon. The Python CLI remains a reference
control surface until Rust reaches full parity.

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
