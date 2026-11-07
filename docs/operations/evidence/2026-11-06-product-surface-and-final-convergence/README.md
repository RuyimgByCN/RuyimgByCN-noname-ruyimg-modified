# Batch 21：`nc08` 31 项已解决守卫／探针转绿与全产品面 13 门禁最终收敛证据（2026-11-06）

- **核验日期**：2026-11-06
- **基线提交**：`c0de4b59a`（Batch 20）
- **版本常量（零变更）**：
  - `STANDARD_RULESET_VERSION = 333`（`crates/engine-native-content/src/content/standard.rs`）
  - `ARCHIVE_FORMAT_VERSION = 107`（`crates/engine-native-kernel/src/archive.rs`）
  - `RULE_VM_ABI_VERSION = 759`（`crates/rule-vm/src/lib.rs`）
  - `NETWORK_PROTOCOL_VERSION = 8`（`crates/protocol/src/lib.rs`）
  - `CONTENT_SCHEMA_VERSION = 5`（`crates/content-schema/src/lib.rs`）
- **14 Golden Hashes**：零改动（14/14 全部通过）

## 1. Part A — 解除 `crates/engine-native/tests/nc08_registered_defect_guards.rs` 中 28 项已解决守卫的 `#[ignore]`（保持全部测试函数名不变）

在不重命名任何测试函数、保证 `scripts/report_guard_coverage.py`、`scripts/check_guard_references.py --strict` 与 `scripts/check_comment_counts.py --strict` 100% 兼容的前提下，将底层引擎缺陷已修复的 28 项守卫转为活跃回归断言（`nc08_registered_defect_guards` 由 `2 passed; 48 ignored` 收敛为 **`30 passed; 0 failed; 20 ignored`**；`nc08` 测试组由 `94 passed; 48 ignored` 升至 **`122 passed; 0 failed; 20 ignored`**）：

1. **直接移除 `#[ignore]` 即通过的守卫（4 项，连同基线已激活的 `guard_prepare_phase_exists` 与 `guard_rangjie_branch1_draws` 共 6 项）**：
   - `huajin2_must_not_add_a_phase_branch`
   - `ziyangdan_must_not_pin_the_end_phase`
   - `cap_damage_begin_second`（`capability_guard!` 宏支持 `active` 分支）
   - `guard_yueyin_fourth_damage`
2. **对齐 R05（规则 333）与拆库后真实源码路径／事件变体并移除 `#[ignore]` 的守卫（24 项）**：
   - `reguhuo_must_record_a_per_turn_use_marker`：读取 `step/skills_virtual_use.rs` 并断言 `HostBehavior::Reguhuo` 写入 `vm_aux.once` 每回合标记；
   - `g20_guard!` × 5（`g20_jiuwei_turn_end`、`g20_liyutang_turn_end`、`g20_zhucheng_turn_end`、`g20_paotai2_turn_end`、`g20_ziyu_turn_end`）：兼容 R05 `EventData::TurnEndWindow`；
   - `g23_guard!` × 10（`g23_huanling_turn_end`、`g23_lingxin_turn_end`、`g23_lqingcheng_turn_end`、`g23_donghuangzhong_turn_end`、`g23_guangshu_spade_turn_end`、`g23_jidong_turn_end`、`g23_binxin_turn_end`、`g23_lingyu_turn_end`、`g23_xuanyuan_turn_end`、`g23_menshen_turn_end`）：增加 `filtering/xianjian.rs` 查找回退并兼容 R05 `EventData::TurnEndWindow`；
   - `guard_yxs_menshen3_extra_phase`：读取 `../engine-native-kernel/src/vocabulary/host/metadata/common.rs` 并校验 `EventKind::Death` 额外触发幕；
   - `meta_event_guard!` × 4（`guard_niepan_dying`、`guard_shouyin_dying`、`guard_chengxin_dying`、`guard_shuiyun5_dying`）：遍历 `../engine-native-kernel/src/vocabulary/host/metadata/*.rs`（跳过 `source_order.rs`）校验 `EventKind::Dying`；
   - `guard_mopaifanbei_second_draw`：校验 `event.rs` 中 `DrawPhaseBegin2` / `DrawPhaseBefore` 枚举变体；
   - `pin_phase_guard!` × 2（`guard_hutian_pin_end`、`guard_mufeng_pin_end`）：跳过 `filtering.rs` 中的 `filter_swd(` 委派臂并兼容 R05 `EventData::TurnEndWindow`。

## 2. Part B — 修复并解除 3 项上游对照探针（`nc08_upstream_*.rs`）的 `#[ignore]` 及 GT-014 跨端断言同步

