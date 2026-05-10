# ADR 0007: Screen And Context Observation Uses Permissioned Capabilities

Date: 2026-05-10

Status: Accepted

## Context

Product Phase 5 needs screen and active-work context so the AI command center
can answer questions like "what is open?" and later reason about visible UI.
Screen capture, OCR, clipboard access, and active-window observation can expose
sensitive data, so they must not bypass policy, confirmation, privacy checks, or
audit logging.

## Decision

Add screen and context observation as Rust agent capabilities:

- `screen.status` for graphical session, capture backend, OCR backend,
  active-context backend, and privacy policy readiness.
- `screen.capture` for confirmed screenshot capture into the configured safe
  workspace.
- `context.snapshot` for confirmed active-window and system-context metadata.
- `screen.ocr_image` for confirmed OCR over a provided local image path.

Use discovered Linux tools for the first hosted slice: `grim`,
`gnome-screenshot`, `spectacle`, `scrot`, ImageMagick `import`, `xdotool`, and
`tesseract`. Do not fake screen contents, OCR output, browser tabs, or
accessibility trees when a backend is unavailable.

Confirmed screen capture must have active-window context backend and metadata
available first, so privacy exclusions can reject private windows before
capture. Clipboard content is reported as disabled until explicit consent and
redaction rules exist.

## Consequences

- Screen/context features inherit registry, policy, confirmation, verifier, and
  JSONL audit behavior.
- Headless CI can validate contracts with status and dry-run paths.
- Real confirmed screen capture works only on Linux desktops with a supported
  backend and non-private active context.
- Sensitive window titles/apps are redacted, and confirmed capture is denied for
  private active contexts.
- Future XDG Desktop Portal, PipeWire, accessibility tree, browser context, and
  overlay work must extend these capabilities instead of adding unchecked screen
  scraping.
