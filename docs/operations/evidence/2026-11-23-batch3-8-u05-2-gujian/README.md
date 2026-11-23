# Batch U05-2（`U05` 古剑奇谭 136 项）收口与验收证据（2026-11-23）

- **基线 HEAD**：`211c32d1adcbd8f1810f9896266a60bffd0d800f`（Batch U05-1 已发布，CI Run `45097178310` Attempt 1 `10/10 success`；同步纳入其未提交的 `ci-runs.md` 实测行）
- **合入来源**：`swarm/conv-u05-2-gujian`（`586b51cdc..921a8f499d16973d1797ff073cb0c85956140a33`，`136` 项候选范围）
- **独立复核裁决**：`verifier-u05-a` `#95 APPROVE`（审查 `921a8f499` Round 4 修复后全量 `136` 项等价、`0` 项保留 `简化近似`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（136 项全量升格 / 0 项保留）

- **升格为 `已复核等价`（`136` 项，全部 `reviewer="verifier-u05-a"`、`review_date="2026-11-23"`、`test_name="u05_b2_gujian_fixes + yange_carriers + verifier-u05a-cdp-probe.mjs"`）**：
  - `36` 张卡牌（`card/gujian/card.js` 32 张食物/武器/暗器/宝物 + `character/gujian/card.js` 4 张 `yanjiadan_*` 衍生机关牌）
  - `18` 名武将（`gjqt_bailitusu`、`gjqt_fengqingxue`、`gjqt_fanglansheng`、`gjqt_xiangling`、`gjqt_yinqianshang`、`gjqt_hongyu`、`gjqt_yuewuyi`、`gjqt_wenrenyu`、`gjqt_xiayize`、`gjqt_aruan`、`gjqt_ouyangshaogong`、`gjqt_xunfang`、`gjqt_xieyi`、`gjqt_chuqi`、`gjqt_beiluo`、`gjqt_yunwuyue`、`gjqt_cenying`、`gjqt_yanjia`）
  - `82` 个技能（`card/gujian/skill.js` 23 项 + `character/gujian/skill.js` 59 项，覆盖 `HostBehavior` tags `980..=985` `U05GujianCardBatch`/`CharBatchA..E`、既有食物/玉衡/木头面具/帝女玄霜分支与 `yange` 克隆载体 / `yanjia` 副装备合并双生命周期）
- **最新计数**：`U05` **`224 / 1,397 / 0`**（`1,621` 项）；全局 **`2,544 / 1,880 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突、行数拆分与集成修复

- **3-way 合并与模块拆分**：合并 `U05-2` 与主线 `L01`/`L11`/`U01-3`/`U05-1` 共存点（`known_prompts_u05_2.rs` 尾追加、`d34_flags.rs` 透传 `catalog`/`Some(card)`、`u08_batch_d_hooks.rs` 保留 `l11_common_b2(..., fid, u)` 并接入 `on_card_use_after_gujian`、`d34_l28_mods::is_bagua_armor_card` 供公共信息 AI 调用）；将 `crates/engine-native-content/src/content/host.rs`（原 1,009 有效行超限）拆出 `crates/engine-native-content/src/content/host/skill_defs.rs`（拆后 `host.rs` 与全部 Rust 非测试源码均 `<= 1,000` 有效行）。
- **集成回归修复 1（`declared_kinds` 生命周期清理）**：`U05-2` 将 `play_rules.rs` 与核心查询切换为 `Catalog::resolved_card_kind` 支持 `yange` 克隆载体后，`huyizhen`（`use_as: Sha`）等结算完毕进弃牌堆的实体牌（如 `u05_b1_r3_fix04` 中的 `Armor`）在洗回牌堆或重入手牌时仍残余陈旧 `declared_kinds`。已在 `crates/engine-native-kernel/src/kernel/equipment.rs`（`cleanup_discard_to_deck`）、`crates/engine-native-content/src/content/standard/skills/jian_xiong.rs` 与 `crates/engine-native/tests/u05_b1_fixes.rs`（`move_card_to_hand`/`put_deck_top`）同步清理离堆恢复牌的陈旧 `declared_kinds`/`declared_slots`，同时完整保留 `zhangba_two_cards_as_sha` 与 `yange_carriers` 语义。
- **集成回归修复 2（`decision34_round4_u05_shahai.rs`）**：随 `U05-1`/`U05-2` 将已等价的 `gujian` 等技能从 `IMPLEMENTED_U05_VM_SKILLS`（`919 -> 842`）迁入真实 `HostBehavior`（`HostBehavior::ALL: 611`），Catalog 中 VM 技能数为 `608`（总技能数仍 `>= 900`），将 `vm_skill_names.len()` 下界从 `680` 校准为 `600`。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-2/`）

- `probe-u05-2.mjs` sha256 `9ceecb79c5e97dda55e7c02c815b7b8614c5ea3760caf2e220c3e221aa734bbc`
- `upstream-u05-2-cdp-results.json` sha256 `41fb0b5415fd54c7dcc94097328946e45a34696089b3f2e9740f08dc17402c73`
- `verifier-u05a-cdp-probe.mjs` sha256 `cac7615fc3edf1899c5b8ced22b258139e2e2c2612a0f0657a651b3cb6df50b5`
- `verifier-u05a-cdp-results.json` sha256 `9aa3e20bd213ac7696fff98f94b9eff3288b0c16420238114ac0a5c6563bf566`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py`（`9,673` 文件 `<= 2,000` 行，Rust 非测试 `<= 1,000` 有效行）与 `scripts/check-native-layering.py`（`0` 上行引用）通过；`RUSTFLAGS="-D warnings"` 原生与 `wasm32-unknown-unknown` 零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44/44` 块、`2,952` passed、`0` failed、`6` ignored（含 `g1_shahai` `234/0/0`、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12/12` 块、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3/3` 块、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4/4` 块、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178/178` 块、`8,259` passed、`0` failed、`8` ignored。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`build:native` 通过、`smoke:native` `83/83` PASS（`failed: 0`、`37/37`、`46/46`）、`build`（`244` 模块 `vite build`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`scripts/cross-target-hash.sh` `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)
