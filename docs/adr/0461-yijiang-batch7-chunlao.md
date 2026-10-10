# 0461 yijiang 批次七（醇醪,Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

本批新增 1 个技能（`chunlao` 醇醪,程普）,纯 DSL 表达,复用既有技能牌堆管线
（store_selected_in_skill_pile + choose_spend_skill_pile_for_dying_jiu）,零新 ABI。

## 变更

| 技能 | DSL 表达 | 未建模/近似 |
|---|---|---|
| `chunlao`（醇醪） | `phase_end owner_alive optional` → `store_selected_in_skill_pile(self)`;`dying owner_alive_dying_has_skill_pile_cards optional` → `choose_spend_skill_pile_for_dying_jiu(self, event_player)` | 「仅限【杀】/无牌堆时」条件未建模（存牌由玩家自控,不限杀） |

濒死救援复用引擎既有 choose_spend_skill_pile_for_dying_jiu 管线
（弃技能牌堆牌当【酒】救濒死者）,过滤器
owner_alive_dying_has_skill_pile_cards 语义已核对（拥有者牌堆非空,非濒死者）。

## 落地

- yijiang 128 技能 / 27 武将 / ABI 566。
- `extensions/yijiang/` 同步（package.npack.json 同源,README 更新）。

## 验收

- content-cli pack 编译通过;chunlao 两个 trigger 索引正确
  （phase_end + dying,均 optional）。
