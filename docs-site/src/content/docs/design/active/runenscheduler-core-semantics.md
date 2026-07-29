---
title: RunenScheduler Core Semantics
description: Exact V1 planning semantics for identities, dependencies, access compatibility, ambiguity, readiness, determinism, provenance, errors, and host integration.
status: draft
owner: scheduler
layer: domain/scheduler
canonical: false
last_reviewed: 2026-07-29
related_docs:
  - ./runenscheduler-design-canvas.md
  - ../../architecture/repository-family-architecture.md
  - ../../reports/investigations/runenscheduler-ownership-investigation.md
  - ./runenecs-extraction-boundary-design.md
---

# RunenScheduler Core Semantics

## Status and purpose

This document specifies the candidate V1 semantics for RunenScheduler. It is draft
planning authority until the complete exact-current-main command census, repository
validation, independent review, and planning pull request are accepted.

The core is an inert deterministic planner. Runtime execution protocols and domain
adapters are separate layers.

## Semantic layers

```text
Domain adapter
    maps domain tasks and capabilities into neutral definitions

RunenScheduler core
    validates and prepares an immutable readiness plan

Runtime protocol or host adapter
    tracks completion and offers ready tasks to an executor

Executor
    owns threads, queues, work stealing, affinity, blocking, and shutdown
```

The core has no callback, `World`, thread, future, device, filesystem, or global
telemetry authority.

## Identity model

### Schedule identity

A schedule instance owns a checked `ScheduleId`. A prepared plan belongs to exactly
one schedule identity and one accepted definition revision.

A schedule identity is runtime-local. It is not a persistence, network, or cross-run
identifier.

### Task identity

`TaskId` is an opaque checked schedule-local identity allocated by the builder or
normalizer.

Required properties:

- no public arbitrary raw construction in the safe API;
- no silent wrapping, saturation, or terminal-ID reuse;
- exhaustion returns a structured error and leaves candidate state unchanged;
- an ID from another schedule is rejected;
- removed IDs are not silently rebound inside the same accepted revision;
- display and diagnostic accessors do not expose representation as stable authority.

### Stable task key

A task may carry a stable key supplied or derived by the owning adapter. It exists
for deterministic normalization, plan comparison, diagnostics, and conformance.

A stable key:

- is unique within one normalized definition;
- is not a substitute for `TaskId` during execution;
- has an explicit namespace or adapter owner;
- is not inferred solely from unstable Rust type names or registration addresses;
- is rejected on collision rather than resolved by last-writer wins.

### Labels and provenance

Human-readable labels and origin records are metadata. They may include names, source
locations, adapter identities, or owning subsystem facts.

Labels never determine safety identity. Changing a label alone does not silently
change task semantics.

## Definition model

A schedule definition contains:

```text
TaskSpec[]
Dependency[]
AccessClaim[]
PlacementClaim[]
AmbiguityPolicy
DefinitionMetadata
```

The builder may provide ergonomic grouped or fluent authoring, but normalization must
produce one explicit canonical representation before validation.

### Task specification

A V1 task specification contains at least:

- stable key;
- optional label and provenance;
- zero or more explicit dependencies;
- zero or more access claims;
- zero or more opaque placement constraints if placement is retained in V1 planning;
- no executable callback.

### Groups and sets

Groups or sets are authoring conveniences, not automatic execution barriers.

A group may expand into explicit membership or dependency relationships. The
normalized plan records the expanded relationships and their provenance.

A set name does not imply order. `before(set)` or `after(set)` expands into explicit
edges under deterministic rules.

## Dependency semantics

An explicit dependency `A -> B` states that `B` cannot become ready until `A` reaches
the completion state required by the host protocol.

V1 planning treats dependency edges as semantic causality. The planner does not
remove an edge merely because access claims would otherwise permit overlap.

### Unknown references

A dependency referencing an unknown schedule, task, group, or key returns a structured
build error. It is not ignored and does not produce an empty or partial plan.

### Duplicate edges

Duplicate equivalent edges normalize to one relationship while preserving or
aggregating origin records. Duplicate input is not allowed to change readiness or
ordering.

### Cycles

