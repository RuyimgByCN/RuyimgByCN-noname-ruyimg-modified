# Batch L01-Rules（L01 AI优化 69 技能＋1 卡牌 `yiyi`＋27 规则耦合配置共 97 项收口 → 133/133）独立复核与集成摘要

- **基线 HEAD**：`042f5b811927476b1bf056bc73e462194eee34ce`（Batch L11-2 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45008789520` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l01-rules` `1906e7f15..f5875e668e8b166241d20a933deeed69f363ede4`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-rules-2` 首轮（#180，含 10 项反例及 Chrome CDP 上游实跑探针）+ `verifier-ui` 第三轮复核（#196 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,842 / 2,582 / 0` -> **`1,939 已复核等价 / 2,485 简化近似 / 0 缺失 / 0 上游未定义`**（`L01 AI优化` 由 `36 / 97 / 0` 升为 **`133 / 0 / 0`** 达 100% 单项等价；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与后置兼容性修复

1. **词汇表、提示词索引与宿主分发协同**（与 L11 / L25 / U01 / L10 / L27 共存）：
   - `vocabulary/host/{behavior,behavior_list,keys,metadata,tag_decode,tags}.rs` 与 `crates/engine-native/src/lib.rs`：保留既有 `574..=575`、`654` 与 `L11` `720..=732` 标签，合入 `L01` 标签 `860..=864`（`L01YiyiCard`、`L01FirstKs`、`L01NeiKey`、`L01AttitudeTracker`、`L01AiRuleHooks`；`HostBehavior::ALL: 590`）。
   - `archive/known_prompts.rs`：保留主线全量既有提示词索引不变，将 `l01_yiyi_discard`..`l01_chongxu_action` 7 项仅追加至表尾。
   - `content/host/{activation,filtering,intrinsic_named_ext,step}.rs`、`kernel.rs` 与 `ai/extension_choices.rs`：保留 `txhj_*`/`l11_*`/`l10_*` 共存分发与 `xumou:confirm`、`extension_choices_misc`..`extension_choices_l11_b2` 回退链，叠合 `yiyi`/`l01_rules`/`l01_attitude` 及 `l01_policy` AI 决策路由。
2. **`play_equip` / `xinfu_limu` / `legal_actions` 合并协调**：
   - `play_card.rs` / `play_equip.rs` / `play_rules.rs` / `play_sha.rs` / `legal_actions.rs`：在 `play_equip` 补充 `!player(player).hand.contains(&card)` 拦截 `xinfu_limu`（`hes` 装备区虚拟【乐】路径误重装装备）；组合 `is_ssr_luoyi_trick_to_duel`、`tanbei_unlimited_target`、`eval_card_distance_unlimited_to` 与 `xinfu_limu_in_range`；并在 `legal_actions` 同时保留 `txhj_*` 与 `yiyi`（1–3 目标与 `[seat, ...]` 组合）。
3. **无扩展空包 `on_game_start` 零写入修复**：
   - 全仓回归 `installed_package_payload::identity_preserves_roles_lord_bonus_and_extension_init_hooks` 捕获到 `l01_on_game_start` 在未加载 `L01` 的空包身份建局中无条件写入 `l01_first_ks_done = 1`，导致 `bare.state_hash() != standard.state_hash()`。现将 `l01_first_ks_done` 写入置于 `catalog.skill_by_name(L01_FIRST_KS_KEY).is_some()` 门控内，兼顾无 `L01` 空包哈希严格一致与全量 `L01` 测试通过。

---

## 2. 独立复核 97 项升级实体清单（`#196` `APPROVE`，`AI优化/extension.js`）

