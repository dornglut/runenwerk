---
title: Documentation Structure
description: Source-of-truth rules, document lifecycles, placement policy, naming rules, and maintenance expectations for Runenwerk documentation.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-26
related_adrs:
  - ../adr/accepted/0001-use-domain-owned-commands.md
  - ../adr/accepted/0002-keep-ai-out-of-foundation.md
  - ../adr/accepted/0003-ratification-is-domain-specific.md
  - ../adr/accepted/0004-separate-description-from-execution.md
  - ../adr/accepted/0005-projections-are-derived-state.md
---

# Documentation Structure

## Purpose

This document defines how Runenwerk documentation is organized, where different document types belong, how document lifecycle states work, and which documents are authoritative when information overlaps.

The goal is to keep documentation:

- easy to navigate;
- explicit about ownership;
- clear about current versus historical truth;
- useful for humans and AI-assisted contributors;
- resistant to drift as the repository grows;
- aligned with Runenwerk's architecture, dependency direction, and domain ownership model.

Runenwerk documentation should make it obvious whether a document is:

- current doctrine;
- an architectural decision;
- an active design;
- an accepted design;
- an implemented design;
- an implementation roadmap;
- a historical closeout;
- a benchmark or audit report;
- a usage guide;
- a local crate/domain/app reference;
- archived non-authoritative material.

## Core Model

Runenwerk uses a two-level documentation model:

```text
repository root markdown files
  quick operational summaries for humans and AI agents working from the repository root

docs-site/src/content/docs
  canonical long-form documentation tree
```

Root markdown files are entrypoints and summaries.

The docs-site tree is the canonical location for detailed documentation.

When a root document and a docs-site document overlap, update the docs-site document first, then align the root summary.

## Repository Root Documents

The following root files are intentionally kept at repository root:

```text
README.md
AGENTS.md
AI_GUIDE.md
ARCHITECTURE.md
CRATES.md
DEPENDENCY_RULES.md
DOMAIN_MAP.md
GLOSSARY.md
TESTING.md
```

These files exist because contributors and AI agents often begin from the repository root.

Root documents should be:

- concise;
- stable;
- operationally useful;
- safe to read before making changes;
- linked to canonical docs-site pages where detail exists.

Root documents should not become:

- full design documents;
- detailed implementation roadmaps;
- benchmark reports;
- long historical records;
- duplicated copies of docs-site pages;
- dumping grounds for incomplete notes.

## Canonical Documentation Tree

The canonical documentation tree is:

```text
docs-site/src/content/docs
```

In exported or shortened paths, this may also be referred to as:

```text
content/docs
```

Recommended high-level structure:

```text
content/docs/
├── index.mdx
├── workspace/
├── guidelines/
├── adr/
├── design/
├── domain/
├── apps/
├── net/
├── adapters/
├── reports/
└── archive/
```

## Folder Responsibilities

### `workspace/`

Use `workspace/` for repository-wide process, structure, validation, status, and maintenance documentation.

Examples:

```text
workspace/documentation-structure.md
workspace/root-docs-map.md
workspace/crate-docs-status.md
workspace/roadmap-index.md
workspace/validation-commands.md
```

`workspace/` is for documentation about the repository itself.

It should not contain crate-specific implementation detail unless the document is an index or status tracker.

### `guidelines/`

Use `guidelines/` for stable doctrine that applies across multiple crates, domains, apps, or workflows.

Examples:

```text
guidelines/runenwerk-architecture.md
guidelines/dependency-rules.md
guidelines/domain-map.md
guidelines/module-structure-guidelines.md
guidelines/testing.md
guidelines/ai-guide.md
```

Guidelines define rules. They should be stable and intentionally maintained.

Guidelines may be summarized by root files, but the docs-site guideline page is the canonical long-form source.

### `adr/`

Use `adr/` for architectural decision records.

ADRs record accepted, proposed, rejected, or superseded decisions that are expensive to reverse or important to long-term architecture.

Recommended structure:

```text
adr/
├── README.md
├── proposed/
├── accepted/
├── superseded/
└── rejected/
```

ADRs should not be used for temporary implementation plans.

If a design creates a long-term architectural rule, create or update an ADR.

If an ADR and another document conflict, the accepted ADR wins unless it has been superseded.

