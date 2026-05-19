---
title: Authority-Centered Boundary Architecture
description: A general-purpose long-term software architecture doctrine based on authority, invariants, contracts, flows, policy, time, consistency, storage, execution, failure, observation, evolution, and cost.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-19
related_docs:
  - ../design/active/runenwerk-capability-workbench-target-architecture.md
---

# Authority-Centered Boundary Architecture

## Status

This is a general software architecture doctrine for the repository. It is not
an ADR and does not by itself mark any Runenwerk-specific target architecture as
accepted or implemented.

Runenwerk applies this doctrine in the active future target design:
[`../design/active/runenwerk-capability-workbench-target-architecture.md`](../design/active/runenwerk-capability-workbench-target-architecture.md).

## 1. Purpose

This document defines a general-purpose architecture doctrine for building clean, long-lived software systems.

It is not tied to one specific pattern such as microservices, ECS, DDD, actors, event sourcing, clean architecture, plugins, or cell-based architecture. Those are implementation choices. This design starts from the deeper question:

> What owns truth, what may cross the boundary, what changes state, what is observed, and what failure semantics apply?

The central model is:

```text
Authority
+ Invariants
+ Contracts
+ Flows
+ Policy
+ Time
+ Consistency
+ Storage
+ Execution
+ Failure
+ Observation
+ Evolution
+ Cost
```

The short doctrine:

```text
Local code can be simple.
Boundary code must be explicit.
Authority code must be strict.
Distributed code must be observable.
Persistent code must be versioned.
Policy code must fail closed.
```

## 2. Core Principle

Do not start by choosing a pattern.

Do not begin with:

```text
Should this be a microservice?
Should this be ECS?
Should this be an actor?
Should this use event sourcing?
Should this be a plugin?
Should this be a cell?
```

Start by identifying boundaries:

```text
truth boundary
change boundary
read boundary
policy boundary
execution boundary
failure boundary
deployment boundary
```

Then choose the lightest pattern that protects those boundaries.

## 3. Core Concepts

### 3.1 Authority

An Authority owns truth and invariants.

Authority means:

```text
This part of the system decides what is valid here.
```

Examples:

```text
InventoryAuthority
MaterialGraphAuthority
UserAccountAuthority
CombatAuthority
RenderAuthority
WorkspaceAuthority
BuildPipelineAuthority
```

A database is not automatically authority. A service is not automatically authority. A provider/UI is not authority just because users interact with it.

Rule:

```text
No invariant, no authority.
```

Bad examples:

```text
ButtonRendererAuthority
StringFormatterAuthority
PanelHeaderAuthority
```

Those are helpers or components, not authorities.

Good examples:

```text
InventoryAuthority owns item movement and duplication rules.
MaterialGraphAuthority owns valid graph edits.
RenderAuthority owns GPU execution and render resource lifetime.
WorkspaceAuthority owns workspace layout mutations.
```

### 3.2 Invariants

An invariant is a condition that must not be violated.

Examples:

```text
An inventory item cannot be duplicated.
A material graph edge must connect compatible ports.
A workspace surface must have a stable identity.
A product artifact must match its source hash.
A quest reward cannot be claimed twice.
```

For each invariant, ask:

```text
Where is it enforced?
Can it be enforced locally?
Does it require a transaction?
Does it cross authority boundaries?
What happens if enforcement fails?
```

If an invariant matters, it must live in an authority or domain validation boundary, not only in UI code.

### 3.3 Contract

A Contract defines what crosses a boundary.

Contracts include:

```text
function signatures
traits/interfaces
commands
queries
events
DTOs
file formats
database schemas
API endpoints
message schemas
protocols
product descriptors
```

Rule:

```text
Boundaries should speak contracts, not internals.
```

Bad:

```text
Module A reaches into Module B's internal mutable state.
```

Good:

```text
Module A sends a command/query/DTO.
Module B returns a result/event/product/projection.
```

### 3.4 Flow

Flow describes how information and change move through the system.

Common flow types:

