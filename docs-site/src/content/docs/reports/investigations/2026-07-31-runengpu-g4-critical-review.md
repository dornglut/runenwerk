---
title: RunenGPU G4 Critical Review
description: Exact-revision critical review of the G4A implementation and the accepted G4B/G4C delivery plans.
status: active
owner: gpu
layer: reports
last_reviewed: 2026-07-31
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ./runengpu-g4-context-program-realization-investigation.md
  - ../../workspace/specs/pt-runengpu-g4a-context-admission.ron
  - ../../workspace/specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../../workspace/specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU G4 Critical Review

## Record classification

| Field | Value |
|---|---|
| Evidence class | Dated implementation and architecture review |
| Observation date | 2026-07-31 |
| Owning repository | `dornglut/runenwerk` |
| Owning work item | Issue `#186` |
| Delivery under review | Pull request `#199` |
| Review state | Changes required |
| Acceptance authority | Not held by this report or reviewer |
| Durable authority | Accepted design and specifications, owning issues, code, tests, pull request, and exact-head CI |
| Staleness rule | This review applies only to the exact reviewed head identified below |

This report preserves review evidence and rationale. It does not replace the owning
issue, accepted G4 design, phase specifications, pull-request review, implementation,
tests, or repository validation.

## Reviewer provenance

| Field | Value |
|---|---|
| Reviewer kind | Model-backed reviewer |
| Assistant-reported runtime identity | OpenAI GPT-5.6 Thinking |
| User-reported interface label | GPT-5.6 Sol, high reasoning |
| Review role | Architecture and complete-diff reviewer |
| Interaction surface | ChatGPT web |
| Repository access | Connected GitHub application |
| External references | Official WGPU and WebGPU documentation |
| Independence classification | Not inferred from model identity; model assessment is distinct from CI and acceptance |
| Validation authority | Repository-owned validation and exact-head GitHub Actions |
| Acceptance authority | None |

The report records auditable evidence, interpretation, contract impact, required
correction, and acceptance proof. It does not retain private chain-of-thought or treat
model output as a truth certificate.

## Subject identity

| Field | Value |
|---|---|
| Repository | `dornglut/runenwerk` |
| Parent program | Issue `#167` |
| G4 planning issue | Issue `#182` |
| G4A implementation issue | Issue `#186` |
| Pull request | PR `#199` |
| Accepted implementation base | `62c3949d31a7c03f1f554f8108120d9767139123` |
| Reviewed feature head | `324f82c126ef32cdee906d8c72860bd4bf6faebc` |
| Pull-request state at observation | Open draft and mergeable |
| Changed files at observation | 19 |
| Exact-head CI at observation | Green |
| Documentation build at observation | Green |
| Current acceptance | Not accepted |
| Downstream authorization | G4B and G4C remain blocked |

A moved pull-request head invalidates this report as a current acceptance review. The
report remains historical evidence and must be followed by a new review round against
the replacement head.

## Question

Does the current G4A implementation establish a correct, future-transferable RunenGPU
context and adapter/device admission authority, and are the accepted G4B and G4C plans
sufficiently coherent, bounded, and decision-complete to continue after G4A acceptance?

## Executive verdict

The overall G4 architectural direction is sound:

- RunenGPU becomes the owner of backend-neutral GPU contracts and private WGPU
  realization;
- raw WGPU authority is removed from reusable renderer-facing surfaces;
- context admission is asynchronous and headless-first;
- renderer-owned device creation is removed rather than retained behind an alias;
- G4B separates logical programs, interfaces, bindings, and pipelines from backend
  objects;
- G4C plans a clean realization and renderer-authority cutover.

The current G4A implementation is nevertheless **not ready for acceptance**.

Green validation demonstrates that the reviewed head compiles and satisfies the
current executable repository baseline. It does not resolve semantic defects in
presentation admission, downlevel capability reporting, limit and alignment truth,
portability classification, admission-report completeness, adapter ambiguity recovery,
environment-test strictness, or test isolation.

