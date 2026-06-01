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

Full-track AI execution requires a machine-readable Track Execution Lock. The
canonical lock source path is:

```text
docs-site/src/content/docs/workspace/track-execution-locks/<track-id>.yaml
```

Historical legacy lock inputs may still exist at the path above. The clean
Track Execution Harness uses the canonical Execution Lock path:

```text
docs-site/src/content/docs/workspace/execution-locks/<track-id>.yaml
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

## Command Model

These commands are the workflow interface. `plan-track`,
`complete-track-contracts`, `next`, `audit-track`, and `run-track` are
implemented as repository tooling. `expand-track` remains a read-only candidate
listing command. `complete-track-contracts` fills missing machine-readable
action contracts from manifest templates before full-track execution so the
runner does not discover missing contracts one milestone at a time. The first
automation layer is Manifest Runner V1 can perform one permission-gated
`auto_safe` Track Expansion action and then must stop unless V2 `agent_design`
is also explicitly allowed. Manifest
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
Manifest Runner V5 is the separate bounded product implementation authoring
layer. It requires both `product_code` and `product_implementation`, writes only
the deterministic files covered by the active WR and accepted plan, requires
new files to be marked with `new:`, runs validation after mutation, and stops
before runtime closeout unless `agent_closeout` is also explicitly allowed.

```text
task production:plan-track -- --track <TRACK_ID>
task production:complete-track-contracts -- --track <TRACK_ID>
task production:expand-track -- --track <TRACK_ID>
task production:lock-track -- --track <TRACK_ID> --locked-by <IDENTITY>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --max-actions <N>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --allow agent_design --deny product_code --max-actions <N>
task production:run-track -- --track <TRACK_ID> --allow auto_safe --allow agent_design --allow agent_closeout --deny product_code --max-actions <N>
task production:run-track -- --track <TRACK_ID> --mode full-track --allow auto_safe --allow agent_design --allow agent_closeout --allow product_code --allow product_implementation --max-actions <N>
task production:run-track -- --track <TRACK_ID> --mode agent-track --allow auto_safe --allow agent_design --allow agent_closeout --allow product_code --allow product_implementation --max-actions <N>
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

The clean Track Execution Harness is the long-term execution kernel. It
compiles manifest, production, roadmap, WR, and plan authority into a typed
Execution Contract Pack under
`docs-site/src/content/docs/workspace/execution-contract-packs/<track-id>.yaml`.
Harness commands consume that Contract Pack instead of interpreting loose
manifest fields. The existing `production:*` manifest commands remain the
public compatibility surface, but manifest-backed executable tracks with a
valid Execution Contract Pack delegate locked full-track execution, full-track
audit, and next-action inspection to the harness. Unsupported or unpacked
tracks report the legacy fallback path explicitly. Executable tracks do not
fall back to the legacy Manifest Runner when their Contract Pack or Execution
Lock is missing or stale.

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

- loads the machine-readable Track Execution Manifest and audits production,
  roadmap, and WR state;
- inspects all remaining milestones;
- generates missing `auto_safe_contract`, `agent_design_contract`,
  `product_code_contract`, `agent_closeout_contract`,
  `runtime_closeout_contract`, and handoff contract blocks from manifest
  templates and milestone parameters;
- updates the machine-readable manifest and human-readable manifest report;
- fails closed when exact product-code scope, template data, or evidence
  categories cannot be generated safely;
- runs validation;
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

- runs the full manifest audit before mutating anything;
- has explicit modes: `single-step`, `bounded-segment`, `full-track`, and
  `agent-track`;
- requires `--mode full-track` for full-track permission sets with more than
  one action unless `--mode agent-track` is explicitly selected;
- requires a current Track Execution Lock before any `--mode full-track`
  mutation;
- runs full automation readiness preflight before any `--mode full-track`
  mutation;
- supports `--preflight-only` to inspect every remaining milestone from the
  current milestone to track completion without mutating production, roadmap,
  manifest, docs, or product files;
- refuses full-track execution when any remaining milestone lacks the required
  machine-readable action contracts;
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
- appends a Track Execution Run ledger entry after every successful
  `full-track` or `agent-track` action.