Any directed dependency cycle rejects the candidate definition.

Cycle diagnostics must be deterministic and include:

- an ordered cycle path;
- task IDs or stable keys;
- edge provenance for each path segment;
- enough context to locate the originating adapter or authoring declaration.

The public contract does not promise that every possible cycle is enumerated in one
error. It does promise stable evidence for at least one detected cycle and a
structured way to report additional diagnostics when available.

## Access model

### Access key

`AccessKey` is opaque to the core. Equality and deterministic ordering are provided by
an accepted key registry or normalized key representation.

The core does not know whether a key denotes memory, an ECS component, a service, a
queue, a chunk, a product, a hardware interface, or another capability.

Strings are diagnostic labels, not operational key identity.

### Access modes

V1 defines:

```text
Shared
Exclusive
```

Compatibility is:

| First | Second | Compatible concurrently |
|---|---|---|
| Shared | Shared | yes |
| Shared | Exclusive | no |
| Exclusive | Shared | no |
| Exclusive | Exclusive | no |

`Exclusive` means non-overlapping use is required. It does not necessarily mean a
memory write.

### Internal task claims

A single task that declares both incompatible claims on the same normalized key is
rejected unless the accepted authoring API normalizes them safely to one stronger
claim before validation.

The preferred V1 rule is:

- repeated `Shared` claims normalize to one `Shared` claim;
- any `Exclusive` claim dominates repeated `Shared` claims from the same task only
  when the adapter explicitly requests claim strengthening;
- contradictory claims from independent sources are reported rather than silently
  rewritten.

The planning PR must select one exact strengthening rule before implementation.

### Aliasing and partitions

The core performs no arbitrary user-defined alias callback during planning.

Adapters provide canonical keys whose equality expresses aliasing. A domain may
represent partitions through separate keys only when it can prove their disjointness.
Unknown aliasing must map to a broader shared key and therefore serialize
conservatively.

Example:

```text
PhysicsWorld
    partitioned by adapter into
PhysicsIsland/17
PhysicsIsland/18
```

The adapter owns the proof that those keys are disjoint for the accepted run.

## Conflict and ambiguity

### Access incompatibility

An access incompatibility proves that two tasks cannot overlap. It creates an
exclusion fact or requires an accepted ordering relationship.

It does not prove which task must run first.

### Semantic ambiguity

A semantic ambiguity exists when incompatible tasks have no explicit causal order and
their relative order may affect observable results.

V1 must expose an explicit policy. Candidate values are:

```text
Error
Warn
SerializeStable
```

Recommended accepted behavior:

- framework and production definitions default to `Error`;
- diagnostic migration tooling may use `Warn`;
- `SerializeStable` is an explicit opt-in for work whose order is declared
  semantically irrelevant.

Stable serialization is deterministic but is never presented as inferred business or
domain meaning. Its provenance records the policy and affected access key.

### Exclusion versus edge

The prepared plan should distinguish:

- causal dependency edges;
- access exclusions;
- explicit barriers;
- placement or concurrency constraints.

An implementation may lower exclusions into internal edges for a particular serial or
batch representation, but inspection must retain the original reason and must not
misreport the lowered edge as an authored dependency.

## Readiness semantics

### Prepared readiness DAG

A prepared plan stores sufficient facts for a runtime protocol to determine readiness
without reconstructing domain metadata:

- normalized tasks;
- predecessor counts or predecessor lists;
- successors;
- access exclusions or accepted deterministic serialization relationships;
- placement facts;
- canonical serial rank;
- provenance and diagnostics.

### Initial readiness

A task is initially ready when:

- all required references are valid;
- it has no incomplete causal predecessors;
- its access and placement admission can be granted by the runtime protocol;
- it is not rejected, cancelled, or inactive under host-owned run policy.

Host-owned activation conditions are resolved before or outside V1 core preparation.
Arbitrary stateful callbacks are not part of the core definition.

### Completion propagation

When task `A` completes successfully under the host protocol, only direct successors
whose remaining predecessor count reaches zero become causally ready.

For:

```text
A -> C
B -> D
```

