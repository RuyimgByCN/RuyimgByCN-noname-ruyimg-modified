# 2026-10-16 濒死救援名单与轮次：三方对照（上游 / 原生 / 旧线）

核验日期：2026-10-16。**只读核对 + 登记**（无代码/版本/golden 变化；规则 v290 / 存档 v77）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。
源码摘录：[upstream-dying-save.txt](upstream-dying-save.txt)。

覆盖清单行为差异表「濒死应答队列静态」行（原处置：与旧线行为一致性待核对）——本批把三线
（上游/原生/旧线）都对照齐，结论是**名单起点一致、轮次结构三方不同**。

## 1. 三方事实

| 维度 | 上游 | 原生 | 旧线 |
|---|---|---|---|
| 名单起点 | `_save` 起点 = `[_status.currentPhase, event.source, event.player, game.me, game.players[0]]` 首个在场（`content.ts:12422-12433`） | 首轮 `seat_order_from(turn.current)`（`kernel.rs`） | `[turn.current, source, player]` 首个命中（`pack_tw_1.rs`） |
| 轮内名单 | **不预建名单**：循环 `event.player = player.next` 推进，`acted` 集合记录已问者；推进时遇 `acted`/出局即 `break outer`（`content.ts:98-116`） | 预建队列（帧启动时），逐项消费（`kernel.rs`） | 预建队列（请求时一次），逐项消费（`pack_tw_1.rs`） |
| 能力动态性 | 每人被问时实时 `player.canSave(dying)` + `cardSavable`（`:29-30`） | 逐应答者复查 `alive` + `is_tao_like` 持桃（`kernel.rs`） | 逐应答者复查 `alive` + 可救牌/无消耗响应（`pack_tw_1.rs`） |
| 救回者复问 | 原位循环：`result.bool` 但 hp 仍 ≤0 → 同一 `event.player` 再次被问（直到 hp>0 或不再救）（`:88-95`） | 本轮内被移除；**下一轮重建**后可再问（`round_used` + 场上仍有桃 → 重建名单，`:7427-7434`） | 救回者被**前插回队首**，立即复问（`resolve_choice/mod.rs`） |
| 轮次重建起点 | 无重建（单趟） | `seat_order_from(self.dying)`（濒死者，`:7431`） | 无重建（同一名单，save 者前插） |

## 2. 差异登记（方向明确）

1. **逐应答者能力复查一致**（原行「名单静态」描述在「轮内不随技能变动重建」这一点上三线都成立；
   不同的是**能力**每项都实时复查，三线一致）。
2. **轮次结构三方不同**：上游单趟不重建（每人在一次救援中至多被问一次）；
   原生按轮重建（放弃者可在新一轮再被问）；旧线前插复问（救回者立刻再问、无轮概念）。
3. **原生轮次重建起点**用濒死者，与首轮起点（当前回合玩家）不一致；上游/旧线无此二次起点问题。

## 3. 处置

- 原行处置「与旧线行为一致性待核对」→ **已完成核对**：轮内名单静态这一点与旧线一致；
  轮次结构与旧线不同（旧线前插复问 vs 原生按轮重建），与上游也不同（上游单趟）。
- **差异保留 + 列待决策**：是否改判上游单趟（`acted` 语义）属语义变化（版本次＋golden 重录＋
  濒死相关测试改造），与「座位序起点」族一并作为一个决策包；本批不改行为。
- 现有冻结：`negative_hp_rescued_by_three_taos`、`pause_roundtrip_negative_hp_multi_tao_rescue`、
  `dying_save_then_kill...`（旧线侧）、`pause_roundtrip_dying_save` 覆盖轮内多桃与恢复；重建轮方向
  （同一持桃者跨轮连续多张）由 `negative_hp_rescued_by_three_taos` 断言。

## 4. 影响

覆盖清单行为差异表「濒死应答队列静态」行改写；规则基准「当前声明与验证边界」濒死行补上游口径；
无代码、版本与 golden 变化。
