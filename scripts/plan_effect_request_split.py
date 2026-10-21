#!/usr/bin/env python3
"""Plan the split of the 815-variant `EffectRequest` enum into modules.

Background
----------
`crates/rule-api/src/effect.rs` is 10,778 lines, of which 10,573 are the body of a
single `pub enum EffectRequest` with 815 variants.  The source line gate
(`scripts/check-source-lines.py`, limit 2,000) therefore fails on that one file.

The gate cannot be satisfied by moving the enum to another file: the body is the
same size wherever it lives.  It must be **split by function** into several
enums, each of which stays under the limit.

This script does the plan, not the refactor.  It:

1. parses `effect.rs` and measures each variant's line span (docs + body);
2. assigns every variant to a domain module by name prefix/heuristic;
3. prints the projected size of each module;
4. `--check` fails when any projected module exceeds the limit, so the grouping
   can be tuned before anyone touches the source.

Why a plan instead of the refactor: splitting the enum changes the type of every
call site, and 1,069 of the 1,217 referencing files are untracked (other agents'
work), so the edit cannot be performed here.

Usage
-----
    python3 -B scripts/plan_effect_request_split.py            # report
    python3 -B scripts/plan_effect_request_split.py --check    # exit 1 if any module > limit
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EFFECT_RS = ROOT / "crates" / "rule-api" / "src" / "effect.rs"
LIMIT = 2000
ENUM_NAME = "pub enum EffectRequest"

#: Ordered (module, predicate) pairs.  First match wins, so put narrow rules
#: before broad ones.  Each predicate receives the variant name.
GROUPS: list[tuple[str, "callable"]] = []


def group(name: str, *patterns: str) -> None:
    rx = [re.compile(p) for p in patterns]

    def predicate(variant: str, _rx=rx) -> bool:
        return any(p.search(variant) for p in _rx)

    GROUPS.append((name, predicate))


group("draw", r"^Draw", r"CardsDrawn", r"Gain$")
group("damage", r"Damage", r"^Deal", r"^Recover", r"^LoseHp", r"MaxHp")
group(
    "cards",
    r"^Move", r"^Discard", r"^Take", r"^Remove", r"Card", r"^Reveal", r"^Pindian",
)
group("choice", r"^RequestChoice", r"Choice", r"^Choose", r"^Ask", r"^Query")
group("skills", r"Skill", r"^Add", r"^Remove$")
group("marks", r"Mark")
group("turn", r"Turn", r"^Advance", r"^SetTurn", r"Phase")
group(
    "life",
    r"Dying", r"^Kill", r"Death", r"^Rescue", r"^Revive", r"^Bury",
)
group("emit", r"^Emit", r"Event", r"Debate")
# Per-skill resolvers, judge-roll helpers and Sha plumbing form their own lumps.
# `resolve` alone is far over the limit (405 variants / ~5.7k lines): before the
# refactor it must be subdivided by source package (shahai / swd / gujian /
# yunchou / hearth / ...).  That taxonomy is a design decision, so this script
# reports the shortfall instead of inventing one.
group("resolve", r"^Resolve")
group("judge", r"Judge")
group("sha", r"Sha")
group("givegain", r"^(Give|Gain)")
group("target", r"^Target", r"Target")
group("misc", r".")


def parse_variants(path: Path) -> tuple[list[tuple[str, int, int]], int]:
    """Return [(name, first_line, line_span)] plus the enum body length.

    Each variant owns its body plus any contiguous `///` / `#[..]` lines directly
    above it; blank lines and the previous variant's terminator end the span.
    """
    lines = path.read_text(encoding="utf-8").splitlines()
    start = next(i for i, l in enumerate(lines) if l.startswith(ENUM_NAME))
    variants: list[tuple[str, int, int]] = []
    idx = start + 1
    while idx < len(lines):
        m = re.match(r"^    ([A-Z][A-Za-z0-9_]*)\b", lines[idx])
        if not m:
            idx += 1
            continue
        # Depth 1 == inside the enum.  A variant ends on the first line that
        # returns to that depth and terminates the variant (`},` / `),` / `,`).
        depth = 1
        end = idx
        j = idx
        while j < len(lines):
            depth += lines[j].count("{") - lines[j].count("}")
            if depth == 1 and lines[j].rstrip().endswith(","):
                end = j
                break
            j += 1
        first = idx
        while first - 1 > start:
            prev = lines[first - 1].strip()
            if prev.startswith("///") or prev.startswith("#["):
                first -= 1
            else:
                break
        variants.append((m.group(1), first, end - first + 1))
        idx = end + 1
    return variants, len(lines) - start


def plan(variants: list[tuple[str, int, int]]) -> dict[str, list[tuple[str, int]]]:
    out: dict[str, list[tuple[str, int]]] = {name: [] for name, _ in GROUPS}
    for name, _line, span in variants:
        for module, predicate in GROUPS:
            if predicate(name):
                out[module].append((name, span))
                break
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit 1 if any planned module exceeds the limit")
    ap.add_argument("--limit", type=int, default=LIMIT, help=f"line limit (default {LIMIT})")
    args = ap.parse_args()

    if not EFFECT_RS.exists():
        print(f"missing {EFFECT_RS}", file=sys.stderr)
        return 2

    variants, body = parse_variants(EFFECT_RS)
    total_lines = len(EFFECT_RS.read_text(encoding="utf-8").splitlines())
    print(f"{EFFECT_RS.relative_to(ROOT)}: {total_lines} lines, enum body {body}, variants {len(variants)}")

    grouped = plan(variants)
    over = []
    print(f"\n{'module':<10}{'variants':>9}{'lines':>8}  status")
    for module, items in grouped.items():
        lines = sum(span for _n, span in items)
        status = "OK" if lines <= args.limit else f"OVER by {lines - args.limit}"
        if lines > args.limit:
            over.append((module, lines))
        print(f"{module:<10}{len(items):>9}{lines:>8}  {status}")

    planned = sum(span for items in grouped.values() for _n, span in items)
    print(f"\naccounted {planned}/{body} enum-body lines ({100.0 * planned / body:.1f}%)")

    if over:
        print(f"\n{len(over)} module(s) exceed {args.limit} lines; refine the grouping in GROUPS.")
        return 1 if args.check else 0
    print(f"\nall projected modules <= {args.limit} lines")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
