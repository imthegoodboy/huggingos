# Product Phase 1 Kickoff

This file is the build-start checklist for the Linux product track. It keeps the
first implementation work small, real, and reviewable.

## Active Tracking

- Epic: [#13 Product Phase 1 Linux product foundation](https://github.com/imthegoodboy/huggingos/issues/13)
- Base strategy: [#14 Choose Linux base image strategy](https://github.com/imthegoodboy/huggingos/issues/14)
- Product tree and commands: [#15 Create product tree and dev/build commands](https://github.com/imthegoodboy/huggingos/issues/15)
- CLI: [#17 Add first huggingos CLI](https://github.com/imthegoodboy/huggingos/issues/17)
- Config and secrets: [#18 Add runtime config layout and no-secret policy](https://github.com/imthegoodboy/huggingos/issues/18)
- Smoke test and CI: [#19 Add product smoke test and CI](https://github.com/imthegoodboy/huggingos/issues/19)

## Build Order

1. Decide and document the Linux base strategy in #14. Done in ADR 0002.
2. Add only the product folders needed by the first runnable slice in #15. Done
   for CLI, config, distro, policy, services, and tests.
3. Add a real `huggingos` CLI command in #17. Done with `status`, `doctor`, and
   `config`.
4. Add runtime config and secret rules before any AI provider work in #18. Done
   with `product/config/defaults.toml`, `.env.example`, and ignore rules.
5. Add local smoke tests and CI in #19. Done with `product/tests` and the
   Product Phase 1 workflow.

## First Slice Rule

The first runnable product slice should prove the repo can execute product code
from a fresh checkout. It should not claim to be a full AI OS yet.

Minimum acceptable first slice:

- A documented command starts the product prototype or CLI.
- The CLI reports real local product/system information.
- Missing dependencies produce clear errors.
- No root permission is required unless the action truly needs it.
- No API key, token, username, absolute host path, generated image, or private
  artifact is committed.

## Validation Commands

From the repository root:

```bash
python3 product/cli/huggingos.py status
python3 product/cli/huggingos.py doctor
python3 -m unittest discover -s product/tests -p "test_*.py"
```

With `make`:

```bash
make product-smoke
```

## Not Yet

Do not start these until their prerequisites exist:

- Cloud AI providers before config, secret loading, policy, and audit are ready.
- Browser automation before a browser capability backend exists.
- Desktop overlay before the CLI and capability boundary are stable.
- Persistent memory before storage, retention, inspect, export, and delete
  controls are designed.
