# 2026-10-11 T11 boss 内容第五批首片：梼杌拟态（规则 v217）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `f964afd7`；梼杌批按指令最小切片——仅拟态单技）。上游依据：`mode/boss.js` `boss_nitai`（5804：`_in` damageBegin4 currentPhase==player → cancel；`_out` damageBegin1 currentPhase!=player ∧ hasNature(fire) → num++）与 `boss_taowu`（895：hp25/五技）。

## 勘误

- **乱常非交替回合序**：`boss_luanchang(+switch)` 实为「体力≤半激活+准备阶段视为用【南蛮入侵】+结束阶段视为用【万箭齐发】」的**虚拟 AOE 双技**；交替回合序属关卡 `bossinfo.loopType==2`（内容面）。此前主稿/模式行的「交替回合序」描述已随本批更正。

## 建模

- `BOSS_GENERALS` 增**梼杌**（群/hp25/`[凶兽, 拟态]`）——**登记**：冥顽（回合内用牌伤害后目标锁定集+每用牌摸 1+目标 mod）/乱常(+switch)（虚拟南蛮/万箭 AOE）未落随下批。
- **拟态**两面（权限型 Noop+commit_damage 内联查询）：
  1. **回合内受伤防止**（诚信免伤同位：目标持技 ∧ `turn.current==target` → `DamagePrevented` return）；
  2. **回合外火伤 +1**（amount 链赤龙牙旁：`nature==Fire` ∧ 持技 ∧ `turn.current!=target` → +1）。
- FROZEN 尾加「拟态」（72 名）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_taowu_nitai_in_shield_out_fire` | 回合内受伤 hp 不变；回合外火伤 +1（25→23）；回合外普通伤害不加成（23→22） |
| 既有回归 | 饕餮四技/穷奇/混沌/烛阴/关卡全部不变；native_kernel 219/0 |
| 版本回退探针（217→216 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **564 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 216→**217**（存档不变 v48） |
