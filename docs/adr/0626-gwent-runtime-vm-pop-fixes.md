# ADR 0626：gwent 运行时修复——hupeng/cancel_incoming_card_use 的 VM 弹栈与 def 读取

> 承接 [ADR 0625](0625-guozhan-zhibi-metadata-only.md)：gwent（昆特牌）卡运行时收尾。
> 修复 ABI 643 `cancel_incoming_card_use` 与 ABI 644 `choose_other_gain_one_of_two_skills`
> 两个宿主效果在**实际执行**时的两处缺陷（此前仅通过编译期 conformance 向量，从未被
> 真正执行——`shahai_impl_repro`/`gwent_dudayuanshuai_abi643`/`shahai_hupeng_abi644`
> 均只检查 skill_index 与 ABI 版本）。

## 缺陷一：宿主效果多弹一次栈（潜在地下溢）

`rule-vm/vm.rs` 的 `CANCEL_INCOMING_CARD_USE`（弹 4）与
`CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS`（弹 5）都把**当前技能 id** 当作栈顶值弹出；
但 codegen 只推了语句实参（前者 3 个、后者 4 个，`self` 经 `Op::HostSelf` 推玩家）。
任何一次实际激活 hupeng / 触发 gw_dudayuanshuai1 都会 `stack underflow`。

**修复**：弹栈只取语句实参，技能 id 改从 `host.context().skill_id` 读取（VM 执行技能
体时由引擎写入该上下文，与 `Op::HostSkill` 同源）。

## 缺陷二：def 实参被 `.max(0)` 截断

`CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS` 的 `first_def/second_def` 用 `pop()?.0.max(0)`
读取——杜达元帅 def（3583584906 / 3832623127）超过 `i32::MAX`，`ConstU32` 以 i32 位
模式压栈后 `.max(0)` 会把负数钳成 0，导致 `SkillDefId(0)`。

**修复**：def 实参按位保留读取 `pop()?.0 as u32`（与 `ADD_SKILL` / `REMOVE_SKILL_BY_DEF`
既有约定一致）；玩家/目标/卡 id 保持 `.max(0)`。

## 验证

`crates/devtools/tests/gwent_runtime.rs`（VM 级回归守卫，真实执行编译后的
shahai_impl.skill 模块）：
- hupeng 活动模块 → 先发 `DiscardCards{费用}`，再发
  `ChooseOtherGainOneOfTwoSkills{ player, target, first_def=3583584906,
  second_def=3832623127, skill=<上下文> }` ✅
- gw_dudayuanshuai1 触发模块 → 发 `CancelIncomingCardUse{ player, source, card,
  skill=<上下文> }` ✅

| 检查 | 结果 |
|---|---|
| rule-vm 单测 | 15 passed ✅ |
| gwent_runtime 回归 | 1 passed（覆盖两缺陷）✅ |
| conformance / workspace | 全量重跑中（本修复不改变任何已通过向量的输入/输出） |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——仅修正宿主效果弹栈/读取，EffectRequest
形状与字节码 tag 不变。

## 后续

1. **gwyewu（叶舞）造卡入口**：需随机弃牌原语 + 造卡（雾妖/狼 以
   `add_skill` 近似）；宿主效果管道现已验证可用，可在此基础上新增 ABI。
2. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
