# Batch U05-6b（`U05` 炉石传说 `hearth` Part 2 208 项等价 / 8 项保留）收口与验收证据（2026-11-23）

- **基线 HEAD**：`cc0401e8ec93c64098fc989e385ad2a30800ed86`（Batch U05-6a 已发布，CI Run `45126700137` Attempt 1 `10/10 success`；同步纳入其未提交的 `ci-runs.md` 实测行）
- **合入来源**：`swarm/conv-u05-6b-hearth-part2`（`e350ea568a709b6945a052a283ff39fac5baa545..92e1b4cacfc3afd0d1bf569f018af06535bd86eb`，基于 `e350ea568` 三路重放以保留主线 `U05-6a` Round 3 全部修复）
- **独立复核裁决**：`verifier-u05-b` `#45 APPROVE`（审查 Round 1–3 全量修复：批准 `208` 项升格 `已复核等价`，明确保留 `8` 项为 `简化近似`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（208 项升格 / 8 项明确保留）

- **升格为 `已复核等价`（`208` 项，全部 `reviewer="verifier-u05-b"`、`review_date="2026-11-23"`、`test_name="u05_b6b_hearth_fixes + verifier-probe-u05-6b.mjs"`）**：
  - `105` 项 `/hearth/` 剩余技能（除 `shixu` 外的 `anying`..`zuzhou`；协同 `qianghua2`/`shengyan2`/`fengxing2`/`bianxing2` 与 `stone_chenshui` 共 `111` 条原生条目映射于 `HEARTH_B6B_SKILL_MAP`：`HostBehavior` tags `1250..=1255` `U05Hearth2BatchA..=F`）
  - `103` 名 `hs_*` 炉石传说武将（`hs_aedwin`..`hs_zhouzhuo`，含依赖核心 `mashu` 的 `hs_brann` 与依赖 `U08` `qiangxi` 的 `hs_jgarrosh`）
- **明确保留为 `简化近似`（`8` 项：`1` 技能 + `7` 武将）**：
  - `U05:skills:shixu`（上游基于墙钟耗时，原生内核禁止非确定性墙钟改用状态旗标 `hs_shixu_duration > 3000` 确定性代理，见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md)）
  - `7` 名存在未闭合/简化依附技能之武将：`hs_nozdormu`（依附 `shixu`）、`hs_jaina`（`swd:huopu`）、`hs_kcthun`（`swd:luanji`）、`hs_xsylvanas`（`swd:busi`）、`hs_alleria`（`xinliegong` 桩）、`hs_tgolem`（`xinwuyan` 桩）、`hs_wuther`（`rejieming` 桩）
  - 继续保持 Batch U05-1 保留的 `7` 项边界（含 `U05:skills:stone_chenshui@main/content.js:397`、`U05:cards:qixingzhen`、`U05:generals:diy_caiwenji` 与 4 项子包开关）与 Batch UI-7 `tip` 状态不变。
- **最新计数**：`U05` **`726 / 895 / 0`**（`1,621` 项）；全局 **`3,046 / 1,378 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突与集成回归修复

- **三路合入与模块衔接**：以 `e350ea568` 为基准三路合入保留 `HEAD` `U05-6a` R3 修正（`hs_shihuawuqi_used_turn` 仅于 `host_on_phase_end_hearth` 清零、`tanmi`/`xiyong`/`shandianjian`/`qianfu2`），更新 `HostBehavior::ALL: 627`（新增 `1250..=1255`），尾追加 `known_prompts_u05_6b.rs`（`59` 条提示词），保持独立 `OptionalGate`（`resolution/optional_gate.rs`）、`judge_hearth.rs`、`extension_choices_u05_6b.rs` 并接入 `CardUsed` `BeforeTriggers` 按技能按需升窗（`maybe_promote_card_used_before` / `push_card_used_frame`）与 `ContentHooks` 3 项 Hearth 钩子。
- **主线回归修复 1（`virtual_diaobing_use` 双删，`g1` `u07` 3 项回归）**：`crates/engine-native-kernel/src/kernel/intrinsic.rs:285-291` 此前对 `virtual_diaobing_use` 调用 `.remove(...)` 再进入 `card_intrinsic`（`intrinsic_named_ext.rs:405-415` 再次 `.remove(...)`），导致虚拟筹度→调兵遣将退化为无中生有。已将 `kernel/intrinsic.rs:285-291` 改为 `.contains_key(...)` 仅判定是否转入 `card_intrinsic`，保留 `intrinsic_named_ext.rs` 单次消费语义。
- **主线回归修复 2（`stone_chenshui` `u05_stone_leader` 指向）**：`shahai_u05_hearth_b.rs` 将 `stone_chenshui` 映射至 `U05Hearth2BatchE`，而 `group_e.rs:661-664` 直接传入 `m.owner` 忽略 `u05_stone_leader`；已更新 [group_e.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b6b_hearth/group_e.rs#L661-L674) 先按 `u05_stone_leader` 解析 1-based 目标座次再从 `detached_cards` 发放梦境牌并维持物理牌守恒。
- **`decision34_round4_u05_shahai.rs` 下界校准**：`107` 项炉石技能自 VM 迁入 `HostBehavior` 后，`vm_skill_names` 实测余 `323` 项，校准断言下界至 `>= 300`（[decision34_round4_u05_shahai.rs](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L536-L541)）。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-6b/`）

- `probe-u05-6b.mjs` sha256 `ac4cb11fa0657d3ccc47b03d746b1ff833a146dafd5a610798d7a9cf323cf07e`
- `upstream-u05-6b-cdp-results.json` sha256 `43323755b20b36863afbdc23ac6698c8bb197a70809800fd6065c18e72409246`
- `verifier-probe-u05-6b.mjs` sha256 `364415431b957fe362cec664eabb3024ae562f6374716a98b0e1d25c3fe2895f`
- `verifier-u05-6b-cdp-results.json` sha256 `975915e461dbb4b0f6a0cd766cb6be82b2d4a497ec9bd5f621d3cad99a452421`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py`（`9740` 文件）与 `scripts/check-native-layering.py`（`0` 上行引用）通过；`RUSTFLAGS="-D warnings"` 原生（5 原生 crate `--all-targets`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44/44` 块、`3,033` passed、`0` failed、`6` ignored（含 `g1_shahai` `314/0/0`、`u05_b6b_hearth_fixes` `23/0/0`、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12/12` 块、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3/3` 块、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4/4` 块、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178/178` 块（`146` 非空套件）、`8,340` passed、`0` failed、`8` ignored。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`wasm-pack` 部署 `native-pkg`/`wasm-pkg` 后 `smoke:native` `83/83` PASS（`failed: 0`、`sources=37/37`、`packages=46/46`）、`build`（`244` 模块 `vite build`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`engine-cli` / `wasm-hash.mjs` 跨端校验 `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)。
