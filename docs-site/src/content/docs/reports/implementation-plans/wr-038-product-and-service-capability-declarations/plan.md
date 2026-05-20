---
title: WR-038 Product And Service Capability Declarations Implementation Contract
description: Design and readiness contract for product and service capability declarations in the Capability Workbench Platform.
status: active
owner: editor
layer: domain / app
canonical: true
last_reviewed: 2026-05-20
related_designs:
  - ../../../design/active/runenwerk-capability-workbench-target-architecture.md
  - ../../../design/active/editor-tool-suite-registry-and-workbench-host-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
related_adrs:
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
---

# WR-038 Product And Service Capability Declarations Implementation Contract

## Status

WR-038 is a design and implementation-readiness slice for PM-WB-CAP-003. It
must not start product code changes until WR-037 host policy is implemented and
the roadmap gates allow enforcement work.

This contract is the promotion package for the future implementation pass. It
records the exact ownership, future interface direction, non-goals, validation,
and stop conditions so implementation does not move product semantics or service
execution authority into `editor_shell`.

## Goal

Make product and service needs explicit Workbench declaration data.

The completed WR-038 design is acceptable when the active Capability Workbench
architecture explains how suites declare product capabilities and service
dependencies while host policy, domain validation, app/service product
formation, and diagnostics remain separate authorities.

## Source Of Truth

- ADR 0012 governs the clean break from legacy Workbench/tool-surface identity.
- The active Capability Workbench target architecture governs capability,
  product, service, and status planes.
- The editor tool-suite registry design governs Workbench suite, profile,
  provider, and host composition boundaries.
- `domain/editor/editor_shell/src/tool_suite` may own stable declaration
  vocabulary only.
- Owning domains keep product semantics, command validation, lowering,
  ratification, and product descriptor ownership.
- Apps and supervised services may form products under host policy, but they do
  not become semantic source truth.

## Readiness

WR-038 is ready for design completion but not ready for Rust implementation.

The blocking dependency is WR-037. Host policy must exist before product and
service declarations can be enforced at provider proposal, product formation,
service invocation, or resource access boundaries.

No new ADR is required for this design slice because it preserves existing
ownership. Add or update an ADR before implementation only if the work moves
product semantics, service execution authority, sandbox policy, or process
trust boundaries into a new owner.

## Implementation Scope

This slice owns:

- a concrete product and service capability plane in the active target design;
- a Material Lab example for product declarations and service dependencies;
- future interface direction for `ToolServiceKey`,
  `ToolSuiteProductDeclaration`, and `ToolSuiteServiceDependency`;
- explicit non-authority rules for `editor_shell`, providers, and services;
- validation and closeout requirements for the future implementation pass.

This slice does not own:

- Rust API changes;
- service execution protocol;
- process ABI or sandbox runtime;
- external dynamic components;
- roadmap or production state promotion;
- host policy implementation;
- product graph implementation;
- Material Lab product behavior changes.

## Future Interface Direction

Future implementation may add `ToolServiceKey`,
`ToolSuiteProductDeclaration`, and `ToolSuiteServiceDependency` under
`domain/editor/editor_shell/src/tool_suite`.

`ProductCapabilityKey` remains declaration and permission vocabulary. It must
not become a domain product descriptor, renderer product id, product cache key,
or semantic product payload type.

`ToolServiceKey` identifies a desired service dependency. It must not grant
permission by itself and must not define process ABI, wire protocol, sandbox
runtime, or external component lifecycle.

The minimum future data flow is:

```text
suite declares product/service need
  -> Workbench registry validates stable keys and duplicates
  -> host policy allows or denies use
  -> owning domain validates source semantics
  -> app or supervised service forms candidate product
  -> product/domain adoption validates outputs where required
  -> status diagnostics record the observed result
```

`editor_shell` may validate declaration shape, stable identity, duplicate
keys, unknown references, and diagnostic vocabulary. It must not import
material, texture, procgen, render, runtime, app provider, or app service types.

## Material Lab Contract Example

Material Lab may declare product capabilities for preview and runtime material
products, using stable keys such as:

```text
runenwerk.material_lab.product.preview_material
runenwerk.material_lab.product.runtime_material
```

These keys allow the host to reason about product permission and status. They
do not make `editor_shell` own material graph validation, shader lowering,
texture binding, renderer payload layout, material product identity, or
prior-valid preservation behavior.

Material Lab may declare service dependencies for future supervised material
workloads, using stable keys such as:

```text
runenwerk.material_lab.service.material_compilation
runenwerk.material_lab.service.preview_formation
```

These keys are metadata only in WR-038. Service protocol, sandbox policy,
external component execution, and service lifecycle belong to later rows.

## Acceptance Criteria

- The active Capability Workbench design contains a concrete Product And
  Service Capability Plane.
- The design states that product declarations are keyed by
  `ProductCapabilityKey` and are not product descriptors.
- The design states that service dependencies are keyed by `ToolServiceKey` and
  are declarations only.
- The design names the enforcement chain from suite declaration through host
  policy, domain validation, app/service formation, and diagnostics.
- The Material Lab example preserves domain-owned material semantics.
- The contract records no Rust API changes for this slice.

## Stop Conditions

Stop before implementation if any step requires:

- moving product semantics into `editor_shell`;
- letting providers or services mutate source truth directly;
- granting capability by installation alone;
- choosing a service process ABI or sandbox runtime;
- implementing external dynamic components;
- enforcing product/service declarations before WR-037 host policy exists;
- introducing compatibility or legacy Workbench identity paths.

## Validation

Required for this design slice:

```text
task docs:validate
```

If roadmap or production YAML is touched in a later pass, also run:

```text
task roadmap:validate
task roadmap:render
task roadmap:check
task production:validate
task production:render
task production:check
```

Future implementation validation must include focused tests for:

- product and service declaration key validation;
- duplicate declaration rejection;
- unknown product or service diagnostics;
- host policy denying product or service use before formation;
- Material Lab declarations remaining semantic-free in `editor_shell`;
- no dependency from `editor_shell` to material, texture, procgen, render,
  runtime, app provider, or app service crates.

## Closeout Requirements

Closeout for this design slice must record:

- changed design and contract paths;
- docs validation result;
- confirmation that no Rust APIs, roadmap state, or production state changed;
- remaining blocker that WR-037 must land before enforcement implementation.

The expected completion-quality tier is `bounded_contract`. The slice is
complete when the design/readiness contract is decision-complete, not when
product or service enforcement exists.

## Perfectionist Closeout Audit

This slice cannot honestly be `perfectionist_verified` because it intentionally
does not implement enforcement. The known gap is active by design: product and
service declarations remain unenforced until WR-037 supplies the host policy
gate and a later implementation pass adds declaration APIs and tests.

Anti-drift checks for the future implementation pass must prevent:

- descriptor-only declarations claimed as enforced policy;
- product status panels claimed as product formation;
- service metadata claimed as service execution;
- provider-owned product semantics;
- `editor_shell` importing domain/app/render service types.
