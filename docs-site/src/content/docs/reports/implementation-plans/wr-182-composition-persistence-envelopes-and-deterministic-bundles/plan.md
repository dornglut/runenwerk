---
title: Composition Persistence Envelopes And Deterministic Bundles Implementation Plan
description: Decision-complete WR-182 contract for canonical composition documents, linked core/app envelopes, atomic generation activation, last-good recovery, scope precedence, legacy rejection, and persistence benchmarks.
status: accepted
owner: ui
layer: report
canonical: false
last_reviewed: 2026-06-19
wr: WR-182
milestone: PM-UI-COMPOSITION-003
related_designs:
  - ../../../design/accepted/app-neutral-ui-composition-design.md
related_adrs:
  - ../../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-182: Composition Persistence Envelopes And Deterministic Bundles

## Authority And Promotion

PM-UI-COMPOSITION-002 and WR-181 are complete with `runtime_proven` evidence.
WR-182 is promotable from `ready_next` to `current_candidate` after this
accepted contract validates. Implementation remains inside
`domain/ui/ui_composition`; no editor, Draw, adaptive, engine, windowing,
provider/session, or legacy workspace runtime is wired in this checkpoint.

The roadmap and manifest previously declared both the existing crate parent and
its new `benches` child as separate write scopes. That self-overlap was removed:
the existing crate scope already authorizes exact new files within the crate.

## Current Code Truth And Reuse Inventory

- `CompositionDefinitionV1` is serde-capable, formation-normalized, and already
  sorts targets, roots, regions, and mounted units by typed ID.
- semantic profile/capability references validate on construction and
  deserialization; their private canonical strings are safe in deterministic
  paths and filenames.
- `CompositionHistory` owns structural journal order. The persisted journal
  projection is an audit/reproducibility document only; `CompositionState`
  remains non-persisted authority.
- `CompositionFixture` uses `BTreeMap`/`BTreeSet` for unordered declarations;
  fixture catalogs still need explicit fixture-ID ordering.
- `LayoutPromotion` currently derives a normalized core candidate only. WR-182
  adds explicit layout metadata, scope, compatibility, extension snapshots,
  bundle validation, and atomic activation without importing adaptive state.
- `foundation/diagnostics` remains the portable diagnostic substrate.
  Persistence owns separate `composition_persistence.*` codes and records.
- the workspace already uses `ron`, `blake3`, and `tempfile`; WR-182 consumes
  those crates directly rather than creating hash, temp-file, or RON helpers.
- editor V1-V5 files expose a top-level `version` field. This checkpoint tests
  neutral unsupported-source probing only and never imports editor types,
  paths, readers, or writers.

## Exact Module Ownership

Add one responsibility boundary under `src/persistence/`:

| Module | Ownership |
|---|---|
| `canonical` | Closed canonical RON codecs for approved composition documents and canonical-input verification. |
| `digest` | Validated lowercase `blake3:<64-hex>` digests and typed preimage formation. |
| `envelope` | Shared metadata, compatibility range, core envelope, extension envelope, and sorted extension links. |
| `bundle` | Candidate formation, cross-document validation, exact extension-set validation, and generation identity. |
| `generation` | Generation IDs, active/previous pointer document, path layout, activation and recovery result contracts. |
| `repository` | Platform-neutral filesystem repository orchestration over private file operations; production native operations and test failure injection. |
| `scope` | Built-in/project/user whole-definition precedence and explicit selection outcomes. |
| `legacy` | Read-only unsupported-source probe for top-level legacy `version` 1 through 5. |
| `diagnostic` | `composition_persistence.*` codes, stages, typed subjects, rejections, ordering, and foundation conversion. |

`promotion/layout.rs` gains explicit promotion metadata and bundle formation.
`history/journal.rs` gains deterministic read-only journal projection accessors;
history semantics do not move into persistence. `lib.rs` re-exports normal
persistence workflows through `persistence::prelude`, while low-level hash
preimage and file-operation helpers remain private.

No catch-all `codec.rs`, `io.rs`, `utils.rs`, or generic serialization escape
hatch is added.

## Canonical Document Contract

Canonical persistence is UTF-8 RON with one reviewed configuration:

- schema-defined struct field order;
- two-space indentation, LF line endings, and exactly one trailing newline;
- no comments, array enumeration, implicit `Some`, or default-driven field
  omission;
- unknown fields rejected on every persisted DTO;
- integer fixed-point fractions only;
- structural unordered collections normalized before encoding;
- decoding succeeds only if re-encoding produces byte-identical input.

