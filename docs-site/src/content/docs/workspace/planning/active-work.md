---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-03
related_docs:
  - ../engineering-workflow.md
  - ./roadmap.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runengpu-shader-authoring-artifact-boundary.md
  - ../../design/active/runengpu-g4b-contracts-g4c-delivery-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/2026-08-03-runengpu-g4b-g4c-finalization.md
  - ../specs/pt-runengpu-g4a-context-admission.ron
  - ../specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../specs/pt-runengpu-g4c1-resource-realization.ron
  - ../specs/pt-runengpu-g4c2-program-binding-realization.ron
  - ../specs/pt-runengpu-g4c3-pipeline-cutover.ron
---

# Active Work

GitHub issues and pull requests own live delivery. This page records durable accepted
state, the only authorized next RunenGPU slice, and the immediate dependency gates.

## Accepted RunenGPU foundation

```text
S0 inventory                         complete
G1A logical work-resource identity   complete
G2 capabilities and resources        accepted at 709aa6aced020ee99405e1e1c3dde7703c77a4d4
G3 decision phase                    accepted at 5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
operational hardening                accepted at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
G3 Rust implementation               accepted at 39d6fe65a334502bdfba0b1a2ce3b365099fcf28
verified-head maintenance            accepted at 6bbd341691a34763ef54c8ca059940cac8981265
G4 planning                          accepted at 62c3949d31a7c03f1f554f8108120d9767139123
G4A context admission                accepted at 501b9fd58e56d33708573e47faf0e5026b5a1ff2
shader authoring boundary            accepted at 23bc982703f93d15ac39dd71d61bae9e23854141
```

G4A now owns the asynchronous headless-first `GpuContext`, deterministic adapter/device
admission, normalized admitted facts, process-local context/generation identity, private
WGPU instance/adapter/device/queue containment, and the bounded current-host migration
seam.

The accepted shader boundary keeps one runtime path:

```text
consumer-owned meaning
    -> Runenwerk-owned authoring/toolchain policy
        -> canonical WGSL
            -> explicit RunenGPU contracts
                -> private WGPU/Naga realization
```

Plain WGSL is the only initial runtime source kind. WESL and Slang remain separately
gated authoring candidates outside RunenGPU.

## Only authorized RunenGPU continuation

Issue `#209` owns one documentation-only planning correction:

```text
Finalize G4B contracts and decompose G4C before implementation
```

It must bind and publish:

- one bounded stateful source-admission owner for source owner/key/revision/full-source
  consistency;
- a shader-resource-only `GpuProgramInterfaceDescriptor`;
- separate render vertex-input and color-target pipeline state;
- mandatory pinned WGSL/WGPU/Naga agreement while explicit declarations remain
  authoritative;
- concrete compile-shaped compute and render public examples;
- ordered G4C1, G4C2, and G4C3 delivery specifications;
- corrected durable planning and issue ownership.

This is the only authorized RunenGPU slice until its reviewed merge and accepted-main
validation. It changes documentation and planning authority only.

Do not modify RunenGPU Rust, create a G4B implementation branch, or activate a G4C child
through issue `#209`.

## Blocked implementation sequence

```text
#209 accepted planning correction
    -> #187 G4B implementation
        -> G4C1 resource realization
            -> G4C2 program and binding realization
                -> G4C3 pipeline realization and final cutover
                    -> separately planned G5
```

### G4B — blocked issue #187

G4B owns:

- canonical WGSL source admission and consistency;
- typed compute, vertex, and fragment entry points;
- shader-visible resource interfaces and binding declarations;
- bind-group and pipeline-layout descriptors;
- specialization schemas and normalized values;
- generic compute and render pipeline descriptors;
- deterministic runtime-binding compatibility;
- public compile-pass and compile-fail contract proof.

It creates no WGPU object. Issue `#187` activates only after issue `#209` is accepted
and its exact implementation base is re-resolved from current `main`.

### G4C — blocked umbrella issue #188

Issue `#188` remains an umbrella and is not directly implementable.

```text
G4C1 resource realization
    buffers, textures, views, samplers, query sets
    affinity, transactional registries, resource cache compatibility

G4C2 program and binding realization
    canonical WGSL modules
    mandatory parser/reflection agreement
    bind-group layouts, pipeline layouts, typed bind groups

G4C3 pipeline realization and final cutover
    compute/render pipelines
    complete cache keys
    every current consumer migrated
    renderer-owned realization/cache authority deleted
    one bounded G5 bridge retained
```

Each child requires one issue, branch, PR, exact-head validation, complete-diff review,
and accepted merge. No child consumes an unmerged predecessor branch.

## Later RunenGPU program

The remaining program stays sequential and separately authorized:

- G5: headless execution, uploads, submission, progress, pressure, completion,
  asynchronous readback, cancellation, pending-work shutdown, and delayed retirement;
- G6: offscreen graphics, shared render/non-render consumers, direct-WGPU comparisons,
  and cost characterization;
- G7: surfaces, thread affinity, device replacement, loss, and reconstruction facts;
- G8: operational conformance, reproducibility facts, diagnostics, shutdown, cache, and
  residual reach-through audit;
- GX: clean transfer to `dornglut/runen-gpu` only after accepted G2-G8 evidence.

G5, G7, RunenRender implementation, and package extraction remain unauthorized.

## RunenRender boundary

RunenRender remains architecture/design only until accepted external RunenGPU cutover
and a separately authorized R-phase issue.

Its permanent semantic spine is:

```text
RenderSceneStore
    -> RenderSceneCommit(RenderSceneSnapshot + RenderChangeSet)

RenderSceneSnapshot + RenderRequest + RenderInputSet
    -> RenderMethod
        -> RenderPlan
            -> AdmittedRenderPlan
                -> RenderWorkSet
                    -> RunenGPU
```

The canonical RunenRender design owns scene revision, representation, protocol,
material, method, planning, output, derived-state, session, scalability, and
conformance semantics. This page records only the current dependency gate and does not
authorize an R phase or duplicate the R1-R8 plan.

G4 removes GPU/backend realization authority from the current render tree. It does not
rename, move, wrap, or extract the renderer wholesale and does not implement image
formation.

## Acceptance discipline

For the active planning correction and every later implementation slice:

```text
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
```

Repository-owned exact-head CI and Documentation Build plus independent complete-diff
review are required. A green branch does not become accepted authority until merge and
accepted-main verification.
