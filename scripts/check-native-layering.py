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

BASELINE = 0
SRC = Path("crates/engine-native/src")
KERNEL_MODULES = (
    "ids", "rng", "choice", "event", "catalog", "state", "machine", "resolution",
    "vocabulary", "kernel", "snap", "archive", "rvm_ops", "rvm_bridge", "rvm_named",
)
USE = re.compile(r"\buse\s+([^;]+);", re.S)
TOKEN = re.compile(r"::|(?:r#)?[A-Za-z_]\w*|[{}*,]")


def use_paths(tree):
    """Expand Rust use groups, retaining aliases (including `crate::{self as root}`)."""
    tokens = TOKEN.findall(tree)
    index = 0

    def branch(prefix):
        nonlocal index
        path = list(prefix)
        while index < len(tokens):
            token = tokens[index]
            if token in (",", "}"):
                break
            index += 1
            if token == "::":
                continue
            if token == "{":
                while index < len(tokens) and tokens[index] != "}":
                    yield from branch(path)
                    if index < len(tokens) and tokens[index] == ",":
                        index += 1
                index += 1
                return
            if token == "as":
                alias = tokens[index].removeprefix("r#")
                index += 1
                yield path, alias
                return
            path.append(token.removeprefix("r#"))
        if path:
            yield path, path[-2] if path[-1] == "self" and len(path) > 1 else path[-1]

    yield from branch([])


def resolve_path(parts, module, aliases):
    first, *rest = parts
    if first in aliases:
        return aliases[first] + rest
    if first == "crate":
        return rest
    if first in ("self", "super"):
        path = list(module)
        for part in parts:
            if part == "super":
                path = path[:-1]
            elif part != "self":
                path.append(part)
        return path
    return parts


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
        code = "\n".join(line.split("//", 1)[0] for line in path.read_text(encoding="utf-8").splitlines())
        module = list(path.relative_to(root / SRC).with_suffix("").parts)
        if module[-1] == "mod":
            module.pop()
        imports = list(USE.finditer(code))
        declarations = [(match, list(use_paths(match.group(1)))) for match in imports]
        aliases = {}
        # Use declarations are not order-dependent in Rust.
        for _ in declarations:
            for _, paths in declarations:
                for parts, alias in paths:
                    resolved = resolve_path(parts, module, aliases)
                    if parts[0] in ("crate", "self", "super") or parts[0] in aliases:
                        aliases[alias] = resolved[:-1] if resolved[-1:] == ["self"] else resolved
        for match, paths in declarations:
            for parts, _ in paths:
                resolved = resolve_path(parts, module, aliases)
                if resolved[:1] in (["content"], ["engine"]):
                    yield path.relative_to(root), code.count("\n", 0, match.start()) + 1
        # Mask imports so grouped and ordinary use paths are counted once.
        body = list(code)
        for match in imports:
            body[match.start():match.end()] = " " * (match.end() - match.start())
        body = "".join(body)
        roots = "|".join(re.escape(name) for name in ("crate", "self", "super", "content", *aliases))
        paths = re.compile(r"\b(?:r#)?(?:" + roots + r")\s*(?:::\s*(?:r#)?[A-Za-z_]\w*)+")
        for match in paths.finditer(body):
            parts = [part.removeprefix("r#") for part in re.findall(r"(?:r#)?[A-Za-z_]\w*", match.group())]
            if resolve_path(parts, module, aliases)[:1] in (["content"], ["engine"]):
                yield path.relative_to(root), code.count("\n", 0, match.start()) + 1


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