### `design/`

Use `design/` for architectural design documents.

A design document defines target architecture, ownership boundaries, invariants, dependency constraints, tradeoffs, migration paths, and validation expectations.

Recommended structure:

```text
design/
├── README.md
├── active/
├── accepted/
├── implemented/
├── deferred/
├── superseded/
├── rejected/
├── archived/
└── templates/
```

A design document is not an ADR, not a roadmap, and not a report.

Design documents explain architecture. Roadmaps explain implementation sequence. Reports record evidence. ADRs record decisions.

### `domain/`

Use `domain/` for engine-agnostic domain documentation.

Examples:

```text
domain/ecs/
domain/editor/
domain/ui/
domain/sdf/
domain/scene/
domain/scheduler/
```

Domain docs should describe:

- purpose;
- ownership;
- public API usage;
- internal architecture where useful;
- invariants;
- extension points;
- testing;
- active or completed roadmaps where relevant.

Domain docs must not depend on app-specific assumptions unless clearly marked as integration notes.

### `apps/`

Use `apps/` for application-specific documentation.

Examples:

```text
apps/runenwerk-editor/
```

Application docs may describe:

- product scope;
- MVP definitions;
- user workflows;
- app-specific architecture;
- app-specific integration plans;
- app roadmaps.

Application docs may reference domain docs, but should not redefine domain invariants.

### `net/`

Use `net/` for networking, replication, synchronization, replay, transport, and simulation/runtime communication documentation.

Examples:

```text
net/ecs-runtime-feature-inventory.md
net/ecs-runtime-gap-summary.md
net/ecs-runtime-prioritized-roadmap.md
```

Use this folder when the topic is cross-cutting runtime/network behavior rather than one local domain crate.

### `adapters/`

Use `adapters/` for external integration layers and backend-specific bridges.

Adapter docs may describe how an adapter consumes domain or engine contracts.

Adapter docs must not define core domain invariants.

### `reports/`

Use `reports/` for evidence and historical analysis.

Recommended structure:

```text
reports/
├── audits/
├── benchmarks/
├── closeouts/
└── decisions/
```

Reports may include:

- audit results;
- benchmark summaries;
- final decision reports;
- closeout reports;
- migration evidence;
- validation captures.

Reports are not doctrine by themselves.

If a report changes architecture, the resulting decision must be promoted into a design document, ADR, or guideline.

### `archive/`

Use `archive/` for non-authoritative historical material.

Archived documents must state one of:

- what replaced them;
- why no replacement exists;
- why they are retained.

Do not use archived docs as implementation guidance unless an active document explicitly references them as historical context.

## Document Types

## Doctrine

Doctrine defines stable rules.

Doctrine belongs in:

```text
content/docs/guidelines/
```

Doctrine may be summarized at repository root.

Examples of doctrine topics:

- architecture layers;
- dependency direction;
- domain ownership;
- module structure;
- testing expectations;
- AI-assisted contribution rules;
- documentation rules.

Doctrine should be written as rules, not suggestions, when the project has already made a decision.

## ADR

An ADR records a decision.

Use an ADR when the decision is:

- architectural;
- long-term;
- expensive to reverse;
- likely to affect multiple crates;
- likely to constrain future implementation;
- important enough that future contributors need to know why alternatives were rejected.

An ADR should normally include:

```text
Status
Context
Decision
Rejected Alternatives
Consequences
```

ADRs should be concise. They should link to larger designs when more explanation is needed.

## Design

A design document explains proposed, accepted, implemented, deferred, rejected, superseded, or archived architecture.

Use a design when the topic affects:

- ownership;
- invariants;
- dependency direction;
- public API shape;
- persistence;
- diagnostics;
- ratification;
- commands;
- inspection;
- schema;
- runtime integration;
- migration;
- validation strategy;
- long-term extensibility.

A design should normally include:

```text
Purpose
Scope
Non-Scope
Architectural Position
Ownership Rules
Dependency Rules
Public API Policy
Invariants
Failure Modes
Diagnostics
Ratification
Commands
Inspection
Persistence and Versioning
Extension Points
Testing Strategy
Migration Plan
Validation Plan
Negative Doctrine
```

Not every design needs every section, but missing sections should be intentional.

