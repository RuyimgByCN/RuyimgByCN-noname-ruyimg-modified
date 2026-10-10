# ADR 0612：gwent 呼朋（hupeng）运行时 + ABI 644「择一加技能」原语

> 承接 [ADR 0608](0608-gwent-dudayuanshuai-abi643-cancel-use.md)「后续 1」：为 gw_zhuoertan
> 的主技能 hupeng（呼朋）补全运行时——它把杜达元帅置入目标手牌，是激活
> gw_dudayuanshuai1/2 两张特殊卡运行时缺的一环。核心交付为「择一加技能」原语
> （ABI 644，完整 VM 驱动接线 + 复用 `ContinuationKind::Custom` 魔数分发）。

## 背景

ADR 0608 完成 gwent 4 张特殊卡运行时（gw_wuyao→杀、gw_lang→酒 view_as；gw_dudayuanshuai1
取消使用牌 ABI 643；gw_dudayuanshuai2 阶段末弃置），但持有杜达元帅卡片的入口
hupeng（呼朋）仍为空元数据占位（`skill hupeng(3607688462) {}`），杜达元帅无法进入游戏，
运行时处于休眠态。本 ADR 补全 hupeng。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

hupeng（gw_zhuoertan 卓尔坦）：`enable: "phaseUse"`、`usable: 1`、`filterCard: true`——
出牌阶段限一次，弃置一张手牌或装备牌（filterCard 消耗），`chooseVCardButton` 择一
杜达元帅1/2 实体卡置入目标手牌。AI 按 `get.attitude` 正负选卡（友给 1 小伙伴、敌给 2
捣蛋鬼）。

## 做了什么

### ABI 644：`ChooseOtherGainOneOfTwoSkills` 原语（择一加技能）

**效果**：玩家在两个技能定义间二选一，将所选技能置入目标（本引擎以技能近似杜达元帅
卡片——gw_dudayuanshuai1_skill=3583584906 / gw_dudayuanshuai2_skill=3832623127 即卡之运行时）。

**接线**（完整 VM 驱动）：
| 文件 | 内容 |
|---|---|
| rule-api/effect.rs | `ChooseOtherGainOneOfTwoSkills { player, target, first_def, second_def, skill }` 变体 |
| rule-vm/bytecode.rs | `effect_tag::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS: u16 = 642` |
| rule-vm/vm.rs | pop 臂（player/target/first_def/second_def/skill）|
| engine-core/hash.rs | ABI 644 tag 编码 |
| engine-core/primitives/mod.rs | 调度臂 → `choose_other_gain_one_of_two_skills` |
| engine-core/primitives/pack_sp_2.rs | 原语实现 + 魔数 `CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE` |
| engine-core/effect/mod.rs | 魔数 re-export（`crate::effect::`）|
| engine-core/command/resolve_choice/mod.rs | `ContinuationKind::Custom` 分发臂 |
| engine-core/command/resolve_choice/group_pre_2.rs | 解析器 `resolve_choose_other_gain_one_of_two_skills`（校验 → 依选项 1/2 发 `AddSkill`）|
| content-schema/ir.rs | `StmtIr::EffectChooseOtherGainOneOfTwoSkills { player, target, first_def, second_def }` |
| content-compiler/typecheck.rs | `"choose_other_gain_one_of_two_skills(player, target, first_def, second_def)"` 臂（两个 def 非零且互异）|
| content-compiler/codegen.rs | lowering（player/target + ConstU32×2 + HostEffect）+ ABI 644 |
| content-compiler/lib.rs | `ADDITIVE_RULE_VM_ABI_VERSION_644` + 判断链 |

**关键设计**：choice 续体复用既有的 `ContinuationKind::Custom`（tag 255）+ payload[0] 魔数
分发（兵势 potweiyan、TW 系等先例）——**不新增 ContinuationKind 变体**。该枚举 256 个
判别式已占满 0..255，新增变体必然破坏回放序数稳定；魔数分发是既定免破坏路径。

**ABI**：`ABI-MAX: 644` + `RULE_VM_ABI_VERSION: 644`，`abort_bump_without_claim` 测试过。

### hupeng DSL（shahai_impl.skill）

```
// 呼朋：出牌阶段限一次，你可以弃置一张牌并将一张杜达元帅置入一名角色的手牌。
skill hupeng(3607688462) {
  active play_phase uses_per_turn 1 targets 1 1 other select_he 1 1 {
    effect discard_selected(self);
    effect choose_other_gain_one_of_two_skills(self, event_source, 3583584906, 3832623127);
  }
}
```

shahai_meta.skill 中原空占位 `skill hupeng(3607688462) {}` 已注释（避免 def 重复）；
characters.json 中 gw_zhuoertan 对 3607688462 的引用不变，天然联动。

**近似说明**（已写入 DSL 注释）：杜达元帅为特殊卡，引擎以技能近似；「择一杜达元帅1/2
实体卡置入目标手牌」以「择一技能置入目标」表达，等价上游 chooseVCardButton。hupeng2/3/4
机制已在 gw_dudayuanshuai1/2_skill 内落地/登记（ABI 643/ADR 0608）。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（+shahai_hupeng_abi644：编译 shahai_impl + 断言 hupeng def 在 skill_index + 断言 rule_vm_abi_version ≥ 644）✅ |
| `abort_bump_without_claim` | **过**（ABI-MAX 644 = RULE_VM_ABI_VERSION 644）✅ |
| 失败集合 | **247 = 247**，双向差异仅 2 项均为他人域：`original_standard_remaining` 出集（并行代理修复标准包金标）、`standard_npack` 入集（content-runtime 标准包资产安装域，本 ADR 未触碰 content-runtime 代码与标准包文件）✅ |
| `cargo check` | 无编译错误；本次改行 **0 警告** ✅ |

**ABI 644 全链**：DSL `effect choose_other_gain_one_of_two_skills(self, event_source, 3583584906, 3832623127)`
→ codegen（player/target + ConstU32×2 + HostEffect）→ VM pop → `EffectRequest::ChooseOtherGainOneOfTwoSkills`
→ hash（644 tag）→ 原语（校验 + open_choice Custom[1,2]）→ Custom 魔数分发 → 解析器
（AddSkill 所选 def）。端到端可编译可执行。

## 后续

1. **gw_dudayuanshuai2 per-card 禁弃置**：需按自定义卡 def 的 card 门禁选择器（ADR 0608 已登记）。
2. **vanish 机制**（回合末消失）：需卡牌 vanish 引擎支持（ADR 0607 已登记）。
3. **国战 12 锦囊结算**：ABI 645+ 待办（复用本模式）。
