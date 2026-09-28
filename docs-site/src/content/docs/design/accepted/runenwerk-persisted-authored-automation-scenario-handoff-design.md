---
title: Runenwerk Persisted Authored Automation Scenario Handoff Design
description: Accepted design for strict one-product authored scenario artifacts, product-owned persisted steps, local artifact references, and terminal handoff through AutomationSession.
status: accepted
owner: engine
layer: engine-runtime / integration
canonical: true
last_reviewed: 2026-09-27
publication: reference
pagefind: false
related_designs:
  - ./runenwerk-application-automation-session-semantic-model.md
  - ./runenwerk-terminal-automation-orchestration-and-authored-scenario-design.md
  - ./runenwerk-app-runtime-and-composition-semantic-model.md
---

# Runenwerk Persisted Authored Automation Scenario Handoff Design

## Status and scope

This design is the A14 persistence decision after accepted A13.

Reviewed accepted state at preparation:

```text
RUNENWERK_MAIN=5e47c0d8faad1f7898dfb9153f14dc1f7d5bf849
A12_MERGE=9c894ff25a986d87f5f516b76f843a46f233882d
A13_MERGE=cb0a5e2d33ff8e3848cd442a0897aab05f4a7c71
RUNENINPUT_PIN=2751e19fa42255b86e786e7cd837198c917b7b25
```

A13 establishes two maintained product-local terminal automation callers:

- Render Lab persisted NormalizedInput replay;
- Editor ProductSemantic command/query/assertion.

The remaining A0 workflow gap is persisted handoff of **authored multi-step intent**.

This design fixes that persistence boundary. It does not authorize a central CLI, product
registry, multi-product orchestration, attach/IPC, native OS injection, or a generic
cross-product command model.

## Current mechanism disposition

| Mechanism | Disposition | Reason |
| --- | --- | --- |
| A12 `AutomationSession` | retain | one runtime orchestration owner |
| A8 persisted normalized trace | retain | observed input artifact, not authored scenario |
| Render Lab `--replay-trace` | retain | useful direct trace workflow; not replaced by scenario persistence |
| Editor `--automation-viewport-tool` | retain | useful bounded direct product command |
| product-owned adapters | retain | commands/queries/targets/observations stay typed |
| RON + Serde in Engine | reuse | already maintained for strict bounded automation persistence |
| product-local filesystem/path UX | retain | A11 assigns path/tool UX to caller |
| central `runenctl` | defer | no meaningful product launch/discovery duplication yet |
| generic product/provider registry | reject for V1 | would duplicate product authority |
| persisted generic result/history | defer | no current handoff consumer requires another result schema |
| trace -> scenario generator | defer | transformation needs authored semantic intent |

## Decision summary

Persisted authored scenarios use a **shared strict one-product envelope parameterized by a
product-owned persisted step type**.

The lowering is:

```text
scenario file
  -> shared envelope probe / bounds
    -> expected product-contract identity/version check
      -> typed product-step decode + validation
        -> product-local launch
          -> one AutomationSession
            -> sequential typed lowering
              -> finish on success
              -> cancel on first failure
```

The artifact is data. It is not a second runtime or scheduler.

## Post-A13 central terminal decision

Do not introduce a central `runenctl` in the first persisted-scenario implementation.

Render Lab and Editor share Engine orchestration but intentionally differ in:

- command-line surface;
- App construction;
- target selection;
- owner commands and assertions;
- product output.

Each existing binary is already an unambiguous product selector.

A central executable would currently add:

- a product catalog;
- build/release coupling between applications;
- type-erasure or static dispatch plumbing;

without deleting substantial semantic duplication.

Reconsider a central terminal only after product-local `--automation-scenario` callers
demonstrate repeated launch/discovery plumbing that is not product semantic policy.

## Scenario V1 identity

The shared artifact kind is:

```text
runenwerk.automation.authored-scenario
```

The shared envelope schema version begins at:

```text
1
```

Envelope version and product-contract version are separate compatibility axes.

Conceptually:

```text
PersistedAutomationScenarioV1<ProductStep> {
    artifact_kind
    schema_version
    product_contract_id
    product_contract_version
    steps
}
```

The envelope contains no implicit current product, no binary filename identity, and no
runtime registry key.

## One product per scenario

V1 binds exactly one product automation contract:

```text
one artifact
  -> one product contract
  -> one launch-owned App
  -> one AutomationSession
```

Cross-product scenarios, multiple launched Apps, subprocess choreography, and remote
orchestration are not V1 capabilities.

This matches A0's explicitly selected application boundary and keeps failure/cleanup ownership
unambiguous.

