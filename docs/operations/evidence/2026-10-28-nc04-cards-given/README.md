# 2026-10-28 NC04：`cards_given` 未映射事件接入原生（给牌触发窗）

范围：NC04「未映射事件（如 `cards_given`）阻断完整 shiji」（领取顺序第 5 项示例；本轮指令批）。**规则版本 v325、存档 v105、ABI 753 均不变**——本批是事件面新增：无新持久化状态、无语义分支变化（golden 位移唯一来源经探针证明为事件 id 计数，见 §3）。

## 1. 勘察与语义依据

- 旧线：`DomainEvent::CardsGiven { giver, recipient, cards }`（`give_cards` 原语，giver≠recipient 且非空才发）；dispatch 上下文 `TriggerContext { player: recipient, source: giver, amount: cards.len() }`；滤镜 `owner_alive_gave_cards_to_other_alive` 谓词=owner 存活 ∧ kind=CardsGiven ∧ owner==source ∧ 事件主玩家存活 ∧ owner≠事件主玩家 ∧ 数量>0。
- 使用面：`shiji/ejian`（锁定，`resolve_ejian`）、`sp/weicheng`（optional）、标准 `xinfu_jiyuan`（optional）、`standard_m1`——前二者为扩展包 VM 面（本批解除装载阻断），标准 .skill 为旧线/迁移素材。
- 原生共享点已存在：`machine.rs give_hand_card(from,to,card)`（仁德/突袭/集结/三才/反间/反馈/仙剑等全部经它给牌），已发 `CardsLost`/`CardsMoved`/`LogEvent::CardsGiven`——缺的只是**触发帧**。
- 阻断实证（红，[red-loader.log](red-loader.log)）：DSL `trigger cards_given owner_alive_gave_cards_to_other_alive` 的包被装载器整包拒绝（`VM 技能事件名 cards_given 未映射（增量 3 严格面）`）——即「阻断完整 shiji」的装载层。

## 2. 改动（九处接线）

| 位置 | 改动 |
|---|---|
| `event.rs` | `EventKind::CardsGiven`（枚举尾追加）；`EventData::CardsGiven { giver, recipient, cards }` + `kind()` 臂；文档注明**逐张帧**（与 CardsLost/CardsMoved 同口径；旧线 give_cards 批量单事件——多张给牌触发次数不同，登记差异） |
| `machine.rs` | `give_hand_card` 在 `CardsMoved` 后发射给牌帧（所有给牌路径自动获得） |
| `archive/values.rs` | event_kind tag 22（编/解码） |
| `archive/events.rs` | event_data tag 20（编码 + 解码臂，表尾追加） |
| `engine/hash.rs` | `push_event_data` 判别 24（挂起栈含帧时入摘要） |
| `snap/validation.rs` | `check_event_data` 臂（giver/recipient 界内、cards 界内） |
| `rvm_bridge.rs` | `trigger_ctx_of` 臂（事件主玩家=recipient、来源=giver、amount=张数、kind 9）；`VmFilterPolicy::OwnerAliveGaveCardsToOtherAlive` 变体 + 谓词（含 kind==9 闸，旧线同款显式 kind 检查）+ `map_vm_filter` 映射 |
| `content-native-loader/lib.rs` | `"cards_given" => EventKind::CardsGiven` |
| `tests/nc04_cards_given_event.rs`（新增） | 见 §4 |

替代方案：在 VM 字节码面扩 HOST 事件钩（不取——旧线即宿主事件帧模型，且不动 VM 提交协议）；批量给牌帧（不取——原生 CardsLost/CardsMoved 已逐张，帧口径一致优先；批量差异登记）。

## 3. golden：单场位移 + NOEMIT 探针归因

14 场中仅 `sancai_nested_chain`（唯一含给牌的场景）位移。**NOEMIT 探针**（[golden-probe-noemit.log](golden-probe-noemit.log)：临时停发新帧 → 14 场全绿）证明位移唯一来源是新帧使 `next_event_id` 前移、而该计数在规范摘要（`hash.rs push_u32(next_event_id)`）——**无行为变化、无新持久化语义**，据此重录该场（[golden-after-fix.log](golden-after-fix.log)）并在 `replay_scenarios.rs` 注明原因；规则/存档版本不动（同类先例：PA12 批⑤-2 调度内部态位移）。

## 4. 回归（反例反转 + 完整链）

`nc04_cards_given_event` **4/0**（[green-targeted.log](green-targeted.log)）：