## Roadmap

A roadmap explains implementation sequence.

Use a roadmap when the question is:

```text
What should be implemented, in what order, and how do we know each phase is complete?
```

A roadmap should normally include:

```text
Goal
Prerequisites
Phases
Non-Goals
Validation Gates
Completion Criteria
Linked Designs
Linked ADRs
Linked Reports
```

Roadmaps should not define architecture from scratch. They should link to designs and ADRs.

Completed roadmaps should be marked clearly and linked from closeout reports.

## Report

A report records evidence.

Use a report for:

- benchmark results;
- audit findings;
- closeout evidence;
- comparison data;
- migration verification;
- implementation validation;
- final decision evidence.

A report should normally include:

```text
Question
Method
Evidence
Findings
Decision Impact
Linked Roadmap
Linked Design
Linked ADR
```

Reports must not silently become architecture.

If a report changes the architectural direction, update the relevant design, ADR, or guideline.

## Guide

A guide teaches usage or workflow.

Use a guide when the document is meant to help a reader do something.

Guides should have a clear audience:

```text
normal users
advanced users
runtime integrators
contributors
AI-assisted contributors
maintainers
```

Usage guides and architecture docs should remain separate.

## Reference

A reference document describes the current shape of an API, crate, feature map, or capability map.

Reference docs should avoid speculative language unless clearly marked as future work.

Use references for:

- crate capabilities;
- public API summaries;
- feature maps;
- command lists;
- diagnostic code catalogs;
- configuration options.

## Status Model

Every significant docs-site page should include frontmatter with a status.

Allowed status values:

```text
draft
active
accepted
implemented
completed
deferred
superseded
rejected
archived
```

Status meanings:

| Status | Meaning |
|---|---|
| `draft` | Work in progress, not approved |
| `active` | Currently used, discussed, implemented, or validated |
| `accepted` | Approved direction, not necessarily implemented |
| `implemented` | Accepted design has corresponding implementation checked against it |
| `completed` | Roadmap, phase, or report is closed |
| `deferred` | Valid but intentionally postponed |
| `superseded` | Replaced by another document |
| `rejected` | Explicitly considered and not chosen |
| `archived` | Historical and non-authoritative |

## Required Frontmatter

Significant docs-site pages should use frontmatter.

Minimum recommended frontmatter:

```yaml
---
title: Example Title
description: Short description.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-26
---
```

Optional fields:

```yaml
replaces: []
replaced_by: null
related_adrs: []
related_designs: []
related_roadmaps: []
related_reports: []
```

## Frontmatter Fields

### `title`

Human-readable title.

### `description`

Short summary suitable for navigation and search.

### `status`

Lifecycle state.

Must use one of the allowed status values.

### `owner`

The owning conceptual area.

Examples:

```text
workspace
foundation
domain
engine
editor
ui
ecs
net
apps
adapters
```

### `layer`

The architectural layer.

Examples:

```text
workspace
foundation
domain
engine-runtime
app
adapter
test-support
```

### `canonical`

Use `true` when the document is the authoritative source for its topic.

Use `false` for summaries, mirrors, historical material, or secondary references.

### `last_reviewed`

Date of last intentional review.

Use ISO date format:

```text
YYYY-MM-DD
```

### `replaces`

List of documents this document replaces.

### `replaced_by`

Document that replaces this one.

Required for superseded docs when known.

### `related_adrs`

List of ADRs related to this document.

### `related_designs`

List of designs related to this document.

### `related_roadmaps`

List of roadmaps related to this document.

### `related_reports`

List of reports related to this document.

## Source-of-Truth Priority

When documents conflict, use this priority order:

```text
1. Accepted ADRs
2. Guidelines / doctrine
3. Accepted or implemented design documents
4. Active crate/domain/app architecture docs
5. Active roadmaps
6. Reports and closeouts
7. Root summaries
8. Archived material
```

Root summaries are intentionally lower priority than canonical docs-site pages because they are abbreviated.

Root summaries should still never knowingly contradict canonical docs.

## Root Summary Policy

Root markdown files should summarize, not duplicate.

When editing root docs:

1. check whether a docs-site canonical page exists;
2. update the docs-site canonical page first;
3. update the root summary second;
4. keep the root file short;
5. link to the canonical page for detail.

