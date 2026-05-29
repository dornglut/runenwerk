---
title: Tauri Project Docs Starter
description: Lightweight documentation starter for preserving reusable software-development practices in a new Tauri application.
status: active
owner: workspace
layer: software-development
canonical: true
last_reviewed: 2026-05-29
related:
  - ./README.md
  - ./principles.md
  - ../guidelines/authority-centered-boundary-architecture.md
---

# Tauri Project Docs Starter

Use this guide when starting a new Tauri application and you want to preserve
the useful software-development practices from Runenwerk without carrying over
Runenwerk-specific governance, roadmap machinery, or product architecture.

The goal is a compact starter docs set that protects long-term quality:
ownership, contracts, validation, diagnostics, security, migrations, testing,
public API usability, and honest closeout.

## Carry-Over Rule

Carry over principles, not bureaucracy.

Preserve:

- source-of-truth ownership;
- frontend/backend boundary contracts;
- explicit Tauri IPC command contracts;
- validation before acceptance;
- diagnostics and stable errors;
- security and capability policy;
- persisted data versioning;
- behavior-focused tests;
- ADRs for durable decisions;
- lightweight closeout evidence.

Do not preserve:

- Runenwerk production tracks;
- roadmap scoring;
- SDF, renderer, ECS, editor, or Workbench-specific architecture;
- large closeout trees;
- heavy routine machinery before the project needs it.

## Recommended Docs Tree

Start with this:

```text
AGENTS.md
docs/
|-- README.md
|-- software-development/
|   `-- principles.md
|-- architecture.md
|-- tauri-boundaries.md
|-- ipc-command-contracts.md
|-- persistence.md
|-- frontend-state.md
|-- security-and-capabilities.md
|-- diagnostics.md
|-- migration.md
|-- testing.md
|-- adr/
|   |-- README.md
|   `-- template.md
`-- closeouts/
    `-- README.md
```

Add these later only when the project grows:

```text
docs/
|-- routines/
|   |-- implementation.md
|   |-- refactor.md
|   `-- docs-refactor.md
|-- public-api-review.md
|-- release-closeout.md
`-- roadmap.md
```

## Minimum Docs

### `AGENTS.md`

Contributor and AI-agent rules for the project root.

Include:

- read `docs/README.md` before broad changes;
- inspect existing code before editing;
- preserve unrelated dirty work;
- prefer long-term fixes over surface patches;
- run the smallest meaningful validation;
- report changed files, validation, and remaining risks.

### `docs/README.md`

The docs map.

Include:

- links to architecture, Tauri boundaries, testing, ADRs, and software
  development principles;
- which docs are authoritative;
- where new docs belong.

### `docs/software-development/principles.md`

The project-neutral engineering guide.

Recommended source:

- `docs-site/src/content/docs/software-development/principles.md` from
  Runenwerk, copied into the new project and trimmed as needed.

### `docs/architecture.md`

The source-of-truth and dependency-direction document.

Include:

- frontend state is derived unless explicitly designed otherwise;
- Rust app core owns domain validation and persistence decisions;
- Tauri commands are adapter boundaries, not domain owners;
- database and filesystem are storage, not authority;
- dependency direction runs from shared contracts and core logic outward to UI,
  Tauri adapters, plugins, and external services.

### `docs/tauri-boundaries.md`

The Tauri-specific boundary map.

Use this generic model:

```text
frontend UI
  -> proposes commands and displays projections

Tauri commands
  -> IPC adapter boundary

Rust app core
  -> owns validation, state transitions, persistence policy, diagnostics

database/filesystem
  -> storage, not authority

frontend stores/caches
  -> derived UI state unless explicitly authoritative
```

### `docs/testing.md`

The validation and test strategy.

Include:

- frontend unit/component tests;
- Rust unit/domain tests;
- Tauri command contract tests;
- persistence migration tests;
- frontend projection tests;
- smoke tests;
- behavior-based test names;
- local validation commands.

