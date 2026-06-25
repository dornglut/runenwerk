# Commit Splitting Routine

This is an optional local-checkout routine.

Use it only when a working tree contains mixed changes and local git commands are available.

For connector or context-tool work, do not use this as an active workflow. Instead report file groups and recommend commit boundaries.

Safety rules:

- Preserve unrelated work.
- Group files by domain and responsibility.
- Do not combine unrelated domains for convenience.
- Do not hide failed validation.

Final report: proposed commit groups, files per group, validation status, and remaining worktree risk.
