# 第 0 项最终独立复核（2026-11-13）

- 复核人：`queue0_independent_review`；源码 HEAD：`3a6e398485db6ef0729abe3013c3c4b7705a5d6d` + tracked dirty；规则 333／NGSA 107／content schema 5／ABI 759／WS8。
- 工具链：cargo 1.99.1／rustc 1.99.1。最终核对的 25 个文件哈希、HEAD 对比与工具链原文见 [final-source-state.json](final-source-state.json)，实际 tracked diff 见 [final-reviewed-tracked.diff](final-reviewed-tracked.diff)，新增回归源码快照见 [final-regression-source.rs.txt](final-regression-source.rs.txt)。
- 本 agent 未参与生产实现，未读实现者说明或本轮实现证据 README；依据上游与原生源码、实际 diff、测试和原始运行输出形成结论。编译按主 agent 协调使用默认 target，独立定向结束后已释放；未修改生产／测试、未运行 rustfmt、未提交或推送。

本批已审变更没有新增阻断：暗织 20 项独立定向回归全部通过，前端守卫 7 项独立回归通过，Fz 同值登记与正式 U01 装载构成符合实际调用链。暗织和 Fz 均保持「简化近似」；本结论只覆盖第 0 项所列修复，完整三 crate、产品门禁、14 场 golden、发布与 CI 由主 agent 另行留证，不能用本次定向结果代替。

## 实际执行与证据边界

| 独立核验 | 实际结果 | 原始证据 |
| --- | --- | --- |
| 仓库根目录：`cargo test --offline -p engine-native --test g1 u01_dddanzhi_regression -- --nocapture` | 20 passed／0 failed／0 ignored，221 filtered；2026-11-13 21:29:58–21:30:41 +08:00 | [命令与时刻](final-targeted-command.json)、[完整输出](final-targeted-test.raw.txt) |
| `apps/web-client`：`node --import ./scripts/ts-register.mjs --test scripts/approved-b-metadata.test.mjs` | 7 passed／0 failed／0 skipped | [完整输出](approved-b-metadata-test.raw.txt) |
| 独立数组内容／正则结果对照、U01 上游／原生名称集合对照 | 6 个注册模块，3,136 个唯一提示键，遗漏 0；64／64 技能名称集合双向差集为空 | [复跑脚本](static-comparison.py)、[最终输出](final-static-comparison.json) |
| 上游浏览器实际触发 | **0 项**；只做源码比对 | 本文上游依据与下表；不作 U01 整源或 13 来源等价签署 |

前述 Cargo 命令明确是定向复核，含过滤；不冒充用户要求的无过滤完整三 crate 命令。整合过程的首轮与第二轮退回记录保留在 [static-review-round1.md](static-review-round1.md)、[static-review-round2.md](static-review-round2.md)。实际曾失败的原始输出也保留，例如 [18 项时 12／6](../dddanzhi/green-final-attempt.log)、[20 项时 17／3](../dddanzhi/green-final-rerun.log)、[20 项时 19／1](../dddanzhi/green-final-fixtures.log)；这些运行不改写为全绿。

## 暗织：上游条件与最终源码

上游 [skill.js](../../../../../../archive/upstream/noname/apps/core/extension/3D精选/character/skill.js) 的 `dddanzhi`（5796–5839）在全局真回合 `phaseBegin` 检查最后一次 `useCard`／`respond` 的颜色；无历史不发动。可选任一目标，取消不授予效果；只给接收者临时技能，第一次成为黑色牌目标时移除该目标并消费效果。全局历史在 [gameEvent.ts](../../../../../../archive/upstream/noname/apps/core/noname/library/element/gameEvent.ts) 的 `start()`（225–234）写入；临时技能默认到 [player.js](../../../../../../archive/upstream/noname/apps/core/noname/library/element/player.js) 的 global `phaseAfter`／`phaseBeforeStart`（11957–11969）到期。上游用牌流程 [content.ts](../../../../../../archive/upstream/noname/apps/core/noname/library/element/content.ts) 的 `useCardToTarget`（10220）发生在目标追加相关的用牌触发后。

