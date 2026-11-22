# Batch L11-6a（L11 太虚幻境 320 武将档案与净集 111/320 武将→519/936）独立复核与集成摘要

- **基线 HEAD**：`b27a0968b13355a35292c767a3d3635401f5d31d`（Batch L11-5 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45065560795` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l11-b6a-generals` `ce6b9126b..5803787d9`（`d8d1434fe`、`8a974b73f`、`5803787d9`）
- **版本与内容锁**：规则 `333`／NGSA `107`／content schema `5`／ABI `759`／WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-l11`（`#57` **`APPROVE`**，含 Round 3 校验与两轮 Headless Chrome CDP）
- **清单变更**：全仓 `2,175 / 2,249 / 0` -> **`2,286 已复核等价 / 2,138 简化近似 / 0 缺失 / 0 上游未定义`**（`L11 太虚幻境` 由 `408 / 528 / 0` 升为 **`519 / 417 / 0`**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与主线对齐修复

1. **核心机制、性别与 `L11-5` 共存协调**：
   - `judge.rs` / `judge_rewrite.rs` / `judge_apply.rs`：保留主线 `judge_apply.rs` 内联 `OrdinaryJudge` 与 `txhj_zhuwei`（`emit_zhuwei_ordinary_judge_if_needed`、`settle_inline_judge_card`），并将 `b6a` Round 3 的 `judge_provider_discards_old` 与 `JudgmentRewriteHandEquip` 改判合并入 `judge_rewrite.rs`（避免与主线拆分冲突）。
   - `sha.rs` / `sha/step.rs`：保留主线 `L11-5` `_txhj_equip5` 宝物槽筛选、`resolve_shan_card` 拆分及 `tieji_red.is_none()` 守卫，并在 `ShaResolveMachine::needed_shan_count` 中合并 `eval_sha_response_extra_count`（同时作用于初查与 `sha/respond.rs::post_dodge_step`）且在 Qilin 目标过滤保留 `!eval_card_discard_protected`。
   - `content/host/step/l27_fixes.rs`、`u01_fixes_3/helpers.rs` 与 `l11_taixu_b6a/round3_fixes.rs`：保留 `HEAD` `refresh_xiefang_mark` 与 `U01-3` `dddshixing`（`u01_fixes_3` 实际授予 `xiangle`/`jiushi`/`rezhiheng` 掩码），将二者剩余 `effective_gender` 判断统一为 `core.matches_gender(catalog, ..., Gender::Female)`，使 `"double"` 性别角色 `txhj_niufudongxie` 在 `L27`/`U01`/`U08` 均按上游逻辑同时匹配男女。
   - `content-native-loader/src/decision34_pack9.rs` & `decision34_pack9_ui.rs`：合并 `taixu` VM `lookup_upstream_host_behavior`/`jueqing` 过滤与被动 `feiying` 修饰，同时将 `U01GalleryState`、`L24SkinTransform`、`L25DiyEditorDraft`、`L27DualGeneralPick` UI/编辑辅助结构拆出至 `decision34_pack9_ui.rs`（原样重导出），维持所有 Rust 非测试文件有效行 `<= 1,000`。
   - `taixu_general_profiles.rs`、`state/{mod.rs, general_assembly.rs, core_ops_modifiers.rs}`、`l11_boss_c{,2,_asks}.rs`、`d34_flags.rs`、`eval.rs`（`can_gain_link`/`can_lose_link`）、`tricks.rs`、`equipment.rs`、`events.rs`、`sha/armor.rs`、`phase/{end,judge,play}.rs`：保留 `L11-5` Round 2 全部修复并完整合入 `320` 武将档案（含 `starting_hp < max_hp`、`starting_hujia`、`"double"` 性别、正反主副 `dualside` 与真实 `xueji`／`bazhen`／`roulin`／`reguicai`／`drlt_qianjie`／`nzry_jieying`／`jueqing`／`feiying`）。

---

## 2. 独立复核 111 项升级与 209 项保留清单（`#57`）

- **升格为 `已复核等价`（111 项 `L11:generals:*`）**（`reviewer: "verifier-l11"`，`test_name: "l11_taixu_b6a"`，`review_date: "2026-11-23"`）：
  - **纯已等价依赖技能全定义武将（103 项）**：`txhj_reganning` .. `txhj_qianchaotongren`（按 `#57` 逐项净集核定清单，依赖技能均已为 `已复核等价`）。
  - **已等价技能＋仅上游未定义（真实 no-op）武将（8 项）**：`txhj_huanguan`、`txhj_huanguandaofushou`、`txhj_xiaoweijun`、`txhj_chunyuqiong`、`txhj_jinweijun`、`txhj_zhaorong`、`txhj_zhaozhong`、`txhj_jianshuo`（上游 Chrome CDP 实测 `txhj_xiehui` 等未定义引用确为 `"undefined"`，与原生惰性 no-op 1:1）。
- **保持 `简化近似`（209 项 `L11:generals:*`）**：按 `#57` 与 Coordinator `#90` 严格保留涉及旧 `vm_modifiers`／差异技能（15 项）、`twzuhuo`（1 项）、`L11_TAIXU_OWN_NPC_SKILLS`（24 项）与 `decision34_l11_npc_a/b` 跨包 NPC 技（169 项，含 `txhj_huangjinyaoshushi`）的 209 名武将为 `简化近似`，待后续 `L11-6b`／专项批次闭合。
- **回归测试入口**：`crates/engine-native/tests/l11_taixu_b6a_generals.rs`（7 项：`b6a_generals` 6 项全量静态／实例化／护甲开局血量／`dualside`／`double` 性别／存档恢复测试 + `l11_taixu_b6a/round3_fixes.rs` `round3_fixes`，经 `l_rounds` 运行，0 `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l11-6a/`）

- `976a89d879e810ad86ac8c3423d52bb8a62e77c0402599f63032a90c5faf8a6a` — `docs/operations/evidence-raw/review-l11-6a/probe-l11-6a.mjs`
- `2c0ff583371a3b047c9177d586c0fb7cfac99699bcda0b3560e48dff4b59e0b7` — `docs/operations/evidence-raw/review-l11-6a/probe-l11-6a.json`
- `a0df3c1e0e437bd6001ad8f3721dbdaee91359d28188fd26398a07a3408edd72` — `docs/operations/evidence-raw/review-l11-6a/probe-l11-6a-r3.mjs`
- `2d6965d986552437f328a3f4b73eeff0acb6fc607421a8ecaac7cf65119db9b9` — `docs/operations/evidence-raw/review-l11-6a/probe-l11-6a-r3.json`

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`, `engine-native-wasm`, `content-native-loader`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`44` 组、**`2,882` passed / `0` failed / `6` ignored**（含 `l_rounds` **`277/0/0`**、`g1` **`395/0/1`**、`kernel` **`435/0/0`**、`engine-native-wasm` `26` 组 **`89/0/0`**）
   - `content-native-loader` offline：`12` 组、**`48` passed / `0` failed / `0` ignored**
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py`（`native_wasm_match: true`）& `cross-target-hash.sh`（`start_4_42=8fe8316a...` / `start_2_1=6fb9d95a...` / `manifest_4_42=11214341...`）：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,189` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client run test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`recount_g0.py` 与 `preflight.py` 全过（提交前终验见发布日志）。
