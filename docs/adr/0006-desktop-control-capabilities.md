# ADR 0006: Desktop Control Uses Permissioned Capabilities

Date: 2026-05-10

Status: Accepted

## Context

Product Phase 4 needs desktop and app control, but direct UI scraping or
arbitrary shell execution would bypass the Phase 2 and Phase 3 safety model.
Desktop actions can start apps, open URLs, and affect user focus, so they need
policy, confirmation, audit, and clear headless failure behavior.

## Decision

Add desktop integration as Rust agent capabilities:

- `desktop.status` for graphical-session and backend readiness.
- `apps.list` for `.desktop` registry discovery.
- `apps.launch` for confirmed app launch by safe visible desktop ID.
- `browser.open_url` for confirmed HTTP/HTTPS URL opening.
- `workspace.mode.plan` for inspectable mode previews.

Use Linux desktop-native launch backends first: `gio`, `gtk-launch`, and
`xdg-open`. Do not add screen scraping, DOM automation, global hotkeys, or
window manipulation until a desktop service and permission model exist.

## Consequences

- Desktop actions inherit policy, confirmation, audit, and verification.
- Hidden and `NoDisplay` desktop entries are not launch targets.
- Headless CI and WSL can validate contracts with dry runs and readiness checks.
- Browser URL opening is real, but it is not represented as full browser
  automation.
- Future overlay, hotkey, window, notification, and browser automation features
  must extend the capability model instead of bypassing it.
