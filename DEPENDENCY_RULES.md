# Dependency Rules

## Target repository direction

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk adapters/integration --> applications
RunenRender ----+
RunenUI --------+   (separate external workstream)
```

Framework repositories must not depend on Runenwerk.

The default rule is that framework repositories do not depend directly on one
another. A direct dependency requires a separate ADR proving independent value,
correct ownership, and absence of a Runenwerk adapter responsibility.

In particular:

- RunenRender does not require RunenECS, RunenSDF, or RunenUI;
- RunenECS does not require Runenwerk geometry, renderer, networking, or app
  lifecycle;
- RunenSDF does not require Runenwerk geometry, world, renderer, or ECS;
- Runenwerk owns cross-domain translations.

## Current in-repository direction

Until each clean cutover completes:

```text
foundation -> domain crates -> engine/runtime -> apps/adapters/tools
```

Current source location is transitional implementation fact, not permanent
ownership authority.

## Foundation rules

Foundation may depend only on justified foundation crates and appropriate
low-level external libraries.

Foundation must not depend on domain, runtime, editor, app, adapter, AI, UI, or
concrete backend code.

Do not create a universal shared-core/meta repository to avoid explicit adapter
boundaries.

## Domain rules

Domain crates may depend on foundation and carefully selected lower-level domain
contracts.

Domain crates must not depend on runtime, app code, backend adapters, editor app
wiring, AI integrations, or concrete rendering/windowing/input/audio backends
unless the domain explicitly owns that backend.

During extraction, accidental domain dependencies must be removed rather than
copied into the new repository.

## Framework rules

An extracted framework:

- validates independently;
- uses repository-local identities and diagnostics;
- exposes only owned semantics;
- has at least one public downstream conformance consumer;
- does not import Runenwerk source or types;
- does not require a source mirror or compatibility crate.

Before stable publication, Runenwerk pins an exact Git revision or exact
pre-release version. Moving branch dependencies are forbidden.

## Engine/runtime rules

Runtime may depend on foundation, current domains, extracted frameworks, and
backend/runtime implementation dependencies.

Runtime owns lifecycle and integration. It must not move product/editor/domain
semantics into generic framework APIs.

## Adapter rules

A Runenwerk adapter may depend on both Runenwerk and one external framework.

Adapters translate:

- identities;
- inputs and outputs;
- lifecycle facts;
- diagnostics and provenance;
- resource ownership.

Adapters must not duplicate framework algorithms or become writable parallel
authorities.

## App/tool rules

Apps and tools may compose higher layers but must not define core framework or
domain invariants.

## Test-support rules

Reusable fixtures should live in explicit test-support crates/modules. Production
APIs must not be widened solely for tests.

Cross-repository conformance tests belong in the framework repository for public
consumer proof and in Runenwerk for integration compatibility.

## Clean-cutover rules

Completed extraction leaves:

- one external source authority;
- one-way dependency direction;
- exact dependency pinning;
- no original source copy;
- no forwarding package or namespace;
- no submodule;
- no long-lived migration facade.

Temporary duplication is allowed only on an unmerged extraction branch.

## Boundary escalation

When one owner wants another owner's internals, first determine whether the
missing boundary is:

- a public value/DTO;
- command or request;
- diagnostic/report;
- adapter;
- capability contract;
- test-support contract.

Do not solve boundary pressure by creating a universal abstraction or by exposing
mutable internals.
