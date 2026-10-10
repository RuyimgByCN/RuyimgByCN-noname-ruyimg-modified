# ADR 0446 — shenhua 刘禅【享乐/放权/若愚】(ABI 560)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0445(张郃)、0435(于吉)
- ABI:560

## 目标

发布 shenhua 刘禅完整武将:

- **享乐**(锁定技):你成为其他角色使用【杀】的目标后,其弃置一张基本牌,否则此杀
  对你无效。
- **放权**:你可以跳过出牌阶段;结束阶段开始时,弃置一张手牌并令一名其他角色进行
  一个额外回合。
- **若愚**(主公技):准备阶段,若你的体力值为全场最少,你加 1 点体力上限、回复 1
  点体力并获得【激将】。

## 决策

### 杀目标排除能力(新引擎原语)

`exclude_saved_sha_target(card, target)` 效果:将当前正在结算的【杀】的保存目标中
移除指定目标(此杀对该目标无效——无响应、无伤害)。保存目标在
`run_effects_with_saved_resume` 于牌使用队列分发**之前**写入 resolution,故
`card_targeted` 触发阶段即可安全移除;享乐/父荫/拒战等「目标被指定后令其无效」
类技能今后复用此原语。

### 享乐 (ABI 560)

- `trigger card_targeted owner_is_target`(强制)→ `resolve_xiangle(player, source, card)`:
  - 校验事件卡为【杀】;无可弃基本牌时直接排除目标。
  - 开「使用者弃基本牌」选择(actor=source);弃牌 → 结算继续;取消 → 排除目标。

### 放权 (ABI 560)

- 出牌阶段可选触发 `resolve_fangquan_skip`:置「FQ1」技能状态标记并跳过出牌阶段。
- 结束阶段可选触发 `resolve_fangquan_grant`:校验标记 → 弃一张手牌 → 选一名其他
  角色 → 压入 `extra_turn_queue`(上限 8)并清除标记;任一取消即放弃并清除标记。

### 若愚 (ABI 560)

- `skill lord_only` + 准备阶段强制触发 `resolve_ruoyu(player, jijiang_def=2377283906)`:
  体力值等于全场最低时(同值允许,与旧 JS `isMinHp` 一致)觉醒一次(「RY1」标记):
  `ChangeMaxHp +1`、`RecoverHp 1`、`AddSkill 激将`。

## 接线

`ExcludeSavedShaTarget`(522)/`ResolveXiangle`(523)/`ResolveFangquanSkip`(524)/
`ResolveFangquanGrant`(525)/`ResolveRuoyu`(526)五个操作码 → vm 弹栈(原始 id 弹栈
用 `as u32` 位宽重解释,修正 def id > i32::MAX 被 `.max(0)` 截为 0 的问题)→
vm_host 注入 → primitives 复合 → 续体标签 `0xDCDC_5600..5602` → hash 臂 517-521 →
DSL 词五个 → IR/codegen/门控 `ADDITIVE_RULE_VM_ABI_VERSION_560`。

## 测试与发布

- Golden `original_shenhua_liushan.rs` 5 项:契约(ABI 560/四触发/三技能)、享乐拒
  弃牌排除保存目标、享乐弃牌保留目标、放权跳过出牌阶段并授额外回合、若愚觉醒
  (加上限/回复/授激将/仅一次)。全绿。
- 48 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 560、技能 127、角色 37(刘禅 id 638667665,蜀 3 体力 男 主公)、触发
  88、active 14、模块 102;`extensions/shenhua/{shenhua,package}.npack.json` 同步。

## 近似与遗留

- 享乐/若愚的旧 JS 主公主公技标识由 `lord_only` 声明表达(引擎按身份门控)。
- 放权「跳过出牌阶段」复用 `SkipCurrentPhase`(含 `restore_skipped_phase` 修正器
  兼容)。
- 并发修复:补齐 Codex 在建 561 potzhouyu 的 codegen 臂与临时桩(其实现落盘后
  移除);修复 `dcchiying_in_range` 签名(改收 `&GameState`)与其调用点;`Op::ConstI32`
  对 u32 定义 id 的位宽重解释。
