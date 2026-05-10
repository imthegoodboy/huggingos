# Product Policy

Product Phase 2 executes only typed local capabilities through the policy engine.
It still does not execute privileged OS actions or arbitrary shell commands.

Current rules:

- No action may require root unless the command documents why.
- Read-only actions are allowed when scoped and audited.
- Read-only file capabilities must still deny obvious secret path names such as
  `.env`, `.ssh`, `credentials`, API keys, tokens, and private keys.
- Low-risk write actions must be constrained to a safe workspace.
- Medium-risk actions require confirmation.
- High-risk or destructive actions are denied by default until a stronger
  approval path exists.
- Dry-run actions must not mutate state.
- Every capability decision and result must be audited.
- Audit input summaries must recursively redact secret-like keys.
- If the audit path cannot be written, capabilities fail closed.
- AI provider calls must wait for config, secret loading, audit, and policy.
- Browser, screen, shell, network, and system actions must go through capability
  APIs.
- Screen/context observation must redact private active-window data and block
  confirmed capture when the active context is private or unknown.
- Memory writes and semantic indexing must stay opt-in, inspectable, and
  deletable.
- Agents must call only capabilities listed in their allowlist, and delegated
  calls still pass through policy, audit, and verification.
- Predictive and self-healing behavior must stay suggestion-first until
  confirmation, rollback, and daemon controls exist.
- Plugins must stay manifest-only and read-only until sandboxing, signatures,
  and rollback controls exist.
- Plugin signature fields are metadata-only until cryptographic verification is
  implemented.

The current CLI can report product status, inspect non-secret config, list
capabilities, execute safe read-only capabilities, create notes inside the safe
workspace, and list audit records. The Rust agent adds AI planning,
desktop/app/browser capabilities, and the first screen/context observation
capabilities, local memory/search, multi-agent orchestration, and
predictive/self-healing suggestions, plus manifest-based plugins. It does not
call cloud AI providers yet and does not execute plugin code.
