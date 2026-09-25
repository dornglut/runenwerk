---
title: "Multiplayer Replication Implementation Roadmap"
description: "Post-RN8 multiplayer integration baseline and evidence gates for future Runenwerk replication work."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
publication: primary
related_designs:
  - ../design/active/net-authoritative-replication-protocol.md
  - ../design/active/net-plugin-runtime-bridge.md
  - ../design/active/ecs-net-replication-boundary.md
  - ../design/active/net-reconnect-history-recovery.md
  - ../design/active/net-transport-lanes-delivery.md
  - ../design/active/net-diagnostics-inspection.md
---

# Multiplayer Replication Implementation Roadmap

## Purpose

This document records the accepted post-RN8 Runenwerk multiplayer integration baseline and the
constraints for selecting later replication work.

RN8 parent #359 is complete and closed. This roadmap does not restart that migration and does not
authorize final Replicated View authoring syntax. New work must be derived from current maintained
consumers, current RunenNet/RunenECS authority, and the evidence boundary preserved by #322.

Current networking ownership and architecture are defined by:

- [net goals](goals.md);
- [networking architecture](net-architecture.md);
- [engine integration design](../design/active/net-plugin-runtime-bridge.md).

## Accepted Post-RN8 Baseline

Standalone RunenNet owns reusable realtime-networking semantics:

- compatibility negotiation and connection/session membership lifecycle;
- participant authority-input admission and finite resource policy semantics;
- client replication consistency, retained history, recovery, and acknowledgement state;
- participant prediction/reconciliation lineage and replay eligibility;
- authority replication cursor, baseline/history, recovery, delivery evidence, and ACK
  classification;
- reusable delivery/resource-pressure semantics and transport abstraction.

Runenwerk Engine integration owns product and host integration around those semantics:

- network schedule placement and role composition;
- engine-visible session/routing projections and diagnostics;
- bounded inbox/outbox work staging;
- current-frame inbound/outbound message projections;
- gameplay snapshot, delta, and input codecs/driver escape hatches;
- complete client replicated-product realization into gameplay/ECS state;
- host execution staging for remote input already admitted by RunenNet;
- authority snapshot capture/candidate formation and the host feedback boundary that records actual
  RunenNet `DeliveryAcceptance`;
- game/world relevancy, presentation, reconnect/deployment, and product policy.

The former `net/engine_net` migration shell is deleted. It is not a compatibility namespace and
must not be recreated.

Engine work-queue admission and frame-end projection are not transport emission and are not
`DeliveryAcceptance`. `NetworkInboundQueue` and `NetworkOutboundQueue` are Engine-owned
current-frame projections whose client/server directions compose independently.

Concrete transport remains a maintained-consumer concern. Runenwerk currently consumes
`runen-net-quic` for the separate Editor ↔ Runtime Preview product channel; that consumer does
not authorize a generic QUIC runtime inside the Engine gameplay Net plugin.

## RN8 Result

RN8 established the current boundary through bounded accepted cuts rather than compatibility
mirrors:

- N5 moved remote participant-input admission to RunenNet `AuthorityInputSession`;
- Replicated View R0 (#772 / PR #775) proved a complete derived replicated-state product, exact
  accounting, stable replicated identity, and owner-level atomic activation;
- N7 (#781 / PR #782) moved client replication consistency/history/recovery to RunenNet
  `ClientReplicationSet`;
- N9 (#807 / PR #810) moved tracked client prediction/reconciliation to RunenNet
  `PredictionLineage` while keeping delivery admission orthogonal;
- #823 / PR #853 removed the legacy component-registration authoring predecessor;
- N12 (#864 / PR #865) moved authority/server replication consistency and real delivery evidence
  to RunenNet `AuthorityReplicationSession` and `DeliveryAcceptance`;
- N13 (#886 / PR #889) removed the final `engine_net` migration shell.

Those cuts do not define final Replicated View macros, schema-generation syntax, audience
vocabulary, transport composition, or the ordinary developer-facing authoring API.

## Open Evidence Areas

The following areas may justify later bounded work when a current consumer demonstrates the
missing contract:

- ordinary ECS-to-network Replicated View authoring/projection beyond the low-level driver escape
  hatch, under #322;
- standard extraction/application ergonomics that preserve explicit network-visible contract and
  RunenNet schema authority;
- interest/relevancy resolution, explanation, and presentation policy where current product
  behavior demonstrates a concrete gap;
- per-connection diagnostics, desync inspection, and explanation surfaces;
- concrete gameplay transport/product composition once a maintained gameplay consumer exists.

This is a capability/evidence inventory, not an implementation sequence. Absence of a polished
ordinary authoring path is not permission to guess one.

## Dependency and Ownership Constraints

Future work must preserve these boundaries:

- RunenNet owns reusable networking semantics; Runenwerk does not recreate them in Engine resources.
- Runenwerk owns ECS/scheduler/gameplay/world/product policy, realization, and presentation.
- ECS storage/layout is not the network contract.
- Replication work follows available RunenECS contracts rather than freezing a future Replicated
  View syntax early.
- Full/clean projection or resynchronization remains the correctness reference; change evidence is
  an optimization.
- Concrete transport realization is added to Engine gameplay networking only for a proven
  maintained gameplay consumer.
- The separate Runtime Preview QUIC channel must not be generalized into Engine gameplay
  networking by convenience.
- Engine staging/projection is not RunenNet delivery acceptance.
- Host reconnect/deployment policy remains separate from RunenNet membership retention/recovery.
- No aliases, forwarding APIs, compatibility runtimes, or parallel semantic authorities may
  recreate deleted RN8 predecessors.

## Validation Invariants

Any later replication change must continue to prove, as applicable:

- deterministic snapshot/delta behavior;
- adversarial ACK/baseline and delivery-feedback handling;
- independent participant/connection replication state;
- authorized connection identity sourced from RunenNet;
- prediction/reconciliation behavior preserved unless the bounded issue explicitly owns a change;
- queue/projection state cannot masquerade as transport or delivery evidence;
- game/world relevancy policy remains outside reusable RunenNet protocol authority;
- canonical repository validation and focused owner tests pass on the exact reviewed head.

## Next Work Rule

Do not derive a new implementation merely because a capability appears in the open-evidence list.

Before activating another multiplayer slice:

1. re-establish current Runenwerk, RunenNet, RunenECS, and open-writer state;
2. identify a maintained consumer and the exact user-visible or integration contract that is
   missing;
3. identify the semantic owner and the smallest boundary that can prove that contract;
4. use #322 as architectural direction for Replicated View work without inferring final syntax;
5. reject work that would recreate RN8 migration layers, invent transport demand, or duplicate
   RunenNet authority.

If those facts do not establish one bounded contract, networking implementation should remain
inactive rather than manufacture a speculative API.
