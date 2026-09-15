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

## Decision

Runenwerk needs a leaner live corpus and a publication projection, not a second
documentation system. The implementation target is one canonical Starlight
source tree with explicit primary, reference, and history projections. This
pull request is report-only: it does not delete paths or change the build.

The deletion-first target is decision-complete: retain current product,
developer, integration, package, ADR, and active/accepted design owners; retain
only named implemented/deferred/investigation exceptions with live authority,
current code/test consumption, or unique evidence; delete ordinary plans,
closeouts, intake, batches, audits, generated execution evidence, obsolete
Workspace records, and unowned lifecycle history; reclassify the RunenGPU
proof matrix to repository history; and create no alias, forwarding page,
mirror, ledger, or second authority.

## Evidence and authority

~~~text
Audit-base main:      bb10167a7fbddc5638884b4313a5388b6555524a
Audit-base tree:      6bde4be304d2407ab3e2f9f671f814f1343fc6b7
Current main:         60e97651a88de7ffdd2e13313c68277dd310a145
Current tree:         ce31d594c4fe8a74039b7692e5cf52ecc840bbd6
Engineering main:     782125ad30fa14ebcb1a875e3c4d7f78e8b89e7e
Issue:                dornglut/runenwerk#690 (open)
Correction parent:    4af00cdb8b37e174e2935f7c1af8a1efa4bf13b2
~~~

The current-main delta from the audit base is exactly #703: deletion of seven
runtime-bootstrap lines and addition of the native-window ownership test. No
in-scope documentation, asset, docs control, tooling, or publication input
changed. The correction therefore continues linearly from 4af00cdb without a
blind rebase. Unrelated Render Lab changes in the user's main checkout were
not used. The current-main census is from:

~~~text
git ls-tree -r --name-only 60e97651a88de7ffdd2e13313c68277dd310a145 docs-site/src/content/docs
~~~

Engineering main contains and was read in full:
standards/software-design.md. It is the project-neutral authority for
authority/invariants, contracts and flows, dependency direction, derived state,
policy/capability/validity, consistency, failure diagnostics, evolution,
abstraction cost, tests, public surfaces, automation, and design-for-deletion.
It requires compatibility surfaces to have real consumers and removal
conditions. The previous report's missing-file finding was false and is removed.
Runenwerk guidelines retain only local specialization; generic doctrine is not
recreated.

The authority chain is code/tests for behavior, accepted ADRs for durable local
architecture, active/accepted designs for current design ownership, maintained
roadmaps for future sequence, GitHub/Portfolio for active work, and Git/GitHub
history for ordinary delivery records. Runenwerk owns integration/product
policy; Engineering owns organization-wide doctrine and Runen-family
architecture.

## Issue and open writers

Issue #690 was updated while still open. Its original SHA
26e33c390ffeba1b708978ac5353407d4d47cc17 is historical context only. The issue
now explicitly authorizes reconsidering live-tree retention outcomes from #573,
#492, #529, and #205 when current authority, unique value, recoverability, and
consumer analysis support deletion or reclassification. Earlier semantic
decisions and ADR policy remain historical authorities; they are not silently
reversed. The issue remains report-only and does not authorize cleanup or build
changes.

The only other open writer is #703, a runtime Rust/test PR with no documentation
overlap. PR #699 is the sole documentation writer:

| Writer | Scope | Overlap |
| --- | --- | --- |
| #699, codex/docs-publication-corpus-audit | This report and index | Documentation audit only |
| #703, agent/app-runtime-native-hook-ownership-700 | Runtime bootstrap and test | None |

PR #699 remains draft. Its final follow-up head is recorded in the PR body after
publication; it must stay draft until exact-head CI and full diff review pass.

## Exact corpus census

The in-scope corpus is 1,341 tracked paths: 913 routed source documents, 398
content assets, 14 repository/package docs, and 16 build/validation controls.
These sets are disjoint and complete.

| Scope | Paths |
| --- | ---: |
| Starlight documents | 913 |
| Starlight assets | 398 |
| Repository/package docs | 14 |
| Build/validation controls | 16 |
| **Total** | **1,341** |

| Routed family | Docs | Final result |
| --- | ---: | --- |
| index.mdx, adapters/, apps/, domain/, engine/, foundation/, net/ | 143 | PUBLISH_PRIMARY |
| architecture/ | 6 | 5 reference, 1 delete |
| adr/ | 31 | 26 reference, 5 history |
| design/ | 178 | 87 reference, 2 current, 89 delete |
| guidelines/ | 7 | 6 reference, 1 delete |
| reports/ | 520 | 15 reference, 1 history, 504 delete |
| workspace/ | 28 | 10 current, 18 delete |
| **Total** | **913** | **143 primary + 139 reference + 12 current + 5 history + 1 reclassify + 613 delete** |

