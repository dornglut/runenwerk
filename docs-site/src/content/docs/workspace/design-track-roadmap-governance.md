---
title: Design Track Roadmap Governance
description: Canonical authority model for Runenwerk designs, ADRs, production tracks, Track Execution Manifests, WR roadmap rows, implementation plans, closeouts, generated registers, and current architecture docs.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-31
related_docs:
  - ./documentation-structure.md
  - ./planning-and-implementation-workflow.md
  - ./production-track-planning-model.md
  - ./track-execution-manifest.md
  - ./prompt-templates/goal-execution.md
  - ../adr/README.md
  - ../design/README.md
---

# Design Track Roadmap Governance

## Purpose

This guide defines the authority model for Runenwerk planning and documentation.
It exists so designs, ADRs, production tracks, Track Execution Manifests, WR
roadmap rows, implementation plans, closeouts, generated registers, and current
architecture docs do not drift or claim the wrong kind of truth.

Use this guide when a change affects planning state, architecture authority,
roadmap execution authority, production milestone completion, closeout evidence,
or generated planning docs.

This guide is governance doctrine. It does not authorize product code, crate
creation, runtime behavior changes, roadmap promotion, or closeout completion by
itself.

## Core Rule

Every document has one job.

Do not use a higher-level planning document to claim lower-level evidence, and
do not use lower-level implementation evidence to silently change architecture
policy.

When documents overlap, first identify the kind of truth being claimed:

- implementation truth: what code and runtime evidence actually do;
- completion truth: what a closeout proves was completed;
- execution truth: which WR row authorizes a bounded change;
- sequencing truth: which milestone should happen next;
- decision truth: which architecture direction is accepted;
- current-reference truth: what a domain or app currently exposes.

## Authority Ladder

The authority ladder is ordered by proof strength for claims about actual
repository state. A document lower in the ladder may guide work, but it cannot
claim a stronger truth without the stronger artifact.

| Authority | Owns | Cannot claim |
|---|---|---|
| Code and runtime evidence | What actually exists and runs. Tests, captures, fixtures, benchmarks, and generated artifacts prove behavior. | Long-term policy by itself. If behavior changes architecture, update an ADR, design, or guideline. |
| Closeouts and reports | Completion evidence, validation output, known gaps, migration proof, runtime proof, and audit findings. | Future implementation authority or new architecture doctrine by themselves. |
| Track Execution Run ledgers | Machine-readable record of each successful locked full-track action, source digests before/after, files changed, validation results, evidence paths, closeout paths, and stop reason. | Architecture doctrine, WR authority, or evidence that was not recorded by the action. |
| WR roadmap rows | Implementation eligibility, dependency legality, blocker state, write scopes, validation expectations, and completion quality for one bounded row. | Strategic product completion without production milestone evidence. |
| Implementation plans | The bounded write contract for one WR or production milestone slice. | Permission to exceed the linked WR, skip roadmap state, or claim completion. |
| Track Execution Locks | Digest-locked permission envelope for AI-executable full-track runs. | Implementation authority without a manifest, WR, plan, writer strategy, validation, and closeout evidence. |
| Track Execution Manifest | Full-track sequencing, explicit next legal action, missing WR blockers, write scopes, forbidden scopes, evidence gates, and closeout paths. | WR authority, implementation permission, roadmap promotion, or closeout evidence. |
| Production tracks | Strategic product outcomes, milestone order, production acceptance criteria, design gates, evidence gates, and target completion quality. | Code permission without WR authority. |
| ADRs | Durable accepted architectural decisions and rejected alternatives. | Detailed implementation sequence or runtime proof. |
| Accepted designs | Approved target architecture, ownership, invariants, migration strategy, and validation expectations. | Implementation completion unless checked against code and evidence. |
| Active designs | Current design work, proposed target shape, proving-domain decisions, and open questions. | Implementation authority or completed behavior. |
| Domain/app current architecture docs | Current reference shape for a domain, app, API, or integration surface. | Future target architecture unless explicitly marked as future work. |
| Generated registers | Searchable projections from structured sources. | Source authority. They must not be edited directly. |

Accepted ADRs and guidelines remain stronger than code for policy. Code can show
that the implementation drifted from policy; it does not make the drift
accepted. The correction is to update code, or to update the governing ADR,
design, or guideline through the normal review path.

## Document Type Responsibilities

### Guidelines

Guidelines define stable repository doctrine.

They own rules such as dependency direction, module structure, documentation
placement, workflow governance, validation policy, and AI-assisted contribution
expectations.

Guidelines must not contain one-off implementation plans or completed phase
evidence.

### ADRs

