# 原生重写统一规则覆盖清单

- 状态：活文档，登记**机制级**证据与未对齐边界。当前进度和待办只在 [进度主稿](architecture/native-progress.md) 维护；扩展逐包状态见 [重构台账](migration/upstream-extension-reconstruction.md)。本表历史条目不自动代表最新整包验收。
- **版本与验证口径（2026-10-17 复核；2026-10-18 NC11 勘正）**：源码实测 `STANDARD_RULESET_VERSION = 322`（工作树 `crates/engine-native/src/content/standard.rs:126`；已提交 HEAD 为 321）、`ARCHIVE_FORMAT_VERSION = 105`（`crates/engine-native/src/archive.rs:113`），仍在开发；扩展切片的逐技回归（如 swd/ow/xianjian 各批）登记在[两仓台账](migration/upstream-extension-reconstruction.md)与各批证据，本表只登记机制键与 golden 场景；当前测试结论与最近一次全绿只在[进度主稿](architecture/native-progress.md#当前核验快照)维护，本表不复制。[第 48 轮 v59](operations/evidence/2026-09-23-native-review-48/README.md)（原生 335 项、全工作区 4,901 项）与 v28/238 项的[第 16 轮](operations/evidence/2026-09-23-native-upstream-review-16/README.md)均为历史快照。规则版本变更见 [玩法基准](architecture/native-rules-baseline.md)。
  测试通过、实现符合声明与官方版本等价分别举证；原稿“引擎全部完成”不再作为验收结论。
- 用途：区分每条规则机制的**已验证 / 近似 / 缺失 / 行为差异**四类状态，
  并给出可执行证据的位置；新内容迁移以此为登记入口。
- 联动守护：`crates/engine-native/tests/replay_scenarios.rs` 中的
  `scenarios_are_registered_in_coverage_manifest` 测试要求本文登记全部
  场景名（`replay:<name>`）与机制键（`` `<key>` ``），清单与代码不可脱钩。

> 本表“已验证”限于列出的原型场景，不表示与官方规则完全等价。14 个 golden 不额外计为 14 个测试；源码变更后须重新跑通测试，不能沿用旧轮次结论。

## 类别判据

| 类别 | 判据 |
|---|---|
| 已验证 | 有**可执行**证据（测试/golden 场景）断言语义结果，且随 `cargo test` 运行 |
| 近似 | 有实现，但语义以注释/声明为主、未经逐项核对（**不等于完成**） |
| 缺失 | 原生线无实现 |
| 行为差异 | 原生线与上游/旧线语义已知不同，或存疑待核对（迁移前必须消解） |

## 范围 A：原生内核机制（`crates/engine-native`）

| 机制键 | 类别 | 证据 | 备注 |
|---|---|---|---|
| `event_stack` | 已验证 | replay:sancai_nested_chain、replay:shouyi_dynamic_skill | 帧父子/弹栈/恢复深度唤醒 |
| `damage_before_window` | 已验证 | replay:yishang_redirect | 伤害前置触发窗口 |
| `payload_rewrite_redirect` | 已验证 | replay:yishang_redirect | 窗口内改写帧载荷 |
| `damage_commit` | 已验证 | replay:sha_dodge_then_damage | 扣血 + 内联濒死检查 |
| `damage_prevent` | 已验证 | replay:suidun_window_matrix | 防止路径（不扣血、有日志） |
| `optional_trigger` | 已验证 | replay:sancai_nested_chain | OptionalGate 确认闸门 |
| `nested_choice` | 已验证 | replay:sancai_nested_chain | 应答只消费一次、跨子事件续行 |
| `once_per_turn` | 已验证 | replay:suidun_window_matrix | 回合开始重置 |
| `mid_resolution_disable` | 已验证 | replay:suidun_window_matrix | 队列运行前重查 enabled/alive |
| `dynamic_gain` | 已验证 | replay:shouyi_dynamic_skill | 运行期添加技能实例 |
| `skill_expiry` | 已验证 | replay:shouyi_dynamic_skill | 到期移除（语义偏差见行为差异表） |
| `redirect_after_gain` | 已验证 | replay:shouyi_dynamic_skill | 动态获得的转移技能即刻可用 |
| `dying` | 已验证（含负体力多桃轮次自救，ADR 0726） | replay:death_terminal_single、`judge_lightning_saved_by_enough_taos`、`negative_hp_rescued_by_three_taos`、`pause_roundtrip_negative_hp_multi_tao_rescue` | 濒死队列、0 血用 1 桃救活、-2 负体力跨三轮连用三桃救回（轮次制 round_used）及三个救援询问点恢复矩阵；死亡分支由既有用例覆盖 |
| `death` | 已验证（所测路径） | replay:death_terminal_single、`death_clears_judge_zone`、`judge_owner_death_settles_zones_and_conservation` | 新用例证明判定伤害结算后终态守恒；因伤害延后、队列先处理完，不等于覆盖死亡时清所有剩余机器持牌 |
| `victory` | 已验证 | replay:death_terminal_single、`identity_lord_side_wins_when_rebels_and_traitor_dead`、`identity_lord_death_decides_by_faction` | 原型模式存活数 ≤ 1；身份局测试范围另见 `identity_mode`，尚未完成全模式验收 |
| `card_use` | 已验证 | replay:sha_dodge_then_damage | 出牌校验与区移动 |
| `sha_respond` | 已验证 | replay:sha_dodge_then_damage | 闪响应/无闪直伤 |
| `sha_limit` | 已验证 | replay:sha_dodge_then_damage | 出牌阶段限一次 |
| `seat_range` | 已验证（含独立参照差分，ADR 0723） | replay:sha_dodge_then_damage、`distance_range_reference_differential` | 座位距离（基础射程 1）；差分用独立参照模型全对全比对存活环距离/范围/next_alive/seat_order_from |
| `draw` | 已验证（所测路径；ADR 0722 含真实洗回断言） | replay:sancai_nested_chain、`reshuffle_conserves_every_card_exactly_once`、`savage_multi_nullify_judge_reshuffle_restore` | 摸牌/洗回实现及长流程守恒；组合用例显式断言 DeckReshuffled 恰一次并逐 Answer 守恒 |
| `give_cards` | 已验证 | replay:sancai_nested_chain | 手牌直给 |
| `hand_limit` | 已验证 | replay:discard_to_limit | 弃牌至上限（Intrinsic 机器） |
| `intrinsic_machine` | 已验证 | replay:discard_to_limit | 帧固有流程（濒死/杀/弃牌共用机制） |
| `reject_rollback` | 已验证（连续段） | replay:rollback_invariants | 所测非法命令整段丢弃；分片后的失败不能据此推定，见 `time_slice` |
| `budget_rollback` | 已验证（所测路径） | replay:rollback_invariants、`aborted_segment_restores_pre_command_state`、`abort_after_restore_restores_segment_start` | 连续段失败自动回滚；分片失败可主动中止，原实例/恢复件均退回段前并同键重试；N07-a/b 原反例关闭 |
| `replay_determinism` | 已验证（所测路径） | `replay_determinism_same_seed_identical`、`replay_different_seed_diverges`、`replay_and_restore_per_command_parity` | seed 307 Prototype 60 命令三线逐步摘要/日志一致、半程一次恢复；同宿主跨进程另有示例，不等于跨平台或所有保存点 |
| `catalog_shared` | 已验证 | `catalog_shared_across_commands` | 目录不随状态克隆 |
| `catalog_inventory` | 已验证 | `standard_catalog_matches_declared_inventory`、`defective_kinds_hold_declared_counts` | A0 标准目录构成检查：`standard_catalog()` 牌名×数量×花色点数与[规则基准声明表](architecture/native-rules-baseline.md#标准目录构成声明表)逐一相等（2026-09-24 修复拆牌循环重复入目录后锁定；2026-10-07 规则 v62/v63 批①、v64 批③-a（含仁王盾编码 94）、v65 批③-b① 青釭剑（编码 95，无视防具）、v66 批③-b② 麒麟弓（编码 96，射程 5 弃马）、v67 批③-b③ 寒冰剑（编码 97，防伤改弃两牌）、v68 批③-b④ 诸葛连弩（编码 98，杀不限次，♣1/♦1×2）、v69 批③-b⑤ 贯石斧（编码 99，闪后弃二强伤）、v70 批③-b⑥ 青龙偃月刀（编码 100，闪后再杀）、v71 批③-b⑦ 丈八蛇矛（编码 101，两牌当杀）、v72 批③-b⑧ 雌雄双股剑（编码 102，异性弃牌/令摸）、v73 批③-b⑨ 方天画戟（编码 103，多目标杀）增至 **124 张**——缺牌种清零、非武器段数量对齐上游；v74 批③-c 对齐段 79 张（杀/闪/桃/南蛮/决斗/顺牵/过拆/无懈/乐/闪电）花色点数改上游 standard list 逐张落位（闪/桃全红为上游口径；原生扩充段 25 张花色为原生取值并登记）；声明表变更须递增规则版本 |
| `tag_registry` | 已验证 | `machine_snapshot_tags_unique_and_roundtrip`、`host_behavior_tags_unique`、`xiaoji_machine_byte_roundtrip_and_rejects`、`host_behavior_keys_bijective`、`card_kind_keys_bijective`、`card_kind_companion_skills_consistent`、`card_kind_encoding_roundtrip_all_variants`、`standard_skill_ids_append_only`（后五个为 10-10 事故恢复中丢失的 lib 单元测试，2026-10-11 T17 重建于 `lib.rs::a0_lib_tests` 与 `archive.rs::a0_rebuilt_tests`） | A0 编码 tag 唯一性：`MachineSnapshot` 59 变体（57 结构+2 单元；2026-10-11 更正，原记 42）tag 互异且编码→解码往返相等、`HostBehavior` **226** tag 唯一（2026-10-11 更正，原记 74 为 v60 时点）；枭姬挂起字节往返（XiaoJi=43）与 Dismantle（29 语义不变）旧字节兼容，未分配 tag 报 BadTag、短输入报 UnexpectedEof（2026-09-24 修复 tag 29 冲突）。扩展包数据键表（ADR 0778）：`CardKind` **133**、`HostBehavior` **226** 个数据键双射且唯一（计数 2026-10-11 更正；双射断言随 T17 重建后回归），牌种伴随技能名全局唯一并覆盖固有装备技。A0 第 3/4 点补齐（2026-10-07）：`CardKind` 133 牌种经 declared_kinds 全量编码往返（测试体待 T17 重建）；standard 46 技能编号声明表只追加（同前） |
| `flip_primitive` | 已验证（所测路径） | `huanle_mianju_blocks_flip_while_equipped`、`flip_frame_lifecycle_no_residue`、`caoren_jushou_flips_and_mianju_cancels_flip` | E03 翻面原语：`PlayerState.face_down` + `TurnOverBefore` 前置取消窗口（漩涡面具锁定技强制取消）+ 翻转/`TurnOvered`；装备面具翻面被取消、摘除后翻面正常（正反例）；`debug_flip` 事件驱动化（压帧→run 驱动到弹栈）。帧生命周期（2026-10-07 缺陷修复固化）：机器 emit 的 TurnOverBefore 子事件必须从 BeforeTriggers 起步、翻面帧 after 窗口静默（无「翻面后」触发种类）；回归守卫断言翻面后 stack/parks 清空；据守（曹仁）经 TurnOverBefore 子事件翻面/翻回+摸 1、面具下取消（登记差异：摸牌仍执行）。规则 v61/格式 v4 递增依据见规则基准 |
| `use_cancel_window` | 已验证（所测路径） | `huanle_monkey_cancels_peach_and_takes_it` | E03 使用前取消窗口：`CardUseData.cancelled`（不入哈希，格式 v3）+ `CardKind::Monkey`（87）+ `HostBehavior::HuanleMonkeyCard`（CardUsed 可选触发，真实装备驱动 filter）；命令层挂起机器逐持有者确认（挂起态可恢复）——确认：取消使用（无 CardUsed 帧）、夺桃、弃猴子离场撤销装备技；全部放弃：桃正常结算。多持有者座位序与两路应答有测试 |
| `pack_loader` | 已验证 | `loader_acceptance_bounded_names_and_clear_errors` | C0 生产侧装载器（ADR 0777/0778 `content-native-loader`）：cards.json 自描述（`pack` 段、规范牌种键、射程/装备技）→`ExtensionPack`，牌种伴随技能由引擎表登记；`BridgeError` 12 个变体均有错误断言；循环内每轮重新装载牌与武将并建局、启停，名称符号表计数不变且名称/名单指针复用（2026-09-24 审查前的"50 轮装载"实为循环外装载一次，已更正）；**八份 g1 测试桥接全部接入**（2026-10-07 六份补齐：huanle/hearth/gwent/yunchou/d3/wuxing，包数据自描述 pack 段；`cards` 可选、`generals` 合并、`link_on_equip` 解析）；依赖图五场景见 `c3_dependency_conflict`（激活强校验/吊销级联剪枝/升级受阻/原子性/revoke 拒绝）；卸载进行中对局见 `e03_c3_uninstall_during_active_game_lock_pinned`（宿主注册表卸载不影响活局 Arc 目录、卸载后恢复的锁绑定拒绝/原包集合成功、新开局不含扩展内容；宿主须保留包字节直到无对局引用为宿主策略登记） |
| `pack_payload_upstream_parity` | 已验证（欢乐包逐张） | `huanle_pack_payload_matches_upstream_list`、`huanle_jiuwei_recovers_or_draws_at_turn_end`、`huanle_combo_stolen_jushou_fires_for_thief`、`huanle_combo_stolen_jushou_flip_cancelled_by_mask`、`huanle_combo_kuwu_sha_like_and_non_sha_damage` | E03 欢乐包独立规则载荷验收（2026-10-07）：上游 7 张实体牌（list.js 权威）逐张对照名称序/花色/点数/牌种/装备栏（酒尾 ♥9 补齐、猴子 ♦5/面具 ♣3 修正、面具防具栏修正）；酒尾真实装备驱动（受伤回血/满血摸牌/放弃三路径 + OptionalGate 恢复）；跨牌组合矩阵（窃取×据守触发、窃取×据守×面具取消、苦无×手里剑负例×雷杀超集）。苦无/写轮眼语义差异登记见规则基准；其余来源包照此模板分批验收；欢乐包整包验收完成（真实宿主全生命周期 `huanle_real_host_full_lifecycle`，2026-10-08——安装/建局/对局中卸载/锁定副本续玩/重装守卫，**生命周期，不计整包**）；杀海 gwent 子包生命周期同模板完成（`gwent_real_host_full_lifecycle`，2026-10-08——U05 子包验收第二例）；古剑子包同模板完成（`gujian_real_host_full_lifecycle`，第三例）；炉石子包同模板完成（`hearth_real_host_full_lifecycle`，第四例）；运筹子包同模板完成（`yunchou_real_host_full_lifecycle`，第五例）；wuxing 子包同模板完成（`wuxing_real_host_full_lifecycle`，第六例）——有转换包子包全部验收；U03 cardpile 来源级验收（`cardpile_real_host_full_lifecycle`，双仓同装单撤场景，**生命周期，不计整包**）；L04 cardpile-liuli 镜像验收（`cardpile_liuli_real_host_full_lifecycle`，**生命周期，不计整包**）；U01 d3 来源级验收（`d3_real_host_full_lifecycle`，**生命周期，不计整包**）；U07 玩点论杀墨守装配接线（`feishi_moshou_declaration_wired_to_host_behavior`，未映射 10→9）；U07 wudian 来源级验收（`wudian_real_host_full_lifecycle`，**生命周期，不计整包**）；U05 swd 首切片（护符 hufu 编码 104+仅自救语义+转换包生命周期，规则 v85）；swd 第二批流星火雨（编码 105+弃二或火伤宿主行为，v86）；swd 第三批草药（编码 106+距离 1 受伤回复，v87）；swd 第四批赤龙牙（编码 107+火伤 +1 锁定，v88）；swd 第五批天仙酒（编码 108+酒的变体，v89）；swd 第六批吸血鬼指环（编码 109+杀伤吸血，v90）；swd 第七批归有界（编码 110+他人延时黑判失体力+随机弃牌，v91）；swd 第八批潜偷面具（编码 111+装备主动技偷牌，v92）；swd 第九批星君眼+青龙令珠（编码 112/113，v93）；swd 第十批贴房绅视（编码 114+距离豁免 TurnState 目标，v94）；swd 第十一批风雪刃（编码 115+杀伤后翻面+摸牌可选拒伤，v95） |
| `equip_slots` | 已验证 | replay:equip_distance_matrix、`equip_horses_weapon_range_and_replacement`、`armor_bagua_judgement_semantics` | 四标准栏；同栏唯一；防具=八卦阵（出闪询问前判定，红桃免伤，ADR 0729） |
| `distance_modifier` | 已验证（含独立参照差分，ADR 0723） | replay:equip_distance_matrix、`equip_horses_weapon_range_and_replacement`、`distance_range_reference_differential` | +马加他人到持有者距离、-马减持有者到他人距离；差分覆盖饱和减与下限 1 |
| `attack_range` | 已验证 | replay:equip_distance_matrix、`equip_horses_weapon_range_and_replacement` | 武器射程覆盖基础 1，替换即时生效 |
| `equip_replace` | 已验证（含卸装效果终止行为证明，ADR 0724） | `equip_horses_weapon_range_and_replacement`、`weapon_replacement_terminates_old_range` | 旧牌入弃牌堆恰一份、帧载荷写回 replaced；行为级证明旧武器射程即时终止（替换后射程回落、远距目标再次不可达） |
| `distance_alive_skip` | 已验证（含独立参照差分，ADR 0723） | `equip_distance_skips_dead_players`、`distance_range_reference_differential` | 座位距离只计存活玩家；差分含死座压缩阶段与传导（next_alive/seat_order_from）参照 |
| `chain_state` | 已验证 | replay:chain_fire_propagation、`chain_card_toggles_link`、`chain_nullify_is_per_target_first_only`、`chain_nullify_is_per_target_second_only`、`chain_double_nullify_leaves_nobody_linked`、`pause_roundtrip_chain_target_window` | 连环横置/重置（铁索对 1-2 名目标翻转）；铁索经 CardUsed+**逐目标**无懈窗口结算（v290 对齐上游 `useCardToBegin` 逐目标窗口）、牌离手入弃牌堆（离手缺陷修复见 ADR 0705；逐目标窗口见规则基准 v290） |
| `damage_nature` | 已验证 | replay:chain_fire_propagation、`fire_sha_propagates_through_chain`、`thunder_damage_propagates_and_unlinks_all` | 普通/火/雷三属性；火杀入线；`debug_deal_damage` 为技能侧属性伤害内核入口 |
| `chain_propagation` | 已验证 | replay:chain_fire_propagation、`propagated_damage_is_full_resolution` | 属性伤害传导 + 全员解链；传导为完整结算（可被转移）；普伤不传导不解链 |
| `aoe_queue` | 已验证 | replay:aoe_response_queue、`savage_attack_response_skip_and_pass`、`archery_attack_all_pass_takes_damage` | 南蛮/万箭多目标响应队列：按冻结目标顺序询问，显式事件栈实现 |
| `aoe_skip_no_card` | 已验证 | replay:aoe_response_queue、`savage_attack_response_skip_and_pass` | 无响应牌者跳过询问直接受伤（挂账见行为差异表） |
| `aoe_full_resolution` | 已验证 | `aoe_continues_after_midqueue_death` | 队列中濒死/死亡后继续结算后续目标；AOE 伤害为完整结算帧 |
| `duel_alternating` | 已验证 | replay:duel_alternating_response、`duel_alternating_sha_until_pass`、`duel_no_sha_immediate_loss`、`duel_damage_can_kill` | 目标先出杀、交替轮转、无杀不询问即败、可触发濒死 |
| `duel_no_distance_limit` | 已验证 | replay:duel_alternating_response、`duel_alternating_sha_until_pass` | 决斗不受计算距离/攻击范围限制；响应杀不占出牌阶段限一次（`duel_does_not_consume_sha_limit`） |
| `nullify_window` | 已验证 | replay:nullify_chain_on_savage、`nullify_cancels_savage`、`duel_can_be_nullified` | 锦囊生效前的无懈询问窗口（南蛮/万箭/决斗/铁索/延时锦囊）；`no_nullify_window_for_basic_card` 断言基础牌不开窗 |
| `nullify_chain` | 已验证 | replay:nullify_chain_on_savage、`nullify_on_nullify_reverses` | 对无懈的无懈反转抵消状态，自出无懈者下家重新询问 |
| `nullify_active_play` | 已验证（所测路径） | `nullify_rejected_when_played_actively` | 无懈可击只能在无懈窗口响应：出牌阶段主动使用（无目标/有目标）以 WrongPhase 拒绝，状态不变，恢复件同样拒绝（2026-09-24 修复：原先落入 `unreachable!` panic） |
| `nullify_scope` | 已验证（部分） | `chain_nullify_is_per_target_first_only`、`chain_nullify_is_per_target_second_only`、`chain_double_nullify_leaves_nobody_linked`、`delayed_trick_play_time_nullify_window`、`savage_per_target_nullify` | AOE 按目标、决斗单目标、**铁索按目标**（v290 对齐上游逐目标 `useCardToBegin` 窗口；原「整牌级窗口」行为差异本批关闭）；延时锦囊改为使用时开窗，判定阶段不再开窗（ADR 0708） |
| `judge_zone` | 已验证 | replay:judge_phase_delayed_tricks、`le_judgement_skips_play_phase`、`bing_judgement_skips_draw_phase`、`judge_zone_same_name_limit`、`judge_order_delayed_first_then_lightning_hit_kills`（双放置序）、`judge_order_grouping_multi_cards_and_restore`（多张 + 快照往返） | 主动放置查重；判定排序为**非闪电放置序在前、闪电在后、队首消费**（ADR 0713，源自旧线），双放置序与多张/恢复已锁定；**顺序规则与上游不同（上游 LIFO、无闪电分组，2026-10-16 核对）**；**已定案保留原生**，差异见行为差异表「判定顺序」——[证据](operations/evidence/2026-10-16-judge-order-upstream/README.md) |
| `delayed_trick` | 已验证（部分） | replay:judge_phase_delayed_tricks、`shandian_hits_and_transfers`、`delayed_trick_play_time_nullify_window`、`judge_order_grouping_multi_cards_and_restore` | 乐/兵/闪电；使用时无懈，判定阶段直接翻牌；闪电黑桃 2–9 命中 3 雷伤（无来源），未中传下一存活玩家，若其已有闪电则弃置；多张时按判定区分组序（见 `judge_zone`，与上游 LIFO 差异登记） |
| `phase_skip` | 已验证 | replay:judge_phase_delayed_tricks、`bing_judgement_skips_draw_phase` | 乐跳过出牌阶段（Draw 后直达 Discard）、兵跳过摸牌（不摸） |
| `machine_state_digest` | 已验证（所测字段） | `park_machine_state_changes_digest`（src 单元）、replay:rollback_invariants（挂起终态 golden） | 已编码机器字段参与摘要（ADR 0704）；后续流离/护驾与扩展机器字段须随规则版本逐次核对，不由早期测试外推完整性 |
| `rule_version_binding` | 已验证 | `ruleset_version_changes_digest`（src 单元）、`restore_rejects_rule_version_mismatch`、`restore_rejects_tampered_core_ruleset_version` | 核心版本入摘要、顶层与核心副本版本不符均被拒绝（并行评审复现的「核心副本被篡改」洞已修复并有回归测试） |
| `snapshot_restore` | 已验证（部分） | `save_restore_at_rest_continues_identically`、`restore_at_every_choice_point_continues_identically`、17 个 `pause_roundtrip_*`、`restore_guidao_rewrite_equips_options`、`open_segment_checkpoint_association_positive`、`restore_rejects_segment_checkpoint_mismatches`、`restore_rejects_{trailing,all_cards,middle}_card_missing` | 原 N04-d/e 与尾端/全量缺牌反例已拒绝；护驾和离间等新增选择恢复见[第 17 轮证据](operations/evidence/2026-09-23-native-review-17/README.md)。后续新机器/选择仍需逐点验收 |
| `time_slice` | 已验证（所测路径） | `time_slice_resumes_to_identical_result`、`suspended_segment_rejects_commands_and_resume_requires_open_segment`、`snapshot_roundtrip_while_suspended`、`aborted_segment_restores_pre_command_state`、`abort_requires_open_segment_and_nonsliced_rollback_unchanged`、`abort_after_restore_restores_segment_start`、`abort_contract_direct_vs_restored_identical` | 分片成功路径同终态/事件；检查点随内存快照保存，恢复件可中止。工程新增测试实际均从耗尽后开始；独立探针另验耗尽前/后 × 空/非空日志四组合，N07-b 已关闭。宿主超时/策略尚未接入 |
| `command_boundary` | 已验证 | `answer_by_wrong_player_rejected`、`commands_by_non_current_player_rejected`、`out_of_bounds_actor_rejected`、`stale_revision_rejected_then_retry_same_dedup`、`duplicate_and_zero_dedup_rejected`、`out_of_bounds_ids_rejected_not_panic`、`answer_without_open_choice_rejected`、`duplicate_selection_rejected_by_spec` | 命令信封 actor/expected_revision/dedup_id；所测错误玩家、过期修订、重复命令、越界 ID 被拒绝；ChoiceSpec 拒绝重复选项（ADR 0704）。尚无服务器认证接入 |
| `nullify_per_target` | 已验证（所测路径） | `savage_per_target_nullify`、`savage_nullified_target_discards_once` | AOE 单目标抵消、其余目标继续，外层统一弃牌；N01 重复弃牌反例已关闭 |
| `nullify_judge_window` | 已验证 | `delayed_trick_play_time_nullify_window`、`delayed_trick_bing_and_shandian_nullify_window` | 延时锦囊使用时窗口（乐/兵/闪电一致）：抵消后弃置且不入判定区；兵/闪电与同目标多牌组合已扩面（2026-10-17）——[证据](operations/evidence/2026-10-17-cov-nullify-judge-expand/README.md) |
| `judge_same_name_limit` | 已验证（所测路径） | `judge_zone_same_name_limit`、`lightning_miss_to_next_with_same_name_discards`、`lightning_miss_transfer_skips_dead_seat` | 主动放置同名限制；闪电转移跳过死亡座位，下家同名则弃置，已替换第九批绕行扫描策略 |
| `identity_mode` | 已验证（部分） | `identity_matrix_for_supported_seats`、`identity_full_match_replay_reaches_lord_side_victory`、`identity_pure_command_match_reaches_outcome`、`identity_pure_command_traitor_victory` | 4–8 人建局/恢复人数拒绝、三阵营终局已有；内奸锚点改为 seed 28。新增 5 人身份终局同种子双运行通过，事件摘要已遍历全部 Debug 内容；仍未从中间恢复件续跑；既有三线是 Prototype 60 命令、半程恢复 |
| `death_rewards` | 已验证（所测路径） | `identity_rebel_death_rewards_killer`、`identity_lord_killing_loyalist_discards_all`、`identity_lord_penalty_discards_equips`、`last_rebel_reward_precedes_victory` | 主公末反贼奖励与终局同段入账已有；不外推所有杀者/终局组合 |
| `visible_views` | 已验证 | `seat_views_hide_opponent_hands_and_deck`、`choice_options_visible_only_to_observer`、`event_view_hides_other_players_drawn_cards`、`isolation_extension_cards_hidden_like_standard`、`isolation_generals_public_in_non_kingdoms_modes`（2026-10-17）、`identity_role_visibility_and_death_reveal`、[第 17 轮 AI 归属回归](operations/evidence/2026-09-23-native-review-17/README.md) | 原观星答案与护驾直接牌 ID 泄露、规划采样复制牌 ID/空牌堆保留真实暗牌的反例已修；非国战模式武将公开可见（含扩展/Boss 段经 `catalog.general_meta`）、国战暗置门控保持、身份可见性按本人/主公/死亡公开；仍不外推为所有扩展与客户端渲染均隔离——[扩面证据](operations/evidence/2026-10-17-rule-identity-view-restore/README.md) |
| `session_auth` | 已验证（部分） | `authed_submit_rejects_impersonation_and_unknown_token`、`auth_token_reissue_revokes_old` | 原同种子跨实例及恢复前旧令牌均被拒，正确重签可用；不代表权威网络已接入 |
| `byte_archive` | 已验证（部分） | `snapshot_archive_roundtrip_rest_and_window`、`archive_detects_tampering_and_version`、n12_crossproc；[第 17 轮字节存档 v2 回归](operations/evidence/2026-09-23-native-review-17/README.md) | 无懈窗口 ChoiceKind、`jiji_target` 映射、`tian_du_gain` 注册等原反例已修；Dismantle/XiaoJi 共用 tag 29 已于 2026-09-24 修复（XiaoJi=43，见 `tag_registry`）；伪造存档中的未知牌种编号已改为 `UnknownCardKind(u32)`、载荷读尽检查 `TrailingBytes`、势力编码走显式表（2026-10-07 加固，负例随测试运行）；同宿主特定保存点跨进程通过不等于所有保存点验收 |
| `kingdoms_mode` | 已验证（原型所测路径） | `guozhan_general_slots_roundtrip`、`guozhan_m3_victory_regression_matrix`、`guozhan_compensatory_reveal_during_targeting` | 槽位/势力/胜负与明置实际授表内技能/体力已有回归；暗置不授实例，六技能已接表，完整装配/模式组合仍待验 |
| `kof_session` | 已验证（会话行与单局恢复路径） | `kof_session_consistency_and_midrestore`、`kof_engine_mode_session_advance`、`kof_m1_two_player_pure_command_endgame`、n11_kof | 会话行（双方将池/比分/局数）文本往返一致（结构非法/将池越界拒绝）；每局后存档恢复续跑与不恢复路径终局一致；单局中途引擎快照恢复哈希轨迹一致（30/60 命令检查点）；终局败者死亡清理（2026-10-07 固化）。语义矛盾行（字段间不一致，如比分与池淘汰不符）仍接受；KOF 多局连续会话循环与示例策略超时判负仍在宿主/示例层（非内核，见规则基准模式行） |
| `presentation_projection` | 已验证（文本示例） | `presentation_visibility_and_render_determinism`、`render_frame_guozhan_generals_isolation`、n10_render_host | 文本帧/增量日志消费；不是原生 GPU 牌桌或输入呈现性能 |

N01 重复弃牌、N02 判定中死亡丢牌、N03 同名堆叠均已关闭，证据见上表及 [当前进度](architecture/native-progress.md)。

### 行为差异（迁移/对齐前不得当作与旧线等价）

> 2026-10-16 勘正：①原「铁索无懈仍为整牌级」行已关闭（规则 v290 对齐上游逐目标 `useCardToBegin` 窗口，见[规则基准 v290 条目](architecture/native-rules-baseline.md)与[证据](operations/evidence/2026-10-16-n06-chain-per-target-nullify/README.md)）；②原「距离下限 1」「座距只计存活」两行按[采用玩法版本与距离语义基准](operations/evidence/2026-10-16-rule-source-distance-basis/README.md)收口（采用版本=本地 upstream noname v1.11.7 的 `get.distance`；仍不声称所有距离技能等价）。本表仅剩以下未对齐项。

| 条目 | 差异 | 处置 |
|---|---|---|
| 临时技能存续期 | **三点语义已勘验（2026-10-16，只读源码）**：原生=至**拥有者**回合结束（5 处授技：妄尊印/移伤/乐学/幻形/石会粉禁用）；旧线=至**授技时当前回合玩家**的回合结束；上游=逐站点 `addTempSkill` 规格（默认 `{global:['phaseAfter','phaseBeforeStart']}`，`phaseAfter` 无触发点、`phaseBeforeStart` 每阶段开始触发→默认≈至当前阶段结束；乐学=addSkill+拥有者阶段边界重掷；幻形=addAdditionalSkill+受伤移除） | **已定案：保留原生口径**（至拥有者回合结束为上界近似；逐站点粒度差异在台账登记）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-temp-skill-expiry-audit/README.md) |
| 触发全序细节 | **已对照并决策（2026-10-16）**：上游为**玩家主序**（`playerMap.sortBySeat(start)`，`start`=当前回合玩家优先，含死者）× 桶内 `priority` 降，另设 `firstDo`/`lastDo` 两桶（执行序 firstDo → 逐玩家 → lastDo），同玩家同优先级并列先取 `silent`/无描述技、否则**交互选择**（`chooseControl`）；原生键 = `(Timing::{First,Normal,Last}, -priority, optional, seat_distance(turn.current, owner), def_id)`（`resolution.rs:290-315`）——**座位距起点与上游一致**，差异在：①主序（优先级主序 vs 玩家主序）②同优先级并列（确定性 `def_id` vs 交互选择）；`getIndex` 多次触发未建模 | **决策：保留原生确定性全序**（重放/恢复逐命令一致为项目契约，上游交互并列不可无交互复现）；跨玩家优先级主序差异属已知语义差异，涉及具体技能组合时按各批次逐条验收；**`getIndex`/桶已定案明确不立项（2026-10-17）**——`firstDo`/`lastDo` 已由 Timing 桶承载、`getIndex` 无内容需求，有内容依赖时再立项——[定案证据](operations/evidence/2026-10-17-cov-permanent-diffs/README.md) |
| 濒死应答队列静态 | **已三方对照（2026-10-16）**：轮内**名单静态、能力实时复查**三线一致（上游 `acted` 单趟 + 每项 `canSave`/`cardSavable`；原生预建队列 + 逐项复查持桃/存活；旧线预建队列 + 逐项复查可救牌）；**轮次结构不同**——上游单趟不重建（每人至多被问一次，救回者原位复问至 hp>0）、原生按轮重建（放弃者新一轮可再被问，重建起点用濒死者）、旧线无轮概念（救回者前插回队首立即复问） | **已定案：保留原生口径**（按轮重建名单，重建起点=濒死者）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-dying-save-roster-upstream/README.md) |
| 死亡清技能 | **已决策（2026-10-16）**：原生死亡结算即删技能实例；旧线保留实例并靠 `alive` 过滤（观察面等价、哈希面不同）；上游为对象式技能表 + `getSkills` 过滤（无独立实例表） | **决策：保留原生「死亡即删实例」**——采用玩法版本是上游，旧线只作迁移对照，**跨线原始哈希等价不是目标**；「迁移哈希门禁」改为按**投影后语义**（可见状态/事件轨迹/终局结果）对照，原生线自身哈希与恢复一致性由 `canonical_hash` 与恢复矩阵守护；删实例使状态更小、恢复更严且不改变观察面——[证据](operations/evidence/2026-10-16-death-reward-order-and-skill-clearing/README.md) |
| 装备即时生效 | 原生线装备提交后立刻参与距离/射程判定（同回合可打）；上游「装备时机」 触发窗口（如 +1 马对结算中之杀的回溯影响）未实现 | **已定案：保留原生「装备即时生效」**（不实现上游 `equipBegin` 窗口化；有真实内容需求时另立 ADR 与版本次）——[定案证据](operations/evidence/2026-10-17-cov-permanent-diffs/README.md) |
| 传导伤害来源 | **已核对一致（2026-10-16）**：上游 `_lianhuan` 以 `event._args = [num, nature, cards, card, source || "nosource"]` 再对每目标 `target.damage.apply(...)`（`library/skill.js:2039-2050`）→ 传导保留**原始来源**、不改写 | 一致；既有 `chain_propagation_preserves_source` 断言来源字段——[证据](operations/evidence/2026-10-16-chain-propagation-seat-order/README.md) |
| 解链时点 | **已核对出差异（2026-10-16）**：上游**逐个受害者在自己的伤害结算时**解链（`_lianhuan4`：changeHp + 父事件 `damage` 且 `hasNature("linked")` → `player.link()`，`library/skill.js:2058-2076`），防级联靠 `notLink()` 守卫（父事件名 `_lianhuan*` → false，`gameEvent.ts:1059`）**事后**判定；传导目标表在传播开始时冻结但每目标结算前**复查 `isLinked()`**（`:2037/2046`）。原生在发出传导帧**之前**先一次解除全部连环（`kernel.rs`） | **已定案：保留原生口径**（发传导帧前一次解除全部连环）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-chain-propagation-seat-order/README.md) |
| 传导结算完整性 | **已核对一致（2026-10-16）**：上游传导是**完整 `damage` 事件**（可触发前置/后置触发、濒死、转移；`_lianhuan4` 即其 changeHp 钩子，`library/skill.js:2058-2076`） | 原生传导 `emit(Damage)` 走完整伤害帧（既有 `propagated_damage_is_full_resolution`）；「旧线 follow-up 队列项」为历史对照口径——[证据](operations/evidence/2026-10-16-chain-propagation-seat-order/README.md) |
| AOE 目标冻结时机 | **已核对一致（2026-10-16）**：上游 `useCard` 在 `num==0 && targets.length>1` 时 `sortTarget(true,true)` 定序一次、结算每目标前查 `targets[num].isDead()` 跳过（`content.ts:10298/10300/10730`）→ 与原生「提交时冻结 + 逐个存活过滤」同口径 | 一致；中途死亡继续结算见 `aoe_continues_after_midqueue_death`——[证据](operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| AOE 无牌跳过可见性 | **已核对一致（2026-10-16）**：上游 `autochoose = lib.filter.autoRespondSha/Shan`（无牌即 `result={bool:false}`，`library/index.js:12002/12009`、`content.ts:5489`）同样**不发起询问**（`skip_shan` 配置关/真人皮下表现为空询问后强制放弃） | 语义一致；**事件轨迹差异**登记（上游存在立即取消的响应事件，原生不建）；询问次数由 `savage_attack_response_skip_and_pass` 断言冻结（无杀目标不产生询问）——[证据](operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| AOE 询问顺序 | **已核对（2026-10-16，含勘正）**：上游多目标**非 multitarget** 锦囊（铁索类）在 `useCard` step 5 由 `sortTarget()`→`targets.sortBySeat(_status.currentPhase \|\| player)` **按当前回合玩家**座位序重排（`content.ts:10136-10152`）；step 12 的 `sortTarget(true,true)` 因 `sort===true` **跳过**重排、仅动画（`:10298-10306`）⇒ **multitarget（南蛮/万箭）不经该重排**，按目标数组逐个结算；`reverseOrder`（顺时针/逆时针方向选择）**仅 versus 生效**（`content.ts:10267`） | 结算都迭代目标数组；**站位起点**不同源（上游多目标非 multitarget＝当前回合玩家；原生＝使用者）；`selectTarget:-1` 自动目标数组构造起点未逐项确认；versus 结算方向选择未建模 | **已定案：保留原生口径**（多目标非 multitarget 锦囊按命令提交序；不以 `_status.currentPhase` 重排）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-chain-propagation-seat-order/README.md) |
| 决斗轮转起手 | **已核对一致（2026-10-16）**：上游 `event.turn = target`（`card/standard.js:2131`）目标先，原生 `DuelMachine.on_target` 初态同 | 起手一致；「一方中途死亡」上游不校验存活仍轮转、原生按轮到方死亡走无杀受伤分支且同样不校验来源存活 → 当前已建模技能集内**不可达**，登记未覆盖边界——[证据](operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| 决斗无杀跳过 | **已核对一致（2026-10-16）**：上游同 `autochoose` 短路后 `await event.turn.damage(event.source)`（`card/standard.js:2227`），轮到方无杀不询问直接受伤 | 一致；询问次数 0 由 `duel_no_sha_immediate_loss` 断言冻结——[证据](operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| 决斗伤害来源 | **已核对一致（2026-10-16）**：上游 `event.turn.damage(event.source)` + 轮转 `[source,turn]` 互换（`card/standard.js:2227/2232`）→ 失败方受伤、来源=对方；「来源已死亡」上游与原生均不校验存活（不可达） | 一致；`duel_alternating_sha_until_pass` 新增来源字段断言（`source:Some(0),target:2` 恰 1 次、无 `source:Some(2)`）——[证据](operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| 无懈询问顺序与重开 | **已核对出差异（2026-10-16）**：上游 `_wuxie` 名单 `filterPlayer(hasWuxie)` 后 **`sortBySeat(_status.currentPhase)`**——自**当前回合玩家**起座位序（含本人，`card/standard.js:4473-4487`）；「无懈对无懈」是**嵌套窗口**（无懈本体重走 `useCardToBegin` 新建 `_wuxie`，`map.state` 按嵌套层翻转）。原生自**牌来源**起（ADR 0708 按旧线 `alive_from_seat(actor)`），且单窗口内取反 + 自出无懈者下家**重建环** | **已定案：保留原生口径**（自牌来源起；单窗口取反 + 重建环）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| 延时锦囊无懈时点 | 已按旧线改为使用时开窗，来源起含本人；被抵消则弃置、不入判定区；判定阶段不再开窗 | **已定案：保留原生口径**（仅使用时开窗；上游另有 `phaseJudge` 判定时窗口，登记为已知差异）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-delayed-trick-nullify-timing/README.md) |
| 逐目标窗口重开细节 | **已核对出差异（2026-10-16）**：上游逐目标窗口 = 每个目标各一个由 `useCardToBegin` 触发的 `_wuxie`（名单与该目标绑定、按当前回合玩家排序）；「抵消后重开」由**无懈自身的嵌套窗口**承担。原生为同窗口单次遍历 + 抵消后重建环（`queue = seat_order_from(next_alive(responder))`，含已放弃者绕回复问） | **已定案：保留原生口径**（同窗口单次遍历 + 抵消后重建环；不引入无懈嵌套窗口）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| 奖惩与终局判定顺序 | **已对照并保留原生（2026-10-16）**：上游同一次死亡内 `dieAfter`（含 `game.checkResult()`→`game.over()` 胜负结算）**先于** `dieAfter2`（奖惩：杀反贼来源摸 3、主公杀忠臣弃全部，`content.ts:12537/12615`、`identity.js:3103-3190`）；原生为**奖惩先于终局判定**（ADR 0705，`last_rebel_reward_precedes_victory`） | **已决策：保留原生顺序**（保证奖励计入终局前状态）。**待核项已关闭（2026-10-17）**：上游 `game.over` 只置 `_status.over` 并快照手牌、不中断 death 内容链，`dieAfter2` 仍执行 ⇒ 上游 over 后**仍发奖惩**，两边都发放，差异仅在顺序与终局快照口径——[2026-10-16 证据](operations/evidence/2026-10-16-death-reward-order-and-skill-clearing/README.md)、[2026-10-17 审计](operations/evidence/2026-10-17-cov-reward-over-audit/README.md) |
| 改判与天妒切片 | 原范围/旧牌/装备资格与恢复/Finalized 反例已修复，主动与放弃恢复已固化 | 关闭原 N08-c/d；新增判定技能与复合路径继续按逐规则证据验收 |
| 判定顺序 | **已核对出差异（2026-10-16）**：上游 `phaseJudge` 以判定区数组（DOM 顺序，`$addVirtualJudge` 新牌 insertBefore firstChild）为队列并 **`shift()` 队首消费 ⇒ LIFO（最后放置的先判）、无闪电分组**（`content.ts:4931-4960`、`player.js:5391-5412/15791`）；原生为**非闪电按放置序 FIFO、闪电恒最后**（`kernel.rs`，源自旧线 `collect_delayed_queue`/ADR 0713） | **已定案：保留原生口径**（非闪电放置序 FIFO + 闪电恒最后，ADR 0713）——[语义选择定案](architecture/native-rules-baseline.md) / [对照证据](operations/evidence/2026-10-16-judge-order-upstream/README.md) |

第 16 轮列出的主动技阶段门控、激将/护驾暗手预筛与逐人询问、仁德分次累计等差异已在[第 17 轮证据](operations/evidence/2026-09-23-native-review-17/README.md)逐项修复并加入回归；不再列为当前行为差异。逐技能、复合场景与上游等价性仍须单独举证。

## 范围 B：原生内容与完整性

| 条目 | 类别 | 证据与范围 |
|---|---|---|
| 散财/移伤/碎盾/肃清/授艺 | 已验证（机制样例） | replay:sancai_nested_chain、replay:yishang_redirect、replay:suidun_window_matrix、replay:shouyi_dynamic_skill；不计官方迁移 |
| 基础牌、装备、锦囊切片 | 已验证（所测路径） | 范围 A；顺手/拆桥仍确定性取首牌，防具八卦为现行切片 |
| 鬼道/天妒 | 已验证（所测路径） | 真实装备三组、`judgement_event_order_rewrite_then_tiandu`、主动/确认/放弃 roundtrip；原 N08-c/d 关闭 |
| 鬼才/反馈、奸雄/遗计、闭月/洛神、武圣/龙胆、无双/集智 | 已验证（局部语义） | native_kernel 对应技能用例；ADR 0736–0745；并非整将技能集均完整 |
| 苦肉/制衡/青囊、英姿/枭姬/克己、国色/奇袭/咆哮 | 已验证（局部语义） | native_kernel 主动/阶段/转化用例；ADR 0746–0752 |
| 刚烈/突袭/裸衣、观星/空城、谦逊/连营、马术/铁骑 | 已验证（局部语义） | native_kernel 对应用例；ADR 0758–0763；主动选择与组合恢复仍需扩面 |
| 仁德/激将/护驾 | 已验证（第 17 轮反例） | 阶段门控、仁德分次累计、激将/护驾按资格逐人询问与公开信息隔离、选择恢复有[回归](operations/evidence/2026-09-23-native-review-17/README.md)；只关闭列出的反例，不计整将上游等价 |
| 倾国/奇才/救援、急救/流离/结姻/反间/离间 | 已验证（v202 差异消除批后局部语义对齐） | 第 17 轮补了离间性别选择恢复及倾国选项派生；v202 消除四条登记差异（倾国黑手牌/救援主公+吴势力/结姻受伤男性/急救回合外，[证据](operations/evidence/2026-10-11-t11-standard-diff-v202/README.md)）；反间（v203）与护驾自选代出+倾国转化（v204）亦已消除（[证据](operations/evidence/2026-10-11-t11-hujia-v204/README.md)）——本行六技+流离登记差异全部消除 |
| 标准武将 | **缺失来源将已清零（整将迁移仍差牌表/模式）** | 上游 33 条目 2026-10-07 五批全部补齐（乐进/潘凤/华雄/甘夫人/李典/公孙瓒/伊籍/袁术：骁果/狂斧/耀武/淑慎/神智/恂恂/忘隙/义从/机捷/急援/妄尊/同疾，装配+行为测试），另有原生新增曹仁；牌表逐张对照、模式对齐与组合验收仍待；后续原生扩展卡/将切片不等于标准整将包完成。逐项见[规则基准](architecture/native-rules-baseline.md#本地上游对齐的新增核查) |
| 标准物理牌与牌种 | 已验证（声明表逐张对齐；原生扩充段花色登记） | 上游 standard 为 108 张物理牌、32 牌种，逐张对照完成（2026-10-07，[牌表对照证据](operations/evidence/2026-10-07-n08-cardlist-parity/README.md)）；原生标准目录 v74 起 124 张：缺牌种 4 补齐（v62/v63）、装备命名化 11 张逐把特效（v64–v73）、对齐段 79 张花色点数逐张落位（v74 批③-c，闪/桃全红为上游口径）；原生扩充段 25 张（±马/酒/雷杀/泛型武器/八卦/火杀/铁索/万箭/兵粮）花色为原生取值并登记（[规则基准声明表](architecture/native-rules-baseline.md#标准目录构成声明表)）。由 `catalog_inventory` 守护；扩展目录增卡不等于标准牌包验收；逐技能语义对照仍待做 |
| 武将装配 | 已验证（部分） | `n11_reveal_grants_skills_and_hp`、`general_assembly_grants_and_remove_revokes_slot_skills` | 国战明置实际授技能/体力（孙权）；P2/N08 装配授予原语（2026-10-07）：`debug_assemble_general` 与明置共用 `apply_general_assembly`（授予语义不分叉）——装配曹仁即授予据守+体力上限；替换装配为并集授予、摘除只撤销当前槽技能集并清槽；`debug_set_general` 保持仅装槽（势力/性别资格测试依赖）。原型模式无命令层选将入口（登记）。完整模式/性别/恢复组合继续验收 |
| 身份/KOF/国战/对决/斗地主/Boss | 已验证（模式切片；KOF 会话态已入引擎层 v83） | 三阵营终局与模式回归已有；明置授予/暗置无实例已有；KOF 轮换、空池解析实现；会话行往返/恢复续跑终局一致、单局中途快照哈希轨迹一致、死亡清理固化；versus 2v2 基线奖惩/终局/存档往返固化（`versus_mode_team_victory_and_rewards`，2026-10-08，替补等未建模登记）；doudizhu 终局/存档往返固化（`doudizhu_mode_victory_and_roundtrip`，2026-10-08，特殊开局摸牌变体未建模登记）；Boss 重整复活回环/双终局/存档往返固化（`boss_mode_chongzheng_revival_and_victory`，2026-10-08）；addFellow 座位扩张/fellows 往返固化（`boss_mode_add_fellow_seat_expansion`，2026-10-08，入队队尾差异登记；技能包未建模登记）；KOF 引擎层会话推进/往返固化（`kof_engine_mode_session_advance`，2026-10-08，多局连续循环仍在宿主层登记）（`kof_session_consistency_and_midrestore`，2026-10-07）；完整模式仍待验收 |
| 所有历史包/正式完整内容 | **未完成** | 先固定玩法版本和逐技能语义清单；不按武将名称或测试数量计算完整迁移率 |

## 扩展机制（v3.3 新增必做范围）

| 条目 | 当前状态 | 完成条件 |
|---|---|---|
| 指定两仓全部扩展 | **来源清点完成、多个子包/纵向样本部分转换；原生整包验收（2026-10-17 批次快照为 20/37；**2026-10-18 NC11 勘正为登记表机器字段 24/37 标签、13 项仍 open，且标签不等于验收，见本行末**）（U03、U06、L06、L07、L23、U04、L04、L05、L22、L02、L15、L17、L18、L26、L21、L16、U02、L03、L13 与 L29，2026-10-17，[U03](operations/evidence/2026-10-17-t12-u03-eight/README.md)、[U06](operations/evidence/2026-10-17-t12-u06-eight/README.md)、[L06](operations/evidence/2026-10-17-t12-l06-eight/README.md)、[L07](operations/evidence/2026-10-17-t12-l07-eight/README.md)、[L23](operations/evidence/2026-10-17-t12-l23-eight/README.md)、[U04](operations/evidence/2026-10-17-t12-u04-eight/README.md)、[L04](operations/evidence/2026-10-17-t12-l04-eight/README.md)）、[L05](operations/evidence/2026-10-17-t12-l05-eight/README.md)、[L22](operations/evidence/2026-10-17-t12-l22-eight/README.md)、[L02](operations/evidence/2026-10-17-t12-l02-eight/README.md)、[L15](operations/evidence/2026-10-17-t12-l15-eight/README.md)、[L17](operations/evidence/2026-10-17-t12-l17-eight/README.md)、[L18](operations/evidence/2026-10-17-t12-l18-eight/README.md)、[L26](operations/evidence/2026-10-17-t12-l26-eight/README.md)、[L21](operations/evidence/2026-10-17-t12-l21-eight/README.md)、[L16](operations/evidence/2026-10-17-t12-l16-eight/README.md)、[U02/L03](operations/evidence/2026-10-17-t12-u02-l03-boss-play-eight/README.md)；苦无、五行随机与弃牌选择、联机/录像/帮助，全能搜索的界面、语音和脚本执行，电脑适配的窗口、DOM 和抛表情，牌堆随机点数，金币的画布、音效、提示、CSS 和粒子，以及段位制裁的将池不筛、烧条台词和调用方分数，以及 SJ Settings 不请求网络、不 eval update.js、不打开 PDF、不启动 eruda、不调用 game.reload 或 game.loop，以及手杀全扩不播放奶龙音频、不画界面、不调用 clsPrefixName，以及斗转星移不打开菜单、不播放音频、不调用 game.reload、不 eval，以及更新保护不请求网络、不写盘、不 eval、不睡眠，以及笨蛋插件不画界面、不播放音频、不采样 Math.random、不执行已关闭的选择卡牌优化，以及武皇争霸不画界面、不采样 Math.random、不播放录像、不执行已注释的双将体力公式，以及拖拽读取不 eval 单文件 extension.js、不请求网络、不读写文件、不启动 Electron，以及 boss play 包不读写 localStorage、不加载模式、不画界面，仍为登记差异。U01 刘冲末焱已映射，其余 30 将与 56 条技能引用未实现，不计整包，[证据](operations/evidence/2026-10-17-t12-u01-moyan/README.md)），仍需处理 13/37**（2026-10-18 NC11 勘正：整包数字以[待做登记表机器字段](operations/evidence/2026-10-16-todo-register/todo-register.json)为准——tags 含「整包」的 37 条中 `resolved` **24** / `open` **13**（余 U01、U05、U07、U08、L08、L11、L12、L14、L19、L20、L25、L27、L28）；本行原写「20/37、仍需处理 17/37」及台账的 18/19/37 都是 2026-10-17 批次快照，彼此矛盾，已按机器字段统一。**24 仅为「已登记为整包」的标签计数，不是整包验收通过数**：方案阶段 5 要求功能/配置/依赖/组合/保存恢复/目标平台/规则差异逐项通过，[T9 证据关系登记表](operations/evidence/2026-10-17-t9-source-samples/README.md) 实测 24 个标签行中 19 行仍在行内登记未迁移的功能/表现/平台面，故当前结论为**整包验收未完成**） | [逐项重构台账](migration/upstream-extension-reconstruction.md) 的全部功能、子包/插件、依赖、组合与目标平台验收；子包卡牌收官不等于来源整包完成 |
| 扩展包生命周期 | 第 17 轮注册表已拒绝未安装包启停/卸载及重复安装，冲突返回可处理错误；扩展集合内容锁已接真实建局/恢复。两仓**原始目录**仍不能被 `discover_extensions` 直接发现；转换产物已在 `packages/extensions/` 逐步建立 | 补全真实来源转换、升级、在局卸载、失败回滚及依赖/组合验收，不破坏进行中的内容锁 |
| 独立逻辑扩展 | G1 真实包已有卡牌/武将与规则宿主样本，可发现、导入、建局、触发选择、保存恢复；具体切片与数量见[台账](migration/upstream-extension-reconstruction.md)，内核牌种与宿主行为仍为封闭枚举 | 将剩余来源与子包按规则差异重构；证明无需重编译客户端的技能/卡牌/模式能力及重放、组合、故障恢复；不可用示例测试名代替整包验收 |
| AI/配置/资源/界面扩展 | 必须保留，不能整体归为不可迁移 | 合法信息视图、配置保存、资源/显示/交互接口；平台实际集成按另轨安排 |
| 旧 JS 扩展兼容 | 不承诺零改动运行（2026-10-12 用户决定 D2，[ADR 0782](adr/0782-extension-backend-and-legacy-js-scope.md)） | 37 来源按能力重构并逐项验收；第三方扩展经新 SDK（包数据 + rule-vm）发布；兼容层不立项 |

详细契约、旧线依据及 E01–E06 待办见 [扩展重构台账](migration/upstream-extension-reconstruction.md)。

## 范围 C：后续验收

- 存档未知牌种编号与载荷读尽已于 2026-10-07 加固，回归 `archive_hardening_rejects_unknown_kind_and_trailing_bytes` 随全量运行，2026-10-10 通过。2026-10-06 的[诊断探针](operations/evidence/2026-10-06-native-progress-audit/archive-probes.log)只描述当时源码。已完成的 v60 golden、目录和 tag 修复不再列为当前待办。
- E01–E06 与两仓 37 个来源整包重构继续；现有 G1 样本和卡牌切片只证明相应路径。
- 标准将/牌/模式、KOF 一致性、规则差异、扩展组合与 AI/客户端边界继续按[进度主稿](architecture/native-progress.md)逐项验收；正式客户端、目标平台和发布性能须有独立证据。

优先级、复现与完成条件只在 [进度主稿](architecture/native-progress.md) 维护，避免清单与 ADR 各自关闭同一未验项。
性能数据见 [phase19](operations/phase19-native-baseline.md)；17 案例、14 golden 的边界沿用各机制行。

## 范围 D：旧线（`engine-core`）映射与口径警告

- 内容级（5,595 个 `.skill` 文件）的旧线状态以
  [content-fidelity-ledger](content-fidelity-ledger.md) 为准：其 kind 轴
  （todo/approx/complete）按**注释文本**分类，noop 轴按
  `INTENTIONAL_EMPTY_NOOP` 标记计数——**「无近似注释」≠ 语义验证通过**，
  故旧线 ledger 的 complete **不**映射为本清单的「已验证」。
- 旧线自有可执行证据（golden/conformance/差分等套件）只证明**旧线**行为，
  不随内容自动迁移到原生线；迁移时须在原生线重立测试与场景。
- 旧线 AI 基线（engine-ai）当前不满足信息隔离条件（见 ADR 0703 背景），
  不进入本清单任何「已验证」行。

## 维护约定

1. 新机制落地：先列入范围 C（缺失）→ 实现 + 测试 + golden 场景 →
   改判「已验证」并回填证据列；场景名必须 `replay:` 前缀登记。
2. 任何 golden 变更须先说明原因：区分规则语义、状态编码、目录和脚本变更，
   在行为差异表或 ADR 登记后再更新哈希，不能仅重录哈希来消除失败。
3. 本清单只回答「规则机制与内容的覆盖状态」；性能口径见
   [operations/phase17](operations/phase17-test-perf-infrastructure.md) 的基准记录。
4. **golden 重录触发条件**除语义变化外还包括：牌库目录扩容（洗牌序列随之改变，
   2026-09-15 五次入线（装备/铁索与属性伤害/AOE/决斗/无懈与延时锦囊）各触发
   一次全量重录，现 14 个场景；其中判定区批次的判定区字段与跳阶段旗标入哈希，
   属编码扩展双重触发）、**状态哈希编码扩展**（新增状态字段入哈希，如铁索 `linked` 与伤害
   `nature` 编码）、场景脚本自身调整。
   重录须在本文件登记原因，禁止无说明改哈希。
   2026-09-16 全量重录（14 个 golden）：规则版本入哈希（全部场景）+
   挂起机器快照入哈希（`rollback_invariants` 挂起终态）；第七批时 36 项机制测试通过，
   不从重录后的哈希一致推导所有规则语义不变，决策见 ADR 0704。
   2026-09-16 第二次全量重录（14 个 golden，STANDARD_RULESET_VERSION 1→2）：
   语义变化见 ADR 0705（逐目标无懈、铁索/判定链无懈窗口、同名判定牌限制、
   身份局胜负与奖惩、弃牌责任调整/铁索离手/死亡清当前判定区；当时剩余 N01/N02，已于第九批修复）；
   `chain_fire_propagation`/`judge_phase_delayed_tricks` 场景布阵调整
   （清全部手牌，避免随机无懈持有者被窗口询问触发重问）。
   2026-09-16 第十一批再次全量重录（14 个 golden）：每段预算重置改变摘要，
   同批调整无懈询问起点、延时锦囊开窗时点、闪电去向并以 PlaceDelayed 替换 JudgeTrick；
   决策见 ADR 0708；该批时版本仍为 2，第十二批已补做递增，重录不是规则正确性的独立证据。
   2026-09-17 第十二批全量重录（14 个 golden）：STANDARD_RULESET_VERSION 2→3，
   补登记第十一批语义/机器编码变化（ADR 0709）；该批段检查点尚未入快照。第十三批已补检查点保存/恢复（ADR 0710），规则版本仍为 3、14 个 golden 未重录；N07-b 以行为回归与探针关闭。
   2026-09-17 第十四批：14 个 golden 未改且通过，但判定从尾部改队首已改变混合牌序语义，
   STANDARD_RULESET_VERSION 仍为 3、版本登记未完成；分组也未实现。补齐后按版本约定更新 golden，见 ADR 0711。
   2026-09-17 第十五批全量重录（14 个 golden，STANDARD_RULESET_VERSION 3→4）：
   判定排序分组落地（非闪电放置序在前、闪电在后、队首消费，ADR 0713），
   分组改变混合牌序语义；双放置序锁定测试同批落地。同轮版本 4 还包含 ADR 0712 的
   鬼道/天妒目录追加及 JudgePhase.current 快照/摘要编码扩展；技能语义未据此自动验收。
   2026-09-18 第十六批全量重录（14 个 golden，STANDARD_RULESET_VERSION 4→5）：
   天妒询问范围收窄为判定者本人，鬼道由弃置旧判定牌改为改判者获得（ADR 0714）。
   当轮独立复验 14 golden 通过；N04-d 拒绝回归不代表完整恢复关联已验收，N04-e 当时开放、后在 ADR 0715 关闭。
   2026-09-20 第二十一批后复验：14 golden 未改且通过，版本仍为 5；宿主判定日志不入摘要，
   但装备改判选项/牌区语义已有变化，须补递增版本及 golden 变更说明。合法装备恢复和事件反例见当前进度。

5. 2026-09-22 历史复核：当时规则版本 **24**，14 golden 通过；版本 6–24 的分批语义与编码变化见
   [规则基准](architecture/native-rules-baseline.md) 和 ADR 0720–0763。
   本轮未重录 golden、未改代码；新增反例及内容缺口不会因现有摘要一致而自动关闭。
