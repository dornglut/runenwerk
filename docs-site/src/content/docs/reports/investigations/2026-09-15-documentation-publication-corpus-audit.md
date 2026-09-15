---
title: Documentation Publication and Retention Audit
description: Exact-main audit of the Runenwerk documentation corpus, publication outputs, and deletion-first target live tree.
status: completed
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-09-15
related_docs:
  - ../../workspace/start-here.md
  - ../../workspace/documentation-structure.md
  - ../../workspace/crate-inventory.md
  - ../../workspace/planning/README.md
  - ../../workspace/planning/roadmap.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../README.md
---

# Documentation Publication and Retention Audit

## Decision summary

Runenwerk has a source-retention and publication problem, not a need for a
second documentation system. The smallest justified arrangement is one
Starlight source tree with explicit publication projections, plus a much smaller
live corpus.

The target is deletion-first:

- current product, developer, API, integration, and local architecture material
  remains current or reference documentation;
- ordinary implementation, delivery, planning, audit, and closeout history is
  deleted from the live tree after the owner and inbound-reference gates below;
- accepted ADR history is retained because ADR policy is a separate authority;
- the three visual-direction records and their three images remain as unique
  reference evidence;
- the RunenGPU proof matrix is reclassified to repository history;
- stale workspace ledgers, obsolete prompt/template material, duplicate status
  bookkeeping, superseded design lifecycle material, and ordinary report
  history are deletion candidates.

This report does not perform those deletions. It makes the later cuts bounded
and reviewable. The target counts are recommendations, not an accepted change
to #573, #492, #529, or #205.

## Exact evidence basis

The source census uses an immutable checked-out snapshot and `git ls-tree`, not
GitHub search:

```text
Runenwerk accepted main: bb10167a7fbddc5638884b4313a5388b6555524a
Runenwerk tree:          6bde4be304d2407ab3e2f9f671f814f1343fc6b7
Candidate predecessor:   d989534cab7bbc74813417992cb05c15402066ad
Candidate predecessor tree: dad25e958e299c31e1edafc232b6c1f7062f1eae
Engineering main:        782125ad30fa14ebcb1a875e3c4d7f78e8b89e7e
Issue:                   dornglut/runenwerk#690, opened 2026-09-15
```

The candidate predecessor was inspected in its own worktree. The current
checkout contains unrelated Render Lab changes and was not used as evidence.

The exact production build was run twice: once from the accepted Runenwerk
main snapshot and once from the candidate snapshot. The accepted-main build
used the checked-out `docs-site` lockfile and produced:

```text
HTML files:                 914 (913 document pages + 404.html)
Documentation routes:       913
Pagefind fragments:          913
Sitemap document URLs:       913
Homepage sidebar links:      376
```

The candidate adds this report, so its build produced 914 document routes,
914 Pagefind fragments, 914 sitemap document URLs, and 915 HTML files
including `404.html`. The candidate build is evidence of the delta, not the
current-main census.

Commands actually used for the source/build census included:

```text
git ls-tree -r --name-only 01e1c1be6c35b6d20e8ec1f897d73c7be2b04993
CI=true pnpm --dir docs-site install --frozen-lockfile
CI=true pnpm --dir docs-site build
```

## Changed retention policy

The live repository should contain current authority, useful current reference
material, durable decisions that still matter, and genuinely unique evidence.
Git history and GitHub issues, pull requests, and commits are the default
archive for ordinary implementation and delivery history. A historical file
must earn continuing live-tree presence through one of these tests:

1. current semantic authority;
2. current user/developer reference value;
3. necessary repository navigation;
4. durable architectural rationale still needed today;
5. unique evidence required for reproducibility, compatibility, migration,
   validation, or a future decision.

If none applies, the disposition is `DELETE`. Historical age, existing
presence, historical inbound links, an old workflow, or “just in case” value
is not a retention reason. A compatibility stub is not a retention strategy.

This is a deliberate change from the earlier audit’s bias toward preserving
already-retained reports and implemented/deferred design files. The policy
change does not make this report an implementation PR: path-specific owner
review, issue-scope reconciliation, and final inbound guards remain required.

## Governing authority

The authority chain is:

- code and tests own current behavior;
- accepted ADRs own durable local architecture;
- accepted designs own durable design contracts while they still govern;
- maintained roadmaps own durable sequence only;
- GitHub issues and the Engineering Portfolio own active work and live state;
- pull requests and exact-head CI own delivery evidence;
- Git history and GitHub preserve ordinary historical delivery records;
- Runenwerk owns integration/product policy, while Engineering owns
  organization-wide doctrine and Runen-family architecture.

