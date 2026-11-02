# 规则 332：改命取消归还后走正常判定初始化（2026-11-03）

按[裁决 24](../2026-10-30-nc09-referee/gaiming-cancel-proposal.md)（用户批准「改命修复 v332」）在已验收并入的 331（`f5e560db`）之后执行。规则 **331→332**（`STANDARD_RULESET_VERSION`）；存档 NGSA **106**／ABI **758**／WS **7** 不变；不新增状态字段、机器变体、选择/命令、prompt 或存档布局。

## 红（331 主树上再证，2026-11-03）

新增 `crates/engine-native/tests/nc09_gaiming_cancel.rs`（misc 组，3 测）在修复前 **3 红**（[01](01-red.log)）：

1. `cancel_preserves_physical_multiset_and_restores`：取消后物理多重集出现重复实体 **[65]**（124→125 张）——与 2026-10-30 首次复现同一实体号；
2. `cancel_then_normal_judgement_and_pick_control`：取消后 Judged 日志 **0 次**（未走正常判定）；
3. `genuine_331_pending_archive_is_version_rejected`：升号前 331 档尚未被拒（预期红，随批转绿）。

根因（同提案）：`kernel/judge.rs` 取消分支把 7 张候选回堆，但 `current.judge_card` 仍指向首候选并由机器持有，后续按判定牌结算并弃置同一实体。旧测试 `t11_boss_zhugeliang_hlss::hlss_gaiming_picks_judge_result_or_declines` 路径 B 把「牌堆不变（含重复）」锁成期望。

## 修法

`crates/engine-native-kernel/src/kernel/judge.rs`：

1. 把正常判定初始化尾段（抽判定牌→`Judged` 日志→昊天塔候选窗→鬼道/鬼才/天道/改天道/天伦改判队列→本人天妒资格→设置进行态）提取为固有方法 `begin_judgement(ctx, trick) -> Option<Flow>`（`Some`＝昊天塔开窗；`None`＝已设置正常进行态，调用方继续本机循环——与提取前落空语义逐字相同，正常入口行为零变化）。
2. 改命取消分支：7 张按原序回牌堆顶后 `self.current` 整体让位于 `begin_judgement(cur.trick)`——不保留指向已归还候选的机器持有判定牌、不重复询问改命；选牌分支逐字保留（同字段、同日志）。取消后的判定牌由正常抽牌取得（归还后堆顶＝原首候选），允许既有洗牌/插窗逻辑继续生效。

## 旧断言更新（逐条解释）

- `t11_boss_zhugeliang_hlss` 路径 B：`deck == deck_before` → `deck == deck_before - 1`。修复前 7 张回堆后不再抽牌（judge_card 别名结算→重复实体），牌堆确实「不变」；修复后正常判定抽走恰好一张（原首候选）。断言处注明规则 332 与红→绿证据位置。
- `pa12_tag656_version`：`ruleset_version` 断言 331→332（版本门禁随批）。

## 验收

- 新增 3 测全绿（升号后，[03](03-green-versioned.log)）：物理多重集守恒且取消后可恢复；取消后恰一次 `Judged` 且抽走原首候选、不再询问改命；真实 331 挂起档（`tests/fixtures/gaiming-window-331.ngsa`，2,157 B，sha256 `192611c2…`，修复前在 331 主树由探针生成）在 332 下 `RuleVersionMismatch` 拒绝、原件不动。选牌分支正控制（选中候选成为判定结果）保留。
- 所有权拒绝门禁未放松：重复实体反例即本批红①（修复前恢复拒绝 `card_in_multiple_zones`，修复后无重复可恢复）。
- golden 14 场三步探针（[04](04-golden-first.log)／[05](05-golden-probe-revert-version.log)／[06](06-golden-rerecorded.log)）：①修复+332 全位移；②版本回退 331 保留修复 **14 场全绿**＝修复对标准场景零足迹，位移唯一来源为规则版本进入状态摘要；③恢复 332 按实际值重录（`replay_scenarios.rs` 表内注明三步与行为差异边界）。
- 全量门禁：三 crate **1911/0/75**、workspace **7170/0/77**（[10](10-three-crates.log)／[12](12-workspace.log) 后台首跑各含 1 失败＝与断言同步竞态的 `adr0786…catalog 331`，[17](17-rerun-after-sync.log) 同步后补跑全绿、首跑失败保留）；wasm+game-server 59 段 0 失败（[11](11-wasm-server.log)）；strict Gate1b＋wasm32 独立 target exit 0（[13](13-strict-wasm32.log)）；WASM 重建＋真实 Worker smoke **82/0**（[14](14-wasm-smoke.log)）；跨目标 `native_wasm_match: true`（[15](15-cross-target.log)）；preflight 21 项全过（[18](18-preflight-final.log)，台账收敛后两次口径修正如实登记）。
- 缺陷台账 L03c（HlssGaiming）移入 `FIXED`（`scripts/report_defect_ledger.py` 注册表＋台账 README 轮次留痕），台账 **1→0（A 0／B 0）**；`report_registered_deviations`／守卫覆盖／账目留痕等脚本口径随空台账同步。

## 边界

- 登记差异保持：上游取消后 7 张滞留处理区、原生归还牌堆后正常判定——本批只是把「归还后」接到真正的正常初始化。
- 不承诺修复已含重复牌的旧坏档（此类档本应被所有权校验拒绝）；v331 及更早档按版本门禁拒绝，无迁移。
- Boss 诸葛亮包整包验收与此批无关；NC09 其余（超时执行、GaiMing 之外的缺口）照旧。