ADRs record durable architecture decisions.

Use an ADR when the decision is long-term, expensive to reverse, cross-domain,
or likely to constrain future implementation. ADRs explain the decision,
context, rejected alternatives, and consequences.

ADRs must not replace implementation plans, roadmaps, or closeouts.

### Active Designs

Active designs define architecture currently being discussed, validated, or
implemented.

They may define target architecture, proving-domain shape, ownership rules,
contracts, migration paths, and open questions. They must be explicit when they
do not authorize implementation.

Active designs must not claim product completion.

### Accepted Designs

Accepted designs record approved architectural direction.

They may authorize planning and implementation design, but implementation still
flows through WR rows, production milestones, implementation plans, validation,
and closeouts.

Accepted designs must not claim runtime proof unless evidence and closeouts
exist.

### Implemented Designs

Implemented designs are accepted designs checked against actual code and
evidence.

Moving a design to implemented requires verification that implementation exists,
tests pass, and known divergence is resolved or documented.

### Production Tracks

Production tracks define strategic product outcomes.

They own the track id, owner, state, target completion quality, strategic goal,
success criteria, ordered production milestones, design gates, evidence gates,
and production acceptance criteria.

Production tracks guide sequencing. They do not authorize code without WR
roadmap authority.

### Track Execution Manifests

Track Execution Manifests make full-track execution explicit.

They own the complete milestone sequence, authority level, accepted design
dependencies, owning WRs or expansion blockers, predecessor dependencies, write
scopes, forbidden scopes, required contracts, validation commands, evidence
gates, expected closeout paths, stop conditions, and next legal action.

They authorize planning and sequencing only.

### Track Execution Locks

Track Execution Locks make locked AI execution explicit.

They own `ai_executable`, lock author, lock time, manifest digest, source
digests for planning sources and workflow runner sources, granted permissions,
denied permissions, strategic human gates, and invalidation rules. Full-track
execution must fail before mutation when the lock is missing, stale, grants
less authority than requested, denies a requested permission, or lists a
strategic human gate crossed by remaining milestones.

Locks do not authorize implementation by themselves. They only allow the
runner to use manifest, WR, implementation-plan, writer, validation, and
closeout authority without a manual per-milestone prompt.

### Track Execution Run Ledgers

Track Execution Run ledgers record locked full-track execution.

They own the machine-readable action history for a run: pre-action digests,
post-action digests, milestone id, WR id, strategy used, files changed,
validation results, evidence paths, closeout paths, next legal action, and stop
reason. Markdown reports may summarize ledgers, but YAML ledgers are the run
authority.

### Roadmaps And WR Rows

WR roadmap rows are the execution authority for bounded work.

They own readiness state, dependency level, blocker state, scoring, write
scopes, validations, dependencies, current call, first move, next evidence,
completion quality, known gaps, and completion audit links.

A WR row must not close without evidence.

### Implementation Plans

Implementation plans are bounded write contracts.

They own one slice's source of truth, readiness, exact scope, non-goals,
expected owners/modules/files, implementation steps, validation, stop
conditions, and closeout requirements.

Implementation plans must not broaden the WR scope or change architecture
policy silently.

### Closeouts

Closeouts record what actually happened.

They own evidence, changed files, validation results, runtime proof, known
gaps, completion quality, roadmap updates, production updates, and follow-up
handoff decisions.

Closeouts must not implement missing behavior or claim a stronger quality tier
than the evidence supports.

### Generated Registers

Generated registers summarize structured sources.

Examples include roadmap registers, production track indexes, production
milestone registers, and diagrams generated from YAML sources.

Generated registers are outputs. Edit the source YAML or source docs, then run
the render/check tasks.

### Domain And App README / Current Architecture Docs

Domain and app current architecture docs describe the current public shape,
ownership, and integration boundaries.

They should link to accepted designs, ADRs, roadmaps, and closeouts when future
or historical context matters. They must not become hidden roadmaps or policy
overrides.

## Lifecycle Rules

### Design Lifecycle

Designs move through:

```text
active -> accepted -> implemented
```

They may also move to:

```text
deferred
superseded
rejected
archived
```

Rules:

- active designs may guide discussion and planning;
- accepted designs may guide implementation planning;
- implemented designs require code and evidence review;
- deferred designs must state reactivation conditions;
- superseded designs must link to the replacement;
- rejected designs must explain the chosen alternative.

### ADR Lifecycle

ADRs move through:

```text
proposed -> accepted
proposed -> rejected
accepted -> superseded
```

Rules:

- proposed ADRs may inform planning, but do not overrule accepted ADRs;
- accepted ADRs are durable decision authority;
- superseded ADRs must link to the replacing ADR or guideline;
- rejected ADRs preserve alternatives and rationale.

### Production Track Lifecycle

Production tracks move through states such as:

```text
designing
ready_next
active
completed
blocked
deferred
```

Rules:

- designing tracks or milestones resolve architecture, ownership, contracts, or
  acceptance criteria;
- ready_next means required gates are satisfied and WR links exist;
- active means the milestone is current focus, still governed by WR rows;
- completed requires evidence gates;
- blocked must name a concrete blocker;
- deferred must state why it is not active.

### Track Execution Manifest Lifecycle

A manifest is created before full-track `/goal` execution.
The machine-readable source under
`docs-site/src/content/docs/workspace/track-execution-manifests/<track-id>.yaml`
is the execution authority. Human-readable reports under
`docs-site/src/content/docs/reports/track-execution-manifests/` are mirrors and
must not be parsed as authority.

Lifecycle:

```text
create or update manifest
run Track Expansion if WR rows are missing
run Track Readiness Audit
use manifest for one legal action at a time
update manifest when the track sequence, WR links, blockers, or closeout paths change
archive or supersede only when the track is completed or replaced
```

Rules:

- the manifest must make missing WR rows blockers;
- the manifest must never invent WR authority;
- the manifest must stop `/goal` at unmet gates;
- if the manifest disagrees with production track or roadmap state, `/goal`
  must fail closed with diagnostics;
- a stale manifest is a blocker for full-track execution.

### WR Roadmap Lifecycle

WR rows move through the repository roadmap lifecycle defined by the structured
roadmap sources.

Rules:

- new work enters through intake or explicit roadmap editing;
- implementation requires dependency legality and ready state;
- write scopes and validation must be explicit before code changes;
- completed rows must include completion quality, known gaps, and a completion
  audit path;
- deferred rows must not be described as completed.

### Implementation Plan Lifecycle

Implementation plans are created or updated after the production milestone and
WR row are known.

Lifecycle:

```text
task production:plan
write or update plan
critically review plan
implement only the bounded scope when authorized
validate
close out
update roadmap and production evidence
```

Rules:

- the plan must be decision-complete before code starts;
- if an answer is still a choice, update the plan instead of coding;
- plans expire when source truth, write scope, dependencies, or gates change.

### Closeout Lifecycle

Closeouts are written after bounded work and validation.

Rules:

- record exact changed files and modules;
- record validation commands and results;
- record evidence paths and artifact identity;
- record known gaps honestly;
- update roadmap state after evidence exists;
- update production milestone state only when evidence gates are satisfied.

### Generated Docs Lifecycle

Generated docs are rebuilt from structured sources.

Rules:

- never edit generated registers directly;
- update YAML or owning source docs first;
- run render commands after source changes;
- run check commands before claiming generated docs are current.

## Quality Levels

### `not_applicable`

Use when a row, milestone, or governance action does not complete an
implementation or product capability.

Examples:

- intake-only rows;
- governance-only activation;
- deferred planning;
- design-only blockers.

### `bounded_contract`

Use when a bounded contract or slice is complete and validated, but it is not a
full runtime product proof.

Examples:

- design promotion;
- contract-only milestones;
- scoped implementation with explicit known gaps.

### `runtime_proven`

Use when runtime, headless, artifact, diagnostic, source-map, visual, GPU, or
other product-relevant evidence proves the claimed capability through the
actual runtime path.

Documentation-only completion cannot be `runtime_proven`.

### `perfectionist_verified`

Use only when a completed audit path exists and the known gap list is empty.

This tier requires stronger proof than `runtime_proven`: no unresolved
architecture, module-structure, UI, runtime, validation, documentation, or
future-scope gaps may remain inside the claim.

## Roadmap Rules

Use these boundaries exactly:

- production track = strategic outcome;
- WR roadmap = implementation authority;
- Track Execution Manifest = execution sequence;
- implementation plan = bounded write contract;
- closeout = evidence.

Additional rules:

- production intent never overrides execution legality;
- WR readiness never proves production completion by itself;
- implementation plans cannot skip WR state;
- manifests cannot fill in missing WR rows;
- closeouts cannot claim runtime proof without runtime evidence;
- generated registers cannot become source truth;
- `/goal` coordinates one legal action at a time and must stop at unmet gates.

## Required Fields And Checklists

### Design Docs

Required or intentionally omitted:

- title, description, status, owner, layer, canonical marker, last reviewed;
- purpose and scope;
- non-goals;
- architectural position;
- ownership rules;
- dependency rules;
- public API or contract policy when applicable;
- invariants;
- failure modes and diagnostics;
- persistence, schema, migration, or versioning when applicable;
- validation plan;
- open questions;
- relationship to ADRs, roadmaps, and closeouts.