completion of `A` may make `C` ready while `B` remains in flight. Diagnostic
layers such as `[A, B]`, `[C, D]` do not impose a global barrier.

### Barriers

A barrier is explicit or adapter-generated with structured provenance. It is not
inferred from a Rust type-name suffix or inserted after every topological layer.

The principle is:

> No unexplained synchronization.

A generated barrier identifies:

- owning adapter;
- affected tasks;
- correctness or policy reason;
- source declaration or normalization rule.

## Determinism

### Canonical normalized input

Determinism is defined over normalized semantic input, not over arbitrary unordered
container iteration or wall-clock arrival order.

Normalization must define stable ordering for:

- task keys;
- access keys and claims;
- dependency relationships;
- groups and expanded memberships;
- provenance records;
- diagnostics.

Hash-map iteration order, pointer addresses, thread scheduling, and process-global
allocation order cannot influence the prepared plan.

### Planner version

The plan records a planner semantic version or contract revision. The deterministic
promise is:

```text
same normalized definition
+ same planner contract revision
+ same explicit planning policy
= equivalent prepared plan
```

Equivalent means the same task ordering, dependency/exclusion facts, diagnostics,
cycle evidence, serial order, and stable inspection representation.

### Canonical serial order

The planner produces one total serial order that respects all explicit dependencies
and accepted exclusions.

Tie-breaking among otherwise equivalent tasks uses documented normalized keys, never
runtime completion timing or unordered registration containers.

The serial order is a reference and conformance product. It does not require
production executors to dispatch ready independent tasks in that order.

### Diagnostic layers

A diagnostic layer groups tasks by a stable topological or readiness property. It may
help humans understand available parallelism.

A layer is not a runtime stage unless a host explicitly selects conservative
layer-by-layer execution.

## Prepared plan model

Illustrative shape:

```rust
pub struct PreparedSchedule {
    schedule_id: ScheduleId,
    revision: PlanRevision,
    tasks: Box<[PreparedTask]>,
    dependencies: Box<[PreparedDependency]>,
    exclusions: Box<[PreparedExclusion]>,
    serial_order: Box<[TaskId]>,
    diagnostic_layers: Box<[DiagnosticLayer]>,
    diagnostics: Box<[PlanDiagnostic]>,
}
```

This is illustrative, not accepted Rust API. The eventual API must preserve the
semantic distinctions even if storage differs.

### Immutability

After preparation, the plan is read-only. Execution state such as ready, running,
completed, failed, or cancelled lives in a separate run instance or host adapter.

### Transactional activation

Building a candidate plan does not modify the accepted active plan.

Activation occurs only after complete validation. A build error leaves the previously
accepted plan untouched.

V1 may provide plan construction without a resident active-plan manager. The
transactional invariant still applies to any later manager or hot-reload adapter.

## Provenance and inspection

Every relationship has an origin category such as:

```text
ExplicitDependency
ExpandedGroupDependency
AccessExclusion
StableSerializationPolicy
ExplicitBarrier
AdapterSynchronization
PlacementConstraint
```

Inspection must answer:

- why does `A` precede `B`?
- why can `C` and `D` not overlap?
- which key caused an incompatibility?
- which declaration or adapter introduced this fact?
- which tasks participate in a cycle?
- what is the canonical serial order?
- what diagnostic layer contains a task?

The core returns structured DTOs. File writing, DOT rendering, JSON output, logs, and
UI presentation are caller-owned transformations.

## Error model

Public planning failures are structured and owned. `anyhow`, panics, flattened joined
strings, and missing `Option` results do not define the durable contract.

Candidate categories include:

```text
IdentityExhausted
DuplicateStableKey
ForeignTaskId
UnknownTask
UnknownGroup
UnknownAccessKey
InvalidClaim
ContradictoryClaim
DuplicateOrInvalidPlacement
DependencyCycle
UnresolvedAmbiguity
UnsatisfiableConstraint
InternalInvariantViolation
```

Each error carries stable machine-readable classification and bounded owned evidence.
Diagnostic labels may be included but are not parsed as authority.

### Panic policy

Ordinary invalid user or adapter input returns errors. Panics are reserved for
unreachable internal invariant violations and must be contained by host execution
policy when callbacks eventually run outside the core.

