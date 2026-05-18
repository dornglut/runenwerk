---
title: Planning and Implementation Workflow
description: Task-shape guide for planning, implementation, routines, prompt templates, validation, and closeout in Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-17
related_docs:
  - ./agents.md
  - ./architecture-governance-review.md
  - ./parallel-roadmap-batch-automation.md
  - ./documentation-structure.md
  - ./routines/README.md
  - ./routines/architecture-governance-review-routine.md
  - ./routines/parallel-roadmap-batch-routine.md
  - ./prompt-templates/README.md
  - ./prompt-templates/architecture-governance-review.md
  - ./prompt-templates/parallel-roadmap-batch.md
  - ../guidelines/architecture.md
---

# Planning and Implementation Workflow

Use this guide to choose the right shape for AI-assisted repository work before code or documentation changes start.

This guide routes existing rules. It does not override `AGENTS.md`, `AI_GUIDE.md`, architecture doctrine, dependency rules, domain ownership, routines, or prompt templates.

## Automation

Use the workflow kickoff helper when starting a new AI-assisted task:

```sh
task --list
task batch:kickoff -- --next
task roadmap:intake -- --idea "<design or change idea>"
task production:plan -- --milestone "<PM-ID>" --roadmap "<WR-ID>"
task production:validate
task production:check
task ai:architecture-governance -- --task "<decision>" --scope "<crate/files/subsystem>"
task ai:parallel-roadmap-batch -- --task "<batch goal>" --scope "<roadmap rows or docs>"
task ai:implementation -- --task "<task>" --scope "<crate/files/subsystem>"
task ai:closeout -- --task "<completed phase>" --roadmap "<owning roadmap/design path>"
task roadmap:validate
task roadmap:check
task planning:validate
task puml:validate
task batch:propose -- --goal "<batch goal>" --scope "L0"
task docs:validate
task ci:local
```

Task-shape commands such as `architecture-governance`,
`parallel-roadmap-batch`, `implementation`, `milestone`, and `closeout` are
non-mutating. They print the relevant docs, first inspection commands, a
ready-to-use prompt, validation expectations, and stop conditions for the chosen
task shape.

Validation commands execute checks:

- `task docs:validate` runs the repository docs validation.
- `task ci:local` runs the full local host validation pipeline.
- `task roadmap:validate` checks `roadmap-items.yaml` score math,
  dependencies, gates, write-scope overlap, completion evidence, and
  completion-quality claims.
- `task roadmap:check` rejects stale generated roadmap Markdown
  and PUML.
- `task production:validate` checks `production-tracks.yaml` structure,
  milestone dependencies, design gates, evidence gates, WR links, and
  production completion-quality claims.
- `task production:check` rejects stale generated production Markdown,
  PUML, and JSON Schema.
- `task production:plan -- --milestone <PM-ID> --roadmap <WR-ID>` prints a
  readiness report and reusable implementation-contract prompt for one
  production milestone and WR row.
- `task planning:validate` runs the roadmap, production, and docs planning
  gates together.
- `task puml:validate` validates workspace PlantUML diagrams with PlantUML.

There is no repository-managed remote CI. Run `task ci:local` explicitly before
push, PR, integration, and batch closeout when a broad gate is required.

The lower-level prompt generator remains available at `uv run python tools/workflow/ai_task.py`, but the stable repo entrypoint is `task`.

The historical `tools/docs/add_agent_workflow_docs.sh` entrypoint is also non-mutating. It exists only for compatibility and validates docs instead of regenerating workflow pages.

For new Codex threads, prefer one-line command prompts:

```text
Run task batch:kickoff -- --next and follow the generated workflow.
```

```text
Run task roadmap:intake -- --idea "<design/change idea>" and prepare it for roadmap review.
```

```text
Run task planning:validate before changing production or roadmap planning state.
```

```text
Run task production:plan -- --milestone PM-SDF-OW-001 --roadmap WR-019 before writing a substantial production implementation contract.
```

`batch:kickoff` creates the proposed batch from current `planning_state=current_candidate`
items and prints the exact approve, prepare, validate, worker prompt, scope-check,
and closeout commands. It does not approve implementation unless `--approve` is
explicitly passed.

`roadmap:intake` creates a review proposal for new ideas. It does not edit
`roadmap-items.yaml`; accepted proposals are applied with
`task roadmap:apply-intake -- --proposal <proposal.yaml>`, then rendered and
validated.