Root docs should be optimized for fast orientation.

Docs-site pages should be optimized for completeness and long-term maintenance.

## Placement Rules

### New repository-wide rule

Place in:

```text
content/docs/guidelines/
```

Then update the relevant root summary if one exists.

### New long-term architectural decision

Place proposed ADR in:

```text
content/docs/adr/proposed/
```

Move to:

```text
content/docs/adr/accepted/
```

only after the decision is accepted.

### New subsystem, crate, or capability design

Place in:

```text
content/docs/design/active/
```

Move to:

```text
content/docs/design/accepted/
```

once approved.

Move to:

```text
content/docs/design/implemented/
```

only after implementation has been checked against the design.

### New rejected design

Place in:

```text
content/docs/design/rejected/
```

A rejected design must explain what to use instead.

### New superseded design

Place in:

```text
content/docs/design/superseded/
```

A superseded design must link to the replacement.

### New deferred design

Place in:

```text
content/docs/design/deferred/
```

A deferred design must explain what would reactivate it.

### New archived design

Place in:

```text
content/docs/design/archived/
```

Use `archived/` only when the document is historical and does not fit `rejected/` or `superseded/`.

Do not use `archived/` as a junk drawer.

### New implementation plan

Place near the owning area if local.

Examples:

```text
content/docs/domain/ecs/roadmaps/
content/docs/apps/runenwerk-editor/roadmap.md
content/docs/net/
```

If the roadmap is repository-wide, link it from:

```text
content/docs/workspace/roadmap-index.md
```

### New benchmark, audit, or closeout evidence

Place in:

```text
content/docs/reports/
```

Use closeout reports for completed phases.

Do not keep completed phase evidence mixed into active domain guidance unless it is explicitly indexed as historical material.

### New crate documentation

Place under the owning subtree:

```text
content/docs/domain/<domain-or-crate>/
content/docs/apps/<app>/
content/docs/net/<area>/
content/docs/adapters/<adapter>/
```

Track missing or thin crate docs in:

```text
content/docs/workspace/crate-docs-status.md
```

### New template

Place in the owning template folder.

Examples:

```text
content/docs/design/templates/
content/docs/adr/templates/
```

Only create a new templates folder when there are multiple expected documents of that kind.

## Design Folder Lifecycle

The design folder uses lifecycle subfolders:

```text
content/docs/design/
├── active/
├── accepted/
├── implemented/
├── deferred/
├── superseded/
├── rejected/
├── archived/
└── templates/
```

### `active/`

Use for designs currently being discussed, implemented, or validated.

A design belongs here when the direction is useful but not yet fully accepted or not yet checked against implementation.

### `accepted/`

Use for designs whose architectural direction is approved.

A design may be accepted before it is fully implemented.

### `implemented/`

Use for accepted designs that have been checked against actual code.

Do not move a design here just because some code exists.

Move only when:

- implementation exists;
- tests pass;
- known design divergences are documented or resolved;
- the design still describes the actual architecture.

### `deferred/`

Use for designs that remain valid but are intentionally postponed.

Deferred designs should explain:

- why they are deferred;
- what would reactivate them;
- what should be done instead for now.

### `superseded/`

Use for designs replaced by newer designs, ADRs, or guidelines.

A superseded design must link to the replacement.

### `rejected/`

Use for designs explicitly considered and not chosen.

Rejected designs should explain:

- the rejected approach;
- why it was rejected;
- what design, ADR, or guideline should be followed instead.

### `archived/`

Use for historical or imported design material that is no longer authoritative and does not fit `rejected/` or `superseded/`.

Archived designs must link to the replacement document or explain why no replacement exists.

Do not use `archived/` for rejected or superseded designs.

### `templates/`

Use for reusable design templates.

Templates should be generic and should not contain project-specific decisions unless they are part of the documentation standard.

## ADR Lifecycle

Recommended ADR structure:

```text
content/docs/adr/
├── README.md
├── proposed/
├── accepted/
├── superseded/
└── rejected/
```

### `proposed/`

Use for decisions under consideration.

### `accepted/`

Use for binding decisions.

Accepted ADRs have high source-of-truth priority.

### `superseded/`

