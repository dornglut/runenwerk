---
title: Track Execution Manifest
description: Planning-stage contract that makes full production-track goal execution explicit, gated, and drift-resistant before slice implementation begins.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-01
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

## Execution Lock

Full-track AI execution requires a machine-readable Execution Lock. The clean
Track Execution Harness uses the canonical lock source path:

```text
docs-site/src/content/docs/workspace/execution-locks/<track-id>.yaml
```

Historical legacy lock inputs may still exist under:

```text
docs-site/src/content/docs/workspace/track-execution-locks/<track-id>.yaml
```

The lock is generic and digest-based. It records `track_id`,
`ai_executable`, `locked_by`, `locked_at`, the manifest digest, source digests
for production, roadmap, accepted designs, manifest inputs, and workflow runner
sources, granted permissions, denied permissions, strategic human gates, and
invalidation rules.

`--mode full-track` must refuse to mutate unless the lock exists, the manifest
declares `ai_executable: true`, the lock declares `ai_executable: true`, all
digests match current sources, requested permissions are granted, denied
permissions are not requested, and no remaining milestone crosses a locked
strategic human gate. Locks authorize execution only while their source digests
remain current. They do not authorize implementation by themselves.

Strategic human gates include accepting new ADRs, changing locked design
direction, foundation/meta extraction, crate creation unless pre-authorized,
external plugin or security boundaries, second-domain extraction, and
`perfectionist_verified` release certification.

## Run Ledger

Full-track runs write machine-readable run ledgers under:

```text
docs-site/src/content/docs/reports/track-execution-runs/<track-id>/<run-id>.yaml
```

Each successful action appends pre-action digests, post-action digests,
milestone id, WR id, strategy used, files changed, validation results,
evidence paths, closeout paths, next legal action, and stop reason. Generated
Markdown reports may mirror these ledgers later, but the YAML ledger is the run
authority.

## Manifest Content

A manifest must include these fields for the track:

- track id;
- authority level;
- optional `ai_executable` declaration for locked full-track execution;
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

## Generic Strategies And Evidence

Manifest execution is strategy-driven. Runner code must dispatch by
manifest-declared strategy names, not by production-track, milestone, or WR id.

Writer strategies:

- `no_writer`: default, fail closed.
- `template_writer`: writes only declared template outputs inside exact scope.
- `patch_writer`: applies bounded edits to declared existing files; new files
  still require explicit `new:` scope.
- `proof_aggregation_writer`: aggregates prior closeout evidence records and
  must not patch prior milestone product files.
- `agent_writer`: uses the scoped-diff protocol. It runs only inside an
  isolated action workspace, may touch only declared output files, and imports
  only accepted diffs after scope, forbidden-pattern, new-file, validation, and
  digest checks pass.

Closeout strategies:

- `bounded_contract_closeout`;
- `runtime_proven_closeout`;
- `handoff_closeout`;
- `extraction_gate_closeout`.

Runtime closeouts must include machine-readable `closeout_evidence`
frontmatter or equivalent sidecar metadata. Reusable evidence categories are
`runtime_test`, `fixture`, `diagnostics`, `source_maps`, `artifact`,
`migration`, `reproducibility`, `visual`, and `handoff`. Proof aggregation reads
these records, not prose. Missing evidence returns to the owning milestone.

`plan.contract.yaml` evidence entries distinguish the evidence record output
from the proof subject:

- `paths` names the machine-readable evidence record file that the executor may
  create.
- `validation_command_ids` is required for `runtime_test` evidence and must
  match typed validation commands that actually ran.
- `subject_paths` is required for non-runtime evidence and must list exact
  repository files that already exist inside the isolated action workspace.

Closeout and truth-claim checks must reject non-runtime evidence records that do
not carry `subject_paths`. A generated evidence record that only points back to
itself is not proof.

## Command Model

These commands are the workflow interface. `plan-track`,
`complete-track-contracts`, `next`, `audit-track`, and `run-track` are
implemented as repository tooling. `expand-track` remains a read-only candidate
listing command. `complete-track-contracts` compiles the typed Execution
Contract Pack from machine-readable production, roadmap, manifest, WR, and
implementation-plan sidecar authority. It does not fill missing contracts from
prose and it does not create implementation authority. Missing structured
contracts must be authored explicitly before the pack can compile.

