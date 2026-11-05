# R05-E 批验收报告：三固定 compound（涕泣/宝梳/避祸）＋四角色 helper 派生

- 日期：2026-11-03
- Worktree：`/private/tmp/noname-r05/worktree`（分支 `r05-private`，起点 HEAD `386d8611`；期间并行批次追加了 docs-only 提交 `0609e494`/`90711a67`，非本批改动）
- 合同：裁决 28 → proposal-v3 §2（身份/四角色/marks 隔离）＋ §5（三技能完整分支）
- ABI 语义来源：上游 noname ABI 759（tiqi/baoshu/dcbihuo）；效果标签取未占用号 781/782/783（`rg` 全仓确认无占用）
- 测试统计：新文件 `crates/engine-native/tests/r05e_compounds.rs`（登记组 misc）14 通过 / 0 失败
- 全量：`cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast`
  = 1930 通过 / 1 失败 / 75 忽略（基线 1916/1/75 ＋ 本批 14；唯一失败 `replay_scenarios::replay_scenarios_match_golden`
  ——基线同败：stash 前后 golden 失败集合逐字节一致（14 个场景 mismatch 完全相同），非本批引入）
- 严格警告门（`RUSTFLAGS="-D warnings"`、独立 target `strict-e`）：错误集合与基线**完全一致**
  （仅既有 `kernel/events.rs:738` unused doc comment，由 `386d8611` 引入，非本批文件）；本批零新增警告
- 行数门禁：`check-source-lines.py` 报告与基线一致（既有 `machine_encode.rs`/`kernel/events.rs` 超限，
  均非本批文件）；本批触达文件全部在限内（loader 有效行 950/1000）

## 实现点（file:line 为提交时行号）

| 环节 | 位置 |
| --- | --- |
| 标签常量 781/782/783 | `crates/rule-vm/src/bytecode/effect_tag.rs:1617-1630` |
| 弹栈解码臂（player, skill 双弹栈，同胆守形） | `crates/rule-vm/src/vm/effects/resolve_potjixi_packs.rs:361-386` |
| EffectRequest 变体（choices 宏表尾追） | `crates/rule-api/src/effect/choices.rs:1517-1553`；注册表尾追 `crates/rule-api/src/effect.rs:703-707` |
| 入队校验 gate 臂（player 在座＋skill 非 NONE） | `crates/engine-native-kernel/src/rvm_ops.rs:126-140` |
| 内容机 dispatch（step/active_ask/validate）＋共享基建（helper 名模板、marks 保零加法、窗口读取、helper 判定、通用 AddMark/RemoveMark 复检拒绝） | `crates/engine-native-content/src/content/vm_effects.rs`（dispatch ~48-137；R05 基建 ~97-260；validate 尾段） |
| 涕泣状态机 `[stage,n,target+1]` | `crates/engine-native-content/src/content/vm_effects/tiqi.rs` |
| 宝梳状态机 `[stage,m_bits,k,index,targets...]` | `crates/engine-native-content/src/content/vm_effects/baoshu.rs` |
| 避祸状态机 `[stage,mode]`＋take 面 | `crates/engine-native-content/src/content/vm_effects/dcbihuo.rs` |
| 四角色 helper 派生（模块扫描＋模板追加）＋触发面事件名映射 | `crates/content-native-loader/src/lib.rs`（`derive_r05_helpers`/`r05_module_compounds`；事件映射 damage_source/damage_end/draw_phase_after/skill_log_begin） |

## 分支矩阵（14 测试）

涕泣（781）：
- 正常：他人 D 终窗 live n=0（双枯竭）→确认→真实 start_draw(d=2)+Yield→目标在场选 −d→Limit=−2
  （`r05e_tiqi_confirm_draw_and_negative_limit`）
- 确认取消：无符号问、无槽（`r05e_tiqi_cancel_confirm_and_cancel_sign_keep_no_slot`）
- 符号取消：不撤已摸牌、不写槽（`r05e_tiqi_sign_cancel_keeps_drawn_and_writes_nothing`）
- n==2 与自己的 D 均静默 no-op（`r05e_tiqi_n_equals_two_and_own_draw_are_noop`）
- 身份隔离：两包各写各的 Limit 槽（def 不同、值互不串）（`r05e_tiqi_two_packages_do_not_share_slots`）

宝梳（782）：
- 正常：成本 k=2（含自己）→座次逐目标 仅当前 linked 才解链、仅当前 face_down 才翻回→装 helper、
  m=3 各 +3（`r05e_baoshu_cost_links_turns_over_and_combs_positive`）
- 成本取消：无 marks、不改横置（`r05e_baoshu_cost_cancel_writes_nothing`）
- m==0→+1、窗口间 maxHp 改写（skill_log_begin 双 handler 各 −1）后 m=−1→负保留、两轮 +1+(−1)=Some0 保留
  （`r05e_baoshu_m_zero_and_negative_via_inter_window_max_hp_change`）
- 身份隔离：两包 owner 各写各的 Combs（`r05e_baoshu_two_packages_do_not_share_slots`）

