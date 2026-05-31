---
title: Track Execution Manifest
description: Planning-stage contract that makes full production-track goal execution explicit, gated, and drift-resistant before slice implementation begins.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-31
related:
  - ./planning-and-implementation-workflow.md
  - ./production-track-planning-model.md
  - ./production-tracks.yaml
  - ./roadmap-items.yaml
  - ./roadmap-deferred.yaml
  - ./prompt-templates/goal-execution.md
---

# Track Execution Manifest

## Purpose

A Track Execution Manifest is the planning contract between a production track
and `/goal` execution. It makes a full-track plan explicit before Codex or a
human coordinator starts stepping through milestones.

The manifest exists because production tracks are strategic and WR rows are
execution authority. Without a manifest, a full-track `/goal` must infer missing
WR rows, evidence gates, dependency order, closeout paths, and blockers from
scattered production and roadmap sources. That inference is where drift enters.

## Workflow Stage

Long-running production work should follow this model:

```text
Track Definition
-> Track Execution Manifest
-> Track Expansion
-> Track Readiness Audit
-> Slice Contracting
-> Slice Implementation
-> Slice Closeout
-> Track Closeout
```

## Authority

The manifest authorizes planning and sequencing only.

It does not authorize implementation by itself. It does not create runtime
permission, product-code permission, crate-creation permission, roadmap
promotion permission, or closeout evidence.

Rules:

- each implementation slice still requires an active WR row or accepted
  roadmap action;
- each implementation slice still requires `task production:plan -- --milestone
  <PM-ID> --roadmap <WR-ID>`;
- `/goal` may use the manifest to choose exactly one next legal action;
- `/goal` must stop at unmet design, ADR, WR, validation, evidence, dependency,
  or closeout gates;
- `/goal` must never invent WR authority or closeout evidence;
- completed milestones must still have evidence gates that reference completed
  closeout or report paths;
- generated roadmap and production docs remain outputs, not authority.

## Manifest Source

`/goal` execution authority comes from the machine-readable manifest source, not
from free-form Markdown report text. The canonical source path is:

```text
docs-site/src/content/docs/workspace/track-execution-manifests/<track-id>.yaml
```

Human-readable reports may mirror that source under:

```text
docs-site/src/content/docs/reports/track-execution-manifests/<track-id>/manifest.md
```

If the YAML source and production or roadmap state disagree, workflow tooling
must fail closed and report the conflict rather than guessing.

## Manifest Content

A manifest must include these fields for the track:

- track id;
- authority level;
- accepted design dependencies;
- global forbidden scope;
- global validation commands;
- global stop conditions;
- milestone sequence.

Each milestone entry must include:

- milestone id and title;
- milestone kind;
- next legal action;
- owning WR, or the explicit Track Expansion requirement that must create it;
- predecessor dependencies;
- exact write scope;
- forbidden scope;
- required contracts;
- validation commands;
- evidence gates;
- expected closeout path;
- stop conditions;
- whether the milestone is docs-only, design-only, implementation, hardening,
  or closeout;
- whether the milestone may create code;
- whether the milestone may create crates;
- whether the milestone may modify production behavior.

For milestones with an owning WR, every path-like manifest write-scope entry
must be covered by that WR's write scopes. Machine-readable manifest YAML is
authority, not generated output, unless a write-scope entry explicitly uses a
`generated:` or `derived:` marker. Unmarked generated or derived scope text is a
validation blocker.

## Command Model

These commands are the workflow interface. `plan-track`, `next`, `audit-track`,
and `run-track` are implemented as repository tooling. `expand-track` remains a
read-only candidate listing command. The first automation layer is Manifest
Runner V1 can perform one permission-gated `auto_safe` Track Expansion action
and then must stop unless V2 `agent_design` is also explicitly allowed. Manifest
Runner V2 can perform bounded design/planning document writes for the current
docs-only or design-only milestone when the manifest provides an `agent_design`
contract and the owning WR covers the exact write scope. Manifest Runner V3 can
close docs, design, or governance milestones as `bounded_contract` only when the
required evidence exists, validation passes, and the milestone remains
implementation-forbidden. Manifest Runner V4 can pass a product-code execution
gate only for the current implementation or hardening milestone when a concrete
active WR, accepted production plan, exact write scopes, forbidden scopes,
validation commands, closeout path, rollback/compatibility plan, and stop
conditions are all present.

