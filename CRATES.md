# Crate Inventory

This root file is the short crate-inventory entrypoint.

Detailed inventory:

```text
docs-site/src/content/docs/workspace/crate-inventory.md
```

Crate documentation status:

```text
docs-site/src/content/docs/workspace/crate-docs-status.md
```

Layer summary:

- `foundation`: low-level reusable primitives.
- `domain`: engine-agnostic contracts and logic.
- `engine`: runtime composition and plugin integration.
- `net`: transport, replication, session, and history contracts.
- `apps`: runnable applications.
- `adapters`: external host integrations.

When workspace members change, update the docs-site crate inventory first, then keep this root summary aligned.
