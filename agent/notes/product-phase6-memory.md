# Product Phase 6 Memory

Date: 2026-05-10

## Finding

Phase 6 memory is local and user-controlled. It stores session facts,
preferences, audit-derived events, and opt-in semantic file indexes under the
configured state directory.

## Guidance

- Do not call the Phase 6 file index "embeddings"; it is
  `local.token_overlap.v1`.
- `files.semantic.index` requires confirmation and must stay opt-in by root.
- Hidden files and sensitive names such as `.env.*`, credentials, tokens, and
  private keys must be skipped or denied.
- Secret-like memory keys such as API keys, tokens, passwords, and credentials
  are denied so memory does not become secret storage.
- Memory inspection, export, and deletion should remain capabilities.

## Evidence

- Implementation: `product/agent/src/main.rs`
- Phase doc: `product/PHASE6.md`
- ADR: `docs/adr/0008-local-memory-semantic-files.md`
- Tracking: https://github.com/imthegoodboy/huggingos/issues/72
