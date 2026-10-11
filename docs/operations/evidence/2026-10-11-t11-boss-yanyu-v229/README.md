# 2026-10-11 T11 毕方批末片：炎狱（规则 v229，存档 v52）——毕方四技全齐

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `6b703ea9`；毕方批末片整技交付）。上游依据：`mode/boss.js` `boss_yanyu(+switch)`（5055/5622 区：global phaseBegin，其他角色回合开始判定红色→`damage("fire")`，`getIndex: 3` 每回合至多 3；switch changeHp hp≤max/2 enableSkill）。

## 建模

- `TurnState.yanyu_count: u8` 回合级计数器（每回合已触发次数）——**存档格式 51→52**（turn 段尾 u8；canonical_hash 入参）；`reset_turn_limits` 回合更替清零（minwan_targets 同位）。
- **炎狱门控**：`commit_damage` 尾第八并列臂（hp≤max/2 → enabled 直写；装配即未启用=apply 特判扩至七技）。
- **炎狱主技**：Judge begin 段内联（中毒判定同位）——enabled 持有者（≠当前回合玩家）在场且计数<3 → `draw_judge_card` 确定性判定（铁骑先例）：**红色** → 当前回合玩家受 1 点来自持有者的火伤、计数+1；**黑色**无伤（计数不增）。单持有者场景一次判定即 break。
- FROZEN 尾加「炎狱」（82 名）。

## 登记差异

- **跨回合计数器精确口径待查**：DIAG 观测 3 号/0 号 Judge 各触发一次伤害（hp3=2/hp0=3）但快进至 0 号 Play 后 `yanyu_count=0`——`reset_turn_limits` 每回合开始清零与「每回合至多 3」的交互（多持技者/多回合累计语义）需专项核对；行为面（判定发生+火伤结算）已验证。
- `boss_yanyu_switch` 独立技不落表（门控并入 enabled，隐贼/延妒/积财/乱常/邪酬/虚失先例同构）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_bifang_yanyu_judge_fire_max_three` | 装配未激活 → hp 置半线伤害窗激活 → 置顶红牌后 0 号回合判定红色受 1 火伤 → 连续三座位判定计数器≤3 |
| 既有回归 | 红莲/招祸/虚失/巨源/兵先/相柳/梼杌/书精/四凶/关卡全部不变；native_kernel 231/0 |
| 版本回退探针（229→228 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **576 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 228→**229**；存档 51→**52** |