### ADR Index And Template

Use ADRs only for durable decisions.

Create an ADR when a decision changes:

- source-of-truth ownership;
- frontend/backend boundary shape;
- persisted data format;
- security policy;
- long-term dependency direction;
- migration strategy;
- major framework or library choice.

## Tauri-Specific Docs

### `docs/ipc-command-contracts.md`

Treat every Tauri command as a boundary contract.

For each command family, document:

- command name;
- caller intent;
- input DTO;
- output DTO;
- validation owner;
- permission or capability requirement;
- failure modes;
- diagnostic codes;
- persistence effects;
- frontend projection updates.

### `docs/persistence.md`

Document storage without making storage the authority by accident.

Include:

- data stores and file locations;
- schema or format versions;
- migration path;
- backup/export/import policy;
- corruption handling;
- last-good or rollback behavior;
- what owns semantic validity.

### `docs/frontend-state.md`

Document which frontend state is derived and which state, if any, is
authoritative.

Include:

- route state;
- UI form drafts;
- optimistic updates;
- caches;
- server/Rust projections;
- local-only preferences;
- stale-data behavior;
- invalidation rules.

### `docs/security-and-capabilities.md`

Tauri apps need explicit security posture early.

Include:

- allowed Tauri plugins;
- filesystem access policy;
- shell/process/network access policy;
- command allowlist;
- capability checks;
- denied-action diagnostics;
- fail-closed rules;
- secrets handling.

### `docs/diagnostics.md`

Define user-visible and developer-visible diagnostics.

Include:

- stable error codes;
- user-facing messages;
- developer context;
- log levels;
- telemetry policy if any;
- privacy constraints;
- where diagnostics surface in the UI.

### `docs/migration.md`

Preserve app data across versions.

Include:

- persisted format versions;
- migration ordering;
- rollback or backup policy;
- migration diagnostics;
- compatibility windows;
- tests required before release.

## Suggested Project Shape

One practical Tauri app structure:

```text
src-tauri/
  src/
    app_core/
      mod.rs
      commands/
      diagnostics/
      persistence/
      state/
    tauri_adapter/
      mod.rs
      commands.rs
      capabilities.rs
src/
  features/
  routes/
  state/
  api/
  components/
docs/
  ...
```

The exact folders can change. Preserve the ownership model:

- frontend owns presentation, local interaction, and projections;
- Tauri adapter owns IPC mapping and host integration;
- Rust app core owns invariants, validation, persistence policy, and state
  transitions;
- storage persists decisions but does not decide semantic validity.

## Preservation Workflow

Use this process to preserve the guidance cleanly:

1. Copy this starter into `docs/tauri-project-docs-starter.md` or split it into
   the docs tree above.
2. Copy `software-development/principles.md` and trim examples that do not fit
   the new project.
3. Create a short `AGENTS.md` that points contributors to `docs/README.md`,
   `docs/architecture.md`, `docs/tauri-boundaries.md`, and `docs/testing.md`.
4. Write the first ADR for the app boundary decision:
   frontend projection, Tauri IPC adapter, Rust app core authority, storage as
   persistence.
5. Add validation commands to `docs/testing.md` as soon as the project has a
   build and test setup.
6. Keep closeouts lightweight: one Markdown file per significant phase or
   release is enough at first.
7. Promote repeated lessons into the maintained docs. Keep one-off evidence in
   closeouts or reports.

## First ADR Suggestion

Recommended first ADR in the new project's ADR folder:

```text
adr/0001-tauri-boundary-and-source-of-truth.md
```

Decision summary:

```text
The frontend proposes user intent and renders derived projections.
Tauri commands are IPC adapters.
The Rust app core owns validation, state transitions, diagnostics, and
persistence policy.
The database and filesystem persist data but do not decide semantic validity.
```

This one decision prevents most early Tauri architecture drift.
