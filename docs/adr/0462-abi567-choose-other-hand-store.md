# 0462 ABI 567 choose_other_hand_store_skill_pile + onlyOL 储元（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

expansion（技能牌堆）链路第三积木：「选目标一张手牌置入拥有者技能牌堆」。解锁
onlyOL 曹丕链的存牌侧 chuyuan（储元）。配合 565 计数表达式,曹丕链的存牌/计数已齐,
dengji（登极）觉醒的「取回」为下一积木。

## 变更

### 引擎（ABI 567）

| 文件 | 变更 |
|---|---|
| `rule-api/effect.rs` | `EffectRequest::ChooseOtherHandStoreSkillPile { player, target, skill }` |
| `content-schema/ir.rs` | `EffectChooseOtherHandStoreSkillPile { player, target, span }` |
| `content-compiler/typecheck.rs` | 注册 `choose_other_hand_store_skill_pile(player, target)`（修复误删 take_one_event_target_hand 返回语句） |
| `content-compiler/codegen.rs` + `lib.rs` | codegen + ABI 567 常量/链/import |
| `rule-vm/bytecode.rs` + `vm.rs` | effect_tag 529 + 解码 |
| `engine-core/primitives.rs` | 实现：open_choice(PickTargetCard, mode=20, payload[target, player, 20, skill]) |
| `engine-core/resolve_choice.rs` | PickTargetCard 回调 mode 20：校验手牌 → 移入 actor 技能牌堆（`skill_pile_def` + `ensure_skill_pile_zone` + MoveCards） |
| `engine-core/hash.rs` | hash 分支（修复误改 TakeOneHandCard 既有 tag 267） |
| `engine-api/version.rs` | 567 |

复用现有 PickTargetCard 续体（mode 0/1/2/3/10..14 之后新增 20）,无需新 ContinuationKind
（repr(u8) 枚举已满,255=Custom）。

### 技能（1 个,onlyOL 曹丕链存牌侧）

| 技能 | DSL 表达 | 未建模/近似 |
|---|---|---|
| `chuyuan`（储元） | `damage owner_alive optional` + `if skill_pile_count(self, id) < max_hp(self)` → `draw_cards(event_player, 1)` + `choose_other_hand_store_skill_pile(self, event_player)` | 「选牌者为受伤者本人」以拥有者选择近似;自伤时效果内 player==target 校验为空操作 |

## 落地

- onlyOL 24 技能 / ABI 567。
- `extensions/onlyOL/` 同步（package.npack.json 同源,README 更新）。

## 备注

实现期间多次遭遇其他 agent 并发编辑（ResolvePotcaoshuang/Potjianzhuan 未覆盖、
rule_api 增量目录被删）,等待后 6 crate cargo check 全绿。

## 验收

- 6 crate cargo check 通过;onlyOL 打包 ABI 567;chuyuan 触发索引正确。