```text
Command      -> request to change state
Query        -> request to read state
Event        -> accepted fact that happened
Product      -> derived artifact or deployable output
Projection   -> derived read/view/subscription model
Status       -> current observed state/condition
Diagnostic   -> explanation of failure or warning
```

A typical command flow:

```text
command proposal
  -> policy check
  -> authority/domain validation
  -> accepted command
  -> state transition
  -> event
  -> projection/product update
  -> diagnostics/status
```

### 3.5 Policy

Policy decides whether something is allowed.

Examples:

```text
permissions
roles
feature flags
host restrictions
rate limits
environment rules
admin gates
security rules
```

Policy is not domain validation.

Example:

```text
Policy:
  user may edit this document

Domain validation:
  the requested edit is structurally valid
```

Rule:

```text
Policy decides allowed.
Domain/authority decides valid.
```

### 3.6 Time

Every operation has a temporal model.

Ask whether work is:

```text
synchronous
asynchronous
scheduled
streaming
batch
fixed tick
interactive
eventual
transactional
```

Examples:

```text
Inventory transfer:
  synchronous or transactional

Shader compile:
  asynchronous job

Combat simulation:
  fixed tick

Search indexing:
  eventual projection

Deployment:
  reconciled desired state

Telemetry:
  streaming or batch
```

Without a time model, architectures become vague.

### 3.7 Consistency

Every authority needs a consistency model.

Options include:

```text
single-writer
strong transaction
optimistic concurrency
eventual consistency
append-only event log
snapshot + replay
last-write-wins
CRDT
server-authoritative tick
```

Examples:

```text
Inventory:
  strong consistency; no duplication

Presence:
  eventual consistency is acceptable

Collaborative cursor:
  last-write-wins or CRDT may be acceptable

Economy:
  audited transactions/events

Combat:
  server-authoritative tick plus reconciliation
```

The wrong consistency model can break the system even if the module structure looks clean.

### 3.8 Storage

Storage persists state. Storage is not automatically authority.

Rule:

```text
Storage persists.
Authority decides.
```

Examples:

```text
Postgres stores inventory rows.
InventoryAuthority owns inventory rules.

Object storage stores asset bundles.
ProductPipeline owns product validity.

A realtime table stores state.
Reducer/domain logic owns valid transitions.
```

Sometimes storage and authority are colocated. Conceptually, they are still different responsibilities.

### 3.9 Execution

Execution is where work runs.

Execution forms:

```text
function
module
component
job
actor
ECS system
service
process
runtime cell
remote service
sandboxed component
```

Execution mode should be chosen after authority and contract boundaries are understood.

Example:

```text
A material compiler may start as an in-process job,
then become a background worker,
then become an out-of-process tool service,
without changing the domain product contract.
```

### 3.10 Failure

Every boundary needs failure semantics.

Possible failure policies:

```text
fail closed
retry
rollback
compensate
degrade
queue
preserve last-good
ignore
panic
```

Examples:

```text
Unknown stable key:
  fail closed

Preview compile failure:
  preserve last-good

Telemetry write failure:
  degrade and retry

Payment failure:
  reject transaction

Search index failure:
  stale read acceptable
```

Failure policy is part of the contract.

### 3.11 Observation

Every serious system needs observability.

Observation includes:

```text
status
diagnostics
logs
metrics
traces
audit
health
events
debug projections
```

Rule:

```text
If failure cannot be observed, the design is incomplete.
```

For complex systems, build inspectors and diagnostics early.

### 3.12 Evolution

Software changes. Architecture must support promotion, demotion, splitting, merging, and deletion.

Evolution operations:

```text
promote
demote
split
merge
inline
extract
delete
deprecate
migrate
```

A design that only supports growth but not simplification is not long-term clean.

### 3.13 Cost

Every boundary has a cost:

```text
code
tests
latency
debugging
versioning
cognitive load
deployment
observability
coordination
```

Rule:

```text
Create a boundary only when the benefit buys authority, isolation, scale, security, clarity, or long-term maintainability.
```

Do not architect for fashion.

## 4. Universal Design Checklist

For every serious subsystem, ask:

