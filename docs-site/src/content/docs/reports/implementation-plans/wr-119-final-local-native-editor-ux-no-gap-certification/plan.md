---
title: WR-119 Final Local Native Editor UX No Gap Certification Design Contract
description: Design-first contract for PM-EDITOR-UX-009 final local-native editor UX certification, zero known gaps, and completion-quality evidence.
status: active
owner: editor
layer: app / domain/editor / domain/ui / workspace
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md
related_reports:
  - ../../closeouts/pm-editor-ux-002-native-editor-ux-story-lab-and-evidence-harness/closeout.md
  - ../../closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md
  - ../../closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md
  - ../../closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md
  - ../../closeouts/pm-editor-ux-006-shell-and-product-pattern-polish/closeout.md
  - ../../closeouts/pm-editor-ux-007-all-registered-visible-surface-wave/closeout.md
  - ../../closeouts/pm-editor-ux-008-game-ui-readiness-seam/closeout.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-deferred.yaml
---

# WR-119 Final Local Native Editor UX No Gap Certification Design Contract

## Goal

Clear the design-first blocker for `PM-EDITOR-UX-009` and prepare `WR-119` for
promotion planning. This action is planning and metadata only. It does not
change product code and does not claim final certification.

This contract is produced from:

```text
task production:plan -- --milestone PM-EDITOR-UX-009 --roadmap WR-119
```

Expected production outcome for the later final slice:

- every `PT-EDITOR-UX` prerequisite closeout from PM002 through PM008 remains
  completed, valid, and internally linked;
- local-native evidence is present where supported and platform-impossible
  reports are explicit where native capture is unavailable;
- retained manifests, visible-widget scans, focus traversal, accessibility,
  interaction, diagnostics, timing, performance, surface readiness, provider,
  and game-readiness seam evidence agree;
- `./quiet_full_gate.sh` and documentation/planning gates pass;
- `PM-EDITOR-UX-009` and `PT-EDITOR-UX` may claim `perfectionist_verified`
  only when known quality gaps are empty.

## Source Of Truth

- Production milestone: `PM-EDITOR-UX-009` in
  `docs-site/src/content/docs/workspace/production-tracks.yaml`.
- Roadmap row: `WR-119` in
  `docs-site/src/content/docs/workspace/roadmap-items.yaml` after this design
  action is accepted.
- Active editor UX doctrine:
  `docs-site/src/content/docs/design/active/editor-product-ux-story-lab-and-game-ui-ready-foundations-design.md`.
- Completed prerequisite closeouts:
  - `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-002-native-editor-ux-story-lab-and-evidence-harness/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-003-layered-editor-design-system-migration/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-004-standalone-ui-designer-workbench/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-005-graph-canvas-and-node-editor-productization/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-006-shell-and-product-pattern-polish/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-007-all-registered-visible-surface-wave/closeout.md`;
  - `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-008-game-ui-readiness-seam/closeout.md`.
- App-owned native evidence and manifest validation:
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs` struct
  `EditorUxEvidenceManifest` and function `EditorUxEvidenceManifest::validate`.
- App-owned Story Lab evidence execution:
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs` function
  `run_story`.
