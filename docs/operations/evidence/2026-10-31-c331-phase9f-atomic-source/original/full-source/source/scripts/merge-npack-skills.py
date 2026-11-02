#!/usr/bin/env python3
"""Surgically replace skill modules/indexes in a host npack from a donor npack.

Keeps the host manifest ABI/version so unrelated goldens that pin the host
package ABI stay valid. Re-run `cargo run -p content-schema --example rehash_npack`
afterwards so logic_hash matches.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


INDEX_KEYS = (
    "skill_index",
    "trigger_index",
    "modifier_index",
    "active_index",
    "view_as_index",
)


def load(path: Path) -> dict:
    return json.loads(path.read_text())


def skill_ids(pkg: dict) -> set[int]:
    return {int(s["skill_def_id"]) for s in pkg.get("skill_index", [])}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", required=True)
    parser.add_argument("--donor", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    host = load(Path(args.host))
    donor = load(Path(args.donor))
    ids = skill_ids(donor)
    if not ids:
        raise SystemExit("donor npack has no skills")

    donor_modules = {m["name"]: m for m in donor.get("modules", [])}
    host["modules"] = [
        m for m in host.get("modules", []) if not any(
            str(sid) in m.get("name", "")
            or any(
                t.get("module") == m.get("name") and int(t.get("skill_def_id", 0)) in ids
                for t in host.get("trigger_index", [])
            )
            for sid in ids
        )
    ]
    # Drop host modules referenced by replaced triggers
    replaced_modules = {
        t["module"]
        for t in host.get("trigger_index", [])
        if int(t.get("skill_def_id", 0)) in ids and t.get("module")
    }
    host["modules"] = [m for m in host["modules"] if m.get("name") not in replaced_modules]
    for name, module in donor_modules.items():
        host["modules"] = [m for m in host["modules"] if m.get("name") != name]
        host["modules"].append(module)

    for key in INDEX_KEYS:
        host_items = [
            item
            for item in host.get(key, [])
            if int(item.get("skill_def_id", 0)) not in ids
        ]
        host_items.extend(donor.get(key, []))
        host[key] = host_items

    if host.get("trigger_index"):
        used = set()
        nxt = 1
        for trigger in host["trigger_index"]:
            tid = int(trigger.get("trigger_def_id", 0))
            if tid in used or tid == 0:
                while nxt in used:
                    nxt += 1
                trigger["trigger_def_id"] = nxt
                used.add(nxt)
                nxt += 1
            else:
                used.add(tid)

    host["manifest"]["content_schema_version"] = 4 if any(
        t.get("timing", "normal") != "normal"
        or t.get("priority", 0) != 0
        or t.get("order_selectable", True) is not True
        for t in host.get("trigger_index", [])
    ) else 3

    Path(args.out).write_text(json.dumps(host, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print(
        f"merged {sorted(ids)} into {args.out}: "
        f"modules={len(host['modules'])} skills={len(host['skill_index'])}"
    )


if __name__ == "__main__":
    main()
