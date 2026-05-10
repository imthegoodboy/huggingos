# Product Services

This directory is reserved for Linux userspace daemons and local APIs.

Phase 1 does not start a resident service yet. The first service should be
added only when there is a real capability API or runtime bridge to expose.

Service rules:

- Prefer a user service before requiring system-level privileges.
- Use explicit local API schemas; do not expose arbitrary shell execution.
- Load secrets only through the approved config/secret path.
- Audit OS-changing actions before reporting success.
- Keep CLI commands usable when the service is not installed, with clear errors.
