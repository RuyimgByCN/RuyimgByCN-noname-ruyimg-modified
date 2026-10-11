# 2026-10-11 T11 朱厌批续片：虚失（规则 v225）——朱厌三技位其三

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `acad0556`；朱厌批续片按指令最小切片——仅虚失(+switch)，巨源 selectTarget mod 登记下批）。上游依据：`mode/boss.js` `boss_xushi(+switch)`（5804：phaseUseEnd turnOver + turnOverEnd 全场 `[1,2].randomGet()` 伤；switch changeHp hp≤max/2）。

## 建模

- **虚失**两面（权限型 Noop+查询）：
  1. **出牌阶段结束自翻面**——锚=**弃牌阶段 begin**（原生无 PhaseEnd 帧——PhaseEnd 段为 begin:false 死区的筑城/光塔特例；神智判定锚同族换相）——Discard begin 臂 emit `TurnOverBefore` 走翻面提交；
  2. **翻回正面全场伤 1**（`commit_turn_over_before` 翻回段：随机 [1,2] **恒 1 登记差异**）。
- 激活门控：第七并列臂（隐贼/延妒/积财/乱常同款；装配即未启用=apply 特判扩至六技）。
- FROZEN 尾加「虚失」（81 名）。

## 登记差异

- 翻正伤害恒 1（上游 [1,2] 随机——随机近似先例族）。
- 巨源（体力上限+5+selectTarget mod）未落（下批：涉 PlayerState.juyuan_storage 字段）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_zhuyan_xushi_flip_and_aoe` | 装配未激活 → hp 置半线伤害窗激活 → 朱厌自己弃牌阶段开始自翻面 → 翻面者回合跳过翻回正面时全场其他角色各-1 |
| 既有回归 | 兵先/兽九/毒妒/妖兽/梼杌/书精/四凶/关卡全部不变；native_kernel 227/0 |
| 版本回退探针（225→224 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **572 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 224→**225**（存档不变 v50） |
