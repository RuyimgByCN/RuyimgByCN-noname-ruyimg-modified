# R05-F2 批报告（2026-11-04；分支 r05-private，基线 `cf7c44d1`）

闭合 F 批（真包接入）报告的三个内核/内容语义缺口＋两个行数门禁拆分。先红后绿：红证 [f2-01.log](f2-01.log)（6 红＋1 守卫，HEAD `cf7c44d1`＋测试文件），绿证 [f2-02.log](f2-02.log)（三 crate 1948 通过／1 失败／75 忽略；行数门禁 exit 0）。

## 缺口1（核心）：dcbihuo Current 槽的 Begin2 内核消费点

- **上游对照**：`archive/upstream/noname/apps/core/character/sp2/skill.js:7828-7838`（subSkill sha）——`trigger {player:"phaseDrawBegin2"}`、`filter: hasStorage("dcbihuo_sha") && !event.numFixed`、content `trigger.num += num`（有符号，可负可 0；hasStorage 为存在性谓词）。
- **红断言**（`crates/engine-native/tests/r05f2_semantic_gaps.rs`）：
  - `r05f2_current_minus_consumed_at_begin2_signed`：真包 sp2 引擎布 `dcbihuo#r05:current`=−1 → 本人摸牌阶段实际摸 1（2−1），消费后条目与 helper 实例清除。修前红：摸 2、条目残留（f2-01.log）。
  - `r05f2_current_plus_consumed_at_begin2`：Current=+1 → 摸 3。修前红：摸 2。
  - `r05f2_current_not_consumed_when_num_fixed`（守卫面，修前自然通过、修复后防过度消费）：`num_fixed=true` 的 D（新测试载体 `Engine::debug_start_draw_phase_fixed`，`crates/engine-native/src/engine/debug.rs`；正式生产者随后续批次接入）按固定张数摸 2、Current 条目保留（上游 `!numFixed` 同门）。
- **修法落点**：`crates/engine-native-kernel/src/kernel/draw.rs` `DP::Begin2` 臂——Combs 消费循环后并列增加 `#r05:current` 消费循环：门 `!num_fixed`（与 Combs 同门同一步），`marks.remove` 取值（存在即消费，含 0/负），`pending += value` 有符号累加（先例＝Begin 前窗 `host_pending_adjust` 口径），清槽并移除 helper 实例（与 Combs 同款清理）。

## 缺口2：摸牌阶段跳过路径的恒发射终窗 After

- **上游对照**：`archive/upstream/noname/apps/core/character/xianding/skill.js:41912`（tiqi）——`trigger {global:["phaseDrawEnd","phaseDrawSkipped","phaseDrawCancelled"]}`，filter 非自己 D 且该阶段 gain 总数≠2。
- **红断言**：`r05f2_skipped_draw_phase_emits_after_and_tiqi_asks`——真包 xianding，seat0 持 xnd冯妤；seat0 出牌阶段给 seat1 判定区布兵粮寸断＋牌库顶置♥（跳过条件的真实生产者＝`kernel/judge.rs` `CardKind::Bing` 非梅花分支 → `turn.skip_draw_phase=true`）；seat1 摸牌阶段被跳过后 tiqi 收确认→符号两问（n=0≠2 → d=2 → +2 写目标 Limit），且目标未摸牌。修前红：跳过路径 D 帧直接 Intrinsic→Done 不发任何窗，无任何询问（f2-01.log `left: []`）。
- **修法落点**：
  - `crates/engine-native-kernel/src/kernel/phase/draw.rs:24-36`：跳过路径 D 帧改以 `progress: Before`＋`outcome: Skipped` 入机（原 `progress: Done` 无窗终止）。
  - `crates/engine-native-kernel/src/kernel/draw.rs` `DP::Before` 臂：预置终态（`outcome != Running`）只发恒发射终窗 After——Ended/Skipped 都落 `DrawPhaseAfter`（EventKind 由 `DrawWindowKind::After` 映射，触发面经 `plan_triggers` 派发），D 载荷 `outcome` 字段区分（Skipped 不被 `DP::After` 臂的 `outcome = Ended` 覆盖：跳过路径不经该臂）。
- **Cancelled 登记**：原生无摸牌阶段取消的生产者（`DrawOutcome::Cancelled` 在 D 帧上无发射点；`DamageOutcome::Cancelled` 属伤害族枚举，不相干）。按 F 批定案口径登记为差异：Cancelled 面留待有生产者时接，不算缺口关闭失败。登记见本文件＋测试文件头注释＋`docs/operations/evidence/2026-11-03-r05-private-wip/README.md` F2 段。

## 缺口3：dcbihuo 伤害两面的 owner 角色门

