# 2026-10-16 铁索传导/解链语义与「座位序起点」族差异核对

核验日期：2026-10-16。**只读核对 + 登记**（无代码/版本/golden 变化；规则 v290 / 存档 v77）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。
源码摘录：[upstream-chain-propagation.txt](upstream-chain-propagation.txt)。

覆盖清单行为差异表本批处理 4 行：传导伤害来源、传导结算完整性、解链时点、AOE 询问顺序
（并对上一批 AOE 询问顺序的结论作勘正）。

## 1. 核对为一致

| 行 | 上游事实 | 原生 | 判定 |
|---|---|---|---|
| 传导伤害来源 | `_lianhuan` 以 `event._args = [trigger.num, trigger.nature, trigger.cards, trigger.card, trigger.source \|\| "nosource"]` 再对每个目标 `target.damage.apply(target, args)`（`library/skill.js:2039-2050`）→ **保留原始来源**，传导本身不改写来源 | `DamageData{ source: d.source, from_chain: true }`（`kernel.rs`） | 一致（既有 `chain_propagation_preserves_source`） |
| 传导结算完整性 | 上游传导是**完整 `damage` 事件**（可触发伤害前置/后置触发、濒死、转移；`_lianhuan4` 即成链受害者 changeHp 钩子） | 原生传导 `emit(Damage)` 走完整伤害帧 | 一致（既有 `propagated_damage_is_full_resolution`）；"旧线为 follow-up 队列项" 属历史对照口径 |

## 2. 核对出差异（登记，候选对齐）

| 行 | 上游事实 | 原生 | 处置 |
|---|---|---|---|
| 解链时点 | 上游**逐个受害者在自己的伤害结算时解链**：`_lianhuan4`（`trigger: player changeHp`, filter=父事件为 `damage` 且 `hasNature("linked")` 且 `player.isLinked()`）在被伤害者 changeHp 时 `player.link()` 解链（`:2058-2076`）；传播由 `notLink()` 守卫（父事件名 `_lianhuan/_lianhuan2` → false，`gameEvent.ts:1059`）**事后**防级联；传导目标表在传播开始时冻结（`filterPlayer(current != event.player && current.isLinked())`, `:2037`），但每个目标在被结算前**复查 `isLinked()`**（`:2046`） | `commit_damage` 在发出传导帧**之前**把全体链上玩家一次解链（`kernel.rs`），再逐目标 emit 传导 | **差异登记（候选对齐）**：①传导进行中「尚未承受伤害的目标」在上游**仍为链上**、原生**已解链** —— 任何在该窗口读取 `linked` 的触发（如上游 `checkDamage1` 的 `jyliezhou` 钩子 `event.checkJyliezhou = event.player.isLinked()`）两线取值不同；②解链事件（`Linked{linked:false}` 日志与 `TurnOver`-无关的解链子帧）顺序与数量口径不同。对齐 = 改为逐受害者伤害结算后解链 + `from_chain` 防级联，属专项批次（涉及伤害提交时序与哈希） |
| AOE 询问顺序（**勘正**上一批结论） | 上游多目标且**非 multitarget** 的锦囊（如铁索）在 `useCard` step 5 由 `sortTarget()` → `targets.sortBySeat(_status.currentPhase \|\| player)` **按当前回合玩家座位序**重排（`content.ts:10136-10152`）；而 step 12 的 `event.sortTarget(true, true)` 因 `sort===true` **跳过**重排、仅做动画（`:10298-10306`），即 **multitarget（南蛮/万箭）不经该重排**，按目标数组逐个结算 | AOE 按命令提交时的目标序（使用者下家起座位序）逐个结算 | **勘正**：上一批把「非 versus 座位序」写为一致过于笼统。准确结论：结算都迭代目标数组；**站位起点**两线不同源（上游非 multitarget 多目标锦囊=当前回合玩家；原生=使用者），`selectTarget:-1` 自动选择的目标数组构造起点未逐项确认 → 与下一行同属「座位序起点」族差异，登记候选对齐 |

## 3. 「座位序起点」族（本轮核对出的系统性差异）

上游在多处把候选/目标**按当前回合玩家（`_status.currentPhase`）为起点**排序，原生统一以**行为主体（牌来源/受害者）**为起点：

| 位置 | 上游 | 原生 |
|---|---|---|
| 无懈应答名单 | `list.sortBySeat(_status.currentPhase)`（`card/standard.js:4487`） | `seat_order_from(source)`（ADR 0708） |
| 铁索传导目标 | `lib.tempSortSeat = _status.currentPhase \|\| player; event.targets.sort(lib.sort.seat)`（`library/skill.js:2037-2039`） | `seat_order_from(target)`（受害者） |
| 多目标非 multitarget 锦囊目标定序 | `targets.sortBySeat(_status.currentPhase \|\| player)`（`content.ts:10143`） | 命令提交序（使用者下家起） |

共同点：当**使用者/受害者 == 当前回合玩家**时两线同序（标准大多数路径），因此多数既有测试不可区分；
当出现「令他人用牌」「非当前回合出牌」等路径时两线分离。**处置**：作为一个族登记，是否整体改判上游口径需 ADR（改则版本次 + golden 重录 + 各测试断言更新），另立项。

## 4. 本批影响

- 覆盖清单：2 行改判为一致并给出来源；解链时点行写入上游精确时序；AOE 询问顺序行勘正并纳入座位序族。
- 无代码改动（规则 v290 / 存档 v77 / 14 golden 不变）。
