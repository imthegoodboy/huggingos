# ADR 0002: Linux Base Strategy

Status: accepted

Date: 2026-05-10

## Context

Product Phase 1 needs a Linux foundation that can run from a fresh checkout,
work in WSL and CI, and later become a bootable desktop image. The choices from
the roadmap are Debian/Ubuntu live image, Buildroot, and Yocto.

Current official release facts checked on 2026-05-10:

- Ubuntu LTS releases are recommended for production environments and receive
  five years of standard security maintenance:
  https://ubuntu.com/about/release-cycle
- Debian stable is Debian 13 "trixie", with a five-year support life cycle:
  https://www.debian.org/releases/index
- Buildroot is focused on building complete embedded Linux systems by
  cross-compilation:
  https://buildroot.org/downloads/manual/manual.html
- Yocto provides tools and methods for customizable Linux-based systems:
  https://www.yoctoproject.org/about/project-overview/

The current local development environment is Ubuntu 24.04 LTS under WSL2, so the
first product slice must work there without special machine setup.

## Decision

Use an Ubuntu LTS hosted prototype as the Product Phase 1 reference base.

Pin CI and development commands to the Debian/Ubuntu family, starting with
Ubuntu 24.04 LTS because it is already available in the local WSL environment
and broadly available in CI runners. Keep the product layout compatible with a
future Ubuntu/Debian live image or rootfs, but do not build a full image until
the CLI, config, smoke tests, and policy boundaries are stable.

Phase 1 will therefore produce:

- A Python standard-library-only `huggingos` CLI that runs on Linux.
- Debian/Ubuntu-oriented dev commands.
- A non-secret config layout.
- Product smoke tests and CI.
- Distro documentation that can later grow into live image/rootfs build files.

## Alternatives Considered

- Ubuntu or Debian live image immediately:
  - Good final direction, but too heavy before there is a product service or CLI
    to put inside the image.
- Debian stable first:
  - Strong stability and long support. Keep compatibility, but use Ubuntu LTS as
    the first reference because the repo already validates on Ubuntu WSL and
    GitHub Actions.
- Buildroot:
  - Good for small embedded systems, but the AI OS needs desktop integration,
    app automation, browsers, secret storage, and package availability.
- Yocto:
  - Strong for custom distributions, but it adds build-system complexity before
    the product boundary is proven.

## Consequences

Easier:

- Fast local iteration from a fresh checkout.
- CI can validate the product slice without generated images or root access.
- Future `.deb`, systemd, desktop integration, and live-image work has a
  familiar base.

Harder:

- A full bootable product image is intentionally deferred.
- Ubuntu-specific assumptions must be kept out of core code so Debian-family
  compatibility remains possible.
- Later image work still needs a separate ADR for live image tooling.

## Validation

Product Phase 1 validates this decision when:

- `wsl make product-smoke` or `make product-smoke` passes on Ubuntu.
- CI runs the same product smoke test on an Ubuntu runner.
- The CLI reports real host/product state without root permission.
- Runtime config exists without committed secrets.