最终原生选择代码在 [u01_batch_d.rs](../../../../../crates/engine-native-content/src/content/host/step/u01_batch_d.rs)，条件／问询在 [u01_batch_d_asks.rs](../../../../../crates/engine-native-content/src/content/host/step/u01_batch_d_asks.rs)。共同历史在 [kernel/events.rs](../../../../../crates/engine-native-kernel/src/kernel/events.rs) 的 `record_vm_history`／`record_dddanzhi_card`；机器牌的最终目标检查在 [kernel/intrinsic.rs](../../../../../crates/engine-native-kernel/src/kernel/intrinsic.rs)。独立复跑的 20 项均来自 [u01_dddanzhi_regression.rs](../../../../../crates/engine-native/tests/u01_dddanzhi_regression.rs)，并已登记到 `tests/groups/g1.rs`。

| 回归函数（`dddanzhi_` 前缀省略） | 独立核对的条件／分支及最终结果 |
| --- | --- |
| `discard_without_history_and_last_red_do_not_trigger` | 仅弃牌无历史不问；先黑后红即使弃牌堆非空也不问，原「或弃牌非空」条件已去除。 |
| `last_response_color_overrides_used_card_color` | 红决斗后黑杀打出可问；黑决斗后红杀打出不问，末次 respond 覆盖 use。共同记录不依赖当时技能持有／启用。 |
| `target_choice_cancel_and_pending_archive_are_distinct` | 选自己、A、B、取消分别执行；仅所选目标免疫，选他人不再附送自己。选择挂起归档恢复后，开放选择、归一化 auth_generation 后全快照及续行日志一致。 |
| `only_first_black_target_is_immune_and_red_does_not_consume` | 红杀正常伤害且不消耗；首张黑杀只消费免疫、实体牌正常弃置；第二张黑杀正常伤害。 |
| `ward_expires_at_turn_end_and_survives_grantor_skill_loss` | 授予者技能被关闭后，接收者已授效果仍有效；回合结束后无效，独立接收者归属正确。 |
| `black_multitarget_card_only_loses_the_selected_target` | 南蛮仍伤害未受保护者，铁索仍横置未受保护者；只移出所选目标，整牌实体正常回收。 |
| `black_implicit_self_target_is_consumed_and_counted` | 黑酒与黑闪电的隐式自身目标可免疫；total_used 仍增加，酒效／判定区不生效，实体进弃牌堆；后续黑酒正常。 |
| `black_view_as_wugu_skips_only_immune_picker_after_restore` | 真实良策黑牌转五谷只跳过接收者；五谷亮牌持有态可归档恢复，最终状态与日志一致，载体准确回收。 |
| `pre_fix_pending_ngsa_retains_content_lock_and_resumes` | 修复前 2,149 字节 NGSA107 原档可 decode／encode 原字节往返；三个 content lock 断言保持原值；恢复时只增加 auth_generation，继续所选目标分支正确。 |
| `wugu_without_skill_restores_held_reveal_cards` | 不加载暗织的普通五谷也能恢复亮牌持有态，状态／选择／续行日志一致；所有角色正常拿牌、五谷实体正常回收。 |
| `no_ward_virtual_group_subset_keeps_global_recipients` | 真实 VM 发 targets=[self] 的虚拟五谷／桃园，未消费暗织时保留旧机器全体／其他存活角色语义。新增 retain 由本事件确实消费免疫的标记限定。 |
| `final_targets_include_after_trigger_additions` | 两方向均测：先指定非保护者再追加保护者；先指定唯一保护者再追加非保护者。最终都仅未受保护者受伤，探针确实 SkillTriggered，牌正常弃置。 |
| `actual_equipment_use_updates_last_color` | 真实 PlayCard 装备的黑／红末色都覆盖此前相反色；装备仍正常入槽。 |
| `direct_equipment_move_neither_changes_history_nor_consumes_ward` | VM 直接从牌堆强制装备不写使用历史、不消费免疫；随后黑酒才消费。区分实际使用与移入装备区。 |
| `nullified_equipment_keeps_old_slot_and_has_no_equip_after` | 黑装备首次无效保留旧槽、不授新装备技，不伪发 Equipped／劫行重置；仍写黑色末牌史。再次使用同牌正常入槽、授技和触发劫行。 |
| `nested_use_and_response_do_not_get_overwritten_by_parent` | 外层红决斗后奇佐黑色副本／黑杀打出，两条末牌都保持黑。旧 CardUsed 追色分支现为 no-op，未改其触发元数据／内容锁。 |
| `sibling_duels_record_execution_order_instead_of_frame_ids` | 真实 wn_dutan 多目标决斗逆序压栈，先场黑色 respond 后末场无色 use 仍覆盖为非黑；已去除「最大创建 frame ID」伪顺序守卫。 |
| `black_fengze_taoyuan_restores_nullify_window_and_carrier` | 凤泽黑牌转桃园，无懈窗口有／无无懈两分支都可恢复；只跳过保护者，其他原生目标正常回血，载体不再按原牌种漏回收，临时目标标记清净。保留现有桃园不回自身模型。 |
| `protects_phase_end_card_until_phase_after` | phaseEnd 内黑色嵌套用牌仍有免疫；phaseAfter 内已到期。只在发出 phaseAfter 前清两暗织旗，不再被通用 clear_turn 过早清除。 |
| `wugu_pending_init_restores_and_nullification_cleans_marker` | 五谷尚未亮牌的 Init 被无懈窗口包裹时可恢复；有／无无懈均状态及日志一致；结算／无效后 dddanzhi_targets 标记清净。 |

