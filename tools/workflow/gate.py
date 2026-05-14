#!/usr/bin/env python3
"""
Deprecated compatibility entrypoint for old workflow commands.

File: tools/workflow/gate.py
Function: main
"""

from __future__ import annotations


def main() -> int:
    print("tools/workflow/gate.py is deprecated.")
    print("Use Taskfile as the canonical workflow entrypoint:")
    print("  task --list")
    print("  task roadmap:validate")
    print('  task batch:propose -- --goal "<goal>" --scope L0')
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