- App-owned visible-widget scan:
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/visible_widget_scan.rs`
  function `scan_editor_ux_story`.
- Editor Story Lab source truth:
  `domain/editor/editor_shell/src/story_lab` modules `catalog`, `readiness`,
  `design_system`, `workbench`, `graph_canvas`, `product_patterns`,
  `surface_wave`, and `game_ui_readiness`.
- Surface readiness source truth:
  `domain/editor/editor_shell/src/workspace/surface_contract.rs` functions
  `editor_surface_definitions` and
  `tool_surface_readiness_for_definition_id`.
- Full validation entrypoint: `./quiet_full_gate.sh`.

## Readiness

`task production:plan -- --milestone PM-EDITOR-UX-009 --roadmap WR-119`
reported:

```text
Production milestone state: designing
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-118:completed
Next action: design_first
```

All milestone dependencies are now completed. Implementation and final
certification remain illegal until this contract validates,
`PM-EDITOR-UX-009` is moved to `ready_next`, `WR-119` is moved out of deferred
planning, and `task ai:goal -- --track PT-EDITOR-UX` is rerun. The next legal
action after that rerun should be promotion-readiness planning, not product
code.

## Promotion And Implementation-Readiness Contract

`task production:plan -- --milestone PM-EDITOR-UX-009 --roadmap WR-119`
reported the ready-next state as promotable after this design contract was
accepted:

```text
Production milestone state: ready_next
Roadmap planning_state: ready_next
Roadmap blocker: B2
Roadmap dependencies: WR-118:completed
Next action: write_promotion_contract
Promotion preflight status: promotable
```

Promotion is allowed only with this evidence:

- `PM-EDITOR-UX-009` is `ready_next`.
- `WR-119` is `ready_next`.
- Dependency `WR-118` is completed with closeout evidence.
- PM002 through PM008 have completed closeouts and evidence gates.
- This contract is active and names source truth, ownership, non-goals,
  implementation scope, validation, stop conditions, closeout requirements, and
  perfectionist audit rules.
- Product code remains unchanged by the design and promotion actions.

Promotion evidence string:

```text
Accepted PM-EDITOR-UX-009 final local-native no-gap certification design and promotion contract at docs-site/src/content/docs/reports/implementation-plans/wr-119-final-local-native-editor-ux-no-gap-certification/plan.md; completed PM002-PM008 closeouts and WR-118 game UI readiness seam closeout; production plan preflight status promotable.
```

After promotion, the next legal action is to write a narrowed final
implementation and certification contract before running the full final gate.
That implementation contract must name:

- the exact final closeout path;
- the exact prerequisite closeouts that must be verified;
- the exact local-native evidence or platform-impossible evidence expected;
- the exact hard zero-budget gates;
- the exact validation commands and completion metadata updates;
- the exact condition for `perfectionist_verified`: empty known quality gaps.

Do not run final certification from this design/promote action.

## Final Implementation And Certification Contract

`task production:plan -- --milestone PM-EDITOR-UX-009 --roadmap WR-119`
reported the promoted current-candidate state:

```text
Production milestone state: ready_next
Roadmap planning_state: current_candidate
Roadmap blocker: B2
Roadmap dependencies: WR-118:completed
Next action: write_implementation_contract
```

This section is the complete implementation contract for the final certification
slice. This action changes planning metadata only. Product code remains
unchanged until this contract validates and
`task ai:goal -- --track PT-EDITOR-UX` is rerun.

### Final Slice Owners And Files

- `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-009-final-local-native-no-gap-certification/closeout.md`
  owns the final PM009 closeout and zero-gap audit.
- `docs-site/src/content/docs/workspace/production-tracks.yaml` owns the
  `PM-EDITOR-UX-009` and `PT-EDITOR-UX` completion states, evidence gate, audit
  path, `completion_quality`, and `known_quality_gaps`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` owns the active
  `WR-119` row until final closeout.
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml` owns the completed
  `WR-119` archive row after final closeout.
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` must not regain
  `WR-119` unless a gate fails and the row is intentionally deferred again.
