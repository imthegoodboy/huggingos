# Product Phase 3 AI Runtime

Date: 2026-05-10

Area: ai | security

Related:

- Issue: [#39](https://github.com/imthegoodboy/huggingos/issues/39)
- PR: Phase 3 implementation PR
- Files: `product/agent/src/main.rs`, `product/PHASE3.md`,
  `product/config/defaults.toml`

## Finding

The production AI path now starts in the Rust agent. The executable provider is
`local.rules`, which converts supported natural-language prompts into typed
capability calls. `ai run` does not bypass the policy, audit, executor, or
verifier path.

## Why It Matters

Future desktop, browser, app-control, and cloud-model features should not create
parallel automation paths. If they bypass capabilities, the OS loses permission
checks, audit records, and observable verification.

## Rule For Future Agents

Add new AI behaviors as providers or planner rules that emit capability plans.
Do not hardcode API keys, fake cloud responses, or mutate the OS directly from a
model/provider adapter.

## Evidence / Validation

Validated with Rust unit tests, `ai status`, `ai plan`, `ai run`, and redacted
`secrets status` smoke commands.
