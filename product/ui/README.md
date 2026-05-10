# Product UI

This folder is reserved for the future huggingOS desktop overlay and control
center.

The first real UI contract is implemented in the Rust agent rather than as a
fake screen:

```bash
cd product/agent
cargo run -- run plugins.approval.surface --param source=../plugins/hello-assistant --json
```

That command returns `huggingos.plugin.approval.v1`, which a future renderer can
use for plugin trust, permissions, sandbox, update, rollback, and confirmed
next-action review.

Run `product.readiness.audit` after UI contract changes so the product gate
continues to prove docs, smoke targets, and approval surfaces are aligned.

Do not add static mock screens that claim to control the OS. UI work should
consume real capability payloads and keep lifecycle actions routed through the
policy, confirmation, verification, and audit path.
