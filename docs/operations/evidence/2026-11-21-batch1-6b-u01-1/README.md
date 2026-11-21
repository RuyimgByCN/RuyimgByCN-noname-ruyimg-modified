# Batch U01-1（U01 3D精选首批 20 项技能收口 → 53/95）规则修复与独立复核摘要

- **基线 HEAD**：`b7ac33cb6d6164be6b4f3e124c24c668e666cac6`（Batch Q2-L19 前向修复已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44954522303` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-q1-u01` `af6ef4899..44c3ae9119ee4154a5b371a265bbce2f5956116b`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁保持不变
- **独立复核员**：`verifier-rules`（未参与 Batch U01-1 代码实现；经三轮边缘测试与上游 headless Chrome CDP 实跑比对，在 `#93` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,313 / 3,111 / 0` -> **`1,333 已复核等价 / 3,091 简化近似 / 0 缺失 / 0 上游未定义`**（`U01 3D精选` 由 `33 / 62 / 0` 升为 **`53 / 42 / 0`**，+20）；整包八项签署维持 **`0/37`**

---

## 1. 集成冲突协调与单文件拆分

1. **`crates/engine-native-kernel/src/catalog.rs` 与 `catalog/hooks.rs`**：为严格满足非测试 Rust 有效行数 `<= 1,000` 上限，`ContentHooks`、`impl ContentHooks` 及相关 Hook 类型别名按职责拆入 `crates/engine-native-kernel/src/catalog/hooks.rs` 并由 `catalog.rs` 公开重导出，保留 `reguhuo_*`/`on_game_start`/`on_player_died`/`on_phase_use_end`/`on_round_start`/`on_hand_card_removed`/`on_recover_before`/`preserves_skills_on_death`（L27/L10/L19）并与 U01-1 `respond_skill_options`/`try_activate_respond_skill` 共存。
2. **`crates/engine-native-kernel/src/archive/known_prompts.rs`**：严格保持 `HEAD` 全量既有索引（L27/L10 30 项与 L19 `wujinmoshiRE1:confirm`/`wujinmoshiRE2:choose_skill`）不变，于表尾顺序追加 20 项 U01-1 提示词，保证既有 `.ngsa` 存档与 14 golden 位一致恢复。
3. **响应窗与杀解析器协同（`kernel/dying.rs`、`kernel/sha/{step,respond,respond_skill}.rs`、`snap/asks.rs`、`snap/asks/sha.rs`）**：组合保留 `reguhuo`/`anruo`（`host_response_skills`、`reguhuo_spawn_response`/`reguhuo_spawn_final`）、`dddtongyu` 手牌锁（`hand_locked`）、`respond_skill_options`/`try_activate_respond_skill`（`dddxiaxing`/`dddbingjian`/`dddshilie`/`dddfushi`）及 `sha_u01_shan_pending`/`complete_shan_dodge -> post_dodge_step`（保留 `mbcmfangzhu` 禁装检查与 `liaoyuan_count = 0` 复位），避免 `respond_skill.rs` 与 `respond.rs` 逻辑重复。
4. **`crates/engine-native-content/src/content/host.rs` 与 `host/step/u01_fixes_1/`**：在 `kind_trigger_meta` 保留 L29 `MbQianlong`/`MbcmQingzheng`/`MbcmJiushi` 与 L19 `L19DoomsdayBatchA`/`B`，并并存路由 `D3Moyan`（`dddmoyan`）与 `U01DddBatchA..D -> trigger_meta_u01_fix`。
5. **`crates/engine-native/tests/u01_fixes_1/part6_recheck.rs:516-519`**：因 Q2-L27 已对齐上游 `xingchong` 在 `roundStart`（3 号位 -> 0 号位换轮时）清空标记，`recheck_03_dddguiying` 改以同在共存包内的 `ssr_bailun`（空手且 `hp <= 1` 时失去装备立即摸牌）驱动 `dddguiying` 结算期动态拾取吴国目标并验证完整存档往返。

---

## 2. 独立复核 20 项升级清单（`#93` `APPROVE`）

`dddzhishu`、`dddxiaxing`、`dddjuxian`、`dddjungui`、`dddjinggou`、`dddmoyan`、`dddshilie`、`dddyeshen`、`dddqiaoduan`、`ddddiedang`、`dddanliu`、`dddguiying`、`dddbingjian`、`dddtongyu`、`dddzhilian`、`dddjijian`、`dddjijian_old`、`dddfushi`、`dddlangzhi`、`dddfuzong`（均位于 `archive/upstream/noname/apps/core/extension/3D精选/character/skill.js`，`inventory_status.json` 全量由 `简化近似` 升为 **`已复核等价`**，`reviewer: "verifier-rules"`，`review_date: "2026-11-21"`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `evidence-raw/review-u01-1/`）

- `ee9d7913a7528716b985573a044de9c89715e56bd1d276d521c946ceda01a978` — `docs/operations/evidence-raw/review-u01-1/probe-u01-1.mjs`
- `726766164381b75e23513ce678a1e23dfb288ac8c614c722e61eafc566f5959d` — `docs/operations/evidence-raw/review-u01-1/upstream-u01-1-cdp-results.json`
- `deea469c44bf15c15f18339fd66da4c047919cee008ab3e5432ec54739135ce4` — `docs/operations/evidence-raw/review-u01-1/probe-u01-1-r2-edges.mjs`
- `fbef670961629e17f9023d2766e85f07569e943566e9b36489a645310db864cc` — `docs/operations/evidence-raw/review-u01-1/upstream-u01-1-r2-edges-results.json`
- `7048748464cbfba4552e3ce433385b9b7cd67408ea7e5e524453742570d2d7ee` — `docs/operations/evidence-raw/review-u01-1/probe-u01-1-r3-edges.mjs`
- `78b18e440b4427e16efa413cddd914fdb47d93eb8833a5acd6fb4794300a91ef` — `docs/operations/evidence-raw/review-u01-1/upstream-u01-1-r3-edges-results.json`

---

## 4. `main` 全量未过滤门禁实测记录

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-u01-1 CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：exit `0`（已即时 `rm -rf` 清理）
   - `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`（`-D warnings`）：exit `0`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader`：30 组测试输出，**`2,656` passed / `0` failed / `6` ignored**
   - `cargo run --offline -p conformance`：**19 / 19 (`ALL PASS`)**；`cargo test --offline -p conformance`：3 组、`4` passed / `0` failed / `0` ignored
   - `cargo test --offline -p desktop-client`：4 组、**`20` passed / `0` failed / `1` ignored**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：`native_wasm_match: true`（标准 `40`）；`bash scripts/cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：178 组测试输出，**`8,004` passed / `0` failed / `8` ignored**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`514` passed / `0` failed**
   - `pnpm --dir apps/web-client run build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`（`2 OK`）、`check-source-lines.py`（9,448 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_ledger_consistency.py --strict`（`台账与枚举一致`）、`recount_g0.py` 与 `preflight.py` 全过。
