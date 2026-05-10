# Product Services

This directory is reserved for Linux userspace daemons and local APIs.

Phase 2 keeps the capability engine in-process for speed and testability. The
first resident service should be added when the runtime bridge or desktop
integration needs long-running state, IPC, subscriptions, or background
automation.

Service rules:

- Prefer a user service before requiring system-level privileges.
- Use explicit local API schemas; do not expose arbitrary shell execution.
- Load secrets only through the approved config/secret path.
- Audit OS-changing actions before reporting success.
- Keep CLI commands usable when the service is not installed, with clear errors.
