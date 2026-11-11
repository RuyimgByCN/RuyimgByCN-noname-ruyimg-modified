#!/usr/bin/env python3
"""Initial generator for docs/migration/g0/inventory_status.json.

Extracts all 4,424 upstream entities from 37 sources, computes unique primary key
`{sid}:{cat}:{id}@{clean_ref}`, and applies initial demotion:
- 3,768 rule items -> status: "简化近似", difference: "简化近似（待复核）"
- 656 UI items -> status: "简化近似", difference: "示意（待对照）"
- Baseline: exactly 4,424 items, 0 collisions, 0 已复核等价.
"""

from __future__ import annotations

import json
from pathlib import Path
import sys

REPO = Path(__file__).resolve().parents[2]
OUT_FILE = REPO / "docs" / "migration" / "g0" / "inventory_status.json"
INV_FILE = REPO / "docs" / "migration" / "evidence" / "2026-09-23-extension-scope" / "inventory.json"

sys.path.insert(0, str(REPO / "scripts" / "legacy-harness"))
import decision34_inventory_builder as builder

RULE_CATEGORIES = {"generals", "skills", "cards", "modes", "configs"}
UI_CATEGORIES = {"ui_hooks", "surfaces"}
ALL_CATEGORIES = ("generals", "skills", "cards", "modes", "configs", "ui_hooks", "surfaces")

L11_UNDEFINED_SKILLS = {
    "txhj_jiaoxie", "txhj_jinwei", "txhj_jinxing", "txhj_quanqin",
    "txhj_xiehui", "txhj_yishi", "txhj_zhanhuo", "txhj_zhuosheng", "xinchenhui"
}


def generate_inventory_status() -> dict:
    inventory = json.loads(INV_FILE.read_text(encoding="utf-8"))
    formal_pkgs = builder.load_formal_packages()

    noname = [e for e in inventory["extensions"] if e["repo"] == "noname"]
    liuli = [e for e in inventory["extensions"] if e["repo"] == "noname-liuli"]

    sources: list[tuple[str, dict]] = []
    for i, e in enumerate(noname, 1):
        sources.append((f"U{i:02d}", e))
    for i, e in enumerate(liuli, 1):
        sources.append((f"L{i:02d}", e))

    assert len(sources) == 37, f"Expected 37 sources, got {len(sources)}"

    ledger = {
        "version": 1,
        "updated_at": "2026-11-11T07:00:00Z",
        "total_items": 0,
        "counts": {
            "已复核等价": 0,
            "简化近似": 0,
            "缺失": 0,
            "上游未定义": 0,
        },
        "items": {},
    }

    cat_counts: dict[str, int] = {cat: 0 for cat in ALL_CATEGORIES}

    for sid, entry in sources:
        src_dir = builder.WORKSPACE / entry["path"]
        inv = builder.extract_source_inventory(sid, src_dir, formal_pkgs)

        for cat in ALL_CATEGORIES:
            cat_items = inv.get(cat, [])
            cat_counts[cat] += len(cat_items)
            for it in cat_items:
                raw_ref = it["ref"]
                clean_ref = raw_ref.strip("`")
                item_id = it["id"]
                pk = f"{sid}:{cat}:{item_id}@{clean_ref}"

                assert pk not in ledger["items"], f"Primary key collision detected: {pk}"

                diff: str | None = None
                note: str | None = None
                if cat in RULE_CATEGORIES:
                    diff = "简化近似（待复核）"
                elif cat in UI_CATEGORIES:
                    diff = "示意（待对照）"

                if sid == "L11" and cat == "generals":
                    if any(sk in it.get("summary", "") for sk in L11_UNDEFINED_SKILLS):
                        note = "何进诛宦 NPC 引用但全仓无 lib.skill 定义，保持运行时惰性不自造"

                ledger["items"][pk] = {
                    "source_id": sid,
                    "category": cat,
                    "id": item_id,
                    "upstream_ref": clean_ref,
                    "target": it.get("target", ""),
                    "status": "简化近似",
                    "reviewer": None,
                    "review_date": None,
                    "upstream_file_line": None,
                    "test_name": None,
                    "difference": diff,
                    "note": note,
                    "reason": None,
                }

    total = len(ledger["items"])
    ledger["total_items"] = total
    ledger["counts"]["简化近似"] = total

    # Strict invariant assertions
    assert total == 4424, f"Expected 4,424 items, got {total}"
    assert cat_counts["generals"] == 834, f"generals expected 834, got {cat_counts['generals']}"
    assert cat_counts["skills"] == 1908, f"skills expected 1908, got {cat_counts['skills']}"
    assert cat_counts["cards"] == 377, f"cards expected 377, got {cat_counts['cards']}"
    assert cat_counts["modes"] == 215, f"modes expected 215, got {cat_counts['modes']}"
    assert cat_counts["configs"] == 434, f"configs expected 434, got {cat_counts['configs']}"
    assert cat_counts["ui_hooks"] == 507, f"ui_hooks expected 507, got {cat_counts['ui_hooks']}"
    assert cat_counts["surfaces"] == 149, f"surfaces expected 149, got {cat_counts['surfaces']}"
    assert ledger["counts"]["已复核等价"] == 0
    assert ledger["counts"]["简化近似"] == 4424
    assert ledger["counts"]["缺失"] == 0
    assert ledger["counts"]["上游未定义"] == 0

    return ledger


def main() -> None:
    data = generate_inventory_status()
    OUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    OUT_FILE.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Successfully generated {data['total_items']} items into {OUT_FILE}")
    print("Counts:", data["counts"])


if __name__ == "__main__":
    main()
