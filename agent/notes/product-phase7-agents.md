# Product Phase 7 Agents

Date: 2026-05-10

## Finding

Phase 7 agents are permissioned manifests over the existing capability registry.
The orchestrator delegates only to capabilities an agent is allowed to call, and
execution still passes through policy, verification, audit, and traces.

## Guidance

- Do not give agents raw shell or filesystem access outside capabilities.
- Add new agent permissions by updating the catalog and tests together.
- `agents.orchestrate` is medium-risk and requires confirmation.
- Traces are local JSON Lines and should stay inspectable through
  `agents.trace.list`.

## Evidence

- Implementation: `product/agent/src/main.rs`
- Phase doc: `product/PHASE7.md`
- ADR: `docs/adr/0009-permissioned-agent-orchestration.md`
- Tracking: https://github.com/imthegoodboy/huggingos/issues/73
