---
title: "Agents Instructions"
description: "Repository-level coding agent instructions."
---
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

- Prefer explicit types and clear interfaces.
- Avoid global mutable state unless already established in the architecture.
- Follow the structure and conventions used in nearby modules.

## Module Structure

Follow the module organization rules defined in `docs/guidelines/module_structure_guidelines.md`.

### Key expectations

- Organize modules by **subdomain responsibility**, not technical layers.
- Prefer **subdomain folders with `mod.rs` boundaries** for larger subsystems.
- Use explicit module names that describe responsibility.

### Example pattern

```text
render/
├── mod.rs
├── renderer/
│   ├── mod.rs
│   ├── config.rs
│   └── runtime.rs
├── frame_graph/
│   ├── mod.rs
│   ├── builder.rs
│   └── registry.rs
└── shader_manager/
    ├── mod.rs
    ├── compiler.rs
    └── types.rs
```

### Avoid the following patterns

- `include!` module composition
- `_internal` module suffixes (e.g. `renderer_internal`)
- catch-all files such as `utils.rs`, `helpers.rs`, or `misc.rs`

### When adding new code

1. Choose the owning domain (`foundation`, `engine`, `net`, `games`, `apps`).
2. Choose the owning crate.
3. Choose the owning subsystem inside that crate.
4. Add the file/module there.

## Code Discovery

Before implementing new functionality:

- Search the repository for existing implementations, helpers, or patterns.
- Prefer reuse over duplication.
- Follow patterns from nearby modules when implementing new features.
- If an existing helper solves the problem, use it instead of introducing new abstractions.

## Documentation Structure

Documentation should be organized intentionally, not scattered opportunistically.

Use the following structure when applicable:

- `README.md`
  - concise entry point for a crate or subsystem
  - should explain what it is, how to get started, and where deeper docs live
  - should not become the full manual when a docs tree exists
- `docs/index.md`
  - navigation hub for substantial crate-level documentation
  - should link clearly to user guides, advanced guides, architecture docs, roadmaps, benchmark docs, examples, and test maps where relevant
- `docs/reference/`
  - user-facing guides and reference material
  - separate by audience where useful, e.g.:
    - `usage-guide.md`
    - `advanced-guide.md`
    - `architecture.md`
- `docs/roadmaps/`
  - implementation plans, phase docs, migration plans, closeout roadmaps
- `docs/benchmarks/`
  - human-readable benchmark and profiling reports

Raw benchmark artifacts should live in dedicated artifact folders near the owning crate, not mixed into prose docs.

When creating or editing docs:

- Keep filenames in docs trees in kebab-case unless an existing local convention intentionally differs.
- Keep usage guides, advanced guides, and architecture docs distinct by audience and purpose.
- Avoid duplicating the same material across `README.md`, `usage-guide.md`, and `architecture.md` unless there is a clear reason.
- Update internal links whenever files move or are renamed.
- If a crate has multiple substantial docs, maintain a `docs/index.md` navigation page when useful.

## Documentation Ownership

Documentation should live with its owning crate or subsystem.

When deciding where docs belong:

1. Choose the owning domain.
2. Choose the owning crate.
3. Choose the owning subsystem.
4. Place the doc where users or contributors would naturally look for it.

Guidelines:

- Crate-level usage docs belong with the crate.
- Subsystem-specific docs belong with the subsystem.
- Roadmaps and migration docs belong under the owning crate's roadmap area.
- Benchmark reports belong with the owning crate's benchmark docs.
- Colocated docs beside subsystem code are acceptable when that subsystem already uses colocated documentation intentionally.
- Do not scatter markdown files across unrelated folders without a clear navigation story.

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
- Keep usage guides, advanced guides, and architecture docs distinct:
  - usage guide = normal users
  - advanced guide = advanced/extensibility topics
  - architecture doc = internals and invariants
- `README.md` should be a concise entry point, not the full manual.
- If a crate has multiple substantial docs, maintain a `docs/index.md` navigation page when useful.

Examples are part of the public API experience:

- Examples should demonstrate the preferred public usage style.
- Avoid examples that rely on internal shortcuts when public APIs are available.
- If a crate has multiple substantial examples, maintain an examples index or map when useful.
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