### Atomic failure

Identity allocation, task addition, dependency expansion, and preparation either
commit a complete accepted candidate mutation or leave builder state in a documented
valid state. Public methods must not partially add relationships and then return a
failure without rollback or explicit transaction semantics.

## Host and executor boundary

The core plan contains task IDs, not callbacks. A host adapter maps IDs to executable
work.

Illustrative host contract:

```rust
pub trait TaskHost {
    type Error;

    fn execute(&mut self, task: TaskId) -> Result<(), Self::Error>;
}
```

A production concurrent runtime will require richer dispatch and completion contracts.
That is not part of V1 core acceptance.

The existing Runenwerk runtime job executor remains physical execution authority for
the preferred non-ECS proof. RunenScheduler must not duplicate its worker pool,
work-stealing queues, backpressure, generations, or publication policy.

## RunenECS adapter boundary

RunenECS owns conversion from ECS metadata into neutral definitions:

```text
ECS component/resource read   -> Shared AccessKey
ECS component/resource write  -> Exclusive AccessKey
system sets                   -> normalized grouping/dependencies
commands                      -> independent command-buffer production
command application           -> explicit ECS-owned task or boundary
```

ECS broadcast, work-queue, tick-buffer, drain, orphaned-component, and structural
semantics remain in RunenECS. They may map to neutral claims or explicit tasks, but do
not become built-in RunenScheduler access modes or domains.

`ParamSlotDescriptor`, unsafe `SystemParam` extraction, `World`, and system callbacks
remain outside the core.

## Non-ECS proof boundary

The preferred Runenwerk product proof maps product derivations into neutral task and
access keys. The scheduler prepares legal readiness; the runtime job executor performs
work; Runenwerk generations and publication validate outcomes.

The proof must demonstrate actual value beyond a queue:

- more than one dependency branch;
- at least one access exclusion or shared access fact;
- deterministic serial reference;
- equivalent accepted output/publication under serial and concurrent execution;
- inspection that explains dependency and exclusion provenance.

## Dynamic, asynchronous, physics, and GPU extensions

These are later layers.

### Dynamic scopes

A later runtime protocol may support bounded child tasks, fork/join, parallel ranges,
and continuations. Child work may inherit or narrow parent authority but not acquire
unrelated global access or retroactively alter dispatched tasks.

### External completion

A later protocol may represent externally in-flight work and completion tokens. The
external runtime performs waiting; scheduler workers do not block on GPU, file, or
network completion.

### Physics

Third-party physics engines may adapt their job callbacks to a host executor while
retaining tested internal simulation graphs. Physics islands do not become V1 core
concepts.

### GPU

A host plan may express CPU preparation, opaque GPU workload submission, completion,
and CPU continuation. RunenGPU retains resource states, hazards, barriers, queues,
submissions, and device generations.

## V1 conformance

At minimum, conformance must prove:

1. independent tasks are initially ready;
2. a chain has exact predecessor propagation;
3. a diamond graph permits independent middle tasks;
4. `Shared`/`Shared` claims are compatible;
5. `Shared`/`Exclusive` and `Exclusive`/`Exclusive` claims are incompatible;
6. ambiguity policy is explicit and deterministic;
7. cycles return stable structured evidence;
8. unknown references fail atomically;
9. duplicate stable keys fail atomically;
10. ID exhaustion cannot wrap or reuse authority;
11. repeated preparation produces equivalent inspection products;
12. diagnostic layers do not impose barriers;
13. canonical serial interpretation respects dependencies and exclusions;
14. two adapters can consume the same neutral plan contract without core domain types.

## Open decisions required before implementation

The planning pull request must bind:

- exact checked ID representation and allocation API;
- stable key and opaque access-key registry representation;
- whether placement claims are V1 core or deferred;
- exact same-task claim-strengthening behavior;
- exact ambiguity policy types and defaults;
- representation of exclusions versus lowered deterministic edges;
- structured error taxonomy and owned evidence bounds;
- serial conformance interpreter location;
- private graph implementation candidate and replacement boundary;
- exact RunenECS and non-ECS proof specifications.
