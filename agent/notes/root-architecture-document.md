# Root Architecture Document

Date: 2026-05-10

Area: architecture, product, agents

Related:

- File: `ARCHITECTURE.md`
- File: `product/architecture.md`

## Finding

The root `ARCHITECTURE.md` is now the project-wide north-star document. It
describes what huggingOS is, how Linux is used, how agents get broad control,
and which technologies should be used now, later, or avoided.

## Why It Matters

Future agents need one high-level source of truth before implementing Phase 2
and beyond. The product must stay focused on the capability control plane rather
than drifting into unsafe shell execution or fake AI features.

## Rule For Future Agents

- Read `ARCHITECTURE.md` before starting a new phase or major subsystem.
- Keep agent control behind typed capabilities, policy, verifier, and audit.
- Add or update ADRs when changing a major architecture decision.
- Keep source-backed technology choices current when a phase depends on them.

## Evidence / Validation

The document was created after checking current primary documentation for Ubuntu,
Debian, Buildroot, Yocto, MCP, XDG Desktop Portal, PipeWire, OpenTelemetry, and
eBPF.