```text
1. Authority
   Who owns the truth?

2. Invariants
   What must never be violated?

3. Contract
   What crosses the boundary?

4. Flow
   Is it command, query, event, product, projection, or status?

5. Policy
   Who is allowed to request it?

6. Validation
   Who decides semantic correctness?

7. Time
   Is it sync, async, streaming, batch, scheduled, tick-based?

8. Consistency
   Strong, eventual, single-writer, optimistic, transactional, CRDT?

9. Storage
   Where is state stored, and is storage separate from authority?

10. Execution
   Function, module, job, actor, service, process, cell, component?

11. Failure
   Fail closed, retry, rollback, compensate, degrade, preserve last-good?

12. Observation
   What logs, metrics, traces, diagnostics, audit, status exist?

13. Evolution
   How does this change, version, migrate, split, merge, or disappear?

14. Cost
   Is this boundary worth maintaining?
```

## 5. Promotion Ladder

Use the lightest form that protects the boundary.

```text
1. Function
   pure local logic

2. Module
   cohesive implementation hiding

3. Component
   explicit interface and local state

4. Authority
   owns invariants and validates commands

5. Subsystem
   several authorities/products under one runtime/app boundary

6. Service
   independent execution or deployment boundary

7. Cell
   isolated authority/failure/scaling/deployment boundary

8. Platform
   shared contracts, policy, observability, tooling
```

Reverse moves are equally valid:

```text
delete
inline
merge
demote
split
extract
```

Perfectionist architecture allows simplification.

## 6. Pattern Selection Guide

### 6.1 Simple Module

Use when:

```text
logic is local
no independent authority
no separate persistence
no policy boundary
no deployment boundary
no cross-team contract
```

Examples:

```text
string parser
math helper
view formatting
local renderer utility
small validation helper
```

### 6.2 Component

Use when:

```text
local state exists
clear interface is useful
reuse is likely
there is no independent authority boundary
```

Examples:

```text
UI widget
editor panel component
graph canvas widget
local cache component
```

### 6.3 Authority / Domain

Use when:

```text
it owns truth
it validates commands
it protects invariants
other code depends on its decisions
```

Examples:

```text
MaterialGraphAuthority
InventoryAuthority
WorkspaceAuthority
SceneAuthority
BuildPipelineAuthority
```

### 6.4 DDD Bounded Context

Use when:

```text
domain language differs
business/game rules are complex
teams need independent model ownership
```

Examples:

```text
inventory
economy
combat
material graph
procgen
editor scene
```

### 6.5 ECS

Use when:

```text
large homogeneous data
high-performance simulation
many entities
cache-friendly processing
parallel systems
```

Examples:

```text
world simulation
combat tick
physics update
animation sampling
render extraction
```

Avoid using ECS as the whole app architecture. ECS is an execution/data-processing pattern.

### 6.6 Actor Model

Use when:

```text
isolated state owners
message passing
sessions/rooms/instances
concurrency boundaries
```

Examples:

```text
game session
dungeon instance
chat room
party session
world shard
```

Risk:

```text
message spaghetti
weak consistency if boundaries are unclear
```

### 6.7 Event Sourcing

Use when:

```text
audit matters
replay matters
accepted facts matter
transactions need history
```

Examples:

```text
inventory
economy
admin actions
live ops changes
quest progress
account state
```

Avoid event-sourcing every transient high-frequency update.

### 6.8 Product Graph / Job Graph

Use when:

```text
source-to-derived artifacts
incremental rebuilds
cache/invalidation
compiler/build-like flow
heavy computation
```

Examples:

```text
asset import
material compile
texture generation
procgen evaluation
shader products
world products
```

### 6.9 Service

Use when:

```text
independent scaling
failure isolation
deployment independence
security/resource boundary
long-running work
external access boundary
```

Do not create a service only for code organization.

### 6.10 Cell

Use when:

```text
blast radius matters
tenant/shard/region/world isolation matters
data placement matters
deployment independence matters
scaling boundary matters
```

Examples:

```text
EU world cell
NA inventory cell
dungeon instance cell
housing cell
live event cell
```

### 6.11 Capability Policy

Use when:

