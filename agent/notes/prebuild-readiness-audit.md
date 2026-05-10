# Prebuild Readiness Audit

Date: 2026-05-10

Area: process, product, security

Related:

- Files: `.github/PULL_REQUEST_TEMPLATE.md`, `.github/ISSUE_TEMPLATE/phase_task.yml`
- Files: `.gitignore`, `product/PHASE1.md`, `PLAN.md`

## Finding

Before Product Phase 1, the roadmap was ready but some process files still
looked kernel-lab-first. The PR template, issue template, and task checklist
needed product-track validation fields so future product work does not inherit
kernel-only checks by accident.

## Why It Matters

Product work must prove Linux userspace behavior, protect local secrets, and
avoid fake AI features. Kernel-lab checks are still useful, but they are not the
right proof for product CLI, service, config, or CI work.

## Rule For Future Agents

- Choose the roadmap track before coding.
- Use product validation for `product/` work and kernel-lab validation for
  `kernel/` or ISO work.
- Keep local runtime files, `.env` files, secrets, tokens, and generated product
  artifacts out of Git.
- Start Product Phase 1 with #14, then #15, #17, #18, and #19.

## Evidence / Validation

Readiness checks run during the audit:

- `git diff --check`
- `rg -n "[^\x00-\x7F]" PLAN.md README.md UPDATE.md agent .github docs product`
- `python C:\\Users\\parth\\.codex\\skills\\.system\\skill-creator\\scripts\\quick_validate.py agent`
- `wsl make clean all iso`