## Product-contract identity

A product that supports scenario persistence defines a stable automation contract ID and
contract version.

First implementation pressure contracts are:

```text
runenwerk.editor.automation      version 1
runenwerk.render-lab.automation  version 1
```

These identifiers name automation persistence contracts, not executable filenames.

The product binary supplies its expected ID/version to the importer. Mismatch fails before
typed step execution.

No global registry is required.

V1 product-contract IDs use a canonical portable grammar:

- total UTF-8/ASCII length: 1–128 bytes;
- lowercase ASCII dot-separated segments;
- every segment is non-empty;
- each segment begins with `[a-z0-9]`;
- remaining segment characters are `[a-z0-9-]*`.

Product contract version is a nonzero `u32`.

Case, whitespace, path-like spellings, and executable aliases are not accepted identities.

## Canonical launch fixture contract

The product-contract ID/version binds not only persisted owner-step syntax but also the
canonical launch fixture expected by those steps.

For V1:

```text
runenwerk.editor.automation/v1
  -> fresh launch-owned headless Full Editor
  -> canonical default composition

runenwerk.render-lab.automation/v1
  -> fresh launch-owned headless Render Lab automation App
```

The generic scenario envelope does not contain an arbitrary fixture selector.

If a product changes its canonical fixture in a way that invalidates scenario meaning or
target selection, the product automation contract version must change.

V1 does not claim repository-revision identity. Exact source revision remains external
execution evidence (for example CI or packaged build provenance). Product contract version is
the semantic compatibility boundary; a generic revision string would not make an incompatible
fixture safe.

## Product-owned persisted step schema

The shared layer does not persist runtime `AutomationOwnerAdapter::Command`, `Target`, or
`Query` values directly.

Each product owns a small explicit persistence DTO, for example conceptually:

```rust
enum EditorAutomationScenarioStepV1 {
    ActivateViewportTool {
        target: EditorScenarioTargetV1,
        tool: EditorViewportToolV1,
    },
    AssertViewportTool {
        target: EditorScenarioTargetV1,
        tool: EditorViewportToolV1,
    },
}
```

The persisted DTO is distinct from the runtime command/query types.

The product owns:

- Serde shape;
- unknown-field/variant strictness;
- product-specific resource validation;
- target-selector meaning;
- lowering into current typed runtime values.

The shared layer owns only the envelope and generic step wrappers.

## Narrow product codec contract

The first implementation may use a small generic persistence contract equivalent to:

```text
product contract ID
product contract version
Serialize / strict Deserialize
validate persisted product step
```

This is a codec contract only.

It MUST NOT expose:

- App construction;
- runtime target lookup;
- command/query dispatch;
- capability discovery;
- provider registration.

Therefore it is not the provider/harness trait deferred by A11.

## Shared step forms

V1 needs only two shared step forms:

```text
Owner(ProductStep)
ReplayNormalizedTrace(RelativeArtifactRef)
```

No generic command/query/assertion enum is introduced.

A V1 scenario may contain **at most one** `ReplayNormalizedTrace` step.

A12 intentionally retains a completed replay lease through later owner query/assertion until
finish/cancel. Supporting a second replay would require an intermediate teardown/reacquisition
contract that current authored intent does not need. V1 does not invent that lifecycle step.

No explicit persisted `Finish` or `Cancel` step exists.

Lifecycle closure belongs to the executor:

- all authored steps succeed -> `AutomationSession::finish`;
- first authored step fails -> stop execution and attempt `AutomationSession::cancel`.

The primary authored-step failure remains primary. Cleanup failure is additional truthful
failure/history and must not overwrite it.

## Normalized replay step

`ReplayNormalizedTrace` means exactly the accepted NormalizedInput replay path.

It references an A8 V1 artifact and lowers through A12:

```text
resolve local artifact
  -> bounded read
    -> A8 import
      -> A12 session replay
        -> A6 replay
```

The scenario schema does not duplicate A8 DTOs.

The scenario does not infer expected product state from the trace.

The scenario does not persist
`AutomationInputReplayStateAssumption::RecordedAndReplaySourcesPristine`.

That value is a runtime/environment witness. A fresh product launch fixture may establish and
supply it at execution time where truthful.

A persisted artifact cannot assert facts about future runtime input-source state.

## No generic execution-mode field in V1

V1 does not persist an arbitrary `AutomationExecutionMode`.

The shared replay step is intrinsically NormalizedInput.

Product-owned V1 steps lower only to the product-semantic operations explicitly defined by
that product contract.

NativeOs has no V1 scenario representation.

