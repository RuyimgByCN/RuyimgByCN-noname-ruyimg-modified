# 0443 newjiang 批次一（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

newjiang 包此前仅 10 个技能（kangli/yizu/mpbishi/lkzhongzhuang/dcyiju/biancai/spqishe/
weifeng2 等）。本批新增 6 个技能，全部纯 DSL 表达，零新 ABI（E 段 559 起未占用）。

## 变更

| 技能 | 武将 | DSL 表达 | 未建模/近似 |
|---|---|---|---|
| `duanwan`（断腕） | 吴安国 | `trigger dying owner_is_target` + `mark_count` 限定标记 + `recover_hp(self, 2 - hp(self))` | 转换技切换与觉醒分支；限定技以每局一次标记近似 |
| `beiyu`（备预） | 司马孚 | `active play_phase uses_per_turn 1` + `draw_cards(self, max_hp(self) - hand_count(self))` | 「将一种花色手牌置于牌堆底」分支 |
| `dunyong`（遁勇）+ `dunyong_effect` | 藤甲 | `damage_before owner_source_other_damage_before` → `deal_damage(self, self, event_amount)`；`dying owner_alive` → `draw_cards(self, max_hp(event_player))` + `add_skill_until_turn_end(3140406282)`；辅助技能含 `card_use_unlimited`/`card_distance_unlimited` 修正器 | 镜像伤害来源以自身近似（旧 JS 无来源）；「体力不为全场最少/最多」豁免 |
| `mpjiusong`（酒颂） | 刘玲 | `view_as jiu any card_trick from hand_expansion` | 「醉」标记计数（每回合至多三次）与酕醄联动 |
| `jingyi`（精艺） | 马钧 | `trigger card_used owner_alive_equip_used` → `draw_cards(self, 5 - empty_equip_slots(self))` + `discard_hand_equip_count(self, 2)` | 「每类副类别每回合一次」限制；viewAs 子牌计数以装备数近似 |

## 落地

- 新增 6 个 .skill 文件（`content/original/ported/newjiang/`）。
- 重打包 `newjiang.npack.json`：16 技能 / ABI 460（jingyi 引入 discard_hand_equip_count）。
- `extensions/newjiang/` 同步（package.npack.json 与 newjiang.npack.json 同源，README 更新）。
- 武将暂不登记（断腕等技能完整但所属武将技能未齐全，遵循「暂不发布整名武将」惯例）。

## 验收

- `cargo run -q -p content-cli -- pack` 全量编译通过（16 技能）。
- 触发器/主动技/视为技/修正器索引与预期一致（见打包产物 trigger_index 等）。
