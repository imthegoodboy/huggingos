# Product Phase 4 Desktop Control

Date: 2026-05-10

Area: ai | security

Related:

- Issue: [#56](https://github.com/imthegoodboy/huggingos/issues/56)
- PR: Phase 4 implementation PR
- Files: `product/agent/src/main.rs`, `product/PHASE4.md`,
  `docs/adr/0006-desktop-control-capabilities.md`

## Finding

Desktop control now starts as Rust capabilities, not shell snippets or screen
scraping. App launches and browser opens are medium-risk actions and require
confirmation unless they are dry runs.

## Why It Matters

Future overlay, hotkey, browser automation, and window-management work can be
tempting to implement as direct process or UI commands. That would bypass audit
and make the OS less trustworthy.

## Rule For Future Agents

Add desktop features as capabilities first. Use desktop-native APIs and safe
IDs, require confirmation for mutating actions, and keep headless sessions
honest with clear readiness errors.

## Evidence / Validation

Validated with Rust unit tests, desktop status, app listing, browser dry-run,
workspace mode planning, and CI smoke coverage.