Production tracks are the long-term sequencing layer. Review
`production-tracks.yaml` before choosing roadmap rows for broad product work.
If the relevant production milestone is `designing`, do the design or ADR work
first. If it is `ready_next` or `active`, use its WR links to inspect the legal
roadmap execution rows. The production track guides direction; the WR roadmap
still governs implementation eligibility.
Use `task production:plan -- --milestone <PM-ID> --roadmap <WR-ID>` as the
normal bridge from production intent to a durable implementation contract. The
command is read-only unless `--write-scaffold` is explicitly passed.

Codex CLI `/goal` is execution persistence, not roadmap authority. Use it only
after there is a written execution contract from a plan, batch, accepted design,
or approved roadmap row. A `/goal` run must not directly promote or complete
roadmap items unless architecture governance has run when the change is
architecture-sensitive, bounded implementation or batch validation has passed,
closeout or drift-check evidence exists, and `task roadmap:render`,
`task roadmap:validate`, and `task roadmap:check` pass. Completed roadmap rows
must reference an existing completed closeout or finalized batch evidence path,
and that path must be included in the row's `write_scopes`. They must also set
`completion_quality`, list any `known_quality_gaps`, and only claim
`perfectionist_verified` when a completed audit path exists and the gap list is
empty.

Use workflow commands to automate Codex prompt, checklist, and gate setup, not
blind mutation. A typical architecture-sensitive flow is:

```sh
task ai:architecture-governance -- --task "<decision>" --scope "<scope>"
task ai:planning -- --task "<change>" --scope "<scope>"
task ai:implementation -- --task "<bounded implementation>" --scope "<scope>"
task ai:closeout -- --task "<completed phase>" --roadmap "<roadmap>"
```

A typical parallel roadmap batch flow is:

```sh
task batch:kickoff -- --next
task roadmap:validate
task batch:propose -- --goal "<batch goal>" --scope L0 --out docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
task batch:approve -- --batch docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
task batch:prepare -- --batch docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
task batch:scope-check -- --batch docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
task batch:continue -- --batch docs-site/src/content/docs/reports/batches/<date>-<slug>/batch.toml
```

The coordinator proposes candidate rows, worker scopes, validations, and
closeout docs first. Implementation starts only after explicit user approval.
Use `batch:continue` only after finalization, when integrated items remain
current and should become the next proposal. It writes a proposed manifest; it
does not approve or start implementation.

## Purpose

Runenwerk work usually falls into one of four shapes:

1. investigation;
2. planning or design;
3. bounded implementation;
4. closeout and drift repair.

Choose the shape explicitly so AI-assisted work does not drift from a small answer into a broad refactor, from a plan into unapproved implementation, or from implementation into undocumented architecture.

For long-term product sequencing, first identify the production track and
milestone in `production-tracks.yaml`. Then decide whether the next step is
design work, WR roadmap intake/promotion, bounded implementation, or closeout.

## Start Every Task

Before changing files:

1. Read `AGENTS.md`.
2. Read the root entrypoints referenced by `AGENTS.md` when the task may touch code, architecture, dependencies, docs placement, or validation.
3. Check current workspace state with `git status --short`.
4. Locate the owning area by using `DOMAIN_MAP.md`, current docs, and `rg`.
5. Inspect current code and tests before designing or editing.
6. Identify the smallest validation command that can prove the change.

Preserve unrelated dirty work. If a dirty file is relevant, inspect its current diff and work with it instead of overwriting it.

## Task Shape Decision