The public API is closed and document-specific:

- `CanonicalCompositionDocuments::definition`;
- `CanonicalCompositionDocuments::fixture_catalog`;
- `CanonicalCompositionDocuments::journal`;
- `CanonicalCompositionDocuments::core_envelope`;
- `CanonicalCompositionDocuments::extension_envelope`;
- `CanonicalCompositionDocuments::generation_pointer`.

There is no public generic `to_ron<T>` API. App-owned extension payloads enter
as `CanonicalExtensionPayload`, constructed from a typed app codec's canonical
UTF-8 RON bytes, profile ID, and schema version. Core validation rejects empty,
non-UTF-8, CRLF, missing-trailing-newline, comment-bearing, or non-canonical
payload bytes before hashing. Consumer checkpoints must test each typed app
codec for insertion-order-independent bytes; WR-182 uses a typed test extension.

Canonical sorting rules are exact:

- definition targets, roots, regions, and mounted units by typed ID;
- capability/profile sets by canonical semantic ID;
- fixture catalog entries by fixture ID;
- journal entries by applied revision, then transaction ID;
- extension links and extension documents by extension profile ID, then schema
  version;
- diagnostic records by their existing stage/severity/code/subject order;
- semantic vectors retain meaning: stack order, overlay z-order, transaction
  command order, and named split children are not re-sorted.

Display names are metadata only and never enter an identity comparison or sort
key. Golden tests cover definition, fixture catalog, journal, core envelope,
extension envelope, and pointer documents. Each proves repeated-byte identity,
insertion-order independence where applicable, decode/encode byte identity,
and stable digest spelling.

## Identity, Compatibility, And Digest Contract

Add validated namespaced `AppProfileId` and `ExtensionProfileId` semantic
references. Add non-zero `ExtensionSchemaVersion` and `AppSchemaVersion` value
types plus `CompositionGenerationId`, whose canonical value is the full bundle
digest.

`CompositionCompatibility` contains:

- `app_profile: AppProfileId`;
- inclusive `minimum_app_schema_version`;
- inclusive `maximum_app_schema_version`.

Formation rejects zero versions and inverted ranges. Loading receives one
`CompositionCompatibilityRequirement { app_profile, app_schema_version }` and
rejects profile or range mismatches before exposing payloads.

`CompositionDigest` accepts and emits only lowercase
`blake3:<64-lowercase-hex>`. Hash preimages are canonical DTO bytes with an
explicit schema/domain marker; string concatenation and display/debug output are
forbidden hash inputs.

The non-circular digest rules are:

1. core payload digest hashes canonical shared metadata, sorted extension
   identities (profile plus schema version, excluding digest fields), and the
   canonical definition payload;
2. each extension payload digest hashes the same shared metadata, its extension
   identity, the core schema version, and its canonical typed payload;
3. the generation digest hashes the complete canonical core envelope followed
   by every complete canonical extension envelope using length-delimited,
   sorted records.

The full generation digest is the immutable generation directory identity. No
truncated digest is used for identity or collision handling.

## Core And Extension Envelope Contract

`CompositionSharedMetadataV1` is repeated exactly in core and extension
documents:

- layout ID (`CompositionDefinitionId`);
- definition revision;
- core schema version;
- compatibility declaration.

`CompositionExtensionLinkV1` adds extension profile, extension schema version,
core payload digest, and extension payload digest. The core envelope contains
the exact sorted link set. Every extension envelope repeats shared metadata,
its link, and its typed canonical payload.

`CompositionBundleCandidate::form` is the only public route from a definition
plus extension snapshots to a savable bundle. It normalizes and reforms the
definition, validates compatibility and canonical payloads, rejects duplicate
extension identities, forms all digests, and then validates the complete linked
bundle again. Loading uses the same validator.

Any missing, extra, duplicate, unreadable, unsupported, non-canonical, or
metadata/digest-mismatched extension rejects the whole bundle. No partial core
or extension result escapes. Saving, loading, validation, and promotion return
one atomic `CompositionPersistenceRejection` with stable diagnostics.

## Promotion Contract

`LayoutPromotion` gains:

- validated non-empty `LayoutDisplayName` metadata;
- `CompositionLayoutScope` (`BuiltIn`, `Project`, or `User`);
- source state revision and candidate definition;
- explicit compatibility declaration.

`LayoutPromotion::form_bundle(extension_snapshots)` snapshots all required
typed app extensions in one call and forms a complete bundle candidate. If any
snapshot is missing, invalid, or mismatched, no bundle or storage mutation is
produced.

