# Product Phase 2 Capability Layer

## Context

Product Phase 2 adds the first executable capability control plane for the
Linux-based product track. The code lives in `product/huggingos_core/`, and the
CLI entrypoint is `product/cli/huggingos.py`.

## What Future Agents Should Reuse

- Register OS actions as typed capabilities with metadata, risk, permissions,
  input schema, executor, and verifier.
- Build new production runtime work in Rust under `product/agent/`; keep the
  Python layer as a reference until Rust reaches full parity.
- Execute actions through `CapabilityEngine`, not by directly calling capability
  functions from product commands.
- Keep every action auditable, including denied, failed, and dry-run actions.
- Use `HUGGINGOS_STATE_DIR` for runtime state tests and
  `HUGGINGOS_WORKSPACE_DIR` for low-risk write tests.
- Keep read-only file capabilities away from obvious secret paths. Secret access
  needs a separate higher-risk capability and stronger approval path.
- Use exclusive create semantics for low-risk file writes so capabilities do not
  overwrite or race an existing file.
- Keep high-risk app, browser, shell, network, screen, and secret actions
  deferred until their policy and permission model exists.

## Useful Commands

```bash
python3 product/cli/huggingos.py capabilities --json
python3 product/cli/huggingos.py run product.status --json
python3 product/cli/huggingos.py run fs.list --param path=. --json
python3 product/cli/huggingos.py run notes.create --param title=Scratch --dry-run --json
python3 product/cli/huggingos.py run audit.list --param limit=10 --json
python3 -m unittest discover -s product/tests -p "test_*.py"
```

## Evidence

- Product tests cover action serialization, registry duplicate rejection,
  parameter validation, allow/deny/confirmation/dry-run policy paths, audit
  writes, CLI capability listing, safe note creation, and failed actions.
- Product smoke workflow runs the same test suite on Ubuntu in CI.
