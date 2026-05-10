# Product Phase 5 Screen Context

Date: 2026-05-10

## Finding

Phase 5 screen/context work belongs in the Rust agent capability layer, not in
the custom kernel and not as ad hoc shell automation. Screen capture and OCR are
sensitive observation actions, so they are medium-risk capabilities with policy,
confirmation, audit, and privacy checks.

## Guidance

- Use `screen.status` first to see which host backends are available.
- Use `--dry-run` for headless CI and WSL validation.
- Confirmed `screen.capture` requires active-context backend and metadata so
  privacy markers can block private windows before capture.
- Do not read clipboard contents by default. Phase 5 reports clipboard readiness
  only.
- Do not fake OCR or screen contents. If `tesseract` or a capture backend is
  missing, return a clear backend error.
- Future portal/PipeWire/accessibility/browser-tab work should extend the same
  capability names or add typed sibling capabilities, not bypass the registry.

## Evidence

- Implementation: `product/agent/src/main.rs`
- Phase doc: `product/PHASE5.md`
- ADR: `docs/adr/0007-screen-context-engine.md`
- Tracking epic: https://github.com/imthegoodboy/huggingos/issues/65
