# Phase 6 Roadmap - Archetype Storage Redesign

## Status
- Phase 5B is complete.
- Phase 6 is approved with the following direction:
  - Proceed with archetype-backed storage redesign for dominant broad-query forms.
- This roadmap defines the execution plan, sequencing, validation gates, risks, and exit criteria for Phase 6.

## 1. Executive Summary
Phase 5B completed the high-value query-layer optimization work:
- dominant-form fast paths
- typed store lookup caching
- reduced hot-loop indirection
- refreshed benchmark and profile artifact generation
- final recommendation to move below the query layer

Those changes materially improved historical baseline performance, especially for dominant broad mutable paths and representative mixed-frame engine workloads. However, refreshed same-session measurements show that further gains from query-layer specialization alone are now mixed, smaller, and less stable at higher scales.

The dominant remaining cost is no longer primarily query dispatch structure. It is now the cost of traversing the current storage model under broad iteration pressure.

Phase 6 therefore focuses on replacing the current broad traversal/storage cost model with an archetype-oriented dense iteration model.

## 2. Phase 6 Objective
### Primary objective
Reduce the cost of dominant broad query execution by moving from current storage traversal toward:
- archetype-based matching
- dense column iteration
- improved cache locality
- lower cross-store fetch overhead
- reduced per-entity traversal overhead

### Secondary objective
Preserve all existing ECS semantics while making storage/layout the new optimization surface.

This includes preserving correctness for:
- `Changed<T>`
- `Added<T>`
- remove/reinsert flows
- command flush timing
- scheduler conflict behavior
- query result correctness
- mutation visibility rules

## 3. Why Phase 6 Is Necessary
Phase 5B established three important conclusions.

### 3.1 Query-layer optimization is no longer the main frontier
The highest-value fetch-path improvements are already implemented for dominant forms:
- `&mut T`
- `(&mut A, &B)`
- `(&mut A, &mut B)`

Additional tuning in the same area is now likely to produce:
- smaller wins
- less stable wins
- more complexity per unit of gain

### 3.2 Broad mutable iteration remains the primary high-scale pressure
The remaining top-end pressure shows up in:
- broad mutable loops
- broad mixed tuple loops
- storage traversal cost
- cache behavior
- data access locality

### 3.3 Scheduler and flush remain secondary
Phase 5B profiling and mixed-frame results continue to indicate that:
- scheduler planning is not the principal limiter
- flush cost is not the principal limiter
- query/storage traversal still dominates representative workloads

This means Phase 6 should target storage/layout first, not scheduler throughput.

## 4. Phase 6 Scope
### In scope
- archetype identity and registry
- entity-to-location tracking
- dense component column storage
- structural migration between archetypes
- archetype-based query matching for dominant forms
- change/add tracking compatibility
- benchmark and profiling validation
- compatibility with current public ECS semantics

### Out of scope for initial Phase 6
- full ECS rewrite in one pass
- immediate removal of all legacy storage paths
- scheduler-first optimization work
- aggressive parallel execution redesign
- speculative storage compaction features not justified by profiling
- highly advanced chunk packing before a simpler archetype path is validated

## 5. Architecture Goals
### 5.1 Storage goals
The storage layer should evolve toward:
- archetypes keyed by component-set identity
- dense component columns within each archetype
- efficient row-based iteration
- minimal pointer chasing inside hot loops
- predictable memory access under broad scans

### 5.2 Query goals
The query layer should evolve toward:
- matching archetypes once per prepared query state
- binding typed column views once per archetype
- scanning rows directly from dense storage
- minimizing per-entity branching and generic fetch overhead

### 5.3 Semantic goals
The redesign must preserve:
- correctness of `Changed<T>` and `Added<T>`
- structural mutation semantics across flush boundaries
- query correctness under reused `QueryState`
- expected behavior after remove/reinsert flows
- scheduler conflict correctness

### 5.4 Delivery goals
Phase 6 must remain incremental and benchmark-driven.

That means:
- introduce the new path in bounded slices
- validate each slice with tests and artifacts
- avoid "big bang" migration
- remove complexity only after measured replacement success

