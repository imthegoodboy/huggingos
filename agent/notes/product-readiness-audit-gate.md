# Product Readiness Audit Gate Notes

## Context

The repo now has an executable product readiness gate in the Rust agent.

## What Future Agents Should Know

- Run `product.readiness.audit` before claiming the product track is ready.
- The schema is `huggingos.product.readiness.v1`.
- The gate checks capability registration, audit path scoping, dangerous
  feature flags, required trust/readiness controls, signed sample plugin trust,
  approval-surface generation, docs, smoke targets, and the plugin
  code-execution block.
- A passing gate still lists deferred work. Do not remove those limitations
  until the referenced feature actually exists and is tested.
- Keep this gate current when adding sandboxing, signed archives, update feeds,
  rendered UI, or cloud providers.

## Evidence

- Source: `product/agent/src/main.rs`
- Docs: `product/PRODUCTION_READINESS.md`
- ADR: `docs/adr/0015-product-readiness-audit-gate.md`
