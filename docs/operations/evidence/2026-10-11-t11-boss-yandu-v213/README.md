# 2026-10-11 T11 boss 内容第三批再续片：穷奇延妒（规则 v213）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `f3548517`；boss_qiongqi 再续片按指令最小切片——仅延妒主技+激活门控，覆泰2 全场禁桃登记下批）。上游依据：`mode/boss.js` `boss_yandu(+_switch)`（5710/5622：global phaseJieshuBegin → `gainPlayerCard(trigger.player, true)`；switch changeHp hp≤max/2 enableSkill）。

## 建模

- **延妒**：权限型 Noop+两面查询——
  1. **激活门控**（`commit_damage` 尾，与隐贼门控并列独立判定）：持有者（含未启用实例）体力>0 且 ≤max/2 → `SkillInstance.enabled=true`+SkillToggled；装配即未启用（apply 特判扩为隐贼+延妒，上游 switch gameStart disable 同口径）；
  2. **主技面**（kernel `Phase::End` begin 臂）：其他角色结束阶段开始时，若其本回合 `turn_damage==0`（v160 既有统计字段，本人回合开始清零口径）且有手牌 → 穷奇获得其**手牌首张**（确定性首张；上游 `gainPlayerCard` 由穷奇选牌——登记差异）。
- 激活语义：enable 后不撤销（幂等）——上游 switch「首次激活永久」等价。
- FROZEN 尾加「延妒」（68 名）。

## 登记差异

- 获得牌为手牌首张（上游穷奇从其手牌/装备任选——选牌交互面简化）。
- `boss_yandu_switch` 独立 switch 技不落表（门控并入主技 SkillInstance.enabled）。
- 覆泰2 全场禁桃未落（下批：濒死救援 `is_tao_like`+PlayCard 桃校验两面）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_qiongqi_yandu_gain_at_end_phase` | 装配即未激活 → 穷奇 hp 置半线后一次伤害（10→9）激活 → 0 号回合内出杀造成伤害后其结束阶段**不**触发 → 3 号回合无伤害其结束阶段穷奇获得其首张手牌 |
| 既有回归 | 覆泰/逐厄/混沌/烛阴/关卡全部不变；native_kernel 215/0 |
| 版本回退探针（213→212 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **560 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 212→**213**（存档不变 v48） |
