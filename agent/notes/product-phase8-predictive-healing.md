# Product Phase 8 Predictive And Self-Healing Notes

Date: 2026-05-10

Phase 8 is suggestion-first. Do not add silent background fixes, destructive
cleanup, service restarts, or app launches without a separate confirmed
capability and rollback design.

Implemented surfaces:

- `proactive.workflow.detect` reads audit history and suggests repeated-workflow
  automations.
- `proactive.suggest` aggregates workflow and failure-review suggestions.
- `selfheal.diagnose` diagnoses simulated or reported failures and returns safe
  recovery steps.
- `timeline.explain` summarizes recent audited activity with memory and trace
  context.

Future agents should keep predictive features grounded in real local signals:
audit events, explicit memory, traces, desktop status, and screen/context
metadata. Avoid fake telemetry.
