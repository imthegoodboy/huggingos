# Product Policy

Product Phase 1 does not execute privileged OS actions yet. It establishes the
policy boundary that later capability APIs must use.

Current rules:

- No action may require root unless the command documents why.
- Destructive actions must require confirmation once they exist.
- AI provider calls must wait for config, secret loading, audit, and policy.
- Browser, shell, network, and system actions must go through capability APIs in
  later phases.

The current CLI can report product status, inspect non-secret config, and run
environment checks. It does not automate apps or change OS state yet.
