---
title: Typed App Program Cross-Cutting Design Review
description: Critical cross-cutting review of Typed App Program pressure from security, permissions, persistence, migration, observability, accessibility, localization, tooling, packaging, failure recovery, and compatibility.
status: active
owner: ui
layer: reports
canonical: false
last_reviewed: 2026-07-04
related_docs:
  - ./typed-app-program-current-state-investigation.md
  - ./typed-app-program-engine-pressure-and-design-review.md
  - ./typed-app-program-multiplayer-concurrency-design-review.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ../../workspace/complete-design-gate.md
---

# Typed App Program Cross-Cutting Design Review

## Purpose

Review additional cross-cutting pressure not fully covered by the current-state investigation, the main design, the engine/runtime pressure review, or the multiplayer/concurrency review.

This review focuses on:

```text
security and permissions
privacy and sandboxing
persistence and migration
observability and diagnostics
accessibility and localization
tooling and AI review
packaging and hot reload
failure recovery and compatibility
versioning and schema evolution
time, randomness, and determinism
resource budgets and report size
```

This document is design-gate hardening only. It does not authorize implementation.

## Review Verdict

The Typed App Program design is now architecturally sound in its main spine, engine pressure, and multiplayer/concurrency pressure. However, a complete long-term design also needs explicit cross-cutting rules so the first proof does not accidentally create dead-end assumptions.

The first proof can remain bounded to a headless counter app, but the durable structure must not assume:

```text
no permissions
no privacy boundaries
no persistence
no migration
no localization
no accessibility
no diagnostic taxonomy
no compatibility policy
no hot reload
no failure recovery
no report size limits
no time/randomness control
```

## Cross-Cutting Classification

| Concern | Consider In Typed App Program? | Implement In First Proof? | Owner Boundary |
| --- | --- | --- | --- |
| Security / permissions | Yes | Only local capability denial proof | Host/security owner defines policy; app program records required capabilities and denial diagnostics. |
| Privacy / data exposure | Yes | No sensitive data in first proof | Host/product owner controls data access; app program reports must avoid leaking private payloads by default. |
| Sandboxing | Yes | No sandbox runtime | Host/platform owner enforces sandbox; app program can require explicit effect proposals. |
| Persistence | Yes | Deterministic snapshots only | App/domain owns persistence; app program requires model/action/effect versions and migration diagnostics. |
| Migration | Yes | Schema/version rejection only | Domain/app owner provides migrations; app program records compatibility and rejection. |
| Observability | Yes | Required as reports | App program owns trace/report shape, not telemetry transport. |
| Telemetry | Yes as future pressure | No | Product/platform owner owns telemetry emission and privacy policy. |
| Accessibility | Yes for UI proof pressure | Use existing UI proof where available | UI owns accessibility facts; app program preserves source/action context. |
| Localization / i18n | Yes | No localized UI required | Product/UI/content owners own localization; app program must not bake strings into durable action IDs. |
| Tooling / AI review | Yes | Reports must be readable | Docs/proof owners define report readability; no whole-engine execution required. |
| Packaging / distribution | Yes as future pressure | No | Product/app recipe owners own packaging; app program stays inspectable artifact. |
| Hot reload | Yes | No runtime reload | Host/app owner owns reload; app program needs versioned replacement and compatibility rules later. |
| Failure recovery | Yes | Route/reducer/projection failure proof | Host/app owner owns recovery UX; app program records failure states and diagnostics. |
| Undo/redo | Yes | Reducer trace only | Domain/app owner owns history policy; trace must support future history adapters. |
| Time/randomness | Yes | Must be explicit or absent | Host/test owner supplies deterministic time/random seeds; reducer must not query globals. |
| Report size/budget | Yes | Simple report only | Proof/tooling owners define budgets; app program should expose size diagnostics later. |

## Security And Permission Requirements

Typed App Program must treat authorization as explicit data and diagnostics, not hidden behavior.

Required future structural pressure:

```text
AppActionCapability
HostCapabilityFacts
HostPolicyDecision
ActionAuthorizationReport
EffectAuthorizationReport
DeniedActionDiagnostic
DeniedEffectDiagnostic
```

Rules:

```text
Reducers must not bypass host policy.
Host policy must be separate from reducer logic.
Denied actions/effects must fail closed.
Effects must be inert proposals until a host accepts them.
Reports must show whether an action was accepted, denied, unsupported, or malformed.
```

First proof requirement:

```text
include at least one missing-capability or unauthorized-route rejection case
```

## Privacy And Report Redaction

Replay reports are useful for AI review and debugging, but they can leak payloads if later actions contain asset paths, user identifiers, tokens, private project data, chat messages, or network payloads.

Future pressure:

```text
redaction policy
safe debug summary
full diagnostic payload gated by host policy
private payload marker
stable resource ref instead of raw file path
```

First proof can avoid sensitive data, but the report shape must not require dumping raw payloads forever.

Stop condition:

```text
implementation prints arbitrary action payloads into durable reports without a redaction policy
```

## Persistence, Save/Load, And Migration

Typed App Program must be versioned from the start.