## 6. Dominant Workloads to Optimize First
Phase 6 should target the workloads that Phase 5B proved matter most.

### Core query forms
- `Query<&mut T>`
- `Query<(&mut A, &B)>`
- `Query<(&mut A, &mut B)>`

### Primary benchmark gates
- W1 - broad transform-style update
- C3 - broad simple write query
- C4 - broad double-mutable query
- W2 - representative gameplay mixed workload
- engine mixed frame - headless runtime mixed-frame scenario

### Secondary validation gates
- C2 - broad read query parity
- W3 - structural churn cost control
- W4 - event-heavy workload regression watch
- W5 - scheduler stress regression watch

## 7. Deliverables
### 7.1 Design document
Create:
- `foundation/ecs/docs/PHASE6A_ARCHETYPE_STORAGE_PLAN.md`

This document must define:
- archetype identity model
- entity location model
- storage layout
- migration model
- change/add tracking behavior
- query matching plan
- compatibility strategy
- risks and invariants

### 7.2 Implementation slices
Create the new storage path in incremental slices, with each slice landing in reviewable units.

### 7.3 Benchmark artifact set
Create a Phase 6 artifact folder similar to Phase 5B:
- `foundation/ecs/benchmarks/phase6/`

Expected contents:
- `README.md`
- `PHASE6_PROGRESS_REPORT.md`
- `phase6_baseline_refresh_ecs_bench.txt`
- `phase6_baseline_refresh_profile.txt`
- `phase6_baseline_refresh_engine_bench.txt`
- `phase6_optimized_ecs_bench.txt`
- `phase6_optimized_profile.txt`
- `phase6_optimized_engine_bench.txt`

### 7.4 Regression tests
Add dedicated storage/query regression coverage for archetype migration and query execution.

Suggested new test files:
- `foundation/ecs/tests/storage_phase6.rs`
- `foundation/ecs/tests/query_phase6.rs`

## 8. Phase 6 Execution Plan
### Phase 6A - Design and invariants
#### Goal
Define the storage redesign precisely before implementation starts.

#### Files to create or update
- `foundation/ecs/docs/PHASE6A_ARCHETYPE_STORAGE_PLAN.md`

Likely design-adjacent notes in:
- `foundation/ecs/README.md`
- `foundation/ecs/USAGE_GUIDE.md`

#### Required decisions
##### Archetype identity
Define how an archetype is keyed.

Recommended direction:
- canonical component-set key
- deterministic ordering
- stable hashing/equality behavior
- cheap comparison for cache lookup

##### Entity location
Define exact entity location tracking.

Recommended model:
- `Entity -> { archetype_id, row }`

This becomes mandatory for:
- direct row access
- migration correctness
- despawn cleanup
- stable query execution assumptions

##### Column storage
Define storage per archetype as dense columns.

Recommended first pass:
- one dense vector per component type present in the archetype
- one dense entity vector
- aligned row index across all columns

##### Change tracking
Define row-level tracking behavior for:
- added generation/tick
- changed generation/tick

This must be specified before migration code is written.

##### Structural mutation
Define how insert/remove moves entities between archetypes and how command flush applies those changes.

Recommended rule:
- preserve stage-boundary flush semantics
- do not apply structural changes directly during active query iteration

#### Exit criteria
- design doc complete
- invariants explicit
- migration semantics defined
- change/add semantics explicitly mapped
- open risks documented

### Phase 6B - Archetype skeleton and entity location system
#### Goal
Build the minimum archetype storage foundation without changing all query execution yet.

#### Files likely to touch
- `foundation/ecs/src/world/...`
- `foundation/ecs/src/storage/...`
- `foundation/ecs/src/entity/...`

Exact file placement depends on the current repository layout, but this phase should introduce or isolate:
- archetype definitions
- archetype registry
- entity location tracking
- column storage primitives

#### Required implementation
##### Archetype registry
Introduce a registry that can:
- find or create archetypes by component-set key
- allocate archetype ids
- expose metadata needed for query planning

##### Entity location map
Introduce a location map that tracks the current row of every live entity.

This must support:
- spawn placement
- migration updates
- despawn cleanup
- consistency assertions in tests

##### Dense column primitives
Introduce the core internal representation for a component column.

