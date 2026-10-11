# 2026-10-11 T11 覆泰2 全场禁桃两面（规则 v214）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `a97568e1`；boss_qiongqi 批三续片）。上游依据：`mode/boss.js` `boss_futai2`（5745：cardSavable+cardEnabled 双 mod——`card.name=="tao"` ∧ 存在**其他**持 `boss_futai` 者 ∧ 当前回合非该持者 → false）。

## 建模

- `tao_futai2_banned(core, catalog, user)`（kernel，pub(crate)）：持覆泰的存活穷奇（排除用桃者自身）在场 ∧ 当前回合≠穷奇 → true。两面消费：
  1. **PlayCard 桃校验**（engine.rs `CardKind::Tao` 分支）：banned → `IllegalTargets` 拒（护符濒死专用不受此 mod——上游 mod 只对 tao）；
  2. **濒死救援**（kernel `is_tao_like` 真桃分支）：banned → 真桃不可救（选项不入）。
- 穷奇自己回合禁令解除（`current == boss` → false）——穷奇自用桃可用。

## 登记差异

- 上游 mod 作用域为全部玩家（含穷奇自己被 current!=player 排除）——原生等价；`_status.event.skill`（技能转化用桃）上下文未建模（原生转化救援走独立谓词，无 futai2 面——登记）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_futai2_tao_banned_both_sides` | ①非穷奇回合 2 号用桃 `Rejected(IllegalTargets)`；②0 号濒死救援面真桃不入选项→死亡；③穷奇自己回合穷奇自用桃可用（禁令解除） |
| 既有回归 | 延妒/覆泰/逐厄/混沌/烛阴/关卡全部不变；native_kernel 216/0 |
| 版本回退探针（214→213 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **561 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 213→**214**（存档不变 v48） |
