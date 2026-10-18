# 0446 clan 批次一（Hermes E 段）

> 2026-10-18 标题勘正（NC11）：原标题写作 `0444`，与文件名 `0446` 及已占用编号
> `0444-shiji-sp-cuiyan-yajun-zundi.md` 冲突，按文件名更正为 0446；正文与决策未改。
> 检查脚本：`python3 -B scripts/check-adr-index.py`。

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

clan 族包共 37 武将、约 110 个上游技能，此前仅 4 个技能（clanhuanyin / clanqingli /
chenliuwushi / clanbalong）。本批对全部技能做了可表达性评估，纯 DSL 可保真表达的技能
极少——族包普遍依赖家族（hasClan）查询、横置/重置、转换技状态、gaintag、牌名字数等
引擎能力，这些均不在当前 DSL 词汇表内。

## 变更

| 技能 | 武将 | DSL 表达 | 未建模/近似 |
|---|---|---|---|
| `clandandao`（弹道） | 荀爽 | `trigger judgment_resolved owner_alive` + `if mark_count(event_player, 2693545350) == 0` → `change_max_hp(self, 1)` + `add_mark(event_player, …)` | 无（「每名角色整局首次判定」以判定者标记等价近似） |

## 原语缺口登记（后续 E 段 ABI 候选）

为推进族包其余技能，已登记 6 类缺口（详见 `extensions/clan/README.md`）：家族查询、
横置指定角色/自身、refreshSkill 重置、牌名字数、转换技状态读写、手牌点数和、gaintag。
后续按 ABI-COORDINATION.md 规则在 E 段（559 起）逐项领取后再实现。

## 落地

- 新增 `clandandao.skill`；重打包 `clan.npack.json`（5 技能 / ABI 336）。
- `extensions/clan/` 同步三件套（package.npack.json 与 clan.npack.json 同源），补 README。
- 武将暂不登记（族包武将技能未齐全）。

## 验收

- `content-cli pack` 全量编译通过；`trigger_index` 含 `judgment_resolved + owner_alive` forced 触发。
- 零新 ABI（E 段 559 起未占用）。
