#!/usr/bin/env python3
"""Ratchet engine-native's kernel layer away from content and engine.

The kernel layer (see KERNEL_MODULES) must eventually stop referencing
`crate::content` and `crate::engine`, so it can become its own crate.  Every
non-comment occurrence of such a path is counted; the count may only go down.
When it drops, lower BASELINE in the same change.
"""

import os
from pathlib import Path
import re
import sys

BASELINE = 240
SRC = Path("crates/engine-native/src")
KERNEL_MODULES = (
    "ids", "rng", "choice", "event", "catalog", "state", "machine", "resolution",
    "kernel", "snap", "archive", "rvm_ops", "rvm_bridge", "rvm_named",
)
UPWARD = re.compile(r"\bcontent::(?:host|standard|extension|pile|names)\b|\bcrate::engine::")


def kernel_files(root):
    src = root / SRC
    for name in KERNEL_MODULES:
        single = src / f"{name}.rs"
        if single.is_file():
            yield single
        for directory, children, files in os.walk(src / name):
            children.sort()
            for file in sorted(files):
                if file.endswith(".rs"):
                    yield Path(directory) / file


def upward_references(root):
    for path in kernel_files(root):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            code = line.split("//", 1)[0]
            for _ in UPWARD.finditer(code):
                yield path.relative_to(root), number


def main():
    root = Path(__file__).resolve().parents[1]
    refs = list(upward_references(root))
    count = len(refs)
    if count > BASELINE:
        per_file = {}
        for path, _ in refs:
            per_file[path] = per_file.get(path, 0) + 1
        for path, n in sorted(per_file.items(), key=lambda item: -item[1])[:10]:
            print(f"{path}: {n}", file=sys.stderr)
        print(
            f"engine-native kernel layer references content/engine {count} times "
            f"(baseline {BASELINE}); route new content needs through catalog hooks",
            file=sys.stderr,
        )
        return 1
    note = f"; lower BASELINE to {count}" if count < BASELINE else ""
    print(f"engine-native layering passed: {count} upward references <= {BASELINE}{note}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