The current Engineering snapshot contains `governance/authority-and-work.md`,
`standards/github.md`, `standards/validation.md`,
`standards/repositories.md`, `architecture/runen-family.md`, and
`tooling/gpt-web-github.md`. The path named by Runenwerk `AGENTS.md`,
`standards/software-design.md`, is absent from Engineering `main` at
`782125ad30fa14ebcb1a875e3c4d7f78e8b89e7e`. This is an authority-link defect,
not permission to recreate generic doctrine locally. It is an
`OWNER_REVIEW_REQUIRED` item for Engineering before local generic guideline
deletions are merged.

The current Runenwerk root entrypoints are distinct and justified:
`README.md` is the public repository landing page, `AGENTS.md` is the executor
contract, `ARCHITECTURE.md` is the root architecture map, `TESTING.md` owns
local validation semantics, and `LICENSING.md` owns current licensing policy.

## Current corpus census

The complete in-scope corpus is 1,341 tracked paths. The accounting is
disjoint and complete:

| Scope | Count | Accounting rule |
| --- | ---: | --- |
| Starlight source documents | 913 | Every tracked `.md`/`.mdx` under `docs-site/src/content/docs`. |
| Starlight subordinate assets | 398 | Every other tracked path under that content tree: RON/YAML/JSON/WGSL/PlantUML/PNG/KTX2 and similar evidence/assets. |
| Repository/package documentation | 14 | Root entrypoints, `docs-site/README.md`, five benchmark-artifact READMEs, `foundation/id/README.md`, and the two tooling READMEs. |
| Build/validation controls | 16 | Three workflows, docs-site config/lock/control files, docs tooling, and the docs-authority xtask tests listed below. |
| **Total** | **1,341** | Every in-scope path belongs to exactly one row. |

The 16 controls are `.github/workflows/ci.yml`,
`.github/workflows/docs-deploy.yml`, `.github/workflows/docs-validation.yml`,
`docs-site/.gitignore`, `docs-site/astro.config.mjs`,
`docs-site/package.json`, `docs-site/pnpm-lock.yaml`,
`docs-site/pnpm-workspace.yaml`, `docs-site/src/content.config.ts`,
`docs-site/tsconfig.json`, `tools/docs/add_agent_workflow_docs.sh`,
`tools/docs/migrate_docs_readmes_to_uppercase.py`,
`tools/docs/validate_docs.py`, `tools/xtask/src/main.rs`,
`tools/xtask/tests/docs_authority_cutover.rs`, and
`tools/xtask/tests/retired_workflow_authority.rs`.

### Routed source-document families

| Top-level family | Current routes | Source disposition summary |
| --- | ---: | --- |
| `index.mdx` | 1 | `PUBLISH_PRIMARY` |
| `adapters/` | 1 | `PUBLISH_PRIMARY` |
| `apps/` | 14 | `PUBLISH_PRIMARY` |
| `architecture/` | 6 | 5 reference, 1 delete candidate |
| `adr/` | 31 | 26 reference, 5 repository history |
| `design/` | 178 | 76 reference, 102 delete candidates |
| `domain/` | 27 | `PUBLISH_PRIMARY` |
| `engine/` | 76 | `PUBLISH_PRIMARY` |
| `foundation/` | 10 | `PUBLISH_PRIMARY` |
| `guidelines/` | 7 | 6 reference, 1 delete candidate |
| `net/` | 14 | `PUBLISH_PRIMARY` |
| `reports/` | 520 | 7 reference, 1 reclassify, 512 delete candidates |
| `workspace/` | 28 | 10 current, 2 merge sources, 16 delete candidates |
| **Total** | **913** | **143 primary + 120 reference + 10 current + 5 history + 1 reclassify + 2 merge + 632 delete** |

The `reports/` route count is separate from its 356 subordinate assets. The
site has 398 subordinate assets in total: 356 reports, 15 design, 23
workspace, and 4 architecture/app assets.

### Lifecycle status census

The current 913 source documents have these frontmatter statuses:

| Status | Count |
| --- | ---: |
| active | 400 |
| accepted | 92 |
| archived | 7 |
| completed | 255 |
| deferred | 28 |
| draft | 45 |
| implemented | 40 |
| superseded | 47 |
| **Total** | **913** |

Status is evidence for review, not a retention rule. In particular, a page
marked active can be obsolete, and an implemented design can be removable once
code/tests/current architecture own its meaningful content.

## Existing publication census

`docs-site/astro.config.mjs` currently autogenerates Workspace, an absent
Software Development directory, Domain, Engine, Net, Apps, Adapters, ADRs,
Design, and Guidelines. The build proves that source routing, sidebar
navigation, sitemap inclusion, and Pagefind indexing are currently coupled:

