# Batch U05-5a（`U05` 昆特牌 `gwent` 卡牌及伴生技能 80 项全量等价）收口与验收证据（2026-11-23）

- **基线 HEAD**：`d6d411439fc2f763ceeeef359e10372f702dd0b0`（Batch U05-6b 已发布，CI Run `45138296268` Attempt 1 `10/10 success`；同步纳入其未提交的 `ci-runs.md` 实测行）
- **合入来源**：`swarm/conv-u05-5a-gwent-cards`（`b7ac33cb6..278b0ac8af762ef10759d81afa3017aa4036c939`，基于 `b7ac33cb6` 增量补丁经三路重放合入主线，完整保留 `U05-2` / `U05-3` / `U05-6a` / `U05-6b` / `L11` / `L01` / `U01` 修复与 Round 2 全量修正）
- **独立复核裁决**：`verifier-u05-a` `#96 APPROVE`（审查 Round 2 全量修复与 5 组 Chrome CDP 探针：批准全部 **`80`** 项 `5a` 卡牌与卡牌伴生技能升格为 `已复核等价`，`0` 项保留 `简化近似`；`/character/gwent/` 121 项留待 `U05-5b`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（80 项升格 / 0 项保留）

- **升格为 `已复核等价`（`80` 项，全部 `reviewer="verifier-u05-a"`、`review_date="2026-11-23"`、`test_name="u05_b5a_gwent_cards_fixes + verifier-u05a-cdp-probe.mjs"`）**：
  - `69` 项 `U05:cards:*` 昆特卡牌（`43` 项 `@.../card/gwent/card.js:*` + `26` 项 `@.../character/gwent/card.js:*`，含 13 铜卡法术/天气、12 金法术与 `gw_dieyi` 蝶翼体系、12 银法术、20 张冒险金卡及特殊衍生牌）
  - `11` 项 `U05:skills:*@.../card/gwent/skill.js:*` 昆特卡牌伴生技能（`gw_aerdeyin`、`gw_kunenfayin`、`gw_baobaoshu`、`gw_nuhaifengbao`、`gw_youer`、`gw_qinpendayu`、`gw_birinongwu`、`gw_ciguhanshuang`、`gw_dieyi`、`gw_leizhoushu`、`_gainspell`）
- **未动范围与保留边界**：
  - `/character/gwent/` 下 `121` 项（`52` 武将 + `69` 武将技能）与 `U05:configs:gwent@.../main/config.js:39` 保持不变，留待后续 `U05-5b`；继续保持 Batch U05-6b 的 `8` 项保留边界与 Batch U05-1 的 `7` 项保留边界不变。
