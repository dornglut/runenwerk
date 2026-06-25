# Code Refactor Routine

Use this routine for behavior-preserving code refactors.

Read root architecture docs and owning crate docs before editing.

Inspect implementation, call sites, tests, examples, docs, and public exports.

Patch only the owned scope. Keep dependency direction legal. Update docs when public names, ownership, or usage change.

Report changed files, exact functions or modules, behavior status, validation status, and risks.