| Output | Count | Observed behavior |
| --- | ---: | --- |
| document routes | 913 | Every source document is routed, including reports and lifecycle material. |
| HTML files | 914 | 913 document pages plus `404.html`. |
| Pagefind fragments | 913 | Every document page is in the default search population. |
| sitemap document URLs | 913 | Every document page is in `sitemap-0.xml`; the sitemap index is an additional wrapper. |
| homepage sidebar links | 376 | Workspace, product, engineering, ADR, design, and guideline links are all mixed. |
| route-only pages | 537 | Routable/searchable pages absent from the homepage sidebar. |

The empty Software Development section is a concrete configuration defect. A
sidebar omission is not publication hiding: the 537 route-only pages remain
routable, searchable, and sitemapped. The current source has no `pagefind:
false` or `draft: true` document. `sidebar.hidden` only changes navigation.
The sitemap has no content filter. The Starlight page template does support
`pagefind: false`, but that control is not currently used.

The current landing page begins with Workspace Start Here, platform
architecture, documentation structure, programming principles, and Workspace
Overview. It is an engineering cold start, not a clear project/developer
entrypoint.

## Live-tree retention standard

Physical source location, route generation, sidebar navigation, sitemap
membership, and Pagefind membership are separate decisions.

The selected target uses one canonical source tree and four projections:

```text
source owner
  -> PUBLISH_PRIMARY       normal project/developer journey + default search
  -> PUBLISH_REFERENCE     stable web reference, separate navigation/search entry
  -> KEEP_REPOSITORY_CURRENT public repository authority, not ordinary site UX
  -> KEEP_REPOSITORY_HISTORY exceptional provenance, no normal site discovery
```

Primary documentation is the root project page, current product/domain/app,
engine, networking, foundation, adapter, and public integration material: 143
routes. Reference documentation is current local architecture/guidance,
accepted/proposed ADR navigation, active/accepted design authority, and the
small retained report evidence package: 120 routes. Repository-current
material is workspace orientation, planning, inventory, and reusable templates:
10 document paths, including one merged decision-method page. Repository
history is limited to the five superseded/rejected ADR paths plus the one
reclassified RunenGPU matrix; it is not a default search or sitemap population.

The landing page should become project/developer-first while
`workspace/start-here.md` remains the engineering cold-start page. No second
content tree, CMS, or audience-specific semantic copy is justified.

## Previous retention decisions and supersession

| Previous family/disposition | Previous rationale | New evidence/policy | New disposition | Superseded? |
| --- | --- | --- | --- | --- |
| #573 `reports/**` | Retained heterogeneous evidence and rejected broad deletion; allowed only bounded sidecar deletion. | Exact current census shows ordinary closeouts/plans/intake/batches/audits are delivery history recoverable through named PRs/issues/commits; only unique evidence earns live retention. | `DELETE` ordinary report history; retain benchmark/visual evidence; final path guard required. | **Yes, as a proposed retention model; no, as accepted issue scope.** |
| #492 design lifecycle | Preserved lifecycle/provenance and assigned owner-specific promotions/retirements. | Implemented, deferred, superseded, archived, and rejected files now frequently have code/tests/ADR/roadmap owners; the live repository is not a design museum. | Delete absorbed history; retain active/accepted authority; preserve ADR policy. | **Yes, for file retention only; lifecycle semantics remain.** |
| #529/#205 workspace cleanup | Removed retired bureaucracy but deliberately retained some compatibility/history records and closed the cleanup boundary. | The requested lean policy rejects compatibility stubs and manually maintained status mirrors as live material. | Delete the 16 obsolete workspace records; keep current owners; repair tooling first. | **Yes, proposed; owner-scope amendment required.** |
| #520/#688 relation semantics | Relation metadata and validation are current repository authority. | Deletions must not bypass relation/provenance checks. | `NO_CHANGE_PRIOR_DISPOSITION`; use as deletion gate. | No |
| ADR policy | Accepted ADRs are durable local architecture; history is not ordinary report history. | User request explicitly says to respect current ADR policy. | Keep accepted ADRs and five historical ADR paths until policy changes. | No |

The corrected investigation therefore does not pretend earlier decisions never
existed. It supersedes their retention premise only where the new deletion rule
and path evidence support it. Because #690’s current body explicitly says it
does not reopen #573, #492, #529, or #205, every such deletion is
`OWNER_REVIEW_REQUIRED` before implementation. This is the issue-scope
reconciliation, not silent contradiction.

## Historical deletion audit

The following families are deletion candidates under the new policy. The
family names are exact `git ls-files` predicates; the counts include every
member, not just Markdown. Historical inbound references are not treated as
current consumers.