### ADRs

Required or intentionally omitted:

- status;
- context;
- decision;
- rejected alternatives;
- consequences;
- related designs, guidelines, or implementation evidence;
- supersession link when replaced.

### Production Tracks

Required:

- stable track id;
- title;
- state;
- owner;
- target completion quality;
- strategic goal;
- success criteria;
- ordered milestones;
- explicit non-extraction or non-scope statements when relevant.

### Production Milestones

Required:

- stable milestone id;
- title;
- kind;
- state;
- goal;
- outcome;
- dependencies;
- roadmap links or explicit reason they do not exist yet;
- design gates;
- evidence gates before completion;
- acceptance criteria;
- completion quality for completed milestones;
- known quality gaps;
- completion audit for completed milestones.

### Track Execution Manifests

Required:

- track id;
- authority level;
- accepted design dependencies;
- milestone sequence;
- owning WR per milestone, or explicit Track Expansion blocker;
- predecessor dependencies;
- exact write scope;
- forbidden scope;
- required contracts;
- validation commands;
- evidence gates;
- expected closeout path;
- stop conditions;
- next legal action;
- milestone kind: docs-only, design-only, implementation, hardening, or
  closeout;
- whether code is allowed;
- whether crate creation is allowed;
- whether production behavior may change.

### WR Roadmap Rows

Required:

- stable WR id;
- title;
- lane;
- dependency level;
- planning state;
- priority and scoring fields;
- dependencies;
- write scopes;
- validations;
- next evidence;
- current decision;
- current call;
- first move;
- main blocker or why not ready;
- completion quality;
- known quality gaps;
- completion audit when completed;
- decision gates when architecture-sensitive.

### Implementation Plans

Required:

- goal;
- source of truth;
- readiness and blocker state;
- exact write scope;
- forbidden scope;
- owning domains, crates, modules, and expected files;
- non-goals;
- implementation steps;
- public API, data-flow, diagnostics, persistence, or migration impact;
- validation commands;
- stop conditions;
- closeout requirements;
- completion-quality expectation.

### Closeouts

Required:

- completed scope;
- changed files and owning modules;
- evidence paths and artifact identity;
- validation commands and results;
- runtime, headless, visual, benchmark, migration, or diagnostic evidence
  relevant to the claim;
- known quality gaps;
- completion quality;
- roadmap updates made or required;
- production updates made or required;
- follow-up work and handoff boundaries.

## Anti-Patterns

- active design claims implementation;
- accepted design claims runtime proof without evidence;
- production track authorizes code without WR authority;
- manifest invents WR authority;
- roadmap row closes without closeout evidence;
- implementation plan expands beyond the WR write scope;
- generated register is edited directly;
- closeout claims `runtime_proven` from docs-only evidence;
- root docs become authority instead of summaries;
- implementation starts before write scopes are explicit;
- deferred work is described as completed;
- `blocked_deferred` hides multiple unrelated blockers without a concrete next
  legal action;
- `/goal` crosses milestone boundaries without closeout and rerun;
- closeout milestone implements missing prior-slice behavior;
- current architecture docs silently override accepted ADRs or designs.

## Validation Rules

Run the smallest validation that proves the changed authority layer, and run the
broader gate when the change touches multiple layers.

| Change type | Required validation |
|---|---|
| Docs-only prose change | `task docs:validate` |
| Roadmap source or roadmap evidence change | `task roadmap:render`, `task roadmap:validate`, `task roadmap:check` |
| Production track or milestone source change | `task production:render`, `task production:validate`, `task production:check` |
| Planning workflow, manifest, roadmap, production, or generated planning docs change | `task planning:validate` |
| Broad integration, code behavior, or runtime evidence change | `task ci:local` when required by the implementation contract or closeout tier |

Validation success means the checked layer is structurally consistent. It does
not by itself prove runtime behavior or completion quality.

For manifest-backed tracks, `task production:validate` also audits the
machine-readable Track Execution Manifest. It fails on manifest/production/WR
conflicts, missing manifest governance fields, docs-only milestones that
authorize code, docs-only `runtime_proven` claims, completed milestones without
expected closeout evidence, and completed manifest-backed milestones whose
owning WR rows are not completed.
Tracks marked `full_automation_target: true` must also pass full automation
readiness validation. Remaining milestones must declare strict automation
`execution_kind` values, required permission classes, exact scopes, validation
commands, closeout contracts, evidence categories, and stop conditions.
Tracks intended for locked AI full-track execution must also have
`ai_executable: true` in the manifest and a current Track Execution Lock under
`workspace/track-execution-locks/<track-id>.yaml`.

