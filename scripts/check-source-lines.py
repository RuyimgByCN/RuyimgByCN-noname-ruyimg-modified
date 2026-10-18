#!/usr/bin/env python3
"""Enforce the 2,000-line limit for maintained source and test files."""

import os
from pathlib import Path
import sys

LIMIT = 2000
ROOTS = ("crates", "apps", "packages", "scripts", "content", "fuzz", "tests")
EXTENSIONS = {
    ".rs", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".css",
    ".scss", ".html", ".py", ".sh", ".vue", ".svelte", ".wgsl", ".skill", ".inc",
}
SKIP = {
    "target", "node_modules", ".git", "dist", "build", ".venv", "__pycache__", "legacy-source",
}


def source_files(root):
    for name in ROOTS:
        for directory, children, files in os.walk(root / name):
            children[:] = sorted(child for child in children if child not in SKIP)
            for name in sorted(files):
                path = Path(directory) / name
                if path.suffix in EXTENSIONS:
                    yield path


def oversized(root):
    for path in source_files(root):
        with path.open("rb") as source:
            count = sum(1 for _ in source)
        if count > LIMIT:
            yield path.relative_to(root), count


def main():
    root = Path(__file__).resolve().parents[1]
    failures = list(oversized(root))
    for path, count in failures:
        print(f"{path}: {count} lines (limit {LIMIT})", file=sys.stderr)
    if failures:
        return 1
    print(f"Source line limit passed: {sum(1 for _ in source_files(root))} files <= {LIMIT} lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