G4B must not start until G4A is corrected and accepted.

G4B's direction is strong, but its planning should be amended before implementation to
separate shader interfaces from vertex and render-target memory state, make WGSL
reflection mandatory validation evidence, establish the stateful owner of source
revision consistency, and demonstrate the intended public API ergonomics.

G4C is currently too broad for one safe implementation slice. It should remain one
umbrella outcome but be decomposed into resource realization, program/binding
realization, and pipeline/final-cutover deliveries.

## Scope

### Included

- the accepted G4 architecture and G4A/G4B/G4C specifications;
- issue `#186` and PR `#199`;
- every changed file in PR `#199` at the reviewed head;
- relevant unchanged owners and consumers needed to assess context, adapter, device,
  queue, renderer, capability, timing, and migration semantics;
- existing PR review rounds and their claimed corrections;
- exact-head validation evidence;
- official WGPU/WebGPU behavior relevant to limits, downlevel capabilities, adapter
  selection, and device creation;
- human readability, ownership clarity, implementation ergonomics, and cold-start
  executability.

### Excluded

- G5 command encoding, submission, progress, retirement, and readback;
- G7 reusable surface and device-loss architecture except where the temporary G4A seam
  affects ownership;
- RunenRender image-formation design beyond G4 boundary implications;
- package extraction;
- runtime performance benchmarking;
- a WGPU dependency-version upgrade;
- implementation of the recommended corrections.

## Review method

The assessment used this reviewable reasoning form:

```text
observed evidence
-> contract or ownership interpretation
-> defect, ambiguity, or risk
-> required correction
-> acceptance proof
```

Findings are not based on numerical confidence scoring. Each blocking finding must be
resolved, invalidated with evidence, or explicitly accepted by an authorized owner
where repository policy permits risk acceptance.

## Evidence inspected

### Repository authority

- `AGENTS.md`;
- `TESTING.md`;
- the canonical Runenwerk engineering workflow;
- the Runenwerk authority model;
- the Runenwerk documentation-structure contract;
- the accepted RunenGPU architecture;
- the accepted G4 design;
- the accepted G4 investigation;
- the G4A, G4B, and G4C phase specifications;
- the roadmap and active-work documents.

### GitHub authority and delivery evidence

- parent issue `#167`;
- planning issue `#182`;
- implementation issue `#186`;
- planning PR `#185` and accepted planning merge;
- implementation PR `#199`;
- all recorded review submissions on PR `#199`;
- accepted implementation base;
- reviewed feature head;
- exact-head validation and documentation-build state.

### External primary references

