# ADR 0001: Linux Kernel Product Strategy

Status: accepted

Date: 2026-05-10

## Context

huggingOS started as a 32-bit custom x86 hobby kernel. That path is useful for
learning operating-system internals, but it is not the fastest route to a
normal working AI-native OS.

The product vision needs real hardware drivers, networking, filesystems,
process isolation, graphical desktop integration, browser control, secret
storage, packaging, and a large userspace ecosystem. Rebuilding those from
scratch would delay the AI OS layer by a long time and would likely produce a
less reliable system.

## Decision

Use the Linux kernel for the main huggingOS product track.

Keep the current custom x86 kernel as a separate kernel-lab track for education,
experiments, and low-level prototypes. Product features such as AI command
center, app control, screen understanding, memory, agents, and automation should
be implemented as Linux userspace services, desktop integrations, packages, and
capability APIs.

## Alternatives Considered

- Continue building the whole OS from the custom kernel.
  - Best for learning, but too slow for a practical AI desktop.
- Fork or deeply patch Linux immediately.
  - Powerful later, but unnecessary before userspace product primitives exist.
- Build a Linux distribution/layer first.
  - Gives real OS behavior now and keeps the door open for kernel patches later.

## Consequences

Easier:

- Real networking, storage, permissions, GUI, browser, and app integration.
- Faster path to a usable AI OS.
- Better security primitives and update model.
- Easier CI and automated testing.

Harder:

- Need packaging and distro/build discipline.
- Need careful boundaries between host Linux, huggingOS services, and AI agents.
- Need to avoid becoming just another app by owning the OS-level capability and
  policy layer.

The custom kernel remains in the repo, but it is no longer the blocker for the
main product roadmap.

## Validation

Product Phase 1 should prove this decision by creating a reproducible Linux
product foundation with:

- A documented base image/rootfs strategy.
- A real `huggingos` CLI or service entrypoint.
- Runtime config that does not commit secrets.
- A smoke test that runs on a fresh checkout.
