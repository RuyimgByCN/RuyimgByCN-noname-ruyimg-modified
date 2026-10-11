# 2026-10-11 T11 boss 内容第二批续片：混沌相德+隐贼（规则 v210）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `9f05a026`；混沌四技至此**全部落齐**）。上游依据：`mode/boss.js` `boss_xiangde`（5678：damageBegin3 来源 getEquip(1) → num++）与 `boss_yinzei(+_switch)`（5688/5538：damageEnd 无手牌 → source.randomDiscard；switch changeHp hp≤max/2 enableSkill）。

## 建模

- **相德**：触发技形态（`TriggerMeta::DamageBefore` forced，YiShang 同款窗口）——`XiangDeBehavior` filter（target=持有者 ∧ 来源非己 ∧ 来源装备区有 `CardKind::Weapon`）+ `XiangDeMachine` 一步直改 `current_damage_mut().amount += 1`（无挂起）。新 `MachineSnapshot::BossXiangde { owner }` 变体：archive tag **61**/canonical_hash **62**（两空间独立）/快照校验/恢复臂（None——永不等待）/a0 样例表尾+variant_index 59。
- **隐贼**：权限型 Noop+`commit_damage` 尾双臂——①**激活门控**：持有者（含未启用实例 `has_skill_any_state`）体力>0 且 ≤max/2 → `SkillInstance.enabled=true`+SkillToggled（装配即未启用=apply 特判，上游 gameStart switch_off 同口径）；②**伤害后弃牌**：激活态 ∧ 持有者无手牌 ∧ 来源存活非己有牌 → 来源弃手牌首张（**随机改首张登记差异**）。
- 混沌 `skills` 补全四技 `[凶兽, 无脏, 相德, 隐贼]`；FROZEN 尾加两技（65 名）。

## 登记差异

- 隐贼激活以 SkillInstance.enabled 单旗承载；上游 switch 的「首次激活永久」在本面等价（enable 后不撤销；再激活幂等）。
- 伤害后弃牌为确定性首张（上游 randomDiscard 随机——随机改首张先例族）。
- 相德直改发生在 DamageBefore 触发机（对齐上游 damageBegin3 窗口）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_hundun_xiangde_yinzei` | 相德：2 号装武器杀混沌 25→23（+1）；隐贼：装配即未激活 → hp 12≤12 伤害窗激活 → 无手牌时来源伤害后弃首张；无脏上限基数 0 复确认 |
| a0 守卫 | `machine_snapshot_tags_unique_and_roundtrip`（60 变体样例齐/tag 唯一/往返）、双射测试不变 |
| 既有回归 | native_kernel 212/0； Boss 关卡/烛阴/混沌无脏回归不变 |
| 版本回退探针（210→209 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **557 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 209→**210**（存档不变 v48——相德机器无挂起态不入存档路径） |
