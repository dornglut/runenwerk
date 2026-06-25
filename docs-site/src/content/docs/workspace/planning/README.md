---
title: Planning Records
description: Markdown-first planning records for scriptless Runenwerk workflow.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ../authority-model.md
  - ../routines/roadmap-update-routine.md
---

# Planning Records

Planning is Markdown-first from the scriptless workflow cutover onward.

Use these files:

- [Active Work](active-work.md)
- [Roadmap](roadmap.md)
- [Deferred Work](deferred-work.md)
- [Completed Work](completed-work.md)
- [Production Tracks](production-tracks.md)
- [Decision Register](decision-register.md)

## Rule

A planning change must be understandable by reading Markdown. Do not require generated views, rendered diagrams, YAML parsing, Taskfile tasks, or local scripts to know the current state.

Legacy YAML and generated Markdown may remain as migration context or optional mirrors. If they disagree with these planning records, update the Markdown planning record first and report the mirror drift.

## Update checklist

- Active work has one clear current focus or an explicit no-current-focus state.
- Deferred work names the reason and reactivation condition.
- Completed work links evidence.
- Production tracks name strategic order and gates.
- Decision register explains priority changes.