- **Skills (69)**：`_aiyh_fake_prohibited`、`_aiyh_firstKs`、`_aiyh_fixECf`、`_aiyh_fixEQz`、`_aiyh_fixWj`、`_aiyh_lianhe`、`_aiyh_meks`、`_aiyh_neiKey`、`_aiyh_nhFriends`、`_aiyh_reserved_shan`、`_aiyh_wqjz`、`_attzhi`、`_fenghuyunlong`、`_findZhong`、`_gjcx_correct`、`_mjAiSkill`、`_shyg`、`_shyg_Discard`、`_shyg_Discard_du`、`_shyg_qjAi`、`_shyg_wugu`、`_shyg_wuxie`、`_shyg_wuxie_jiedao`、`_shyg_wuxie_lei`、`_tianxiayitong`、`_wanjunpiyi`、`_yingshilanggu`、`aiMangju`、`aiyh_disabled_shan`、`aiyh_gjcx_qj`、`attbaolu`、`attbo`、`attcard`、`attdamage`、`attfanzhuang`、`attgain`、`attlose`、`attlosehp`、`attmingbian`、`attmingbian1`、`attrecover`、`atttf`、`atttiaofan`、`attturnover`、`chaofeng_common`、`chaofeng_epic`、`chaofeng_legend`、`chaofeng_rare`、`dcwumei_all`、`gjcx_neiAi`、`gjcx_neiFan`、`gjcx_neiJiang`、`gjcx_neiZhong`、`gjcx_zhuAi`、`jlnj`、`shyg_Discard_du_useful`、`shyg_Discard_key`、`shyg_mz`、`shyg_smz`、`shyg_tf`、`shyg_tz`、`shyg_wugu_ss_pai`、`shyg_wugu_wss_pai`、`shyg_wuxie_fy`、`tfattzhong`、`tfattzhu`、`twzhian_ai`、`xinfu_limu`、`zhenlie_lose`
- **Card (1)**：`yiyi`（每目标独立 `NullifyWindow`、`CardsLost`、`CardDiscarded`、国战/对决全友与 `[seat, ally]` 组合、与 `L01` 开关兼容性降级隔离）
- **Rule-coupled Configs (27)**：`Wj`、`applyCf`、`applyQz`、`attceshi`、`bandizhu`、`banfan`、`bannei`、`bannongmin`、`banzhong`、`banzhu`、`btAi`、`ckQz`、`findZhong`、`fixECf`、`fixEQz`、`fixFour`、`hsyh`、`mjAi`、`neiKey`、`nhFriends`、`ntAoe`、`qzCf`、`sfjAi`、`wjAi`、`wjs`、`xyyh`、`zbAi`
- **原生实现与回归测试**：`crates/engine-native-content/src/content/host/step/{l01_rules,l01_attitude,l01_yiyi}.rs`、`crates/engine-native/src/ai/{l01_policy,l01_reserved_shan}.rs`、`crates/content-native-loader/src/ai_tuning.rs`、`crates/engine-native/tests/l01_ai_optimize_fixes.rs`（`5/5`）、`crates/engine-native/tests/l01_round2_regressions.rs`（`1/1`）、`l01_eight_dimension_from_fresh_install` 与 `crates/content-native-loader/tests/ai_tuning_loader.rs`（`4/4`）（`0` `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l01-rules/`）

- `55353b88a3bab516addb8622081ced9bf8311ae7f80eeb6459c446f366925456` — `docs/operations/evidence-raw/review-l01-rules/probe-l01-rules.mjs`
- `17cbfa05328f196c70b4149bf5a32cd2ed6df5fa2dfe49291742018c96d2ec5b` — `docs/operations/evidence-raw/review-l01-rules/upstream-l01-rules-cdp-results.json`（`pageExceptionCount: 0`）

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-integrator-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（已清理 `$T`）
   - `4-crate` offline：`30` 组、**`2,766` passed / `0` failed / `6` ignored**（含 `l_rounds` **`223/223`**、`g1` **`374/0/1`**）
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `build:native` 后 `check-native-wasm-cross-target.py`（标准 `40` + 欢乐 `20` 均 `native_wasm_match: true`）& `cross-target-hash.sh`：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,114` passed / `0` failed / `8` ignored / `0` filtered**（含 `installed_package_payload` **`8/8`**，exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`check_ledger_consistency.py --strict`、`check_doc_line_refs.py --strict`、`recount_g0.py` 与 `preflight.py` 全过。