| 测试 | 断言 |
|---|---|
| `cards_given_dsl_package_bridges_to_native_event` | 红的反转：DSL 包经 `bridge_vm_skill_decls` 通过，decl 事件=CardsGiven、滤镜=OwnerAliveGaveCardsToOtherAlive |
| `rende_give_triggers_cards_given_probe_for_recipient` | 仁德给 1 张 → 订阅技真实触发：0 号手牌 −1、**1 号 +2（收牌+探针 `draw_cards(event_player=recipient,1)`）**、牌堆 −1；无残留选择 |
| `probe_on_recipient_side_does_not_trigger` | 滤镜方向负例：持有者=接受者（owner≠source）不触发，1 号仅 +1、牌堆不变 |
| `optional_probe_confirms_and_restores_across_save` | optional → `trigger_confirm` 窗口暂停；**含给牌帧的存档**编码/解码/恢复哈希一致，恢复后应答完成同一效果（与未中断路径逐项一致）——归档 tag 20/判别 24/校验臂被真实行使 |

## 5. 真实 shiji 阻断面（探针，[shiji-bridge-probe.rs](shiji-bridge-probe.rs)）

产品装载顺序中 **ABI 窗口先于事件映射**：真实 `shiji.npack.json` 现以 `rule_vm_abi host=753 package=756` 被拒（与 onlyOL 758 同属 ABI 阶梯，升号须授权）。探针（仅分析用，临时视包 ABI=753）显示事件面全景：`cards_given`(1) 与 `owner_alive_gave_cards_to_other_alive`(1) **已可映射**（桥推进至下一阻断），剩余未映射事件=`phase_prepare`(10，登记差异拒装)、`game_started`(2)、`judgment_resolved`(1)、`sha_resolved`(1) 等（迁移计划 C 行清单），滤镜面另有数十个未映射名。**结论：`cards_given` 层解除；完整 shiji 仍被 ABI 756 与其余事件/滤镜阻断，逐层另批。**

## 6. 影响面验证（2026-10-28 22:15–22:20，HEAD `39dab651` + 本批工作树，规则 v325／存档 v105／ABI 753）

| 命令 | 结果 | 日志 |
|---|---|---|
| `cargo test -p engine-native --no-fail-fast` | **1624/0/73**（333 二进制；基线 1620+4） | [engine-native.log](engine-native.log) |
| `cargo test -p engine-native-wasm --no-fail-fast` | **68/0/0** | [engine-native-wasm.log](engine-native-wasm.log) |
| `cargo test -p content-native-loader --no-fail-fast` | **32/0/0** | [loader.log](loader.log) |
| `cargo test -p game-server --no-fail-fast` | **362/0/0** | [game-server.log](game-server.log) |
| `cargo check --workspace --all-targets` | exit 0（867 条既有警告，0 错误；本批新文件零警告） | [workspace-check-tail.log](workspace-check-tail.log) |
| `pnpm build:native`（WASM 重建）+ `smoke:native` | **72/0** | [smoke-native.log](smoke-native.log) |
| 前端 `pnpm test`／`typecheck` | **123/0/0**／0 错 | [web-test.log](web-test.log)／[web-typecheck.log](web-typecheck.log) |
| 行数门禁 | 单测 OK；**8657** 文件 ≤2000 行 | [source-lines.log](source-lines.log) |

## 7. 兼容性与未覆盖项

- 规则/存档/ABI/协议：全部不变。归档为表尾追加（event_kind 22 / event_data 20 / hash 判别 24）；旧档不含该帧，不受影响。
- **版本口径待裁决**：sancai 单场 golden 位移说明事件 id 计数已入哈希域——按 v312 先例（CardsLost 事件原语即升版）此类哈希域变化应升规则版本；本批无升号授权，按「无版本批 + NOEMIT 探针归因」收口（唯一来源=新帧事件 id，无行为变化）。后果：同 v325 的先后二进制在含给牌轨迹上的哈希对照有漂移（单构建内一致）。**是否补授权 v326（含版本门禁拒绝 v325 及更早）由用户裁决**。
- 未覆盖：完整 shiji 整包（ABI 阶梯 + 其余事件/滤镜，另批）；多张给牌的逐张 vs 旧线批量触发差异已登记（event.rs 文档）；ejian 的 `resolve_ejian` 权威结算为旧线 ABI 414 面，原生等价未做（来源台账范围）；`card_targeted` 逐目标窗（§2 L）与 §2 J 选择链不属本批。

结束时间见 [end-time.txt](end-time.txt)。
