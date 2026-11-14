---
title: Native application profiler (experimental)
description: Opt-in bounded Runenwerk profiling for actual native applications.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-11-14
publication: primary
---

# Native application profiler (experimental, #1494)

This first-party capture observes a **real native application**, including RunenUI input
settlement and publication when the application installs UiPlugin. The profiler is not
the Render Lab benchmark. The feature is experimental until the parent issue's
platform, privacy and overhead gates pass.

The application is still an ordinary Cargo binary. The binary must be built
against a Runenwerk revision **containing this profiler**. The currently published
Runen Lab Todo PR #8 pins an older Engine revision and does **not** gain this
feature automatically. For a qualified compatible native application, run from
**that application's Cargo project directory** on Windows PowerShell:

~~~powershell
New-Item -ItemType Directory -Force .\artifacts | Out-Null
$env:RUNENWERK_PROFILE_OUTPUT = Join-Path (Get-Location).Path 'artifacts\native-profile.json'
cargo run --release --locked
Remove-Item Env:RUNENWERK_PROFILE_OUTPUT
~~~

Create the output directory beforehand. Each file path must be new: the profiler
refuses to replace an existing file. Export first writes and syncs a same-directory
temporary file, then publishes the final name with a non-overwriting hard link.
The destination filesystem must support hard links; unsupported filesystems fail
rather than exposing an incomplete final JSON. Ordinary failures remove the temporary
file; an abrupt process crash can leave a temporary artifact.
No application-specific instrumentation,
Tracy or graphical editor is required. The profiler reports status and counts at
shutdown; a normal run without this environment variable does not activate it.

The CI native typed-UI smoke retains a seven-day artifact named
`runenwerk-native-profiler-smoke`, containing the exact original `capture.json`,
a derived `summary.json` with per-stage duration histograms, `manifest.json`
with the checked-out source revision and requested backend, `validation.json`
with the checks performed, a readable `report.md`, and `SHA256SUMS` for integrity. CI validates these
files and their histogram counts before upload. This fixture is controlled
and privacy-reviewed; arbitrary real application captures are **not** uploaded
automatically. The short smoke is **not** a performance benchmark: its
distributions are real samples, but not statistically qualified for latency
or regression decisions. Interactive HTML inspection belongs to the later
graphical profiler reader, not this backend delivery.

Each capture has a non-secret session identity and monotonically increasing
CPU completion-order event ordinals; retained ordinals expose event-loss gaps
without implying that completion order is start-time order. A session identity
groups owner observations **within one capture**, but does not claim an
otherwise unavailable Host/Render per-frame join. The raw `source_revision`
is stamped at **build time** only when the application is compiled with
`RUNENWERK_SOURCE_REVISION`; otherwise the value stays `unknown`. The
native CI smoke compiles with its verified exact checkout revision and rejects
a mismatched raw capture.

Schema v1 contains bounded CPU event records (monotonic nanoseconds from one
process epoch), per-stage median/P95 grouped by actual window/slot identity,
native callback categories and window identities, Engine
schedule-call timing (not standalone RunenECS per-system timings), RunenUI M13
pump status, numeric receipt counts and up to 16 exact terminal receipt
sequence/finality samples per pump (with omissions reported), RunenUI host
settlement/publication, and up to 512 existing Render-owned frame
observations (with export omissions reported) when available. The Render frame index
is **not** assumed equal to the Host frame ordinal. GPU-composed durations are
GPU-clock facts, not synchronized with CPU events, scanout or display FPS.

After the CPU buffer fills, older events are evicted so late interactions remain visible.
`dropped_cpu_events` counts those evictions; stage statistics describe the retained
window only, not the full application lifetime. A pump that fails without returning a
batch is recorded as a failure event without claiming input-settlement evidence.

Per-stage CPU duration histograms have fixed, upper-exclusive nanosecond bounds
(50,000; 100,000; 250,000; 500,000; 1,000,000; 2,000,000; 4,000,000;
8,000,000; 16,000,000; 33,333,334), plus a final unbounded bin.
Their counts add to the retained stage sample count. Render observations
remain a separate source-owned clock and are never combined into a CPU
frame-duration histogram.

The report includes dropped event counts, Render observation drop counters and
explicit unavailability for process CPU/RAM, inner RunenUI shaping/layout
durations, per-system timings and physical input-to-photon/scanout. It excludes
input text, IME content, task data, UI tree and arbitrarily formatted event
debug payloads. Reported completion status indicates the native app lifecycle
outcome, not proof that every GPU timestamp became available.

The artifact also contains a **capture-close-only** snapshot of the real Host
pacing policy (including its target FPS when capped) and the last native
window registry dimensions, scale and focus state. These are not historical
frame-by-frame values and do not prove physical presentation or monitor refresh.

Do not compare raw durations from runs with different application content,
window extent, compiler profile, GPU/backend, OS or pacing policy. Quantify
capture overhead against matched opt-out runs and test input, resize, focus,
shutdown and persistence before treating the instrumentation as accepted.
