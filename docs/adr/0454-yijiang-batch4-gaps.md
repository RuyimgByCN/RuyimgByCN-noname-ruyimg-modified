# 0454 yijiang 批次四 + 四包缺口登记（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

本批新增 1 个技能（`oldrenxin` 旧仁心,曹冲）,纯 DSL 表达,零新 ABI。

## 变更

| 技能 | DSL 表达 | 未建模/近似 |
|---|---|---|
| `oldrenxin`（旧仁心） | `trigger dying owner_alive_other_dying_hand_nonempty_once_per_game optional` → `turn_over(self)` + `give_all_hand(self, event_player)` + `recover_hp(event_player, 1)` | 「每局限一次」以 once_per_game 过滤器近似（旧 JS 无明确次数限制） |

`give_all_hand` 效果的首个用例;与 turn_over/recover 组合构成濒死救援链路。

## 缺口登记

对 yijiang / newjiang / clan / key 四包剩余缺失技能完成第二轮类型学评估,合并登记
到 ABI-COORDINATION.md「E 段原语缺口登记」:expansion 技能牌堆链、家族查询与横置、
技能继承、衍生卡 createCard2、改判、观星 chooseToGuanxing、距离方向标记等 8 类,
为后续 E 段（565 起）原语领取提供完整地图。

## 落地

- yijiang 124 技能 / 27 武将 / ABI 562。
- `extensions/yijiang/` 同步（package.npack.json 同源,README 更新）。

## 验收

- content-cli pack 编译通过;oldrenxin 触发索引正确（dying, optional）。
