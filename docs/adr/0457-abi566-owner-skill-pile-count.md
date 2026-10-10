# 0457 ABI 566 owner_skill_pile_count + 钟会权计链 quanji/zili（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)
说明：ADR 0456 已被其他 agent 领取,本 ADR 顺延 0457。

## 背景

expansion（技能牌堆）链路第二个积木：`owner_skill_pile_count` modifier 值源,使
`hand_limit`（手牌上限）等 modifier 按本技能牌堆牌数加成。配合 ABI 565 计数表达式,
钟会旧版链 quanji（存权）/zili（觉醒）闭环。

## 变更

### 引擎（ABI 566）

| 文件 | 变更 |
|---|---|
| `engine-core/query/modifier.rs` | `ModifierValueSource::OwnerSkillPileCount` 枚举 + 实现（`skill_pile_cards(owner, skill_def.raw()).len()`） |
| `engine-core/content/registry.rs` | 字符串双向映射 `owner_skill_pile_count` |
| `content-compiler/typecheck.rs` | value_source 白名单 + 校验（add 模式,hand_limit/attack_range） |
| `content-compiler/codegen.rs` + `lib.rs` | ABI 566 提升 + 常量/链（565 曾被其他 agent 重写链时覆盖,已补回） |
| `engine-api/version.rs` | 566 |

### 技能（2 个,yijiang 钟会旧版链）

| 技能 | DSL 表达 | 未建模/近似 |
|---|---|---|
| `quanji`（权计） | `damage owner_is_target optional` → `draw_cards(1)` + `store_selected_in_skill_pile(self)`;`modifier hand_limit add owner_skill_pile_count` | — |
| `zili`（自立） | `phase_prepare` + 觉醒标记 + `if 2 < skill_pile_count(self, 3650058084)` → `change_max_hp(-1)` + `choose_draw_or_recover(self, 2, 1, 0)` | 「获得排异」未建模 |

## 落地

- yijiang 128 技能 / 27 武将 / ABI 566。
- `extensions/yijiang/` 同步（package.npack.json 同源,README 更新）。

## 备注

并发说明：实现期间 lib.rs 的 `required_rule_vm_abi` 链被其他 agent 重写
（564 置顶,565 被移除）,已把 566/565 补回链顶,避免高版本模块被错误标记为 564。

## 验收

- 6 crate cargo check 通过;yijiang 打包 ABI 566;quanji/zili 触发索引正确;
  quanji modifier 入包为 `hand_limit add owner_skill_pile_count`。
