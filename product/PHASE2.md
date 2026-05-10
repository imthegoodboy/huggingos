# Product Phase 2: Capability API And Local Automation

Phase 2 creates the first real control plane for huggingOS. Agents and users do
not mutate the OS by calling random shell strings. They request typed
capabilities, then policy decides, executors act, verifiers check the result, and
audit records what happened.

## Delivered Scope

- Serializable action request/result structures.
- Risk levels: `read`, `low`, `medium`, and `high`.
- Action statuses: `succeeded`, `failed`, `denied`,
  `confirmation_required`, and `dry_run`.
- Local capability registry with duplicate-name protection.
- Policy engine for allow, deny, confirmation-needed, and dry-run decisions.
- Append-only JSON Lines audit log for successful, failed, denied, and dry-run
  actions.
- CLI listing and execution through the control plane.
- First safe local capabilities:
  - `product.status`
  - `fs.list`
  - `fs.read_text`
  - `notes.create`
  - `audit.list`

## Local Commands

Run from the repository root on Linux or WSL:

```bash
python3 product/cli/huggingos.py capabilities
python3 product/cli/huggingos.py run product.status --json
python3 product/cli/huggingos.py run fs.list --param path=. --json
python3 product/cli/huggingos.py run fs.read_text --param path=product/README.md --json
python3 product/cli/huggingos.py run notes.create --param title=PhaseTwo --param content="hello" --json
python3 product/cli/huggingos.py run audit.list --param limit=10 --json
```

Use a temporary state/workspace path when testing write behavior:

```bash
HUGGINGOS_STATE_DIR=/tmp/huggingos-state \
HUGGINGOS_WORKSPACE_DIR=/tmp/huggingos-workspace \
python3 product/cli/huggingos.py run notes.create --param title=Scratch --dry-run --json
```

## Safety Rules

- Read-only capabilities can observe local state and must still be audited.
- Low-risk write capabilities must be constrained to a safe workspace.
- Medium-risk capabilities require confirmation.
- High-risk capabilities are denied by default until a stronger approval path
  exists.
- Dry runs must not mutate state.
- Every denied, failed, dry-run, and successful action must write an audit
  record.

## Runtime State

The default state root follows XDG conventions:

```text
~/.local/state/huggingos/
```

Override it with:

```bash
HUGGINGOS_STATE_DIR=/path/to/state
```

Low-risk workspace writes default to:

```text
<state-dir>/workspace/
```

Override them with:

```bash
HUGGINGOS_WORKSPACE_DIR=/path/to/workspace
```

## Validation

```bash
python3 product/cli/huggingos.py doctor --json
python3 product/cli/huggingos.py capabilities --json
python3 -m unittest discover -s product/tests -p "test_*.py"
```

or:

```bash
make product-doctor
make product-capabilities
make product-smoke
```

## Still Intentionally Deferred

- AI provider calls and API keys.
- Arbitrary shell execution.
- Browser control.
- Desktop app/window control.
- Screen capture, OCR, audio capture, and memory indexing.

Those features belong in later phases, after secrets, desktop APIs, and stronger
permission prompts are implemented.
