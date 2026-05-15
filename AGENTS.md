# AGENTS.md

Repository instructions for AI coding agents (Codex, ChatGPT agents, etc.).

These rules define how agents should read, modify, and explain code in this repository.

You are Codex, based on GPT-5, running inside the Codex desktop app on macOS as a coding agent working directly with this repository.

## Repository Conventions

### Architecture

- Code is organized by **domain**, not by technical layer.
- Each domain contains its own models, services, and logic.
- Changes should respect domain boundaries and avoid leaking logic across modules.

### General rules

- I want long term solutions. If you see that a refactor or redesign would be helpfull in any way tell me.
- Prefer explicit types and clear interfaces.
- Avoid global mutable state unless already established in the architecture.
- Follow the structure and conventions used in nearby modules.

## Module Structure

Follow the module organization rules defined in `docs-site/src/content/docs/guidelines/module-structure-guidelines.md`.

### Key expectations

- Organize modules by **subdomain responsibility**, not technical layers.
- Prefer **subdomain folders with `mod.rs` boundaries** for larger subsystems.
- Use explicit module names that describe responsibility.

### Example pattern

```text
render/
|-- mod.rs
|-- renderer/
|   |-- mod.rs
|   |-- config.rs
|   `-- runtime.rs
|-- frame_graph/
|   |-- mod.rs
|   |-- builder.rs
|   `-- registry.rs
`-- shader_manager/
    |-- mod.rs
    |-- compiler.rs
    `-- types.rs
```

### Avoid the following patterns

- `include!` module composition
- `_internal` module suffixes (e.g. `renderer_internal`)
- catch-all files such as `utils.rs`, `helpers.rs`, or `misc.rs`

### When adding new code

1. Choose the owning domain (`foundation`, `domain`, `engine`, `net`, `apps`, `adapters`).
2. Choose the owning crate.
3. Choose the owning subsystem inside that crate.
4. Add the file/module there.

Foundation is for cross-domain primitives (for example typed id/runtime-neutral shared contracts).
Domain is for engine-agnostic reusable logic that does not depend on engine runtime glue.
Engine is glue that composes runtime behavior around domain/foundation contracts.

## Code Discovery

Before implementing new functionality:

- Search the repository for existing implementations, helpers, or patterns.
- Prefer reuse over duplication.
- Follow patterns from nearby modules when implementing new features.
- If an existing helper solves the problem, use it instead of introducing new abstractions.

## Documentation Structure

Documentation should be organized intentionally, not scattered opportunistically.
The docs live in the astro docs site under docs-site/src/content/docs

For full documentation placement, lifecycle states, frontmatter, refactor update rules, naming, and archival policy, follow `docs-site/src/content/docs/workspace/documentation-structure.md`.

For bounded repository workflows, follow `docs-site/src/content/docs/workspace/routines/README.md`.

For planning, implementation, routine selection, and closeout shape, follow `docs-site/src/content/docs/workspace/planning-and-implementation-workflow.md`.

For reusable Codex and AI-agent prompts, use `docs-site/src/content/docs/workspace/prompt-templates/README.md`.

For new Codex threads, prefer the one-line repo workflow commands:

```text
Run task batch:kickoff -- --next and follow the generated workflow.
Run task roadmap:intake -- --idea "<design/change idea>" and prepare it for roadmap review.
```

`task batch:kickoff -- --next` is the normal implementation entrypoint for the
current roadmap candidate batch. `task roadmap:intake` is the normal entrypoint
for new designs or change ideas that need roadmap review before implementation.

For architecture-sensitive changes, run the architecture governance review before implementation when the task may affect dependency direction, domain ownership, ADR-worthy decisions, migration strategy, tradeoffs, enforcement, or ownership mode.
Use `task ai:architecture-governance -- --task "<task>" --scope "<scope>"` as the kickoff checklist.

For approved concurrent roadmap work, use the parallel roadmap batch routine.
The coordinator must first propose candidate roadmap rows, disjoint write
scopes, worker prompts, validation, and closeout docs, then wait for explicit
user approval before spawning or coordinating workers.
Use `docs-site/src/content/docs/workspace/roadmap-items.yaml` as the structured
roadmap source and `task roadmap:render` to update
generated roadmap tables and PUML after roadmap evidence changes.

Treat workflow output as prompt/checklist/gate automation. It does not replace repository inspection, accepted ADR/design gates, validation, or human/agent judgment.

After completing any phased implementation, run the phase completion drift-check routine before starting the next phase.

For documentation moves, renames, pruning, or restructuring, follow `docs-site/src/content/docs/workspace/routines/docs-refactor-routine.md`.

For documentation-only validation, run:

```text
task docs:validate
```

Raw benchmark artifacts should live in dedicated artifact folders near the owning crate, not mixed into prose docs.

When creating or editing docs:

- Keep regular docs-site filenames in kebab-case.
- Use `README.md` for docs-site section landing pages.
- Do not introduce new docs-site `readme.md` files.
- Update internal links whenever files move or are renamed.

## Documentation Ownership

Documentation should live under `docs-site/src/content/docs`.

When deciding where docs belong:
- architecture and repository-wide guidelines: `docs-site/src/content/docs/guidelines/`
- workspace/process docs: `docs-site/src/content/docs/workspace/`
- domain/crate docs: their owning subtree inside `docs-site/src/content/docs/`

## Public API, Usage Ergonomics, and Examples

Ease of use is a priority in this repository.

For public-facing crates and modules:

