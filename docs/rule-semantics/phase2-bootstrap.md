# 阶段 2 Bootstrap 规则语义

> **历史快照（阶段 2 时点，2026-08-21 注）**：本文记录 bootstrap 子集的语义决策，
> 不随引擎演进而改写；当前完整语义见 `phase5-identity.md` 与
> `docs/content-sdk/phase4-dsl-vm.md`。

对照当前选定原仓库（`../archive/Noname-ruyimg-Modified/apps/core`，相对
本仓库根的路径）的阶段流做**可运行子集**，
不复刻 Promise/事件树。

## 阶段顺序

与 noname `phase` content 对齐的简化映射：

| noname | 本引擎 Phase |
|---|---|
| phaseZhunbei | Prepare |
| phaseJudge | Judge |
| phaseDraw | Draw（进入时摸 `DEFAULT_DRAW_COUNT=2`） |
| phaseUse | Play |
| phaseDiscard | Discard（手牌 > 体力则 Choice 弃牌） |
| phaseJieshu | End |

`StartGame` 结束后直接进入 **Draw** 并为当前角色摸 2 张（跳过首轮准备/判定，便于 headless）。

## 卡牌类别（bootstrap）

| 花色 | CardKind | 出牌阶段 |
|---|---|---|
| ♠ | Sha | 指定 1 名攻击范围内目标，造成 1 点伤害 |
| ♥ | Tao | 已受伤时对自己回复 1 |
| ♣ | Shan | 不可主动打出 |
| ♦ | Jiu | 已受伤时回复 1（阶段 2 简化，非官方“酒”全语义） |

## 伤害 → 濒死 → 死亡

对齐 noname `damage` → `dying` 心智模型（非内部事件对象一一对应）：

1. `DealDamage` 扣减体力
2. `hp <= 0` 且未在濒死 → 进入濒死 + `ChoiceRequest`（token `0`=死亡，`1`=系统求桃）
3. 提交事务段并暂停；`ResolveChoice` 开启下一段
4. 选 `1` → `RecoverHp(1)`，若仍 `hp<=0` 则击杀
5. 选 `0` 或超时取消策略 → `KillPlayer`（手牌置入弃牌堆）

## 选择屏障

- 选择打开期间，仅 `ResolveChoice` / `TimeoutAction` / `AdminAbortMatch` 合法
- 错误 `choice_id`、错误 actor → 拒绝且零状态修改
- Continuation 写入规范状态并参与 State Hash：`DyingSave` / `DiscardToLimit`

## 事务与故障

- 每段 `Transaction::begin` 重置 effect/trigger budget
- 变更记入 `JournalEntry`（诊断）；回滚恢复 baseline
- `BudgetExceeded` / `ContentFault` / `EngineFault` 回滚当前段，不提交

## 与上游差异（刻意）

- 无技能 Trigger 链（阶段 3）
- 无真正“桃”结算顺序（多角色求桃）
- 无闪响应杀
- 弃牌上限 = 当前体力（`needsToDiscard` 简化）
