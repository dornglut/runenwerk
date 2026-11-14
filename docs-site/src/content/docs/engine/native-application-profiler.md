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

The application remains an ordinary Cargo binary and must be built against a
Runenwerk revision **containing this profiler**. The currently published Runen
Lab Todo PR #8 pins an older Engine and does **not** gain the flag automatically.
For a compatible native application, run from its Cargo project directory:

~~~powershell
cargo run --release -- --profile
~~~

From a Cargo workspace, choose the package before the `--` separator:

~~~powershell
cargo run -p todo --release -- --profile
~~~

`--profile` after `--` is a reserved **native Runenwerk runtime argument**,
not Cargo's compilation-profile `--profile NAME`. The exact flag enables
first-party capture, automatically allocates a unique non-overwriting session
directory beneath `target/runenwerk/profiles` relative to the process's working
directory (or `CARGO_TARGET_DIR` when set), and writes both `capture.json` and
`report.html` on normal close. It prints both paths. No application source changes,
manual directories, server, separate reader, Tracy, or browser launch are needed.

The existing `RUNENWERK_PROFILE_OUTPUT` environment variable remains available
for controlled/CI captures: alone it writes JSON at the caller-selected path;
combined with `--profile` it also writes a sibling HTML file and creates the
parent directory when needed. Exact `--profile` recognition leaves other native
application arguments untouched; applications must not use this reserved token
for unrelated product behavior.

Exports first write/sync a same-directory temporary file and publish by a
non-overwriting hard link. The filesystem must support hard links; unsupported
ones fail explicitly. A failure writing HTML after successfully publishing JSON
leaves the valid JSON available and reports an error; it does not claim an HTML
report was created. Unclean process termination can prevent capture/export.
Ordinary launches without the flag or environment variable do not activate
instrumentation.

The CI native typed-UI smoke retains a seven-day artifact named
`runenwerk-native-profiler-smoke`, containing the exact original `capture.json`,
a derived `summary.json` with per-stage duration histograms, `manifest.json`
with the checked-out source revision and requested backend, `validation.json`
with the checks performed, a readable `report.md`, and `SHA256SUMS` for integrity. CI validates these
files and their histogram counts before upload. This fixture is controlled
and privacy-reviewed; arbitrary real application captures are **not** uploaded
automatically. The short smoke is **not** a performance benchmark: its
distributions are real samples, but not statistically qualified for latency
or regression decisions. The command-line `--profile` path also emits a static offline HTML report
from the **same schema-v3 typed capture**. This is inspection output, not an
independent measurement or physical performance benchmark.

Each capture has a non-secret session identity and monotonically increasing
CPU completion-order event ordinals; retained ordinals expose event-loss gaps
without implying that completion order is start-time order. A session identity
groups owner observations **within one capture**, but does not claim an
otherwise unavailable Host/Render per-frame join. The raw `source_revision`
is stamped at **build time** only when the application is compiled with
`RUNENWERK_SOURCE_REVISION`; otherwise the value stays `unknown`. The
native CI smoke compiles with its verified exact checkout revision and rejects
a mismatched raw capture.

Schema v3 contains bounded CPU event records (monotonic nanoseconds from one
process epoch), per-stage median/P95 grouped by actual window/slot identity,
native callback categories and window identities, Engine
schedule-call timing (not standalone RunenECS per-system timings), RunenUI M13
pump status, numeric receipt counts and up to 16 exact terminal receipt
sequence/finality samples per pump (with omissions reported), plus a bounded
read-only projection of the existing canonical RunenUI pointer trace. Each
sample carries its original M13 work sequence, source-recorded pointer
phase, displayed hit-test generation and current-vs-retained snapshot,
physical-path depth and target presence, and positive canonical
`PointerActivateCollected` trace evidence. An Up may compare its physical hit
with the last retained Down from the same pointer stream without exporting
the opaque target. Such a comparison is not a claim about the framework's
pressed-owner eligibility. Missing or evicted canonical evidence remains
explicitly unavailable; `not_observed` means only that a source trace record
was not retained, not that the application action failed. No trace Debug string,
authored identifier, text, credential or raw mounted identity is exported.
An M13 `committed` input receipt is never relabeled as a successful UI
activation or Todo operation. The M13 CPU measurement ends at the actual
RunenUI pump/settlement boundary; read-only profiler trace projection and
export bookkeeping occur afterward and are not attributed to M13 work.
Observer overhead requires its own matched on/off workload qualification.
Schema v1/v2 captures remain historical and are not silently upgraded to v3.

Schema v3 also records up to 1,024 **source-owned native UI presentation transitions**
independently from timed CPU spans: `source_staged_for_render` only after a
successful aligned source/paint staging, and
`terminal_present_promoted_for_input` only when the existing
Render terminal Present completion actually promotes that exact immutable
generation for native input. Each point observation includes the original
RunenUI publication ID, native slot/window and Render surface identifiers,
source hit-test generation, coordinate revision and original Host frame when
known. The stage includes the source ingress epoch; a terminal promotion does
not invent one. Both transitions use the same monotonic CPU process clock.
Their independent ordinals, capacity and eviction count prevent missing
history from being misrepresented as no promotion.

**Important:** Terminal Present promotion is an authorized native input
generation transition, **not physical display scanout, GPU execution time,
or pixel visibility**. Do not equate it with a CPU-to-GPU clock join.
A test that verifies Todo storage after an action has not thereby verified
that a later UI generation is promoted for native input. Capture time can
distinguish when a new generation was staged versus promoted, but cannot
supply a portable native display-ready barrier by itself.

RunenUI Host settlement/publication and up to 512 existing Render-owned
frame observations (with export omissions reported) remain available. The Render frame index
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
