# 2026-10-29 NC04：`cards_given` 改为每次给牌一帧并携带全部牌（规则 v326）

范围：[裁决登记](../2026-10-28-user-decisions/README.md)第 10 项——`d7b46ea6` 接入的给牌触发帧按张发射（一次给 N 张触发监听 N 次，旧线 `give_cards` 单事件只触发一次），且该批事件 id 计数入摘要域未升号。本批：**改为每次给牌动作一帧、携带全部牌；规则 v325→v326**（本批独占）；v325 及更早旧档按版本门禁拒绝；golden 三步探针归因后重录。存档 **v105**、全局原生 ABI **753**、Worker／WS 协议不变。开始 [start-time.txt](start-time.txt)，源码基准 HEAD `95f51867`（[source-head.txt](source-head.txt)，B8 批同工作树之上）。

## 1. 反例（先红，[red-counterexample.log](red-counterexample.log)，v325 源码）

| 测试 | 修复前（红） |
|---|---|
| `giving_two_cards_at_once_triggers_cards_given_once` | 仁德一次给 2 张：1 号 `+4`（两次触发各补摸 1），期望 `+3`（一次触发） |
| `single_cards_given_frame_carries_all_cards` | 确认窗栈上的 `CardsGiven` 帧 `cards=[44]`（单张），期望 `[44, 4]`（全部两张） |

## 2. 改动

| 位置 | 改动 |
|---|---|
| `machine.rs` | 新增 `give_hand_cards(from,to,&[CardId])`：`CardsLost`／`CardsMoved` 仍逐张，**`CardsGiven` 每次动作一帧携带全部牌**（事件主玩家=recipient、来源=giver、amount=张数，旧线同口径）；`give_hand_card` 委托批量原语（单张路径帧序不变） |
| `rende.rs` | 按张循环 → `give_hand_cards(self.owner, target, &self.cards)` 一次交付 |
| `xianjian_cards.rs`（蝶舞） | 同上（`sel` 批量交付） |
| `event.rs` | `CardsGiven` 文档改为「每次给牌动作一帧、携带全部牌（规则 v326）」；撤销原「多张给牌触发次数不同」登记差异 |
| `standard.rs` | `STANDARD_RULESET_VERSION` 325→**326** + 版本史注释 |
| 版本断言同步 | `nc04_vm_resume.rs:603`、`nc07_lianyaohu_hash.rs:138,150`、`pa12_tag656_version.rs:9,13` 的 `catalog`／`^` 期望 325→326 |
| `tests/nc04_cards_given_event.rs` | 新增 3 测（§1 两反例 + v325 夹具拒绝） |

**调用方盘点**（9 处）：多张同收者 2 处（仁德、蝶舞）改批量；单张 7 处不变（集结每轮 1 张且目标可变、三才/反间/反馈/TakeFromHand/拼点赠牌各 1 张、突袭每目标 1 张且 giver 不同——各自即独立的给牌动作，与旧线一致）。`debug_give_hand_card` 单张便捷形式不变。`LogEvent::CardsGiven` 随帧批量（`presentation.rs` 的 `Given { cards }` 天然接受 Vec）。

替代方案：让 `give_hand_card` 单张路径也保持逐张 `CardsGiven`、由仁德侧聚合——不取（共享根因在原语层，蝶舞等后续多张给牌技能会重复犯错）；批量 `CardsLost`／`CardsMoved`——不取（改连营等失去牌监听触发次数，超出本项授权）。

## 3. v325 夹具（升号前采集，[capture-v325-fixtures.log](capture-v325-fixtures.log)）

在 v325 源码上一次性测试（采集后删除）产出两份 `.ngsa`：`native-v325.ngsa`（标准目录，仁德一次给 2 张后的静止态）；`cards-given-v325.ngsa`（optional 探针在 `trigger_confirm` 挂起，栈上为 v325 逐张口径的**两个各含 1 张的 `CardsGiven` 帧**——v326 下不可能再产生的结构）。升号后测试断言两者均 `RuleVersionMismatch`（挂起档 `snap ^ catalog == 325 ^ 326`，即差异只来自标准版本位）。

