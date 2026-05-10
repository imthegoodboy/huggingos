## Summary


## Issue

Closes or advances:

## Track

- [ ] Product track
- [ ] Kernel-lab track
- [ ] Docs/process only

## What Changed


## Real Behavior Added

Describe the executable behavior. If this is docs-only, say so.

## Validation

Product-track validation:

- [ ] Product command or smoke test:
- [ ] No committed secrets or local runtime files.
- [ ] Product docs updated:

Kernel-lab validation:

- [ ] `make clean all iso`
- [ ] QEMU boot check
- [ ] `selftest`

Shared validation:

- [ ] Feature-specific test:
- [ ] `git diff --check`

Docs-only PRs may replace the build/QEMU checks with link and formatting checks.

## No-Hardcoding / No-Fake-Feature Check

- [ ] No API keys, tokens, local paths, usernames, or machine-specific assumptions.
- [ ] No fake hardware/device addresses or fake network/AI responses.
- [ ] Unsupported behavior fails safely and is documented.
- [ ] Remaining limitations are stated clearly.

## Notes
