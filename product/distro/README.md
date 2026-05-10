# Product Distro Strategy

Product Phase 1 uses an Ubuntu LTS hosted prototype instead of building a full
image immediately. This keeps the first slice runnable in WSL and CI while the
CLI, config, and policy boundaries are still small.

Reference base:

- Ubuntu 24.04 LTS for local WSL and CI validation.
- Debian 13 compatibility should be preserved where practical.
- No generated images are committed to the repository.

Later product phases can add live image or rootfs build files here after the
userspace foundation is useful enough to package.

See `docs/adr/0002-linux-base-strategy.md` for the full decision.