```text
task production:plan-track -- --track <TRACK_ID>
task production:complete-track-contracts -- --track <TRACK_ID>
task production:expand-track -- --track <TRACK_ID>
task production:lock-track -- --track <TRACK_ID> --locked-by <IDENTITY>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --max-actions <N>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --allow agent_design --deny product_code --max-actions <N>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --allow agent_design --allow agent_closeout --deny product_code --max-actions <N>
task production:run-track -- --track <TRACK_ID> --mode full-track --allow auto_safe --allow agent_design --allow agent_closeout --allow product_code --allow product_implementation --max-actions <N>
task production:run-track -- --track <TRACK_ID> --preflight-only --allow <PERMISSION>
task production:next -- --track <TRACK_ID>
task production:audit-track -- --track <TRACK_ID>
task production:audit-track -- --track <TRACK_ID> --full-automation
task production:audit-track -- --track <TRACK_ID> --full-automation --require-lock
task execution:compile -- --track <TRACK_ID>
task execution:preflight -- --track <TRACK_ID>
task execution:lock -- --track <TRACK_ID> --locked-by <IDENTITY>
task execution:next -- --track <TRACK_ID>
task execution:run -- --track <TRACK_ID> --mode full-track
```

The clean Track Execution Harness is the long-term execution kernel. Manifest
YAML is loaded through the source-model layer under `tools/workflow/track_sources/`;
the execution kernel must not import or execute the historical monolithic
manifest runner. The harness compiles manifest, production, roadmap, WR, and
plan authority into a typed Execution Contract Pack under
`docs-site/src/content/docs/workspace/execution-contract-packs/<track-id>.yaml`.
Harness commands consume that Contract Pack instead of interpreting loose
manifest fields. The existing `production:*` commands are public adapters over
the source model and execution kernel. Manifest-backed executable tracks with a
valid Execution Contract Pack delegate locked full-track execution, full-track
audit, and next-action inspection to the harness. Executable tracks do not fall
back to historical loose-manifest execution when their Contract Pack or
Execution Lock is missing or stale.

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

`production:complete-track-contracts`:

- compiles the typed Execution Contract Pack from reviewed machine-readable
  sources;
- requires executable implementation authority to live in plan sidecars named
  `plan.contract.yaml` beside the human-readable plan;
- fails closed when WR, manifest, sidecar, validation, evidence, or closeout
  contracts are missing;
- does not parse implementation-plan prose as authority;
- does not authorize implementation, crate creation, MaterialProgram work, or
  shared `foundation/meta` extraction.

`production:lock-track`:

- audits the manifest, production, roadmap, WR, and full automation contracts;
- writes a digest-locked AI execution lock;
- records granted permissions, denied permissions, strategic human gates, and
  source digests;
- does not authorize implementation without the active WR, plan, writer,
  validation, and closeout gates.

`production:run-track`:

- is the public adapter for locked execution;
- supports `--mode full-track` through the clean Track Execution Harness;
- requires a fresh Execution Contract Pack before full-track guidance or
  mutation;
- requires a current Execution Lock before any full-track mutation;
- runs clean-kernel preflight before mutation;
- executes one compiled `ActionContract` at a time;
- imports only declared outputs after validation and digest checks;
- writes resolver-backed evidence and a run ledger for successful actions;
- fails closed instead of falling back to loose manifest interpretation.

`single-step` and `bounded-segment` are reserved compatibility mode names in the
adapter. They are not runnable mutation paths for executable tracks in the
clean kernel. Legacy V1-V5 manifest-runner mutation remains historical
compatibility only and is not the public authority for locked tracks.

`--mode agent-track` is the clean-kernel preparation-and-execution path for
manifest-declared AI-executable or full-automation tracks. It may prepare a
fresh Contract Pack, run full preflight, create or refresh a missing/stale
Execution Lock when permissions are not denied, and then delegate to the same
scoped workspace/import, resolver-backed evidence, validation, ledger, and
stop-condition guarantees as locked full-track execution.

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
- supports `--full-automation` to validate every remaining milestone's
  automation class, required permission classes, action contracts, exact writer
  scopes, implementation writer strategy, runtime closeout contract, final
  handoff contract, evidence categories, and dependency declarations before a
  broad `run-track` command is attempted;
- supports `--require-lock` to verify the Track Execution Lock is current and
  grants the requested locked-track authority;
- reports whether `/goal --track <TRACK_ID>` can proceed safely.

Full automation preflight uses the manifest `execution_kind`, not production
`kind`, as the automation authority. Valid execution kinds are
`design_contract`, `implementation_proof`, `proof_aggregation`,
`handoff_closeout`, and `extraction_gate`. Legacy production kinds such as
`hardening` and `release` remain production taxonomy only; they are invalid as
automation execution kinds.

