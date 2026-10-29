# 2026-10-29 ADR 0786 J1：VM 提交期选择窗挂起（规则 v328）

范围：[裁决登记](../2026-10-28-user-decisions/README.md)第 13 项采纳的 J1——提交期挂起基建 + `discard_hand_equip_count` 真实选择 + 单族验收；DSL 原语取**方案①**（不新增原语、`RULE_VM_ABI_VERSION` 不变）；规则 **v327→v328**；存档 **v105**（尾部追加族）；应答复用既有 `Answer` 与 `ChoiceKind`。另含 **754–758 原语盘点（只分析）**。开始 [start-time.txt](start-time.txt)，源码基准 HEAD `95f51867`（[source-head.txt](source-head.txt)，B8/v326/GT-014/v327 批同工作树之上）。

## 1. 反例（先红，[red-counterexample.log](red-counterexample.log)，J1 涉案文件回退 HEAD 后运行）

`discard_hand_equip_count(N>0)` 在 Halted 落账时被 `rvm_ops` 内联为「手牌优先确定性首 N、装备补足、不足尽弃」——**不开任何选择窗**：`j1_real_choice_window…` 在 HEAD 源码上 panic「提交期开窗（修复前为确定性首 N，无窗口）」。旧线语义（rule-api 注释）：「选择恰好 count 张可弃置手牌/装备并弃置」。

## 2. 设计与改动（ADR 0786 §A）

| 位置 | 改动 |
|---|---|
| `rvm_bridge.rs` | `VmMachine` 新增 `commit_pause: Option<VmCommitPause{player,count}>`；Halted 落账改走**提交泵** `pump_commit`：顺序落账排队效果，队头为开窗效果（`DiscardHandEquipCount` N>0 且座位有牌）时转入提交期挂起并经 `step` 开窗分支发 `Flow::WaitChoice`（`SelectCards`、恰好 N、手牌+装备、不可取消）；应答后按所选弃置（防御校验区属与数量）并继续提交剩余队列；无牌空过、不足以可用数封顶。once/命中计数仍在 Halted 尾写入（幂等/一次性，恢复续跑不重记） |
| `snap/types.rs` + `archive/machine_{encode,decode}.rs` + `engine/hash.rs` | `MachineSnapshot::Rvm` 尾部追加 `commit_pending`（剩余效果队列，`rvm_ops` 编码）与 `commit_pick`（开窗座位+张数）；归档尾追（旧档读到 EOF 拒绝——v327 及更早已被版本门禁先行拒绝）；两字段入规范摘要（GT-011 身份要求） |
| `rvm_ops.rs` | 效果编码面补 `DiscardHandEquipCount` 臂（tag 26——此前该变体从未出现在挂起队列，编码面未覆盖）；`discard_some`/`seat_of` 改 `pub(crate)` 供桥复用 |
| `snap/restore.rs` + `snap/asks.rs` + `snap/machine_validation.rs` | 提交期挂起档经 `restore_commit_paused` 重建（`asked=true`，恢复后不重问）；`active_ask` 按**推进后状态**派生同一开窗（手牌+装备序、恰好值）；载入校验：开窗座位界内、挂起时队列非空且可解码、未挂起时两者为空、`paused` 与 `commit_pick` 互斥 |
| `archive/resolution.rs` | `KNOWN_PROMPTS` **表尾**追加 `"rvm_discard_pick"`（索引即线上 tag，既有编码不移位）——见 §6 授权解读 |
| `content/standard.rs` | 规则 327→**328** + 版本史注释 |
| 版本断言同步 | 8 处（nc04_vm_resume／nc07_lianyaohu／pa12_tag656／nc04_cards_given／nc07_archive_hash_fields）`catalog`／`^` 期望 →328 |
| `tests/adr0786_j1_commit_window.rs`（新增） | §3 |

**单族验收**：`discard_hand_equip_count` 族（台账批④-4 登记的 80 技能影响面；本批以 DSL 编译包端到端验收该原语的桥路径）。

替代方案：在 VM 执行期（字节码 `HOST_CHOICE`）开窗（不取——旧线该原语的选择发生在效果落账期，且执行期挂起要求把效果先落账再续跑，语义倒置）；为开窗新增 DSL 原语（方案②，须升 ABI——未授权）；AI/机器路径沿用首 N 近似（不取——同一命令流必须确定性可回放，AI 座位以现有启发式应答同一开窗）。

## 3. 回归（4/0，[green-targeted.log](green-targeted.log)）

| 测试 | 断言 |
|---|---|
| `j1_real_choice_window_discards_picked_cards_and_resumes_queue` | 开窗规格（恰好 2、不可取消、候选含全部手牌）；选**非前 N**两张→只弃所选、未选保留（确定性近似必弃前 N）；队列剩余 `draw_cards` 继续落账；所选入弃堆 |
| `j1_window_caps_to_available_and_skips_when_empty` | 可用 1<N2 → 恰好 1；无牌 → 效果空过、后续 draw 照常 |
| `j1_commit_window_saves_and_restores_across_archive` | 开窗挂起中存档→归档往返→恢复哈希一致→派生同一开窗→应答+收尾后终态与未中断路径哈希一致（GT-011 口径） |
| `j1_zero_count_first_launch_remains_rejected` | 方案①边界：N=0 字面量在 **DSL 编译期**拒绝（typecheck「discard count must be a positive integer」）；rvm_ops accept 期 N=0 拒绝是对**手写字节码**的防御层（既有行为照旧） |

## 4. golden（14 场）

