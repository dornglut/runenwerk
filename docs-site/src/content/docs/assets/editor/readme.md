---
title: "Editor Config"
description: "Documentation for Editor Config."
---

# Editor Config

`config.ron` contains local editor/tooling settings.

Supported fields:
- `blender_bin`: Optional absolute path to Blender executable used for `.blend` export.

Example:

```ron
(
  blender_bin: "/Users/<you>/Library/Application Support/Steam/steamapps/common/Blender/Blender.app/Contents/MacOS/Blender",
)
```

Note: `BLENDER_BIN` environment variable overrides this value when set.
Prefer `BLENDER_BIN` in CI/shared environments to avoid machine-specific config churn.