| Task shape | Use when | Primary docs |
|---|---|---|
| Investigation | The goal is to understand current state, find gaps, or compare options without editing. | `AGENTS.md`, owning docs, current code/tests |
| Planning or design | The goal is to decide architecture, ownership, API shape, phase sequence, or whether a refactor is justified. | `docs-site/src/content/docs/design/`, `docs-site/src/content/docs/workspace/documentation-structure.md`, prompt templates |
| Production track planning | The goal is to choose or update long-term product outcomes before selecting WR execution rows. | `docs-site/src/content/docs/workspace/production-track-planning-model.md`, `docs-site/src/content/docs/workspace/production-tracks.yaml` |
| Production implementation contract | The goal is to turn one production milestone and WR row into a reviewed work package before code changes. | `docs-site/src/content/docs/workspace/prompt-templates/production-implementation-contract.md`, `task production:plan -- --milestone <PM-ID> --roadmap <WR-ID>` |
| Bounded implementation | The goal is to make a scoped code/docs change and verify it. | `docs-site/src/content/docs/workspace/prompt-templates/implementation-batch.md`, relevant routine docs |
| Roadmap milestone | The goal is to implement a named phase from an accepted roadmap or design. | `docs-site/src/content/docs/workspace/prompt-templates/roadmap-milestone-kickoff.md`, owning roadmap/design |
| Code refactor | The goal is behavior-preserving cleanup, boundary repair, naming, or API ergonomics. | `docs-site/src/content/docs/workspace/routines/code-refactor-routine.md` |
| New crate or major crate phase | The goal is a crate-level implementation from an accepted boundary. | `docs-site/src/content/docs/workspace/routines/crate-implementation-routine.md` |
| Documentation refactor | The goal is moving, renaming, pruning, or restructuring docs. | `docs-site/src/content/docs/workspace/routines/docs-refactor-routine.md` |
| Public API review | The goal is to review usability, discoverability, examples, and public entrypoints. | `docs-site/src/content/docs/workspace/routines/public-api-review-routine.md` |
| Architecture governance review | The goal is to check dependency direction, domain ownership, ADR need, tradeoffs, migration shape, enforcement, or ownership mode. | `docs-site/src/content/docs/workspace/architecture-governance-review.md`, `docs-site/src/content/docs/workspace/routines/architecture-governance-review-routine.md` |
| Parallel roadmap batch | The goal is to propose, approve, fan out, integrate, and close out independent roadmap slices. | `docs-site/src/content/docs/workspace/parallel-roadmap-batch-automation.md`, `docs-site/src/content/docs/workspace/routines/parallel-roadmap-batch-routine.md` |
| Phase closeout | A phased implementation just completed. | `docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md` |
| Commit organization | The working tree has mixed changes that need coherent commits. | `docs-site/src/content/docs/workspace/routines/commit-splitting-routine.md` |

## Planning Pass

Use a planning pass when scope, ownership, phase boundaries, or validation are not obvious.

A useful plan names:

- owning domain, crate, subsystem, and module;
- exact files or modules likely to change;
- relevant existing APIs, helpers, tests, and docs;
- invariants the work must preserve;
- expected public API or usage impact;
- validation commands;
- stop conditions;
- deferred work that must not be implemented yet.

Do not create a plan-only response when the task clearly asks for implementation and the scope is safe. Use the plan internally, then implement.

## Decision-Complete Planning Gate

For architecture-sensitive product work, planning is not a lightweight preface.
It is the place where implementation and design decisions must be made before
code changes start. Use this gate for production milestones, roadmap rows with
cross-domain write scopes, renderer/runtime handoff work, persistence or
migration changes, new crates, source-of-truth changes, and any task that has
already shown drift in prior implementation attempts.

The planning pass must be decision-complete before implementation starts:

- name the source of truth and every derived projection or product;
- define the full runtime chain, not only the nearest adapter or descriptor;
- name exact owning domains, crates, modules, and files;
- define public APIs, typed data contracts, persistence shape, diagnostics, and
  migration behavior;
- state what must fail closed and what prior-valid state must preserve;
- define anti-drift architecture guards that fail if placeholders, fallbacks, or
  local shortcuts return;
- include end-to-end proof requirements, including GPU/pixel evidence when the
  user-visible outcome is visual;
- identify design, ADR, roadmap, production, and write-scope updates required
  before implementation;
- list explicit non-goals and later slices so they cannot be smuggled into the
  implementation or used as hidden completion blockers.

After drafting the plan, run a critical review pass before implementation. The
review should attack the plan from the repository's long-term architecture:
dependency direction, domain ownership, source truth, runtime consumption,
failure modes, test evidence, closeout evidence, and user-visible completeness.
If the review finds unresolved choices, implementation does not start. Update
the plan until there are no ambiguous ownership, API, persistence, renderer,
diagnostic, validation, or closeout decisions left.

For production work, the implementation contract should be treated as the
accepted decision record for that slice. Product code starts only after the
contract names the long-term solution and the critical review has either found
no blocking gaps or those gaps have been folded back into the contract.

## Design Versus Roadmap Versus Routine

Use the right document type:

- A design doc explains target architecture, ownership boundaries, invariants, dependency constraints, tradeoffs, and migration shape.
- A roadmap explains implementation order and phase completion criteria.
- A routine explains repeatable execution steps with validation and stop conditions.
- A prompt template gives a reusable starting prompt for a human or AI agent.
- A closeout report records evidence after work is complete.

If the work creates a long-term architecture rule, add or update an ADR or guideline instead of burying the decision in an implementation plan.

