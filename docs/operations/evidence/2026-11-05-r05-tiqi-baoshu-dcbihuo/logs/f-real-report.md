# R05-F 真包数据批报告（f-real-01）

日期：2026-11-04。工作树 `/private/tmp/noname-r05/worktree`（分支 r05-private，基线 HEAD 35218101）。
上游核对源：`archive/upstream/noname`（HEAD `1a548c95`）`apps/core/character/xianding/skill.js:41912/41983`、`apps/core/character/sp2/skill.js:7743`；武将 `character/xianding/character.js:1276`（re_fengfangnv＝冯妤，qun，female，hp 3，[tiqi,baoshu]）、`character/sp2/character.js:278`（fengfang＝冯方，qun，male，hp 3，[dcditing,dcbihuo]）。

## 产出

1. **两个真包**（每技能一个 `.skill` DSL 壳；生成期 retrigger：事件名改写＋模块体替换为 compound 单 resolver）：
   - `packages/extensions/xianding-upstream-noname/`：`manifest.json`、`package.npack.json`（2 技能/2 模块）、`assets/characters.json`（xnd冯妤＝tiqi+baoshu）、`src/tiqi.skill`、`src/baoshu.skill`。
   - `packages/extensions/sp2-upstream-noname/`：同构（sp2冯方＝dcbihuo）、`src/dcbihuo.skill`。
2. **装载器注释**：`crates/content-native-loader/src/lib.rs` 的 `draw_phase_after` 臂补 F 批真包定案（上游三面为何落 After 一面、Skipped/Cancelled 缺口）。无新增事件名映射（评估见下）。
3. **测试**：`crates/engine-native/tests/r05f_real_packages.rs`（5 测试），登记于 `tests/groups/misc.rs`。

## 装载路径与触发面映射（对照上游声明逐条）

| 技能 | 上游 trigger 声明 | 真包 trigger_index 事件名（→EventKind） | 说明 |
| --- | --- | --- | --- |
| tiqi | `global: ["phaseDrawEnd","phaseDrawSkipped","phaseDrawCancelled"]` | `draw_phase_after` → DrawPhaseAfter | 主 draw>0 时 End 窗被 R 子帧替代（kernel/draw.rs MainWaiting→End），恒发射终窗＝After；0 张时 End/After 并存、双声明会双发，故只声明 After（E 批登记差异 7 同口径）。机内 `current_draw_phase` 沿栈读 D 帧 live `actual_gain`。**Skipped/Cancelled 两面无原生触发载体＝语义缺口①**。 |
| baoshu | `player: "phaseZhunbeiBegin"` | `phase_prepare` → PhaseBegin+AtPhase{Prepare} | 既有 D02 映射，无改写；授权面＝机内 `at_own_prepare`（本人 Prepare begin 帧）。 |
| dcbihuo | `player:"damageEnd"＋source:"damageSource"`（＋subSkill effect `player:phaseBegin` silent、sha `player:phaseDrawBegin2` silent） | `damage_source`＋`damage_end`＋`phase_begin`（→DamageSource/DamageEnd/TurnBegin） | 三声明经 `join_trigger_only_entries` 并成单 handler 集；Source/End 窗＝−1/+1 写 Next，TurnBegin＝原子 take Next→Current。sha 的 Begin2 额定摸牌 ± 落 Current 槽——**内核无 Current→pending 消费点＝语义缺口②**。 |

filter 面：上游 filter 语义由 compound 机内承担（tiqi 排除自己 D/≠2；baoshu maxHp>0；dcbihuo source 存在/存活/非自伤），包声明 `owner_alive`/`owner_is_target` 只做存活门。

装载路径（生产同构）：`package.npack.json` → `bridge_vm_skill_decls`（validate＋mechanisms＋事件名严格映射＋filter 映射＋`derive_r05_helpers` 扫模块字节 HOST_EFFECT_EXT tag 派生 `tiqi#r05:limit`/`baoshu#r05:combs`/`dcbihuo#r05:next`/`dcbihuo#r05:current`）；武将经 `bridge_generals_with_vm_package`（Worker/服务端安装路径，schema3 模块别名解析＋known VM 技能门）。包 manifest：namespace `upstream-noname`、abi 按壳编译值（≤host 758，E 批登记差异 9 的「≤host 声明」口径）；aggregate logic_hash 经再生回验一致。

