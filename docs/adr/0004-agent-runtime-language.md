# ADR 0004: Production Agent Runtime Language

Status: accepted

Date: 2026-05-10

## Context

Product Phase 2 introduced the capability control plane in Python so the action
schema, policy rules, audit behavior, and first local capabilities could be
validated quickly. That was useful for shaping the contract, but the long-term
agent runtime will sit close to the OS: it will run as a user service, manage
local files and audit logs, broker permissions, and later talk to desktop APIs.

That layer needs strong correctness, safe concurrency, predictable deployment,
and a single-binary path for Linux images.

## Decision

Use Rust as the production language for the huggingOS agent runtime and local
capability daemon.

The Python implementation remains a Phase 2 reference/prototype until the Rust
runtime reaches feature parity and can replace it. New production agent/runtime
work should target `product/agent/` first.

## Why Rust

- Memory safety without a garbage collector.
- Strong type system for capability contracts and policy decisions.
- Good fit for long-running Linux user services.
- Fast startup and low runtime overhead.
- Single-binary packaging for future distro/live-image work.
- Strong ecosystem for async services, D-Bus, desktop portals, keyrings, and
  observability when later phases need them.

## Go Considered

Go is also a strong option for services and would be faster to build in some
areas. Rust is preferred here because the agent runtime is an OS control layer
where type safety, memory safety, and precise resource control matter more than
maximum iteration speed.

Go can still be used later for separate tooling if it clearly fits a task, but
it is not the primary agent runtime.

## Consequences

Easier:

- Safer production daemon and capability execution path.
- Cleaner packaging into Linux product images.
- Better foundation for desktop/system integration.

Harder:

- Initial implementation is more verbose than Python.
- Contributors need a Rust toolchain.
- Rapid AI provider experiments may still be quicker in Python until stable
  contracts are ready.

## Validation

The first Rust crate must:

- Build and test in CI.
- Mirror the Phase 2 capability safety model.
- Deny obvious secret paths until a higher-risk secret capability exists.
- Fail closed when audit logging is unavailable.
- Keep the Python reference passing until it is intentionally retired.