1. **`upstream_luweiyan_chain_kill_fires_on_first_target`（`crates/engine-native/tests/nc08_upstream_yunchou_luweiyan.rs`）**：在测试选择循环中对 `c.spec.prompt == "trigger_confirm"` 应答 `ChoiceOption::Confirm`（使 `luweiyan2` 追击【杀】触发），移除 `#[ignore]` 并实跑通过。
2. **`upstream_baishouzhihu_opens_target_window`（`crates/engine-native/tests/nc08_upstream_swd_baishouzhihu.rs` & `crates/engine-native-content/src/content/host/filtering/swd.rs`）**：移除 `HostBehavior::SwdBaishouzhihuHeld` 中冗余的 `ctx.catalog.cards[*c as usize].name == "baishouzhihu"` 硬编码牌名比对（保留 `hand_has_kind(ctx, owner, CardKind::Baishouzhihu)`，兼容 `bridge_pack` 装载的 `"swd_baishouzhihu_57"`），同步调整 `g1_shahai_swd/{daixing.rs, lianda.rs}` 的选择辅助循环以处理手持【白兽之琥】触发的 `baishouzhihu_target` 选人窗，移除 `#[ignore]` 并实跑通过。
3. **`upstream_zonghuo_discards_one_without_asking`（`crates/engine-native/tests/nc08_upstream_yunchou_zonghuo.rs`）**：在测试选择循环中对 `c.spec.prompt == "trigger_confirm"` 应答 `ChoiceOption::Confirm`（确认发动【纵火】自动弃 1 手牌并转火焰伤害），移除 `#[ignore]` 并实跑通过。
4. **GT-014 `onlyOL.npack.json` 导入期拒装跨端同步**：自 PA12 接入 `game_started` 触发事件与 `view_as_index` 桥接后，完整 `onlyOL.npack.json` 的未移植占位项在 `modifier_index` 阶段被拒装（`apps/game-server/tests/native_onlyol_general.rs` 已于批⑮同步为 `error.contains("modifier_index")`）；本批将 `crates/engine-native-wasm/tests/{gt014_import_validation_parity.rs, vm_package_worker.rs}` 与 `apps/web-client/scripts/native-worker-smoke.mjs` 的同源断言同步为 `modifier_index`。

## 3. Part C — 全产品面 13 门禁实测总表

| 门禁序号 | 验证命令 | 实测结果 | 原始日志 |
|---|---|---|---|
| Gate 1 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **2 tests OK** | [01-source-lines-and-layering.log](01-source-lines-and-layering.log) |
| Gate 2 | `python3 -B scripts/check-source-lines.py` | **9,187 files `<= 2,000` lines；Rust 非测试文件 `<= 1,000` effective lines** | [01-source-lines-and-layering.log](01-source-lines-and-layering.log) |
| Gate 3 | `python3 -B scripts/check-native-layering.py` | **0 upward references <= 0** | [01-source-lines-and-layering.log](01-source-lines-and-layering.log) |
| Gate 4 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader` | **2,212 passed; 0 failed; 39 ignored**（三 crate **2,168/0/39**、loader **44/0/0**，较 Batch 20 新增 31 项通过、减少 31 项 `#[ignore]`，14/14 golden 零位移通过） | [02-native-4crate-tests.log](02-native-4crate-tests.log) |
| Gate 5 | `CARGO_TARGET_DIR=/tmp/noname-gate1b-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && rm -rf /tmp/noname-gate1b-check` | **exit 0（0 warnings，临时目录已清理）** | [03-gate1b-and-wasm-check.log](03-gate1b-and-wasm-check.log) |
| Gate 6 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **exit 0（0 warnings）** | [03-gate1b-and-wasm-check.log](03-gate1b-and-wasm-check.log) |
| Gate 7 | `cargo run --offline -p conformance` | **passed=19, ALL PASS** | [04-conformance-desktop-cross-target.log](04-conformance-desktop-cross-target.log) |
| Gate 8 | `cargo test --offline -p desktop-client` | **19 passed; 0 failed; 1 ignored**（`13 + 6` passed） | [04-conformance-desktop-cross-target.log](04-conformance-desktop-cross-target.log) |
| Gate 9 | `python3 -B scripts/check-native-wasm-cross-target.py` | **3 passed; 0 failed**（`nc08_cross_target_real` **2/0** + `cross_target_package` **1/0**） | [04-conformance-desktop-cross-target.log](04-conformance-desktop-cross-target.log) |
| Gate 10 | `bash scripts/cross-target-hash.sh` | **passed=19, ALL CROSS-TARGET CHECKS PASSED** | [04-conformance-desktop-cross-target.log](04-conformance-desktop-cross-target.log) |
| Gate 11 | `apps/web-client`: `pnpm test`、`pnpm typecheck`、`pnpm build:native`、`pnpm smoke:native` | **`pnpm test` 461/461 pass、`pnpm typecheck` 0 错、`pnpm build:native` 0 错、`pnpm smoke:native` 82/82 pass (`failed: 0`)** | [05-web-client-gates.log](05-web-client-gates.log) |
| Gate 12 | `cargo test --offline --workspace --no-fail-fast` | **7,557 passed; 0 failed; 41 ignored**（145 个非空测试套件全绿） | [06-workspace-tests.log](06-workspace-tests.log) |
| Gate 13 | `python3 -B scripts/preflight.py` | **21/21 全部通过 ✅** | [07-preflight.log](07-preflight.log) |
