# Product Architecture

For the complete project-wide architecture, see
[`../ARCHITECTURE.md`](../ARCHITECTURE.md).

huggingOS is a Linux-based AI operating system layer. Linux provides the kernel,
drivers, process isolation, networking, filesystems, and desktop APIs. The
huggingOS product code provides the control plane that lets users and agents act
on that OS safely.

## Current Layers

```text
Linux kernel and distro base
  -> huggingOS product CLI
  -> Rust huggingOS agent runtime
  -> provider-neutral AI runtime bridge
  -> desktop/app/browser capability bridge
  -> screen/context observation bridge
  -> memory and semantic file bridge
  -> multi-agent orchestration bridge
  -> predictive and self-healing suggestion bridge
  -> in-process capability control plane
  -> non-secret runtime config
  -> policy and service boundaries
  -> product tests and CI
```

## Phase 2 Control Plane Through Phase 8 Predictive Help

Product Phase 2 adds the first executable capability control plane:

```text
Natural-language intent
  -> AI provider bridge
  -> deterministic local planner
  -> capability plan
  -> capability registry
  -> policy decision
  -> executor
  -> verifier
  -> audit log
  -> result
```

Current Python reference implementation lives in `product/huggingos_core/` and
is called by `product/cli/huggingos.py run ...`.

Product Phase 3 adds the first Rust AI bridge above that control plane. The
current executable provider is `local.rules`, which can plan supported prompts
offline and execute them only through policy, audit, and verification.

Product Phase 4 adds the first desktop bridge. The Rust agent can detect desktop
readiness, list `.desktop` apps, request confirmed app launches, request
confirmed browser URL opens, and preview workspace modes through the same
capability path.

Product Phase 5 adds the first screen/context bridge. The Rust agent can report
screen readiness, dry-run or confirm screenshot capture, snapshot active-window
context with privacy redaction, and OCR local images through discovered host
backends. Clipboard content, accessibility trees, browser tab context, and
portal/PipeWire capture remain later.

Product Phase 6 adds local memory and semantic file search. Session facts,
preferences, event history, opt-in token indexes, export/delete, and resume
planning all run through typed capabilities.

Product Phase 7 adds multi-agent orchestration. Built-in agents have explicit
capability allowlists, deterministic plans, confirmed execution, audit records,
and replayable traces.

Product Phase 8 adds predictive and self-healing suggestions. Repeated workflow
detection, proactive suggestions, failure diagnosis, and activity timelines read
local audit/memory/trace context and return recommended next steps without
silently executing fixes.

The production agent runtime lives in `product/agent/` as a Rust crate. Future
daemon and desktop integration work should move there first.

Tracking:

- Epic: [#31 Product Phase 2 capability API and local automation](https://github.com/imthegoodboy/huggingos/issues/31)
- Phase 3 epic: [#39 Product Phase 3 Epic: AI runtime bridge and secrets](https://github.com/imthegoodboy/huggingos/issues/39)
- Phase 4 epic: [#56 Product Phase 4 Epic: Desktop command center and app control](https://github.com/imthegoodboy/huggingos/issues/56)
- Phase 5 epic: [#65 Product Phase 5 Epic: Screen and context engine](https://github.com/imthegoodboy/huggingos/issues/65)
- Phase 6 epic: [#72 Product Phase 6 Epic: Memory and semantic files](https://github.com/imthegoodboy/huggingos/issues/72)
- Phase 7 epic: [#73 Product Phase 7 Epic: Multi-agent orchestration](https://github.com/imthegoodboy/huggingos/issues/73)
- Phase 8 epic: [#75 Product Phase 8 Epic: Predictive and self-healing OS](https://github.com/imthegoodboy/huggingos/issues/75)
- Schema: [#23](https://github.com/imthegoodboy/huggingos/issues/23)
- Registry: [#24](https://github.com/imthegoodboy/huggingos/issues/24)
- Policy: [#26](https://github.com/imthegoodboy/huggingos/issues/26)
- Audit: [#27](https://github.com/imthegoodboy/huggingos/issues/27)
- First capabilities: [#30](https://github.com/imthegoodboy/huggingos/issues/30)
- Tests: [#28](https://github.com/imthegoodboy/huggingos/issues/28)

Agents should never control the OS by directly reaching into random modules or
running arbitrary shell strings. They should call typed capabilities. That still
allows broad control over the computer over time, but each action is visible,
permissioned, testable, and reversible where possible.

## Design Rules

- Keep the first capability engine in-process for speed and easy testing.
- Keep schemas serializable so a local service can use the same contracts later.
- Prefer read-only and low-risk capabilities before destructive actions.
- Require explicit policy for network, shell, browser, settings, and secret
  actions.
- Write audit records for every executed or denied action.
- Keep CLI commands usable without root.
- Do not add cloud AI provider calls until provider clients, budgets, consent,
  retries, and secret storage are ready.
- Do not add silent screen scraping, global hotkeys, clipboard content reads, or
  window manipulation until a desktop service and per-action permission model
  exist.

## Readiness Verdict

Phase 1 through Phase 4 are directionally correct:

- It uses Linux as the real OS foundation.
- It does not copy or fork the Linux kernel before there is a need.
- It creates a runnable hosted product slice that works in WSL and CI.
- It keeps secrets and fake cloud AI out of the product.
- It routes real automated actions through registry, policy, verifier, and
  audit.
- It adds a real local AI planning bridge without allowing models to mutate the
  OS directly.
- It adds real desktop/app/browser contracts without fake UI automation.
- It adds real screen/context observation contracts without fake pixels, fake
  OCR, or silent private-window capture.
- It adds local memory and file search without fake embeddings or invisible
  collection.
- It adds multi-agent orchestration without letting agents bypass policy.
- It adds predictive and self-healing help without silent destructive action.

Executable scripts must keep LF line endings. `.gitattributes` enforces that for
scripts, source, Makefiles, TOML, and workflows.
