# Product Phase 12 Plugin Approval Surface Notes

## Context

Product Phase 12 adds the first desktop-ready plugin approval contract without
building a fake overlay renderer.

## What Future Agents Should Know

- Use `plugins.approval.surface` for UI review payloads instead of duplicating
  plugin trust logic in UI code.
- The schema is `huggingos.plugin.approval.v1`.
- The surface is read-only; install, disable, remove, run, update, and rollback
  execution remain separate capabilities.
- `desktop_overlay_enabled` is still false in defaults. A future renderer must
  consume the approval payload rather than hardcoding plugin state.
- Keep plugin code execution disabled until sandboxing is implemented and
  audited.

## Evidence

- Source: `product/agent/src/main.rs`
- UI contract docs: `product/ui/README.md`
- Phase docs: `product/PHASE12.md`