Canonical save excludes liveness and adaptive state by type: neither can enter
the definition or persistence DTOs. Later adaptive promotion must first produce
a validated structural promotion delta; WR-182 adds no adaptive dependency and
never treats an active adaptive projection as save input.

## Repository And Atomic Generation Activation

`CompositionBundleRepository` owns one scope root with this fixed layout:

```text
<scope-root>/
  active-generation.ron
  generations/
    <full-generation-digest>/
      core.ron
      extensions/
        <extension-profile>.v<schema-version>.ron
```

Semantic IDs already exclude path separators and traversal. Repository methods
still validate every derived component before touching the filesystem.

Activation is single-writer and compare-and-swap:

1. read and validate the current pointer;
2. reject if it differs from the caller's `expected_active_generation`;
3. validate the complete candidate in memory;
4. write a uniquely named staging generation in the same filesystem;
5. sync every file and directory;
6. read the staging generation back and run the complete canonical/link/hash
   validator;
7. rename the immutable staging directory to its digest identity (or validate
   an already-existing byte-identical generation);
8. write and sync a temp pointer containing `active` and prior `previous`;
9. atomically persist/replace `active-generation.ron` in the same directory and
   sync the scope root.

The production implementation uses `tempfile` for same-directory pointer
replacement and `std::fs` sync operations. File operations are behind a
crate-private narrow port so tests can fail each pre-activation stage without
global mutable hooks.

A failure before pointer replacement leaves the old pointer byte-identical.
Orphan staging directories may be cleaned on next repository open; immutable
completed generation directories may be retained. A crash after successful
pointer replacement means the new generation is active and already validated.

The pointer document stores both active and previous full generation IDs. Load
validates active first. If active was externally corrupted, it validates
previous and returns `RecoveredLastGood` plus a stable warning diagnostic; it
does not silently rewrite the pointer. If neither validates, load rejects.

The repository contract is explicitly single-writer per scope. In-process app
lifecycle owns writer serialization. Compare-and-swap detects stale callers;
cross-process locking is a deferred requirement and must be added before any
future multi-process writer is authorized.

## Scope Precedence

`CompositionLayoutCatalog` resolves whole definitions in this order:

```text
User > Project > BuiltIn
```

There is no field-level merge. A missing layout at a higher scope falls through
to the next scope. A present but invalid higher-scope pointer/bundle rejects
selection; it never silently falls back to a lower scope. Last-good recovery is
attempted only within the selected scope. Session recovery is not part of this
catalog and cannot shadow saved scope precedence.

## Legacy Rejection

`probe_composition_source` first looks for the new envelope marker. A RON source
with the legacy top-level `version` field set to 1, 2, 3, 4, or 5 returns
`composition_persistence.schema.unsupported_legacy` with the observed version
and explicit reset/select-valid-layout guidance.

The probe is read-only. Tests create byte fixtures for all five versions,
capture file metadata and bytes, invoke probe/load/save rejection paths, and
prove no file, directory, pointer, or timestamp is changed. No editor legacy
type, filename, reader, writer, migration, or fallback is imported.

Unknown legacy versions and malformed documents use distinct diagnostics. No
automatic migration or success-shaped default is permitted.

## Diagnostic Namespace

Every persistence rejection emits `CompositionPersistenceDiagnosticRecord`
with stable `composition_persistence.*` code, severity, stage, typed subject,
and actionable message. Required code families cover:

- canonical encode/decode/non-canonical input;
- digest syntax/content mismatch;
- compatibility/profile/schema mismatch;
- missing/extra/duplicate extension;
- shared-metadata/link mismatch;
- unsupported legacy and unknown schema;
- stale expected generation;
- staging/write/sync/readback/rename/pointer failures;
- active corruption, last-good recovery, and no-valid-generation;
- scope selection failure.

Filesystem errors retain source details in diagnostic context but never expose a
success-shaped empty bundle.

## Failure Injection And Benchmarks

Repository tests inject failures at: staging creation, core write, each extension
write, file sync, readback, generation rename, pointer temp write, pointer sync,
pointer replacement, and root sync. Every pre-pointer failure proves unchanged
active bytes and load result. Pointer replacement is the commit point; no
post-commit path may return a failure claiming the old generation remains
active.

`benches/composition_persistence.rs` is a harness-free release benchmark with
deterministic small, medium, and large fixtures:

| Fixture | Regions | Mounted units | Extensions | Extension payload total |
|---|---:|---:|---:|---:|
| small | 16 | 16 | 1 | 4 KiB |
| medium | 512 | 1,024 | 4 | 256 KiB |
| large | 4,096 | 8,192 | 16 | 1 MiB |

