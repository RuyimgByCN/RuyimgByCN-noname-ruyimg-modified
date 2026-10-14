# T8 增量 16：伤害面前置 filter 双策略（2026-10-14）

## 任务来源

[T8 余项](../../../architecture/native-progress.md)「具名组合 filter 深化」第四梯队：
两个伤害事件面具名可由现役状态承载（无新字段、无存档/哈希变化、无 golden 重录）：

| DSL filter | 权威语义（lsp） | 承载 |
|---|---|---|
| `owner_target_hand_empty_elemental_or_trick_damage_before`（ABI 358） | 伤害前置窗：owner 手空 ∧ 即将承受元素性（火/雷）或普通锦囊伤害 | `VmTriggerCtx.nature` 新增捕获（Damage 载荷）+ `kind_is_ordinary_trick`（锦囊三类且非延时：乐/兵/闪电/洪水排除、异类不计） |
| `owner_is_target_source_movable_field_card`（ABI 202） | 伤害事件：owner=受伤者、来源存活，且任意其他存活角色场牌（装备/判定区延时）至少一张可移至来源 | 场牌查询（equips/judge）+ 可移性（装备按来源同栏空 `equip_slot`、判定区按同名种唯一——v173 moveCard 基建口径） |

**串帧防护**：`plan_triggers` 的 before 窗对全帧型查 DamageBefore 候选——非伤害
载荷帧 nature/card 双 None → 前者恒否；后者无 event_source → 恒否。

装载器事件面补 `"damage_before" => EventKind::DamageBefore`；filter 精确名 +2
（装载器锁面扩至 14 名，词表差集口径不变）。

**调试面**：`Engine::debug_deal_damage_with_card`（同 debug 管线，card 注入帧
载荷）——"普通锦囊伤害"事件卡面此前无测试通道。

## 定向回归（t8_rvm_bridge +2 测，26/26；反例实证）

- `t8_filter_damage_before_hand_empty_elemental_or_trick`：火伤触发/手牌非空
  负例/无卡普通伤负例/锦囊卡触发（无中生有）/延时锦囊卡负例（乐）/owner 门。
- `t8_filter_damage_source_movable_field_card`：无场牌负例/他人装备可移触发/
  来源同栏被占负例/判定区延时可移触发/来源自身场牌不计（"another"排除来源）/
  owner 门。装备与判定区经上帝视角布置（`state_core_mut` 直改，绕过装备技能
  生命周期——与 filter 无关面）。
- **反例实证**：两臂桩化恒 false → `t8_filter_damage` 2 测 FAILED。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t8_rvm_bridge **26/26** |
| engine-native 全量 | **726/0/0**（14 golden 不变——无哈希面变化）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界

- ABI 202 的"可移至来源"为查询近似：栏空/同名唯一之外，不校验移动类技能自身
  的规则面（距离/次数）。
- ABI 358 的"普通锦囊"按词表异类排除集判定（机关/地不计；未归类扩展种取三类
  近似再排除延时四种），同 is_basic_trick_or_equip 先例。
- `debug_deal_damage_with_card` 为调试面，不进产品路径。

## 过程记录

反例实验中的 `git checkout` 误回退未提交的 rvm_bridge 改动（增量 16 全部丢失），
按本 README 交付清单重放重建——工作区改动未提交时禁用 checkout 回退（先
`git diff > patch`），登记为操作纪律。