Use for decisions replaced by newer ADRs.

Superseded ADRs must link to the replacement.

### `rejected/`

Use for decisions that were considered and explicitly not chosen.

Rejected ADRs should explain why the decision was rejected and what was chosen instead.

## Naming Rules

Use kebab-case filenames for docs-site files.

Preferred:

```text
foundation-diagnostics.md
foundation-ratification.md
crate-design-template.md
documentation-structure.md
root-docs-map.md
first-3d-editor-mvp.md
```

Avoid:

```text
foundation_diagnostics.md
FoundationDiagnostics.md
new doc.md
misc.md
notes.md
design.md
```

Exceptions:

- numbered index pages may use numeric prefixes when helpful;
- existing files may be migrated gradually;
- external generated files may keep their generated names if renaming would break tooling.

## Title Rules

Document titles should be clear and specific.

Preferred:

```text
Foundation Diagnostics Design
Runenwerk Editor MVP Acceptance Criteria
ECS Phase 6 Final Decision Report
```

Avoid:

```text
Design
Notes
Misc
Plan
Stuff
```

## Link Rules

Use relative links inside docs-site.

When moving a document:

1. update inbound links;
2. update local README/index pages;
3. update root summary links if relevant;
4. leave a compatibility stub if the old path is widely referenced;
5. record the move in the relevant index if needed.

Do not move many documents at once without first creating index pages and lifecycle folders.

## Compatibility Stubs

When a document is moved and the old path may be referenced, leave a short stub.

Example:

```markdown
---
title: Moved Document
status: superseded
canonical: false
replaced_by: ../workspace/new-path-example.md
---

# Moved

This document moved to `../workspace/new-path-example.md`.
```

Use stubs temporarily. Remove them only after links and references are stable.

## Roadmap Policy

Roadmaps define execution order, not architecture.

A roadmap should link to:

- relevant ADRs;
- relevant designs;
- validation commands;
- closeout reports once complete.

Roadmaps should avoid duplicating long design sections.

A roadmap may summarize design context, but the design remains canonical for architecture.

Completed roadmaps should be marked `completed` and linked to closeout reports.

## Report Policy

Reports record evidence.

Reports should not be used as the source of long-term rules.

A report may conclude that a phase is complete or that a tradeoff is acceptable, but if the result changes architecture, update the relevant ADR, design, or guideline.

Completed engineering phase evidence should live under:

```text
content/docs/reports/closeouts/
```

Benchmarks should live under:

```text
content/docs/reports/benchmarks/
```

Audits should live under:

```text
content/docs/reports/audits/
```

## Crate Documentation Policy

Each important crate should have enough docs to explain:

```text
Purpose
Scope
Non-Scope
Ownership
Allowed Dependencies
Forbidden Dependencies
Public API
Invariants
Failure Modes
Testing
Extension Points
```

For small crates, a concise README may be enough.

For architectural crates, add or link a design document.

Track missing or thin crate docs in:

```text
content/docs/workspace/crate-docs-status.md
```

When adding, removing, renaming, or substantially changing a crate, update:

```text
Cargo.toml
CRATES.md
DOMAIN_MAP.md
content/docs/workspace/crate-docs-status.md
```

Also update any owning domain/app docs.

## Domain Documentation Policy

Domain docs should make ownership explicit.

Each domain area should explain:

- what it owns;
- what it does not own;
- what invariants it protects;
- what crates consume it;
- what crates it may depend on;
- how it is validated;
- what extension points exist.

Domain docs should not read like generic templates.

Remove placeholder examples once a domain has real documentation.

## App Documentation Policy

Application docs may combine product scope and architecture, but large docs should be split when they carry multiple responsibilities.

Split app docs when one file contains distinct concerns such as:

```text
product scope
acceptance criteria
implementation sequence
architecture
roadmap
post-MVP expansion
```

Example structure:

```text
content/docs/apps/runenwerk-editor/
├── README.md
├── mvp/
│   ├── first-3d-editor-mvp.md
│   ├── acceptance-criteria.md
│   └── implementation-sequence.md
└── roadmap.md
```

## Template Policy

Templates define expected structure.

Templates should avoid project-specific decisions unless those decisions are part of the documentation standard.

Templates should be easy to copy and fill out.

