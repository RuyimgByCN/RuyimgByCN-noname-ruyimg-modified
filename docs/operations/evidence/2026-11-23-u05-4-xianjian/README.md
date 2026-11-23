# Batch U05-4（`U05` 仙剑 `xianjian` 36 武将与 120 技能共 156 项等价）收口与验收证据（2026-11-23）

- **基线 HEAD**：`362b01f2fc94c4a74b165041e94212e95a506195`（Batch U05-5b 已发布，CI Run `45167390472` Attempt 1 `10/10 success`；同步纳入其未提交的 `ci-runs.md` 实测行）
- **合入来源**：`swarm/conv-u05-4-xianjian`（`505ed0704..8e14ff93ae948c37135b3a7e0982e1ce1e0a43d8`，三路合入并保留 `main` 既有 `U05-1` / `U05-2` / `U05-3` / `U05-6a` / `U05-6b` / `U05-5a` / `U05-5b` / `L11` / `L01` / `U01` 修复与 `U05-4` Round 2–4 全部修正）
- **独立复核裁决**：`verifier-u05-a` `#109 APPROVE`（审查 Round 2–4 全量修复、静态代码与上游对照以及 Chrome CDP 实跑探针：批准全部 **`156`** 项 `/xianjian/` 武将与技能条目升格为 `已复核等价`，`0` 项保留 `简化近似`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（156 项升格 / 0 项保留）

- **升格为 `已复核等价`（`156` 项，全部 `reviewer="verifier-u05-a"`、`review_date="2026-11-23"`、`test_name="u05_b4_xianjian_fixes + verifier-u05a-cdp-probe.mjs"`）**：
  - `36` 项 `U05:generals:*@.../character/xianjian/character.js:*`（`pal_lixiaoyao`..`pal_jushifang` 全 36 名仙剑武将）
  - `120` 项 `U05:skills:*@.../character/xianjian/skill.js:*`（`lingquan`..`tianshe2` 全 120 项角色技、衍生技与全局技能）
- **未动范围与保留边界**：
  - Batch U05-6b 的 `8` 项保留边界、Batch U05-1 的 `7` 项保留边界、`U05:configs:gwent` 及其余未迁移子包（`swd` 等）继续保持不变。
