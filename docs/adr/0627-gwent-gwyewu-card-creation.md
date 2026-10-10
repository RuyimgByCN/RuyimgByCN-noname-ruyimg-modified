# ADR 0627：gwent 叶舞（gwyewu）造卡入口——随机弃牌链原语（ABI 645）

> 承接 [ADR 0626](0626-gwent-runtime-vm-pop-fixes.md)：gwent（昆特牌）卡运行时收尾。
> 实现叶舞（gwyewu）——「随机敌人随机弃牌、颜色相同重复、每弃敌方手牌/装备牌获得
> 雾妖/狼」的造卡入口。新宿主效果 `gwyewu_random_discard_chain`（ABI 645，
> `RULE_VM_ABI_VERSION` 644 → 645）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

gwent/skill.js:1062 `gwyewu`：出牌阶段限一次，弃置一张手牌，并弃置一名随机敌人的一张
随机牌；若目标弃置的牌与费用牌颜色相同，则重复发动；每以此法弃置敌方手牌获得一张
【雾妖】（gw_wuyao），每弃敌方装备牌获得一张【狼】（gw_lang）。雾妖/狼可当杀/酒，
回合结束从手牌消失（vanish，ADR 0613 已以技能 + phase_end 移除近似）。

## 做了什么

### 宿主效果（ABI 645，8 件套）

- `rule-api/effect.rs`：尾件 `ResolveGwyewuChain { player, skill, cost_cards,
  wuyao_def, lang_def }`。
- `rule-vm/bytecode.rs`：`GWYEWU_RANDOM_DISCARD_CHAIN = 643`；`vm.rs` 弹栈臂——
  弹 `[player, wuyao_def, lang_def]`（def 按位保留 `as u32`），技能 id 取
  `host.context().skill_id`（ADR 0626 约定），费用牌取 `host.selected_cards()`。
- `hash.rs`：ABI 645 臂（`put_u8(h,0)` + `645u16` + 字段）。
- `effect/primitives/pack_sp_2.rs`：`resolve_gwyewu_chain`——
  ①弃费用手牌并记颜色（`effective_card_color_mask`：0b01 红 / 0b10 黑）；
  ②种子 RNG 循环（`RuleRng`，ADR 0617 既有模式）：随机敌方（非同势力且有 he 牌）→
  随机 he 牌 → 模拟弃牌（从候选排除，保证终止）→ 颜色相同则重复，否则止；
  ③按持有者分组弃牌 + `add_skill(雾妖)`×nh + `add_skill(狼)`×ne（雾妖/狼以技能
  近似，造卡入口；重复获得同一技能去重——与多张雾妖卡叠放存在数量近似）。
- `content-schema/ir.rs` / `typecheck.rs` / `codegen.rs`：
  `gwyewu_random_discard_chain(player, wuyao_def, lang_def)` 语句 + 降低 +
  `ADDITIVE_RULE_VM_ABI_VERSION_645`；`content-compiler/lib.rs` 常量 + 链顶；
  `engine-api/version.rs` → 645。

### 内容

`shahai_impl.skill` 新增 `skill gwyewu(2444009580)`（crc32("gwyewu_skill")，
与 gw_wuyao/gw_lang/gw_dudayuanshuai*_skill 同约定）：
```
active play_phase uses_per_turn 1 select_hand 1 1 {
  effect gwyewu_random_discard_chain(self, 784509644, 2836710318);
}
```

### 单测

- `crates/devtools/tests/gwent_runtime.rs`（VM 级）：gwyewu 活动模块 → 正确发出
  `ResolveGwyewuChain{ player, skill, cost_cards, wuyao_def=784509644,
  lang_def=2836710318 }`（与 hupeng / dudayuanshuai1 同守卫）。
- `crates/engine-core/tests/gwent_gwyewu.rs`（引擎级，2 项，全部通过）：
  1. 颜色不同 → 弃费用 + 敌方唯一手牌，获得 1 雾妖（无狼）；
  2. 颜色相同 → 循环至敌方两张同色手牌耗尽，获得雾妖（确定性：候选/牌长 =1 或
     两牌同色迫使 RNG 选点与结果无关）。

## 验证

| 检查 | 结果 |
|---|---|
| gwent_runtime / gwent_gwyewu | 1 + 2 passed ✅ |
| conformance | 全量重跑中（shahai 向量检查 `abi >= 643/644`，645 满足） |
| 失败集合 | 与 claude-a-failures-gw.txt（空）comm 对比，须零新增 |

**ABI 说明**：`RULE_VM_ABI_VERSION` 644 → 645；尾件 EffectRequest + 新宿主效果
tag 643，序数稳定。

## 后续

1. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
2. **gwyewu 角色挂载**：gwyewu 技能已入 shahai_impl 清单（造卡入口就绪）；上游持
   有该技能的角色随杀海拾遗角色迁移时引用 def 2444009580 即可。
