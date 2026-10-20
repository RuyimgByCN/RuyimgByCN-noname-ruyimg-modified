# 37 来源 ↔ 41 包**数据驱动映射**（2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- **映射依据**：包内资产 JSON 的 **`upstream_path`**（38/41 包有）+ `package_id`/`namespace`；**不靠猜**

- 台账 §1 来源数：**37**（U01–U08 + L01–L29）
- 包目录数：**41**
- **已映射来源：32/37**；**未映射 5**；**不属于 37 来源的包：1**

## 1 已映射（36 个包 ↔ 32 个来源）

| 来源 | 名称 | 包目录 | `upstream_path`（首个） |
|---|---|---|---|
| **U01** | 3D精选 | `d3-upstream-noname` | `apps/core/extension/3D精选/` |
| **U02** | boss | `boss-play-upstream-noname` | `apps/core/extension/boss/extension.js` |
| **U03** | cardpile | `cardpile-upstream-noname` | `apps/core/extension/cardpile/extension.js` |
| **U04** | coin | `coin-upstream-noname` | `**（无）**` |
| **U05** | 杀海拾遗 | `shahai-gujian-upstream-noname` | `apps/core/extension/杀海拾遗/card/gujian/` |
| **U05** | 杀海拾遗 | `shahai-mtg-upstream-noname` | `extension/杀海拾遗/card/mtg/` |
| **U05** | 杀海拾遗 | `shahai-swd-upstream-noname` | `apps/core/extension/杀海拾遗/card/swd/` |
| **U05** | 杀海拾遗 | `shahai-hearth-upstream-noname` | `apps/core/extension/杀海拾遗/card/hearth/` |
| **U05** | 杀海拾遗 | `shahai-yunchou-upstream-noname` | `apps/core/extension/杀海拾遗/card/yunchou/` |
| **U05** | 杀海拾遗 | `shahai-zhenfa-upstream-noname` | `extension/杀海拾遗/card/zhenfa/` |
| **U05** | 杀海拾遗 | `shahai-gwent-upstream-noname` | `apps/core/extension/杀海拾遗/card/gwent/` |
| **U05** | 杀海拾遗 | `shaai-ow-upstream-noname` | `extension/杀海拾遗/character/ow/` |
| **U05** | 杀海拾遗 | `shaai-xianjian-upstream-noname` | `extension/杀海拾遗/character/xianjian/` |
| **U06** | 欢乐卡牌 | `huanle-upstream-noname` | `apps/core/extension/欢乐卡牌/` |
| **U07** | 玩点论杀 | `wudian-upstream-noname` | `apps/core/extension/玩点论杀/` |
| **U08** | 英雄杀 | `hero-kill-upstream-noname` | `apps/core/extension/英雄杀/character/skill.js` |
| **L01** | AI优化 | `ai-tuning-upstream-liuli` | `noname/extension/AI优化/extension.js` |
| **L02** | SJ Settings | `sj-settings-upstream-liuli` | `noname/extension/SJ Settings/extension.js` |
| **L03** | boss | `boss-play-upstream-liuli` | `noname/extension/boss/extension.js` |
| **L04** | cardpile | `cardpile-upstream-liuli` | `noname/extension/cardpile/extension.js` |
| **L05** | coin | `coin-upstream-liuli` | `noname/extension/coin/extension.js` |
| **L06** | wuxing | `wuxing-upstream-liuli` | `noname/extension/wuxing/extension.js` |
| **L07** | 全能搜索 | `omnisearch-upstream-liuli` | `**（无）**` |
| **L09** | 千幻聆音 | `qhly-upstream-liuli` | `noname/extension/千幻聆音/extension.js` |
| **L10** | 史诗卡牌 | `epic-cards-upstream-liuli` | `noname/extension/史诗卡牌/extension.js` |
| **L13** | 手杀MVP | `shousha-mvp-upstream-liuli` | `noname/extension/手杀MVP/extension.js` |
| **L15** | 手杀全扩 | `shousha-full-upstream-liuli` | `noname/extension/手杀全扩/extension.js` |
| **L16** | 拖拽读取 | `drag-read-upstream-liuli` | `noname/extension/拖拽读取/extension.js` |
| **L17** | 斗转星移 | `douzhuan-xingyi-upstream-liuli` | `noname/extension/斗转星移/extension.js` |
| **L18** | 更新保护 | `update-guard-upstream-liuli` | `noname/extension/更新保护/extension.js` |
| **L19** | 末日乱斗 | `doomsday-brawl-upstream-liuli` | `noname/extension/末日乱斗/extension.js` |
| **L20** | 标记补充 | `mark-supplement-upstream-liuli` | `noname/extension/标记补充/extension.js` |
| **L21** | 武皇争霸 | `martial-upstream-liuli` | `noname/extension/武皇争霸/extension.js` |
| **L22** | 段位制裁 | `elo-sanction-upstream-liuli` | `noname/extension/段位制裁/extension.js` |
| **L23** | 电脑适配 | `desktop-adapt-upstream-liuli` | `**（无）**` |
| **L24** | 皮肤切换 | `skin-switch-upstream-liuli` | `noname/extension/皮肤切换/extension.js` |
| **L26** | 笨蛋插件 | `fool-plugin-upstream-liuli` | `noname/extension/笨蛋插件/extension.js` |
| **L27** | 选将美化 | `select-beautify-upstream-liuli` | `noname/extension/选将美化/extension.js` |
| **L28** | 雷霆万钧 | `thunder-upstream-liuli` | `noname/extension/雷霆万钧/extension.js` |
| **L29** | 龙血玄黄 | `longxue-upstream-liuli` | `noname/extension/龙血玄黄/extension.js` |

## 2 **未映射**的来源（5 个）

| 来源 | 名称 | 结论 |
|---|---|---|
| **L08** | 十周年UI | **无对应包目录**（本工作区未转换）|
| **L11** | 太虚幻境 | **无对应包目录**（本工作区未转换）|
| **L12** | 如真似幻 | **无对应包目录**（本工作区未转换）|
| **L14** | 手杀ui | **无对应包目录**（本工作区未转换）|
| **L25** | 祖安武将 | **无对应包目录**（本工作区未转换）|

## 3 **不属于 37 来源**的包（1 个）

| 包 | `upstream_path` | 说明 |
|---|---|---|
| `boss-zhugeliang-upstream-liuli` | `noname/character/extra.js + noname/mode/boss.js` | T11 Boss 内容（台账 §2 已登记） |

## 4 判读（**如实**）

1. **32/37** 来源已有**至少一个**转换包（U05 杀海拾遗 占 9 个包：7 个卡牌族 + `ow`/`xianjian` 武将面）；
2. **5 个来源未转换**（十周年UI、太虚幻境、如真似幻、手杀ui、祖安武将）—— 这是**范围缺口**，不是缺陷；
3. **1 个包不属于 37 来源**（`boss-zhugeliang`，T11 Boss 内容）—— 与台账 §2 一致；
4. **`upstream_path` 缺失的 3 个包**（`coin-upstream-noname`、`desktop-adapt`、`omnisearch`）用 `package_id`+`namespace` 判定，
   **建议**补齐该字段（数据规范待办）。

## 5 复现

```bash
cd noname-next
python3 - <<'PY'
import json,glob,os
for d in sorted(glob.glob("packages/extensions/*/")):
    for f in glob.glob(os.path.join(d,"assets","*.json")):
        j=json.load(open(f,encoding="utf-8"))
        if isinstance(j,dict) and "upstream_path" in j: print(d, j["upstream_path"]); break
PY
```
