---
title: Typed App Program Multiplayer And Concurrency Design Review
description: Critical review of Typed App Program design pressure from multiplayer, networking, threading, parallel execution, determinism, rollback, and synchronization.
status: active
owner: ui
layer: reports
canonical: false
last_reviewed: 2026-07-04
related_docs:
  - ./typed-app-program-current-state-investigation.md
  - ./typed-app-program-engine-pressure-and-design-review.md
  - ../../design/active/typed-app-program-and-ui-proof-design.md
  - ../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ../../workspace/complete-design-gate.md
---

# Typed App Program Multiplayer And Concurrency Design Review

## Purpose

Review whether the Typed App Program design properly considers multiplayer, networking, multithreading, parallel execution, deterministic replay, rollback, synchronization, and conflict handling.

This document is design-gate hardening only. It does not authorize networking, multiplayer, scheduler, threading, rollback, ECS, or runtime implementation.

## Review Verdict

The prior design and engine-pressure review considered networking and threading partially, but not completely enough for Runenwerk's long-term use cases.

Existing coverage:

```text
networking / remote execution
remote requests
network session host facts
scheduler/runtime plan pressure
threading as non-owned runtime concern
replay determinism not depending on scheduler order unless explicit
multi-host compatibility
```

Missing explicit pressure:

```text
multiplayer sessions
authoritative host/server ownership
client prediction
rollback/reconciliation
replication and interest management
network transport separation
message ordering and causality
identity, permissions, and authority
conflict resolution
thread-safe model snapshots
parallel reducer/evaluator constraints
cross-thread effect completion
```

Conclusion:

```text
Typed App Program must be concurrency-safe and multiplayer-aware in structure.
It must not own networking, multiplayer simulation, replication, transport, scheduler execution, or thread/job management.
```

## Correct Relationship

Typed App Program may own structural records:

```text
AppModelSnapshot
AppAction
AppActionSource
RouteActionMap
AppReducerOutcome
AppEffectPlan
AppReplayTrace
HostCompatibilityReport
```

Multiplayer/network/runtime systems own execution and meaning:

```text
transport
sessions
replication
prediction
rollback
interest management
authoritative simulation
anti-cheat / trust policy
threading
job scheduling
ECS mutation
network serialization policy
```

## Multiplayer Classification

| Concern | Consider In Typed App Program? | Implement In First Proof? | Owner Boundary |
| --- | --- | --- | --- |
| Local single-player action loop | Yes | Yes, via headless counter | App proof owns only local model/action/reducer/replay. |
| Multiplayer session identity | Yes, as future host fact | No | Network/session owner manages sessions and peers. |
| Player/user identity | Yes, as action source metadata pressure | No | Auth/session/platform owner owns identity and trust. |
| Authoritative server | Yes, as host authority pressure | No | Server/runtime owner decides authority and validation. |
| Client prediction | Yes, as future replay/rollback pressure | No | Game/network owner owns prediction semantics. |
| Rollback/reconciliation | Yes, as trace compatibility pressure | No | Simulation/game/network owner owns rollback policy. |
| Replication | Yes, as host fact/effect pressure | No | Network/world/game owner owns replication. |
| Interest management | Yes, as streaming/network pressure | No | World/streaming/network owner owns visibility/interest. |
| Conflict resolution | Yes, as reducer/authority pressure | No | Domain/app owner defines conflict policy. |
| Latency/loss/reorder | Yes, as event ordering pressure | No | Network transport/session owner observes and reports. |
| Anti-cheat/trust | Yes, as authorization pressure | No | Security/server/game owner owns trust policy. |
| Remote UI interaction | Yes, as host route pressure | No | Host/network owner maps remote events to app/domain proposals. |
| Collaborative editor | Yes, as future multi-author pressure | No | Collaboration/editor domain owns merge/conflict semantics. |

## Concurrency And Multithreading Classification

| Concern | Consider In Typed App Program? | Implement In First Proof? | Owner Boundary |
| --- | --- | --- | --- |
| Immutable model snapshots | Yes | Yes | First proof should use immutable/cloneable snapshots. |
| Single logical event order | Yes | Yes | Replay trace must define canonical event order. |
| Parallel reducer execution | Yes, as future constraint | No | Reducers must be pure before parallelization is considered. |
| Thread-safe effect queue | Yes, as future effect pressure | No | Runtime/scheduler owner manages queues. |
| Cross-thread effect completion | Yes, as event pressure | No | Completion must re-enter as explicit event/fact. |
| Shared mutable model state | Yes, as forbidden pressure | No | App program must reject hidden shared mutation. |
| Job scheduling | Yes, as runtime pressure | No | Scheduler/runtime owns execution. |
| ECS parallel systems | Yes, as host/runtime pressure | No | ECS/runtime owns storage/scheduling; app program owns replay facts only. |
| Determinism under parallelism | Yes | Yes as rejection rule | Replay must not depend on nondeterministic scheduler order. |
| Locks/atomics/channels | Yes, as implementation risk | No | First proof must not require concurrency primitives. |

## Required Structural Concepts

Future implementation planning must reserve conceptual space for:

```text
AppActionSource
AppActorId
AppAuthority
AppCausalityToken
AppSequenceNumber
AppTick
AppFrame
AppRevision
AppBranch
AppConflictDiagnostic
AppPredictionTag
AppRollbackSegment
AppReconciliationReport
HostNetworkSessionFacts
HostThreadingFacts
HostSchedulerFacts
HostAuthorityFacts
```

The first proof may use only local defaults:

```text
AppActionSource::LocalHeadless
AppAuthority::Local
single logical event order
single-threaded deterministic replay
```

But the durable data model must not make those local defaults the only possible shape.

## Required Action Metadata Pressure

Every future app action should be able to carry or derive:

```text
action_id
action_version
payload
source_kind
source_actor
source_host
authority_context
sequence_or_tick
causality token
schema version
capability requirements
source route/source control/source map
```

The first proof may use local placeholder metadata, but the report must make clear that real multiplayer/remote actions require explicit source and authority records.

## Required Replay Rules

Replay must be deterministic and explicit.

Rules:

```text
Replay order is a logical order, not scheduler order.
Wall-clock time must not affect reducer results unless supplied as explicit input.
Randomness must be seeded or supplied as explicit input.
Remote message order must be represented by sequence/tick/causality metadata.
Effect completion must enter as explicit event/fact.
Rollback/reconciliation must be represented as explicit trace segments later.
Parallel execution must not change replay result.
```

The first proof should prove the local subset:

```text
single event sequence
stable model revisions
deterministic reducer result
stable output after replay
```

## Multiplayer Boundary Rules

### Authoritative Server

Typed App Program may later express:

```text
action source metadata
authority requirements
host compatibility diagnostics
server-accepted or server-rejected action facts
```

It must not own:

```text
server simulation
network transport
anti-cheat policy
connection lifecycle
session matchmaking
replication protocol
```

### Client Prediction And Rollback

Typed App Program may later record:

```text
predicted action trace
confirmed action trace
rollback segment
reconciliation report
```

It must not own:

```text
prediction algorithms
simulation rollback mechanics
state compression
delta replication
lag compensation
```

### Collaborative Editing

Typed App Program may later support:

```text
multi-author action source metadata
conflict diagnostics
merge proposal effect records
```

It must not own:

```text
CRDT semantics
OT algorithms
editor document merge policy
permissions model
remote persistence
```

## Multithreading Boundary Rules

Typed App Program may later support:

```text
thread-safe immutable snapshots
queued effect proposals
explicit completion events
scheduler compatibility facts
parallel-safe replay constraints
```

It must not own:

```text
thread pool
job graph
work stealing
locks/atomics policy
ECS system scheduling
runtime freeze points
cross-thread resource lifetime
```

## Effect Taxonomy Additions

Add these future pressure names to the reserved effect vocabulary:

```text
NetworkSendProposal
NetworkSessionRequest
RemoteActionProposal
AuthorityCheckRequest
PredictionBegin
PredictionConfirm
PredictionReject
RollbackRequest
ReconciliationRequest
ReplicationRequest
CollaborationMergeProposal
SchedulerTaskProposal
ThreadedWorkRequest
```

The first proof may still use only:

```text
NoEffect
```

But implementation planning must not make `NoEffect` or local-only effects structural dead ends.

## Host Fact Taxonomy Additions

Add these future pressure names to the reserved host fact vocabulary:

```text
HostAuthorityFacts
HostNetworkSessionFacts
HostReplicationFacts
HostPredictionFacts
HostRollbackFacts
HostThreadingFacts
HostSchedulerFacts
HostParallelismBudgetFacts
HostCollaborationFacts
```

The first proof may still use only:

```text
HeadlessHost capability facts
```

## Required Stop Conditions

Stop if implementation tries to:

```text
make multiplayer transport app-program-owned
make replication app-program-owned
make prediction/rollback algorithm app-program-owned
make server authority implicit
make action source implicit for remote/multiplayer actions
use scheduler order as replay order
allow hidden cross-thread model mutation
store locks/channels/thread handles inside durable app model
execute network sends from reducers
complete async/network effects through hidden callbacks
make ECS parallel systems the app-program source of truth
skip causality/order diagnostics for remote actions
```

## Impact On First Proof

The first proof can still be the Headless Counter App Proof.

However, implementation planning must require these local-concurrency guarantees:

```text
AppModelSnapshot is immutable or cloned per reducer step.
AppReducer is pure and deterministic.
AppReplayTrace records a single canonical event order.
AppAction includes at least local source metadata or reserves the field explicitly.
Effect completion is not hidden; counter uses NoEffect.
No shared mutable global state is used.
No thread or async runtime is required.
```

This keeps the first proof bounded but not narrow.

## Final Verdict

Multiplayer and multithreading were not yet considered fully enough in PR #66 before this review.

After this review is added as companion authority, the design gate is stronger:

```text
Typed App Program is local-first in its first proof,
but multiplayer-aware and concurrency-safe in its durable structure.
```

Do not implement multiplayer or multithreading in the first proof. Do not ignore them either.
