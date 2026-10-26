#!/usr/bin/env python3
"""Audit the functional EffectRequest split and reconstruct its serialized order.

The historical command name remains supported. Both report mode and --check
validate the current macro registry; --check also rejects files over --limit.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EFFECT_RS = ROOT / "crates" / "rule-api" / "src" / "effect.rs"
LIMIT = 2000
ARM = re.compile(
    r"^    \((?P<first>[A-Z]\w*)\s+\[\$\(\$tail:tt\)\*\]\s+"
    r"\[\$\(\$variants:tt\)\*\]\)\s*=>\s*\{\s*\n"
    r"\s*collect_variants!\s*\{\s*\[\$\(\$tail\)\*\]\s*\[\s*\n"
    r"\s*\$\(\$variants\)\*[^\S\n]*\n(?P<body>.*?)"
    r"^\s*\]\s*\}\s*\n\s*\};",
    re.MULTILINE | re.DOTALL,
)


def parse_variant_definitions(text: str) -> list[tuple[str, str]]:
    """Read complete, documented variants from one macro segment or enum body.

    This deliberately accepts the maintained four-space, struct/unit variant
    grammar rather than attempting to parse arbitrary Rust. Unknown syntax fails.
    """
    lines = text.splitlines()
    variants = []
    cursor = 0
    while cursor < len(lines):
        if not lines[cursor].strip():
            cursor += 1
            continue
        first = cursor
        while cursor < len(lines) and lines[cursor].lstrip().startswith(("//", "#[")):
            cursor += 1
        if cursor == len(lines):
            raise ValueError("variant documentation has no definition")
        match = re.fullmatch(r"    ([A-Z]\w*)\s*(\{|,)", lines[cursor])
        if not match:
            raise ValueError(f"invalid variant definition: {lines[cursor]!r}")
        depth = 0
        while cursor < len(lines):
            line = lines[cursor]
            if not line.lstrip().startswith("//"):
                depth += line.count("{") - line.count("}")
            cursor += 1
            if depth == 0 and line.rstrip().endswith(","):
                break
        else:
            raise ValueError(f"unterminated variant {match[1]}")
        variants.append((match[1], "\n".join(lines[first:cursor])))
    return variants


def variant_digest(variants: list[tuple[str, str]]) -> str:
    """Hash full definitions in order, retaining docs, attributes and field order."""
    normalized = "\n".join(re.sub(r"\s+", "", definition) for _, definition in variants)
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def read_split(path: Path = EFFECT_RS) -> tuple[list[tuple[str, str]], dict[str, int]]:
    """Return ordered (name, definition) pairs and physical source file sizes.

    Each declared family must be used, each registry segment must exist exactly
    once, and every defined segment must be consumed exactly once.
    """
    text = path.read_text(encoding="utf-8")
    modules = re.findall(r"^mod ([a-z_]\w*);$", text, re.MULTILINE)
    if not modules or len(modules) != len(set(modules)):
        raise ValueError("missing or duplicate module declarations")
    registries = re.findall(
        r"^collect_variants!\s*\{\s*\[(.*?)\]\s*\[\s*\]\s*\}",
        text, re.MULTILINE | re.DOTALL,
    )
    if len(registries) != 1:
        raise ValueError("expected one ordered variant registry")
    pairs = re.findall(r"\(\s*([a-z_]\w*)\s*,\s*([A-Z]\w*)\s*\)", registries[0])
    residue = re.sub(r"\(\s*[a-z_]\w*\s*,\s*[A-Z]\w*\s*\)", "", registries[0])
    if not pairs or residue.strip(" \t\r\n,"):
        raise ValueError("invalid variant registry")
    if len(pairs) != len(set(pairs)):
        raise ValueError("duplicate registry segment")
    if {module for module, _ in pairs} != set(modules):
        raise ValueError("declared and used modules differ")

    directory = path.with_suffix("")
    if {p.stem for p in directory.glob("*.rs")} != set(modules):
        raise ValueError("declared modules and source files differ")
    sizes = {path.name: len(text.splitlines())}
    segments = {}
    for module in modules:
        source_path = directory / f"{module}.rs"
        source = source_path.read_text(encoding="utf-8")
        sizes[f"{directory.name}/{source_path.name}"] = len(source.splitlines())
        matches = list(ARM.finditer(source))
        # Reject unparsed arms or executable text outside the known macro wrapper.
        wrapper = ARM.sub("", source)
        wrapper = re.sub(r"(?m)^\s*//[^\n]*", "", wrapper)
        wrapper = re.sub(r"\s+", "", wrapper)
        if not matches or wrapper != "macro_rules!variants{}pub(super)usevariants;":
            raise ValueError(f"invalid macro segments in {source_path.name}")
        for match in matches:
            key = (module, match["first"])
            if key in segments:
                raise ValueError(f"duplicate defined segment {key}")
            definitions = parse_variant_definitions(match["body"])
            if not definitions or definitions[0][0] != key[1]:
                raise ValueError(f"segment key does not match its first variant: {key}")
            segments[key] = definitions

    missing = set(pairs) - segments.keys()
    unused = segments.keys() - set(pairs)
    if missing or unused:
        raise ValueError(f"missing segments: {sorted(missing)}; unused segments: {sorted(unused)}")
    variants = [variant for pair in pairs for variant in segments[pair]]
    names = [name for name, _ in variants]
    if len(names) != len(set(names)):
        raise ValueError("duplicate variant definitions")
    return variants, sizes


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="also enforce physical source line limits")
    ap.add_argument("--limit", type=int, default=LIMIT, help=f"line limit (default {LIMIT})")
    args = ap.parse_args()
    try:
        variants, sizes = read_split()
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    print(f"EffectRequest: variants {len(variants)}, functional modules {len(sizes) - 1}")
    for name, lines in sizes.items():
        status = "OK" if lines <= args.limit else f"OVER by {lines - args.limit}"
        print(f"{name:<40} {lines:>5} lines  {status}")
    print(f"ordered definition SHA256: {variant_digest(variants)}")
    return int(args.check and any(lines > args.limit for lines in sizes.values()))


if __name__ == "__main__":
    raise SystemExit(main())
