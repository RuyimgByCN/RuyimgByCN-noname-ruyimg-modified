# 0453 key 批次一（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

key 包（Kanon/Air/CLANNAD 等动漫角色）此前已移植 61 个技能,缺 200。本批对全部
缺失技能做了特征扫描 + 抽样精读,多数为继承（inherit）、衍生卡、觉醒、标记系统、
棋子机制等复合状态机。本批新增 2 个纯 DSL 技能,零新 ABI。

## 变更

| 技能 | 武将 | DSL 表达 | 未建模/近似 |
|---|---|---|---|
| `yuzuru_bujin`（武刃） | 由弦 | `trigger phase_draw owner_alive` + `if relation(event_player, self) == 1` → `increase_current_draw_count(event_player, 1)` | 「友方」以 relation 盟友判定近似 isFriendOf |
| `tomoya_shangxian`（上弦） | 朋也 | `trigger phase_play owner_is_target` → `draw_cards(self, 1)` | 「距离计算方向切换」标记未建模,固定摸一张 |

## 落地

- key 63 技能 / 10 武将 / ABI 233。
- `extensions/key/` 同步（package.npack.json 同源,README 更新）。
- 武将暂不登记（技能未齐全）。

## 备注

key 剩余缺失技能（haruko_haofang 延时当无中、kanade_mapo 魔炮衍生卡、kotori_yumo
势力标记系统等）依赖衍生卡/标记体系,已列入 E 段缺口登记范围,后续批次处理。

## 验收

- content-cli pack 编译通过;yuzuru_bujin / tomoya_shangxian 触发索引正确。