机器牌的检查点在 AfterTriggers 完成后、Intrinsic 建机前；不再在 Commit 永久置 intrinsic_done，故两个追加目标方向均能结算。原生立即生效牌继续在既有 Commit 处检查；保持原用牌计数、历史与其 AfterTriggers 流程。装备保留原 Equipped 路线；实际 PlayCard 前先记录末牌，并用逐帧标记区分直接／强制装备。被免疫装备的 Commit 明确置 Done，调度器保留 Done，故不产生虚假 equipAfter。

桃园收尾现在在 `step` 入口先捕获当前 CardUse 的实体载体，再创建 recover 子帧；五谷也按当前 CardUse 精确回收，不再扫描目录原牌种。最终 [diff](final-reviewed-tracked.diff) 已核对纯空白收口：equipment 无关旧格式改动已撤、events EOF 无新增空白；没有用重新格式化掩盖逻辑改动。

## 归档与恢复校验

[restore.rs](../../../../../crates/engine-native-kernel/src/snap/restore.rs) 新增五谷／桃园的既有 snapshot 形态分派，没有新增字段或 tag；五谷空 cards 恢复为 Init，有亮牌时恢复为 Pick。[asks.rs](../../../../../crates/engine-native-kernel/src/snap/asks.rs) 从非空 cards 与首个 picker 重建既有 `wugu_pick` 询问。[validation.rs](../../../../../crates/engine-native-kernel/src/snap/validation.rs) 将 Wugu.cards 纳入暂离持牌枚举，其原有 NullifyWindow／其他嵌套递归继续生效。

恢复没有跳过 ID 校验：[machine_validation.rs](../../../../../crates/engine-native-kernel/src/snap/machine_validation.rs) 的 Taoyuan.owner 用 check_pid，Wugu.owner／cards／pickers 分别用 check_pid／check_card／check_pids（345–356），NullifyWindow 对 effect 递归 check_machine（139）。`validate_resolution` 对每个 park 调用该校验；[engine/snapshot.rs](../../../../../crates/engine-native/src/engine/snapshot.rs) 在重建机器前先校验 core、牌归属和 resolution（67–71）。持牌归属检查与上述 ID 检查分别保留；未把原先 UnknownMachine 改成接受越界 ID 后运行。

旧档 fixture 与 [修复前 pending.ngsa](../dddanzhi/before/pending.ngsa) 字节相同，SHA256 为 `3158038e27eb46cffd15b4fc1eb4e378daa381cc9a01e16b96ef8f6656178736`。独立测试断言 d3／rest／合并 content lock 分别为 `1553573260256134951`／`9097112637251845257`／`11413175519070831972`，并实际恢复续行。此证据覆盖这一旧档及所列机器分支，不代表所有历史存档都已验收。

## 完整门禁发现的 Adv21 增量

主 agent 首次运行完整三 crate 时，[原始输出](../04-native-three-complete.log) 的 e2e 组为 112 passed／1 failed；失败旧项 `test_t5_adv_21_empirical_reproduce_wugu_restore_defects` 强制要求 seed 10148 在 40 步内复现 UnknownMachine 或 card_missing_from_all_zones。该预期与本次真实修复的 Wugu 恢复分派和持牌枚举相冲突；首次失败与[旧测试源码](../04-wugu-diagnostic-before.rs.txt) 均保留，不能记成完整绿灯。