Templates should not be treated as accepted architecture.

## Archive Policy

Archived documents are non-authoritative.

Every archived document must include one of:

```text
replaced_by
reason retained
historical context
```

Do not archive documents that should be rejected or superseded.

Use the most precise lifecycle state.

## Supersession Policy

When superseding a document:

1. move the old document to the relevant `superseded/` folder, or mark it `superseded`;
2. add `replaced_by`;
3. link to the replacement;
4. update indexes and local READMEs;
5. update any root summary if relevant.

Do not silently leave stale docs in active locations.

## Rejection Policy

Rejected documents are useful when the rejected alternative is likely to come up again.

A rejected design or ADR should explain:

- what was proposed;
- why it was rejected;
- what to use instead;
- which ADR/design/guideline owns the final direction.

Do not delete rejected design rationale if it prevents future churn.

## Deferred Policy

Deferred documents describe valid work that is intentionally postponed.

A deferred document should explain:

- why now is not the right time;
- what dependency or condition would reactivate it;
- what should be done instead;
- whether the idea is still architecturally compatible.

Deferred does not mean rejected.

## Implemented Design Policy

A design can move to `implemented/` only after it has been checked against code.

Before marking a design implemented, verify:

- the relevant crate or subsystem exists;
- the public API shape matches the design or divergences are documented;
- tests validate core invariants;
- dependency rules are respected;
- integration points are documented;
- the design no longer describes purely future work.

If code diverges from the design, update the design or create a superseding design.

Do not use `implemented/` as a trophy folder.

Use it only for designs that remain useful as living architecture references.

## Documentation Review Checklist

Before adding a document, ask:

- What type of document is this?
- Who owns it?
- Is it canonical?
- Is there already a canonical document for this topic?
- Does it belong in root or docs-site?
- Does it belong near a domain/app/crate?
- Is it doctrine, ADR, design, roadmap, report, guide, reference, or archive?
- Does it need frontmatter?
- Does it need a status?
- Does it link to related ADRs or designs?
- Does it duplicate existing docs?
- Does it change architecture?
- Does it need validation commands?

Before moving a document, ask:

- What links will break?
- Is a compatibility stub needed?
- Should old material be superseded, rejected, deferred, archived, or completed?
- Does an index need updating?
- Does a root summary need updating?

Before deleting a document, ask:

- Is it stale or still useful as historical evidence?
- Should it be superseded instead?
- Should it be archived instead?
- Is there a replacement link?
- Would deleting it remove rejected-alternative rationale?

## Documentation Update Triggers

Update documentation when you:

- add a crate;
- remove a crate;
- rename a crate;
- move a domain concept;
- change dependency direction;
- add a foundational abstraction;
- add or change a public API contract;
- add a new command family;
- add a new diagnostic family;
- add a new ratification boundary;
- change persistence format;
- change editor/app MVP scope;
- complete a roadmap phase;
- close out benchmark/audit work;
- supersede a design;
- accept or reject an ADR.

## Refactor Documentation Update Matrix

Use this matrix when code, architecture, or documentation is refactored.

The goal is to keep canonical docs, root summaries, crate indexes, and validation commands aligned after structural changes.