- **上游对照**：`archive/upstream/noname/apps/core/character/sp2/skill.js:7740-7743`——`trigger {player:"damageEnd", source:"damageSource"}`＋`filter: event.source && event.player != event.source`；`event.triggername=="damageEnd" ? 1 : -1`。语义：damageEnd 面只问**受害者**持有者（+1）、damageSource 面只问**来源**持有者（−1）、自伤不触发。
- **红断言**：
  - `r05f2_dcbihuo_bystander_holder_not_asked`：旁观持有者（seat2 持 sp2冯方，seat0 杀 seat1）两面都不被问、不写任何 Next。修前红：旁观者被问（f2-01.log panic）。
  - `r05f2_dcbihuo_victim_holder_end_face_plus_only`：受害者持有者只收 End 面一问，方向 +1（take 后 Current=+1 → 摸 3）。修前红：`[1,1]` 两问。
  - `r05f2_dcbihuo_source_holder_source_face_minus_only`：来源持有者只收 Source 面一问，方向 −1（Current=−1 → 摸 1）。修前红：`[0,0]` 两问。
- **修法落点**：`crates/engine-native-content/src/content/vm_effects/dcbihuo.rs` `step` 凭据段——`damage_root_data` 读权威伤害载荷后按窗口种加门：`DamageWindowKind::Source` 要求 `d.source == Some(owner)`，End 面要求 `d.target == owner`；原 source 存在/存活/非自伤检查保留（合取）。
- **既有测试对齐**（F/E 批曾按无门行为断言，改为上游门控语义；非改断言转绿——对应红测即上三条）：
  - `r05e_compounds.rs`：`r05e_dcbihuo_source_minus_end_plus_keeps_zero_then_take_moves_current`／`r05e_dcbihuo_cancel_end_window_keeps_minus_then_take_moves_minus`／`r05f_next_take_requires_enabled_helper_instance` 改双持（seat0=来源、seat1=受害者）各答各面（End 取消路径仍覆盖）；`r05e_dcbihuo_two_packages_do_not_share_slots` 断言改为「A（来源持有者）Source 面 −1、B（受害者持有者）End 面 +1 各一问」。
  - `r05f_real_packages.rs`：两个 dcbihuo 真包测试同样改双持＋按 observer 应答。

## 缺口4：两文件拆行（合入 main 行数门禁）

- 修前（HEAD `cf7c44d1` 复现）：`archive/machine_encode.rs` 1007 有效行、`kernel/events.rs` 1070 有效行（`python3 -B scripts/check-source-lines.py` exit 1，仅此二文件）。
- **machine_encode.rs 按机族拆**（552 行）：
  - `archive/machine_encode/boss.rs`（113 行）——Boss 机族 20 臂＋Shedu（tag 61-81、64）；
  - `archive/machine_encode/skills.rs`（453 行）——经典武将技能机族 43 臂＋SuiDun（tag 13-55 等）；
  - 主文件保留 D/R/G、judge、invocation、TriggerOrder、Sha、Rvm、锦囊/AOE 牌机、Host 等臂＋两族分派臂（variant 列表在主 match 保持编译期穷尽检查；子模块 `_ => unreachable!` 仅防御分派器漏配，编码字节序与 tag 零变化，内部递归经 `super::enc_machine`）。
- **events.rs 按帧族拆**（737 行）：
  - `kernel/events/turn.rs`（592 行）——回合/阶段帧族：`commit_turn_end_window`／`commit_turn_begin`／`commit_turn_ended`／`turn_handoff`／`reset_new_turn_state`；
  - events.rs 保留卡牌/弃牌/翻面/恢复族＋`pub(super) use turn::{...}` re-export，`kernel.rs` 既有 `use events::*` 路径零变化（turn.rs 函数 `pub(in crate::kernel)`）。
- 拆后门禁：`python3 -B scripts/check-source-lines.py` exit 0（9081 文件全过）；`test_check_source_lines.py` 2/2 OK。行为核验：kernel 组 434 通过（含存档 round-trip／A0 唯一性），编码字节序不变的回归由既有 golden/replay 组覆盖（唯一失败仍为既有 replay_scenarios golden，与 HEAD 相同）。

## 测试统计

- 新增 `crates/engine-native/tests/r05f2_semantic_gaps.rs`（登记 `tests/groups/misc.rs`）：7 测（6 先红后绿＋1 守卫）。
- 三 crate 全量：**1948 通过／1 失败／75 忽略**（基线 1941＋新增 7；唯一失败＝`replay_scenarios::replay_scenarios_match_golden`，HEAD 已存在，与本批无关）。证据 [f2-02.log](f2-02.log)。
- 定向：`--test misc r05` 38/38 绿（r05_turn_model 6＋r05e_compounds 20＋r05f_real_packages 5＋r05f2_semantic_gaps 7）。

## 语义载体说明

- Current 消费门＝`!num_fixed` 与 Combs 同门；Combs 门 `stored>0`、Current 门「条目存在」（hasStorage 存在性）——两门差异为上游语义，非实现自由度。
- 跳过路径 After 窗：D 帧经 Before→(After 窗)→Done，`pending=0` 不发 R 子树；tiqi 机器 `current_draw_phase` 读 D 帧 live `actual_gain=0` → n=0。
- 上游 dcbihuo 自伤不触发由既有 `s != d.target` 检查承担，本批未改。
