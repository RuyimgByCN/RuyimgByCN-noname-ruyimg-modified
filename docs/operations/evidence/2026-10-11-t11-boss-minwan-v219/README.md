# 2026-10-11 T11 梼杌批末片：冥顽（规则 v219，存档 v49）——梼杌四技全齐

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `ecd15bc3`；梼杌批末片整技交付）。上游依据：`mode/boss.js` `boss_minwan`（5829：clear phaseAfter 删 storage；draw useCard 回合内 storage 激活时摸 1；add 回合内 damageAfter storage=[player]+add(target)；mod playerEnabled 目标不在集合禁用）。

## 建模

- **锁定集存储**：`TurnState.minwan_targets: Vec<PlayerId>`（回合级新字段）——回合更替 `reset_turn_limits` 显式清零（TurnState 原地改字段惯例，v128 先例）=上游 phaseAfter clear 同窗；**存档格式 48→49**（turn 段尾追加 len+pids）。
- **三面**：①**伤害后入集**（`commit_damage` 尾：turn.current==source==持技者 ∧ target≠source → 集合空先 push 自己再 push 受伤者[去重]——上游 `storage=[player]+add` 同口径）；②**每用牌摸 1**（`commit_card_used`：turn.current==用户 ∧ 集合非空 ∧ 持技 → draw 1）；③**PlayCard 目标校验**（engine 校验段：持技者出牌 targets 含「非自己且不在集合」→ `IllegalTargets`——上游 mod playerEnabled 同口径）。
- FROZEN 尾加「冥顽」（74 名）。

## 登记差异

- 锁定集清零时点=回合更替（reset_turn_limits）——上游 phaseAfter（阶段帧后）同窗。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_taowu_minwan_lockset_draw_and_target_mod` | 杀伤后锁定集=[自己,受伤者]；用牌后摸 1；非锁定集目标（3 号）`IllegalTargets` 拒；锁定集目标（决斗打 2 号）可用；回合更替清零；v49 存档往返 |
| 既有回归 | 乱常/拟态/饕餮/穷奇/混沌/烛阴/关卡全部不变；native_kernel 221/0 |
| 版本回退探针（219→218 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **566 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 218→**219**；存档 48→**49** |

## boss 内容面进度

十三批（v208–v219）：**四凶全部四~五技位全齐**（烛阴 1、混沌 4、穷奇 5、饕餮 4、梼杌 4——共 18 将位技能面中的 18 技/5 将）。下批=勾陈或其余 boss 将（逐批同构）。