| Change type | Required documentation updates |
|---|---|
| Add crate | Update `Cargo.toml`, root `CRATES.md`, root `DOMAIN_MAP.md`, `content/docs/workspace/crate-docs-status.md`, and the owning docs subtree. |
| Remove crate | Update `Cargo.toml`, root `CRATES.md`, root `DOMAIN_MAP.md`, `content/docs/workspace/crate-docs-status.md`, and archive or supersede old crate docs. |
| Rename crate | Update `Cargo.toml`, root `CRATES.md`, root `DOMAIN_MAP.md`, `content/docs/workspace/crate-docs-status.md`, all internal links, and the owning docs subtree. |
| Split crate | Update `Cargo.toml`, root `CRATES.md`, root `DOMAIN_MAP.md`, `content/docs/workspace/crate-docs-status.md`, dependency docs if boundaries changed, and any affected design docs. |
| Move concept between domains | Update root `DOMAIN_MAP.md`, affected domain docs, affected design docs, and add or update an ADR if the ownership change is long-term architecture. |
| Change dependency direction | Update root `DEPENDENCY_RULES.md`, relevant guideline docs, affected crate/domain docs, and add or update an ADR if the rule is long-term. |
| Add public API | Update owning crate/domain docs, usage guide or examples if user-facing, and tests if the API protects an invariant. |
| Change public API | Update owning crate/domain docs, usage guide or examples, migration notes if breaking, and tests/examples that show the old API. |
| Add architectural rule | Update `content/docs/guidelines/`, then align the relevant root summary if one exists. |
| Make long-term architectural decision | Add or update an ADR under `content/docs/adr/`. |
| Add subsystem or capability design | Add a design under `content/docs/design/active/`. Move it to `accepted/` only after the direction is approved. |
| Mark design implemented | Move the design to `content/docs/design/implemented/` only after code has been checked against the design and divergences are resolved or documented. |
| Complete roadmap or phase | Mark the roadmap or phase completed, add closeout evidence under `content/docs/reports/closeouts/`, and update active roadmap indexes. |
| Add benchmark evidence | Put human-readable reports under `content/docs/reports/benchmarks/` or `content/docs/reports/closeouts/`; keep raw benchmark artifacts outside prose docs. |
| Rename docs file | Update inbound links, local README/index pages, root summary links if relevant, and leave a compatibility stub if the old path is widely referenced. |
| Move docs file | Update inbound links, local README/index pages, root summary links if relevant, and record the new canonical location. |
| Delete docs file | Delete only if the document has no historical value; otherwise move it to `superseded/`, `rejected/`, `archived/`, or `reports/` as appropriate. |
| Change validation command | Update root `TESTING.md`, relevant workspace validation docs, and any routines that call the command. |
| Change AI/contributor workflow | Update root `AGENTS.md`, root `AI_GUIDE.md` if AI-specific, and the relevant workspace routine docs. |

After applying any row in this matrix, run the narrowest relevant validation command.

For documentation-only refactors, prefer:

```text
python3 tools/docs/validate_docs.py
```
If the command is unavailable or incomplete, state that validation could not be run and document what remains unverified.

## Validation

When documentation structure changes, run the repository docs validation command if available.

Recommended command:

```text
python tools/docs/validate_docs.py
```

If that script is missing or incomplete, update this section after the docs tooling is formalized.

For code-related documentation changes, also run the relevant crate validation commands documented by the owning crate or root testing guide.

## Negative Doctrine

Do not scatter documentation opportunistically.

Do not put full designs in root markdown files.

Do not duplicate docs-site pages into root summaries.

Do not leave completed roadmaps mixed with active implementation plans without status labels.

Do not let reports become implicit architecture.

Do not archive rejected or superseded documents when more precise lifecycle states exist.

Do not create new top-level docs-site folders unless the existing taxonomy cannot express the document type.

Do not use `misc`, `notes`, or `old` folders as long-term structure.

Do not keep placeholder text in canonical docs.

Do not let examples contradict current APIs.

Do not use human-readable display strings as machine-readable documentation keys.

Do not describe future plans as implemented facts.

Do not silently rename files without updating links.

## Current Refactor Priorities

The first taxonomy cleanup has established:

1. ADR lifecycle folders and `adr/README.md`.
2. Top-level compatibility stubs for moved `multiplayer/` and `templates/` content.
3. Architecture designs moved out of `guidelines/` into `design/active/`.
4. Completed viewport phase evidence moved into `reports/closeouts/`.
5. Workspace implementation sequencing moved into `workspace/roadmap-index.md`.

Continue documentation cleanup in this order:

1. Finish remaining `design/` lifecycle consistency.
2. Split large app MVP documents by responsibility only if they still mix concerns.
3. Move future completed benchmark and closeout evidence into `reports/`.
4. Clean obvious stale or placeholder domain docs.
5. Update root summaries only after canonical docs-site changes are stable.

The goal is controlled convergence, not a disruptive reshuffle.

## Final Rule

Use the narrowest accurate document type and the nearest owning folder.

When in doubt:

```text
rules go in guidelines
decisions go in adr
architecture goes in design
implementation sequence goes in roadmaps
evidence goes in reports
usage goes in guides or owning domain/app docs
historical non-authoritative material goes in archive
root docs summarize only
```