已只读审查主 agent 修改的 [e2e_tier5_adversarial_c1.rs](../../../../../crates/engine-native/tests/e2e/e2e_tier5_adversarial_c1.rs)：新 `test_t5_adv_21_wugu_restore_preserves_trajectory` 保留 seed 10148／40 步，两会话分别连续运行与逐步 NGSA 恢复，必须观察到 Wugu 挂起（含 NullifyWindow.effect 内嵌），每步比对 state_hash 和仅归一 auth_generation 后的完整快照，因此包含 RNG、选择、帧和机器字段，改为成功不变量是正确验收标准。无关整文件 rustfmt 已撤，当前只留 Adv21 hunk。此版主 agent 的[定向输出](../04a-wugu-restoration-regression.log)为 1 passed／0 failed／0 ignored，112 filtered；本 agent 未占 target 重跑它，不计入独立 20 项结果。

审查 [e2e_harness.rs](../../../../../crates/engine-native/tests/e2e/e2e_harness.rs) 又发现 `auto_step()` 会把 CommandOutcome::Rejected 也包在 Ok 中，因此只 `.expect()` 不能证明每步实际被接受。已要求新测试进一步断言两会话每步 Accepted／Pending 或明确 revision 递增；主 agent 已承接，最终新增断言的源码与运行结果须另行补记。该中间输入的 diff／哈希见 [adv21-source-before-acceptance-strengthening.diff](adv21-source-before-acceptance-strengthening.diff)、[核对记录](adv21-source-before-acceptance-strengthening.json)。上述 25 个原始复核文件未因该测试调整而改变。

2026-11-13 最终增量复核：现有代码已逐一对两个会话的每步 outcome 显式断言 `Accepted`／`Pending`，拒绝动作不能再被 Ok 包装掩盖；仍只有 Adv21 一个 diff hunk。独立静态核对全部必要条件为真，见 [最终 diff](adv21-final-review.diff)、[源码哈希与检查](adv21-final-review.json)。主 agent 的[加强后定向原始输出](../04c-wugu-restoration-regression.log)为 1 passed／0 failed／0 ignored、112 filtered；本 agent 只读核对该输出，未另跑 Cargo。此增量已无上述测试强度问题，且前述 25 个文件哈希全部未变，原独立 20 项验证仍适用。最新完整无过滤重跑结果由主 agent 另行登记，不能由该 1 项定向推断。

采集记录勘误：首次「before-acceptance-strengthening」证据保存时，主 agent 已并发写入加强断言，因此其源码哈希／diff 实际与最终版相同。该首探针用一段不存在的提示文案检测断言，产生的 `requires_command_accepted_or_pending=false` 不成立，不能用于证明当时保存的源码缺断言；原始文件保留，纠正依据及最终真实表达式检查见 [adv21-capture-correction.json](adv21-capture-correction.json)。此前缺断言的反馈来自更早只读源码输出；最终结论以实际 `matches!` 和两会话循环为准。

## Fz 登记、安装断言与提示词守卫

Fz 的最终 [source_order.rs](../../../../../crates/engine-native-kernel/src/vocabulary/host/metadata/source_order.rs) 仅补 `(Normal,1,false)`，与修改前 common 元数据、真实 L20 命名委托相同。两个 [C331 元组守卫](../../../../../crates/engine-native/tests/c331_source_order_phase5.rs)／[全词表守卫](../../../../../crates/engine-native/tests/c331_source33_integration.rs) 分别新增同值 Fz 条目、把实际带触发 HostBehavior 计数从 299 更正为 300，没有放松缺登记断言。调用链和指纹恒等依据见 [fz-selection-boundary.md](fz-selection-boundary.md)。本 agent 未另跑 C331／14 场 golden；不以静态恒等替代主 agent 的完整兼容门禁。

上游 fz_liegong inherit 的 liegong 实际默认 `(Normal,0,true)`，玩家选序差异保留，清单已下调。实际技能是目标手牌数≥持有者体力或≤攻击范围时可确认令杀不可响应，shouyue 增范围；没有伤害+1分支，已纠正旧清单的混入描述。

[game-server 测试](../../../../../apps/game-server/tests/native_product_path.rs) 保留正式安装／装载／建局／选将路径，把过时构成断言改为 64 host、0 VM，并比较完整 host 名称集合与 `U01_DDD_SKILLS`。独立将该数组与上游 skill.js 的 64 个顶层 ddd 键比较，双向差集为空；实际 loader 优先按名原生 host 绑定。这里证明预期构成和回归断言覆盖范围，不证明 64 项规则等价；本 agent 未另跑 game-server Cargo 测试。