```text
task production:plan-track -- --track <TRACK_ID>
task production:expand-track -- --track <TRACK_ID>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --max-actions <N>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --allow agent_design --deny product_code --max-actions <N>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --allow agent_design --allow agent_closeout --deny product_code --max-actions <N>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --allow agent_design --allow agent_closeout --allow product_code --max-actions <N>
task production:next -- --track <TRACK_ID>
task production:audit-track -- --track <TRACK_ID>
```

`production:plan-track`:

- creates a conservative Track Execution Manifest scaffold when one is missing;
- audits an existing Track Execution Manifest without overwriting it by default;
- records staged milestone sequence;
- records accepted design dependencies;
- records authority level and stop conditions;
- does not authorize implementation.

`production:expand-track`:

- prints deferred WR candidates from the manifest;
- remains read-only;
- does not authorize implementation;
- use `production:run-track -- --allow auto_safe` for the guarded V1 mutation
  path.

`production:run-track`:

- runs the full manifest audit before mutating anything;
- supports `--allow auto_safe` for mechanical Track Expansion;
- supports `--allow agent_design --deny product_code` for bounded
  design/planning writes when the current milestone's manifest entry includes
  an `agent_design` contract;
- allocates concrete `WR-000` ids for current `WR-TBD-*` milestones only
  through `auto_safe`;
- creates deferred WR rows with exact governance/design/planning write scope
  only through `auto_safe`;
- links the production milestone to that WR only through `auto_safe`;
- creates or updates the current milestone implementation/design plan only
  through `agent_design`;
- creates bounded-contract closeout reports and marks docs, design, or governance
  milestones completed only through `agent_closeout`;
- may revise design/contract docs only when the machine-readable manifest and
  owning WR both cover the exact paths;
- treats `agent_design` as a one-shot planning action for a milestone; once the
  manifest records that design/planning output exists, reruns fail closed until
  `agent_closeout` is explicitly allowed and evidence is valid;
- treats `agent_closeout` as a closeout-only action for docs, design, or governance
  milestones; it can close as `bounded_contract`, archive the owning WR, update
  manifest next-action metadata, regenerate reports, and run validation;
- supports `--allow product_code` only for the current implementation or
  hardening milestone after active WR and production-plan authority are exact;
- updates manifest/roadmap planning metadata and runs validation;
- stops before crate creation, runtime-proven closeout, the next milestone's
  design authoring, additional WRs, MaterialProgram work, or shared
  `foundation/meta` extraction.

`production:next`:

- runs the full manifest audit before printing normal next-action guidance;
- prints exactly one next legal action;
- explains blockers;
- refuses to skip gates;
- fails closed on alignment errors, missing gates, invalid blocked fields,
  invalid closeout paths, WR scope mismatches, or missing WR authority.

`production:audit-track`:

- checks every milestone has WR ownership or an explicit expansion blocker;
- checks dependencies, evidence gates, validation commands, closeout paths, and
  forbidden scope;
- reports whether `/goal --track <TRACK_ID>` can proceed safely.

## Drift Controls

A manifest is valid only if it makes hidden assumptions visible:

- missing WR rows are blockers, not implied future authority;
- empty evidence gates are blockers for completion, not proof gaps to ignore;
- generated docs are stale until render/check commands pass;
- a docs-only milestone cannot create code;
- a design-only milestone cannot mutate runtime behavior;
- a closeout milestone cannot implement missing behavior from prior slices;
- `runtime_proven` means runtime evidence, not documentation completeness.

If any field is unknown, write `blocked: <reason>` rather than inventing a value.

`task production:validate` audits every track with a machine-readable manifest
source. It checks manifest milestone fields, production milestone alignment, WR
ownership or future WR candidates, write-scope coverage, evidence gates,
closeout paths, permissions, and production/roadmap conflicts. Ordinary
production tracks without manifest sources keep the regular production
validation rules.