| Path/family | Paths | Current owner/purpose | Unique content and current inbound | Recovery evidence | Migration gate | Disposition |
| --- | ---: | --- | --- | --- | --- | --- |
| `reports/closeouts/**` | 341 | Workspace/report evidence; records merged delivery, validation, and changed files. | Ordinary records duplicate PR/issue/commit evidence; retained closeout indexes and `related_docs` are historical/current-link candidates. | Named merged PRs and commits in the bodies; e.g. PR #77/commit `8b7a6b5...`, PR #173, PR #693, and Git history for every path. | Check for unique non-reconstructible fixtures/captures; migrate only if a current validator consumes them. | `DELETE` after `OWNER_REVIEW_REQUIRED` gate |
| `reports/implementation-plans/**` | 186 | Historical implementation plans and 35 plan contracts. | Current code/tests/ADRs/issues own accepted work; historical closeouts and evidence refer to some plans. | Git path history plus named issues/PRs; `wr-150.../plan.contract.yaml` is a concrete historical consumer to check. | Remove only stale relation/provenance consumers; do not rewrite historical claims to fake current truth. | `DELETE` after final consumer guard |
| `reports/roadmap-intake/**` | 152 | Retired generated proposal/intake machinery: 76 Markdown + 76 YAML. | No current roadmap authority; historical closeouts/contracts link individual proposals. | Git history and the corresponding GitHub issue/Portfolio record where one exists. | Ensure no current validator or maintained owner parses intake; historical links may die with the archive. | `DELETE` |
| `reports/batches/**` | 13 | Retired generated batch records. | Batch README and records are workflow history; current roadmap/issue authority is elsewhere. | Git history and PR/issue records named in each batch. | No current navigation or validation consumer. | `DELETE` |
| `reports/audits/**` | 8 | Point-in-time audits and truth-correction reports. | #492/#529 and current ADR/architecture/code absorbed the actionable conclusions; generic doctrine is Engineering-owned. | Git history plus the accepted cleanup PRs and Engineering authority. | Preserve a fact only if a path-specific owner proves it is not in current authority. | `DELETE` |
| `reports/investigations/**` | 33 | Supporting investigations, including the old investigation index. | Investigations do not authorize work; conclusions must be checked against current ADR/design/code. | Git history, issue discussions, accepted ADRs/designs, and merged PRs. | Keep an exception only for unique research/data needed for a future decision; otherwise delete this family after this report’s delivery slices. | `DELETE` with path-specific exception gate |
| `reports/migrations/scriptless-workflow-redesign-2026-06-25.md` | 1 | Point-in-time migration note. | Current workflow is now in root/Engineering authority; report is not current process. | Git history and its two commits `bbfdf443`/`2ce23f74`; current `AGENTS.md`, `TESTING.md`, and Engineering standards. | None unless a legal/recovery owner identifies a fact not present in current authority. | `DELETE` |
| `reports/design/editor-shell-menu-and-tab-chrome-polish-design.md` | 1 | Completed design record. | Current editor architecture/code/tests own behavior; retained visual package is separate. | Git history and accepted editor PR/issue evidence. | None. | `DELETE` |
| `reports/design/options.md`, `decision-analysis.md`, `selection.md` | 3 | Visual-direction decision package. | Unique options/trade-off/selection evidence; no duplicate accepted semantic authority found. | Git history and this retained package. | Keep images with the package; do not rewrite as current architecture. | `PUBLISH_REFERENCE` |
| `reports/design/runengpu-phase-requirements-proof-matrix.md` | 1 | Historical RunenGPU proof-role matrix. | Unique proof mapping may matter for provenance, but current framework semantics belong to standalone RunenGPU and current code/tests. | Git history, #494, accepted RunenGPU authority, and retained PR evidence. | Confirm any surviving proof facts are covered by standalone/current Runenwerk validation. | `RECLASSIFY_OR_MOVE` to repository history |
| `reports/benchmarks/**` | 4 | Benchmark methodology/result reference. | Command, hardware, measurements, and artifact context are unique. | Git history and benchmark artifact READMEs. | Retain only while methodology/baseline remains reproducible. | `PUBLISH_REFERENCE` |
| `reports/execution-evidence/**` | 130 | Generated YAML execution evidence. | No current validator consumes it; #573 identified a historical plan-contract reference to Track Harness evidence. | Git history, named plan contracts, closeouts, PRs, and issues. | Final exact-path guard; migrate a fixture only if current validation/reproducibility still consumes it. | `DELETE` after `OWNER_REVIEW_REQUIRED` gate |

The #573 post-closeout correction is explicitly reconciled: its Track Harness
sidecars were retained because `wr-150.../plan.contract.yaml` named them as
historical outputs. Under the new rule the plan contract and sidecars are both
ordinary historical delivery records, but deletion is not performed here until
the owner confirms that no current compatibility, recovery, or validation
consumer remains.

