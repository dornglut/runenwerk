---
title: Docs Site Vercel Hosting Plan
description: Workspace handoff plan for configuring the existing Astro docs site on Vercel.
status: active
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-12
---

# Runenwerk Docs Site Vercel Hosting Plan

## Purpose

Prepare the existing Runenwerk Astro docs site for Vercel hosting without changing the repository's documentation ownership model.

This is a handoff document for Codex. It should be used as the implementation prompt/checklist when setting up deployment.

## Repository context

The canonical documentation tree is:

```text
docs-site/src/content/docs
```

Do not create a second documentation system. Do not move canonical docs out of `docs-site/src/content/docs`.

The docs site should remain owned by the existing `docs-site` Astro project.

## Target hosting decision

Use Vercel as the preferred hosting provider for the documentation site.

Reason:

- Vercel supports Git-backed automatic deployments.
- Astro projects are supported by Vercel and can be auto-detected.
- A nested project root can be configured as `docs-site`.
- Preview deployments are useful for documentation changes before merging to `main`.

## Desired Vercel project settings

Configure the Vercel project with:

```text
Root Directory: docs-site
Framework Preset: Astro
Build Command: npm run build
Output Directory: dist
Install Command: npm install
Production Branch: main
```

If the repository uses a different package manager inside `docs-site`, do not blindly use `npm install`.
First inspect the docs-site lockfiles:

```text
docs-site/package-lock.json -> npm
docs-site/pnpm-lock.yaml    -> pnpm
docs-site/yarn.lock         -> yarn
docs-site/bun.lockb         -> bun
```

Then align the install/build commands to the actual package manager.

## Automatic deployment behavior

Expected deployment behavior after the GitHub repository is connected to Vercel:

```text
Push or PR branch update -> Preview Deployment
Push or merge to main    -> Production Deployment
```

The production branch should be `main` unless the repository uses another default production branch.

## Implementation approach

Prefer the smallest durable setup:

1. Verify the existing docs-site project.
2. Build it locally.
3. Configure Vercel with `docs-site` as the project root.
4. Only add repository config if needed for reproducibility or if Vercel UI inference is insufficient.

## Codex task

### Step 1: Inspect docs-site

From the repository root, inspect:

```text
docs-site/package.json
docs-site/astro.config.*
docs-site/src/content/docs/
```

Confirm:

- `docs-site/package.json` exists.
- It has a production build script.
- The Astro config does not contain GitHub Pages-only `base` settings unless intentionally required elsewhere.
- The docs content lives under `docs-site/src/content/docs`.

### Step 2: Detect package manager

Check for lockfiles in `docs-site`.

Use the package manager indicated by the lockfile.
Do not switch package managers as part of this task.

### Step 3: Verify local build

Run from the repository root:

```bash
cd docs-site
npm install
npm run build
```

If the project uses `pnpm`, use:

```bash
cd docs-site
pnpm install
pnpm build
```

If the project uses `yarn`, use:

```bash
cd docs-site
yarn install
yarn build
```

If the project uses `bun`, use:

```bash
cd docs-site
bun install
bun run build
```

Record the actual command used in the closeout.

### Step 4: Decide whether to add `vercel.json`

Do not add `vercel.json` if Vercel project settings are enough and the team is comfortable storing deployment config in Vercel.

Add `vercel.json` only if the repository should version deployment behavior.

If adding root-level Vercel config, use this file:

```json
{
  "buildCommand": "cd docs-site && npm run build",
  "outputDirectory": "docs-site/dist",
  "installCommand": "cd docs-site && npm install"
}
```

File location:

```text
vercel.json
```

Module/location:

```text
Repository root deployment configuration
```

If using `pnpm`, use:

```json
{
  "buildCommand": "cd docs-site && pnpm build",
  "outputDirectory": "docs-site/dist",
  "installCommand": "cd docs-site && pnpm install"
}
```

If using `yarn`, use:

```json
{
  "buildCommand": "cd docs-site && yarn build",
  "outputDirectory": "docs-site/dist",
  "installCommand": "cd docs-site && yarn install"
}
```

If using `bun`, use:

```json
{
  "buildCommand": "cd docs-site && bun run build",
  "outputDirectory": "docs-site/dist",
  "installCommand": "cd docs-site && bun install"
}
```

Do not add more Vercel configuration unless there is a concrete need.

### Step 5: Optional documentation update

If the repository should document deployment steps, add a short docs page or workspace note.

Preferred location:

```text
docs-site/src/content/docs/workspace/docs-site-deployment.md
```

Module/location:

```text
workspace documentation
```

Suggested content scope:

- Vercel project settings
- local validation command
- expected automatic deployment behavior
- ownership reminder that docs live under `docs-site/src/content/docs`

Do not over-document Vercel internals.

### Step 6: Optional README pointer

If useful, update the root README with a short pointer.

File location:

```text
README.md
```

Module/location:

```text
Repository entry documentation
```

Suggested addition:

```md
## Docs Site Deployment

The documentation site lives in `docs-site/` and can be deployed to Vercel with `docs-site` as the project root.

Local validation:

```bash
cd docs-site
npm run build
```
```

Only add this if the deployment process should be visible from the repository root.

## Acceptance criteria

The task is done when:

- The existing `docs-site` Astro project is confirmed.
- The local docs build succeeds.
- Vercel project settings are identified precisely.
- Automatic deployment behavior is documented:
  - branch/PR updates create preview deployments
  - `main` creates production deployments
- No duplicate docs system is introduced.
- No canonical docs are moved out of `docs-site/src/content/docs`.
- Any added config uses the same package manager already used by `docs-site`.
- Any repository changes include exact file paths in the closeout.

## Validation commands

Run the relevant docs build command:

```bash
cd docs-site
npm run build
```

Run the repository docs validation through the canonical Taskfile entrypoint:

```bash
task docs:validate
```

## Closeout format for Codex

Use this closeout shape:

```md
## Changed

- `<file path>` — what changed and why.

## Verified

- `<command>` — passed/failed.

## Deployment notes

- Vercel root directory:
- Framework preset:
- Build command:
- Output directory:
- Production branch:
- Automatic update behavior:

## Not verified

- State anything that could not be verified locally.
```

## Non-goals

Do not:

- Move docs out of `docs-site/src/content/docs`.
- Replace Astro with another static site generator.
- Add GitHub Pages config as part of this task.
- Add a broad CI/CD redesign.
- Change package managers.
- Add environment variables unless the docs site actually requires them.
