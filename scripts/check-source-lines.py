#!/usr/bin/env python3
"""Enforce the 2,000-line limit for maintained source and test files.

Rust non-test files additionally have a 1,000 effective-line limit (comments, blank lines and
`#[cfg(test)]` code excluded; files under `tests/` exempt).
"""

import os
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rust_effective_lines import effective_lines  # noqa: E402

LIMIT = 2000
RUST_EFFECTIVE_LIMIT = 1000
RUST_EFFECTIVE_ENFORCED = True
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


def rust_effective_oversized(root):
    for path in source_files(root):
        relative = path.relative_to(root)
        if path.suffix != ".rs" or "tests" in relative.parts:
            continue
        count = effective_lines(path.read_text(encoding="utf-8", errors="replace"))
        if count > RUST_EFFECTIVE_LIMIT:
            yield relative, count


def main():
    root = Path(__file__).resolve().parents[1]
    failures = list(oversized(root))
    for path, count in failures:
        print(f"{path}: {count} lines (limit {LIMIT})", file=sys.stderr)
    rust = list(rust_effective_oversized(root))
    for path, count in rust:
        print(f"{path}: {count} effective Rust lines (limit {RUST_EFFECTIVE_LIMIT})", file=sys.stderr)
    if failures or (rust and RUST_EFFECTIVE_ENFORCED):
        return 1
    note = f"; {len(rust)} Rust files over {RUST_EFFECTIVE_LIMIT} effective lines (not yet enforced)" if rust else f"; Rust non-test files <= {RUST_EFFECTIVE_LIMIT} effective lines"
    print(f"Source line limit passed: {sum(1 for _ in source_files(root))} files <= {LIMIT} lines{note}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
