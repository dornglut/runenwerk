---
title: Phase Completion Drift Check Routine
description: Mandatory end-of-phase completeness and drift check for phased Runenwerk work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-28
related_docs:
  - ../prompt-templates/phase-completion-drift-check.md
  - ./README.md
---

# Phase Completion Drift Check Routine

Use this routine after every phased implementation before starting the next phase.

The routine is documentation and workflow maintenance. It must not start the next implementation phase.

## Inputs

Inspect:

- the accepted design or roadmap for the phased work;
- the implemented code and tests for the just-finished phase;
- workspace manifests and feature declarations;
- crate inventory and root docs;
- docs-site design docs and roadmaps;
- relevant routines and prompt templates;
- validation output from the completed phase.

Treat current code as source of truth. If documentation and code disagree, preserve working code and record the mismatch precisely.

## Checks

### Phase Completeness

Ask whether the just-finished phase implemented exactly what the accepted design promised:

- expected files, modules, features, and public APIs;
- expected tests and feature-matrix coverage;
- expected validation commands;
- no missing promised deliverables.

### Phase Drift

Identify differences between implementation and the phase design:

- changed names, feature flags, public APIs, or file layout;
- omitted or added tests;
- changed validation plan;
- implementation details that were necessary but not described.

Keep useful drift. Do not rewrite working code only to match stale prose.

### Overall Roadmap Drift

Check whether repository docs still match current truth:

- root docs such as `AGENTS.md`, `AI_GUIDE.md`, `ARCHITECTURE.md`, `CRATES.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `GLOSSARY.md`, and `TESTING.md`;
- docs-site design docs;
- roadmap/status text;
- routine indexes;
- prompt-template indexes;
- next-step recommendations.

### Deferred-Work Integrity

Confirm later-phase work did not start early and docs do not imply that it did.

Name:

- the next phase;
- what must not be done yet;
- which later-phase items remain deferred.

### Validation Completeness

Confirm whether validation ran:

- focused tests for the changed crate or domain;
- feature-matrix tests when features changed;
- `python3 tools/docs/validate_docs.py`;
- `./quiet_full_gate.sh` when code or workspace behavior changed.

If any command was skipped, record the reason.

## Required Patch Actions

After every completed phase:

1. update phase status in the owning design or roadmap doc;
2. update current-state docs if implemented repo state changed;
3. update routine or prompt docs if the workflow learned a reusable rule;
4. explicitly name the next phase;
5. explicitly name what must not be done yet;
6. preserve deferred-work boundaries;
7. run docs validation;
8. run the quiet full gate when code or workspace behavior changed.

Do not perform unrelated opportunistic rewrites.

## Output Format

Report findings first, ordered by severity.

For each finding, include:

- exact file path and section;
- exact stale or incorrect statement found;
- correction made or reason it was left unchanged.

Then report:

- files and sections changed;
- phase status after the pass;
- next phase;
- explicit not-yet list;
- validation commands and results.

## Stop Conditions

Stop when:

- stale phase status and roadmap drift have been corrected;
- deferred work is clearly marked;
- validation has passed or failures are reported with concrete output;
- no unrelated docs were rewritten.