`task ai:goal -- --track <TRACK_ID>` and
`task production:next -- --track <TRACK_ID>` use the same manifest audit before
emitting normal next-action guidance. Audit-blocked manifests are stop
conditions, not advisory warnings.

## Manifest Runner V1

Manifest Runner V1 is the first automation layer above manifest audit. It is
deliberately narrow: it may apply only `auto_safe` Track Expansion, and only for
the current legal milestone when the manifest, production track, and WR state
already pass audit.

`auto_safe` may:

- allocate a concrete deferred WR id;
- add one deferred WR row;
- link one production milestone to that WR;
- replace the manifest milestone's `future_wr_candidate` with `owning_wr`;
- tighten the manifest/WR write scope to exact docs/planning paths;
- render production and roadmap generated docs;
- run validation and print the next legal action.

`auto_safe` must not:

- write design content beyond mechanical WR metadata;
- create implementation plans with architecture content;
- close WRs or production milestones;
- claim `bounded_contract`, `runtime_proven`, or closeout evidence;
- modify product/runtime code or production behavior;
- create crates or placeholder future folders;
- start Stage 6 proof slices, MaterialProgram, RenderPlan, or shared
  `foundation/meta` extraction.

## Manifest Runner V2

Manifest Runner V2 is the bounded design/planning layer. It may apply only
`agent_design` for the current docs-only or design-only milestone when the manifest
contains an `agent_design` contract and the owning WR covers the exact write
scope.

`agent_design` must stop after writing the plan/design evidence. It cannot close
WRs or production milestones, start implementation, create crates, modify
production behavior, or extract shared `foundation/meta`.

## Manifest Runner V3

Manifest Runner V3 is the bounded closeout layer for docs, design, or governance
milestones. It may apply only `agent_closeout` after design/planning evidence
exists and the manifest, production track, WR state, and validation commands
agree.

`agent_closeout` may:

- verify required plan and design evidence;
- create the expected closeout report;
- mark the production milestone completed as `bounded_contract`;
- move the owning WR from active/deferred roadmap sources to the archive as
  completed;
- update manifest next-action metadata;
- regenerate production and roadmap reports;
- run validation before reporting the next legal action.

`agent_closeout` must not:

- close runtime proof milestones as `runtime_proven`;
- modify product/runtime code or production behavior;
- create crates or placeholder future folders;
- start the next milestone's design authoring;
- start Stage 6 proof slices, MaterialProgram, RenderPlan, or shared
  `foundation/meta` extraction.

## Manifest Runner V4

Manifest Runner V4 is the product-code gate for implementation and hardening
milestones. It does not infer product edits from prose. It may proceed only when
the current manifest milestone is implementation-class, the owning WR is an
active `current_candidate`, `task production:plan` classifies the WR as
`write_implementation_contract`, and the accepted production plan states the
exact files/modules, functions/methods where possible, forbidden files/modules,
tests, validations, closeout evidence, compatibility/rollback plan, and stop
conditions.

`product_code` may:

- verify the active WR and accepted production plan;
- verify exact manifest and WR write-scope coverage;
- reject broad, wildcard, generated-without-marker, or ambiguous write scopes;
- run the milestone validation commands;
- stop after one implementation WR.

`product_code` must not:

- run for docs, design, governance, or release closeout milestones;
- run from a future WR candidate or blocked/deferred WR;
- create crates unless a future `crate_creation` layer is accepted;
- extract shared `foundation/meta`;
- start MaterialProgram;
- mark `runtime_proven` without runtime/test closeout evidence;
- continue into the next milestone without rerunning the manifest gate.

Crate creation, runtime-proof closeout mutation, and shared foundation
extraction remain out of scope until separate governance accepts future
automation layers.

## Placement

Reusable doctrine lives in this workspace document.

Machine-readable manifest sources live under:

```text
docs-site/src/content/docs/workspace/track-execution-manifests/<track-id>.yaml
```

Concrete human-readable manifest reports live under:

```text
docs-site/src/content/docs/reports/track-execution-manifests/<track-id>/manifest.md
```

Track manifests help execution, but they do not replace `production-tracks.yaml`,
WR roadmap sources, accepted designs, ADRs, implementation contracts, or
closeout evidence. The Markdown report is not parsed as execution authority.
