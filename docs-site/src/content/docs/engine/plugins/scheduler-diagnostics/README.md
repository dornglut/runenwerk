---
title: "Scheduler Diagnostics Plugin"
description: "Documentation for Scheduler Diagnostics Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-24
publication: primary
---

# Scheduler Diagnostics Plugin

## Purpose

Emits periodic scheduler/runtime diagnostics logs.

## Usage

- Plugin: `SchedulerDiagnosticsPlugin`
- Typed schedule: `RenderSubmit`

By default logs every `120` frames.

## Ownership Boundaries

- Owns periodic diagnostics emission policy.
- Consumes runtime state for logging only.
- Does not own scheduler execution or render submission.

The runtime diagnostics consume `Time`, optional simulation tick state, and host-neutral `PrimaryPresentationMetricsResource`. The scheduler payload does not infer native title or headless identity.

## Extension Points

- Add additional structured fields to diagnostics logs.
- Make logging interval configurable via resource/config.

## Guides

- Usage: [../../../docs/reference/plugins/scheduler-diagnostics/usage-guide.md](../../reference/plugins/scheduler-diagnostics/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/scheduler-diagnostics/advanced-guide.md](../../reference/plugins/scheduler-diagnostics/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/scheduler-diagnostics/architecture.md](../../reference/plugins/scheduler-diagnostics/architecture.md)