Required pressure:

```text
AppProgramVersion
AppModelVersion
AppActionVersion
AppEffectVersion
RouteActionMapVersion
ReplayTraceVersion
schema references
migration diagnostics
compatibility checks
```

Rules:

```text
Unknown model/action/effect versions fail closed.
Replay traces record the version set used.
Migrations are domain/app-owned, not generic app-program magic.
Persistence is not implemented in the first proof.
```

First proof requirement:

```text
model/action IDs and versions must exist even if only one version is supported
```

## Observability, Diagnostics, And Error Taxonomy

A durable app-program architecture needs distinct diagnostic namespaces.

Required namespaces:

```text
app.program.id
app.model.schema
app.action.schema
app.route_action.resolve
app.reducer
app.effect.plan
app.effect.authorize
app.projection
app.replay
app.host.compatibility
app.version.compatibility
app.report.budget
```

Rules:

```text
Do not collapse all failures into one generic error.
Diagnostics must include severity, code, summary, and source context where possible.
Reports must be readable without executing the full engine.
```

First proof requirement:

```text
unknown route, bad payload, missing capability, reducer diagnostic, and projection diagnostic must be distinguishable
```

## Accessibility And Localization

For UI proofs, app actions and projections must preserve enough context for accessibility reports while keeping localization separate from durable action identity.

Rules:

```text
Action IDs are stable machine IDs, not localized strings.
Visible labels can be localized later through UI/content owners.
Accessibility facts belong to UI proof outputs.
App replay reports should reference stable IDs and optional display summaries.
```

First proof requirement:

```text
Counter proof should not make visible English labels the durable identity of routes or actions.
```

## Tooling And AI Review

The reports must be useful for review in contexts where the full repo or engine cannot be run.

Required report qualities:

```text
small enough to inspect
stable ordering
deterministic output
clear pass/fail summary
separate diagnostics by phase
source IDs and route/action IDs included
no raw private payloads by default
```

First proof requirement:

```text
single replay report should show model -> projection -> event -> route action -> reducer -> effect -> model revision
```

## Packaging, Hot Reload, And Compatibility

Typed App Program should not implement packaging or hot reload now, but must not block them.

Future pressure:

```text
artifact identity
program version
route/action compatibility
state migration
safe replacement boundary
host compatibility check before activation
reload diagnostics
```

Rules:

```text
Hot reload cannot replace active semantics without compatibility diagnostics.
App programs must be inspectable before activation.
Host activation is separate from app-program construction.
```

First proof does not implement hot reload.

## Failure Recovery

Failures must be modeled explicitly.

Required failure states:

```text
MalformedProgram
IncompatibleHost
RouteResolutionFailed
ActionRejected
ReducerFailed
EffectPlanningFailed
ProjectionFailed
ReplayDiverged
ReportBudgetExceeded
```

Rules:

```text
A failed step must not silently mutate model state.
A rejected action must preserve previous model revision.
An effect planning failure must not execute partial host work.
A replay divergence must report the first divergent step.
```

First proof requirement:

```text
negative cases must prove no accidental model mutation after rejection
```

## Time, Randomness, And Determinism

Reducers must be deterministic.

Rules:

```text
Reducers must not call wall-clock time.
Reducers must not use unseeded randomness.
Time/random/random-like host observations must enter as explicit actions or host facts.
Replay trace must record deterministic inputs.
```

First proof requirement:

```text
counter reducer uses no time, randomness, IO, global state, or scheduler order
```

## Resource And Report Budgets

Reports can become large in real apps.

Future pressure:

```text
max replay steps
max payload summary size
redacted payload count
report truncation diagnostics
snapshot size diagnostics
effect count diagnostics
```

First proof may use small reports, but the design must not assume unbounded traces forever.

## Additional Stop Conditions

Stop if implementation tries to:

```text
make permission checks implicit
mutate model state after rejected action
store raw secrets or private payloads in reports
use localized visible text as durable action identity
query wall-clock time or random state in reducer
use file paths or runtime handles as durable resource identity
collapse all diagnostics into a generic error
activate hot-reloaded programs without compatibility checks
perform telemetry emission inside app-program proof
make app program responsible for sandbox enforcement
make persistence generic magic instead of domain-owned migration
allow report size to grow without budget diagnostics in product-grade use
```

## Impact On First Proof

The Headless Counter App Proof remains the correct first proof, but implementation planning must add these requirements:

```text
model/action IDs are versioned
reports distinguish all failure phases
negative cases prove no mutation after rejection
visible labels are not route/action identity
reducer uses no time/random/global state
missing capability is tested
payload summary is safe and bounded
report ordering is deterministic
```

## Final Verdict

The remaining important cross-cutting concerns are not reasons to expand the first proof into a large implementation.

They are reasons to make the first proof structurally correct:

```text
local-first
versioned
permission-aware
deterministic
reportable
redaction-ready
migration-ready
accessibility-compatible
localization-safe
failure-explicit
budget-aware
```

No code should start until implementation planning consumes this review together with the engine and multiplayer/concurrency reviews.
