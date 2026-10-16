#!/usr/bin/env python3
"""从功能面清点 + 台账摘要生成 37 份 G0 来源文档（docs/migration/g0/）。"""
from __future__ import annotations

import json
import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
OUT = REPO / "docs" / "migration" / "g0"
INV = REPO / "docs" / "migration" / "evidence" / "2026-09-23-extension-scope" / "inventory.json"
FEAT = REPO / "docs" / "operations" / "evidence" / "2026-10-07-e01-feature-inventory" / "feature-inventory.json"
SUMMARY = REPO / "docs" / "operations" / "evidence" / "2026-10-07-e01-feature-inventory" / "feature-summary.json"
LEDGER = REPO / "docs" / "migration" / "upstream-extension-reconstruction.md"

# 目标接口（对照 native-extensions.md）
INTERFACE_MAP = {
    "content": "内容包（武将/技能/卡牌目录合并 + rule-vm/宿主行为）",
    "mode": "模式扩展（建局/胜负/关卡流程钩子）",
    "ai": "AI 策略（合法座位视图 + 预算/取消）",
    "config": "配置 schema + 内容锁敏感项",
    "ui": "界面/主题/部件（命令校验，不直改规则态）",
    "resource": "资源索引（立绘/语音/动画，命名空间）",
    "platform": "平台桥（导入/分享/更新/拖放）",
    "lifecycle": "包生命周期（安装/启停/升级/卸载）",
}

# 按来源名粗分目标接口（可被台账语义覆盖）
NAME_INTERFACES = {
    "3D精选": ["content", "resource", "lifecycle"],
    "boss": ["content", "mode", "lifecycle"],
    "cardpile": ["content", "config", "lifecycle"],
    "coin": ["config", "ui", "resource", "lifecycle"],
    "杀海拾遗": ["content", "config", "lifecycle"],
    "欢乐卡牌": ["content", "lifecycle"],
    "玩点论杀": ["content", "lifecycle"],
    "英雄杀": ["content", "resource", "lifecycle"],
    "AI优化": ["ai", "config", "lifecycle"],
    "SJ Settings": ["platform", "config", "lifecycle"],
    "wuxing": ["content", "lifecycle"],
    "全能搜索": ["ui", "config", "lifecycle"],
    "十周年UI": ["ui", "resource", "lifecycle"],
    "千幻聆音": ["ui", "resource", "config", "lifecycle"],
    "史诗卡牌": ["ui", "content", "lifecycle"],
    "太虚幻境": ["mode", "content", "ui", "lifecycle"],
    "如真似幻": ["ui", "mode", "config", "lifecycle"],
    "手杀MVP": ["ui", "config", "lifecycle"],
    "手杀ui": ["ui", "lifecycle"],
    "手杀全扩": ["config", "content", "lifecycle"],
    "拖拽读取": ["platform", "lifecycle"],
    "斗转星移": ["config", "mode", "lifecycle"],
    "更新保护": ["platform", "lifecycle"],
    "末日乱斗": ["mode", "content", "lifecycle"],
    "标记补充": ["ui", "lifecycle"],
    "武皇争霸": ["mode", "content", "lifecycle"],
    "段位制裁": ["content", "config", "lifecycle"],
    "电脑适配": ["platform", "config", "lifecycle"],
    "皮肤切换": ["ui", "resource", "lifecycle"],
    "祖安武将": ["content", "ui", "lifecycle"],
    "笨蛋插件": ["ui", "lifecycle"],
    "选将美化": ["ui", "mode", "lifecycle"],
    "雷霆万钧": ["content", "ui", "lifecycle"],
    "龙血玄黄": ["content", "ui", "lifecycle"],
}


def slug_name(name: str) -> str:
    return name.replace(" ", "-")


def parse_ledger_rows(text: str) -> dict[str, dict]:
    """从台账表行提取 ID -> 核查/待处理摘要。"""
    rows = {}
    for m in re.finditer(
        r"\| (U\d{2}|L\d{2}) \[([^\]]+)\]\([^)]+\) \| ([^|]+) \| ([^|]+) \| ([^|]+) \|",
        text,
    ):
        sid, name, checked, status, todo = m.group(1), m.group(2), m.group(3), m.group(4), m.group(5)
        rows[sid] = {
            "name": name.strip(),
            "checked": re.sub(r"\s+", " ", checked).strip(),
            "status": re.sub(r"\s+", " ", status).strip()[:400],
            "todo": re.sub(r"\s+", " ", todo).strip()[:500],
        }
    return rows


