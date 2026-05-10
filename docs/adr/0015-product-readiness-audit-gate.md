# ADR 0015: Product Readiness Needs An Executable Gate

Date: 2026-05-10

Status: Accepted

## Context

The product track now has multiple working slices: capability control, AI
planning, desktop contracts, screen/context observation, memory, agents,
predictive suggestions, plugin signing, and plugin approval surfaces. Without a
single executable readiness report, agents can accidentally claim production
readiness from docs alone.

## Decision

Add `product.readiness.audit` as a read-only Rust capability. It returns
`huggingos.product.readiness.v1` and checks the current product contract:
registered capabilities, audit path scoping, dangerous feature flags, required
trust/readiness feature flags, signed sample plugin trust, approval-surface
generation, docs, real smoke targets, and the plugin code-execution block.

The report must include known deferred work. A passing readiness gate does not
mean arbitrary OS autonomy, cloud AI execution, browser DOM automation,
sandboxed plugin code, or rendered overlay UI exists.

## Consequences

- CI and future agents have a concrete readiness command.
- Production claims are tied to executable checks instead of narrative docs.
- Deferred work stays visible and cannot be silently hidden behind green
  status text.
- The gate can grow as later phases add sandboxing, archive bundles, update
  feeds, and rendered UI.
