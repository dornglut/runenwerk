---
title: Runenwerk UI Story V2 Acceptance and Review Checklist
description: Current Runenwerk-local acceptance and review checklist for Story V2 workflow proof, gallery/CLI consumers, and mount decisions.
status: active
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../../design/active/runenwerk-ui-story-driven-golden-workflow-design.md
related_docs:
  - ./architecture.md
  - ./roadmap.md
  - ../../architecture/ui-framework-architecture.md
---

# Runenwerk UI Story V2 Acceptance and Review Checklist

## Purpose

This checklist summarizes the review bar for the current Runenwerk-local Story V2
workflow. It is a consumer/proof checklist over current Runenwerk code; it is not
a reusable UI-framework conformance specification.

Standalone [`dornglut/runen-ui`](https://github.com/dornglut/runen-ui) owns
future reusable framework testing and product-maturity semantics.

## Global rule

A Story V2 claim is valid only when the selected workflow graph and its
owner-produced evidence pass the manifest's declared expectation. A rendered or
visible result alone is not sufficient proof.

The former rule "every UI unit must pass through `UiStoryRunReport`" is obsolete.
Current code uses V2 manifests, workflow profiles, workflow reports, and mount
decisions.

## Manifest V2 checklist

A current local story manifest should provide valid:

- [ ] schema version;
- [ ] stable story id;
- [ ] story revision;
- [ ] title and category id;
- [ ] source descriptor;
- [ ] program id where required by the selected workflow;
- [ ] host profile id;
- [ ] theme profile id;
- [ ] viewport matrix;
- [ ] workflow profile id;
- [ ] expected outcome;
- [ ] mount policy.

The manifest selects a workflow profile; it must not smuggle in a second ad-hoc
stage graph.

## Workflow profile checklist

Current built-in profile ids include:

```text
ui_story.workflow.source_load_only
ui_story.workflow.compiler_only
ui_story.workflow.static_preview
ui_story.workflow.executable_interaction_proof
```

Review passes when:

- [ ] the profile id is registered;
- [ ] the graph is valid and deterministic;
- [ ] required dependencies between nodes are present;
- [ ] no consumer silently inserts a second private workflow;
- [ ] evidence is attached to the node that owns the observation;
- [ ] skipped/blocked nodes remain explicit rather than being reported as pass.

## Source and program evidence

Where the selected workflow includes source/program work:

- [ ] source identity is stable and source maps remain attributable;
- [ ] authored source does not contain runtime widget, ECS, renderer, or app
  mutation identity;
- [ ] definition/schema validation fails closed;
- [ ] unknown control kinds do not fabricate package/program facts;
- [ ] `UiProgram` formation receives explicit package/catalog authority where
  required;
- [ ] diagnostics remain stable and source-attributable;
- [ ] program rows and capabilities are owned by the current local program
  contracts, not inferred by the story runner.

## Compiler and runtime evidence

Where the selected workflow includes compilation/runtime observation:

- [ ] compiler diagnostics pass;
- [ ] a valid runtime artifact is produced when required;
- [ ] artifact tables remain inspectable;
- [ ] runtime/read-model evidence derives from accepted artifact/runtime inputs;
- [ ] runtime evidence does not reread or reinterpret authored source as a
  parallel semantic path;
- [ ] retained-runtime evidence and artifact-backed evidence are not conflated
  into a claim that one local execution path has fully replaced the other.

## Static preview workflow

For `ui_story.workflow.static_preview`, review the current graph as an ordered
proof dependency:

```text
manifest
  -> source_load
  -> source_parse
  -> program_formation
  -> compiler
  -> runtime_view
  -> render_primitives
  -> render_data
  -> static_mount
  -> preview_frame
```

Review passes when every required node has valid owner-produced evidence and the
aggregate workflow outcome matches the manifest expectation.

## Executable interaction workflow

For `ui_story.workflow.executable_interaction_proof`, review the current
interaction proof chain, including:

```text
interaction_story
  -> interaction_replay
  -> live_interaction_proof
  -> replay_live_parity
  -> interaction_static_mount
  -> preview_frame
```

Verify that:

- [ ] replay and live proof use the declared workflow rather than a second
  private runner;
- [ ] route/action facts remain schema/capability checked;
- [ ] interaction does not directly mutate app/editor/game truth inside generic
  UI owners;
- [ ] replay/live parity failures are visible in the workflow report;
- [ ] preview evidence cannot hide a failed interaction dependency.

## Workflow report V2

`UiStoryWorkflowReportV2` is acceptable when it truthfully exposes:

- [ ] workflow identity/graph;
- [ ] node reports and outcomes;
- [ ] diagnostics;
- [ ] aggregate outcome;
- [ ] expected-failure matching where applicable;
- [ ] first blocker when the workflow cannot pass.

The report is proof/inspection data. It is not authored UI source, product state,
or renderer authority.

## Expected-failure checklist

Expected failure is accepted proof only when:

- [ ] the story explicitly expects failure;
- [ ] the observed failing node/diagnostic matches the declared expectation;
- [ ] unrelated failures are not hidden by the expected-failure declaration;
- [ ] the aggregate report records the expected failure truthfully;
- [ ] the story is not treated as production-mount eligible.

## Mount decision V2

`UiStoryMountDecisionV2` must remain fail-closed.

A positive local decision requires:

```text
mount_policy == EligibleWhenPassed
required preview proof == passed
workflow outcome == Passed
```

Review must reject mounting when:

- [ ] workflow validation failed;
- [ ] the story is an expected-failure case;
- [ ] mount policy is `Never`;
- [ ] mount policy is gallery-only for the attempted product mount;
- [ ] required preview evidence is absent, blocked, or failed;
- [ ] aggregate workflow outcome is not `Passed`.

A positive story mount decision still does not bypass app/engine authorization,
route/capability checks, product lifecycle policy, or host-owned mutation.

## Gallery and CLI checklist

Current local gallery/CLI consumers are acceptable when:

- [ ] checked-in stories are discovered through the V2 registry/catalog path;
- [ ] gallery and CLI consume the same domain-owned workflow semantics;
- [ ] neither keeps a button-specific or private semantic runner;
- [ ] inspection presents workflow/node diagnostics rather than guessing success
  from pixels;
- [ ] expected failures are distinguishable from regressions;
- [ ] mount decisions are shown as derived proof, not as product mutation
  authority.

## Ownership checklist

Story orchestration may aggregate evidence but must preserve the semantic owner:

- [ ] `ui_definition` owns definition/normalization facts;
- [ ] `ui_program_lowering` owns local program-formation facts;
- [ ] `ui_compiler` / `ui_artifacts` own compiler/artifact facts;
- [ ] runtime/read-model owners own runtime evidence;
- [ ] render/static/headless owners own renderer-neutral output evidence;
- [ ] app/editor/game hosts own domain state, commands, effects, IO, and concrete
  mutation.

## Local-versus-framework boundary

Review must stop or split when a change tries to use this checklist to define
future reusable framework semantics such as:

- generic future controls or composition kits;
- framework-wide animation/transition policy;
- virtualization architecture;
- platform/windowing/OS integration;
- reusable renderer/backend architecture;
- production accessibility/text maturity;
- reusable framework devtools or release qualification.

Those questions belong to standalone RunenUI unless a new Runenwerk consumer
issue is specifically integrating an accepted RunenUI contract.

## Review evidence

For a Runenwerk Story V2 change, record:

- [ ] exact changed files and owning issue;
- [ ] exact source and test evidence used for behavior claims;
- [ ] focused validation actually executed;
- [ ] canonical repository validation for the unchanged reviewed head;
- [ ] any host/product behavior that remains outside story ownership;
- [ ] whether any standalone RunenUI consumer boundary changed (normally no).

## Stop conditions

Stop and redesign if a change:

- [ ] restores the retired flat `UiStoryRunReport` authority;
- [ ] invents a second workflow runner for gallery or CLI;
- [ ] treats visible output as success after an upstream workflow failure;
- [ ] gives `ui_story` app/editor/game mutation ownership;
- [ ] gives renderer code UI semantic ownership;
- [ ] creates a Runenwerk-local future framework roadmap that competes with
  standalone RunenUI;
- [ ] claims standalone RunenUI adoption without an explicit accepted consumer
  cutover.