## 测试断言（先红后绿，全文见 f-real-01.log）

红＝npack 移出后 5 测试全 FAIL（装载失败：No such file or directory）；绿＝接入后 5/5 通过。

- `r05f_real_packages_regeneration_matches_committed_npack`：盘上 npack 与 `src/*.skill` 壳再生（编译→事件名改写→模块替换→重算哈希）逐字段＋logic_hash 一致；触发面事件名集合与装载面表精确相等；桥接后四角色派生 helper 在 decls 内。
- `r05f_real_tiqi_other_player_draw_confirm_sign_writes_limit`：seat1 双枯竭 D（n=0≠2）→ observer0 两问（confirm→sign，选 −d）→ `tiqi#r05:limit` seat1=Some(−2)，seat2 无串写。
- `r05f_real_baoshu_prepare_cost_resets_and_combs_marks`：本人 Prepare 成本窗选 2 目标（横置/翻面布点）→ `baoshu#r05:combs` 两目标各 Some(2)（m=1+maxHp3−k2）、横置解除、翻回、helper 实例装上。
- `r05f_real_dcbihuo_source_minus_end_plus_take_current`：seat0 杀 seat1 → Source/End 各一问（均选 seat1）→ Next=Some(0)（−1+1 抵消保零）；seat1 回合开始 take → Next 移除、Current=Some(0)、默认 helper 装。
- `r05f_real_dcbihuo_cancel_end_keeps_minus_take_signed`：End 取消 → Next=Some(−1) → take 写 Current=Some(−1)（负值路径）。

## 全量与门禁

- 三 crate（kernel/content/engine-native，--no-fail-fast）：**1941 passed / 1 failed / 75 ignored**。唯一失败＝`replay_scenarios::replay_scenarios_match_golden`（既有基线；基线 1936/1/75 ＋ 本批 5 测试）。
- `python3 -B scripts/check-source-lines.py` exit=1：`kernel/src/archive/machine_encode.rs`（1007）与 `kernel/src/kernel/events.rs`（1070）超 1000 有效行——**干净 HEAD 35218101 同样失败（stash 复跑同输出），非本批引入**；本批新增/修改文件均在限内。

## 语义缺口清单（按任务要求报告，未自行改 kernel/content）

1. **tiqi 的 phaseDrawSkipped／phaseDrawCancelled 两面无原生触发载体**：跳过路径 D 帧直接 Intrinsic→Done 不发窗（`kernel/phase/draw.rs:16-32`），EventKind 无对应面；Cancelled 结局当前无发射点。真包只承载 phaseDrawEnd 面（After 窗）。修复需 kernel 加窗（或映射名），属后续批。
2. **dcbihuo 的 Current 槽（上游 dcbihuo_sha：phaseDrawBegin2 `trigger.num += storage`）无内核消费点**：D 机 Begin2 只消费 `combs`（正值；`kernel/draw.rs:98-108`），`current` 只被写/take/到期清理（events.rs phaseAfter 与死亡清理），额定摸牌 ± 从未进 pending。本批 take 断言止于 Next→Current；「下回合额定摸牌数 ±」的最终生效待 kernel 接线（有符号 pending＋max(0) 执行门，与 Combs 正值口径不同）。
3. **dcbihuo 无 owner 角色门**（轻）：上游 damageEnd 面 player＝受害者、damageSource 面 player＝来源；机内资格只查 source 存在/存活/非自伤，未查 owner∈{受害者,来源}——旁观持有者也会被问。E 批已按此形态验收，本批沿用，登记待上游等价复核。
4. （沿袭登记，非新发现）manifest abi 按壳编译值（50/3）而非 759：E 批登记差异 9（host 仍 758，包按 ≤host 声明）；`validate_vm_effect_abi` 不 gate 781-783。
