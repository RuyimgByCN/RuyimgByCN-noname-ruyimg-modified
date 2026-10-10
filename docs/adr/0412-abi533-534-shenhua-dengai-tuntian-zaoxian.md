# ADR 0412 — shenhua 邓艾【屯田】【凿险】(ABI 533-534)

- 状态:已接受
- 日期:2026-08-02
- 归属:C (omp)
- 关联 ADR:0400(袁术)、0401(祝融)、0398(王平)
- ABI:533(屯田全套)、534(凿险)

## 目标

发布 shenhua 邓艾完整武将:

- **屯田**:当你于回合外失去牌后,你可以进行判定;若判定结果不为红桃,则你将此
  判定牌置于你的武将牌上,称为「田」;你计算与其他角色的距离-X(X 为「田」的数量)。
- **凿险**:觉醒技,准备阶段,若你的「田」数量不小于 3,你减 1 点体力上限,然后
  获得「急袭」。
- **急袭**:你可以将一张「田」当【顺手牵羊】使用。

## 决策

### 田 → 技能牌堆

「田」映射为引擎既有技能牌堆(按 `(玩家, 技能定义)` 键控的公开 Expansion 区,
ABI 287 机制),可直接被 view_as 的 `hand_expansion` 来源区消费,无需新增区域。

### 屯田 (ABI 533)

- 触发:`cards_moved` + 新过滤器 `owner_lost_cards_outside_turn`——非本角色回合、
  本角色手/装/判区牌移出己方控制区(目标区不属于本角色),与旧 JS
  `loseAfter + getl(player)` 语义对齐;本角色自身区间的内部移动(如手→装)不触发。
- 结算:新原语 `resolve_tuntian(player)`——复用 `flip_judgment` 判定,判定牌不为
  红桃时从弃牌堆移入本技能牌堆(田),红桃则留在弃牌堆;触发可选(旧 JS
  `frequent: true` 为可选,文档化一致)。
- 距离:复用既有动态计数修正器机制,新增负计数来源
  `owner_skill_pile_count_negative_<def>`(ABI 533),以
  `modifier distance_from add owner_skill_pile_count_negative_<tuntian_def>` 表达
  距离减田数;既有 `owner_skill_pile_count_<def>` 为正向(孝廉「马」),负向仅限
  `distance_from` 通道(类型检查门控)。

### 凿险 (ABI 534)

- 触发:`phase_prepare` + 新过滤器 `owner_phase_begin_originating_skill_pile_gte_three`
  (本技能牌堆 ≥ 3 张,即田 ≥ 3),强制触发。
- 结算:新原语 `resolve_zaoxian(player, jixi_def)`——复用通用 `ChangeMaxHp(-1)` 与
  `AddSkill(急袭)`;已持有急袭时幂等返回(觉醒一次性,旧 JS `awakenSkill` 语义)。
- 急袭:`view_as shunshou play_phase any from hand_expansion`,纯 DSL,无新机制。

## 接线

`ResolveTuntian`/`ResolveZaoxian` 变体 → 操作码 478/479(表尾追加)→ vm 弹栈 →
vm_host 注入技能实例 → primitives `resolve_tuntian`(判定+移入牌堆)/
`resolve_zaoxian`(觉醒+授技) → hash 臂 503/504 → DSL 词
`resolve_tuntian(player)`/`resolve_zaoxian(player, jixi_def)` → IR/codegen/门控链
`ADDITIVE_RULE_VM_ABI_VERSION_533/534`;过滤器在 catalog/dispatch/registry 三处登记。

## 测试与发布

- Golden `original_shenhua_dengai.rs` 6 项:契约(ABI 534/触发/过滤器/视为来源)、
  屯田非红桃入田+距离减田数、红桃不入田、凿险三田觉醒(上限 4→3+获急袭+田保留)、
  不足三田不觉醒、急袭田为 expansion 区且属主。全绿。
- npack 重建:ABI 534、技能 108(+3)、角色 29(+1,邓艾 id 2912733789,魏 4 体力 男)、
  触发 71(+2)、模块 83(+3);`extensions/shenhua/{shenhua,package}.npack.json`
  与源三副本一致。
- 全量回归与 conformance 见批次收尾(运行时证据在批次总结中)。

## 近似与遗留

- 判定牌移入技能牌堆以公开 Expansion 表达旧 JS 的武将牌上「田」,牌面公开一致。
- 回合外判定(他人回合)的触发窗口以 `state.turn.current != owner` 近似,与旧 JS
  `_status.currentPhase` 一致。
- 急袭使用「田」后是否再次触发屯田:田从牌堆移出不满足「手/装/判区」来源,不触发
  (旧 JS 对使用中的扩展牌亦不视为 lose,一致)。
- 凿险觉醒以「持有急袭」为一次性守卫;若外部移除急袭,准备阶段将再次觉醒
  (文档化近似,正常对局不可达)。