def behavior_sample_refs(sid: str, name: str) -> list[str]:
    refs = [
        "../operations/evidence/2026-10-07-e01-feature-inventory/README.md（功能面清点基线）",
    ]
    if name == "龙血玄黄" or sid == "L29":
        refs.extend(
            [
                "../operations/evidence/2026-10-08-e01-longxue-lx2/behavior-samples.txt（注册期动态样本）",
                "../operations/evidence/2026-10-08-e01-longxue-lx1/README.md（可读化清单）",
                "../operations/evidence/2026-10-16-t9-lx-battle-trace/（战斗触发轨迹，D5）",
                "../operations/evidence/2026-10-11-t9-lx-trace-survey/README.md（阻塞勘察，已由 D5 解锁）",
            ]
        )
    if name in ("欢乐卡牌",) or sid == "U06":
        refs.append("../operations/evidence/2026-10-08-e03-package-acceptance/README.md")
    if name in ("cardpile",) and sid.startswith("U"):
        refs.append("../operations/evidence/2026-10-08-u03-cardpile-lifecycle/README.md")
    if sid == "L04":
        refs.append("../operations/evidence/2026-10-08-l04-cardpile-liuli-lifecycle/README.md")
    if name == "wuxing" or sid == "L06":
        refs.append("../operations/evidence/2026-10-08-u05-wuxing-lifecycle/README.md")
    if name == "3D精选" or sid == "U01":
        refs.append("../operations/evidence/2026-10-08-u01-d3-lifecycle/README.md")
    refs.append("scripts/legacy-harness/（无头旧线采样入口；运行副本 scratch/legacy-harness）")
    return refs


def verify_entry(sid: str, name: str, ifaces: list[str]) -> list[str]:
    entries = [
        f"G0 本文档：`docs/migration/g0/{sid}-{slug_name(name)}.md`",
        "台账行：`docs/migration/upstream-extension-reconstruction.md` 对应 ID",
        "功能面基线：`docs/operations/evidence/2026-10-07-e01-feature-inventory/feature-inventory.json`",
    ]
    if "content" in ifaces or "lifecycle" in ifaces:
        entries.append("原生包生命周期/内容锁：`crates/content-runtime` + 对应 packages/extensions（若已有转换包）")
    if name == "龙血玄黄":
        entries.append("轨迹采样：`node scripts/legacy-harness/sample-lx-battle.mjs` → `docs/operations/evidence/2026-10-16-t9-lx-battle-trace/`")
    entries.append("无头启动探针：`node scripts/legacy-harness/probe-liuli-boot.mjs <scratch/liuli根>`")
    return entries


def menu_rules_block(feat: dict, ledger: dict | None) -> str:
    entries = feat.get("entries") or []
    config_items = []
    settings = 0
    ext_refs = []
    for e in entries:
        if e.get("config_items"):
            config_items.append(f"{e['entry']}: config_items≈{e['config_items']}")
        settings += e.get("settings_refs") or 0
        ext_refs.extend(e.get("external_extension_refs") or [])
    sub = feat.get("subplugins") or []
    lines = [
        "### 菜单 / 配置分支",
        f"- 入口配置项计数（清点近似）：{'; '.join(config_items) if config_items else '未检出独立 config 段（或壳扩展）'}",
        f"- settings/config 引用次数（清点近似）：{settings}",
        f"- 子插件/子目录：{', '.join(sub) if sub else '无'}",
        "### 规则 / 依赖分支",
        f"- 外部扩展引用：{', '.join(ext_refs) if ext_refs else '无（清点未检出）'}",
    ]
    if ledger and ledger.get("checked"):
        lines.append(f"- 台账已核查摘要：{ledger['checked'][:350]}")
    if ledger and ledger.get("todo"):
        lines.append(f"- 台账待处理（含规则分支）：{ledger['todo'][:350]}")
    lines.append(
        "- 语义说明：上表为 G0 登记面；逐菜单文案与运行时分支以旧线动态样本/可读源为准，迁移时按接口拆分验收。"
    )
    return "\n".join(lines)


