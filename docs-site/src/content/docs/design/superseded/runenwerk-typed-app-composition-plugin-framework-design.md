---
title: Superseded Runenwerk Typed App Composition Plugin Framework Design
description: Historical pre-ADR-0019 proposal for typed app recipes, plugin suites, extension points, host profiles, and contribution registries; superseded by accepted App composition authority.
status: superseded
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-09-13
superseded_by:
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../architecture/runenwerk-platform-architecture.md
related_docs:
  - ./runenwerk-typed-app-composition-plugin-framework-implementation-roadmap.md
  - ../implemented/ui-program-architecture.md
  - ../implemented/ui-component-platform-base-control-packages-design.md
---

# Superseded Runenwerk Typed App Composition Plugin Framework Design

## Disposition

This is a historical pre-ADR-0019 architecture proposal. It is superseded and is
not current application-composition authority.

Current durable authority is:

- [ADR 0019: Batteries-Included Application Composition](../../adr/accepted/0019-batteries-included-application-composition.md);
- [Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md);
- current `App` source and tests for implemented behavior.

ADR 0019 establishes one live runtime composition root, `App`, and permits
transparent product/plugin groups only as inspectable composition recipes over
existing owner plugins and configuration. It does not accept a second app
runtime, persistent meta-application authority, universal service locator, or
universal mutable registry.

## Historical proposal preserved

The superseded proposal explored a more explicit meta-composition vocabulary:

```text
AppRecipe
ProductProfile
Plugin / PluginSuite
ExtensionPoint
Contribution
Registry / Descriptor / Catalog
HostProfile / HostAdapter
Capability
PluginGraphReport
AppAssemblyReport
HostCompatibilityMatrix
```

Its intended structural idea was that domains keep semantic ownership while a
shared composition layer coordinates plugin ordering, contribution routing,
host compatibility, freeze points, and reports. It also proposed proving the
shape in UI and a second non-UI domain before extracting shared primitives.

Those ideas are retained as historical alternatives and design pressure only.
The concrete names, extension-point model, registry machinery, plugin-graph
model, freeze model, report set, and public API sketches were **not accepted** as
the Runenwerk composition architecture.

## What replaced it

ADR 0019 resolves the durable product-composition question more narrowly:

```text
one App runtime root
+ ordinary owner plugins/resources/configuration
+ optional transparent product/plugin groups
+ progressive disclosure to direct owner APIs
```

A future group/helper may reduce routine integration ceremony, but it must lower
to the existing `App` composition model and disappear as an independent source
of truth. Exact Rust type names, memberships, customization APIs, and future
implementation remain separately gated.

## Surviving constraints

The following pressures remain useful because they agree with current accepted
authority rather than competing with it:

- domain meaning stays with the owning domain/framework;
- Runenwerk owns genuinely product-generic integration wiring;
- convenience must remain inspectable and typed;
- invalid composition must fail explicitly;
- ECS remains runtime fabric rather than static semantic authority;
- renderer output remains derived execution/product data;
- shared extraction requires concrete repeated neutral proof and a separate
  accepted decision.

These are not re-accepted by this historical file; use ADR 0019, the platform
architecture, and the owning domain authorities for current law.

## Non-authority

Do not use this record to authorize:

- `AppRecipe`, `PluginSuite`, or `ExtensionPoint` implementation;
- a generic plugin/contribution registry framework;
- `foundation/meta` extraction;
- a second runtime beside `App`;
- UI Component Platform phase sequencing;
- host/profile/catalog APIs;
- any current work item.

GitHub issues own activation. If a future concrete product need justifies new
composition machinery, derive the smallest required mechanism from current
`App` source and ADR 0019 under a new owning issue rather than reactivating this
proposal wholesale.
