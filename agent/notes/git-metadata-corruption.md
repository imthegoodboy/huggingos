# Git Metadata Corruption

Date: 2026-05-10

Area: git, process

Related:

- Files: `.git/HEAD`, `.git/refs/heads/*`

## Finding

During Linux product planning, `git status` reported an invalid branch ref
because `.git/HEAD` and the current branch ref contained NUL bytes.

## Why It Matters

A broken ref can make Git commands fail even when the working tree files are
fine. Destructive recovery commands can lose user work.

## Rule For Future Agents

- Inspect `git status -sb` before committing or pushing.
- If Git reports an invalid ref, check `.git/HEAD` and the named ref before any
  reset or checkout.
- Prefer repairing the exact ref from known commit state over destructive
  commands.
- Never run `git reset --hard` or force checkout to fix metadata without explicit
  user approval.

## Evidence / Validation

The branch was restored by rewriting the corrupt ref to the intended branch and
commit, then confirming `git status -sb` showed
`codex/linux-kernel-product-path`.
