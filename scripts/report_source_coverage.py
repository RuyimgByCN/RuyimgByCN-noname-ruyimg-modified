#!/usr/bin/env python3
"""Report the 37-source coverage split: converted / installed-but-empty / no package.

The extension ledger tracks 37 upstream sources.  For the "来源覆盖" acceptance item
what matters is not how many packages exist, but **how many carry converted rule
content**:

* **converted**       — package has at least one card/skill/general entry
* **installed-empty** — package exists (installable) but has **zero** entries
* **no-package**      — no package at all

Mode-type sources (`武皇争霸`, `雷霆万钧`, `boss-play`) land in `installed-empty`
even though upstream has rule logic in `extension.js`: the entries were never
converted.  They are a **coverage** gap, not a rule-difference one.

Usage:
    python3 -B scripts/report_source_coverage.py            # TSV + summary
    python3 -B scripts/report_source_coverage.py --check    # exit 1 when counts drift
"""

from __future__ import annotations

import argparse
import glob
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

#: Measured at rules v322; update together with the evidence README.
EXPECTED = {"converted": 12, "installed-empty": 20, "no-package": 5}


def _load_mapper():
    spec = importlib.util.spec_from_file_location("map_package_sources", ROOT / "scripts" / "map_package_sources.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def entries(packages: list[str]) -> tuple[int, str]:
    """(条目数合计, upstream_path) for a source's packages."""
    total = 0
    upstream = ""
    for package in packages:
        for asset in glob.glob(str(ROOT / "packages" / "extensions" / package / "assets" / "*.json")):
            try:
                doc = json.loads(Path(asset).read_text(encoding="utf-8"))
            except Exception:
                continue
            if not upstream and doc.get("upstream_path"):
                upstream = doc["upstream_path"]
            for key in ("cards", "skills", "generals"):
                value = doc.get(key)
                if isinstance(value, list):
                    total += len(value)
    return total, upstream


def rows() -> list[list[str]]:
    mapper = _load_mapper()
    by_source, _names = mapper.mapping()
    all_sources = sorted(set(by_source) | set(getattr(mapper, "NO_PACKAGE", set())))
    out = []
    for source in all_sources:
        packages = by_source.get(source) or []
        if not packages:
            out.append([source, "", "0", "", "no-package"])
            continue
        count, upstream = entries(packages)
        cls = "converted" if count > 0 else "installed-empty"
        out.append([source, ",".join(packages), str(count), upstream, cls])
    out.sort(key=lambda r: (r[4], r[0]))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit 1 when the class counts drift from EXPECTED")
    ap.add_argument("--write", type=Path, help="write TSV to this path")
    args = ap.parse_args()

    data = rows()
    header = ["source", "packages", "entries", "upstream_path", "class"]
    text = "\n".join(["\t".join(header)] + ["\t".join(r) for r in data]) + "\n"

    counts: dict[str, int] = {}
    for row in data:
        counts[row[4]] = counts.get(row[4], 0) + 1

    if args.write:
        args.write.parent.mkdir(parents=True, exist_ok=True)
        args.write.write_text(text, encoding="utf-8")
        print(f"wrote {args.write} ({len(data)} sources)", file=sys.stderr)
    else:
        print(text, end="")

    print(f"合计 {len(data)} 个来源：{counts}", file=sys.stderr)
    for cls, expected in EXPECTED.items():
        actual = counts.get(cls, 0)
        if actual != expected:
            print(f"⚠ {cls}: 期望 {expected}，实测 {actual}", file=sys.stderr)
            if args.check:
                return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
