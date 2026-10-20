#!/usr/bin/env python3
"""扫全库「宿主行为键」是否被任何扩展包引用（= 可达）。

**为什么需要**：宿主行为（`host:xxx`）只有被某个扩展包通过
`skills[].mapping` / `generals[].skills[].mapping` / `cards[].skills` 引用，
才会被授予到角色身上。**没被引用的行为「已实现但永不执行」**（U08/Gwent 已实证）。

用法：
    python3 -B scripts/host_key_reachability.py            # 汇总
    python3 -B scripts/host_key_reachability.py --list     # 列出全部不可达键
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KEYS = ROOT / "crates/engine-native/src/content/host/keys.rs"
PKGS = ROOT / "packages/extensions"


def host_keys() -> list[str]:
    text = KEYS.read_text(encoding="utf-8")
    return sorted(set(re.findall(r'"(host:[A-Za-z0-9_]+)"', text)))


def declared_in_host_rs() -> set[str]:
    """`content/host.rs` 的 `CardKind → HostSkillDecl` 表（**第三条挂载路径**）。

    该表把「卡牌种类」直接绑定到宿主行为（如 `CardKind::Chunbing =>
    behavior: HostBehavior::GujianFoodChunbing`），**不经过包数据** ——
    漏掉这条路径会把大量「卡牌宿主行为」误判为不可达。
    """
    keys_src = KEYS.read_text(encoding="utf-8")
    beh2key = dict(re.findall(r'HostBehavior::(\w+)\s*=>\s*"(host:[A-Za-z0-9_]+)"', keys_src))
    host_rs = (ROOT / "crates/engine-native/src/content/host.rs").read_text(encoding="utf-8")
    return {
        beh2key[b]
        for b in re.findall(r"behavior:\s*HostBehavior::(\w+)", host_rs)
        if b in beh2key
    }


def referenced() -> set[str]:
    """收集所有包数据里出现的 `host:` 引用（任何 JSON 字段、任何层级）。"""
    refs: set[str] = set()

    def walk(node: object) -> None:
        if isinstance(node, dict):
            for v in node.values():
                walk(v)
        elif isinstance(node, list):
            for v in node:
                walk(v)
        elif isinstance(node, str) and node.startswith("host:"):
            refs.add(node)

    for path in PKGS.glob("*/assets/*.json"):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            continue
        walk(data)
    return refs


def main(argv: list[str]) -> int:
    keys = host_keys()
    refs = referenced()
    declared = declared_in_host_rs()
    reachable = refs | declared
    unreachable = [k for k in keys if k not in reachable]
    print(
        f"宿主键 {len(keys)} 个；包引用 {len(refs)}；host.rs 声明 {len(declared)}；"
        f"并集 {len(reachable)}；仍无引用 {len(unreachable)}（上界）"
    )
    if "--list" in argv:
        for k in unreachable:
            print(f"  {k}")
    else:
        # 按前缀分组统计
        import collections

        groups: dict[str, int] = collections.Counter(k.split("_")[0] for k in unreachable)
        for g, n in sorted(groups.items(), key=lambda kv: -kv[1]):
            print(f"  {g:16s} {n}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
