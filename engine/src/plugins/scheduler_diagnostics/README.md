# Scheduler Diagnostics Plugin

## Purpose

Emits periodic scheduler/runtime diagnostics logs.

## Usage

- Plugin: `SchedulerDiagnosticsPlugin`
- Scheduler node: `scheduler_diagnostics`
- Runs after: `frame_render_submit`

By default logs every `120` frames.

## Ownership Boundaries

- Owns periodic diagnostics emission policy.
- Consumes runtime state for logging only.
- Does not own scheduler execution or render submission.

## Extension Points

- Add additional structured fields to diagnostics logs.
- Make logging interval configurable via resource/config.
