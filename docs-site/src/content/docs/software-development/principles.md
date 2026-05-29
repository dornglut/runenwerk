---
title: General Software Development Principles
description: Reusable software development principles generalized from Runenwerk's architecture, docs, ADRs, routines, validation, and closeout practices.
status: active
owner: workspace
layer: software-development
canonical: true
last_reviewed: 2026-05-29
related:
  - ./README.md
  - ../guidelines/authority-centered-boundary-architecture.md
  - ../reports/audits/general-software-development-principles-extraction-2026-05-29.md
---

# General Software Development Principles

This guide is the maintained, reusable version of the software-development
principles extracted from Runenwerk's docs. It is intentionally project-neutral:
use it for architecture review, implementation planning, code review, docs
organization, testing strategy, and closeout in any long-lived software system.

For the deeper architecture doctrine, start with
[Authority-Centered Boundary Architecture](../guidelines/authority-centered-boundary-architecture.md).

## Core Model

Start from boundaries, not patterns:

```text
authority
invariants
contracts
flows
policy
time
consistency
storage
execution
failure
observation
evolution
cost
```

Then choose the lightest implementation pattern that protects those boundaries.

## Principles

### 1. Put Truth In An Owner

Every meaningful invariant needs an owning authority, domain, module, service,
or bounded context. Storage, UI, transport, and automation do not become
authority merely because they touch the data.

```text
Usage does not imply ownership.
The owner is where invariants are defined and enforced.
```

### 2. Keep Dependency Direction Simple

Separate stable core rules from runtime wiring, app composition, adapters,
tools, UI, transport, persistence, and vendor integrations.

Generic dependency direction:

```text
shared vocabulary -> domain/core logic -> runtime/orchestration -> apps/adapters/tools
```

Concrete folder names can differ by project. The important rule is that stable
policy and domain contracts must not depend on outer implementation details.

### 3. Make Boundaries Speak Contracts

Cross-boundary communication should use explicit contracts:

- function signatures;
- interfaces or traits;
- commands;
- queries;
- events;
- DTOs;
- schemas;
- persisted file formats;
- protocol messages;
- derived product descriptors.

If one subsystem needs another subsystem's internals, first ask whether a DTO,
command, query, schema, validator, contract package, or test-support boundary is
missing.

### 4. Route Important Mutations Through A Change Boundary

Important state changes should not happen through arbitrary direct mutation.
Use a named mutation path that validates intent near the invariants it affects.

Useful forms include:

- command proposals;
- command handlers;
- builders;
- import pipelines;
- transactions;
- migrations;
- controlled reducers.

Keep concrete command families close to the authority that understands their
meaning. Avoid one universal command model for unrelated domains.

### 5. Validate External And Generated State Before Acceptance

Generated, imported, migrated, projected, AI-assisted, or externally modified
state is a candidate, not truth. The owning authority should validate it before
it becomes authoritative.

Good acceptance reports distinguish:

- accepted;
- accepted with warnings;
- rejected;
- fatal or structurally unsafe.

### 6. Separate Descriptions From Execution

Many systems need both an editable description and an optimized runtime object.
Do not collapse them unless the system is truly tiny.

Examples:

```text
workflow definition -> workflow execution
scene document -> loaded scene
query descriptor -> prepared query plan
schema descriptor -> validator/runtime binding
UI definition -> mounted surface instance
build graph -> job execution plan
```

Descriptions should be inspectable, diffable, versioned, testable, and
migratable. Execution objects may be backend-aware, resource-owning, optimized,
and transient.

### 7. Treat Projections And Caches As Derived State

Read models, UI view models, route maps, search indexes, cached products,
render packets, diagnostics tables, and previews should remain rebuildable
unless a design explicitly promotes them to authority.

If a projection becomes mutable source truth by accident, the system now has two
truths and will drift.

### 8. Separate Policy From Validity

Policy answers whether an actor is allowed to request something. Domain
validation answers whether the requested change is semantically valid.

```text
Policy:
  this user may edit this document

Domain validation:
  this document edit preserves the model invariants
```

Security, permission, feature-flag, host, and environment gates should fail
closed when uncertainty would be unsafe.

### 9. Make Failure Semantics Part Of The Contract

Every serious boundary should name how failure behaves:

- reject;
- retry;
- rollback;
- compensate;
- degrade;
- queue;
- preserve last-good output;
- fail closed;
- panic only for impossible internal bugs.

Silent success-shaped failures are architecture debt. If a caller cannot tell
whether work was accepted, rejected, stale, or degraded, the contract is
incomplete.

### 10. Treat Diagnostics As Product Surface

Diagnostics should be stable enough for humans, tests, tools, automation, and
future UI surfaces to consume.

Prefer:

- stable diagnostic codes;
- precise subjects;
- severity;
- context;
- actionable messages;
- links to owning docs where useful.

Diagnostics matter most around validation, imports, migrations, policy
failures, generated state, distributed work, and async jobs.

### 11. Version Durable Contracts From The Start

Persisted formats, schemas, command contracts, protocols, migration inputs, and
generated products should have stable identity and versioning before they are
shared broadly.

Use version `1` for the first real public contract. Avoid unversioned
"temporary" formats that are likely to become permanent through usage.

### 12. Organize Code By Responsibility

Prefer modules, packages, and folders that answer:

