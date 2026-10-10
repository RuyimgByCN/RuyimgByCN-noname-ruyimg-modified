# ADR 0445 — shenhua 张郃【巧变】(ABI 559)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0438(鲁肃)、0431(张角)
- ABI:559

## 目标

发布 shenhua 张郃完整武将【巧变】:

> 你可以弃置一张手牌并跳过判定/摸牌/出牌/弃牌阶段;若跳过摸牌阶段,可获得至多
> 两名其他角色各一张手牌;若跳过出牌阶段,可移动场上的一张牌到另一名角色的同类别区域。

## 决策

- 单复合原语 `resolve_qiaobian(player, phase)`(phase: 0 判定 / 1 摸牌 / 2 出牌 /
  3 弃牌),DSL 四个可选触发共用;阶段以显式参数区分,避免四个复合。
- 续体流程(弃牌 → 收益 → 跳过):**收益选择在跳过前开启**,避免
  `skip_current_phase` 内的下一阶段分发嵌套打开新触发打断本次选择;阶段收益结算
  完毕后统一经 `EffectRequest::SkipCurrentPhase` 跳过(复用引擎阶段跳过与
  `restore_skipped_phase` 修正器)。
- 摸牌收益:选至多两名(0-2,可取消)有手牌的其他角色,逐名由**目标本人**开选牌
  (与旧 JS `gainMultiple` 一致),移入持有者手牌。
- 出牌收益:选场上装备/判定区一张牌,再选目的地(非原拥有者),按牌类别
  (延时锦囊 → 判定区,其余 → 装备区)移入。
- 取消语义:任一选择取消即仅结算已完成部分;阶段跳过在收益完成或取消后执行。
- 判定阶段跳过复用 `finish_current_phase_abruptly`(直接进入摸牌阶段,乐/兵/闪电
  不判定)。

## 接线

`ResolveQiaobian{player, phase, stage, skill}` → 操作码 521(续 520 尾)→ vm 弹
stage/phase/player → vm_host 注入技能实例 → primitives `resolve_qiaobian` 主分发 +
五个自定义续体标签 `0xDCDC_5590..5594`(弃牌/摸牌目标/摸牌给牌/出牌选牌/出牌选
目标)+ resolve_choice.rs 五个解析器 → hash 臂 516 → DSL 词
`resolve_qiaobian(player, phase)` → IR/codegen/门控
`ADDITIVE_RULE_VM_ABI_VERSION_559`。

## 测试与发布

- Golden `original_shenhua_zhanghe.rs` 6 项:契约(ABI 559/四触发可选)、判定弃牌跳
  过、摸牌双目标各给一张、出牌移动场上装备、弃牌跳过、取消保持阶段。全绿。
- 47 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 559、技能 124、角色 36(张郃 id 2468129340,魏 4 体力 男)、触发 84、
  active 14、模块 98;`extensions/shenhua/{shenhua,package}.npack.json` 同步。

## 近似与遗留

- 出牌收益的装备移动不做「同槽位空位」检查(引擎装备区为单区追加),判定牌移入
  判定区亦不做上限检查(旧 JS 判定区多槽);文档化近似。
- 摸牌收益「至多两名」下限为 0(可全部放弃,仅弃牌跳过)。
- 顺带修复并发断点:Codex 在建 558 势魏延的 `SkillInstance.mark` 字段缺口(新增
  `mark: u32` 字段 + 哈希臂,mark==0 时不参与哈希,既有 conformance 向量不变)、
  `EffectRequest::LoseHp` 替换为 `DealDamage{is_lose_hp:true}`、opencode 测试
  `Phase::Begin` → `Prepare`、`ChooseTargetDrawToMaxHpCap` 续体名 → `ChooseTargetDraw`。
