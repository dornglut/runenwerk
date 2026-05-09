---
title: External Runtime Preview Process
description: Decision to run editor preview, simulate, and play execution in an app-owned external runtime child process.
status: accepted
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-09
related_designs:
  - ../../design/active/engine-game-runtime-editor-ecs-scripting-hot-reload-design.md
related_roadmaps:
  - ../../apps/runenwerk-editor/roadmap.md
---

# ADR: External Runtime Preview Process

## Status

Accepted

## Context

M5 needs runtime preview, simulate, play, and data hot reload boundaries that are clear enough to preserve authored editor state, ratified asset products, and runtime execution state independently.

The current editor runtime is single-process. That keeps MVP iteration simple, but it makes preview/play isolation, reload failure containment, and restart-required reporting harder to reason about as project-owned assets and formed products enter runtime consumers.

## Decision

Run M5 preview, simulate, and play execution in an app-owned external runtime child process.

The editor remains the authoring host. The child process is a runtime host with its own preview/play window, loopback QUIC endpoint, and headless mode for tests. The editor spawns and supervises it through an app-owned preview process subsystem.

The preview protocol vocabulary belongs in `domain/editor/editor_preview` and must remain engine-agnostic. Network crates may carry generic typed payload envelopes, but editor preview semantics must not be hard-coded into `engine_net` or `engine_net_quic` protocol enums.

## Rejected Alternatives

Keep preview/play single-process for M5. This preserves today systems but weakens restart boundaries and makes failed reload containment depend on in-process discipline.

Put preview-specific commands directly in generic net protocol enums. That would make networking editor-shaped and would force unrelated runtime clients to carry editor semantics.

Let the editor write runtime internals such as `SdfChunkStore` directly. That bypasses engine/runtime invariants and turns app integration into domain authority.

## Consequences

M5 adds a new app crate for the runtime preview process, an editor-side process manager, a domain-owned preview protocol crate, and generic typed payload support in the network protocol.

Reload classification becomes explicit. Live-reloadable data may update the child process. Unsafe structural changes report preview-session or runtime-process restart requirements. Future asset families without owning domains report unsupported or restart-required status instead of being partially modeled in M5.

The engine remains responsible for runtime intake helpers and status vocabulary. The editor app remains responsible for host IO, process lifecycle, user-facing diagnostics, and mapping engine statuses into preview protocol statuses.