- `apps/runenwerk_editor/src/shell/editor_ux_story_lab/manifest.rs` function
  `EditorUxEvidenceManifest::validate`,
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/runner.rs` function
  `run_story`, and
  `apps/runenwerk_editor/src/shell/editor_ux_story_lab/visible_widget_scan.rs`
  function `scan_editor_ux_story` remain the app-owned evidence sources. They
  are not edited in the final certification slice unless a validation failure
  names an exact blocking defect.
- `domain/editor/editor_shell/src/story_lab` and
  `domain/editor/editor_shell/src/workspace/surface_contract.rs` remain the
  editor-domain source truth for story, readiness, product-pattern,
  registered-surface, and game-readiness evidence. They are not edited in the
  final certification slice unless a validation failure names an exact blocking
  defect.
- `domain/ui` remains the generic UI contract owner and is out of scope for
  final certification edits unless validation exposes a concrete UI-contract
  gap.

### Implementation Steps For The Next Action

After this contract validates and `task ai:goal -- --track PT-EDITOR-UX` is
rerun, the final certification action is:

1. Run
   `task production:plan -- --milestone PM-EDITOR-UX-009 --roadmap WR-119` and
   confirm the row still permits final implementation or closeout execution.
2. Verify PM002 through PM008 closeouts are present, have `status: completed`,
   and still match their production evidence gates.
3. Run `./quiet_full_gate.sh`. Stop immediately if it fails.
4. Run the workspace gates:
   `task docs:validate`, `task puml:validate`, `task roadmap:render`,
   `task roadmap:validate`, `task roadmap:check`, `task production:render`,
   `task production:validate`, `task production:check`,
   `task planning:validate`, and `git diff --check`.
5. Create the final PM009 closeout only if all gates pass. The closeout must
   cite the full-gate output and every prerequisite closeout.
6. Update `production-tracks.yaml` only after the closeout exists:
   `PM-EDITOR-UX-009` becomes `completed`, `PT-EDITOR-UX` becomes `completed`,
   `completion_quality` becomes `perfectionist_verified`, and
   `known_quality_gaps` is exactly empty.
7. Move `WR-119` from `roadmap-items.yaml` to `roadmap-archive.yaml` as
   `completed`, with the final closeout path, validation list, empty known
   gaps, and `perfectionist_verified` quality.
8. Rerun roadmap and production render/validate/check gates after metadata
   moves, then rerun `task ai:goal -- --track PT-EDITOR-UX`.

### Hard Zero-Budget Gates

The final closeout may claim `perfectionist_verified` only if all of these are
true:

- `./quiet_full_gate.sh` passes.
- PM002, PM003, PM004, PM005, PM006, PM007, and PM008 closeouts are completed
  and named by PM009.
- Local-native evidence is present where supported, or a typed
  platform-impossible report exists where capture is unavailable.
- Story Lab evidence, visible-widget scan evidence, retained manifests,
  accessibility reports, interaction/focus reports, diagnostics, timing,
  performance, provider, graph, workbench, product-pattern, registered-surface,
  and game-readiness evidence do not contradict each other.
- Roadmap source, production source, generated registers, PUML diagrams, docs,
  planning validation, and `git diff --check` all pass.
- `known_quality_gaps` is exactly `[]` for the final PM009 claim, the archived
  WR119 row, and the completed `PT-EDITOR-UX` track state.

### Final Certification Stop Rules

Stop and report instead of closing PM009 if any of these occur:

- `task production:plan` no longer reports `WR-119` as eligible for the final
  action.
- A prerequisite closeout is missing, not completed, or contradicts production
  evidence.
- `./quiet_full_gate.sh` fails.
- Any workspace gate fails after the full gate.
- A native capture is missing where the current app evidence contract requires
  local-native evidence.
- A platform-impossible report is used to bypass supported native capture.
- A validation failure requires product-code changes outside the current WR119
  write scopes.
- Any known quality gap remains.

## Architecture Governance Review

Recommendation: clear the design blocker and then run promotion planning before
final certification. Do not claim final certification in this action.

DDD owner:

- `apps/runenwerk_editor` owns local-native capture, app-owned evidence
  manifests, provider fixtures, accessibility/performance runners, and final
  certification artifacts.
- `domain/editor/editor_shell` owns Story Lab catalog, readiness decisions,
  product-pattern evidence, registered-surface evidence, and game-readiness
  seam evidence.
- `domain/ui` owns generic UI contracts consumed by the editor evidence path.
- Workspace roadmap and production docs own completion-quality claims and
  final zero-gap governance.

Vocabulary and invariants:

- Local-native evidence is app-owned evidence, not domain truth.
- Domain/editor and domain/ui contracts decide what must be proven; app
  evidence proves that those contracts execute locally.
- `perfectionist_verified` is allowed only when every hard zero-budget gate
  passes and `known_quality_gaps` is empty.
- Platform-impossible capture reports are acceptable only where native capture
  is genuinely unavailable and explicitly recorded.

Dependency direction:

```text
domain/ui -> domain/editor/editor_shell -> apps/runenwerk_editor -> workspace closeout evidence
```

ADR need: no ADR is required for final certification if it only audits and
records evidence through existing owners. Require an ADR or accepted design
update before moving app evidence authority into domain crates, changing
dependency direction, changing final certification quality semantics, or
claiming perfection without empty gap evidence.

ATAM-lite:

- Quality attributes in tension: no-gap certification rigor, local-native
  evidence availability, deterministic validation, and avoiding false
  perfection claims.
- Chosen option: make the final slice an evidence-gated audit that can close
  only after the full gate and all milestone closeouts agree.
- Sensitivity points: descriptor-only evidence, stale artifacts, local capture
  gaps hidden as success, and completion metadata that says completed while
  closeout evidence still names gaps.

Ownership mode: stream-aligned final certification with complicated-subsystem
support from UI, editor, app evidence, and workspace governance owners.

## Implementation Scope For The Later Slice

The later promoted final slice may change these owners and only these owners
unless the promotion contract narrows them further:

- `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-009-final-local-native-no-gap-certification/closeout.md`:
  create the final closeout with exact evidence and zero-gap audit.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`: mark
  `PM-EDITOR-UX-009` and `PT-EDITOR-UX` completed only if all final evidence
  gates pass and known quality gaps are empty.
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`: archive
  `WR-119` as completed only with the final closeout path and validation list.
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` and
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`: move `WR-119`
  through normal promotion/current-candidate/completed states.
- App or domain code may change only if final validation exposes a concrete
  blocking gap. Any such fix must be scoped to the owning module named by the
  failing evidence and must be validated before the final closeout.

## Non-Goals

- Do not implement game HUD behavior.
- Do not reopen completed PM002-PM008 scope unless validation exposes a
  concrete regression.
- Do not weaken hard zero budgets to finish the track.
- Do not claim `perfectionist_verified` with non-empty `known_quality_gaps`.
- Do not replace local-native evidence with descriptor-only proof.

## Acceptance Criteria For The Later Slice

- `./quiet_full_gate.sh` passes.
- All prerequisite PM002-PM008 closeouts still have completed evidence gates.
- Native screenshots or typed platform-impossible reports exist wherever the
  Story Lab and app evidence contracts require them.
- Accessibility, interaction, focus, visible-widget, diagnostics, timing,
  performance, readiness, provider, graph, workbench, product-pattern,
  registered-surface, and game-readiness evidence agree.
- Roadmap, production, docs, PUML, planning, and `git diff --check` gates pass.
- `PM-EDITOR-UX-009`, `WR-119`, and `PT-EDITOR-UX` completion metadata name
  the final closeout and have empty known quality gaps.

## Stop Conditions

Stop before final certification if:

- `./quiet_full_gate.sh` fails;
- any prerequisite closeout is missing, stale, or contradicts completion
  metadata;
- local-native evidence is missing where native capture is supported;
- a platform-impossible report is used where local-native capture should be
  available;
- any accessibility, focus, interaction, visible-widget, surface, provider,
  diagnostics, timing, or performance hard gate fails;
- `known_quality_gaps` is not empty;
- final certification requires code outside the current WR write scopes without
  a new implementation contract.

## Closeout Requirements

The later final closeout must include:

- a completed closeout at
  `docs-site/src/content/docs/reports/closeouts/pm-editor-ux-009-final-local-native-no-gap-certification/closeout.md`;
- exact validation output for `./quiet_full_gate.sh`, docs, roadmap,
  production, planning, PUML, and `git diff --check`;
- links to every PM002-PM008 closeout and PM009 final evidence;
- a table or bullet list proving every hard zero-budget gate is satisfied;
- explicit statement that `known_quality_gaps` is empty before claiming
  `perfectionist_verified`;
- completed roadmap archive metadata for `WR-119`;
- completed production metadata for `PM-EDITOR-UX-009` and `PT-EDITOR-UX`.

## Perfectionist Closeout Audit

Expected completion quality is `perfectionist_verified`.

`runtime_proven` is not enough for PM009. If any known quality gap remains, the
slice must stop or complete only as a lower-quality non-final closeout that does
not mark `PT-EDITOR-UX` completed.