Manifest-backed `/goal` and `production:next` commands must run the same
manifest audit before printing normal next-action guidance. When the manifest is
a full automation target, they must also report full automation readiness and
must not claim that gates are clear while preflight blockers remain. Alignment
errors, missing gates, invalid blocked fields, invalid closeout paths, WR scope
mismatches, and missing WR authority are stop conditions.

`task production:run-track -- --allow auto_safe` is the accepted Manifest
Runner V1 mutation path. It may perform one mechanical Track Expansion action:
allocating a deferred WR id, creating the deferred WR metadata row, linking the
production milestone, updating the manifest `owning_wr`, regenerating generated
planning docs, and running validation. It is still planning/sequencing
authority only. It cannot write design content, close milestones, create code or
crates, modify runtime behavior, start MaterialProgram or RenderPlan work, or
authorize shared `foundation/meta` extraction.

`task production:run-track -- --allow auto_safe --allow agent_design --deny
product_code` may additionally run the Manifest Runner V2 design/planning layer
for the current docs-only or design-only milestone. It may create or update the current
milestone implementation/design plan and revise bounded design contract sections
only inside the exact manifest and WR write scope. It cannot close WRs or
production milestones, start implementation, create crates, modify product
behavior, or extract shared `foundation/meta`.

`task production:run-track -- --allow auto_safe --allow agent_design --allow
agent_closeout --deny product_code` may additionally run the Manifest Runner V3
closeout layer for docs, design, or governance milestones. It may close only as
`bounded_contract` after the expected evidence exists, validation passes, and
the manifest/production/roadmap/WR state agree. It cannot close runtime proof
milestones, mark `runtime_proven`, start the next milestone's design authoring,
create crates, modify product behavior, or extract shared `foundation/meta`.

`task production:run-track -- --allow auto_safe --allow agent_design --allow
agent_closeout --allow product_code` may run the Manifest Runner V4 product-code
gate only for the current implementation or hardening milestone. Add
`--allow product_implementation` only when Manifest Runner V5 is allowed to
write exact product files from the active WR and accepted production plan. Both
layers require a concrete active WR, accepted production plan, exact write
scope, explicit forbidden scope, validation commands, closeout path,
compatibility/rollback plan, and stop conditions. They must fail closed for
docs, design, or governance milestones, future or deferred WRs, missing plans,
broad scopes, unmarked new files, crate creation, shared `foundation/meta`
extraction, and MaterialProgram implementation. Full-track runs must use
`--mode full-track`; otherwise the runner is limited to single-step or
bounded-segment authority and must not infer full-track intent from the
permission set alone.

`task production:run-track -- --mode full-track ...` is locked-track
automation. It must verify the Track Execution Lock before mutation, run
full-track preflight over every remaining milestone, execute one legal action at
a time, recompute state after each action, append to the Track Execution Run
ledger, and stop only for completion, failed validation, missing evidence,
scope mismatch, ungranted permission, strategic human gate, max-actions, or a
new architecture decision not covered by accepted design.

## Conflict Resolution

When documents disagree:

1. Identify the claim type: implementation, evidence, execution, sequencing,
   decision, or reference.
2. Use the strongest authority for that claim type.
3. If code and accepted doctrine disagree, treat it as drift until an accepted
   ADR, design, or guideline changes the doctrine.
4. If generated registers disagree with YAML or source docs, regenerate the
   registers.
5. If a closeout disagrees with runtime evidence, correct the closeout or add a
   superseding evidence correction.
6. If a production track disagrees with WR state, the WR state governs
   implementation legality.
7. If a manifest disagrees with production metadata or WR rows, stop and update
   the manifest before `/goal` continues.

## Goal Execution Rule

`/goal` is persistence, not authority.

For production-track work, `/goal` must:

- use `production-tracks.yaml` for milestone order;
- consult the Track Execution Manifest when one exists;
- fail closed before normal guidance if the manifest audit is blocked;
- use linked WR rows for execution legality;
- use implementation plans for write scope;
- use closeouts for completion evidence;
- perform exactly one legal next action;
- validate;
- close out when appropriate;
- rerun the coordinator before continuing.

It must stop when authority is missing, evidence is missing, validation fails,
or the next action would cross a milestone boundary.

## Maintenance

Update this guide when Runenwerk adds a new planning artifact type, quality
level, workflow command, generated register, or closeout evidence tier.

Do not duplicate every workflow detail here. Keep detailed procedures in their
own documents and link them from this authority model.
