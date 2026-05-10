# Product Policy

Product Phase 2 executes only typed local capabilities through the policy engine.
It still does not execute privileged OS actions or arbitrary shell commands.

Current rules:

- No action may require root unless the command documents why.
- Read-only actions are allowed when scoped and audited.
- Low-risk write actions must be constrained to a safe workspace.
- Medium-risk actions require confirmation.
- High-risk or destructive actions are denied by default until a stronger
  approval path exists.
- Dry-run actions must not mutate state.
- Every capability decision and result must be audited.
- AI provider calls must wait for config, secret loading, audit, and policy.
- Browser, shell, network, and system actions must go through capability APIs in
  later phases.

The current CLI can report product status, inspect non-secret config, list
capabilities, execute safe read-only capabilities, create notes inside the safe
workspace, and list audit records. It does not automate apps or call AI
providers yet.
