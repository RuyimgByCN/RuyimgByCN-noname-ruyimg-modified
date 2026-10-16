# 2026-10-16 触发全序核对：上游（玩家主序 + 交互并列）vs 原生（确定性键）

核验日期：2026-10-16。**只读核对 + 登记**（无代码/版本/golden 变化，规则 v290 / 存档 v77）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。
源码摘录：[upstream-trigger-order.txt](upstream-trigger-order.txt)。

覆盖清单行为差异表「触发全序细节」行（原处置：扩面时用差分用例对照）——本批逐项对照。

## 1. 上游事实

| 维度 | 上游实现（源码） |
|---|---|
| 扫描起点 | `start = [_status.currentPhase, event.source, event.player, game.me, game.players[0]].find(isPlayer)`（`gameEvent.ts:498-500`）——**当前回合玩家优先** |
| 槽位顺序 | `playerMap = game.players.concat(game.dead).sortBySeat(start)`；`do {…player = player.nextSeat} while (player !== start)` → **玩家主序**（座位序自 start 起，含死者；死者亦按 alive 判定由 filter 处理）（`:509-511/517-627`） |
| 时机桶 | 技能可声明 `info.firstDo` / `info.lastDo`，落入 `firstDo`/`lastDo` 两桶；其余落各玩家桶（`:545`）；执行顺序 `doingList.unshift(firstDo)` → 逐玩家桶 → `push(lastDo)`（`:633-634`） |
| 桶内排序 | firstDo/lastDo 桶（`player` 为字符串）：`priority` 降 → 座位序（`playerMap.indexOf`）；普通玩家桶：仅 `priority` 降（`:565-575`） |
| 并列处置 | 每次取「当前最高优先级」子集（`todoList.filter(i => i.priority <= usableSkills[0].priority)`）；firstDo/lastDo 直接取首个；普通桶内若同优先级存在**多技能**：先取 `silent` 或无技能描述者，否则**询问当前角色选择下一个触发技能**（`chooseControl`，prompt「选择下一个触发的技能」，AI/真人皆可）（`content.ts:4027-4058`） |
| 去重 | 每玩家每事件 `listAdded` 去重（同一技能每玩家每事件至多入队一次）；`getIndex` 可返回数值/可迭代 → 同一技能**多次触发**（`gameEvent.ts:551-560`） |
| 过滤 | 每轮 `lib.filter.filterTrigger(trigger, player, name, skill, indexedData)`；`trigger.filterStop()` 可中断整个扫描（`content.ts:4024-4044`） |

## 2. 原生实现

`resolution.rs:290-315`：触发键 = `(Timing::First/Normal/Last, -priority, optional(0/1), seat_distance(turn_current, owner), def_id)`，`entries.sort_by_key`。
- 座位距起点 = `turn.current`（与上游 `start` 的「当前回合玩家优先」一致）。
- `optional`（非强制技）排在强制技之后，对应上游 `forced` 语义但实现为键位。
- 并列（键全等）以 `def_id` 确定性打破。

## 3. 对照结论（差异登记）

| 差异点 | 上游 | 原生 | 性质 |
|---|---|---|---|
| 主序 | **玩家主序**（座位序）× 桶内优先级 | **优先级主序** × 座位距 | 跨玩家、不同优先级的两个触发先后可能不同 |
| 同玩家同优先级并列 | 先 `silent`/无描述技，其余**交互选择**（非确定：取决于 AI/真人输入） | 按 `def_id` 确定性 | 原生确定性 vs 上游交互性 |
| 多次触发 | `getIndex` 返回数值/可迭代 → 同技多次入队 | 无对应原语（按需在各批次内联实现/登记差异） | 未建模（有内容需求时立项） |

## 4. 处置（决策与依据）

**保留原生确定性全序**，理由：
1. 项目契约要求重放/恢复逐命令一致（`replay_determinism_same_seed_identical`、`restore_at_every_choice_point_continues_identically`）；上游「同优先级交互选择」本身不确定，无法在无交互协议下复现。
2. 原生键位可编码、可比较、可调试；上游顺序依赖运行时交互状态。

因此本行**从「待核对」改为「已对照 + 决策保留（确定性优先）」**；跨玩家优先级主序差异属**已知语义差异**，涉及具体技能组合时按各批次台账逐条验收（现有逐技对照批已按此模式登记）。

**边界**：本批只对照机制全序，不代表任何具体技能组合的先后已逐条验收；`getIndex`/`firstDo`/`lastDo` 三处若未来有内容需要，按新批次立项（原生 `Timing::{First,Last}` 已可承载 firstDo/lastDo 的桶语义）。

## 5. 影响

覆盖清单行为差异表「触发全序细节」行改写；规则基准「当前声明与验证边界」增触发全序行；
无代码、版本与 golden 变化。
