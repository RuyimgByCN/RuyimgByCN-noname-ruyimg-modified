#!/usr/bin/env bash
# O4 交付测量辅助：构建 standard 全依赖闭包的 ContentBundle JSON。
# Usage: scripts/o4_build_bundle.sh <out.json>
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:?usage: o4_build_bundle.sh <out.json>}"
cd "$ROOT"

NPACKS=$(python3 - <<'PY'
import json, os
def find_npack(pid):
    for root, dirs, fs in os.walk('content'):
        if f'{pid}.npack.json' in fs:
            return os.path.join(os.getcwd(), root, f'{pid}.npack.json')
need = ['standard', 'refresh']
# O4 冒烟实测发现：yijiang.liyan/tw.huanjie/tw.sb_sp_zhugeliang 的 roster
# 跨包引用 bingshi/newjiang/sb 提供的技能（manifest dependencies 未声明
# ——存量内容债，见 evidence 2026-10-14-o4-wasm-delivery）。种子补齐三包。
need += ['bingshi', 'newjiang', 'sb']
order, seen = [], set()
while need:
    pid = need.pop(0)
    if pid in seen:
        continue
    seen.add(pid)
    p = find_npack(pid)
    if not p:
        raise SystemExit(f'missing npack for {pid}')
    order.append(p)
    for d in json.load(open(p))['manifest'].get('dependencies', []):
        need.append(d['package_id'])
print('\n'.join(order))
PY
)
COUNT=$(echo "$NPACKS" | wc -l | tr -d ' ')
ARGS=()
while IFS= read -r line; do ARGS+=("$line"); done <<< "$NPACKS"
cargo run --release --offline -p engine-wasm --example o4_bundle -- "${ARGS[@]}" "$OUT"