## Current-document pruning audit

The workspace tree is the clearest unprotected live-tree reduction boundary:

| Exact path(s) | Finding | Required action | Disposition |
| --- | --- | --- | --- |
| `workspace/crate-docs-status.md` | Manual coverage/status ledger duplicates derivable Cargo membership and source/docs inspection. `tools/docs/validate_docs.py` currently reads it. | Remove the validator’s manual-ledger dependency or replace it with source-derived checks; keep `crate-inventory.md` as membership authority. | `DELETE` after tooling migration |
| `workspace/planning-methods.md` + `workspace/architecture-governance-review.md` | Overlapping decision-evaluation, risk, ownership, and sequencing guidance. | Merge unique scoring/risk method material into one concise current Workspace owner and repair stale links. | `MERGE` (2 sources → 1) |
| `workspace/overview.md` | Thin orientation wrapper; its navigation role is covered by Start Here and Documentation Structure. | Update the landing page and links, then remove. | `DELETE` |
| `workspace/prompt-templates/architecture-audit.md` | One-off/generic prompt duplicates AGENTS, Engineering workflow, and current authority docs. | Delete; do not preserve AI prompt history as canonical documentation. | `DELETE` |
| `workspace/specs/README.md` and `workspace/specs/phase-implementation-spec.md` | The README explains a subordinate retired handoff layer; the phase template is explicitly superseded and points at retired workflow authority. | Delete the generic layer and historical RON snapshots after relation/provenance guard; migrate no semantic authority. | `DELETE` |
| `workspace/specs/*.ron` and `workspace/diagrams/**` | Historical G-phase/UI handoffs and diagrams, not current runtime authority. | Delete after checking the small set of current tooling skip rules and historical links. | `DELETE` |
| `workspace/design-implementation-triage.md`, `operating-model.md`, `production-milestone-register.md`, `production-track-index.md`, `production-track-planning-model.md`, `roadmap-archive-register.md`, `roadmap-decision-register.md`, `roadmap-deferred-register.md`, `roadmap-index.md`, `planning/typed-app-program-ui-proof-001-planning.md`, `sdf-first-execution-roadmap.md` | Explicitly superseded/retired/generated or completed historical planning wrappers. Current roadmap/GitHub/Engineering authority owns the live meaning. | Delete without forwarding stubs; keep the current roadmap and historical recovery in Git/GitHub. `sdf-first-execution-roadmap.md` requires #528/#529 owner reconciliation because it was previously retained as history. | `DELETE` after owner-scope gate |

The exact 16 workspace document paths deleted by that table are the 11 named
retired records above, `workspace/crate-docs-status.md`,
`workspace/overview.md`, `workspace/prompt-templates/architecture-audit.md`,
`workspace/specs/README.md`, and
`workspace/specs/phase-implementation-spec.md`. The RON/diagram assets are
accounted for separately in the 23 workspace asset deletions.

The ten current Workspace paths that remain are:

```text
workspace/ai-agent-boundaries.md
workspace/documentation-structure.md
workspace/crate-inventory.md
workspace/glossary.md
workspace/planning/README.md
workspace/planning/decision-register.md
workspace/planning/roadmap.md
workspace/start-here.md
workspace/templates/README.md
workspace/templates/domain-module-template.md
```

`workspace/architecture-governance-review.md` and
`workspace/planning-methods.md` are the two merge sources and produce one
current page, so they are not in that ten-path list.

## Design lifecycle pruning

The exact design route accounting is:

| Design family | Docs | Assets | Disposition |
| --- | ---: | ---: | --- |
| root `design/README.md` | 1 | 0 | `PUBLISH_REFERENCE` navigation |
| `design/active/**` | 21 | 0 | `PUBLISH_REFERENCE`; active designs remain current owners |
| `design/accepted/**` | 54 | 7 | `PUBLISH_REFERENCE`; accepted designs remain where they govern current work |
| `design/implemented/**` | 41 | 5 | `DELETE`; code/tests/current architecture now own implemented behavior |
| `design/deferred/**` | 29 | 3 | `DELETE`; future ideas belong in accepted issues/roadmap, not a design museum |
| `design/superseded/**` | 23 | 0 | `DELETE`; Git history is the archive |
| `design/archived/**` | 7 | 0 | `DELETE` |
| `design/rejected/**` | 1 | 0 | `DELETE` |
| `design/templates/**` | 1 | 0 | `DELETE` unless a current author demonstrates active reuse |
| **Total** | **178** | **15** | **76 retained docs, 102 deleted docs, 7 retained assets, 8 deleted assets** |