If the work changes dependency direction, domain ownership, migration strategy,
or quality-attribute tradeoffs, run the architecture governance review before
implementation. Use Clean Architecture dependency direction, DDD ownership,
ADRs, fitness functions, ATAM-lite, Strangler Fig migration, and Team
Topologies labels only where they change the decision.

## Implementation Pass

For bounded implementation:

1. Confirm the task owner and scope.
2. Inspect nearby code for established patterns.
3. Implement the smallest coherent change.
4. Keep domain logic in the owning domain.
5. Keep runtime/app glue out of foundation and pure domain crates.
6. Add tests for changed invariants, command behavior, projections, ratifiers, or public APIs.
7. Update docs when public behavior, architecture, usage, routines, validation, or roadmap state changes.
8. Run focused validation first.
9. Escalate to broader validation when dependencies, workspace behavior, or cross-domain contracts changed.

Use subagents or parallel workers only when the task prompt and current environment explicitly allow them, and only for bounded exploration or disjoint write scopes.

Use the parallel roadmap batch routine when the user wants Codex to inspect
current roadmap work, propose concurrent tasks, wait for approval, then fan out
subagents or worktrees and integrate the result.

## Closeout Pass

Every implementation closeout should report:

- what changed;
- exact files and functions/modules changed;
- why the change belongs in that owner;
- validation commands and results;
- skipped validation with reasons;
- remaining risks, blockers, or deferred work.

## Perfectionist Closeout Audit

Before changing a WR row or production milestone to `completed`, run a
perfectionist closeout audit. This audit does not mean every row must become
`perfectionist_verified`; it means the completion claim must be honest.

The audit must classify the row or milestone:

- `bounded_contract` when the accepted bounded contract is complete, but the
  long-term product still has known deferred scope or quality gaps;
- `runtime_proven` when the accepted product chain has runtime or GPU evidence,
  but architecture, UI, module-structure, or future product gaps remain;
- `perfectionist_verified` only when a completed audit path exists and
  `known_quality_gaps` is empty.

The audit must explicitly check:

- whether the implementation proves the full source-to-runtime chain instead
  of only descriptors, prepared data, status text, or metadata;
- whether renderer-visible work has GPU/pixel evidence when product-visible
  correctness depends on pixels;
- whether UI work is a real product surface or only a typed/status projection;
- whether modules follow the repository module-structure guidelines and do not
  hide mixed responsibilities in large catch-all files;
- whether fallback, pseudo, migration-only, or test-only paths are named
  honestly and cannot be mistaken for production evidence;
- whether any remaining quality gaps are recorded as `known_quality_gaps` and,
  when appropriate, as explicit follow-up roadmap work.

Completed production milestones inherit the weakest linked WR quality. A
production milestone may only claim `perfectionist_verified` when all linked WR
rows are also `perfectionist_verified` and the milestone has its own completed
audit path.

After any completed phased implementation, run the phase completion drift-check routine before starting the next phase.

Use `./quiet_full_gate.sh` when the change is broad enough that full validation is appropriate.

## Stop Conditions

Stop and report instead of continuing when:

- the owning domain or crate is unclear;
- the required dependency direction is forbidden;
- the task requires an architectural decision that has not been accepted;
- implementation would silently change public behavior outside the requested scope;
- validation fails for unrelated dirty-worktree reasons;
- the task expands into later phases or unrelated domains.

## Useful Prompt Starting Points

Use these existing templates instead of writing new one-off instructions:

- `docs-site/src/content/docs/workspace/prompt-templates/architecture-governance-review.md` for pre-implementation architecture decision gates.
- `docs-site/src/content/docs/workspace/prompt-templates/architecture-audit.md` for findings-only architecture audits.
- `docs-site/src/content/docs/workspace/prompt-templates/parallel-roadmap-batch.md` for approved parallel roadmap fan-out.
- `docs-site/src/content/docs/workspace/prompt-templates/crate-design.md` for crate boundary design.
- `docs-site/src/content/docs/workspace/prompt-templates/implementation-batch.md` for bounded implementation.
- `docs-site/src/content/docs/workspace/prompt-templates/roadmap-milestone-kickoff.md` for named roadmap milestones.
- `docs-site/src/content/docs/workspace/prompt-templates/phase-completion-drift-check.md` for phase closeout checks.
- `docs-site/src/content/docs/workspace/prompt-templates/code-review.md` for review-only work.

Create a new prompt template only when the prompt shape is expected to be reused.
