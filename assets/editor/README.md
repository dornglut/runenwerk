# Editor Config

`config.ron` contains local editor/tooling settings.

Current fields:
- `blender_bin`: Optional absolute path to Blender executable used for `.blend` export.

Example:

```ron
(
  blender_bin: "/Users/<you>/Library/Application Support/Steam/steamapps/common/Blender/Blender.app/Contents/MacOS/Blender",
)
```

Note: `BLENDER_BIN` environment variable overrides this value when set.
