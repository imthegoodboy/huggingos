# Product Phase 8: Predictive And Self-Healing OS

Phase 8 moves huggingOS from purely reactive commands toward safe proactive
help. It does not run background fixes or destructive automation. It detects,
explains, and recommends through the same capability, policy, audit, and agent
contracts used by earlier phases.

## What Works

- `proactive.workflow.detect` inspects recent audited capability events and
  suggests user-approved automations for repeated workflows.
- `proactive.suggest` combines workflow detection with recent failure review
  into suggestion-only proactive recommendations.
- `selfheal.diagnose` diagnoses simulated or user-reported recoverable
  failures, such as app crashes, service failures, memory pressure, and slow
  operations.
- `timeline.explain` summarizes recent audited activity with memory and agent
  trace context.
- The local AI planner maps natural-language prompts like `detect repeated
  workflow`, `app crashed, self heal it`, and `explain what happened` into typed
  Phase 8 capabilities.
- The agent catalog includes predictive and self-healing agents with explicit
  capability allowlists.

## Safety Rules

- Phase 8 capabilities are read-only and suggestion-first.
- Recovery recommendations are returned as future capability steps, not
  executed automatically.
- App launch, browser, deletion, screenshot, and orchestration capabilities keep
  their existing confirmation requirements.
- Workflow detection reads only the local audit log.
- Failure diagnosis can be simulated for tests and demos without touching real
  system services.

## Commands

From `product/agent/`:

```bash
cargo run -- run proactive.workflow.detect --json
cargo run -- run proactive.suggest --json
cargo run -- run selfheal.diagnose --param symptom=app_crashed --param target=editor --param simulated=true --json
cargo run -- run timeline.explain --json
cargo run -- ai plan "detect repeated workflow" --json
cargo run -- ai plan "app crashed, self heal it" --json
cargo run -- ai plan "explain what happened" --json
```

From the repository root:

```bash
make product-agent-workflow-detect
make product-agent-proactive-suggest
make product-agent-selfheal-diagnose
make product-agent-timeline-explain
```

## Still Later

- Long-running proactive daemon.
- Real system service restart integration.
- Resource telemetry from a host service.
- Rollback controls for safe auto-fixes.
- UI approval flow for suggested automations.