This prevents a persisted file from escalating fidelity or permissions merely by naming a
mode.

A future execution mode requires a new reviewed persistence contract.

## Editor V1 pressure contract

The first Editor persisted target selector is product-owned:

```text
UniqueSceneViewport
```

It means: resolve exactly one currently mounted scene viewport through Editor lifecycle
identity.

Zero or more than one match fails closed.

A representative authored scenario is:

```text
Owner(
  ActivateViewportTool {
    target: UniqueSceneViewport,
    tool: Rotate,
  }
)
Owner(
  AssertViewportTool {
    target: UniqueSceneViewport,
    tool: Rotate,
  }
)
```

Lowering remains:

```text
resolve current Editor target
  -> ProductSemantic dispatch
    -> typed owner query
      -> typed owner assertion
```

The persisted selector is not a universal application target.

## Render Lab V1 pressure contract

A representative Render Lab scenario is:

```text
ReplayNormalizedTrace("traces/orbit-pan-zoom.ron")
Owner(
  AssertCamera {
    expected: product-owned camera expectation,
  }
)
```

The replay step proves delivery through A6/A12.

The product-owned camera assertion provides semantic confirmation.

Replay completion alone remains `AdmittedOrDelivered`, not `EffectConfirmed`.

## Relative artifact references

V1 uses a portable UTF-8 relative artifact reference.

The reference syntax is slash-separated and MUST reject:

- absolute paths;
- platform root/prefix syntax;
- empty path;
- empty segments;
- `.`;
- `..`;
- backslash;
- NUL/control characters.

The encoded path is bounded to 1,024 UTF-8 bytes.

The scenario persistence layer validates reference syntax but does not open files.

## Scenario root and filesystem resolution

Filesystem resolution remains terminal/tool caller responsibility.

For `--automation-scenario <path>`, the caller:

1. canonicalizes the scenario file;
2. uses the canonical scenario parent directory as the scenario root;
3. joins validated reference segments under that root;
4. canonicalizes the referenced existing target;
5. verifies the canonical target remains beneath the canonical scenario root;
6. performs a bounded read;
7. passes bytes to the owning importer/session operation.

This rejects symlink escapes as well as textual `..` traversal.

Remote URLs are unsupported.

The handoff unit may therefore be a small local directory:

```text
scenario.ron
traces/
  orbit-pan-zoom.ron
```

No archive/container format is required for V1.

## File IO is not scenario semantics

The shared scenario schema owns relative-reference validity.

It does not own:

- current working directory;
- path display;
- file-open diagnostics;
- packaging UI;
- network fetching.

Those remain caller/tool concerns as established by A11.

A shared helper may implement safe root resolution to avoid security-sensitive duplication,
but using that helper does not move product semantics into Engine.

## Serialization format

V1 uses RON.

Reason:

- Engine already maintains RON + Serde;
- A8 proves strict recursion-bounded RON parsing;
- authored scenarios benefit from reviewable text;
- both first product crates already use Serde;
- no currently maintained alternative materially improves this contract.

RON selection is local to this schema. It does not imply every future automation artifact
must use RON.

## Shared resource bounds

V1 shared limits are:

```text
scenario bytes                  <= 1 MiB
RON recursion depth             <= 64
steps                           1..=4,096
product-contract ID bytes       <= 128
relative artifact reference     <= 1,024 bytes
ReplayNormalizedTrace steps     <= 1
```

The 1 MiB scenario limit is independent of A8's trace byte limit.

Referenced trace bytes continue to use A8's existing `MAX_ARTIFACT_BYTES` limit.

Product step schemas MUST define additional limits for any product-owned strings,
collections, numeric domains, or referenced resources.

Product-owned V1 step payloads MUST NOT introduce raw arbitrary filesystem paths. Any future
product-owned external artifact reference must use the shared validated relative-reference
type or receive separate reviewed path semantics.

The outer byte limit prevents an unbounded product DTO from bypassing the first memory
boundary, but it is not a substitute for product semantic validation.

## Strict import sequence

Import proceeds in phases:

```text
byte limit
  -> RON recursion-bounded envelope probe
    -> artifact kind
    -> envelope schema version
    -> product contract ID/version
      -> full typed decode using expected ProductStep
        -> shared step/resource validation
          -> product-owned persisted-step validation
```

Wrong artifact kind, future envelope version, wrong product identity/version, non-canonical
product identity, zero product-contract version, empty scenario, unknown fields, unknown variants,
malformed values, and resource-limit violations all fail closed.

A raw A8 trace passed to the scenario importer fails artifact-kind validation.

## Runtime representation

