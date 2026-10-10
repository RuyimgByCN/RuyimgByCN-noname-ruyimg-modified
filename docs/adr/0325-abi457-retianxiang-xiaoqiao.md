# ADR 0325：ABI 457 界天香取消+分支原语 + SHENHUA 小乔

本批为 SHENHUA 包小乔（`xiaoqiao`）的「界天香」（`retianxiang`，3203696812）完整移植，
需要新的复合原语 `choose_discard_heart_cancel_damage_branch(player)`（ABI 457）：
在 `damage_before` 触发中弃一张红桃手牌、选择一名其他角色、取消当前伤害，由拥有者
二选一结算分支。复用 ABI 456 天香的红桃选牌/选目标三段选择骨架。

## 原语设计（ABI 457）

- DSL 表面：`choose_discard_heart_cancel_damage_branch(player)`，仅允许在
  `damage_before` 触发中使用（typecheck 显式校验）；编译为
  `effect_tag::RESOLVE_RETIANXIANG`（tag 370）。
- 引擎：`EffectRequest::ResolveRetianxiang { player, target, discard_card, branch,
  source, skill }`；VM 首发由 vm_host 从 `damage_before` 上下文捕获伤害来源。
- 选择结算走 `ContinuationKind::Custom` + 载荷魔法标记
  `RESOLVE_RETIANXIANG_CUSTOM_CHOICE`（0xDCE1_4570）三段：
  红桃手牌 → 其他角色 → 分支（0/1，观察者为拥有者）。
- 应用阶段：将当前 damage_id 加入 `prevented_damage_ids`（取消原伤害）、弃红桃牌；
  - 分支 0：`DealDamage { source, target, 1, 普通 }` 后 `DrawCards { target,
    min(5, 已损失体力) }`（伤害后计算）。
  - 分支 1：`DealDamage { source NONE, target, 1, 失去体力通道 }` 后
    `MoveCards` 将弃牌堆中的红桃牌移入目标手牌。

## 技能移植

「界天香」（`retianxiang`，3203696812）：
- `trigger damage_before owner_is_target optional`，无红桃手牌时原语自行空转。

## 文档化近似

- 旧 JS 于 damageBegin4 触发并 `trigger.cancel()`，DSL 以 `prevent_current_damage`
  等价表达；旧 JS 的 chooseCardTarget 单对话框选牌+目标，DSL 分三段选择
  （红桃→目标→分支）。
- 分支 1 的“获得弃置的牌”以弃牌堆取回表达；若目标先死亡则不再取回（近似）。

## 打包与验证

- `retianxiang.skill` 加入 `content/original/ported/shenhua/`；shenhua 小乔
  （3897256801，女/吴/3 体力，技能 [retianxiang 3203696812, hongyan 3138402391]）
  加入 shenhua `characters.json`；catalog 标记 `retianxiang` 为 `ported`。
- shenhua npack 重建（ABI 457，65 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/shenhua/`。
- 运行时测试 `original_shenhua_retianxiang.rs` 双分支全绿：分支 0（原伤害取消、
  目标受来源 1 点伤害并摸 min(5, 已损体力)）、分支 1（目标失去 1 点体力并获回弃牌）。
- 引擎测试全绿：engine-api 4、rule-vm 10、engine-core 54+4（tianxiang/beizhan/
  retianxiang 新增）；26 个扩展包全新 store 导入成功。
- 旧 JavaScript 不进入 npack 或运行时。
