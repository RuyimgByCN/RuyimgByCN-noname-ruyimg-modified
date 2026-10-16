# 2026-10-16 AOE/决斗响应跳过与无懈窗口询问轮次：上游逐条核对

核验日期：2026-10-16。**只读核对 + 补断言，未改行为**（规则 v290 / 存档 v77 / 14 golden 不变）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`），见
[规则来源基准](../2026-10-16-rule-source-distance-basis/README.md)。源码摘录见
[upstream-ask-semantics.txt](upstream-ask-semantics.txt)。

本批处理覆盖清单行为差异表中「AOE 目标冻结时机 / AOE 无牌跳过可见性 / AOE 询问顺序 /
决斗轮转起手 / 决斗无杀跳过 / 决斗伤害来源 / 无懈询问顺序与重开 / 逐目标窗口重开细节」8 行的
「差分用例核对」处置。

## A. 核对为一致（差异关闭，原生行为用断言冻结）

| 行 | 上游事实（源码） | 原生实现 | 判定 |
|---|---|---|---|
| AOE 无牌跳过可见性 | `chooseToRespond` 由 `event.autochoose = lib.filter.autoRespondSha`（`!player.hasSha("respond")`，`library/index.js:12002`；闪为 `autoRespondShan`，`:12009`）短路为 `result={bool:false}`，**不发起询问**（`content.ts:5489`；仅在 `lib.config.skip_shan` 开时走该短路，真人皮/关配置时表现为空询问后的强制放弃） | 无响应牌者不建询问、直接结算伤害 | 语义一致；**事件轨迹差异**登记（上游存在一个立即取消的响应事件，原生无该事件），断言冻结询问次数 |
| AOE 目标冻结时机 | `useCard` 在 `num == 0 && targets.length > 1` 时 `sortTarget(true, true)` 定序一次（`content.ts:10298`），结算到每个目标前检查 `targets[num].isDead()` 跳过（`content.ts:10300`、效果调用处 `:10730`） | 出牌提交时冻结目标列表，结算中途死亡者按存活过滤跳过 | 一致（既有 `aoe_continues_after_midqueue_death`） |
| AOE 询问顺序 | 非 versus：按 `targets` 已定序数组逐个结算（自使用者下家起的座位序） | 自出牌者下家起按座位顺序 | 一致（既有 `archery_attack_all_pass_takes_damage` 断言观察者序列 1→2→3） |
| 决斗轮转起手 | `event.turn = target`（`card/standard.js:2131`）→ **目标先**出杀 | 目标先（`DuelMachine.on_target` 初态） | 一致（`duel_alternating_sha_until_pass`） |
| 决斗无杀跳过 | 同一 `autochoose`：轮到方无杀 → `result.bool=false` → `await event.turn.damage(event.source)`（`:2227`），**不询问** | 轮到方无杀时不建询问，直接受伤 | 一致；新断言冻结询问次数 0 |
| 决斗伤害来源 | `await event.turn.damage(event.source)`，轮转中 `[source, turn]` 互换（`:2232`）→ 失败方受伤、来源为**对方**（与「最后出杀者」在交替制下同一人） | `emit(Damage{source: opponent, target: responder})` | 一致；新断言冻结 `DamageDealt{source:Some(0),target:2}` 且不存在 `source:Some(2)` |

## B. 核对出真实差异（登记，需专项决策/ADR 才能对齐）

| 行 | 上游事实 | 原生现状 | 处置 |
|---|---|---|---|
| 无懈询问顺序与重开 | `_wuxie` 的应答名单 `list = game.filterPlayer(current => current.hasWuxie(map))` 后 **`list.sortBySeat(_status.currentPhase)`**（`card/standard.js:4473-4487`）——自**当前回合玩家**起（含本人）按座位序；随后 `shift()` 逐个询问（`:4490-4498`） | 自**牌来源**起（含本人）按座位序（`wrap_nullify_window`：`seat_order_from(source)`，ADR 0708 按旧线 `alive_from_seat(actor)` 对齐） | **差异登记**：原生起点与上游不同。当前已建模路径中来源多为当前回合玩家（故不常可观察），但「令他人用牌」类技能（如理断）会令二者分离 → 需 ADR 决策是否改判上游口径（改则需版本次+golden 重录） |
| 逐目标窗口重开细节 | 上游「无懈对无懈」是**嵌套窗口**：无懈本体 `useResult` 再走 `useCardToBegin` → 为无懈自身新建 `_wuxie` 事件，该事件自带按 `_status.currentPhase` 排序的 `hasWuxie` 名单与 `directHit` 过滤，`map.state` 按嵌套层翻转（`:4473-4493`） | 单窗口内 `nullified` 取反 + `queue = seat_order_from(next_alive(responder))` **重建环**（含已放弃者的绕回复问） | **差异登记**：结构不同（嵌套窗口 vs 单窗口重建环）。对齐 = 为无懈本体开独立窗口并保存嵌套层状态（快照/存档面随之变化），属专项批次 |

## C. 未覆盖/不可达边界

- 决斗进行中一方死亡（「来源已死亡时来源字段」）：上游在轮转中**不检查存活**，仍会 `event.turn.damage(event.source)`（来源字段可指向已死亡玩家）；原生在轮到方死亡时走「无杀直接受伤」分支并以对手为来源，同样不校验存活。
  当前已建模技能集中，决斗结算期间双方死亡**不可达**（决斗自身的伤害即结束该次决斗；响应杀触发的技能无致死路径）→ 登记为未覆盖边界，不据此对齐。
- `reverseOrder`（`card/standard.js:1272/1316/1691`，南蛮/万箭/单挑等）**仅 versus 模式**生效（`content.ts:10267` 判 `get.is.versus()`），内容是让玩家选择顺时针/逆时针结算方向（`content.ts:3645-3667`）。原生未建模 versus 结算方向选择 → 归 versus 模式变体登记（模式行），不属标准差异。

## D. 本批改动与验证

- `crates/engine-native/tests/native_kernel.rs`：3 处既有测试补断言（AOE 跳过询问次数 2、决斗无杀询问次数 0、决斗伤害来源字段）。
- 覆盖清单 6 行改判为「已按上游源码核对一致（含事件轨迹/配置面登记）」，2 行登记为结构差异。
- 验证：`cargo test --offline -p engine-native --no-fail-fast`（见 [engine-native-tests.log](engine-native-tests.log)）、Gate 1b（[gate1b-all-targets.log](gate1b-all-targets.log)）。