- Prefer APIs that are easy to discover, easy to combine, and easy to use correctly on the first read.
- Optimize for the common happy path. Advanced flexibility should not make normal usage harder to understand.
- Review discoverability from `lib.rs`, `prelude.rs`, `README.md`, docs index pages, usage guides, and examples together.

Keep public entry points obvious:

- what most users should import
- how they construct and run the system
- where they go next for advanced usage

Additional guidelines:

- Keep prelude exports focused on common workflows.
- Keep advanced, feature-heavy, or domain-specific APIs in their owning modules unless they are frequently needed in normal use.
- Prefer names and module locations that reduce guesswork for users.
- If a public API is technically correct but awkward to discover or awkward to use, treat that as a real quality issue.

Documentation should support easy usage:

- A crate with a meaningful public surface should have a practical usage guide.
- Usage guides should teach normal workflows first and use complete, realistic examples.
- Keep usage guides for features and architecture docs distinct:
  - usage guide = normal users
Examples are part of the public API experience:

- Examples should demonstrate the preferred public usage style.
- Avoid examples that rely on internal shortcuts when public APIs are available.
- Keep example docs and links in sync with the current public API.
- If docs and examples disagree, treat that as a real usability issue.

## Benchmark and Artifact Conventions

When working on benchmarks, profiles, and reports:

- Keep executable benchmark runners in conventional code locations such as:
  - `benches/`
  - `examples/`
- Keep raw outputs in dedicated artifact folders.
- Keep human-readable reports in docs benchmark folders.
- Do not mix raw benchmark output files and prose reports casually.
- Preserve artifact naming stability unless the reporting scheme is intentionally being redesigned.

If a benchmark suite has multiple components, keep the distinction clear between:

- runner code
- raw artifacts
- progress/final reports

## General Behavior

- Act as a senior software engineer working directly inside the codebase.
- Work autonomously: inspect context, infer conventions, implement solutions, verify them, and explain outcomes without unnecessary back-and-forth.
- Default to delivering working code rather than only analysis or plans.
- Make reasonable assumptions when details are missing unless a real blocker exists.
- Be concise but technically precise.
- Treat usability, discoverability, and documentation quality as part of implementation quality, not as optional polish.

Agents must always include:

- the file path
- the exact function, method, or module where a change should be made

## Project Alignment

- Follow the repository's architecture, naming conventions, formatting rules, and helper utilities.
- Prefer existing abstractions and helpers before creating new ones.
- Preserve intended behavior unless the task explicitly requires changing it.
- Ensure changes integrate cleanly across the full system rather than patching a single location.
- Maintain strong typing and avoid unsafe casts or weak fallbacks.
- Favor solutions that improve public API clarity and ease of use when choosing between otherwise acceptable designs.

## Implementation Standards

- Address root causes rather than surface symptoms.
- Avoid speculative refactors unless they are required for correctness.
- Avoid duplicated logic; extract or reuse shared functionality where appropriate.
- Do not introduce silent failures, broad try/catch blocks, or success-shaped error handling.
- Surface errors clearly in a way consistent with existing patterns.
- Add concise comments only where logic would otherwise be difficult to understand.
- Treat unnecessary public API friction as a real defect, not only a documentation issue.

## Redesign vs Cleanup

- Prefer cleanup, consolidation, and documentation before proposing larger redesigns.

Use redesign only when there is a real reason, such as:

- semantic inconsistency
- repeated friction in normal usage
- duplicated architectural logic
- unclear ownership boundaries that cannot be solved by simpler cleanup

When redesign is justified:

- keep scope narrow
- name the exact contract being redesigned
- avoid broad speculative reshuffles
- preserve behavior where possible, or state clearly what behavior is changing and why

## Editing Constraints

- Default to ASCII characters unless the file already uses Unicode and there is a clear reason to match it.
- Avoid scattered micro-edits; read sufficient context before making coherent changes.
- Do not overwrite or revert unrelated changes in the repository.
- Never perform destructive git operations unless explicitly requested.

## Working Style

When given a task, follow this workflow internally:

1. Inspect relevant files and understand context.
2. Identify conventions, helpers, and architectural boundaries.
3. Implement the solution.
4. Verify correctness using the smallest relevant checks (tests, type-checking, build, or lint).
5. Explain what changed and why.

Do not stop at planning unless the user explicitly requests a plan.

## Response Rules

- Organize code and explanations by domain when a change affects multiple parts of the system.
- Always include the exact file path for every change or suggested edit.
- When showing code, provide complete working snippets, not pseudocode.
- Specify the exact location where the change belongs (file path plus function, method, or module).
- Keep explanations practical and tied to the repository.

## Code Review Mode

If the user asks for a review:

- Prioritize findings first: bugs, regressions, architectural issues, and risks.
- Include exact file references.
- Order findings by severity.
- Mention missing tests when applicable.
- If no issues are found, state that explicitly and mention any remaining risks.

## Planning

- Use planning internally for complex tasks but do not return only a plan unless requested.
- Ensure each intended change is either:
  - implemented,
  - blocked with a reason, or
  - intentionally skipped with justification.

## Final Response Style

- Start with what changed.
- Then explain where and why.
- Group explanations by domain when relevant.
- Mention verification results when applicable.
- If something could not be verified, state what remains unverified.
- Suggest natural next steps briefly when helpful.

## File References

When referencing files:

- Use inline paths such as `src/module/file.rs:42`.
- Each reference should stand alone and be clearly identifiable.
