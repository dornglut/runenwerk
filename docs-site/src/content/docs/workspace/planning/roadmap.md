---
title: Roadmap
description: Durable high-level sequence and dependency direction for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../engineering-workflow.md
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../adr/accepted/0021-ratify-runenrender-semantic-rendering-architecture.md
  - ../../design/accepted/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
---

# Roadmap

This page owns **durable sequence and dependency direction only**.

GitHub issues and the Engineering Portfolio own live work, priority, activation, owners,
blockers, and current status. Pull requests own delivery, reviewed feature heads,
validation, and merge evidence. Accepted ADRs and designs own durable architecture.

The roadmap therefore does not record exact accepted revisions, active branches, current
PR/check state, completion tables, or which implementation issue is active today.

## Repository family direction

```text
RunenSDF --------+
RunenECS --------+--> Runenwerk adapters and applications
RunenUI ---------+
                       |
                       +--> RunenRender --> RunenGPU
                       +--> non-render RunenGPU consumers
```

Runenwerk remains the integration and product repository. Framework repositories do not
depend on Runenwerk.

Accepted GPU/render dependency:

```text
RunenRender -> RunenGPU
```

Target layering:

```text
Runenwerk host, source, and product policy
    -> RunenRender semantic rendering and renderer-native planning/admission
        -> standalone RunenGPU public GPU-execution contracts
            -> private backend implementation
```

Non-render consumers may use standalone RunenGPU directly without depending on
RunenRender.

## RunenSDF extraction direction

RunenSDF owns standalone field mathematics at its public boundary. Runenwerk retains
product and domain integration above that boundary.

Runenwerk now contains no `domain/sdf` package, source mirror, forwarding namespace,
submodule, source include, or unused external dependency.

## RunenGPU direction

RunenGPU's internal G-phase proof/extraction program is complete as a Runenwerk
predecessor program. Current reusable RunenGPU semantics, implementation, conformance,
release policy, and future framework evolution belong to `dornglut/runen-gpu`.

Runenwerk's durable responsibility is downstream integration:

```text
accepted standalone RunenGPU revision
    -> exact Runenwerk dependency pin
        -> Runenwerk / RunenRender integration validation
```

A moving standalone branch is not Runenwerk's compatibility contract. Repinning to a
new accepted RunenGPU revision is an explicit integration change with its own validation.
Runenwerk does not keep a parallel G-phase roadmap, semantic design set, or framework
proof matrix after the authority transfer.

Historical G1A-G8/GX ordering and retained proof-role identifiers remain available as
noncanonical evidence in the
[RunenGPU historical proof report](../../reports/design/runengpu-phase-requirements-proof-matrix.md)
and Git history. They are not current activation or framework-roadmap authority.

## RunenRender sequence

RunenRender remains downstream of standalone RunenGPU. ADR 0021 establishes semantic
rendering as its mission and the accepted RunenRender design owns the permanent semantic
architecture.

The normalized direction is:

```text
RenderSceneStore
    -> RenderSceneCommit(RenderSceneSnapshot + RenderSceneChangeSet)

RenderSceneSnapshot + RenderRequest + representation/method contracts
    -> conditional RenderPlan

RenderPlan + current request-scoped semantic bindings
    -> semantic binding admission

semantically admitted candidates
+ representation availability/realization
+ physical output bindings
+ current RunenGPU environment facts
+ execution requirements
    -> execution admission
        -> AdmittedRenderPlan
            -> RenderWorkSet
                -> RunenGPU
                    -> RenderResult
```

Its durable sequence is:

```text
R0  normative semantic-rendering architecture
R1  scene lineage and minimal renderer identity
R2  semantic space/time plus observation/output semantics
R3  representations, intrinsic validity, protocols, relationships, and minimum appearance
R4  RenderMethod plus conditional device-independent semantic planning
R5  semantic bindings, binding admission, operational availability/output bindings, and execution admission
R6  first complete semantic renderer and public RunenGPU lowering
R7  derived state, reconstruction/history/sessions, multiview/multi-output, and advanced output integration
R8  generality, scale, public-surface qualification, conformance, and extraction readiness
RX  standalone RunenRender transfer and clean cutover
```

R0 is architecture/documentation only. The canonical RunenRender design owns detailed
semantics and conformance; the active RunenRender execution plan owns durable delivery
boundaries. GitHub issues determine activation and current status.

## Other repository-family programs

RunenSDF and RunenGPU are standalone semantic owners. RunenECS, RunenSpatial, and RunenUI
continue through separately owned programs and may proceed in parallel only when
repository, branch, workspace, files, authority, and dependencies do not conflict.

A cross-family dependency belongs here only when it is durable architecture or sequence,
not merely because one current implementation happens to be waiting for another.

## Roadmap rules

1. Record only durable phase ordering, dependency direction, extraction direction, and
   cross-family sequencing constraints.
2. Do not copy live issue state, assignees, blockers, branch heads, PR state, check runs,
   exact accepted revisions, or completion tables into this page.
3. Do not use the roadmap to authorize implementation. An owning GitHub issue activates
   work and a reviewed pull request delivers it.
4. Detailed architecture and public contracts belong in accepted ADRs/designs or the
   standalone framework owner, not in roadmap prose.
5. Historical chronology belongs in reports, closeouts, pull requests, and Git history.
6. Do not keep a Runenwerk-local semantic roadmap for a framework after standalone
   authority transfer.
7. Change this roadmap only when durable sequence or dependency truth changes.
