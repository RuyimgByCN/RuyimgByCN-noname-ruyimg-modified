# 2026-11-25 Batch S1：`U05:shixu` / `U05:hs_nozdormu` 独立复核与 `Wave 1`（6 源）八维仓内预验证据

- **基线 HEAD**：`4c87ab8c08aa61669cfc3a076456b2e7a8f9494f`（CI Run `45500154427` Attempt 1 `10/10 success`，已在 [ci-runs.md](../ci-runs.md) 登记待随本批提交）。
- **版本与门禁不变量**：`STANDARD_RULESET_VERSION=333`、`ARCHIVE_FORMAT_VERSION=107`、content schema `5`、`RULE_VM_ABI_VERSION=759`、`WS_PROTOCOL_VERSION=8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动。
- **整包签署纪律**：本批为 `Wave 1`（`U06`、`U07`、`U08`、`U02`、`U03`、`U04`）上调前代码与验证闭环批，整包八项签署在本批提交前维持 **`0/37`**，待本批代码发布并取得精确提交 CI `10/10` 绿灯后于 Batch S2 正式上调 `Wave 1` 仓内可验维度。

---

## 一、`U05:shixu` 与 `U05:hs_nozdormu` 独立核验、缺陷修复与冻结边界

- **独立复核员**：`1cf6b3e7-33a1-432d-8c90-3da79cd80eec`（`research` 只读复核），对照上游 `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/skill.js:4877-4915` 与 `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/character.js:215-220`。
- **已修复非时钟真实偏差**：上游 `shixu` 声明 `locked: true`，依 [catalog_queries_seals.rs](../../../../crates/engine-native-kernel/src/catalog_queries_seals.rs#L8-L9) 不受铁骑（`tieji`）封技、仅受缠怨（`chanyuan`, `hp <= 1`）封技。[hooks.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b6b_hearth/hooks.rs#L97-L301) 第 97、301 行原误检 `!ctx.catalog.is_tieji_sealed(...)`，已修正为 `!ctx.catalog.is_chanyuan_sealed(...)`。
- **边界实证回归**：在 [part3_batch_b.rs](../../../../crates/engine-native/tests/u05_b6b_hearth/part3_batch_b.rs) 新增 `u05_b6b_shixu_and_hs_nozdormu_boundary_audit`（实测 `1 passed; 0 failed`），验证 `hs_nozdormu` 静态字段（`Faction::Qun`/`Male`/`5`/`["shixu"]`）、己方回合全局 `phaseEnd` `> 8000` 触发、`<= 3000ms` 不触发、`he` 强制弃牌（`allow_cancel=false`）与无牌跳过、铁骑豁免（`locked: true`）、`BoundedPlanner` 弃牌与缠怨 `hp <= 1` 封印。
- **无法不升版本等价的精确原因**：上游 `shixu` 使用 `get.time() - time > 8000` 真钟毫秒计算；原生确定性内核与 WAL（[wal.rs](../../../../apps/game-server/src/room_native/wal.rs#L204-L248)）禁止读取 wall clock，且 `CommandKind::EndPhase` / `StoredNativeAction` 不含耗时载荷（若要在联机/WAL 中记录真实耗时，须升 `WS_PROTOCOL_VERSION` `8->9`、`STANDARD_RULESET_VERSION` `333->334`，若单设持久化字段并须升 `ARCHIVE_FORMAT_VERSION` `107->108`）。已在 [inventory_status.json](../../../migration/g0/inventory_status.json) 将两项的 `简化近似（待复核）` 替换为精确边界与审计元数据（`4,419 / 5 / 0` 不变），并并入[待用户裁决清单](../2026-11-22-queue8-9-external-prereqs-and-phase6/README.md)（选项 A 维持 `elapsed_ms` 确定性代理 vs 选项 B 授权升 `WS 8->9` / `Ruleset 333->334`）。

---

## 二、当前 HEAD 性能全矩阵基线（`nc12_bench -- source_matrix`，第 ⑦ 维实测）

- 执行命令：`cargo run --offline --release -p engine-native --example nc12_bench -- source_matrix > docs/operations/evidence-raw/2026-11-25-signoff-perf/source_matrix.tsv`
- 原始产物 SHA-256：`607baebb729aa7794de18dfe8169a3262afd52400bb5bbdc8a98e97dfec2c8da`
- 汇总结论：全量 **49 / 49** `RESULT` 行（`none` 基线 + 37 来源 + 11 多包分拆子包）全部 `result=OK`，`sim_faults=0`、`plan_panics=0`、`roundtrip_failures=0`、`state_hash_mismatch=0`；冷导入全部 48 个正式包 `import_all_ns = 73,188,250`（约 `73.2 ms`）。

| Tier / 来源 | 包目录 | bridge (ns) | build p50 (ns) | cmd p50 (ns) | save/restore p50 (ns) | 结果与故障 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `none` (基线) | `-` | 0 | 13,083 | 24,791 | 292 / 12,625 | `OK` (`0/0/0/0`) |
| `src:U02` (boss) | `boss-play-upstream-noname` | 155,750 | 23,958 | 20,042 | 292 / 22,875 | `OK` (`0/0/0/0`) |
| `src:U03` (cardpile) | `cardpile-upstream-noname` | 314,375 | 41,166 | 24,083 | 417 / 42,583 | `OK` (`0/0/0/0`) |
| `src:U04` (coin) | `coin-upstream-noname` | 166,083 | 23,875 | 19,542 | 292 / 22,500 | `OK` (`0/0/0/0`) |
| `src:U06` (欢乐卡牌) | `huanle-upstream-noname` | 161,834 | 28,250 | 20,876 | 292 / 26,500 | `OK` (`0/0/0/0`) |
| `src:U07` (玩点论杀) | `wudian-upstream-noname` | 235,583 | 33,167 | 25,375 | 333 / 31,750 | `OK` (`0/0/0/0`) |
| `src:U08` (英雄杀) | `hero-kill-upstream-noname` | 152,417 | 26,208 | 20,500 | 292 / 25,667 | `OK` (`0/0/0/0`) |

---

## 三、`Wave 1`（`U06`/`U07`/`U08`/`U02`/`U03`/`U04`）八维仓内预验证据与独立差异复核（`195a4733`）

- **测试闭环**：在 [up37_eight_dim_signoff.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff.rs)、[harness.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff/harness.rs) 与 [wave1.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff/wave1.rs) 实测通过（`up37_wave1_six_sources_full_lifecycle_deps_config_combo_and_restore` 含 42 个宿主单元测试共 **43/43 passed**）：逐来源覆盖未签名拒绝、严格验签安装、幂等重装、启停重启持久化、不可逆升级、同版本回滚、缺失依赖拒绝、依赖方阻塞被依赖方禁停/卸载、物理清空卸载、多包同局组合与反序装配牌堆守恒、正反配置解析（`parse_boss_play_config`、`parse_pile_factors`/`bridge_pile`、`parse_coin_config` 与负例拒收）、以及升级/卸载后旧 `ContentLock` 存档与 wrong-version 拒绝。
- **独立差异复核员**：`195a4733-3093-442e-bf80-801421fd85c0`（`research`）核验 `inventory_status.json` 全数 6 源为 100% 等价（0 `简化近似` / 0 `缺失`），全部尚存差异清单如下：
  - **`U06` 欢乐卡牌（11/11 等价）**：规则/机制尚存差异 **无**（仅 `huanle-upstream-noname/package.npack.json` 译名字符串 `"jiuwei": "酒尾"` 非规则元数据错字）。
  - **`U07` 玩点论杀（36/36 等价）**：运行期规则差异 **无**（仅静态共存锁夹具 `u01-dddanzhi-pending-v107.ngsa` 在静态 coexist 路径保留历史 `wdb_xizhenxihong+jianbi`；正式包加载路径经 `attach_formal_package_upstream_rules` 为上游完全等价 `[jinyan, fuchou]`）。
  - **`U08` 英雄杀（150/150 等价）**：运行期规则差异 **无**；`1` 项内部静态锁兼容边界（为保护 `u01-dddanzhi-pending-v107.ngsa` 内容锁，`host_skill_trigger_meta` 保留 `nichang`/`wange`/`nichang2`/`wange2`/`jieyong` 历史静态元数据，运行期由 `kind_trigger_meta` [host.rs:495-531](../../../../crates/engine-native-content/src/content/host.rs#L495-L531) 无条件覆写为真实上游窗口）及 `hero-kill-upstream-noname/package.npack.json` 译名字符串 `"hero-kill": "雷霆万钧"` 非规则元数据字段。
  - **`U02` boss（17/17 等价）**：`boss_lvbu1`/`boss_lvbu2`/`boss_lvbu3` 上游势力为 `"shen"`，原生目录保持四势力枚举归入 `Faction::Qun`（[boss_play_data.rs:31-33](../../../../crates/content-native-loader/src/boss_play_data.rs#L31-L33)）。
  - **`U03` cardpile（30/30 等价）**：用顺序 `1..=13` / 确定性 `RuleRng::from_seed` 替代 `Math.random()`；比例表键用 `BTreeMap` 按花色字母序（`club`/`diamond`/`heart`/`spade`）稳定排布且增量卡名遵循 `{prefix}_scale_{source}_{suit}_{i}`（[pile.rs](../../../../crates/engine-native-content/src/content/pile.rs)）。
  - **`U04` coin（7/7 等价）**：终局倍率 `0.8 + random * 0.4` 使用确定性 RNG 或调用方注入 `SettleRequest::unit_random` 替代 `Math.random()`；Rust 规则后端 [coin.rs](../../../../crates/content-native-loader/src/coin.rs) 仅管数值/下注/结算、不含画布/音频/弹窗，特效面由 Web 前端 `visualEffectsBatch7Runtime.ts` 承担。
- **第 ⑥ 维保持“部分”原因**：仓库已覆 `Desktop` / `Web WASM` / `iOS arm64 sim` / `Android aarch64` 四端交叉检查，唯缺用户持 Android/iOS 真机实测与三端签名公证（PA08/PA11/NC12）。

---

## 四、本批仓内全量验证实测汇总

- `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：`2 tests OK`
- `python3 -B scripts/check-source-lines.py`：`10,019 files <= 2,000 lines; Rust non-test files <= 1,000 effective lines`
- `python3 -B scripts/recount_g0.py`：`37 sources, 0 missing`; `inventory_status.json` 汇总 **`4,419 / 5 / 0`**
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/noname-strict-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && ... --target wasm32-unknown-unknown --lib && rm -rf /tmp/noname-strict-check`：零警告通过并清理临时目录
- `cargo test --offline -p game-server --test up37_eight_dim_signoff`：**`43 passed; 0 failed`**
- `cargo test --offline -p engine-native --test g1_shahai u05_b6b_shixu_and_hs_nozdormu_boundary_audit`：**`1 passed; 0 failed`**
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`：`18` 块 **`3,222 passed; 0 failed; 6 ignored`**
- `cargo test --offline --workspace --no-fail-fast`：`179` 块（`147` 非空块）**`8,661 passed; 0 failed; 8 ignored`**（注：初次在并发编译/测试负载下运行时 `ai_planner::policy_v2_drives_all_seats_without_answer_routing_panic` 因墙钟 AI 预算发生 1 次轨迹抖动，单项实跑并通过后已无并发干扰重跑全量 `--workspace` 获全绿 **`8,661/0/8`**）
- `cargo test --offline -p conformance && cargo run --offline -p conformance`：`4/0/0` + `passed=19 ALL PASS`
- `cargo test --offline -p desktop-client --all-targets`：`14/0/0` + `6/0/0` + `0/0/1 ignored`（`20/0/1`）
- `pnpm --dir apps/web-client run typecheck && test && smoke:native && vite build --outDir /tmp/noname-web-build`：`typecheck` 零错、`test` `541/0/0`、`smoke:native` 通过、`vite build` `244` 模块通过并已删临时目录
- `python3 -B scripts/check-native-wasm-cross-target.py && bash scripts/cross-target-hash.sh`：`native_wasm_match=true`、`ALL CROSS-TARGET CHECKS PASSED`