- **最新计数**：`U05` **`1,083 / 538 / 0`**（`1,621` 项）；全局 **`3,403 / 1,021 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突处理、协同修复与模块拆分

- **词汇表与静态索引**：新增 `HostBehavior::U05XianjianBatchA..=E`（tags `1100..=1104`，`HostBehavior::ALL: 638`），尾追加独立提示词（[known_prompts.rs](../../../../crates/engine-native-kernel/src/archive/known_prompts.rs) 新增 `34` 条仙剑提示键 `xj_qianfang_cards`..`xj_u05_yujia_card`），接入 `crates/engine-native-content/src/content/host/step/xianjian_batch*.rs`（6 个新模块）并更新 [host.rs](../../../../crates/engine-native-content/src/content/host.rs)（`host_card_use_after_all` 透传 `fid`、`host_on_phase_end_all` 顺序调用 `host_on_phase_end` + `host_on_phase_end_xianjian`）。
- **主线协同与行数拆分保持 `<= 1,000` Rust 非测试有效行**：
  - 将 `turn_handoff` 与 `maybe_abort_ywuhun` 自 [turn.rs](../../../../crates/engine-native-kernel/src/kernel/events/turn.rs)（拆分后 `724` 行 / `640` 有效行）拆出为独立模块 [turn_handoff.rs](../../../../crates/engine-native-kernel/src/kernel/events/turn_handoff.rs)（`452` 行 / `426` 有效行），同时保留主线 `ow_shanxian_resume_target`/`on_turn_before_skip`/`before_facedown_turn_skip`/`extra_turn_force_round_bump` 与 `should_round_start`/`face_down` 逻辑。
  - 协同清理与合入 [catalog.rs](../../../../crates/engine-native-kernel/src/catalog.rs)、[catalog_queries.rs](../../../../crates/engine-native-kernel/src/catalog_queries.rs)、[resolution.rs](../../../../crates/engine-native-kernel/src/resolution.rs) 与 [frame_hooks.rs](../../../../crates/engine-native-kernel/src/resolution/frame_hooks.rs)（清理 `u05_play_equip_frame:` 暂存键）、`aoe.rs`/`duel.rs`/`intrinsic.rs`（`CardKind::Feibiao`）/`nullify.rs`/`sha.rs`/`sha/respond.rs`/`sha/resume.rs`/`combat_rules.rs`/`dying.rs`（`maybe_trigger_longxi*`、`is_guijiang_savable_blocked`、`du`/`xj_du_`）、`machine.rs` + `machine/skills.rs`（`skill_disable_flag_for`）、`play_card.rs` + `play_equip.rs` + [play_rules.rs](../../../../crates/engine-native/src/engine/play_rules.rs#L504)（统一 `&crate::catalog::Catalog` 参数类型）。
- **作用域加载与既有规则测试适配**：
  - 修复 `nc08_install_matrix` 作用域加载 `DuplicateSkill` 碰撞：自 [shahai_attach.rs](../../../../crates/content-native-loader/src/shahai_attach.rs#L494)（`533` 有效行）移除重复注入的 `gj_food_shihuifen_ban`，改于 [standard.rs](../../../../crates/engine-native-content/src/content/standard.rs#L592-L598)（`508` 有效行）按 `tuoqiao` 存在伴生注册 `HostBehavior::GujianFoodShihuifenBan`，兼容分包与统一加载。
  - 适配 `U05-2` `off_deck` 与 `Feibiao` 射程规则：[part1_batch_a.rs](../../../../crates/engine-native/tests/u05_b4_xianjian/part1_batch_a.rs#L112-L158) 过滤 `gouhunluo`/`jiguanshu`/`jiguanyaoshu`/`jiguanyuan` 并按 `mujiaren`/`zhiluxiaohu` 传自目标 `[0]`；[part6_round4_fixes.rs](../../../../crates/engine-native/tests/u05_b4_xianjian/part6_round4_fixes.rs#L236-L248) 为 2 人局目标座装备非 `off_deck` `CardKind::HorsePlus` 满足 `feibiao` `outrange: { globalFrom: 2 }`。
  - 校准 [decision34_round4_u05_shahai.rs](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L537-L565) 剩余 VM 技能下界至 `>= 190` / Host `>= 950` / 总技能 `>= 1350`（实测 `199` 项 VM 技能、`971` 项 Host 技能、`1,372` 项合计技能）。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-4/`）

- `probe-u05-4.mjs` sha256 `d299f485bed7fefccc621aba5d698b198e6c7f398d3fe3bca2d7274dd62e8d42`
- `upstream-u05-4-cdp-results.json` sha256 `ce876788225d333624b08ef56f56a2f698a092669504a697e7a61453c8da1954`
- `verifier-u05a-cdp-probe.mjs` sha256 `40ca1e2a11ccef2654def95f338dad95f4bac44a07f7f9723f00849813f7bb22`
- `verifier-u05a-cdp-results.json` sha256 `90975cae76b43380c9864fdadddd14eae35a70c71e84612be4997f99f266821a`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py`（所有维护文件 `<= 2,000` 行，Rust 非测试源码 `<= 1,000` 有效行）、`test_check_source_lines.py`、`scripts/check-native-layering.py`（`0` 上行引用）与 `scripts/preflight.py` 通过；`RUSTFLAGS="-D warnings"` 原生（5 原生 crate `--all-targets`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44` 块（`40` 非空套件）、`3,102` passed、`0` failed、`6` ignored（含 `g1_shahai` `383/0/0`〔含 `u05_b4_xianjian_fixes` `21/0/0`〕、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12` 块（`10` 非空套件）、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3` 块（`1` 非空）、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4` 块（`3` 非空）、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178` 块（`136` 非空套件）、`8,409` passed、`0` failed、`8` ignored（含 `nc08_install_matrix` `3/0/0`）。
- `scripts/test_audit_upstream_skill_flags.py`：`6/6` tests OK。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`wasm-pack` 部署 `native-pkg`/`wasm-pkg` 后 `smoke:native` `83/83` PASS（`failed: 0`、`sources=37/37`、`packages=46/46`）、`build`（`244` 模块 `vite build`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`engine-cli` / `wasm-hash.mjs` 跨端校验 `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`，`skill-vm` `ok=true`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)。