[前端测试](../../../../../apps/web-client/scripts/approved-b-metadata.test.mjs) 现读取所有真实专用 `known_prompts*.rs`，用 `/"([^"]+)"\s*,/g` 收集同一行多个值，且断言注册值非空。首版修补的行首正则只取 2,703 个唯一值，合并生产 prompt 字面量后仍漏 206 个注册键；最终独立按数组内容核对 3,136／3,136、遗漏 0。旧问题与修后结果同时保存在 [static-comparison.json](static-comparison.json) 和 [最终复算](final-static-comparison.json)，并有上述 7／0 独立测试。

## 保留差异与清单处置

| 剩余边界 | 源码事实及处置 |
| --- | --- |
| 暗织触发时机 | 上游 global 真回合 phaseBegin；原生 `u01_batch_d_asks.rs` 仍匹配 Prepare 的 PhaseBegin。TurnBegin 与 Prepare 之间其他事件的组合尚不等价，未改元数据。 |
| 暗织／Fz 选序 | 原生仍 Normal／priority 1／不可选序，上游有效 Normal／0／可选序。Fz 本批只同值登记；真实纠偏会改变同人并列触发轨迹，应按用户要求先报边界。 |
| 原生桃园目标模型 | 上游 `card/standard.js` 的 taoyuan（1265–1284）不排除使用者；原生 TaoyuanMachine 与免疫目标推导仍只含其他存活角色。此次保持既有模型并明确测试，未签等价。 |
| 实体 Duel CardId=0 | `kernel/duel.rs` 的 finish 把 0 当虚拟牌哨兵，跳过弃牌／table 移除；`intrinsic.rs` 的无实体决斗也传 0，实体 ID=0 会与其冲突。该 duel 文件与 HEAD 完全相同（哈希见 final-source-state），[整合原始失败](../dddanzhi/green-final-rerun.log) 中的载体回收断言保留。主 agent 已明确将此既有字段／哨兵边界登记待办，本次未改 Duel 字段、存档或版本；20 项最终绿灯不代表此边界已修。 |
| 上游实际触发／来源签署 | 本轮实际触发 0；未做 U01 模板表全部重审或 13 来源浏览器对照，不对这些范围签等价。 |

唯一暗织项现已把「修复执行中」描述更新为本次实际核验及以上剩余条件，状态保持「简化近似」。写入前重读清单，逐对象确认另 4,423 项未变，counts 重算未变；证据见 [最终描述改动核对](final-inventory-description-check.json)。Fz 先前下调的原文及暗织原文分别保存在 [fz-inventory-before.json](fz-inventory-before.json)、[ddd-inventory-before.json](ddd-inventory-before.json)，执行中记录仍保留。

| 本次清单快照 | 已复核等价 | 简化近似 | 缺失 |
| --- | ---: | ---: | ---: |
| 全仓 | 849 | 3,575 | 0 |
| U01 | 94 | 1 | 0 |
| L20 | 91 | 4 | 0 |

这些计数只表示当前清单状态。本批截至本报告仍未由本 agent 提交／发布；队列第 0 项的完整门禁、exact-head CI 与后续队列进度由主 agent 的原台账新日期记录说明。

## 2026-11-13 22:00 阶段登记表增量

独立只读核对了 `report_phase_filters.py`、自测和新旧 registry；当前 37 行中的五个新增阶段分支均来自既有 HEAD，旧日期文件及原 32 行状态／依据逐字保留，新增均「未核」，没有增加已核或规则等价认证。独立轻量自测 9／0／0，`--check` 为 37 条一致。具体条件、来源提交、历史字节哈希和原始命令见 [增量复核](phase-registry-incremental-review.md)；本增量未占 Cargo target，未重跑暗织 20 项，未写生产／测试／registry／inventory。

## 2026-11-13 22:12 once／伤害枚举脚本增量

独立确认实际 host_skill_limits 中 65 个直接 once 声明及酒诗既有 Damage 臂导致枚举 88→89。首版新解析的三个块注释反例已退回修正，最终先剔除嵌套块／行注释再定位函数，独立 6 项及三原始反例重放全部通过；伤害枚举自测独立 15 项通过，相关输入未变。具体源码因果、前后记录、文本扫描范围和原始命令见 [once／事件枚举增量](once-event-incremental-review.md)；未修改生产、测试、清单或版本，未占 Cargo target，未将上述定向结果称为完整 preflight 或规则等价。