def render(sid: str, name: str, repo: str, path: str, feat: dict, summ: dict, ledger: dict | None) -> str:
    ifaces = NAME_INTERFACES.get(name, ["lifecycle", "content"])
    iface_lines = "\n".join(f"- `{k}`：{INTERFACE_MAP[k]}" for k in ifaces if k in INTERFACE_MAP)
    entries = feat.get("entries") or []
    entry_lines = "\n".join(
        f"- `{e.get('entry')}`（约 {e.get('entry_lines')} 行；game.import={e.get('game_import')}；"
        f"skills={e.get('skills')}; cards={e.get('cards')}; characters={e.get('characters')}）"
        for e in entries
    )
    res = feat.get("resources") or {}
    res_s = ", ".join(f"{k}:{v.get('files')}文件/{v.get('bytes')}B" for k, v in res.items()) or "无统计"
    obf = []
    for e in entries:
        m = e.get("obfuscation_markers") or {}
        if m.get("packed_header") or (m.get("eval_calls") or 0) > 0 or (m.get("very_long_lines") or 0) > 0:
            obf.append(
                f"{e.get('entry')}: packed={m.get('packed_header')}, eval={m.get('eval_calls')}, long_lines={m.get('very_long_lines')}"
            )
    samples = "\n".join(f"- {r}" for r in behavior_sample_refs(sid, name))
    verifies = "\n".join(f"- {r}" for r in verify_entry(sid, name, ifaces))
    status = (ledger or {}).get("status") or "见台账"
    return f"""# G0 · {sid} {name}

> 核验日期：2026-10-16 · 来源仓：`{repo}` · 入口：`{path}`  
> 本页为 E01/G0 深度清单（功能 / 菜单与规则分支语义 / 目标接口 / 验证入口 / 旧行为样本引用）。非整包验收证明。

## 1. 功能清单

### 入口与形态
{entry_lines or '- （无 entries）'}

### 清点摘要
- 主入口行数：{summ.get('lines')}
- game.import：{summ.get('game_import')}
- 子插件数：{summ.get('subplugins')}
- 资源：{res_s}
- 混淆/动态特征：{'; '.join(obf) if obf else '无'}
- 台账原生重构状态（摘录）：{status}

### 能力面（G0 登记）
- 生命周期：独立安装/启停/升级/卸载（目标能力，见宿主契约）
- 载荷类型：按下方目标接口拆分；旧 JS 不承诺零改动运行（ADR 0782）

## 2. 菜单 / 规则分支语义

{menu_rules_block(feat, ledger)}

## 3. 目标接口

对照 [原生扩展机制](../../architecture/native-extensions.md)：

{iface_lines}

## 4. 验证入口

{verifies}

## 5. 旧行为动态样本引用

{samples}

## 6. 阻塞与后续

- G0 本文档完成不等于 G1–G5；整包状态仍以[台账](../upstream-extension-reconstruction.md)为准。
- 若需补跑动态样本：在 `scratch/legacy-harness` 运行副本上使用 `scripts/legacy-harness/`，勿写 `archive/upstream`。
"""


def main() -> None:
    inventory = json.loads(INV.read_text(encoding="utf-8"))
    features = { (s["repo"], s["name"]): s for s in json.loads(FEAT.read_text(encoding="utf-8"))["sources"] }
    summaries = { (s["repo"], s["name"]): s for s in json.loads(SUMMARY.read_text(encoding="utf-8")) }
    ledger = parse_ledger_rows(LEDGER.read_text(encoding="utf-8"))

    noname = [e for e in inventory["extensions"] if e["repo"] == "noname"]
    liuli = [e for e in inventory["extensions"] if e["repo"] == "noname-liuli"]
    items = []
    for i, e in enumerate(noname, 1):
        items.append((f"U{i:02d}", e))
    for i, e in enumerate(liuli, 1):
        items.append((f"L{i:02d}", e))

    OUT.mkdir(parents=True, exist_ok=True)
    index_rows = ["| ID | 名称 | 文档 |", "|---|---|---|"]
    for sid, e in items:
        name, repo, path = e["name"], e["repo"], e["path"]
        feat = features.get((repo, name), {"entries": [], "resources": {}, "subplugins": []})
        summ = summaries.get((repo, name), {})
        led = ledger.get(sid)
        body = render(sid, name, repo, path, feat, summ, led)
        fname = f"{sid}-{slug_name(name)}.md"
        (OUT / fname).write_text(body, encoding="utf-8")
        index_rows.append(f"| {sid} | {name} | [{fname}](./{fname}) |")

    readme = f"""# G0 来源清单（37）

核验日期：2026-10-16。每份文档含功能清单、菜单/规则分支语义、目标接口、验证入口与旧行为样本引用。

生成：`python3 scripts/legacy-harness/generate-g0-docs.py`  
基线：`docs/operations/evidence/2026-10-07-e01-feature-inventory/` + 台账摘要。

{chr(10).join(index_rows)}
"""
    (OUT / "README.md").write_text(readme, encoding="utf-8")
    print(f"wrote {len(items)} docs + README -> {OUT}")


if __name__ == "__main__":
    main()