避祸（783）：
- Source −1＋End +1 同目标→Next=Some0 保留→目标回合开始原子 take：Next 移除、Current=Some0、装 helper
  （`r05e_dcbihuo_source_minus_end_plus_keeps_zero_then_take_moves_current`）
- End 取消：仅 Source −1→take 写 Current=−1（`r05e_dcbihuo_cancel_end_window_keeps_minus_then_take_moves_minus`）
- 身份隔离：两包×两窗各写各的 Next；take 各自迁移（`r05e_dcbihuo_two_packages_do_not_share_slots`）

门禁与派生：
- 通用 AddMark 碰派生 helper 键：命令被 Fault 拒绝、键未写入（`r05e_generic_add_mark_on_helper_key_is_rejected`）
- 派生确定性：同包两次桥接同名同序；tiqi→[limit]、baoshu→[combs]、dcbihuo→[next,current]；
  无 compound 包不派生（`r05e_loader_derives_roles_deterministically`）

## 登记差异与未尽项（按 v3 原文核对）

1. **codec 未覆盖（最重要）**：`rvm_ops/codec.rs` 不在本批允许文件清单内，三 compound 无 enc/dec 臂。
   后果：compound 机挂起（commit_pending 非空）时调用 `engine.save()`/恢复/AI 规划会命中
   `enc_effects` 的 `unreachable!()`。本批测试均在机器完成后才存档，未触发。后续批需按胆守 tag 30
   同形补 33/34/35 三对臂（机械改动）。
2. **rvm_bridge owner/skill 绑定检查未覆盖三 compound**：`rvm_bridge.rs:1057-1069` 的
   player/skill==ctx 绑定拒绝只列既有三变体（该文件不可改）。本批 gate 只校验 player 在座＋skill 非
   NONE；模块可为他人发 ResolveX。随 codec 批一并补臂。
3. **logSkillBegin**：未新增事件类型（按任务不做）。宝梳用既有 `EventData::SkillLog` begin/end 两帧
   近似子结算＋yield；装载面补 `skill_log_begin` 事件名映射承载窗口触发（测试用它实现 maxHp 改写反例）。
4. **Limit/Current 到期**（全局 phaseAfter/BeforeStart 清理）、**Combs Begin2 消费**、**死亡清理/revive
   保值**（§6 全表）：需 kernel 回调或第 4 个效果标签，均不在本批可改面——随 F 批生命周期接线。
   本批 Limit 槽只做有符号累加/保零；最终手牌上限应用也未接（kernel 弃牌上限路径不可改）。
5. **Next 读取的 enabled 门**：take 面按「marks 条目存在＋目标存活」判定；helper 实例 enabled/disabled
   逐格语义随 F 批（实例当前只会以 enabled=true 装载）。
6. **take 面**：任务文「目标真实 phaseBegin」与 v3 §7「Next 为 owner 真实 turnBegin」并存——装载器既有
   `phase_begin`→TurnBegin 映射统一了两者；take 在 TurnBegin 窗执行。
7. **D 终窗选择**：主 draw 张数>0 时 D 机以 R 子帧替代 End 窗发射（`kernel/draw.rs` 既有行为），恒发射的
   终窗是 After；涕泣装载事件名定为 `draw_phase_after`（End 窗仅在 0 张时发射，机器两面都接受）。
8. **每事件一次**：由触发面结构保证（每窗每技能实例一次入队）；未在机内做事件级去重。
9. **engine-api ABI 版本**仍为 758（engine-api 不在允许清单）；携带 compound 的包按 ≤host 声明 ABI
   （兼容窗 3..=host），无需 bump。
10. **helper 惰性面**：派生 helper 定义为空 Handlers（无 active、无 trigger、单 HALT 模块），仅承载
    `(I,role,target)` marks 键；武将授予清单不含 helper（由机器在目标上 ensure_skill）。

## 命令与结果（全文见 e-01.log）

- `CARGO_TARGET_DIR=/private/tmp/noname-r05/target CARGO_INCREMENTAL=0 timeout 1500 cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` → 1930/1/75（唯一失败＝基线 replay_scenarios golden，失败集与基线逐字节一致）
- `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=/private/tmp/noname-r05/strict-e cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native --all-targets` → 错误集与基线一致（仅既有 events.rs:738）
- `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` → OK
- `python3 -B scripts/check-source-lines.py` → 仅既有两文件超限（与基线一致）

## E 收尾补齐（主代理，2026-11-03，8454c3ca 之上）

- codec 33/34/35 臂（rvm_ops/codec.rs 编解码双向，胆守 30 同形）——修复挂起存档/AI 规划对新变体命中 `unreachable!()`；
- rvm_bridge owner/skill 绑定检查补三 compound 臂；
- 回归 `r05e_compound_effects_roundtrip_through_suspended_codec`（misc 组，15/15 含原 14）；
- 三 crate **1931/1/75**（唯余 golden，失败集与基线逐字节一致）。
- 子代理未尽项 3–7（Limit/Current 到期、Combs Begin2 消费、死亡清理、手牌上限应用、Next enabled 精确门、End 窗被 R 子帧替代的装载面）随 F 批 kernel 回调接线。
