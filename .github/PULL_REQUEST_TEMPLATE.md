## Summary


## Issue

Closes or advances:

## What Changed


## Real Behavior Added

Describe the executable behavior. If this is docs-only, say so.

## Validation

- [ ] `make clean all iso`
- [ ] QEMU boot check
- [ ] `selftest`
- [ ] Feature-specific test:
- [ ] `git diff --check`

Docs-only PRs may replace the build/QEMU checks with link and formatting checks.

## No-Hardcoding / No-Fake-Feature Check

- [ ] No API keys, tokens, local paths, usernames, or machine-specific assumptions.
- [ ] No fake hardware/device addresses or fake network/AI responses.
- [ ] Unsupported behavior fails safely and is documented.
- [ ] Remaining limitations are stated clearly.

## Notes
