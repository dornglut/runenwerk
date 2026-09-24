#!/usr/bin/env python3
"""Export a Blender .blend file to a GLB foreign-reference artifact."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--blender", default="blender", help="Path to the Blender executable")
    parser.add_argument("--input", required=True, help="Source .blend file")
    parser.add_argument("--output", required=True, help="Destination .glb file")
    args = parser.parse_args(argv)

    blender = shutil.which(args.blender) if not Path(args.blender).is_file() else args.blender
    if blender is None:
        print(f"missing Blender executable: {args.blender}", file=sys.stderr)
        return 2

    source = Path(args.input)
    if not source.is_file():
        print(f"missing .blend source: {source}", file=sys.stderr)
        return 3
    if source.suffix.lower() != ".blend":
        print(f"input is not a .blend file: {source}", file=sys.stderr)
        return 4

    output = Path(args.output)
    if output.suffix.lower() != ".glb":
        print(f"output must use .glb extension: {output}", file=sys.stderr)
        return 5
    output.parent.mkdir(parents=True, exist_ok=True)

    export_script = (
        "import bpy; "
        f"bpy.ops.export_scene.gltf(filepath={str(output)!r}, export_format='GLB')"
    )
    result = subprocess.run(
        [
            blender,
            "--background",
            str(source),
            "--python-expr",
            export_script,
        ],
        check=False,
    )
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