The 398 assets are 356 reports, 15 design, 23 Workspace, and 4
architecture/app assets. Four primary assets, ten reference assets, and 384
asset deletions reconcile this total. All 14 repository docs and 14 controls
are KEEP_REPOSITORY_CURRENT; the two historical helper scripts are DELETE.

## Proven publication census and defect

The accepted-main docs build produced:

~~~text
HTML files:             914 (913 document pages + 404.html)
Documentation routes:   913
Pagefind fragments:     913
Sitemap document URLs:  913
Homepage sidebar links: 376
~~~

All source documents are routed, searchable, and in the sitemap. The 537
route-only pages are absent from the homepage sidebar but remain discoverable.
The configured Software Development section is empty. Lifecycle status is not
publication status: accepted, implemented, deferred, superseded, archived, and
rejected pages are currently exposed through route/search/sitemap behavior. The
landing page is engineering-first instead of a project/developer entrypoint.

## Final disposition and arithmetic

| Disposition | Exact set | Paths |
| --- | --- | ---: |
| PUBLISH_PRIMARY | 143 primary docs + 4 primary assets | 147 |
| PUBLISH_REFERENCE | 139 reference docs + 10 reference assets | 149 |
| KEEP_REPOSITORY_CURRENT | 10 Workspace docs + 2 design fixtures + 14 repo docs + 14 controls | 40 |
| KEEP_REPOSITORY_HISTORY | 5 superseded/rejected ADR docs | 5 |
| RECLASSIFY_OR_MOVE | RunenGPU proof matrix | 1 |
| DELETE | every remaining in-scope path, including unlisted assets and two helper scripts | 999 |
| SPLIT / REDUCE / MERGE | none | 0 |
| **Total** | **all in-scope paths** | **1,341** |

Target live tree: 1,341 - 999 = 342. There is no merge output: the two
generic Workspace sources are deleted and only their Runenwerk-specific
material migrates into the existing planning owner.

## Reports and exact exceptions