版本回退探针（保留修复、版本改回 327）：**14 场全绿**（[golden-probe-fix-v327.log](golden-probe-fix-v327.log)）——标准场无 VM 技能包，修复零足迹；v328 重录（[golden-hex-v328.txt](golden-hex-v328.txt)→[golden-after-rerecord.log](golden-after-rerecord.log) 3/3）仅反映版本号进入摘要。

## 5. 754–758 原语盘点（只分析，后续 ABI 授权依据）

宿主 ABI 为单一整数（当前 753）；升到任一更高值即宣称支持该级及以下全部原语。编译器侧现有 756–758 三级（754 由 B6 删除、755 由 B8 删除，均为空号）：

| ABI 级 | 原语（编译器发射） | 效果 tag | 使用面 | 旧线（engine-core） | 原生宿主（engine-native） | 实现前置 |
|---|---|---|---|---|---|---|
| 754 | ——（空号，B6 删除） | — | — | — | — | — |
| 755 | ——（空号，`da8c7ff0` 引入后从未被引用，B8 批删除） | — | — | — | — | — |
| 756 | `choose_give_or_recast_one_hand(player, require/grant/mark)` | 770 | R03 冯河/英霸（交牌或重铸一张手牌） | 已实现 | **未实现** | 选择窗族——J1 提交期挂起即其基建；另需「交牌/重铸」双分支落账 |
| 757 | `resolve_ol_jie_danshou(player, skill)` | 771 | N01 界朱然胆守（四选项主动技段） | 已实现 | **未实现** | **J2 主动技桥**（`active_index`→ActivateSkill）+ 选择窗 |
| 758 | `choose_other_targets_by_skipped_phase_count_damage_draw`（平寇，tag 772 修复冲突后） | 772 | N01 界朱桓平寇；onlyOL 0.1.5 整包最低 ABI=758 | 已实现 | **未实现** | 多目标选择窗 + 按跳过阶段数计数 |

结论：升 ABI 至 756 仅需 756 一个原语；至 758 须 756+757+758 全部落地（757/758 依赖 J2）。三者在旧线的实现入口：`engine-core/src/vm_host.rs` 与 `command/resolve_choice/`（可作语义对照，不照抄实现）。

## 6. 授权解读（如与用户理解不符请指出，可改为既有键）

`KNOWN_PROMPTS` 表尾追加 `"rvm_discard_pick"`：ADR 0786 §A.3 原文「ChoiceKind/prompt 复用既有种类与**受限键**（SelectCards/SelectPlayers + `rvm_effect_pick` 类受限 prompt）」——本批据此把新增受限 prompt 键视为设计内；裁决第 13 项禁止的是**新增 ChoiceKind 变体或 prompt 载荷**（本批无：`ChoiceKind::SelectCards` 复用、prompt 仍为编译期 `&str` 键、Answer 通道不变）。该表是 N11-b 的零分配校验注册表，表尾追加不移位既有线上 tag。

## 7. 验证（2026-10-29 01:30–02:10，HEAD `95f51867` + 本批工作树，规则 v328／存档 v105／ABI 753）

| 命令 | 结果 | 日志 |
|---|---|---|
| 定向 `adr0786_j1_commit_window` | **4/0**（反例红→绿） | [green-targeted.log](green-targeted.log) |
| `cargo test -p engine-native --no-fail-fast` | **1643/0/73**（337 二进制；1639+4；含 a0_tag_registry 构造点同步） | [engine-native.log](engine-native.log) |
| `cargo test -p engine-native-wasm` | **71/0/0** | [engine-native-wasm.log](engine-native-wasm.log) |
| `cargo test -p content-native-loader` | **32/0/0** | [content-native-loader.log](content-native-loader.log) |
| `cargo test -p rule-vm` | **21/0/0** | [rule-vm.log](rule-vm.log) |
| `cargo test -p game-server` | **362/0/0** | [game-server.log](game-server.log) |
| `cargo test -p content-compiler` | **541/0/0** | [content-compiler.log](content-compiler.log) |
| `cargo check --workspace --all-targets` | exit 0，0 错 | [workspace-check.log](workspace-check.log) |
| Gate 1b（`-D warnings`） | exit 0 | [gate1b.log](gate1b.log) |
| 行数门禁 | **8661** 文件 ≤2000 行 | [source-lines.log](source-lines.log) |
| `build:native` + `smoke:native` | 重建 exit 0；**81/0** | [build-native.log](build-native.log)、[smoke-native.log](smoke-native.log) |
| `pnpm --dir apps/web-client test`／`typecheck` | **127/0**／0 错 | [web-test.log](web-test.log)、[web-typecheck.log](web-typecheck.log) |

## 8. 兼容性与未覆盖项

- 规则 **v328**（裁决第 13 项）；存档 v105（`MachineSnapshot::Rvm` 尾追两字段+`KNOWN_PROMPTS` 尾追一键）；ABI 753、`RULE_VM_ABI_VERSION`、Worker／WS 协议不变。v327 及更早旧档版本拒绝（既有夹具随断言同步覆盖）。
- 未覆盖：756/757/758 原语本体（§5，另批）；`ChooseDiscardEquipmentPreventCurrentDamage`/`choose_other_damage`/`choose_use_virtual_sha` 同族迁移（迁移计划 §2 J 后续批——本批交付它们共用的提交期挂起基建）；J2 主动技桥；§2 L 逐目标窗口；真机/上游整包等价（N01 onlyOL 仍被 ABI 758 阻断）。