```text
different users/tools/hosts need different permissions
security matters
in-game/admin/external access exists
resource access must be governed
```

Examples:

```text
full editor vs in-game editor
admin tool permissions
AI command proposal gates
external component permissions
```

## 7. Examples

### 7.1 ECS / Simulation

```text
Authority:
  WorldRuntimeAuthority

Invariants:
  entity ids unique
  transform state valid
  physics constraints respected

Contracts:
  SpawnEntity
  DespawnEntity
  ApplyDamage

Execution:
  ECS scheduler and systems

Observation:
  simulation diagnostics
  event stream
  render extraction
```

ECS is execution inside an authority, not the authority by itself.

### 7.2 Engine / Render

```text
Authority:
  RenderAuthority for GPU execution and render resource lifetime

Contracts:
  RenderMaterialProduct
  TextureProduct
  MeshProduct
  SdfFieldProduct

Execution:
  render graph
  GPU upload
  draw dispatch

Observation:
  frame diagnostics
  GPU resource diagnostics
```

Render consumes formed products. It should not own authoring semantics.

### 7.3 Domain / Material Graph

```text
Authority:
  MaterialGraphAuthority

Invariants:
  graph edges connect compatible ports
  source maps remain valid
  resource requirements are explicit

Commands:
  AddNode
  ConnectEdge
  SetParameter

Products:
  material descriptor
  shader requirements
  source map

Observation:
  graph diagnostics
  validation errors
```

### 7.4 Workbench / Editor

```text
Authority:
  WorkbenchHost for installed tool composition
  WorkspaceAuthority for mounted surface/layout state

Contracts:
  ToolSurfaceStableKey
  ToolSuiteRegistry
  SurfaceProviderRequest

Execution:
  provider registry
  workspace projection
  shell interaction routing

Observation:
  Tool Suite Registry Inspector
  provider diagnostics
```

### 7.5 Provider / UI Surface

```text
Authority:
  none over domain truth

Owns:
  presentation
  view model
  interaction mapping

Flow:
  local interaction -> command proposal

Anti-pattern:
  provider mutates source truth directly
```

### 7.6 Runtime Server

```text
Authority:
  CombatAuthority
  InventoryAuthority
  QuestAuthority

Contracts:
  client commands
  accepted events
  replication views

Execution:
  custom server
  actor
  ECS/job scheduler
  transactional reducer module

Observation:
  server metrics
  audit events
  diagnostics
```

### 7.7 Database

```text
Authority:
  not automatically the database

Storage:
  durable rows, documents, event logs, blobs

Rule:
  Storage persists.
  Authority decides.
```

### 7.8 AI / Automation

```text
Authority:
  none by default

Allowed:
  inspect diagnostics
  propose commands
  generate candidates
  request validation

Not allowed:
  hidden writes
  bypass policy
  bypass domain validation
```

## 8. Anti-Patterns

Avoid:

```text
UI directly mutates domain state
database schema becomes domain authority
service exists only to organize code
every helper gets a command/event
provider owns semantic validation
cache becomes source of truth
policy is documented but not enforced
events are emitted before acceptance
unknown inputs silently remap
plugins get privileged side APIs
hot loop checks heavyweight policy every frame
```

## 9. Best Short Doctrine

```text
Local code can be simple.
Boundary code must be explicit.
Authority code must be strict.
Distributed code must be observable.
Persistent code must be versioned.
Policy code must fail closed.
```

## 10. Final Rule

Choose architecture by boundary pressure, not by fashion.

```text
If the boundary is local:
  use simple code.

If the boundary owns truth:
  use an authority.

If the boundary crosses systems:
  use contracts.

If the boundary changes state:
  use commands and validation.

If the boundary records facts:
  use events.

If the boundary forms artifacts:
  use products and provenance.

If the boundary needs isolation:
  use services or cells.

If the boundary needs permissions:
  use policy.

If the boundary can fail:
  make failure observable.
```

The most universal long-term architecture is not a single named pattern.

It is:

```text
Authority-Centered Boundary Architecture
  with explicit contracts, flows, consistency, time, failure, observation, and evolution.
```
