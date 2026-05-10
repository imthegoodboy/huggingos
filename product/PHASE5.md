# Product Phase 5: Screen And Context Engine

Phase 5 gives the Rust agent its first permissioned view of visible desktop
state. It is intentionally an observation layer, not silent surveillance and
not broad browser automation.

The implementation keeps screen and context work inside the existing capability
path:

```text
AI prompt -> local planner -> capability request -> policy -> executor -> verifier -> audit
```

## Implemented Scope

- `screen.status` reports graphical-session, capture, OCR, active-context, and
  privacy readiness.
- `screen.capture` captures a screenshot to the configured safe workspace when
  a real Linux backend is available and the user confirms the action.
- `context.snapshot` reports desktop status, active-window metadata, clipboard
  readiness, screen readiness, and privacy policy status.
- `screen.ocr_image` reads text from a provided image path through `tesseract`
  when installed and confirmed.
- Privacy markers redact active-window title/app data for sensitive windows.
- AI planner mappings for prompts such as "what is open?", "take a
  screenshot", "screen status", and "ocr image <path>".

## Commands

From `product/agent/`:

```bash
cargo run -- run screen.status --json
cargo run -- run screen.capture --dry-run --json
cargo run -- run screen.capture --confirm --json
cargo run -- run context.snapshot --confirm --json
cargo run -- run screen.ocr_image --param path=../../README.md --dry-run --json
cargo run -- run screen.ocr_image --param path=/path/to/image.png --confirm --json
cargo run -- ai plan "what is open" --json
cargo run -- ai run "what is open" --confirm --json
cargo run -- ai plan "take a screenshot" --json
```

`screen.capture`, `context.snapshot`, and `screen.ocr_image` are medium-risk
capabilities. Use `--dry-run` for CI and headless sessions. Confirmed screen
capture refuses to run unless active-window context backend and metadata are
also present, so the privacy policy can evaluate what is visible before capture.

## Linux Backends

The agent discovers host backends instead of hardcoding one desktop:

- Screen capture: `grim`, `gnome-screenshot`, `spectacle`, `scrot`, or
  ImageMagick `import`.
- Active window context: `xdotool`.
- OCR: `tesseract`.
- Desktop/session readiness: the Phase 4 desktop status checks.

Missing tools are reported in `screen.status`. The capabilities fail with clear
errors instead of pretending the desktop was observed.

## Privacy Model

The default config includes privacy markers for active app/window text:

```toml
[privacy]
private_title_markers = [
  "password", "secret", "token", "credential", "private",
  "incognito", "bank", "vault", "2fa", "otp",
]
private_app_markers = ["password", "secret", "credential", "vault", "bank"]
max_context_text_chars = 240
```

When a marker matches, context metadata is redacted. Confirmed screenshots are
blocked for private active contexts. OCR also refuses obvious secret-like paths.

This is the first privacy layer. Future work should move capture to XDG Desktop
Portal/PipeWire where available and add region-level exclusions once the
desktop service exists.

## Capability Model

- `screen.status`: read-only capability.
- `screen.capture`: medium-risk capability that writes a screenshot under the
  configured safe workspace.
- `context.snapshot`: medium-risk capability because it observes active work.
- `screen.ocr_image`: medium-risk capability because image text may contain
  sensitive information.

All execution is policy-checked, auditable, and verifier-backed. No screenshots,
OCR results, or context data are produced from fake fixtures.

## What Is Still Later

- XDG Desktop Portal/PipeWire capture flow.
- Accessibility tree extraction.
- Browser tab and DOM context.
- Clipboard content reading with explicit consent.
- Region-level redaction.
- Long-running desktop service and global overlay.

These belong after the user service and richer permission model are in place.

## Validation

```bash
cd product/agent
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo run -- run screen.status --json
cargo run -- run screen.capture --dry-run --json
cargo run -- run context.snapshot --confirm --json
cargo run -- run screen.ocr_image --param path=../../README.md --dry-run --json
cargo run -- ai plan "what is open" --json
cargo run -- ai run "what is open" --confirm --json
cargo run -- ai plan "take a screenshot" --json
```