Requirements:
- dense row-addressable storage
- row swap/remove support
- typed access compatible with future query binding
- associated change/add tracking metadata

#### Exit criteria
- entities can be placed into archetypes
- entity locations remain correct after spawn/despawn
- row alignment invariants hold
- foundational tests pass

### Phase 6C - Structural migration path
#### Goal
Make archetype transitions correct for structural mutations.

#### Files likely to touch
- `foundation/ecs/src/world/...`
- `foundation/ecs/src/commands/...`
- `foundation/ecs/src/storage/...`

#### Required implementation
##### Spawn path
Spawn must place new entities into the correct archetype with valid initial tracking metadata.

##### Insert component path
Inserting a component must:
- compute destination archetype
- move entity data
- append the new component data
- update location mapping
- preserve required semantics for existing component tracking

##### Remove component path
Removing a component must:
- compute destination archetype
- move surviving component data
- drop the removed component correctly
- update location mapping
- preserve intended changed/added semantics

##### Despawn path
Despawn must:
- remove the row cleanly
- update swapped entity location if swap-remove is used
- clear entity location mapping
- release archetype row resources correctly

##### Flush integration
Commands must continue to apply structural changes at the correct lifecycle boundary.

#### Tests to add
Suggested file:
- `foundation/ecs/tests/storage_phase6.rs`

Suggested test cases:
- entity location updates after insert migration
- entity location updates after remove migration
- despawn updates swapped row locations correctly
- remove then reinsert preserves intended tracking semantics
- flush applies structural migration at the same boundary as before

#### Exit criteria
- structural mutation correctness established
- command flush timing preserved
- migration tests pass
- no regression in existing structural semantics

### Phase 6D - Archetype query matching for dominant forms
#### Goal
Move dominant broad query forms onto archetype-based execution.

#### Files likely to touch
- `foundation/ecs/src/query/traits_and_state.rs`
- `foundation/ecs/src/query/query_data_impls.rs`
- new or updated storage/query bridge files under:
  - `foundation/ecs/src/query/...`
  - `foundation/ecs/src/storage/...`

#### Required implementation
##### Query preparation
`QueryState` should prepare and cache:
- matching archetype ids
- per-archetype column binding data
- filter-relevant metadata
- mutability/access metadata

##### Dominant forms to land first
- `Query<&mut T>`
- `Query<(&mut A, &B)>`
- `Query<(&mut A, &mut B)>`

##### Query execution model
Execution should become:
- select prepared matching archetype
- bind typed column views once
- iterate dense rows directly
- apply filters
- fetch tuple directly from row-aligned columns
- mark changed metadata for mutable outputs

##### Important rule
Do not attempt to migrate every query form at once. Dominant forms only.

#### Tests to add
Suggested file:
- `foundation/ecs/tests/query_phase6.rs`

Suggested test cases:
- `Query<&mut T>` matches previous query semantics
- `Query<(&mut A, &B)>` fetch order and correctness are unchanged
- `Query<(&mut A, &mut B)>` marks changed correctly for both mutable outputs
- prepared query state invalidates or refreshes correctly after structural changes
- reused query state works across worlds without stale archetype bindings

#### Exit criteria
- dominant broad forms run on archetype storage
- correctness matches prior semantics
- no stale cache binding behavior
- existing query regression coverage still passes

### Phase 6E - Filter and tracking integration
#### Goal
Bring filter support and tracking semantics to the new execution path.

#### Files likely to touch
- `foundation/ecs/src/query/...`
- `foundation/ecs/src/world/...`
- `foundation/ecs/src/storage/...`

#### Required support
##### Filters
Add support for:
- `With<T>`
- `Without<T>`
- `Changed<T>`
- `Added<T>`

Prioritize the filter combinations used by W2 and current regression coverage.

##### Tracking semantics
Ensure the new storage path preserves:
- correct first-add behavior
- correct change visibility
- remove-then-reinsert behavior
- flush boundary expectations

##### Cache invalidation rules
Structural changes that affect archetype matching must invalidate or refresh relevant prepared state correctly.