- [WebGPU specification](https://www.w3.org/TR/webgpu/);
- [WGPU 27 `Instance`](https://docs.rs/wgpu/27.0.1/wgpu/struct.Instance.html);
- [WGPU 27 `Limits`](https://docs.rs/wgpu/27.0.1/wgpu/struct.Limits.html);
- [WGPU 27 `DownlevelFlags`](https://docs.rs/wgpu/27.0.1/wgpu/struct.DownlevelFlags.html);
- [WGPU 27 `AdapterInfo`](https://docs.rs/wgpu/27.0.1/wgpu/struct.AdapterInfo.html).

The repository-pinned WGPU version, not the latest published WGPU release, is the
implementation reference for this review. A dependency-baseline decision belongs
before G4C and must not be mixed into the current G4A correction unless a concrete
compatibility defect requires it.

## File-review coverage

Coverage vocabulary:

| Coverage | Meaning |
|---|---|
| `complete_file` | Entire file inspected at the recorded revision |
| `complete_diff` | Every changed hunk inspected |
| `targeted_context` | Relevant unchanged declarations and surrounding behavior inspected |
| `consumer_census` | Direct and transitive consumers inspected |
| `authority_reference` | Used as accepted policy, design, or specification |
| `validation_evidence` | Used as validation evidence only |
| `external_reference` | External primary technical source |
| `not_reviewed` | Explicitly unavailable or outside scope |

### Changed-file census

| Path | Coverage | Role | Related findings |
|---|---|---|---|
| `docs-site/src/content/docs/workspace/planning/active-work.md` | `complete_diff` | Active-work projection | Documentation truth |
| `docs-site/src/content/docs/workspace/planning/roadmap.md` | `complete_diff` | Durable sequencing | G4B/G4C activation |
| `docs-site/src/content/docs/workspace/specs/pt-runengpu-g4a-context-admission.ron` | `complete_diff`, `authority_reference` | G4A contract | `G4A-CR-001` through `G4A-CR-015` |
| `engine/src/plugins/gpu/api/capability.rs` | `complete_file` | Capability requirement semantics | `G4A-CR-001`, `G4A-CR-007` |
| `engine/src/plugins/gpu/api/context.rs` | `complete_file` | Public context contract and pure admission | `G4A-CR-001`, `G4A-CR-004` through `G4A-CR-015` |
| `engine/src/plugins/gpu/api/mod.rs` | `complete_diff` | Public API exports | API completeness |
| `engine/src/plugins/gpu/backend/mod.rs` | `complete_diff` | Backend boundary | Ownership containment |
| `engine/src/plugins/gpu/backend/wgpu.rs` | `complete_file` | Private native realization | `G4A-CR-002` through `G4A-CR-006`, `G4A-CR-009` |
| `engine/src/plugins/gpu/mod.rs` | `complete_diff` | RunenGPU module surface | Ownership containment |
| `engine/src/plugins/render/adapters/gpu_capabilities.rs` | `complete_file` | Renderer capability migration | Migration truth |
| `engine/src/plugins/render/backend/device.rs` | `complete_diff` | Deleted device-request authority | Deletion completeness |
| `engine/src/plugins/render/backend/mod.rs` | `complete_diff` | Renderer backend exports | Boundary migration |
| `engine/src/plugins/render/backend/wgpu_ctx.rs` | `complete_file` | Temporary host and surface owner | G4A/G7 ownership |
| `engine/src/plugins/render/gpu_primitives/plan.rs` | `complete_diff` | Context-consumer migration | `G4A-CR-012` |
| `engine/src/plugins/render/renderer/mod.rs` | `complete_diff`, `targeted_context` | Renderer device/queue consumption | Temporary G4C bridge |
| `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs` | `complete_file` | Timing capability migration | Migrated capability facts |
| `engine/src/plugins/render/runtime/frame_submit.rs` | `complete_diff`, `targeted_context` | Runtime submission bridge | G5 exclusion |
| `engine/tests/gpu_context_admission.rs` | `complete_file` | Environment-dependent admission proof | `G4A-CR-011`, `G4A-CR-012` |
| `engine/tests/render_gpu_timing.rs` | `complete_file` | Migrated timing proof | Timing migration |

### Non-diff authority and context

| Artifact or area | Coverage | Purpose |
|---|---|---|
| `AGENTS.md` | `authority_reference` | Executor and evidence contract |
| `TESTING.md` | `authority_reference` | Exact-head and validation semantics |
| Runenwerk engineering workflow | `authority_reference` | Review, ownership, and stop conditions |
| Runenwerk authority model | `authority_reference` | Conflict and artifact ownership |
| Runenwerk documentation structure | `authority_reference` | Report placement and non-duplication |
| Issue `#186` | `authority_reference` | Active implementation contract |
| PR `#199` metadata and review history | `validation_evidence`, `authority_reference` | Delivery state and current verdict |
| Accepted G4 design | `authority_reference` | Durable target architecture |
| Existing G4 investigation | `authority_reference`, `targeted_context` | Accepted-main census and planning evidence |
| G4B and G4C specifications | `authority_reference` | Downstream planning assessment |
| WGPU/WebGPU documentation | `external_reference` | Backend and standard semantics |

## Review-dimension assessment

| Dimension | G4A | G4B | G4C |
|---|---|---|---|
| Correctness | Changes required | Planning amendment required | Not implemented |
| Scope coherence | Coherent boundary with semantic defects | Generally coherent | Too broad |
| Ownership direction | Strong direction | Strong direction | Requires atomic cutover slices |
| Dependency direction | Correct | Correct target | Correct target |
| Migration completeness | Partially proven | Not applicable | Insufficiently decomposed |
| Validation sufficiency | Green but not semantically sufficient | Not applicable | Not applicable |
| Documentation truth | Requires correction record | Amend specification | Amend and subdivide plan |
| Compatibility | Downlevel and limit truth incomplete | Interface compatibility incomplete | Cache compatibility over-specified |
| Ergonomics | Public concepts understandable; implementation concentrated | Concrete examples needed | Registry model too dense |
| Agent executability | Correctable through bounded findings | Needs more decision completeness | Unsafe as one broad delivery |
| Acceptance readiness | No | No implementation authorization | No implementation authorization |

## Findings and required fixes

### G4A implementation

| ID | Severity | Category | Finding and evidence basis | Required fix | Acceptance proof |
|---|---|---|---|---|---|
| `G4A-CR-001` | Blocking | Correctness | The headless terminal rejects every `Presentation` requirement before the common evaluator distinguishes required, preferred, and disabled semantics. | Remove the blanket pre-check and let the common typed admission algorithm own all three strengths. | Required rejects; preferred degrades only through its declared fallback; disabled succeeds. |
| `G4A-CR-002` | Blocking | Compatibility | Native headless adapter facts can advertise `Presentation` because absence of a surface is treated as compatibility. | Add presentation support only when explicit surface evidence exists and confirms compatibility. | Synthetic and real headless facts exclude presentation; compatible current-host facts may include it. |
| `G4A-CR-003` | Blocking | Compatibility | `Compute`, render/copy capability, and indirect execution are treated as broadly available without complete downlevel evidence. WGPU exposes downlevel flags specifically because these capabilities are not universal. | Map compute and indirect execution from `DownlevelCapabilities`; narrow coarse capabilities to the exact guaranteed operation subset. | Downlevel synthetic candidates reject unsupported compute and indirect workloads; mapping tests cover known and unknown flags. |
| `G4A-CR-004` | Blocking | Architecture | Adapter-supported limits, actual device limits, and RunenGPU workload policy are represented as one effective-limit authority. Caller maxima can therefore become fictional device facts. | Introduce distinct `GpuAdapterLimits`, `GpuDeviceLimits`, and `GpuWorkloadBudget` concepts. | Published device facts equal `device.limits()`; stricter caller maxima remain separately visible and enforced. |
| `G4A-CR-005` | Blocking | Correctness | Alignment requirements are evaluated against adapter facts but selected values are not fully carried into device creation and verified against the resulting device. | Request all requestable selected alignment limits and compare actual device alignments before publication; retain non-requestable transfer constraints separately. | Actual device alignment facts equal the admitted report; mismatches fail before `GpuContext` publication. |
| `G4A-CR-006` | Blocking | Validation | `requested_limits()` changes a small subset while inheriting WGPU defaults for unmodelled limits. Pure admission can pass a candidate that later fails device creation. | Bind explicit complete device profiles for modern portable, downlevel, and browser/WebGPU paths, or reject unsupported profiles before request. | For every admitted profile, the generated WGPU request is complete and predictable; downlevel fixtures cannot pass pure admission then fail due to hidden defaults. |
| `G4A-CR-007` | Blocking | Ownership | Capability-strength semantics exist in both the existing capability evaluator and context candidate admission. The presentation defect demonstrates drift. | Establish one pure feature-admission authority and layer format, limit, alignment, environment, and ranking checks around it. | Required/preferred/disabled semantics have one implementation and are exercised through both synthetic and native seams. |
| `G4A-CR-008` | Blocking | Documentation | `GpuAdmittedDeviceFacts` does not retain the complete normalized contract needed by G4C: actual device facts, format roles, alignments, required/preferred outcomes, verified disabled requirements, and complete dispositions. | Separate adapter facts, device facts, admission contract, and report; preserve each input requirement's typed disposition. | G4C can validate realization exclusively from admitted facts without reconstructing intent or consulting adapter maxima. |
| `G4A-CR-009` | Blocking | Compatibility | Portability is derived from backend preference and a narrow extension heuristic rather than the admitted contract. A preference is not a requirement. | Introduce explicit baseline, declared-extension, and backend-specialization policy; derive the class and reason set from admitted requirements and contained backend dependencies. | Baseline, declared-extension, specialized, degraded, and unsupported outcomes have deterministic tests and explicit evidence. |
| `G4A-CR-010` | Major | Usability | Ambiguous candidates are correctly rejected, but the public descriptor has no way to retry with one of two otherwise identical devices. Diagnostic names also risk becoming semantic tie-breakers. | Return process-local, nonpersistent candidate IDs and permit exact retry selection; keep names diagnostic only. | Ambiguity reports expose selectable IDs; changing a diagnostic name does not change implicit selection. |
| `G4A-CR-011` | Blocking | Validation | One `NoCandidate` terminal conflates no adapter with all observed candidates being rejected. This permits environment tests to hide admission regressions. | Split `NoAdapterAvailable` from `NoAdmissibleCandidate { dispositions }`. | Tests skip only genuine absence; rejected candidates fail and retain complete dispositions. |
| `G4A-CR-012` | Major | Validation | Environment and migrated runtime tests return early for broad context/backend errors. Device-request or semantic admission regressions can therefore pass. | Skip only typed environment absence; fail on ambiguity, semantic rejection, adapter request defects, and device request failures. | Injected failures produce failing tests; a machine with no adapter produces the one accepted skip outcome. |
| `G4A-CR-013` | Major | Correctness | Context-ID exhaustion tests reset a process-global atomic allocator and can race under parallel Rust tests. | Extract an allocator object; retain a static production instance and use isolated local instances in tests. | Parallel tests require no global reset and deterministically prove exhaustion behavior. |
| `G4A-CR-014` | Major | Documentation | `GpuContextRequestError::Display` reports only a category and hides bounded actionable detail and candidate reasons. | Format category, safe detail, and a concise rejection summary while retaining structured accessors. | Ordinary error output is actionable without `Debug`, bounded, UTF-8 safe, and secret-free. |
| `G4A-CR-015` | Major | Maintainability | `api/context.rs` and `backend/wgpu.rs` concentrate identity, descriptors, facts, admission, limits, ranking, errors, backend mapping, device creation, and host bridging. | Decompose by responsibility after semantic correction without introducing a compatibility layer or second authority. | Modules have one clear responsibility, tests remain owner-local, and the public surface is unchanged except for accepted fixes. |

### G4B planning

| ID | Severity | Category | Finding and reasoning | Required fix before activation | Acceptance proof |
|---|---|---|---|---|---|
| `G4B-CR-001` | Blocking before activation | Architecture | Shader vertex interface and vertex-buffer memory layout risk duplicate authority. Shader locations/types and byte layout are related but not identical. | Program interface owns location and numeric shader type; vertex layout owns slot, stride, step mode, format, and offset; add explicit compatibility validation. | One interface can be paired with multiple compatible memory layouts without restating shader authority. |
| `G4B-CR-002` | Blocking before activation | Architecture | Fragment shader output and color-target format/blend state risk the same duplication. | Program interface owns output location/type; pipeline target owns format, blend, and write mask. | Compatibility is validated without embedding target state into the shader contract. |
| `G4B-CR-003` | Blocking before activation | Validation | WGSL reflection is not clearly mandatory evidence, so entry-point, stage, binding, and override disagreement may be unprovable. | Require Naga parse, validation, and reflection for the WGSL path; explicit descriptors remain authority and reflection may only validate, never mutate. | Wrong stage, missing entry point, binding mismatch, and override mismatch reject deterministically. |
| `G4B-CR-004` | Major | Ownership | `SourceRevisionConflict` implies state, but no logical state owner is defined. | Establish a backend-neutral admitted-program catalog that owns key/revision/source consistency and contains no WGPU objects. | Equal key/revision with different source bytes rejects before G4C realization. |
| `G4B-CR-005` | Major | Ergonomics | The intended public API remains abstract and could grow into repeated descriptor embedding or nested finalization. | Add complete compute and render examples covering source, interface, layouts, specialization, bindings, compatibility errors, and pipeline admission. | Examples compile and demonstrate one ordinary path without caller-supplied duplicate correctness facts. |
| `G4B-CR-006` | Major | API evolution | Public enums and future extensions such as bindless or sparse resources lack an explicit pre-1.0 evolution policy. | Record an additive/nonexhaustive or versioned-extension policy without implementing those capabilities now. | Later capability growth has one documented compatibility path and cannot silently reinterpret existing descriptors. |

### G4C planning

| ID | Severity | Category | Finding and reasoning | Required fix before activation | Acceptance proof |
|---|---|---|---|---|---|
| `G4C-CR-001` | Blocking before activation | Scope | G4C combines ten object families, registries, caches, consumer migration, sidecar cleanup, and final authority deletion. One PR would be difficult to review and unsafe to recover. | Retain issue `#188` as umbrella; create ordered G4C1 resource realization, G4C2 program/binding realization, and G4C3 pipeline/final-cutover slices. | Each slice migrates and deletes one complete authority without leaving an accepted parallel system. |
| `G4C-CR-002` | Blocking before activation | Correctness | Registry concurrency semantics are not bound: reservation, lock hierarchy, duplicate waiters, backend creation, failure cleanup, and WASM/native assumptions remain open. | Specify an in-progress reservation model, creation outside locks, waiter wakeup, retry/failure cleanup, and target-specific synchronization. | Concurrent equal requests realize once; failure leaves no stuck reservation or poisoned logical identity. |
| `G4C-CR-003` | Blocking before activation | Validation | Concurrent WGPU shader and pipeline failures need request-local error-scope attribution. | Define push/pop error-scope ownership and serialization or correlation rules for every realization terminal. | Parallel failing realizations are attributed to the correct logical request. |
| `G4C-CR-004` | Major | Scope | Context-local in-memory caches are specified with redundant context, driver, backend, schema, and corruption semantics suited to persistence that does not exist. | Scope registries to one context/generation and retain only semantic correctness inputs; defer corruption and persistent compatibility envelopes. | Initial outcomes are `Hit`, `Miss`, `DescriptorConflict`, and `BackendCreationFailed`; no unused persistence subsystem appears. |
| `G4C-CR-005` | Major | Ownership | The temporary G5 bridge can become a broad raw-WGPU escape hatch if callers may retain, clone, or use it for lookup authority. | Keep the bridge borrowed, opaque, operation-scoped, and limited to already admitted realizations; prohibit storage and return. | Visibility and source guards prove exactly one bounded call path with a G5 deletion owner. |
| `G4C-CR-006` | Major | Correctness | Batch graph realization semantics are unclear when some individual cache entries succeed and a later object fails. | Permit reusable successful entries to remain but publish no executable graph batch until every required realization succeeds. | Partial failure cannot expose an incomplete execution set; a retry can reuse valid independent entries. |
| `G4C-CR-007` | Major | Ownership | Resource registry lifetime and retirement boundaries are not explicit while runtime retirement belongs to G5. | Define the G4C lifetime envelope and exact handoff to G5 without implementing completion/retirement early. | G4C cannot silently destroy or reuse resources based on unowned progress assumptions. |
| `G4C-CR-008` | Major | Scope | Resource realization could absorb uploads through convenience APIs such as initialized-buffer creation, crossing into G5 transfer work. | Restrict G4C to object realization and metadata; keep initial content/upload operations in the separately authorized execution boundary. | Source and API tests show no hidden upload or submission operation in G4C. |

## Detailed rationale for the current PR blocker

### Headless presentation semantics

Presentation is surface-dependent. Headless admission has no presentation compatibility
evidence, but absence of a surface does not make every presentation-related declaration
equivalent.

| Requirement | Correct headless outcome |
|---|---|
| Required presentation | Reject |
| Preferred presentation with explicit continue-without fallback | Admit with one degradation record |
| Disabled presentation | Admit and verify disabled |

The current blanket rejection bypasses the accepted typed requirement model and creates
a second source of capability-strength semantics.

### Headless presentation facts

No surface evidence means presentation compatibility is unavailable, not supported. A
native headless context must therefore not publish `Presentation` in adapter facts.
Current-host selection may publish it only for a candidate proven compatible with the
specific host surface evidence.

## Limit and alignment model required by G4A

The current implementation should be corrected around three independent facts:

```text
GpuAdapterLimits
    support observed before device creation

GpuDeviceLimits
    actual limits exposed by the successfully created device

GpuWorkloadBudget
    RunenGPU policy applied to admitted workloads and realization
```

A caller maximum such as "this workload may use at most X" is policy, not proof that
the physical or logical device maximum equals X. Device facts must be read back from
the created device and compared with the admitted contract before publication.

The same distinction applies to alignments:

- adapter facts support candidate evaluation;
- requestable selected alignments must enter the WGPU device request;
- actual device alignments must be recorded after creation;
- operation-specific non-requestable transfer alignment remains a separate admitted
  constraint.

## G4B amendments required before issue `#187` activates

The G4B specification should retain its current direction:

- WGSL-first source authority;
- typed binding keys;
- explicit interfaces;
- deterministic specialization;
- no string, `TypeId`, or naked hash as compatibility authority;
- byte-preparation derives do not become universal ABI authority.

Before implementation, bind these decisions:

1. shader vertex inputs own location and numeric shader type;
2. vertex-buffer layouts own slot, stride, step mode, storage format, and byte offset;
3. a compatibility validator joins those contracts;
4. fragment outputs own location and shader type;
5. color targets own format, blend state, and write mask;
6. Naga parsing, validation, and reflection are mandatory evidence for WGSL;
7. reflection validates but never mutates the explicit interface;
8. a backend-neutral program catalog owns source-key/revision/content consistency;
9. pipeline descriptors reference admitted immutable program and layout contracts
   instead of repeatedly embedding the same identity;
10. the specification contains complete compute and render authoring examples and
    representative structured errors.

## G4C decomposition required before issue `#188` activates

### G4C1 — resource realization

Own:

- buffers;
- textures;
- texture views;
- samplers;
- query sets;
- generation-bound resource registries;
- renderer resource-authority deletion.

Exit gate:

- every migrated resource family has one RunenGPU realization authority;
- replaced renderer allocation authority is deleted;
- no forwarding alias or secondary cache remains.

### G4C2 — program and binding realization

Own:

- shader modules;
- bind-group layouts;
- pipeline layouts;
- bind groups;
- admitted-program realization;
- matching renderer cache deletion.

Exit gate:

- program/interface/layout compatibility is validated before WGPU creation;
- context-local realization caches have deterministic semantic keys;
- duplicate renderer program and binding caches are deleted.

### G4C3 — pipeline realization and final cutover

Own:

- compute pipelines;
- render pipelines;
- pipeline realization caches;
- final renderer migration;
- bounded temporary G5 bridge;
- remaining reusable raw-WGPU authority deletion.

Exit gate:

- renderer code no longer owns reusable WGPU realization;
- the temporary execution bridge is the only remaining raw backend seam;
- every remaining seam has an explicit G5 or G7 deletion owner.

## Validation assessment

The reviewed head had green exact-head CI and documentation-build evidence at the
observation time.

This establishes:

- repository automation selected the reviewed revision;
- the repository's current canonical checks passed for that revision;
- the documentation build passed for that revision;
- the branch was not failing the current executable baseline.

It does not establish:

- semantic correctness of normalized GPU facts;
- agreement between published facts and the actual created device;
- correctness on downlevel or browser environments not exercised;
- completeness of the capability model;
- absence of architectural ownership defects;
- acceptance readiness while blocking findings remain.

The correct evidence statement is:

```text
validation: pass at reviewed head
critical review: changes required
acceptance: blocked
```

## Evidence limitations

- Connected GitHub data was used for repository files, PR state, issues, review history,
  and exact-head workflow evidence.
- The review did not independently reproduce the complete branch in a local checkout.
- Source inspection establishes structure and statically demonstrable semantic defects;
  it is not runtime execution proof.
- Hardware- and platform-dependent behavior remains bounded by the environments
  actually exercised.
- The user-reported interface label is provenance only. It does not establish runtime
  identity, independence, validation, or acceptance authority.
- This report contains reviewable rationale, not private chain-of-thought.

## Required correction sequence

1. Keep PR `#199` draft.
2. Correct G4A presentation semantics and surface-specific capability evidence.
3. Correct downlevel capability mapping.
4. Split adapter, device, and workload-limit authority.
5. Request and verify alignment facts against the actual device.
6. Bind explicit complete WGPU device profiles.
7. Consolidate capability-strength admission into one evaluator.
8. Expand adapter, device, admission-contract, and disposition facts.
9. Correct portability policy and derivation.
10. Add adapter ambiguity recovery.
11. Split environment absence from candidate rejection.
12. Harden integration-test skip behavior.
13. Remove the global allocator test race.
14. Decompose the context and backend modules after semantic correction.
15. Rerun focused checks, `cargo validate`, `git diff --check`, and the documentation
    build.
16. Obtain new exact-head CI at one unchanged corrected head.
17. Perform a new complete-diff review against that exact head.
18. Merge G4A only after no blocking findings remain.
19. Amend G4B before activating issue `#187`.
20. Amend and subdivide G4C before activating issue `#188`.

## Review-round history

| Round | Reviewed head | Date | Verdict | Status |
|---:|---|---|---|---|
| Prior GitHub rounds | Earlier PR heads retained in the PR timeline | 2026-07-29 | Changes required | Historical |
| Current report round | `324f82c126ef32cdee906d8c72860bd4bf6faebc` | 2026-07-31 | Changes required | Current for the reviewed code |
| Corrected-head round | Pending | Pending | Not assessed | Required after branch movement |
| Final acceptance round | Pending | Pending | Not assessed | Required before merge |

When the head changes, the current report round becomes stale for acceptance but
remains valid historical evidence for the code that was reviewed. Findings retain
their IDs and receive explicit resolution revisions and reviewer confirmation rather
than being silently rewritten as if they never existed.

## Final disposition

### G4A

**Changes required.**

The architectural direction should be preserved. Repair the new RunenGPU authority
rather than restoring renderer-owned WGPU device authority.

### G4B

**Remain blocked.**

Amend the specification before activation.

### G4C

**Remain blocked.**

Retain the umbrella outcome but decompose the implementation before activation.

### RunenGPU program

**Continue after correction.**

The review does not recommend abandoning G4 or reverting to direct renderer-owned WGPU
access. It recommends completing the intended ownership transfer with truthful
capability facts, exact device-state reporting, one admission authority, bounded
implementation slices, and explicit acceptance evidence.
