# Fixed Step Plugin Usage Guide

## Purpose

Installs fixed-step resources used by the shared runtime fixed-step executor.

## Entry Points

- Module: engine/src/plugins/fixed_step.rs
- Entry: FixedStepPlugin
- Local README: not present (file-based plugin module)

## Minimal Setup

```rust
use engine::plugins::fixed_step::FixedStepPlugin;

app.add_plugin(FixedStepPlugin);
```

## Runtime Contract

- Schedule placement: Resource-only (no systems)
- Ownership: Fixed-step resource installation contract.
- Non-ownership: Fixed-step loop execution logic.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../../src/plugins/README.md)
