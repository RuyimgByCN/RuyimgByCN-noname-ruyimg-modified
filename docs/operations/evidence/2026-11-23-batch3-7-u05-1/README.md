# Batch U05-1 收口与验收证据（2026-11-23）

- **基线 HEAD**：`5de733e5e9209320500a9064a5d73f7e7970b537`（含 Batch Q8/Q9；同步合入其未提交的 `ci-runs.md` `5de733e5e9` CI 实测行）
- **合入来源**：`swarm/conv-u05-1`（`bd43569e1..b4fd79501bd0b6be1ff68d13df2d695420f4329c`，95 项候选范围）
- **独立复核裁决**：`verifier-l11` `#49 APPROVE`（审查 `b4fd79501`；逐项核定 `87` 项升格为 `已复核等价`、保留 `8` 项不变）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（87 项升格 + 8 项边界保留）

- **升格为 `已复核等价`（`87` 项，全部 `reviewer="verifier-l11"`、`review_date="2026-11-23"`、`test_name="u05_b1_fixes + verifier-l11-cdp-probe.mjs"`）**：
  - `33` 张卡牌（`caochuanjiejian`、`changshezhen`、`chenhuodajie`、`dizaizhen`、`fengyangzhen`、`fudichouxin`、`geanguanhuo`、`hongshui`、`huoshan`、`huyizhen`、`longfeizhen`、12 张 `mtg_*`、`niaoxiangzhen`、`pozhenjue`、`shepanzhen`、`shezhanqunru`、`shuigong`、`suolianjia`、`tianfuzhen`、`toulianghuanzhu`、`wuxingpan`、`yunchuizhen`）
  - `2` 项配置（`rand`、`wuxing`）
  - `12` 名武将（`diy_caocao`、`diy_lukang`、`diy_luxun`、`diy_tianfeng`、`diy_xuhuang`、`diy_yuji`、`diy_zhouyu`、`diy_zhugeke`、`mtg_ayala`、`mtg_jiding`、`mtg_jiesi`、`mtg_lilianna`）
  - `40` 个技能（`congjian`、`cuihui`、`diychanyuan`、`diyguhuo`、`dyjianxiong`、`guhuo2`、`guihan`、`guizhan`、`huosha`、`jiaoxia`、`jieyan`、`lingyong`、`luweiyan`、`luweiyan2`、`mbaizhan`、`mduohun`、`mhuanyi`、`msilian`、`msuoling`、12 个 `mtg_*_skill`、`mtongnian`、`shaoying`、`suolianjia`、`wuxingpan_skill`、`xicai`、`xinfu_qiaosi`、`xiongzi`、`yaliang`、`yiesheng`、`zonghuo`）
- **未改动条目（`8` 项）**：
  - `U05:configs:tip` 保持 Batch UI-7 `已复核等价`（`verifier-ui`，`2026-11-21`）。
  - 保留 `7` 项为 `简化近似`：`U05:cards:qixingzhen`（`SwdQixingzhenCard` 仍沿用 `swd_qixingzhen_shan` 服务 `g1_shahai_zhenfa.rs:360`）、`U05:generals:diy_caiwenji`（主线 `host_skill_bindings.rs:24` 与 `nc08_upstream_yunchou_beige.rs` 将 `"beige"` 绑定到 `olbeige`/`YunchouBeige`，而非 `character/shenhua/skill.js:4814` 原版 `beige`）、`U05:skills:stone_chenshui`（无 `GameMode::Stone`/`getLeader()`，沿用 `u05_stone_leader` 标记与牌堆/弃牌堆检索）、以及 `U05:configs:{mtg,zhenfa,gwent,yunchou}`（子包开关未由 `start_locked_engine_with_shahai_config` 过滤）。
- **最新计数**：`U05` **`88 / 1,533 / 0`**（`1,621` 项）；全局 **`2,408 / 2,016 / 0`**（`4,424` 项）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突、行数拆分与回归修复

