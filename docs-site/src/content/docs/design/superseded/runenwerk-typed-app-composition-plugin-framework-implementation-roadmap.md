---
title: Superseded Runenwerk Typed App Composition Plugin Framework Roadmap
description: Historical proof-based rollout proposal for the superseded typed App/plugin meta-framework direction; not current sequencing authority.
status: superseded
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-09-13
superseded_by:
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../workspace/planning/roadmap.md
related_docs:
  - ./runenwerk-typed-app-composition-plugin-framework-design.md
---

# Superseded Runenwerk Typed App Composition Plugin Framework Roadmap

## Disposition

This roadmap is superseded historical planning. It no longer owns Runenwerk
application-composition sequencing, UI Component Platform sequencing, or shared
framework extraction.

Current authority is split deliberately:

- ADR 0019 and the canonical platform architecture own durable application
  composition law;
- GitHub issues own accepted/live work activation;
- the maintained repository roadmap owns durable sequence and dependencies;
- domain/framework owners own their semantic implementation plans.

## Historical sequence preserved

The proposal used the following proof-first decomposition:

```text
A  accept the typed composition direction
B  prove UI control contributions locally
C  prove host compatibility
D  prove a small App recipe
E  prove a non-UI domain
F  prove reports/snapshots
G  bridge frozen assembly to runtime startup
H  extract only repeated domain-neutral structure
```

The intent was to avoid premature `foundation/meta` extraction and to require at
least two structurally different proving domains before shared machinery. That
pressure remains useful historical rationale.

The concrete phase program, however, was never accepted as current Runenwerk
execution authority. In particular, the document's former 011-024 UI Component
Platform sequence mixed already-landed local contracts with never-activated
future targets such as SpatialCanvas, NodeCanvas, PortGraphCanvas,
ProgressionTreeView, TrackSurface/Timeline, and transitions/effects. Current
lifecycle authority now classifies those topics separately; this historical
roadmap must not reactivate them.

## Historical candidate outputs

The proposal discussed candidate outputs including:

```text
BaseControlsPlugin
UiControls extension point
HostCompatibilityMatrix
AppRecipe
ProductProfile
PluginGraphReport
AppAssemblyReport
FrozenAppAssembly
PluginManifest / PluginDependency
```

These names and abstractions are preserved only as historical alternatives. They
are not current public API targets and do not override ADR 0019's narrower law:
product convenience must lower into the one `App` runtime and ordinary owner
plugins/resources/configuration.

## Current continuation rule

Do not continue this phase sequence.

A future application-composition implementation slice must begin from a current
GitHub issue and current source. It must name a concrete product usability gap,
keep `App` as the single runtime composition root, avoid persistent mirrored
composition truth, and prove the minimum required group/helper behavior under
ADR 0019.

Shared extraction still requires independent evidence and its own accepted
boundary; this historical roadmap is not that authorization.