The active and accepted families are not automatically good: each remains only
because it currently owns semantic or architectural reference value. The
accepted-family diagrams are retained with their owners because they are
current architecture visuals, not generic history. The implemented/deferred
deletions are subject to #492 owner review; the recommendation supersedes file
retention, not the accepted lifecycle decisions or downstream RunenUI/RunenGPU
ownership boundaries.

## Reports-family pruning

The exact report route accounting is:

| Reports family | Routes | All tracked paths | Final target |
| --- | ---: | ---: | --- |
| `closeouts/**` | 230 | 341 | `DELETE` |
| `implementation-plans/**` | 150 | 186 | `DELETE` |
| `roadmap-intake/**` | 76 | 152 | `DELETE` |
| `investigations/**` | 33 | 33 | `DELETE` after unique-evidence check |
| `batches/**` | 13 | 13 | `DELETE` |
| `audits/**` | 8 | 8 | `DELETE` |
| `design/**` | 5 | 8 | 3 reference, 1 reclassify, 1 delete; retain the 3 PNGs |
| `benchmarks/**` | 4 | 4 | `PUBLISH_REFERENCE` |
| `migrations/**` | 1 | 1 | `DELETE` |
| `execution-evidence/**` | 0 | 130 | `DELETE` after consumer guard |
| **Total** | **520** | **876** | **7 reference, 1 history, 512 route deletions + 353 asset deletions** |

The `reports/**` path total is 876: 520 document routes and 356 subordinate
assets. The 353 report-asset deletions plus the 3 retained visual PNGs reconcile
to the 356 report assets. The old #573 group decisions are therefore not
silently treated as clear implementation authority; the target is a proposed
supersession that needs the owner-scope gate.

## Duplicate-authority findings

- `crate-docs-status.md` is a manual mirror of Cargo/source-derived facts and
  is not a durable semantic decision. Its current validator dependency is a
  tooling migration, not a reason to keep the page.
- `planning-methods.md` and `architecture-governance-review.md` are the only
  clear current-doc merge boundary. Merge the unique method into the stronger
  governance owner; do not add another generic design standard.
- `workspace/overview.md`, the retired Workspace registers, and the prompt
  template are wrappers or process residue with no current owner.
- Root and package READMEs do not form a proven duplicate pair. The package
  README supplies package identity/discoverability; the docs-site page supplies
  long-form semantics. Keep both unless a later exact comparison proves one is
  empty.
- `foundation/id/README.md` and the corresponding docs-site page serve package
  metadata versus canonical reference. Keep both.
- Current architecture, domain, engine, app, and adapter pages have user or
  developer reference value even where they are short. A short page is not
  deleted merely for being short when it is the obvious owner.
- Generic software-design doctrine belongs to Engineering. The missing
  Engineering file is an `OWNER_REVIEW_REQUIRED` authority defect; no local
  duplicate is created here.
- The configured Software Development sidebar is an empty navigation entry,
  not evidence for a compatibility page. Remove the configuration entry in the
  publication slice.

## Exact destructive-candidate matrix

The source-document matrix below is disjoint and accounts for all 913 routed
documents. Asset, repository-doc, and control paths are accounted for in the
following sections.

| Source disposition | Exact families | Count |
| --- | --- | ---: |
| `PUBLISH_PRIMARY` | root, `adapters/**`, `apps/**`, `domain/**`, `engine/**`, `foundation/**`, `net/**` | 143 |
| `PUBLISH_REFERENCE` | 5 current architecture docs, 6 current guidelines, 26 current ADR docs, 76 active/accepted design docs, 7 retained report docs | 120 |
| `KEEP_REPOSITORY_CURRENT` | 10 current Workspace docs | 10 |
| `KEEP_REPOSITORY_HISTORY` | 5 superseded/rejected ADR docs | 5 |
| `RECLASSIFY_OR_MOVE` | `reports/design/runengpu-phase-requirements-proof-matrix.md` | 1 |
| `MERGE` | `workspace/planning-methods.md`, `workspace/architecture-governance-review.md` | 2 |
| `DELETE` | 1 architecture, 1 guideline, 102 design, 512 reports, 16 workspace | 632 |
| **Total** | **all current routed source documents** | **913** |

For all 398 site assets, 4 current architecture/app assets are
`PUBLISH_PRIMARY`, 7 accepted design diagrams plus 3 retained report images
are `PUBLISH_REFERENCE`, and the remaining 384 assets are `DELETE` candidates.
All 14 repository/package documents and all 16 build/validation controls are
`KEEP_REPOSITORY_CURRENT`.

The global path disposition accounting is therefore:

| Disposition | Source paths |
| --- | ---: |
| `PUBLISH_PRIMARY` | 147 |
| `PUBLISH_REFERENCE` | 130 |
| `KEEP_REPOSITORY_CURRENT` | 40 |
| `KEEP_REPOSITORY_HISTORY` | 5 |
| `RECLASSIFY_OR_MOVE` | 1 |
| `MERGE` | 2 |
| `DELETE` | 1,016 |
| `SPLIT` | 0 |
| `REDUCE` | 0 |
| **Total** | **1,341** |

`MERGE` has two source paths and one output path. After that reduction, the
target live tree is 324 paths. The matrix intentionally uses `DELETE`, not
`REDUCE`, for `crate-docs-status.md`: its useful function should be replaced by
validation/tooling rather than preserved as another manual ledger.

## Unique facts requiring migration

No fact is migrated in this investigation, but the later implementation must
check these exact boundaries:

1. Remove or replace `validate_crate_docs_coverage()` in
   `tools/docs/validate_docs.py` before deleting `workspace/crate-docs-status.md`.
   Cargo membership and canonical crate inventory remain the owners.
2. Merge the unique scoring/risk method from `workspace/planning-methods.md`
   into the selected governance page and repair its retired links.
3. Before deleting report plans and execution evidence, check the named
   `reports/implementation-plans/wr-150-kernel-source-model-closure/plan.contract.yaml`
   consumer and every exact `execution-evidence/**` path consumer. A historical
   contract is not automatically current, but it must not be broken invisibly.
4. Keep the three visual-direction Markdown records and three PNGs together;
   they are the only report-design package with demonstrated unique visual
   decision evidence in this target.
5. Confirm the surviving RunenGPU proof facts are represented by current
   standalone RunenGPU authority or Runenwerk validation before moving the
   proof matrix to history.
6. Repair current links to the deleted Workspace wrappers and the superseded
   guideline. Do not rewrite historical report bodies merely to make their
   point-in-time paths look current.
7. Resolve the missing Engineering `standards/software-design.md` authority
   link before deleting any local generic doctrine that still contains facts not
   present in a verified Engineering successor.

## Rejected deletion candidates

These candidates remain because they earn live-tree presence under the rule:

- root `README.md`, `AGENTS.md`, `ARCHITECTURE.md`, `TESTING.md`, and
  `LICENSING.md`: profile-required or legal/current authority;
- `docs-site/README.md`: the docs-site maintainer entrypoint and build contract;
- current product/domain/engine/net/foundation/app/adapter docs: direct user or
  developer reference owners;
- `workspace/start-here.md`, `documentation-structure.md`,
  `crate-inventory.md`, `glossary.md`, planning README/roadmap/decision
  register, and the two templates: distinct current navigation, vocabulary,
  membership, sequence, or maintained authoring roles;
- accepted ADRs: retained under current ADR policy, not mass-deleted for
  publication tidiness;
- active/accepted design docs: retained only while they govern current work or
  provide current architecture reference;
- four benchmark reports and the three visual-direction records/images:
  reproducibility/methodology or unique visual evidence;
- package/tool READMEs: package identity, benchmark artifact navigation, or
  tooling usage with no proven docs-site duplicate.

## Target live-tree corpus

The projected live tree is:

| Target content | Paths | Publication role |
| --- | ---: | --- |
| Current product/developer docs | 143 docs + 4 assets | Primary site and default search |
| Current/reference docs | 120 docs + 10 assets | Reference site surface; not primary navigation |
| Current repository authority | 10 Workspace docs + 14 repo docs + 16 controls | Public repository, contributor/maintainer discovery |
| Exceptional history | 5 ADR docs + 1 reclassified proof matrix | Repository history, no default search/sitemap |
| Merged governance owner | 1 output replacing 2 Workspace sources | Repository current |
| **Projected live-tree paths** | **324** | **1,341 → 324** |

The 324 count includes the merged output and excludes the 1,017-path reduction
from direct deletion plus the one net path removed by merging two sources into
one output.

## Target publication model

Use one Starlight build over the surviving canonical tree. Add explicit
publication predicates only after the source cleanup is accepted:

- primary navigation includes the 143 primary routes and a link to the
  separate reference index;
- reference navigation includes the 120 reference routes, including accepted
  ADR/design/architecture material and the retained benchmark/visual records;
- repository-current pages remain accessible from GitHub/contributor links and
  are not part of the ordinary product journey;
- repository history is retained only for the six exceptional paths and is not
  in the default sidebar, Pagefind population, or sitemap;
- `index.mdx` becomes a project/developer landing page;
- remove the empty Software Development sidebar entry;
- do not use `draft: true` for retained history until stable-link ownership is
  explicitly accepted; a draft route disappears from production;
- do not create audience-specific content copies. If the existing lifecycle and
  path signals cannot express the projections, prove that ambiguity first and
  add one explicit publication field in the owning build boundary.