`--mode agent-track` is the preparation-and-execution orchestration mode. It is
not a looser full-track path. It may create/link WRs, create plans, write
bounded design contracts, close governance/design milestones, run preflight,
create or refresh the execution lock after preflight passes, and then continue
to implementation only when the active manifest, WR, plan, writer strategy,
validation commands, and permissions are exact. It recomputes production,
roadmap, manifest, and WR state after every action and stops on failed
validation, missing evidence, ambiguous scope, ungranted permission, stale
digest, strategic human gate, or max actions.

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
- create crates unless `crate_creation` is explicitly granted and the active
  manifest, WR, and plan name exact `new: <crate>/Cargo.toml` crate paths and
  validation commands;
- extract shared `foundation/meta`;
- start MaterialProgram;
- mark `runtime_proven` without runtime/test closeout evidence;
- continue into the next milestone without rerunning the manifest gate.

Runtime-proof closeout mutation is allowed only through the runtime closeout
contract. Shared foundation extraction remains out of scope until separate
governance accepts an extraction gate.

## Manifest Runner V5

Manifest Runner V5 writes bounded product/runtime implementation files only
after V4 succeeds and `product_implementation` is explicitly granted. It is not
a general code generator and it must not contain proof-slice-specific writer
logic. The active manifest milestone and accepted implementation plan declare
the writer strategy, exact files, required outputs, forbidden files/patterns,
validation commands, runtime evidence requirements, and expected closeout path.

Supported `implementation_writer.strategy` values are:

- `no_writer`: the default fail-closed strategy.
- `template_writer`: writes declared template contents to declared files only.
- `patch_writer`: applies declared bounded text replacements to declared files.
- `agent_writer`: runs `codex exec` inside an isolated temporary action
  workspace with a prompt generated from the active WR, plan, manifest entry,
  allowed scopes, forbidden scopes, validation commands, required outputs, and
  stop conditions. The runner captures the resulting diff, rejects undeclared
  files, rejects forbidden paths/patterns, rejects undeclared new files, checks
  target-file digests before import, imports only accepted files, records the
  transcript as a non-doc run artifact referenced by the YAML run ledger, runs
  validation, and stops after the current implementation WR by default.
- `proof_aggregation_writer`: aggregation-only strategy for proof milestones
  that validate prior `runtime_proven` closeouts and evidence categories before
  allowing closeout. It must not repair missing prior behavior or patch earlier
  proof-slice product files.

Implementation milestones may declare:

- `implementation_writer.strategy`
- `implementation_writer.allowed_files`
- `implementation_writer.allowed_write_scopes`
- `implementation_writer.aggregation_only`
- `implementation_writer.required_prior_milestones`
- `implementation_writer.required_prior_completion_quality`
- `implementation_writer.required_evidence_categories`
- `implementation_writer.required_outputs`
- `implementation_writer.forbidden_files`
- `implementation_writer.forbidden_scopes`
- `implementation_writer.forbidden_patterns`
- `implementation_writer.new_file_policy`
- `implementation_writer.validation_commands`
- `implementation_writer.closeout_path`
- `implementation_writer.stop_conditions`
- `implementation_writer.templates` for `template_writer`
- `implementation_writer.patches` for `patch_writer`
- `implementation_writer.agent_prompt`
- `implementation_writer.agent_context_files`
- `implementation_writer.agent_required_outputs`
- `implementation_writer.agent_diff_protocol_version`
- `implementation_writer.agent_worktree_policy`

`product_implementation` may:

- create an exact new file only when the owning WR or product contract marks it
  with `new:`. New-file authority is determined from git tracked/index state,
  not just filesystem existence; an untracked pre-existing file is still a new
  file for workflow authority purposes.
- update exact existing files already covered by the active WR and manifest;
- run the active milestone validation commands after writing;
- stop after one implementation WR by default.

`product_implementation` must not:

- run without `product_code`;
- create placeholder folders;
- touch broad or forbidden scopes;
- create crates unless `crate_creation` is granted and exact crate paths are
  pre-authorized;
- extract shared `foundation/meta`;
- start MaterialProgram.

`agent_design_contract.authoring_strategy: codex_contract_writer` uses the
same isolated workspace and scoped-diff protocol for design/contract authoring.
It is for PM-002-style execution contract packs where AI may author bounded
design outputs or manifest contract edits before product code runs. Template
contract generation remains the default for ordinary implementation plans.

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
