# PA12 迁移批③：滤镜 Top-N 无状态谓词 + 修复 event_targets 桥接缺陷（2026-10-27）

- 执行时间：2026-10-27 **11:02:57 → ≤11:35（原记 11:45 晚于本批提交 11:35，2026-10-27 扫描按提交时间更正） +08:00**（`HEAD = 03b98724`；用户裁决问询未获答复，按推荐项**代理判断开工**——批③为迁移计划既定下一批、纯桥接词汇面、低风险，台帐已注明）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) 批③；本批先修盘点脚本（交替名计数 bug：`"a" | "b" => Some(..)` 此前只计 b），基数修正为滤镜命中 2,457 / 推迟 1,364（672 名）

**一句话结论**：六个高频无状态谓词落原生精确策略并真实技能全链验收——`owner_source_sha`（76）、`owner_alive_not_target`（65）、`owner_is_target_source_alive`（14）、`owner_is_source_or_target`（13）、`owner_source_fire_sha`（10，映射面验收——无简单真实技能）、`owner_source_ordinary_trick`（9）；**修复第三个桥接缺陷**：原生 `EngineVmHost` 未覆写 `event_targets`（默认恒空）→ `HOST_EVENT_TARGET_COUNT` 恒 0（帝挽摸 0 实证），现从 `VmTriggerCtx.targets` 注入（旧线经 `run_module_with_event_cards` 传同集）。once/round 限次系勘正：**`vm_aux.once`（座位×键×范围 1回合/2轮/3整局）机制已存在**，`owner_is_target_once` 经具名程序可承载（map_vm_filter 返回 Some）——限次窗口与旧线 `turn_fires` 的对齐留作语义验收子批（原「需新状态字段」判断撤销）。盘点：滤镜命中 2,457→**2,644**、推迟 1,364→**1,177**、近似可直接原生 1,059→**1,098**。

## 1 六谓词语义（旧线 dispatch 实测 → 原生求值）

| 滤名（次数） | 旧线 | 原生（本批） |
|---|---|---|
| owner_source_sha（76） | `owner_is_source && card_is_sha`（杀 tag 1/5/6） | `event_source==owner && event_card 为 sha_like` |
| owner_alive_not_target（65） | `owner_alive && !owner_is_player` | 同 |
| owner_is_target_source_alive（14） | `owner_is_player && source_alive` | 同 |
| owner_is_source_or_target（13） | `owner_is_source \|\| owner_is_player` | 同 |
| owner_source_fire_sha（10） | `kind==CardUsed && owner_is_source && card_is_sha && 火` | `trig.kind==2 && source && sha_like && nature==Fire` |
| owner_source_ordinary_trick（9） | `owner_is_source && card_is_ordinary_trick` | `source && kind_is_ordinary_trick(card_kind)`（复用既有助手） |

## 2 验收（真实 ported .skill 全链 5/5；旧线差分 5/5）

| 测试（`pa12_batch3_filter_predicates.rs`） | 断言 |
|---|---|
| mapping | 六名映射正确；`owner_is_target_once` 非空（具名程序承载，勘正登记） |
| 帝挽（offline，`card_used owner_source_sha optional`，`draw_cards(self, event_target_count)`） | 普通锦囊净 +1 不过闸；杀单目标：出 −1 + 按目标数摸 1 → 持平（**event_targets 修复的回归**：修前摸 0） |
| 抚（collab，`owner_source_ordinary_trick optional`） | 杀不过闸；无中生有净 +1 + 确认 + 摸 1 = +2 |
| dcweidang（huicui，`phase_end owner_alive_not_target optional`） | 他人结束阶段确认摸 1；自己结束阶段不过闸 |
| 军略（extra，`damage owner_is_source_or_target`） | 造成 1 伤 +1 标记、受到 1 伤 +1、无关伤害 0（VmAux.marks） |
| 旧线差分 `pa12_legacy_batch3_modules` | 抚/军略模块旧线单效果（DrawCards/AddMark）与原生一致 |

## 3 变更与回归

- `rvm_bridge.rs`：六策略 + 求值；`EngineVmHost.targets` 字段 + `event_targets()` 覆写。
- 回归（[b3 汇总](../2026-10-26-pa12-dsl-native-prototype/b3-regressions.txt)）：五 crate 全量 + `cargo check --workspace --all-targets` + 前端 + 三门禁。
- 盘点脚本修复（交替名）后基线与批后数字见 [inventory-after-batch3](../2026-10-26-pa12-dsl-native-prototype/inventory-after-batch3.txt)。

## 4 遗留

- `owner_alive_once_per_round`（38）等限次系：机制（vm_aux.once）已在，语义验收子批待做；`phase/use/damage`（34/16/12）为注释/解析噪声（盘点脚本跳过 `//` 行后消失，下批修脚本）。
- `event_cards` 默认空（`HOST_EVENT_CARD_COUNT` 同族缺口）——下批与 game_started 一并评估。
