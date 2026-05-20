# Runenwerk

Canonical documentation lives under `docs-site/src/content/docs`.

## First Clone Bootstrap

Runenwerk uses Taskfile as the human command entrypoint. A clone cannot safely
install tools automatically, so run the explicit bootstrap once after clone:

```powershell
powershell -ExecutionPolicy Bypass -File tools/bootstrap/bootstrap.ps1
```

Then open a new shell and verify:

```powershell
task toolchain:doctor
task roadmap:validate
task roadmap:check
task puml:validate
```

If the shell cannot find `task` or `uv` after bootstrap, activate the
known tool paths in the current PowerShell session:

```powershell
. .\tools\bootstrap\activate.ps1
```

Details: `docs-site/src/content/docs/workspace/toolchain-bootstrap.md`.

Start with:

- `docs-site/src/content/docs/index.mdx`
- `docs-site/src/content/docs/workspace/overview.md`
- `docs-site/src/content/docs/workspace/toolchain-bootstrap.md`
- `docs-site/src/content/docs/guidelines/architecture.md`
- `CRATES.md`
- `DOMAIN_MAP.md`

## AI Context Export

Use this to export repository source and documentation into a single line-numbered context file for AI review:

```bash
python3 tools/context/export_repo_context.py
```

Optional output path:

```bash
python3 tools/context/export_repo_context.py --output ./Runenwerk-content.txt
```

## AI Workflow Kickoff

Use these one-line prompts when starting a new Codex thread from the repository
root:

```text
Run task batch:kickoff -- --next and follow the generated workflow.
```

```text
Run task roadmap:intake -- --idea "<design/change idea>" and prepare it for roadmap review.
```

```text
Run task ai:goal -- --track PT-SDF-OW and use the generated /goal coordinator prompt.
```

`batch:kickoff` creates the next approved-roadmap batch proposal from
`planning_state=current_candidate` rows in
`docs-site/src/content/docs/workspace/roadmap-items.yaml` and prints the exact
approval, preparation, validation, worker prompt, scope-check, and closeout
commands. It does not approve implementation unless `--approve` is explicitly
passed.

`roadmap:intake` creates a review proposal for a new design or change idea. It
does not edit the canonical roadmap until an accepted proposal is applied with
`task roadmap:apply-intake`.

Lower-level prompt and checklist helpers are still available:

```bash
task ai:list
task ai:goal -- --track "<PT-ID>"
task ai:implementation -- --task "<task>" --scope "<scope>"
task ai:closeout -- --task "<completed phase>" --roadmap "<roadmap/design>"
```

Details: `docs-site/src/content/docs/workspace/planning-and-implementation-workflow.md`.

Runenwerk does not configure remote CI. Run these validation shortcuts from the
repository root before pushing, opening a PR, or closing out a batch:

```bash
task docs:validate
task batch:validate -- --batch docs-site/src/content/docs/reports/batches/<batch-id>/batch.toml
task ci:local
```
