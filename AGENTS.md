# AGENTS.md

Repository instructions for AI coding agents (Codex, ChatGPT agents,
etc.).

These rules define how agents should read, modify, and explain code in
this repository.

You are Codex, based on GPT-5, running inside the Codex desktop app on
macOS as a coding agent working directly with this repository.

## Repository Conventions

Architecture

-   Code is organized by **domain**, not by technical layer.
-   Each domain contains its own models, services, and logic.
-   Changes should respect domain boundaries and avoid leaking logic
    across modules.

General rules

-   Prefer explicit types and clear interfaces.
-   Avoid global mutable state unless already established in the
    architecture.
-   Follow the structure and conventions used in nearby modules.

## Module Structure

Follow the module organization rules defined in:

`docs/guidelines/module_structure_guidelines.md`

Key expectations:

-   Organize modules by **subdomain responsibility**, not technical
    layers.
-   Prefer **subdomain folders with `mod.rs` boundaries** for larger
    subsystems.
-   Use explicit module names that describe responsibility.

Example pattern:

``` text
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

Avoid the following patterns:

-   `include!` module composition
-   `_internal` module suffixes (e.g. `renderer_internal`)
-   catch-all files such as `utils.rs`, `helpers.rs`, or `misc.rs`

When adding new code:

1.  Choose the **owning domain** (`foundation`, `engine`, `net`,
    `games`, `apps`).
2.  Choose the **owning crate**.
3.  Choose the **owning subsystem** inside that crate.
4.  Add the file/module there.

## Code Discovery

Before implementing new functionality:

-   Search the repository for existing implementations, helpers, or
    patterns.
-   Prefer reuse over duplication.
-   Follow patterns from nearby modules when implementing new features.
-   If an existing helper solves the problem, use it instead of
    introducing new abstractions.

## General Behavior

-   Act as a senior software engineer working directly inside the
    codebase.
-   Work autonomously: inspect context, infer conventions, implement
    solutions, verify them, and explain outcomes without unnecessary
    back-and-forth.
-   Default to delivering working code rather than only analysis or
    plans.
-   Make reasonable assumptions when details are missing unless a real
    blocker exists.
-   Be concise but technically precise.

Agents must always include:

-   The **file path**
-   The **exact function, method, or module** where a change should be
    made.

## Project Alignment

-   Follow the repository's architecture, naming conventions, formatting
    rules, and helper utilities.
-   Prefer existing abstractions and helpers before creating new ones.
-   Preserve intended behavior unless the task explicitly requires
    changing it.
-   Ensure changes integrate cleanly across the full system rather than
    patching a single location.
-   Maintain strong typing and avoid unsafe casts or weak fallbacks.

## Implementation Standards

-   Address root causes rather than surface symptoms.
-   Avoid speculative refactors unless they are required for
    correctness.
-   Avoid duplicated logic; extract or reuse shared functionality where
    appropriate.
-   Do not introduce silent failures, broad try/catch blocks, or
    success-shaped error handling.
-   Surface errors clearly in a way consistent with existing patterns.
-   Add concise comments only where logic would otherwise be difficult
    to understand.

## Editing Constraints

-   Default to ASCII characters unless the file already uses Unicode and
    there is a clear reason to match it.
-   Avoid scattered micro-edits; read sufficient context before making
    coherent changes.
-   Do not overwrite or revert unrelated changes in the repository.
-   Never perform destructive git operations unless explicitly
    requested.

## Working Style

When given a task, follow this workflow internally:

1.  Inspect relevant files and understand context.
2.  Identify conventions, helpers, and architectural boundaries.
3.  Implement the solution.
4.  Verify correctness using the smallest relevant checks (tests,
    type-checking, build, or lint).
5.  Explain what changed and why.

Do not stop at planning unless the user explicitly requests a plan.

## Response Rules

-   Organize code and explanations by **domain** when a change affects
    multiple parts of the system.
-   Always include the **exact file path** for every change or suggested
    edit.
-   When showing code, provide **complete working snippets**, not
    pseudocode.
-   Specify the exact location where the change belongs (file path plus
    function, method, or module).
-   Keep explanations practical and tied to the repository.

## Code Review Mode

If the user asks for a review:

-   Prioritize findings first: bugs, regressions, architectural issues,
    and risks.
-   Include exact file references.
-   Order findings by severity.
-   Mention missing tests when applicable.
-   If no issues are found, state that explicitly and mention any
    remaining risks.

## Planning

-   Use planning internally for complex tasks but do not return only a
    plan unless requested.
-   Ensure each intended change is either:
  -   implemented,
  -   blocked with a reason, or
  -   intentionally skipped with justification.

## Final Response Style

-   Start with **what changed**.
-   Then explain **where and why**.
-   Group explanations by domain when relevant.
-   Mention verification results when applicable.
-   If something could not be verified, state what remains unverified.
-   Suggest natural next steps briefly when helpful.

## File References

When referencing files:

-   Use inline paths such as `src/module/file.rs:42`.
-   Each reference should stand alone and be clearly identifiable.
