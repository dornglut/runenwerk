---
title: Native application profiler (experimental)
description: Opt-in bounded Runenwerk profiling for actual native applications.
---

# Native application profiler (experimental, #1494)

This first-party capture observes a **real native application**, including RunenUI input
settlement and publication when the application installs UiPlugin. The profiler is not
the Render Lab benchmark. The feature is experimental until the parent issue's
platform, privacy and overhead gates pass.

The application is still an ordinary Cargo binary. To capture its full lifetime,
set an output path before starting a **new native process**, for example in
Windows PowerShell:

~~~powershell
$env:RUNENWERK_PROFILE_OUTPUT = "$PWD\artifacts\todo-native-profile.json"
cargo run --manifest-path projects/todo/Cargo.toml --release --locked
Remove-Item Env:RUNENWERK_PROFILE_OUTPUT
~~~

Create the output directory beforehand. Each file path must be new: the profiler
refuses to replace an existing file. No application-specific instrumentation,
Tracy or graphical editor is required. The profiler reports status and counts at
shutdown; a normal run without this environment variable does not activate it.

Schema v1 contains bounded CPU event records (monotonic nanoseconds from one
process epoch), native callback categories and window identities, Engine
schedule-call timing (not standalone RunenECS per-system timings), RunenUI M13
pump status and numeric receipt counts, RunenUI host settlement/publication,
and existing Render-owned frame history when available. The Render frame index
is **not** assumed equal to the Host frame ordinal. GPU-composed durations are
GPU-clock facts, not synchronized with CPU events, scanout or display FPS.

The report includes dropped event counts, Render observation drop counters and
explicit unavailability for process CPU/RAM, inner RunenUI shaping/layout
durations, per-system timings and physical input-to-photon/scanout. It excludes
input text, IME content, task data, UI tree and arbitrarily formatted event
debug payloads. Reported completion status indicates the native app lifecycle
outcome, not proof that every GPU timestamp became available.

Do not compare raw durations from runs with different application content,
window extent, compiler profile, GPU/backend, OS or pacing policy. Quantify
capture overhead against matched opt-out runs and test input, resize, focus,
shutdown and persistence before treating the instrumentation as accepted.