Projected output counts:

| Output | Current | Target |
| --- | ---: | ---: |
| live-tree paths | 1,341 | 324 |
| routed documentation pages | 913 | 263 |
| default Pagefind population | 913 | 143 |
| sitemap document URLs | 913 | 263 |
| primary sidebar links | 376 mixed links | 143 primary links + reference index |

The target deliberately separates “web-linkable reference” from “default
search.” Reference pages may receive a dedicated reference search/index later;
the default Pagefind population is the primary developer journey only.

## Current vs target counts

| Required measure | Current exact value | Target/projection |
| --- | ---: | ---: |
| complete corpus paths | 1,341 | 324 |
| routed source docs | 913 | 263 |
| Pagefind docs | 913 | 143 default |
| sitemap docs | 913 | 263 |
| `DELETE` | 0 in this investigation | 1,016 proposed path deletions |
| `MERGE` | 0 | 2 source paths → 1 output |
| `SPLIT` | 0 | 0 |
| `REDUCE` | 0 | 0 |
| `RECLASSIFY_OR_MOVE` | 0 | 1 |
| retained current source paths | 0 as a change | 318 after merge |
| retained history source paths | 0 as a change | 6 |

The “current” disposition-action counts are zero because this PR changes only
the investigation report. The target counts describe the later implementation
candidate and reconcile to 1,341 current paths and 324 target paths.

## Ordered implementation plan

The smallest coherent delivery slices are:

0. **Scope and authority reconciliation.** Amend or otherwise explicitly
   reconcile #690 with #573/#492/#529/#205; resolve the missing Engineering
   software-design authority link; re-resolve main, owners, and all exact
   inbound references. No deletion starts before this gate.
1. **Historical corpus deletion.** Delete the report families and design/
   workspace history in the matrix, retaining only the four benchmark routes,
   three visual records/images, five ADR history routes, and the reclassified
   proof matrix. Apply final plan-contract, evidence, link, and unique-fact
   guards. This is one retention-rule delivery, not a broad cleanup umbrella.
2. **Current-doc consolidation.** Merge the two Workspace method/governance
   pages, delete the crate-status ledger after validator migration, remove the
   obsolete prompt/template/wrapper pages, and repair current links. Keep this
   slice separate from report deletion because it changes current authority.
3. **Publication correction.** Change the landing page and sidebar, remove the
   empty Software Development section, add primary/reference/history
   projections, and verify the built route/Pagefind/sitemap populations.

Each slice must be built from current accepted main, have one complete
candidate, run `git diff --check`, `cargo validate`, and the docs build when its
paths require it, and receive exact-head CI. Do not make a generated prompt,
ledger, cleanup umbrella, compatibility alias, or second semantic corpus.

## Final decision

Adopt the deletion-first target as the corrected investigation outcome, subject
to the explicit owner gates. The earlier report’s “no deletion is supported”
conclusion is rejected: it treated accepted historical retention as a reason to
keep ordinary delivery records and did not test whether the retained material
still earned live-tree presence under the new policy.

The correct durable architecture is one canonical Starlight source tree with
separate primary/reference/repository/history projections. The correct source
architecture is a lean current tree with Git/GitHub as the default archive.
The investigation remains implementation-only in scope: no actual corpus path,
publication rule, validator, navigation, or sitemap is changed here.

## Validation and remaining boundary

Observed after the final report replacement and rebase onto the accepted main
snapshot:

```text
CI=true pnpm --dir docs-site build  PASS; 915 HTML files, 914 document routes,
                                   914 Pagefind fragments, 914 sitemap URLs
cargo validate                         all validation stages passed
git diff --check origin/main...HEAD    PASS
```

The final candidate build observed 915 HTML files, 914 document routes, 914
Pagefind fragments, 914 sitemap document URLs, and 376 sidebar links. The exact
accepted-main build at `bb10167a` observed 914 HTML files, 913 document routes,
913 Pagefind fragments, 913 sitemap document URLs, and 376 sidebar links.
`cargo validate` completed tooling/workspace formatting, tests, clippy, docs
validation, and repository audit successfully; the local zsh wrapper reported
a post-command error only because it assigned the read-only variable name
`status` after validation had completed.

No cleanup is performed by this report. `OWNER_REVIEW_REQUIRED` remains for:

- issue-scope amendment/reconciliation against #573, #492, #529, and #205;
- Engineering’s missing `standards/software-design.md` authority path;
- final path-level closeout/plan/execution-evidence consumer guard;
- deletion of the previously retained `sdf-first-execution-roadmap.md`;
- unique-fact review for report investigations and implemented/deferred designs;
- validator migration before deleting `crate-docs-status.md`.