```text
What concept owns this code?
Which subsystem is responsible for this behavior?
```

Avoid catch-all files and modules such as `utils`, `helpers`, `misc`, or
`_internal` when a real responsibility name exists.

Good names usually describe ownership or behavior:

```text
routing
diagnostics
migration
publication
inspection
schema
commands
projection
```

### 13. Reuse Before Abstracting

Before adding an abstraction, search for existing helpers, patterns, tests,
docs, and neighboring implementations.

Add a new abstraction only when it:

- protects a real boundary;
- removes meaningful duplication;
- improves public API clarity;
- reduces future drift;
- matches a repeated local pattern.

Avoid universal objects, global registries, and over-broad extension points that
turn multiple owners into one vague owner.

### 14. Design For Migration, Not Only Growth

Long-lived systems need paths for promotion, demotion, splitting, merging,
inlining, deleting, deprecating, and replacing.

For replacement work, prefer a strangler-style migration:

1. Freeze the old path behind a named compatibility boundary.
2. Add the new path beside it.
3. Route one caller or product slice through the new path.
4. Prove parity or source-marker correctness.
5. Switch remaining callers.
6. Delete the old path and keep regression guards.

### 15. Treat Tests As Executable Architecture

Tests should protect invariants, not only examples of current behavior.

Useful test categories:

- unit tests;
- domain invariant tests;
- command behavior tests;
- validation or ratification tests;
- projection golden tests;
- schema compatibility tests;
- migration tests;
- smoke tests;
- architecture guard tests;
- end-to-end evidence tests for product-visible behavior.

Prefer behavior-based names that describe the invariant being protected.

### 16. Use Fitness Functions For Important Boundaries

If a boundary is important enough to document, consider whether a test, script,
lint, metadata check, schema validation, doc validation, or CI gate can enforce
it.

Examples:

- dependency-direction checks;
- stale-link documentation checks;
- generated-doc freshness checks;
- architecture guard tests;
- public API examples that compile;
- validation gates for planning metadata.

Fitness functions turn architecture from prose into a recurring check.

### 17. Treat Public APIs As A Usability Surface

A technically correct public API can still be defective if normal users cannot
discover or combine it.

Review public APIs from:

- package root exports;
- prelude or common imports;
- README;
- usage guide;
- examples;
- docs index;
- error messages and diagnostics.

Teach the happy path first. Keep advanced, feature-heavy, or domain-specific API
behind explicit modules unless it is needed for normal use.

### 18. Keep Documentation Typed By Purpose

Different documentation types should not collapse into one file:

- guidelines define stable rules;
- ADRs record durable decisions and rejected alternatives;
- design docs explain target architecture and tradeoffs;
- roadmaps sequence implementation;
- routines define repeatable execution steps and stop conditions;
- prompt templates provide reusable task starts;
- closeouts record evidence after work completes;
- reports preserve audits, findings, and historical analysis;
- usage guides teach normal users how to succeed.

When a long-term rule appears in a roadmap, closeout, or implementation plan,
promote it into a guideline or ADR instead of burying it.

### 19. Make Closeout Honest

Completion should say what is actually proven, not what the long-term product
eventually wants to become.

Useful completion tiers:

- bounded contract complete;
- runtime or production evidence complete, with known quality gaps;
- fully verified, with no known quality gaps.

Always record:

- changed files and modules;
- why the change belongs in that owner;
- validation commands and results;
- skipped validation with reasons;
- remaining risks and deferred work.

### 20. Keep Automation Non-Privileged

AI agents, scripts, generated prompts, workflow helpers, and automation should
use the same public contracts, diagnostics, validation, and policy gates as
humans and tests.

Automation may inspect, propose, generate candidates, run validation, prepare
checklists, and summarize evidence. It should not bypass authority, mutate
hidden state directly, skip validation, hide failed validation, or turn
generated output into accepted truth without owner review.

## Engineering Checklist

Use this checklist before a significant software change:

1. Name the owner, authority, or bounded context.
2. Name the invariants that must not break.
3. Identify source truth and derived projections or caches.
4. Define the contract that crosses each boundary.
5. Separate policy checks from semantic validation.
6. Choose the time and consistency model.
7. Name storage separately from authority.
8. Name failure semantics.
9. Define diagnostics and observation.
10. Keep descriptions separate from execution when editability or persistence
    matters.
11. Check dependency direction.
12. Reuse existing patterns before adding abstractions.
13. Organize code by responsibility, not technical dumping grounds.
14. Add tests for invariants, commands, projections, migrations, and public API
    behavior.
15. Update docs in the right document type.
16. Run the smallest meaningful validation first, then broader validation when
    boundaries changed.
17. Record closeout evidence and known gaps honestly.

## Anti-Patterns

Avoid:

- UI code mutating authoritative domain state directly;
- databases or caches silently becoming domain authority;
- one global command enum for every domain;
- one universal object model for unrelated concepts;
- mutable projections as source truth;
- unversioned persisted formats;
- policy enforced only in prose;
- broad best-effort error swallowing;
- helpers named `utils` when a responsibility name exists;
- services created only for code organization;
- docs and examples that teach an internal shortcut instead of the public API;
- roadmap or closeout text that claims product completeness without evidence;
- generated or AI-authored state bypassing validation.