#### Tests to add
Suggested cases:
- `Changed<T>` works after mutable iteration on archetype path
- `Added<T>` works after spawn and insert on archetype path
- remove then reinsert behaves identically to prior semantics
- `With/Without` matching is correct across migrated entities
- prepared state updates correctly when a matching archetype appears later

#### Exit criteria
- W2-class semantics pass
- tracking remains correct
- filter correctness is established
- no mismatch between old and new paths on covered scenarios

### Phase 6F - Benchmarking, profiling, and decision gate
#### Goal
Measure whether storage redesign materially moved the ceiling.

#### Runners to execute
At minimum:

```powershell
cargo test -p ecs
cargo bench -p ecs --bench phase5b --features telemetry -- --quick
cargo run -p ecs --example phase5b_profile --features telemetry --release
cargo bench -p engine --bench phase5b_runtime -- --quick
```

Then add dedicated Phase 6 runners.

Suggested additions:
- `foundation/ecs/benches/phase6.rs`
- `foundation/ecs/examples/phase6_profile.rs`
- `engine/benches/phase6_runtime.rs`

#### Required measurement dimensions
For each decision checkpoint, capture:
- ECS quick bench medians
- engine mixed-frame medians
- query matching attribution
- query iteration attribution
- changed/added attribution where relevant
- any structural churn regression signal

#### Decision rule
Phase 6 is successful only if the new storage path improves the dominant broad query workloads materially enough to justify the added complexity.

#### Exit criteria
- large-scale broad mutable workloads improve materially
- engine mixed frame improves or stays neutral with better headroom
- query-path dominance declines in profiles
- no unacceptable W3/W4/W5 regressions
- correctness remains intact

## 9. Benchmark Success Criteria
### Strong success signals
The redesign is working if these become true:
- W1 improves clearly at 50k and 200k
- C3 improves clearly at 50k and 200k
- C4 scales better than the current path
- W2 shows better broad-scan behavior
- engine mixed frame improves or remains neutral with lower query pressure

### Weak success signals
These are not enough on their own:
- only 10k-class improvement
- only microbench gains with no engine/runtime benefit
- only profile attribution reduction with no wall-time gain
- gains that disappear under same-session refresh

### Failure signals
Phase 6 should be reconsidered if these dominate:
- structural churn regresses heavily
- changed/added semantics become fragile
- engine mixed frame regresses materially
- query-path time falls but total wall time does not
- complexity increases without reliable high-scale improvement

## 10. Risk Register
### Risk 1 - semantic drift
#### Description
Archetype migration can subtly break:
- `Changed<T>`
- `Added<T>`
- remove/reinsert behavior
- command flush semantics
- reused query state semantics

#### Mitigation
- define tracking semantics before implementation
- add migration-specific tests before tuning
- keep old-vs-new semantic parity checks for dominant forms

### Risk 2 - structural mutation regression
#### Description
Archetype systems often make steady-state iteration faster while making insert/remove/despawn more expensive.

#### Mitigation
- benchmark W3 continuously
- avoid tuning only for stable-scan workloads
- keep flush semantics stable to isolate cost changes

### Risk 3 - cache invalidation bugs
#### Description
Prepared query state may become stale after:
- structural mutation
- new archetype creation
- world changes
- removed matching archetypes

#### Mitigation
- formalize invalidation/versioning strategy
- test "matching archetype appears later" cases
- test cross-world and reused-state rebinding explicitly

### Risk 4 - oversized first implementation
#### Description
Trying to migrate all query types and storage rules at once will slow delivery and obscure regressions.

#### Mitigation
- move dominant forms first
- preserve fallback paths temporarily
- benchmark each landed slice before widening scope

### Risk 5 - premature chunk sophistication
#### Description
Adding advanced chunk management too early can add engineering cost before the basic archetype model is proven.

#### Mitigation
- start with dense archetypes first
- add chunk segmentation only if profiling shows need
- keep first implementation legible and measurable

## 11. Test Plan
### Existing tests that must remain green
Continue running:

```powershell
cargo test -p ecs
```

Especially preserve coverage around:
- command flush timing
- scheduler conflict behavior
- changed/added semantics
- query correctness
- reused query state behavior

