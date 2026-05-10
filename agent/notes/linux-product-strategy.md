# Linux Product Strategy

Date: 2026-05-10

Area: docs, ai, security

Related:

- ADR: `docs/adr/0001-kernel-strategy.md`
- Files: `PLAN.md`, `product/README.md`

## Finding

The main huggingOS product should use the Linux kernel. The custom x86 kernel
remains as a kernel-lab track, but product AI features should be built as Linux
userspace services, desktop integrations, capability APIs, and packages.

## Why It Matters

The AI OS vision needs real networking, TLS, filesystems, process isolation,
drivers, GUI integration, browser control, app automation, and secret storage.
Linux provides these foundations now. Rebuilding them in the hobby kernel would
delay the useful product and tempt agents into fake features.

## Rule For Future Agents

- Put product AI features under the Linux product track.
- Use the custom kernel only for kernel-lab issues.
- Do not add cloud AI, browser control, or persistent memory to the hobby kernel
  and claim it is production OS behavior.
- Keep Linux product work permissioned, auditable, and free of hardcoded secrets.

## Evidence / Validation

This is an architecture decision recorded before Product Phase 1 starts. Product
Phase 1 must validate it with a real Linux product foundation and smoke test.
