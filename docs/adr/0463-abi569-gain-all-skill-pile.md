# 0463 ABI 569 gain_all_skill_pile_cards + onlyOL 登极（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

expansion（技能牌堆）链路第四积木（最后一个通用积木）：「获得本技能牌堆全部牌」。
解锁 onlyOL 曹丕链的觉醒取回 dengji（登极）。配合 565 计数、566 modifier 源、
567 选目标牌存、现成濒死用牌管线,expansion 链全部闭环：钟会双链、醇醪、曹丕链。

## 变更

### 引擎（ABI 569）

| 文件 | 变更 |
|---|---|
| `rule-api/effect.rs` | `EffectRequest::GainAllSkillPileCards { player, skill }`（枚举尾追加） |
| `content-schema/ir.rs` | `EffectGainAllSkillPileCards { player, span }` |
| `content-compiler/typecheck.rs` | 注册 `gain_all_skill_pile_cards(player)` |
| `content-compiler/codegen.rs` + `lib.rs` | codegen + 569 常量/链/import（修复并发重复 import/常量） |
| `rule-vm/bytecode.rs` + `vm.rs` | effect_tag 530 + 解码 |
| `engine-core/primitives.rs` | `gain_all_skill_pile_cards`：skill_pile_cards → ensure_skill_pile_zone → move_cards 到手牌;match 分支（修复并发重复分支） |
| `engine-core/hash.rs` | hash 分支（569u16） |
| `engine-api/version.rs` | 569 |

### 技能（1 个,onlyOL 曹丕链觉醒侧）

| 技能 | DSL 表达 | 未建模/近似 |
|---|---|---|
| `dengji`（登极） | `phase_prepare owner_is_target` + 觉醒标记 + `if 2 < skill_pile_count(self, 3324653503)` → `change_max_hp(self,-1)` + `gain_all_skill_pile_cards(self)` + 标记 | 「获天行/界奸雄」未建模（目标技能未移植） |

## 落地

- onlyOL 25 技能 / ABI 569;曹丕链 chuyuan+dengji 闭环。
- `extensions/onlyOL/` 同步（package.npack.json 同源,README 更新）。

## 备注

实现期间并发冲突严重（其他 agent 持续编辑 primitives.rs/codegen.rs/lib.rs）：
重复 match 分支/import/常量多次出现,逐一修复;Codex 的 568 领取行保留,链 569→568→567 有序。

## 验收

- 6 crate cargo check 通过;onlyOL 打包 ABI 569;dengji 触发索引正确。
