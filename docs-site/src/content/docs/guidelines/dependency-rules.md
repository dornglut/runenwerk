---
title: Dependency Rules
description: Canonical dependency direction and clean-cutover rules for Runenwerk and peer frameworks.
status: active
owner: workspace
layer: guideline
canonical: true
last_reviewed: 2026-07-22
related_docs:
  - ../architecture/repository-family-architecture.md
  - ../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../adr/accepted/0015-separate-gpu-execution-from-rendering.md
---

# Dependency Rules

## Repository direction

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

Framework repositories do not depend on Runenwerk. A direct framework dependency requires an accepted ADR proving correct ownership and independent value.

The accepted direct framework dependency is:

```text
RunenRender -> RunenGPU
```

RunenGPU and RunenRender each begin with one public package. Additional packages require a concrete dependency, backend, release, ABI, platform, or compile-time reason.

## Layer direction inside Runenwerk

Until clean cutovers complete:

```text
foundation -> domain -> engine/runtime -> apps/adapters/tools
```

- Foundation contains low-level reusable vocabulary and does not depend on domain, runtime, editor, app, adapter, workflow, UI, or concrete backend code.
- Domain code may depend on foundation and lower-level domain contracts, but not on runtime, app wiring, or concrete backends it does not own.
- Runtime composes domains, frameworks, and backend implementations without moving product or source-domain meaning into reusable APIs.
- Apps and tools compose higher layers but do not define framework or domain invariants.

## Framework boundaries

### RunenGPU

RunenGPU may own WGPU internally and owns normalized capabilities, GPU resources and views, access and lifetime rules, hazards, workloads, submission, upload/readback, low-level surfaces, backend outcomes, and GPU diagnostics.

RunenGPU does not own renderer, ECS, SDF, UI, world, editor, application, or product semantics. It does not own Winit event-loop policy, shader filesystem watching, or product recovery.

### RunenRender

RunenRender depends on RunenGPU. It may own prepared render scenes, views and logical targets, providers and interactions, materials and media, visibility, transport, caches, reconstruction, overlays, color, presentation intent, and lowering render plans into RunenGPU workloads.

RunenRender does not depend directly on WGPU, Winit, Runenwerk, RunenSDF, RunenECS, RunenUI, source scene/material/editor domains, or application lifecycle.

### RunenSDF, RunenECS, and RunenUI

- RunenSDF remains backend-neutral and owns field and numerical semantics.
- RunenECS owns ECS lifecycle, query, storage, and system semantics.
- RunenUI owns semantic UI, state/actions, focus/accessibility, layout/style/text, hit testing, and renderer-neutral paint output.

Display or acceleration does not make these frameworks depend on RunenGPU or RunenRender. Cross-framework translation belongs in Runenwerk adapters until independent reuse proves another owner.

## Adapter rules

A Runenwerk adapter may depend on Runenwerk and the public contracts it translates between. Frameworks do not depend back on the adapter.

Adapters translate identities, prepared inputs and outputs, lifecycle facts, diagnostics, provenance, and ownership. They do not duplicate algorithms, mirror source, expose mutable internals, or hide dependency cycles.

## Version and cutover rules

Before stable publication, Runenwerk pins an exact accepted revision or exact pre-release version. Moving branch dependencies are forbidden.

A completed extraction leaves:

- one external source authority;
- one-way dependency direction;
- exact dependency pinning;
- every active consumer migrated;
- no original source copy;
- no forwarding package or namespace;
- no submodule, source include, moving branch dependency, or long-lived migration facade;
- no duplicate runtime path.

Temporary duplication is allowed only on an unmerged extraction branch. If Runenwerk has no real consumer, internal source may be removed without adding an unused external dependency.

## Boundary escalation

When one owner wants another owner's internals, determine whether the missing boundary is a public value, command, diagnostic, adapter, capability, workload, contribution, or test-support contract. Do not solve boundary pressure with a universal shared core, speculative package, compatibility facade, or exposed mutable internals.