## 4. golden：三步探针归因（全部 14 场）

1. **修复 + v326**：14 场全位移（[step1](golden-probe-step1-fix-v326.log)，版本号入摘要）。
2. **回退 v325 保留修复**：**14 场全绿**（[step2](golden-probe-step2-fix-v325.log)）——批量帧本身对标准 14 场**零足迹**（14 场无一次给多张的动作；单张路径帧序与事件 id 计数逐字相同）。位移唯一来源是版本号（并追认 `d7b46ea6` 未升号时 sancai 场的事件 id 位移）。
3. 恢复 v326 重录：[golden-hex-v326.txt](golden-hex-v326.txt) → [golden-after-rerecord.log](golden-after-rerecord.log) 3/3；`replay_scenarios.rs` 注记 v326 重录原因。

探针过程踩坑记录：`sed -i.bak` 后用 `mv .bak` 恢复，文件 mtime 早于上一次编译 → cargo 判 Fresh 未重编、`pa12_print_golden_hex` 打印的是 v325 值（与旧 golden 相同、伪全绿）；`touch` 强制重编后重取（步骤①②本身经 Edit／`sed -i` 写入、mtime 新于编译，不受影响）。

## 5. 回归与影响面（2026-10-29 00:10–00:40，HEAD `95f51867` + 本批工作树，规则 v326／存档 v105／ABI 753）

| 命令 | 结果 | 日志 |
|---|---|---|
| 定向 `nc04_cards_given_event`／`nc04_vm_resume`／`nc07_lianyaohu_hash`／`pa12_tag656_version` | **7+9+2+1 / 0**（反例反转+旧档拒绝+版本断言同步） | [green-targeted.log](green-targeted.log) |
| `cargo test -p engine-native --no-fail-fast` | **1627/0/73**（334 二进制；基线 1624+3） | [engine-native.log](engine-native.log) |
| `cargo test -p engine-native-wasm --no-fail-fast` | **68/0/0** | [engine-native-wasm.log](engine-native-wasm.log) |
| `cargo test -p content-native-loader --no-fail-fast` | **32/0/0** | [content-native-loader.log](content-native-loader.log) |
| `cargo test -p rule-vm --no-fail-fast` | **21/0/0** | [rule-vm.log](rule-vm.log) |
| `cargo test -p game-server --no-fail-fast` | **362/0/0** | [game-server.log](game-server.log) |
| `cargo check --workspace --all-targets` | exit 0，0 错（801 条既有警告，B8 批后从 867 降） | [workspace-check.log](workspace-check.log) |
| Gate 1b（`-D warnings`） | exit 0 | [gate1b.log](gate1b.log) |
| `build:native`（WASM 从本源码重建）+ `smoke:native` | 重建 exit 0；**72/0** | [build-native.log](build-native.log)、[smoke-native.log](smoke-native.log) |
| `pnpm --dir apps/web-client test`／`typecheck` | **123/0**／0 错 | [web-test.log](web-test.log)、[web-typecheck.log](web-typecheck.log) |
| 行数门禁 | **8657** 文件 ≤2000 行 | [source-lines.log](source-lines.log) |

## 6. 兼容性与未覆盖项

- 规则 **v326**（裁决第 10 项，本批独占）；存档 v105（归档结构不变——`CardsGiven` 帧的 `cards` 向量本就支持多元素，事件 tag／判别／校验臂无需新增）；ABI 753、协议不变。v325 及更早旧档一律版本拒绝（§3 夹具回归）。
- 单张给牌路径帧序与事件计数逐字不变（14 场 golden 零足迹的另一种表述）；`CardsLost`／`CardsMoved` 逐张口径保留。
- 未覆盖：完整 shiji 仍被 ABI 756 与其余未映射事件/滤镜阻断（另批）；旧线 `give_cards` 的 giver≠recipient、非空条件在批量原语同样成立；多张给牌跨**不同收者**的场景（逐张调用方语义不变）无需批量。
