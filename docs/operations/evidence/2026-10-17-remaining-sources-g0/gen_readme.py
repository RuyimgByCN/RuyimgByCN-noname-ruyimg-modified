"""生成余下来源 G0 清点 README（2026-10-17）。

用法：在 noname-next 目录执行 `python3 docs/operations/evidence/2026-10-17-remaining-sources-g0/gen_readme.py`
数据源：同目录 survey.json（由 G0 实测脚本产出）。
"""

import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
ORDER = ["U08", "L08", "L11", "L14", "L19", "L20", "L25"]

KIND = {
    "U08": ("内容面·可批次（体量小）", "96 KB；`character/skill.js` 顶层键 ~305（`yxs_*` 技能表）、`character/character.js` 43 条武将；无混淆", "直接按武将/技能批次移植（先小技，后大技）"),
    "L19": ("内容面·可批次（技能极小）", "148 KB；66 技能、**中位 479 B**；24 处武将引用；配置键 1（`zuan_lock`）", "技能体量最小 → 单轮可落多技；模式面（乱斗）另计"),
    "L20": ("内容面·可批次（标记面）", "364 KB；75 技能、中位 2.6 KB；12 处武将引用；配置键 3", "标记类技能批量移植（多数为 mark 展示 + 少量规则）"),
    "L25": ("内容面·大体量", "4.9 MB；122 技能、中位 2.1 KB、**最大 946 KB**；120 处武将引用", "先小技批次；巨技（>900 KB）单列评估"),
    "L14": ("渲染/UI 面·含小技能", "1.0 MB；48 技能、中位 1.1 KB；配置键 12+", "UI 面不迁移；48 小技可批次"),
    "L11": ("框架/渲染面·无技能", "3.9 MB；0 技能、0 武将；配置键 1（`copy`）", "框架（侍从/收集/祝福/赛季）属宿主契约 → 需先定契约再验收"),
    "L08": ("框架/渲染面·巨型", "3.4 MB；9 技能（**最大 259 KB**）；配置键 12+；无混淆", "UI 面不迁移；巨技单列评估"),
}


def main() -> None:
    data = json.load(open(os.path.join(HERE, "survey.json"), encoding="utf-8"))
    rows = []
    for sid in ORDER:
        d = data[sid]
        rows.append(
            "| {sid} | {name} | {js} | {mb} | {sk} | {med} | {ch} | {cfg} | {obf} |".format(
                sid=sid,
                name=d["name"],
                js="%.0f KB" % (d["total_js_bytes"] / 1024),
                mb="%.0f KB" % (d["main_bytes"] / 1024),
                sk=d["skills"],
                med=("%.1f KB" % (d["skill_median"] / 1024) if d["skill_median"] else "—"),
                ch=d["characters"],
                cfg=len(d["config_keys"]),
                obf=d["obfuscated_identifiers"],
            )
        )
    kinds = []
    for sid in ORDER:
        kind, why, path = KIND[sid]
        kinds.append("| **%s** | %s | %s | %s |" % (kind, sid, why, path))

    md = "\n".join(
        [
            "# 余下来源 G0 清点（7 个：U08/L08/L11/L14/L19/L20/L25）（2026-10-17）",
            "",
            "- 日期：2026-10-17",
            "- 口径：脚本实测（见 [survey.json](survey.json)）——JS 总量、主文件、`lib.skill` 定义数、技能体量中位/最大、`lib.character` 引用、配置键数、混淆标识符数",
            "- 结论用于**排期**（不改变台账完成条件）",
            "",
            "## 实测表",
            "",
            "| 来源 | 名称 | JS 总量 | 主文件 | 技能数 | 技能中位 | 武将引用 | 配置键 | 混淆标识符 |",
            "|---|---|---|---|---|---|---|---|---|",
            *rows,
            "",
            "## 分类结论",
            "",
            "| 类别 | 来源 | 依据 | 建议路径 |",
            "|---|---|---|---|",
            *kinds,
            "",
            "## 与既有结论的关系",
            "",
            "- 本轮实测把「待重构」的 7 个来源分成 **内容面可批次（5 个：U08/L19/L20/L25/L14）** 与 **框架/渲染面（2 个：L08/L11）**，后者与 L12（已实测无规则面）同类：其完成条件涉及宿主契约（页面/框架提供者），需先定契约口径。",
            "- 本轮**不改**任何台账完成条件、**不标整包**；仅登记实测与建议路径。",
            "",
            "## 复现",
            "",
            "```bash",
            "python3 docs/operations/evidence/2026-10-17-remaining-sources-g0/gen_readme.py   # 依据 survey.json 重生成本表",
            "```",
            "",
        ]
    )
    out = os.path.join(HERE, "README.md")
    open(out, "w", encoding="utf-8").write(md)
    print("written", out, len(md), "bytes")


if __name__ == "__main__":
    main()
