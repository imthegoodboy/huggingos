# Product Phase 4: Desktop Command Center And App Control

Phase 4 makes the Rust agent aware of the Linux desktop without bypassing the
capability control plane.

This is the first real desktop-control slice. It does not scrape pixels and it
does not pretend a browser was automated. Desktop actions are explicit
capabilities with policy, confirmation, audit, and dry-run support.

## Implemented Scope

- Desktop/session readiness detection through environment and host tool checks.
- Installed application listing from `.desktop` entries.
- Confirmed application launch by `.desktop` ID through `gio` or `gtk-launch`.
- Confirmed browser URL opening through `xdg-open` or `gio open`.
- Workspace mode planning for `coding`, `study`, `deep-work`, `gaming`, and
  `travel`.
- AI planner mappings for desktop status, app listing/launch, browser open, and
  workspace modes.

## Commands

From `product/agent/`:

```bash
cargo run -- run desktop.status --json
cargo run -- run apps.list --json
cargo run -- run apps.launch --param app_id=firefox.desktop --dry-run --json
cargo run -- run apps.launch --param app_id=firefox.desktop --confirm --json
cargo run -- run browser.open_url --param url=https://example.com --dry-run --json
cargo run -- run browser.open_url --param url=https://example.com --confirm --json
cargo run -- run workspace.mode.plan --param mode=coding --json
cargo run -- ai plan "open browser https://example.com" --json
cargo run -- ai run "switch to coding mode" --json
```

`apps.launch` and `browser.open_url` are medium-risk. Without `--confirm`, they
return `confirmation_required`. In headless CI or WSL sessions, use `--dry-run`
or run from a graphical Linux desktop.

## Capability Model

- `desktop.status`: read-only desktop readiness.
- `apps.list`: read-only `.desktop` registry listing.
- `apps.launch`: medium-risk app launch by safe visible desktop ID. Hidden and
  `NoDisplay` entries are refused.
- `browser.open_url`: medium-risk HTTP/HTTPS URL open.
- `workspace.mode.plan`: read-only mode preview.

All execution still flows through:

```text
AI prompt -> local planner -> capability request -> policy -> executor -> verifier -> audit
```

## What Is Still Later

- Global hotkey daemon.
- Graphical overlay/sidebar.
- Window placement and workspace arrangement.
- Browser DOM automation.
- Notification controls.
- Screen capture, OCR, and accessibility-tree context.

Those require a long-running desktop service and stronger per-action permission
rules. The Phase 4 slice creates the safe contracts they will use.

## Validation

```bash
cd product/agent
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo run -- run desktop.status --json
cargo run -- run apps.list --json
cargo run -- run browser.open_url --param url=https://example.com --dry-run --json
cargo run -- run workspace.mode.plan --param mode=coding --json
```