| Family | All paths | Final result |
| --- | ---: | --- |
| reports/closeouts/** | 341 | DELETE |
| reports/implementation-plans/** | 186 | DELETE; migrate then remove the two test dependencies |
| reports/roadmap-intake/** | 152 | DELETE |
| reports/investigations/** | 33 | 8 reference, 25 DELETE |
| reports/batches/** | 13 | DELETE |
| reports/audits/** | 8 | DELETE |
| reports/design/** | 8 | 3 visual reference, 1 history, 4 DELETE |
| reports/benchmarks/** | 4 | PUBLISH_REFERENCE |
| reports/migrations/** | 1 | DELETE |
| reports/execution-evidence/** | 130 | DELETE |

The eight individually retained investigations are:

~~~text
2026-08-04-runenrender-long-term-capability-and-scalability-review.md
2026-08-12-application-composition-and-networking-ergonomics.md
2026-08-12-semantic-federation-and-inspection-provenance.md
live-uiplugin-runtime-current-state-investigation.md
runen-family-operational-hardening-investigation.md
runenecs-issue-198-current-main-census.md
runengpu-runenrender-application-domain-fit.md
runenrender-extraction-investigation.md
~~~

They are linked from current accepted ADRs, architecture, or accepted/active
decomposition authority and contain unique current-state rationale. Every other
path in reports/investigations/**, including README.md, typed-app archives,
RunenGPU G2-G5/S0 reviews, RunenSDF extraction, surface2d branches, and the
2026-09-03 backend-neutrality audit, is DELETE.

Retain reports/design/options.md, decision-analysis.md, selection.md, and their
three PNGs as unique visual evidence. Retain the four benchmark reports for
reproducible commands, hardware, measurements, and artifact context. Move the
RunenGPU proof matrix to repository history, not current framework authority.

Consumer resolution is exact:

- Tests currently read implementation-plans/wr-029-model-mesh-material-binding/plan.md
  and wr-030-model-mesh-renderable-scene-contract/plan.md. They protect the
  current source-of-truth ownership, SDF non-regression, and Mesh Preview-first
  boundary already carried by the active Material Lab design and current code.
  Migrate the assertions to those current owners, then DELETE both plans; they
  are not retained fixtures.
- wr-150-kernel-source-model-closure/plan.contract.yaml only names historical
  execution-evidence outputs and has no current code/tool consumer; delete it.
- reports/execution-evidence/** has no current reader. The matching editor visual
  test path is a generated output directory, not a checked-in consumer.
- Links from retained docs to deleted historical reports are link-repair work,
  not retention reasons.

## Design lifecycle decisions

Active/accepted design authority remains reference material. Implemented and
deferred paths were checked against current code, tests, architecture, ADRs, and
roadmaps. The exact retained exceptions are:

~~~text
design/implemented/editor-self-authoring-and-final-ui-design.md
design/implemented/editor-ui-runtime-v2-and-interaction-formation-design.md
design/implemented/field-visualizer-product-workflow-design.md
design/implemented/render-product-surface-foundation-bundle-design.md
design/implemented/ui-component-platform-render-surface-output-design.md
design/implemented/ui-definition-formation-foundation-design.md
design/implemented/viewport-dynamic-product-target-allocation-design.md
design/implemented/workspace-viewport-expression-upgrade-design.md
design/implemented/editor-rendered-world-and-multi-entity-viewport-design.md
design/implemented/ui-program-architecture-owner-map.md
design/implemented/ui-program-architecture.md
design/deferred/sdf-prefab-composition-system-design.md
design/deferred/ui-model-multiple-execution-strategies-design.md
~~~

The nine retained implemented docs are PUBLISH_REFERENCE because current
editor/UI/render roadmaps or architecture use them as durable reference points,
including editor-rendered-world-and-multi-entity-viewport. The two UI program
docs are KEEP_REPOSITORY_CURRENT because the current design-conformance fixture
and architecture roadmap treat them as current semantic-program authority; that
is semantic retention, not merely a test-path exception. The two deferred docs are
PUBLISH_REFERENCE because current editor/UI roadmaps and the active asset
pipeline design identify them as future intent.

Every other path in design/implemented/** and design/deferred/** is DELETE,
including both README files and all 8 implemented/deferred diagram assets.
Implemented behavior is owned by code/tests/current architecture; deferred
ideas without a current owner do not earn live-tree presence. Lifecycle meaning
and accepted ADR authority remain unchanged.

## Workspace and duplicate authority

Retain exactly these ten Workspace docs:

~~~text
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
~~~

Delete crate-docs-status.md, overview.md, the prompt-template page, both
Workspace specs Markdown pages, all RON specs and diagrams, retired/generated
planning registers, triage/operating-model/milestone/track pages, the typed-app
planning page, and sdf-first-execution-roadmap.md. This is 18 Workspace doc
deletions plus 23 asset deletions.

tools/docs/validate_docs.py currently reads crate-docs-status.md. Before that
ledger is deleted, the cleanup must replace validate_crate_docs_coverage() with
source-derived Cargo/inventory checks. The generic sources
workspace/planning-methods.md and workspace/architecture-governance-review.md
are both DELETE, not MERGE: Engineering owns their generic doctrine, and only
Runenwerk-specific scoring/ownership rules migrate into the existing planning
README or decision register. No new authority is created.

Current links and the docs_authority_cutover test are bounded migration/link
repair work. They do not justify retaining obsolete corpus.

## Build and validation controls

The 16 controls were audited individually. Fourteen remain current:

| Control | Why it remains |
| --- | --- |
| .github/workflows/ci.yml | Canonical repository validation caller |
| .github/workflows/docs-deploy.yml | Current Pages publication caller |
| .github/workflows/docs-validation.yml | Current documentation build/validation caller |
| docs-site/.gitignore | Keeps generated site output out of source |
| docs-site/astro.config.mjs | Current Starlight publication input |
| docs-site/package.json | Build scripts and dependency contract |
| docs-site/pnpm-lock.yaml | Reproducible docs dependency resolution |
| docs-site/pnpm-workspace.yaml | Current package workspace boundary |
| docs-site/src/content.config.ts | Current docs collection/schema loader |
| docs-site/tsconfig.json | Current docs type-check configuration |
| tools/docs/validate_docs.py | Invoked by TESTING.md, docs-site/README.md, and xtask |
| tools/xtask/src/main.rs | Owns cargo validate orchestration |
| tools/xtask/tests/docs_authority_cutover.rs | Current authority-boundary regression test |
| tools/xtask/tests/retired_workflow_authority.rs | Current retired-workflow regression test |

The two remaining scripts are DELETE. The README case migration script has no
caller or maintained reference; its one-off result is already present in the
current tree. The agent-workflow script identifies itself as a historical
compatibility entrypoint and has no legitimate caller beyond its own comments;
current docs are maintained directly and it only invokes validate_docs.py. No
forwarding compatibility entrypoint is retained.

The resulting control disposition is 14 KEEP_REPOSITORY_CURRENT and 2 DELETE.

## Supersession and retained boundaries

| Earlier authority | Earlier rationale | Current decision |
| --- | --- | --- |
| #573 reports retention | Broad deletion was not then accepted; bounded sidecars remained | Supersede ordinary delivery-history retention; retain benchmark/visual evidence |
| #492 design lifecycle | Lifecycle and owner-specific promotion/retirement | Supersede file retention only where code/tests/current roadmap own substance; lifecycle remains |
| #529/#205 Workspace cleanup | Retired bureaucracy was removed but some context remained | Supersede retention of named obsolete records; no compatibility mirror/ledger |
| ADR policy | Durable local decisions are ADR-governed | NO_CHANGE_PRIOR_DISPOSITION; five historical ADR paths remain |

The updated #690 issue explicitly authorizes these re-evaluations. No
OWNER_REVIEW_REQUIRED exception remains for the Engineering file, generic
guidelines, Workspace method sources, investigations, or implemented/deferred
design retention. Remaining dependencies are concrete tooling/link migrations,
not unresolved retention decisions.

## Publication architecture and feasibility

Use one Starlight build over the surviving canonical tree and one explicit
frontmatter field such as publication: primary|reference|repository-current|history.
Do not duplicate documents.

A disposable build experiment used Astro 6.1.1 / Starlight 0.38.2 and a
temporary page with:

~~~yaml
pagefind: false
sidebar:
  hidden: true
~~~

Observed for pagefind:false and sidebar.hidden: the page generated a stable
HTML route and sitemap URL, was absent from the homepage sidebar, and was absent
from Pagefind output. A second disposable build with draft:true generated no
HTML route, Pagefind entry, or sitemap URL. The selected model is therefore:
repository-current and history documents remain in the repository but use
draft:true and are outside production route generation; primary/reference docs
remain routed, with reference pages excluded from default Pagefind as needed.
Direct repository links remain GitHub links, while published primary/reference
links remain stable site links. Minimal implementation: publication metadata with
all four classes, filtered primary/reference sidebars, draft:true for
repository-current/history, pagefind:false for non-default-search reference
pages, removal of the empty Software Development entry, and a
project/developer-first index.mdx.

Projected counts, not yet built:

| Measure | Current | Target |
| --- | ---: | ---: |
| live-tree paths | 1,341 | 342 |
| routed primary/reference docs | 913 | 282 |
| default Pagefind docs | 913 | 143 primary |
| sitemap primary/reference docs | 913 | 282 |
| homepage sidebar | 376 mixed links | 143 primary links plus reference entry |

The 282 route projection excludes ten repository-current docs, five
repository-history ADR routes, and the reclassified proof matrix. The proven
current values are only the accepted-main build values above; 282 is projected
from the draft:true route-suppression experiment.

## Implementation order and validation

1. Delete the exact historical corpus with no current executable or tooling
   authority: ordinary closeouts, intake, batches, audits, execution evidence,
   unowned investigations/design history, obsolete Workspace records, and the
   two helper scripts.
2. Migrate crate-docs-status validation, migrate WR-029/WR-030 assertions to
   current Material Lab/code/design owners, remove the historical test
   dependency, migrate the small Runenwerk planning subset, and update
   docs-authority tests. Then delete those now-unowned plans.
3. Apply publication metadata, draft-based route suppression, navigation,
   sitemap, Pagefind, and landing-page corrections.
4. On each bounded implementation candidate run git diff --check, cargo validate,
   the docs build, exact-head hosted CI, and a full diff review. Keep PR #699
   draft until all pass.

Observed for the final candidate:

~~~text
Accepted-main docs build: 914 HTML files, 913 routes, 913 Pagefind fragments,
                          913 sitemap document URLs, 376 sidebar links
Candidate final build:   915 HTML files, 914 routes, 914 Pagefind fragments,
                          914 sitemap document URLs
Candidate final local:   cargo validate PASS; git diff --check PASS
PR #699 final CI:        repository baseline PASS; docs build PASS; Vulkan PASS
~~~

Only these two files are in scope:

~~~text
docs-site/src/content/docs/reports/investigations/2026-09-15-documentation-publication-corpus-audit.md
docs-site/src/content/docs/reports/investigations/README.md
~~~

The final follow-up is a normal non-rewriting commit on top of
4af00cdb8b37e174e2935f7c1af8a1efa4bf13b2. Its exact head, current base/tree,
Engineering SHA, counts, dispositions, validation, and blockers are recorded
in the PR body. PR #699 remains draft pending independent diff review.
