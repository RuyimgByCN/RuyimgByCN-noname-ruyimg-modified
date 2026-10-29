#!/usr/bin/env python3
"""Keep the native kernel independent of content and the engine facade.

Scan every mandatory kernel module and the crate root; also check the normal
Cargo dependency graph so aliases and intermediate workspace crates cannot
reverse the layer direction.
"""

import json
import os
from pathlib import Path
import re
import sys
import subprocess

BASELINE = 0
SRC = Path("crates/engine-native-kernel/src")
UPPER = ("content", "engine", "engine_native_content", "engine_native")
KERNEL_MODULES = (
    "ids", "rng", "choice", "event", "catalog", "state", "machine", "resolution",
    "vocabulary", "kernel", "snap", "archive", "rvm_ops", "rvm_bridge", "rvm_named",
)
USE = re.compile(r"\buse\s+([^;]+);", re.S)
EXTERN_CRATE = re.compile(r"\bextern\s+crate\s+(?:r#)?(?:engine_native_content|engine_native)\b")
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
    for name in ("lib", *KERNEL_MODULES):
        single = src / f"{name}.rs"
        module_root = src / name / "mod.rs"
        entry = single if single.is_file() else module_root
        if not entry.is_file() or not entry.read_text(encoding="utf-8").strip():
            raise ValueError(f"missing or empty mandatory kernel module: {entry}")
    for directory, children, files in os.walk(src):
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
        for declaration in EXTERN_CRATE.finditer(code):
            yield path.relative_to(root), code.count("\n", 0, declaration.start()) + 1
        imports = list(USE.finditer(code))
        declarations = [(match, list(use_paths(match.group(1)))) for match in imports]
        aliases = {}
        # Use declarations are not order-dependent in Rust.
        for _ in declarations:
            for _, paths in declarations:
                for parts, alias in paths:
                    resolved = resolve_path(parts, module, aliases)
                    if parts[0] in ("crate", "self", "super") or parts[0] in aliases or parts[0] in UPPER:
                        aliases[alias] = resolved[:-1] if resolved[-1:] == ["self"] else resolved
        for match, paths in declarations:
            for parts, _ in paths:
                resolved = resolve_path(parts, module, aliases)
                if resolved and resolved[0] in UPPER:
                    yield path.relative_to(root), code.count("\n", 0, match.start()) + 1
        # Mask imports so grouped and ordinary use paths are counted once.
        body = list(code)
        for match in imports:
            body[match.start():match.end()] = " " * (match.end() - match.start())
        body = "".join(body)
        roots = "|".join(re.escape(name) for name in ("crate", "self", "super", *UPPER, *aliases))
        paths = re.compile(r"\b(?:r#)?(?:" + roots + r")\s*(?:::\s*(?:r#)?[A-Za-z_]\w*)+")
        for match in paths.finditer(body):
            parts = [part.removeprefix("r#") for part in re.findall(r"(?:r#)?[A-Za-z_]\w*", match.group())]
            if resolve_path(parts, module, aliases)[0] in UPPER:
                yield path.relative_to(root), code.count("\n", 0, match.start()) + 1


def dependency_violations(metadata):
    packages = {package["name"]: package for package in metadata["packages"]}
    layers = ("engine-native-kernel", "engine-native-content", "engine-native")
    for name in layers:
        if name not in packages:
            raise ValueError(f"missing mandatory native package: {name}")
    violations = []
    for index, source in enumerate(layers[:-1]):
        pending: list[tuple[str, ...]] = [(source,)]
        visited = set()
        while pending:
            chain = pending.pop()
            name = chain[-1]
            if name in visited:
                continue
            visited.add(name)
            for dependency in packages.get(name, {}).get("dependencies", []):
                if dependency["kind"] is not None:
                    continue
                target = dependency["name"]
                path = (*chain, target)
                if target in layers[index + 1:]:
                    violations.append(" -> ".join(path))
                else:
                    pending.append(path)
    return violations


def main():
    root = Path(__file__).resolve().parents[1]
    try:
        refs = list(upward_references(root))
        metadata = subprocess.run(
            ["cargo", "metadata", "--offline", "--no-deps", "--format-version", "1"],
            cwd=root, check=True, capture_output=True, text=True,
        )
        violations = dependency_violations(json.loads(metadata.stdout))
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"native layering input error: {error}", file=sys.stderr)
        return 1
    if violations:
        print("native normal dependency direction violated: " + "; ".join(violations), file=sys.stderr)
        return 1
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
