# ADR 0324：ABI 456 天香伤害重定向原语 + OLD 小乔

本批为 OLD 包小乔（`old_xiaoqiao`）的「天香」（`tianxiang`，3182807327，shenhua 包登记）
完整移植，需要一个新的复合原语
`choose_discard_heart_redirect_current_damage(player, draw_skill_def)`（ABI 456）：
在 `damage_before` 触发中弃一张红桃手牌、选择一名其他角色，将当前伤害转移给该角色，
并为其附加按已损失体力摸牌的临时技能。

## 原语设计（ABI 456）

- DSL 表面：`choose_discard_heart_redirect_current_damage(player, draw_skill_def)`，
  仅允许在 `damage_before` 触发中使用（typecheck 显式校验）；编译为
  `effect_tag::RESOLVE_TIANXIANG`（tag 368）。
- 引擎：`EffectRequest::ResolveTianxiang { player, target, discard_card, source,
  amount, nature, card, card_kind, card_flags, draw_skill_def, skill }`。
  - VM 首发（`target == NONE`）由 vm_host 从 `damage_before` 上下文捕获
    source/amount/nature/card/kind/flags（沿用 ABI 374 `ResolveTwluanchou` 先例）。
  - 选择结算走 `ContinuationKind::Custom` + 载荷魔法标记
    `RESOLVE_TIANXIANG_CUSTOM_CHOICE`（0xDCD8_4560）：先选红桃手牌，再选其他角色，
    最后重发应用。
  - 应用阶段：弃置红桃牌；将当前 damage_id 加入 `prevented_damage_ids`，置
    `damage_redirect_in_progress` 守卫，以同源/同量/同属性/同牌 `DealCardDamage`
    重新结算给新目标，随后为其 `AddSkill(draw_skill_def)`。
- 伤害重定向采用「取消 + 重放」而非改写事件目标，与 共患（ABI 374）一致；
  守卫位防止链式重定向（共患/天香互转）。

## 技能移植

「天香」（`tianxiang`，3182807327）：
- `trigger damage_before owner_is_target optional`：无红桃手牌时原语自行空转。
- 「天香·摸牌」（`tianxiang2`，4111110111，新派生技能）：
  - `trigger damage owner_is_target` / `damage_prevented owner_is_target`：
    `draw_cards(self, max_hp(self) - hp(self))` 后 `remove_skill(self_skill)`。

## 文档化近似

- 旧 JS 于 damageBegin3 改写 `trigger.player` 转移伤害，新目标后续阶段仍会重新评估
  护甲/受伤修正；DSL 以已计算的伤害量直接转移，新目标不再重评估（文档化近似，
  常见无护甲场景等价）。
- 旧 JS 的 `tianxiang2` 于 damageAfter/damageCancelled/damageZero 触发，DSL 以
  `damage` / `damage_prevented` 双触发覆盖完成与取消/归零分支。

## 打包与验证

- `tianxiang.skill`（shenhua 包，供 OLD 旧小乔跨包引用）加入
  `content/original/ported/shenhua/`；`old_xiaoqiao`（3863368903，女/吴/3 体力，
  技能 [tianxiang 3182807327, hongyan 3138402391]）加入 old `characters.json`；
  catalog 标记 `tianxiang` 为 `ported` 并新增 `tianxiang2`。
- shenhua npack 重建（ABI 456，含 tianxiang/tianxiang2），old npack 重建（新增
  小乔选将池）；两包 README 构建命令逐字节复现（`cmp` 通过），部署至
  `extensions/shenhua/` 与 `extensions/old/`。
- 引擎测试全绿：engine-api 4、rule-vm 10、engine-core 54；26 个扩展包全新 store
  导入成功。
- 旧 JavaScript 不进入 npack 或运行时。