A successfully imported scenario is an inert typed value.

It contains:

- verified product-contract identity;
- ordered shared/product step values.

It owns no App and no AutomationSession.

Execution is a separate product-local operation.

This preserves:

```text
persisted scenario != runtime session != observed trace
```

## Sequential execution

V1 executes authored steps in listed order.

The product-local executor lowers each step into accepted A12 operations.

It stops at the first unsuccessful authored step.

Success path:

```text
all steps succeed
  -> finish
  -> success exit
```

Failure path:

```text
first step failure
  -> preserve primary failure
  -> cancel / cleanup
  -> report primary + cleanup failure if cleanup also fails
  -> nonzero exit
```

No hidden retries or semantic fallback are introduced by the scenario layer.

## Scenario-step identity and session-history correlation

V1 scenario-step identity is its zero-based ordinal in the immutable imported step sequence.

A single authored step may lower to multiple A12 operations. For example:

```text
scenario step 1: AssertViewportTool(Rotate)
  -> OwnerQuery
  -> OwnerAssertion
```

The executor correlates them without modifying `AutomationSession`:

1. record the current session-history sequence/length before lowering the scenario step;
2. lower the typed step;
3. record the resulting half-open history range;
4. on failure, report the authored step ordinal plus the typed/session cause.

This mapping is an in-memory execution result only in V1.

No persisted generic execution-history schema or arbitrary stable step ID is introduced.

If later editing/composition requires identity that survives step reordering, V1 ordinals are
insufficient and a later scenario schema must decide stable authored-step identity explicitly.

## Assertions and waits

Assertions remain product-owned persisted steps.

A product assertion lowers to typed query/assertion behavior and preserves:

- definitive false -> `AssertionFailed`;
- unavailable/timeout -> `Inconclusive`;
- infrastructure failure -> `InfrastructureFailure`.

V1 does not introduce a generic persisted predicate language.

A product may define a bounded persisted wait/assert step if its owner semantics require it.

Generic wait persistence is deferred until at least two product contracts demonstrate a
shared non-product-specific structure.

## Result and evidence handoff

V1 does not define a persisted generic result artifact.

A terminal execution returns:

- process exit status;
- bounded product-local stdout/stderr summary;
- A12 in-memory ordered session history available to tests/tooling in-process.

This is sufficient to prove handoff execution without freezing another persistence schema.

A persisted result/history artifact requires a separate consumer-driven design.

## Security

Scenario files and referenced artifacts are untrusted input.

V1 therefore requires:

- bounded scenario bytes;
- bounded recursion and step counts;
- strict unknown-field/variant rejection;
- stable expected product-contract identity;
- product-specific step validation;
- safe relative artifact references;
- canonical-root containment for referenced files;
- bounded referenced artifact reads;
- no URLs;
- no shell command steps;
- no arbitrary environment mutation;
- no native OS injection representation;
- no implicit capability escalation.

Target ambiguity/staleness remains product-owned and fail-closed.

A12 remains responsible for automation-owned cleanup after partial execution.

## Trace -> scenario transformation boundary

A future trace-to-scenario tool may:

- import A8 trace evidence;
- emit an editable product-owned scenario draft;
- reference the original trace artifact;
- ask the author to add/confirm product target selectors and semantic assertions.

It MUST NOT:

- infer that replay success proves semantic correctness;
- synthesize product assertions without product/author authority;
- mutate the original trace into scenario authority.

No transformation tool is authorized by A14.

## Pressure-test lowering

### P1 — Editor handoff

```text
runenwerk_editor --automation-scenario scenario.ron
  -> strict scenario import expects runenwerk.editor.automation/v1
    -> launch fresh headless Full Editor
      -> Owner ActivateViewportTool(UniqueSceneViewport, Rotate)
        -> A12 ProductSemantic dispatch
      -> Owner AssertViewportTool(UniqueSceneViewport, Rotate)
        -> typed query + AssertionPassed
      -> finish
      -> bounded success summary
```

Wrong product identity, unknown tool variant, zero/multiple viewport target, assertion mismatch,
or cleanup failure exits nonzero.

### P2 — Render Lab trace-reference handoff

```text
runenwerk-render-lab --automation-scenario scenario.ron
  -> strict scenario import expects runenwerk.render-lab.automation/v1
    -> resolve traces/orbit-pan-zoom.ron under scenario root
      -> bounded A8 read
        -> fresh launch-owned headless Render Lab
          -> runtime supplies truthful pristine replay witness
            -> A12/A6 replay
              -> product camera assertion
                -> finish
```

The trace remains an A8 artifact.

