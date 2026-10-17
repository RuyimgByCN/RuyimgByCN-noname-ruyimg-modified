# 2026-10-17 T8 余项：具名 filter、效果查询与新原语

**核验日期**：2026-10-17。**规则** `STANDARD_RULESET_VERSION` = **295**（本批不递增）。**存档** `ARCHIVE_FORMAT_VERSION` = **80**（核心末尾 VmAux；空表仍写长度 0）。**增量 1–17 不重做。**

## 结论

词表 642 个 filter 名称都能映射。`map_vm_filter` 先走增量 1–17 的精确策略，再编译具名程序。词表外的名称返回 `None`，装载器放入 `deferred_filters` 并跳过该声明，不恒真也不恒假。`owner_is_target_*` 的每个 OR 组都含「事件主玩家 = 持有者」。

新原语落在 `VmAux`（标记、技能牌堆、本回合丢失、once、旗标、命中、摸牌计数）。标准目录没有 VM 技能时不记弃牌/摸牌史，空 `VmAux` 不写确定性摘要，14 个 v295 golden 保持原值。

从 `compile_source_named` → `bridge_vm_skill_decls` → `Engine::new_with_extensions` → `debug_grant` 起步的双闸：

| 测试 | 正例 | 反例 |
|---|---|---|
| `t8_named_wounded_sha_dual_gate` | 伤害前置、已受伤、事件卡为杀：摸 1 | 满血、他人承受、非杀：不摸 |
| `t8_named_sha_once_dual_gate` | 本回合第一次被他人杀伤害：摸 1 并写入 once | 同回合第二次不摸 |
| `t8_skill_pile_then_nonempty_filter` | 出牌阶段且持有者已有技能牌堆：下一事件摸 1 | 同波牌堆仍空、判定阶段：不摸 |
| `t8_add_mark_writes` / `t8_change_max_hp_writes` | 标记层数 2、体力上限 +1 | — |
| `t8_hand_count_query_gates_draw` | 有手牌时查询放行摸 1 | 空手不摸 |
| `t8_lost_history_dual_gate` | 下一伤害看见本回合手牌丢失后摸回 | 丢失尚未发生、同波弃牌：不摸 |
| `t8_move_batch_dual_gate` | 真实 `CardsMoved` 批次摸 1 | 关掉该技能后再移牌不摸 |
| `t8_skill_uses_remaining_dual_gate` | 实例未用尽时摸 1 | `once_used` 后不摸 |
| `t8_named_filter_vocabulary_maps` | 642 名均 `Some`，`owner_is_target_*` 含 `EvPlayer` | `not_a_real_filter` 为 `None` |

`t8_rvm_bridge` 全文件 **38/0**（含增量 1–17 旧测）。`replay_scenarios` **2/2**（golden 未改）。Gate 1b `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets --offline` exit 0。engine-native 全量未在本目录记，收口门禁另记。

## 登记差异

这些不是上游等价：

1. 具名结算器（`resolve_*` 一类）仍由宿主 `HostBehavior` 承载。VM 桥拒绝这些效果，避免记成已支持的空成功。
2. `card_targeted` 没有独立帧，装载为 `CardUsed` 加目标集。
3. 无内核帧的 DSL 事件（例如 `phase_play`、`game_started`、`debate_resolved`）整包拒装，不伪装成已触发。已映射：`phase_begin` / `phase_end` / `damage` / `damage_before` / `card_used` / `card_targeted` / `card_responded` / `cards_moved` / `cards_drawn` / `dying` / `dying_after` / `death` / `turn_ended` / `hp_lost` / `hp_recovered`。
4. 终身伤害只累计本桥提交的 `DealDamage`。内核伤害仍只在 `turn_damage`。
5. 规则牌 id 0 与哨兵冲突，效果里的 0 视为无牌。
6. 挂起恢复后续跑不再次消费 once。once 旗标在 `VmAux` 里随存档走，消费清单不进 `MachineSnapshot::Rvm`。
7. 阶段帧用事件自己的阶段。伤害、用牌、移牌不带阶段，filter 读当前回合阶段。
8. 具名程序按词表名称切词。含 `_or_` 的复合短语先粘成一个原子，不把整串 AND 成恒真。个别名称的切词比该条 lsp 散文更粗，精确策略（增量 1–17）仍优先于切词。

## 命令

```
cargo test --offline -p engine-native --test t8_rvm_bridge --test replay_scenarios -- --test-threads=8
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets --offline
```

日志：[targeted.log](targeted.log)、[gate1b-all-targets.log](gate1b-all-targets.log)。