- **冲突合并（7 项 `UU`）与模块拆分**：完成 `vocabulary/host/*`、`known_prompts.rs`、`catalog/hooks.rs`、`judge*.rs`、`dying.rs`、`equipment.rs`、`duel.rs`、`intrinsic.rs`、`nullify.rs`、`tricks.rs`、`machine.rs`、`modifiers.rs` 等 3-way 合并；按行数约束拆分 `crates/content-native-loader/src/generals_bridge.rs` 与 `crates/engine-native-kernel/src/snap/asks/standard_skills.rs`。
- **集成回归修复 1（`crates/engine-native/tests/u01_fixes_3/part3_d2.rs`）**：`u01_fix3_14_dddlianer` 与 `u01_fix3_15_dddanzhi` 原通过 `find_card_by` 寻找黑桃/梅花 `Wuzhong` 命中了 `decision34_pack9.rs` 的 `mtg_duzhao`；Batch U05-1 升级该牌为开启 `u05_mtg_duzhao_target` 选择后改为使用标准黑色 `CardKind::Chain`（`blk_chain`，目标 `[2]`）。
- **集成回归修复 2（`MachineSnapshot::Jiedao` 快照往返）**：Batch U05-1 让 `Jiedao`（`play_card.rs:603-604` / `tricks.rs`）正确识别装备栏 `EquipSlot::Weapon` 武器后，`game-server` `adr0787_timeout` 种子局首次触发借刀挂起状态并暴露 `MachineSnapshot::Jiedao` 缺失恢复/问询推导；已在 `snap/restore.rs:24`、`snap/asks.rs:878` 与 `snap/asks/standard_skills.rs:81` 补齐 `JiedaoStage::{Ask, Read}` 与 `"jiedao_sha_or_give"` 恢复及问询。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-1/`）

- `verifier-l11-cdp-probe.mjs` sha256 `33e8bbbe1657f8831dbbbcb20723bd83f78eca63e9eee5830a0376f5f4caa226`
- `verifier-l11-cdp-results.json` sha256 `e4f67d8e39df5d52c7aadaa96846392a1f4cd4e18eec1b596d616efa0d9f39b9`
- `verifier-u05a-cdp-probe.mjs` sha256 `7664bc69e28f7188cb7fb34f399c6016970345106ad22f67ba2b13af2d4a08bc`
- `verifier-u05a-cdp-results.json` sha256 `ea2a4fa63e7e4f84f2e51d9961aa7f0e7b7c0216d30e0bd485c08b6fefb84f25`
- `probe-u05-1-r2.mjs` sha256 `2a5bd7f19d77eea2a10b2c089d1f6b9aec9ffd82bde0ca599d5da19f3807c573`
- `probe-u05-1.mjs` sha256 `8926718656b1d7c309e19a71f58727d107aca9d4b50fc724216179ff0099846e`
- `upstream-u05-1-cdp-results.json` sha256 `ca823c8e5b4ff819e312022e3cd6fd2c7c309ade45242976306c4e68d9edfa81`
- `upstream-u05-1-r2-cdp-results.json` sha256 `9d9e260359610ad0f2b1197b62c0f263889f43b5dba0e8f240eba2ffd0f18a1d`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44/44` suites，`2,921` passed、`0` failed、`6` ignored
- `cargo test --offline -p content-native-loader --no-fail-fast`：`12/12` suites，`48` passed、`0` failed
- `cargo test --offline -p engine-native --test g1_shahai -- u05_b1_fixes`：`24` passed、`0` failed（`181` filtered out）
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance --no-fail-fast`：`3/3` suites，`4` passed、`0` failed
- `cargo test --offline -p desktop-client --no-fail-fast`：`4/4` suites，`20` passed、`0` failed、`1` ignored
- `cargo test --offline --workspace --no-fail-fast`：`178/178` suites，`8,228` passed、`0` failed、`8` ignored
- `apps/web-client`：`typecheck` 通过；`test` `540/540` pass；`build:native` + `smoke:native` `83/83` PASS（`failed: 0`）；`build`（`vite build` `244` modules）通过并还原 `apps/web-client/dist/`
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996...`、欢乐包 20 步 `42d74566...`）；`scripts/cross-target-hash.sh` `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`）
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)