### P3 — malformed/mismatch/path failure

The following fail before semantic success:

- A8 trace supplied as scenario -> WrongArtifactKind;
- future envelope version -> UnsupportedSchemaVersion;
- wrong product-contract ID/version -> ProductContractMismatch;
- unknown product step -> typed parse failure;
- missing reference -> caller IO failure;
- `../trace.ron` or absolute path -> invalid reference;
- symlink escape -> root containment failure;
- ambiguous Editor viewport -> product failure;
- assertion mismatch -> AssertionFailed;
- replay failure -> A12/A6 failure;
- cleanup failure -> primary failure plus cleanup failure/history.

### P4 — NativeOs

V1 has no NativeOs representation.

No scenario can obtain native input by changing a string field.

### P5 — trace is not scenario

Artifact kinds are distinct:

```text
runenwerk.automation.normalized-replay-trace
runenwerk.automation.authored-scenario
```

The scenario importer probes and rejects the first when the second is expected.

## First implementation successor

After A14 acceptance, create exactly one A15 implementation issue.

A15 should:

1. add a shared Engine scenario-persistence module with:
   - strict generic V1 envelope;
   - product codec identity/version contract;
   - RON import/export;
   - shared limits;
   - validated relative artifact references;
2. leave filesystem opening/resolution at the terminal/tooling boundary, with one reusable
   safe-root helper only if current code proves it avoids duplicated security logic;
3. add Editor product-owned V1 persisted steps for:
   - unique-scene-viewport target;
   - activate viewport tool;
   - assert viewport tool;
4. add `runenwerk_editor --automation-scenario <path>` while retaining A13 direct mode;
5. add Render Lab product-owned V1 persisted camera assertion step;
6. add `runenwerk-render-lab --automation-scenario <path>` while retaining A9 direct
   `--replay-trace`;
7. prove a Render Lab scenario referencing a separate A8 trace file;
8. prove actual terminal processes for Editor and Render Lab;
9. prove wrong-kind/version/product/unknown-step/path-escape/missing-reference/assertion-failure
   behavior;
10. use success -> finish and failure -> cancel semantics through A12;
11. add no central CLI/provider registry, generic result persistence, multi-product scenario,
    attach/IPC/native path, A6/A8 redesign, or new repository.

No A13/A9 direct terminal mode is removed by A15. They remain useful narrow workflows.

## Central CLI successor gate

Do not reconsider a central terminal merely because A15 adds two
`--automation-scenario` flags.

Reassess only if their implementations duplicate concrete product selection, launch, or
scenario-file handling beyond small caller UX.

If a shared terminal is later justified, it remains a caller and must not own product step
semantics.

## Persisted result successor gate

Do not add persisted generic execution history after A15 by default.

Require an explicit external handoff consumer that cannot use process exit status,
product-local output, or in-process history.

## ADR disposition

No new ADR is required.

This design does not transfer semantic authority between repositories or introduce a new
runtime/service boundary. It defines a Runenwerk-owned persistence projection of already
accepted A1/A11/A12 automation semantics.

A standalone automation repository, remote service, or cross-process attach protocol would
require a new decision.

## Non-goals

A14 does not authorize:

- central `runenctl`;
- global product/provider registry;
- universal product command/query/target/assertion schema;
- multi-product scenario;
- generic persisted result/history;
- trace -> scenario generator;
- IPC/attach/remote control;
- native OS injection;
- native-window fidelity claims;
- A6 replay redesign;
- A8 trace schema change;
- A12 session redesign;
- RunenInput semantic changes;
- RunenUI or Scene replay absorption;
- new repository/service.

## Acceptance and implementation gate

The design is acceptable only if cold review confirms:

- the shared envelope does not own product semantics;
- product persistence DTOs remain distinct from runtime owner types;
- A8 trace identity and schema remain separate;
- runtime replay freshness is not persisted as an artifact fact;
- local artifact references cannot escape the scenario root;
- lifecycle finish/cancel remains executor-owned;
- authored step ordinals correlate deterministically to A12 history ranges without modifying session history;
- product payloads cannot bypass shared artifact-reference path rules;
- result persistence remains deferred;
- the central CLI remains deferred for evidence-based reasons;
- Editor and Render Lab pressure tests lower through A12 without type erasure;
- V1 requires at least one authored step and enforces at most one normalized replay-reference step, matching A12's replay lease lifecycle;
- product-contract identity/version use the canonical V1 grammar;
- exact-head repository and documentation validation pass;
- the diff remains design-only.

After accepted-main verification, proceed automatically to the single bounded A15
implementation successor.
