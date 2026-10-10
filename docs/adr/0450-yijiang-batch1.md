# 0450 yijiang 批次一（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

yijiang 此前已定义 118 个技能（前序批次）。本批新增 3 个,全部纯 DSL 表达,零新 ABI
（E 段 562 起未占用;559-561 已由其他 agent 占用）。

## 变更

| 技能 | 武将 | DSL 表达 | 未建模/近似 |
|---|---|---|---|
| `renxin`（仁心） | 曹冲 | `damage_before owner_alive optional` + `if hp(event_player) == 1` → `choose_discard_equipment_prevent_current_damage(self)`（复用 ABI 398 骄矜管线）+ `turn_over(self)` | 旧 JS 排除「受伤者为拥有者自身」场景 |
| `zhiyu`（智语） | 虞翻 | `damage owner_is_target optional` → `draw_cards(self, 1)` + `reveal_hand(self)` | 「手牌均为同色→伤害来源弃牌」分支（无按颜色判定+来源弃牌组合原语） |
| `danxin`（丹心） | — | `damage owner_is_target optional` → `draw_cards(self, 1)` | 「更改矫诏描述」分支以固定摸一张近似 |

## 说明

评估中发现 `increase_current_card_base_damage` 等伤害修改效果仅可在 card_used 挂起结算
窗口（Sha/Juedou/Nanman/Wanjian）调用,damage_before 时点调用会 ContentFault——因此
lianhuo（yijiang）/ dckanghui（newjiang）等「伤害+1」技能不适用该效果,已列入 E 段
原语缺口（通用当前伤害修改）。

## 落地

- 新增 3 个 .skill 文件；重打包 `yijiang.npack.json`：121 技能 / 27 武将 / ABI 529。
- `extensions/yijiang/` 同步（package.npack.json 与 yijiang.npack.json 同源,README 更新）。
- 武将暂不登记（技能未齐全）。

## 验收

- `content-cli pack` 全量编译通过；trigger_index 含 renxin/zhiyu/danxin 可选触发。
- 零新 ABI。
