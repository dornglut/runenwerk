---
title: Documentation Refactor Routine
description: Bounded routine for safely moving, renaming, pruning, and restructuring Runenwerk documentation.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
related_docs:
  - ../documentation-structure.md
---

# Documentation Refactor Routine

## Purpose

Use this routine when moving, renaming, pruning, or restructuring Runenwerk documentation.

The goal is to keep documentation changes controlled, source-of-truth aligned, link-safe, and easy to verify.

This routine exists for humans and AI coding agents.

## Preconditions

Before editing documentation structure:

1. Read `docs-site/src/content/docs/workspace/documentation-structure.md`.
2. Read `AGENTS.md`.
3. Identify all affected files before editing.
4. Classify each affected document by type.
5. Do not perform broad speculative reshuffles.
6. Do not rename or move files only for aesthetics.
7. Prefer the smallest coherent change set.

## Document Classification

Classify each affected document as one of:

```text
doctrine
ADR
design
roadmap
report
guide
reference
template
archive
root summary
```

Use the narrowest accurate document type.

If a document combines several types, split it only when the split improves ownership, lifecycle clarity, or maintenance.

Bounded Repair Loop

Run at most 3 repair passes.

Do not use an unbounded loop.

Pass Steps

For each pass:

Inspect the affected files.
Determine the owning folder for each document.
Apply one coherent patch.
Update all internal links affected by the patch.
Update local README or index pages.
Update root summaries when canonical docs-site content changes.
Run the narrowest relevant validation command.
Repair only issues found by validation or direct review.
Stop Conditions

Stop when one of these is true:

validation passes;
maximum 3 repair passes have been reached;
a semantic conflict requires a human decision;
a move would require broad unrelated changes.

If the routine stops before validation passes, report the remaining issue clearly.

Placement Rules

Use these placement rules during the routine:

rules go in guidelines
decisions go in adr
architecture goes in design
implementation sequence goes in roadmaps
evidence goes in reports
usage goes in guides or owning domain/app docs
historical non-authoritative material goes in archive
root docs summarize only
Common Refactor Cases
Rename docs file

When renaming a docs file:

Rename the file.
Update inbound links.
Update local README or index pages.
Update root summary links if relevant.
Leave a compatibility stub if the old path is widely referenced.
Move docs file

When moving a docs file:

Confirm the document type and lifecycle state.
Move it to the nearest owning folder.
Update inbound links.
Update local README or index pages.
Update root summary links if relevant.
Record the new canonical location if the document was important.
Delete docs file

Delete only when the document has no current or historical value.

Otherwise use the precise lifecycle state:

superseded
rejected
archived
completed

Do not use archive/ for rejected or superseded documents.

Split large docs file

Split a large doc only when it mixes distinct responsibilities.

Common split reasons:

product scope
architecture
implementation sequence
acceptance criteria
roadmap
historical report
usage guide

Each split document must have one clear owner and one clear lifecycle state.

Move completed evidence

Completed phase evidence belongs under:

docs-site/src/content/docs/reports/closeouts/

Benchmark reports belong under:

docs-site/src/content/docs/reports/benchmarks/

Raw benchmark artifacts should not be mixed into prose docs.

Required Validation

For documentation-only refactors, prefer:

task docs:validate

If the repository currently uses python3, use:

task docs:validate

If validation cannot be run, state that explicitly.

Validation Checklist

Before finishing, verify:

moved files have updated links;
renamed files have updated links;
significant docs have frontmatter;
lifecycle folder matches status where applicable;
root summaries still agree with canonical docs;
crate inventory docs still agree with Cargo.toml;
reports are not treated as doctrine;
archived docs are clearly non-authoritative;
no new misc, notes, old, or catch-all docs folders were introduced.
Required Final Report

The final response must include:

changed files
moved files
deleted or pruned files
compatibility stubs created
validation commands run
validation result
remaining risks or unverified items
Negative Doctrine

Do not run broad cleanup while performing a narrow docs fix.

Do not silently change architecture while moving docs.

Do not hide unresolved link or validation failures.

Do not create new top-level docs folders unless existing taxonomy cannot express the document type.

Do not use an unbounded loop.

Do not treat reports, benchmarks, or closeouts as source-of-truth architecture.