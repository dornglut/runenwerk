---
title: "Net Prediction Reconciliation Boundary Design"
description: "Boundary design for prediction, authoritative correction, input replay, and gameplay-owned smoothing."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Prediction Reconciliation Boundary Design

## Purpose

This design defines which layer owns prediction and reconciliation
contracts, and which layer owns gameplay correction and smoothing policy.

## Ownership Rules

`engine_net` owns:

- prediction vocabulary;
- profile-level prediction flags;
- authoritative versus predicted snapshot comparison helpers;
- reconciliation result contracts.

`engine/src/plugins/net` owns:

- pending input frame storage;
- replay of pending inputs after authoritative snapshots/deltas;
- net diagnostics resources for corrected/replayed counts;
- schedule placement for prediction and replication systems.

Gameplay/app modules own:

- what an input means;
- how input mutates state;
- whether a correction is smoothed, snapped, animated, or ignored for
  presentation;
- interpolation and visual error hiding.

Transport adapters own none of this.

## Implemented Substrate

Implemented now:

- `ReplicationProfilePreset::PredictedMovement` and
  `PredictionMode::OwnerPredicted`.
- `PredictionState` and `ReconciliationResult` in `engine_net`.
- Engine plugin `PredictionState<TInput>` pending frame storage.
- Input collection through `InputDriver::take_local_input`.
- Tick-buffer insertion for local and remote inputs.
- Replay of pending input frames after authoritative snapshot/delta
  application.
- `PredictionDiagnostics` counters for applied, replayed, and corrected
  behavior.

## Partial Contracts

Partial now:

- Prediction is driver-based, not yet a zero-boilerplate declarative ECS
  workflow.
- Reconciliation is snapshot/delta contract-level; there is no generic
  interpolation or smoothing API.
- Pending input replay exists, but input ACK semantics are still tied to
  snapshot ACK progression rather than a dedicated input ACK channel.
- Gameplay code still implements the actual predicted simulation rule via
  `InputDriver::apply_input`.

## Future Work

Future work:

1. Add explicit input ACK metadata if prediction needs independent input
   confirmation.
2. Add gameplay-facing prediction policy hooks without moving smoothing
   into net core.
3. Add inspection for pending input frames by tick and owner.
4. Add tests for replay order after corrections and reconnect.
5. Add examples that show normal predicted movement without custom
   transport code.

## Invariants

- Clients may predict only locally and only for data the profile permits.
- Server snapshots remain authoritative.
- Reconciliation updates local state from authoritative data before
  replaying pending predicted input.
- Smoothing is presentation policy and belongs in gameplay/app code.
- Prediction code must not make transport aware of gameplay semantics.

## Validation Plan

Required validation:

- unit tests for `engine_net` reconciliation helpers;
- engine plugin tests for pending-frame retain and replay order;
- diagnostics tests for corrected/replayed counters;
- example-level validation for a predicted input driver.
