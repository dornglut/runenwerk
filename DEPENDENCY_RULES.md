# Dependency Rules

## Global Direction

```text
foundation -> domain crates -> engine/runtime -> apps/adapters/tools
```

## Foundation Rules

Foundation may depend only on other justified foundation crates and appropriate low-level external libraries.

Foundation must not depend on domain crates, runtime crates, editor crates, app crates, adapters, AI integrations, UI frameworks, or concrete backends.

## Domain Rules

Domain crates may depend on foundation and carefully selected lower-level domain contract crates.

Domain crates must not depend on runtime, app code, backend adapters, editor application wiring, AI integrations, or concrete rendering/windowing/input/audio backends unless the domain explicitly owns that backend.

## Engine / Runtime Rules

Runtime may depend on foundation, domain crates, and backend/runtime implementation dependencies. Runtime must not introduce editor-specific concepts into generic runtime APIs.

## App / Adapter / Tool Rules

Apps and tools may compose higher layers but must not define core domain invariants.

## Test-Support Rules

Reusable fixtures should live in explicit test-support crates/modules. Production APIs must not be widened solely for tests.

## Boundary Escalation

When one crate wants internals from another, first ask whether a DTO, command, ratifier, contract crate, or test-support crate is missing.