- **最新计数**：`U05` **`806 / 815 / 0`**（`1,621` 项）；全局 **`3,126 / 1,298 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突处理、协同修复与模块拆分

- **词汇表与静态索引**：新增 `HostBehavior::U05GwentCardEffect`（tag `1160`）与 `HostBehavior::U05GwentCardSkill`（tag `1161`），`HostBehavior::ALL: 629`；尾追加独立提示词表 [known_prompts_u05_5a.rs](../../../../crates/engine-native-kernel/src/archive/known_prompts_u05_5a.rs)（`21` 条提示词，经查与其他 `known_prompts*.rs` `0` 重名）；接入 [u05_b5a_gwent/mod.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b5a_gwent/mod.rs) 及其 7 个子模块（`gold_spells`/`silver_spells`/`adventure_a`/`adventure_b`/`adventure_huoge`/`skills`/`asks`）。
- **装备别名（`yange`/`yanjia`）与 `gw_dieyi` 屏蔽联动**：`U05-2` 已将实体装备牌种查询切换为 `Catalog::equip_card_has_kind(core, c, kind)`，`U05-5a` 新增 `gw_dieyi` 装备屏蔽检查 `is_gw_dieyi_masked` / `has_active_equip_kind`；主线在 [core_ops_modifiers.rs](../../../../crates/engine-native-kernel/src/state/core_ops_modifiers.rs) 中统一使 `has_active_equip_kind` 同时调用 `catalog.equip_card_has_kind(self, c, kind) && !self.is_gw_dieyi_masked(catalog, owner, c)`，并协调 `sha.rs`/`sha/armor.rs`/`sha/respond.rs`/`play_sha.rs`/`equipment.rs`/`tricks.rs`/`d34_flags.rs`/`d34_l28_mods.rs`/`snapshot.rs`/`resolution.rs`（沿用 `frame_hooks.rs` 出栈清理）与 `legal_actions.rs`（合并 `gw_*`/`gwmaoxian_*`、`gw_zhuoshao` 多目标组合、`yiyi` 及 `U05-6a` `Wuzhong` 合法性校验）。
- **行数拆分保持 `<= 1,000` Rust 非测试有效行**：
  - 将 `clean_gwent_name`、`validate_u05_gwent_targets` 与 `validate_u05_gwent_bronze_targets` 自 [play_card_targets.rs](../../../../crates/engine-native/src/engine/play_card_targets.rs) 拆出为独立模块 [play_card_targets_gwent.rs](../../../../crates/engine-native/src/engine/play_card_targets_gwent.rs)（458 行），保留 [play_card.rs](../../../../crates/engine-native/src/engine/play_card.rs) 对 `validate_tail_ext_targets`（含 `CardKind::Wenyi`）与 `try_recast_l11_trick` / `push_play_card_frame` 的主线拆分结构。
  - 恢复 [extension_choices.rs](../../../../crates/engine-native/src/ai/extension_choices.rs) 全部既有批次模块并拆出默认分派链 [extension_choices_dispatch.rs](../../../../crates/engine-native/src/ai/extension_choices_dispatch.rs)（198 行），末尾串接 `extension_choices_u05_5a::plan_u05_5a_choice_action`。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-5a/`）

- `probe-u05-5a.mjs` sha256 `60da20ed2f24a28d2f60ff9902ad3de77135755846cddc27e3827a326cb977c4`
- `upstream-u05-5a-cdp-results.json` sha256 `006d8bb875c2f5f43e6aabb18e94e97834bd0bc73426eb7f5d78e210f33ab5ee`
- `verifier-u05a-cdp-probe.mjs` sha256 `26c004fb5e0f5b1ebfa677bb5c5908f946264c14f19f2ded9ec1788729098751`
- `verifier-u05a-cdp-results.json` sha256 `4ab2f517cd65aaf90d8402a2575af639b119f5e31ee01c1a7b1cbb9efc05407f`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py`（`9761` 文件 `<= 2000` 行，Rust 非测试源码 `<= 1000` 有效行）与 `scripts/check-native-layering.py`（`0` 上行引用）通过；`RUSTFLAGS="-D warnings"` 原生（5 原生 crate `--all-targets`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44/44` 块（`40` 非空套件）、`3,055` passed、`0` failed、`6` ignored（含 `g1_shahai` `336/0/0`〔含 `u05_b5a_gwent_cards_fixes` `22/0/0`、`g1_shahai_gwent` `7/0/0`、`c331_source33_integration`〕、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12/12` 块（`11` 非空套件）、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3/3` 块、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4/4` 块、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178/178` 块（`146` 非空套件）、`8,362` passed、`0` failed、`8` ignored。
- `scripts/test_audit_upstream_skill_flags.py`：`6/6` tests OK；`audit_upstream_skill_flags.py` 11 项 `5a` 卡牌伴生技能检查均 `exit 0`。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`wasm-pack` 部署 `native-pkg`/`wasm-pkg` 后 `smoke:native` `83/83` PASS（`failed: 0`、`sources=37/37`、`packages=46/46`）、`build`（`244` 模块 `vite build`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`engine-cli` / `wasm-hash.mjs` 跨端校验 `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)。
