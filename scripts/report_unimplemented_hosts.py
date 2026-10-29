#!/usr/bin/env python3
"""列出「在 `factory.rs` 里是 `(None, None)`」的宿主行为，并按出现次数分级。

**背景**：`HostBehavior::X => (None, None)` 表示**既无主动面、也无效果步**。
但直接把它当「未实现」会误判 —— 效果可能写在别处（`metadata` / `filtering` / `step`）。

**判据**：数 `HostBehavior::X` 在 `content/host/**` 的**出现次数**：
- **11 次** = 纯注册样板（enum + keys×2 + factory + metadata + filtering + step 各 1）⇒ **无任何逻辑**；
- **13 次** = 多两处 metadata 变体 ⇒ **多半也无逻辑**，仍标「待核」；
- **≥21 次** = 至少一处有内容 ⇒ **不是「没做」**。

用法：
    python3 -B scripts/report_unimplemented_hosts.py           # 汇总
    python3 -B scripts/report_unimplemented_hosts.py --tsv F    # 写明细
"""

from __future__ import annotations

import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates/engine-native-content/src/content/host"
NONE_RE = re.compile(r"HostBehavior::(\w+)\s*=>\s*\(None,\s*None\)")
KEY_RE = re.compile(r'HostBehavior::(\w+)\s*=>\s*"host:([^"]+)"')
PURE = 11
TBD = 13


def analyze() -> tuple[list[tuple[str, str, int, str]], Counter]:
    factory = (HOST / "factory.rs").read_text(encoding="utf-8")
    none_beh = sorted(set(NONE_RE.findall(factory)))
    texts = {f: f.read_text(encoding="utf-8") for f in list(HOST.rglob("*.rs")) + list((ROOT / "crates/engine-native-kernel/src/vocabulary/host").rglob("*.rs"))}
    keys = {b: k for b, k in KEY_RE.findall((ROOT / "crates/engine-native-kernel/src/vocabulary/host/keys.rs").read_text(encoding="utf-8"))}

    rows: list[tuple[str, str, int, str]] = []
    dist: Counter = Counter()
    for beh in none_beh:
        n = sum(s.count("HostBehavior::" + beh) for s in texts.values())
        dist[n] += 1
        verdict = "纯注册(无逻辑)" if n == PURE else ("待核" if n == TBD else "有逻辑")
        rows.append((beh, keys.get(beh, ""), n, verdict))
    return rows, dist


def main(argv: list[str]) -> int:
    rows, dist = analyze()
    if "--tsv" in argv:
        out = ["行为\tkey\t出现次数\t判定"] + ["\t".join(str(c) for c in r) for r in rows]
        Path(argv[argv.index("--tsv") + 1]).write_text("\n".join(out) + "\n", encoding="utf-8")
        print(f"写入 {argv[argv.index('--tsv') + 1]}")

    by_verdict: dict[str, int] = defaultdict(int)
    for _b, _k, _n, v in rows:
        by_verdict[v] += 1
    print(f"(None, None) 共 {len(rows)}")
    print(f"分布 {dict(sorted(dist.items()))}")
    print(f"判定 {dict(by_verdict)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