It measures canonical core serialization, canonical core deserialization,
complete bundle hash/validation, and repository generation read separately.
After 10 warmups it records 50 samples and reports p50, p95, bytes/second, and
items/second. The release smoke gate is p95 <= 250 ms for large serialization
and p95 <= 500 ms for large deserialization on the local reference machine;
closeout records measured values rather than extrapolating from the 8.3 ms drag
budget. The benchmark must not be mixed into prose docs or executed during
normal unit tests.

## Implementation Sequence

1. Add exact dependencies and the persistence module surface.
2. Implement typed compatibility/version/digest identities and persistence
   diagnostics.
3. Implement closed canonical codecs and deterministic definition, fixture,
   journal, envelope, and pointer DTO projections.
4. Implement extension snapshots, digest preimages, linked bundle formation,
   and complete bundle validation.
5. Extend `LayoutPromotion` with name/scope/compatibility and atomic bundle
   formation while excluding liveness/adaptive state.
6. Implement generation repository activation, readback, compare-and-swap,
   last-good recovery, and scope selection.
7. Add V1-V5 read-only rejection fixtures and full failure-injection tests.
8. Add the dedicated benchmark runner and persistence usage documentation.
9. Run focused, benchmark-build, dependency, docs, roadmap, and production
   validation; write resolver-backed runtime, reproducibility, migration, and
   benchmark-artifact evidence.

Each step must leave focused tests green. No placeholder, `todo!`, panic-based
normal error path, weak string digest, generic arbitrary serializer, silent
fallback, compatibility alias, or automatic legacy migration may remain.

## Tests And Acceptance

Focused tests must prove:

- every approved document's exact golden bytes and stable digest;
- insertion-order-independent definition, fixture, journal, and extension-link
  serialization;
- canonical decode/re-encode byte identity and unknown-field rejection;
- invalid namespaced IDs, versions, compatibility ranges, and digest spellings
  reject;
- missing, extra, duplicate, reordered, tampered, and cross-layout extensions
  reject atomically;
- promotion snapshots all extensions or produces no bundle;
- every injected repository failure before pointer commit preserves the prior
  active pointer and load result;
- active corruption returns explicit previous-generation recovery, while dual
  corruption rejects;
- stale compare-and-swap activation rejects;
- user/project/built-in whole-definition precedence and invalid-higher-scope
  fail-closed behavior;
- V1-V5 bytes and filesystem metadata remain unchanged;
- fixture/journal canonical documents do not become structural authority;
- no app/editor/Draw/adaptive/provider/session/windowing imports enter the
  crate;
- the benchmark target builds and its fixtures match the declared sizes.

Required commands:

```text
cargo fmt --all --check
cargo test -p ui_composition persistence
cargo test -p ui_composition deterministic
cargo test -p ui_composition --bench composition_persistence --no-run
task ui:dependencies
task production:validate
task roadmap:validate
task docs:validate
task planning:validate
```

## Non-Goals

- editor or Draw extension schemas, providers, paths, save commands, or load
  integration;
- adaptive projection persistence or adaptive promotion-delta implementation;
- native windows, OS lifecycle, engine, renderer, app IO policy, or multi-
  process writers;
- V1-V5 migration, mutation, deletion, aliasing, or fallback;
- field-level scope merging, cloud sync, collaboration, CRDT/OT, encryption, or
  compression;
- cleanup of `ui_surface`, workspace state, or `ui_hosts`.

## Rollback And Stop Conditions

This checkpoint is additive and has no runtime consumer. Reject it and preserve
all existing runtime behavior if bytes are unstable, hashes are circular or
weak, app payload semantics enter core, a partial bundle escapes, any failure
can move the active pointer early, last-good recovery is silent, higher-scope
corruption falls through, legacy bytes change, benchmark fixtures are not
measurable, or exact resolver evidence cannot be produced.

Stop rather than widening scope into app adapters, editor/Draw state, adaptive
behavior, engine/windowing, `ui_surface`, foundation extraction, or a generic
serialization framework.

## Closeout

Close only as `runtime_proven` after deterministic golden tests, complete bundle
and promotion tests, filesystem failure injection, scope/legacy tests, benchmark
target build and measurements, dependency guards, docs, roadmap, production,
and resolver-backed evidence pass. The closeout must list editor static
projection/read-only legacy authority, Draw, adaptive, runtime chrome,
accessibility, full performance, cleanup, and final truth work as owned later
scope.
