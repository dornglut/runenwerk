---
title: Active Designs
description: Architecture designs currently being discussed, implemented, or validated.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-09-13
---

# Active Designs

Use this folder for designs that are useful but not yet fully accepted or not yet
checked against implementation.

Active does not mean wrong or deprecated. It means the design is still part of
current planning, implementation, or validation work.

## Architecture spine

For Runenwerk-wide architecture, start with:

- [Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md)

For Runenwerk-local UI architecture and the standalone ownership boundary, start with:

- [Runenwerk UI Local Runtime and Integration Architecture](../../architecture/ui-framework-architecture.md)

Active design docs below are slice-level authorities, proposals, implementation
designs, or product references. They should not restate standalone reusable
framework authority inside Runenwerk. Accepted ADRs and canonical architecture
spines win when an active design contains older conflicting target language.

## Current Designs

### Editor Domain

- [Editor Asset Pipeline and Content Workflow Design](editor-asset-pipeline-and-content-workflow-design.md)
- [Editor Procedural Content and Simulation Workflow Plan](editor-procedural-content-and-simulation-workflow-plan.md)
- [Editor UI Workspace Tool Surface Architecture](editor-ui-workspace-tool-surface-architecture.md)

### Engine Runtime

- [Game Runtime, Editor, ECS, Scripting, and Hot Reload Design](engine-game-runtime-editor-ecs-scripting-hot-reload-design.md)

### Gameplay

- [Gameplay Graph ATR IR and ECS Lowering Design](gameplay-graph-atr-ir-and-ecs-lowering-design.md)

### Net

- [ECS Net Replication Boundary](ecs-net-replication-boundary.md)
- [Net Authoritative Replication Protocol](net-authoritative-replication-protocol.md)
- [Net Diagnostics Inspection](net-diagnostics-inspection.md)
- [Net Plugin Runtime Bridge](net-plugin-runtime-bridge.md)
- [Net Reconnect History Recovery](net-reconnect-history-recovery.md)
- [Net Transport Lanes Delivery](net-transport-lanes-delivery.md)

### Drawing / Apps

- [Runenwerk Draw Pen-First Radial Tablet UX Design](runenwerk-draw-pen-first-radial-tablet-ux-design.md)

### Repository Family Extraction

- [RunenRender Decomposition Execution Plan](runenrender-internal-decomposition-execution-plan.md)

Standalone RunenGPU owns current reusable GPU execution semantics, public contracts,
conformance, and framework evolution. Runenwerk does not keep active RunenGPU semantic
predecessor designs after the accepted standalone transfer.

### Domain Authoring / App Proof

- [Typed App Program Counter Proof Design](typed-app-program-counter-proof-design.md)

### Workspace / Cross-Domain

- [Drawing Authoring and Comic Layout Platform Design](drawing-authoring-and-comic-layout-platform-design.md)
- [Drawing Domain Crate Design](drawing-domain-crate-design.md)
- [Material Lab And Material Preview Design](material-lab-and-material-preview-design.md)
- [Native Tablet Input and Latency Contract](native-tablet-input-and-latency-contract.md)
- [Runenwerk UI Story V2 Consumer and Proof Boundary](runenwerk-ui-story-driven-golden-workflow-design.md)
- [Semantic Graph IR and Compilation Design](semantic-graph-ir-and-compilation-design.md)
- [Shader Authoring and Canonical Artifact Policy](shader-authoring-and-canonical-artifact-policy.md)