### New test files
Create:
- `foundation/ecs/tests/storage_phase6.rs`
- `foundation/ecs/tests/query_phase6.rs`

### Required new test cases
#### Storage migration tests
In `foundation/ecs/tests/storage_phase6.rs`:
- entity location updates after insert migration
- entity location updates after remove migration
- despawn clears row and updates swap target location
- archetype transition preserves surviving component values
- command flush applies structural mutation only at the intended boundary

#### Tracking tests
In `foundation/ecs/tests/storage_phase6.rs` or `foundation/ecs/tests/query_phase6.rs`:
- `Added<T>` after spawn on archetype path
- `Added<T>` after insert on archetype path
- `Changed<T>` after mutable query/write on archetype path
- remove then reinsert preserves intended semantics

#### Query execution tests
In `foundation/ecs/tests/query_phase6.rs`:
- `Query<&mut T>` parity with prior path
- `Query<(&mut A, &B)>` parity with prior path
- `Query<(&mut A, &mut B)>` parity with prior path
- `With<T>` and `Without<T>` matching across migrated entities
- prepared query state refresh after structural changes
- reused query state across world changes remains safe

## 12. Recommended Internal Milestones
### Milestone M1 - design locked
Outputs:
- architecture design doc complete
- invariants signed off
- implementation slice plan fixed

### Milestone M2 - storage skeleton landed
Outputs:
- archetype registry
- entity location map
- dense column primitives
- spawn/despawn foundation

### Milestone M3 - structural migration landed
Outputs:
- insert/remove/despawn migration
- flush integration
- migration regression tests green

### Milestone M4 - dominant queries on archetypes
Outputs:
- `&mut T`
- `(&mut A, &B)`
- `(&mut A, &mut B)` on new path
- query parity tests green

### Milestone M5 - filters and tracking landed
Outputs:
- `With`
- `Without`
- `Changed`
- `Added`
- W2 semantics green

### Milestone M6 - benchmark decision checkpoint
Outputs:
- Phase 6 artifacts generated
- profile attribution compared
- engine mixed frame compared
- continuation decision documented

## 13. Documentation Updates Required
Update the following as Phase 6 lands:
- `foundation/ecs/README.md`
- `foundation/ecs/USAGE_GUIDE.md`
- `foundation/ecs/benchmarks/phase6/README.md`

The documentation should explicitly state:
- which query forms are on the archetype path
- which paths still use legacy storage
- any current limitations
- how to run Phase 6 benchmark and profile suites
- how semantic parity is validated

## 14. Completion Criteria for Phase 6
Phase 6 should only be considered complete when all of the following are true:
- archetype-backed storage exists for dominant broad-query forms
- structural migration is correct
- entity location tracking is correct
- `Changed<T>` and `Added<T>` semantics remain correct
- ECS regression tests pass
- dominant large-scale workloads improve materially
- engine mixed frame is improved or not materially regressed
- profiles show reduced query/storage traversal dominance
- implementation complexity remains maintainable for Phase 7 work

## 15. Decision Policy After Phase 6
At the end of Phase 6, choose the next phase based on measured outcomes.

### If Phase 6 materially improves dominant broad iteration
Proceed with:
- expanding archetype coverage
- broader filter/query support
- storage compaction or chunk refinement if profiling supports it

### If Phase 6 improves microbenchmarks but not engine/runtime behavior
Investigate:
- remaining scheduler/runtime costs
- command application overhead
- system mix effects
- false-positive wins limited to synthetic scans

### If Phase 6 causes unacceptable structural regressions
Rebalance by:
- optimizing migration paths
- reducing archetype churn costs
- narrowing the archetype path to dominant stable workloads only

## 16. Final Direction
Phase 6 should be executed as a targeted storage/layout redesign, not as a full unbounded ECS rewrite.

The correct implementation strategy is:
- preserve semantics first
- land the new model in narrow slices
- move dominant broad-query forms first
- benchmark each slice
- only widen scope after measured success

That is the highest-confidence path to turning the Phase 5B conclusion into durable end-to-end performance gains.

## 17. Suggested Roadmap Label
Use:
- Phase 6A - Archetype-backed dense storage for dominant broad-query forms