## Drift Controls

A manifest is valid only if it makes hidden assumptions visible:

- missing WR rows are blockers, not implied future authority;
- empty evidence gates are blockers for completion, not proof gaps to ignore;
- `truth_claims` must distinguish product behavior, proof slices,
  architecture contracts, handoffs, and extraction gates;
- generated docs are stale until render/check commands pass;
- a docs-only milestone cannot create code;
- a design-only milestone cannot mutate runtime behavior;
- a closeout milestone cannot implement missing behavior from prior slices;
- `runtime_proven` means runtime evidence, not documentation completeness.

If any field is unknown, write `blocked: <reason>` rather than inventing a value.

`task production:validate` audits every track with a machine-readable manifest
source. It checks manifest milestone fields, production milestone alignment, WR
ownership or future WR candidates, write-scope coverage, evidence gates,
closeout paths, permissions, declared design outputs, truth claims, and
production/roadmap conflicts. Ordinary
production tracks without manifest sources keep the regular production
validation rules.

`task ai:goal -- --track <TRACK_ID>` and
`task production:next -- --track <TRACK_ID>` use the same manifest audit before
emitting normal next-action guidance. Audit-blocked manifests are stop
conditions, not advisory warnings.

## Truth Claims

Manifest-backed tracks must declare machine-readable `truth_claims`.

Each truth claim records:

- `claim_id`
- `claim_kind`: `product_behavior`, `architecture_contract`, `proof_slice`,
  `handoff`, or `extraction_gate`
- `claim_level`: `bounded_contract`, `runtime_proven`,
  `proof_slice_runtime_proven`, `architecture_runtime_proven`, or
  `perfectionist_verified`
- `claim_status`: `satisfied`, `blocked`, or `superseded`
- evidence resolvers for required docs, code contracts, validation commands,
  and closeout evidence categories
- `known_gaps`, `supersedes`, and `blocks_downstream`

Supported evidence resolvers are:

- `doc_exists`
- `doc_frontmatter_status`
- `rust_symbol_exists`
- `module_path_exists`
- `validation_command`
- `closeout_evidence_category`

Satisfied claims must have resolvable evidence. Blocked claims must list known
gaps. A downstream handoff may not become ready while a truth claim blocks it.

Use `proof_slice_runtime_proven` when bounded runtime/test proof slices passed
but the final architecture is not implemented. Use
`architecture_runtime_proven` only when concrete docs, code contracts, and
executable validations prove the architecture itself.

## Closeout Evidence Metadata

Runtime closeouts that feed automation must include a machine-readable
`closeout_evidence` record in frontmatter. Prose closeouts remain useful for
review, but proof aggregation must not scrape prose as execution authority.

Minimum fields:

- `milestone_id`
- `wr_id`
- `completion_quality`
- `evidence_categories`
- `validation_commands`
- `validation_results`
- `files_changed`
- `runtime_artifacts`
- `diagnostics`
- `source_maps`
- `known_gaps`
- `closeout_path`

`proof_aggregation_writer` reads these records from required prior closeouts.
It fails closed when a prior closeout is missing metadata, claims the wrong
milestone or WR, lacks the required completion quality, or the aggregate prior
records do not cover the required evidence categories. Missing proof evidence
returns to the owning earlier milestone; aggregation must not patch earlier
product files to make evidence pass.

## Legacy Manifest Runner Compatibility

The historical Manifest Runner V1-V5 layers remain in the repository only for
legacy compatibility tests and unsupported transition paths. They are not the
authority for locked/executable tracks.

New executable-track authority is the clean kernel:

```text
production / roadmap / manifest / plan.contract.yaml
-> track_sources source model
-> Execution Contract Pack
-> Execution Lock
-> transactional executor
-> resolver-backed evidence
-> run ledger
-> closeout / truth claims
```

For executable tracks, public `production:*` commands must use the source model
and execution kernel. They must not invoke the monolithic legacy runner to
interpret loose manifest fields, infer implementation authority from prose, or
claim runtime evidence from validation success alone.

Legacy concepts map to clean-kernel concepts as follows:

- V1 `auto_safe` -> `planning_expansion` action;
- V2 design/planning -> `design_authoring` action with structured sidecar
  authority;
- V3 closeout -> typed closeout actions with declared evidence requirements;
- V4 product-code gate -> Contract Pack compilation and preflight;
- V5 implementation write -> transactional `product_implementation` action.

Future agent-style preparation must be added as clean-kernel execution, not by
reviving legacy `agent-track` behavior.

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
