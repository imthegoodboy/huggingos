# Product Production Readiness

This file defines the current executable readiness gate for the Linux product
track. It is intentionally honest: the current repo is ready for the completed
product slices, not for arbitrary full-OS autonomy.

## Readiness Command

From the repository root:

```bash
make product-agent-readiness-audit
```

From `product/agent/`:

```bash
cargo run -- run product.readiness.audit --json
```

The command returns `huggingos.product.readiness.v1`.

## What The Gate Checks

- Required production-track capabilities are registered.
- The audit log resolves under the configured state directory.
- High-trust integrations stay disabled until their backends exist.
- Required plugin trust and readiness feature flags are enabled.
- The sample plugin package verifies cryptographically.
- The sample plugin approval surface can be generated.
- Current phase, architecture, UI, plan, and update docs exist.
- Root Make targets expose real product and plugin trust smoke checks.
- Plugin code execution, auto-update, and automatic rollback remain blocked.

## Current Known Deferred Work

- Rendered desktop overlay and control center screens.
- Sandboxed plugin-provided code execution.
- Signed archive bundles beyond manifest-only packages.
- Trusted plugin update feeds and manual update approval.
- Automatic rollback execution.
- Cloud AI provider execution.
- Browser DOM automation and accessibility-tree extraction.

These are not hidden behind success messages. The readiness audit reports them
as known deferred work so future agents keep building in the right order.

## Full Local Validation

Use this before publishing product readiness changes:

```bash
cd product/agent
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cd ../..
python3 -m unittest discover -s product/tests -p "test_*.py"
make product-agent-readiness-audit
make product-agent-plugin-package-validate
make product-agent-plugin-approval-surface
make clean all iso
```
