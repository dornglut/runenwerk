---
title: AI Agent Boundaries
description: Boundaries for AI-assisted work in Runenwerk.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ../../../AGENTS.md
  - ./operating-model.md
  - ./authority-model.md
---

# AI Agent Boundaries

This is a reference page. The root AI entrypoint is `AGENTS.md`.

## Rule

AI-assisted work uses the same repository contracts as humans, tests, editor tools, and scripts. There is no privileged AI mutation path.

AI may inspect, propose, patch, summarize, and report evidence. It must not bypass domain ownership, accepted ADR/design gates, dependency rules, validation, or closeout evidence.

## Placement

Runtime AI integrations belong in:

```text
apps/
tools/
adapters/
```

Do not add LLM clients, prompts, autonomous agents, or workflow-specific AI policy to `foundation/` or pure `domain/` crates.

## Connector mode

When command execution is unavailable:

- inspect files by exact path;
- name authority files used as evidence;
- patch only scoped files;
- use manual validation checklists;
- report command validation as unavailable;
- stop when required authority cannot be inspected.

## Concept ownership

Use `DOMAIN_MAP.md` for concept placement. Do not duplicate the concept map here.
