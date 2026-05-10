# Product Architecture

For the complete project-wide architecture, see
[`../ARCHITECTURE.md`](../ARCHITECTURE.md).

huggingOS is a Linux-based AI operating system layer. Linux provides the kernel,
drivers, process isolation, networking, filesystems, and desktop APIs. The
huggingOS product code provides the control plane that lets users and agents act
on that OS safely.

## Current Layers

```text
Linux kernel and distro base
  -> huggingOS product CLI
  -> in-process capability control plane
  -> non-secret runtime config
  -> policy and service boundaries
  -> product tests and CI
```

## Phase 2 Control Plane

Product Phase 2 adds the first executable capability control plane:

```text
Intent
  -> capability registry
  -> policy decision
  -> executor
  -> verifier
  -> audit log
  -> result
```

Current implementation lives in `product/huggingos_core/` and is called by
`product/cli/huggingos.py run ...`.

Tracking:

- Epic: [#31 Product Phase 2 capability API and local automation](https://github.com/imthegoodboy/huggingos/issues/31)
- Schema: [#23](https://github.com/imthegoodboy/huggingos/issues/23)
- Registry: [#24](https://github.com/imthegoodboy/huggingos/issues/24)
- Policy: [#26](https://github.com/imthegoodboy/huggingos/issues/26)
- Audit: [#27](https://github.com/imthegoodboy/huggingos/issues/27)
- First capabilities: [#30](https://github.com/imthegoodboy/huggingos/issues/30)
- Tests: [#28](https://github.com/imthegoodboy/huggingos/issues/28)

Agents should never control the OS by directly reaching into random modules or
running arbitrary shell strings. They should call typed capabilities. That still
allows broad control over the computer over time, but each action is visible,
permissioned, testable, and reversible where possible.

## Design Rules

- Keep the first capability engine in-process for speed and easy testing.
- Keep schemas serializable so a local service can use the same contracts later.
- Prefer read-only and low-risk capabilities before destructive actions.
- Require explicit policy for network, shell, browser, settings, and secret
  actions.
- Write audit records for every executed or denied action.
- Keep CLI commands usable without root.
- Do not add AI provider calls until capabilities, policy, audit, and secret
  loading exist.

## Readiness Verdict

Phase 1 and Phase 2 are directionally correct:

- It uses Linux as the real OS foundation.
- It does not copy or fork the Linux kernel before there is a need.
- It creates a runnable hosted product slice that works in WSL and CI.
- It keeps secrets and fake AI out of the product.
- It routes real automated actions through registry, policy, verifier, and
  audit before later AI or desktop control exists.

Executable scripts must keep LF line endings. `.gitattributes` enforces that for
scripts, source, Makefiles, TOML, and workflows.
