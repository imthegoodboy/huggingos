# Agent Notes Index

This folder is the durable memory shelf for future AI agents working on
huggingOS.

Use it for important findings that are stable enough to help the next builder:

- Kernel gotchas.
- Build or QEMU traps.
- Hardware assumptions.
- Roadmap decisions.
- Dangerous areas that need care.
- Patterns that should be reused.

Do not use it for noisy logs, temporary thoughts, or task summaries that belong
in PR descriptions.

## Notes

- [Phase 0 baseline lessons](phase-0-baseline-lessons.md)
- [Linux product strategy](linux-product-strategy.md)
- [Git metadata corruption](git-metadata-corruption.md)
- [Prebuild readiness audit](prebuild-readiness-audit.md)
- [Product Phase 1 foundation](product-phase1-foundation.md)
- [Pre-Phase 2 architecture audit](pre-phase2-architecture-audit.md)

## Knowledge Capture Rule

When an agent discovers something future agents should know:

1. Create a short note from [TEMPLATE.md](TEMPLATE.md), or update an existing
   note if it is clearly the same topic.
2. Add the note to this index.
3. Link source files, issues, or PRs when possible.
4. Keep the note factual and actionable.
5. Include validation or evidence.

Good note: "RAMFS write must allocate new buffer before freeing old data, or
allocation failure can destroy existing file contents."

Bad note: "Worked on RAMFS today."
