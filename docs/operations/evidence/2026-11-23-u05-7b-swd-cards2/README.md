# Batch U05-7b 证据摘要：U05 杀海拾遗 SWD 卡牌 Part 2（40 卡牌 + 64 技能 = 104 项等价，4 项技能保留近似）

- **日期与基线**：`2026-11-23` · 基于主线 `61ca3573d3a0aff5d0e189b703b3afed2f898d8e`（Batch `U05-7a` 已提交并发布，CI Run [`45233301007`](../ci-runs.md) Attempt 1 `10/10 success`），三路合入 `swarm/conv-u05-7b-swd-cards2`（`2099ce1a4..e7ff9868d41813f6d8bb667e79fd345014e04713`，`verifier-u05-b` `#86 APPROVE`）。
- **版本锁与冻结资产不变量**：
  - `STANDARD_RULESET_VERSION = 333`（[standard.rs](../../../../crates/engine-native-content/src/content/standard.rs#L168)）、`ARCHIVE_FORMAT_VERSION = 107`（[archive.rs](../../../../crates/engine-native-kernel/src/archive.rs#L120)）、`CONTENT_SCHEMA_VERSION = 5` / `RULE_VM_ABI_VERSION = 759`（[version.rs](../../../../crates/engine-api/src/version.rs#L6-L17)）、`NETWORK_PROTOCOL_VERSION = 8`（[lib.rs](../../../../crates/engine-protocol/src/lib.rs#L18)）。
  - 14 条标准 golden 向量哈希、`u01-dddanzhi-pending-v107.ngsa`、整包 8 维签字 `0/37` 与 `.git/config` SHA-256（`f862ba299c2bbd23c501ebeb0830f53caabc990a4da7e8005570470ff914bc25`）全部零改动。

## 1. 主线合并、R3 对齐与代码组织

- **行为与提示词接入**：新增 `HostBehavior` `1310..=1339`（`U05SwdXiangyuyeCard..=U05SwdTianxianjiu`，[behavior.rs](../../../../crates/engine-native-kernel/src/vocabulary/host/behavior.rs#L1830-L1889)，`HostBehavior::ALL = 676`），[known_prompts_u05_7b.rs](../../../../crates/engine-native-kernel/src/archive/known_prompts_u05_7b.rs#L3-L26) 尾追加 `22` 条提示词（`u05_xiangyuye_discard..=u05_guiyanfadao_confirm`），挂载 `swd_u05_7b.rs` / `swd_u05_7b_asks.rs` / `swd_u05_7b_hooks.rs` / `swd_u05_7b_lingjian.rs`、`extension_choices_u05_7b.rs` 与 `u05_b7b_swd_cards/*`（`25` 项测试），并将 `decision34_round4_u05_shahai.rs` 剩余 VM 下界校准为 `>= 80`（[decision34_round4_u05_shahai.rs](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L538)）。
- **主线集成与 R3 复核修正**：
  - [shahai_attach.rs](../../../../crates/content-native-loader/src/shahai_attach.rs)：修正 `"shahai-hearth"` / `"shahai-swd"` 精确包名匹配，并使 `swd_beh` 依次查询 `lookup_swd_u05_7a_behavior` / `lookup_swd_u05_7b_behavior`；上游 `qipoguyu` 的 `skills: ["xujin"]` 为未定义孤儿引用，故空 `equip_skills` 与上游等价。
  - [host.rs](../../../../crates/engine-native-content/src/content/host.rs)：`host_on_outer_settled_all` 顺序派发 `host_on_outer_settled` 与 `on_outer_settled_u05_7b`，保持与 `hearth` / `U05-6b` 钩子共存且消除死代码警告。
  - [swd_equipment_skills.rs](../../../../crates/engine-native-content/src/content/host/step/swd_equipment_skills.rs#L785)：`should_defer_yuchan_swap` 将 `g_yuchan_swap` / `yuchan_swap` / `host:yuchan_swap` 延后至 `Stage::Done`（仅保留测试桩 `c331_phase5_yuchan` 于 `AfterTriggers`），修复 `Sha` 濒死结算前手牌交换导致 `part6_n04e` 失配，并保持 `Caoyao` 自救。
  - [play_card_targets.rs](../../../../crates/engine-native/src/engine/play_card_targets.rs#L442) 与 [play_card.rs](../../../../crates/engine-native/src/engine/play_card.rs)：统一 `validate_tail_ext_targets` 目标校验并保留 `try_recast_l11_trick` / `push_play_card_frame`；`turn.rs` / `turn_handoff.rs` / `turn_reset.rs` / `frame_hooks.rs` / `equipment.rs` / `tricks.rs` / `sha/*` 完整协同多批并发旗标。
  - [combat_rules.rs](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs#L224) `is_tao_like_core`、[dying.rs](../../../../crates/engine-native-kernel/src/kernel/dying.rs)、[shapes.rs](../../../../crates/engine-native-kernel/src/snap/asks/shapes.rs) 与 [snap/asks/sha.rs](../../../../crates/engine-native-kernel/src/snap/asks/sha.rs#L35)：统一濒死救命与 `Tianxianjiu` 仅自救门控，补齐 `sadengjinhuan_judge_confirm` / `sadengjinhuan_extra_shan` 存档恢复推导，并对齐 `nc08_upstream_jijiu.rs` 与 `nc08_upstream_qingguo.rs`。

## 2. 清单升级与保留近似边界

- **精确 `104` 项升级（`已复核等价`）**：[inventory_status.json](../../../migration/g0/inventory_status.json) 将 `#86` 批准的全部 **`40`** 张 SWD Part 2 卡牌（`shenmiguo`..`qiankundai`，`card/swd/card.js`）与 **`64`** 项伴生/灵剑/锻造/装备技能（`nvwashi`..`tianxianjiu`，含 `kongxin`/`kongxin2`、`qinglianxindeng`、`hslingjian_*`、`jiguanyaoshu_skill`、`lingjianduanzao`/`_lingjianduanzao`、`sadengjinhuan`、`guiyanfadao`/`2`、`tianxianjiu` 等，`card/swd/skill.js`）升为 `已复核等价`。
- **严格保留为 `简化近似` 的 `4` 项技能**：
  1. `U05:skills:yiluan`（`card/swd/skill.js:1328`）：`swd_card_effects.rs` 选取手牌首张与首个合法座次，替代上游随机可用手牌与 `randomGet()`。
  2. `U05:skills:old_longfan`（`card/swd/skill.js:2403`）：`swd_u05_7b.rs` 采用 10 桶 RNG 抽取替代交互式 `setInterval` 轮盘 UI。
  3. `U05:skills:longfan`（`card/swd/skill.js:2531`）：`swd_equipment_skills.rs` 梅花分支取任意存活他者而非 `getEnemies()`（并保留装备回退）。
  4. `U05:skills:touzhi`（`card/swd/skill.js:2580`）：`swd_equipment_skills.rs` 在致死伤害后仍尝试将武器移入目标手牌。
- **统计更新**：`U05` 升为 **`1,293 / 328 / 0`**（合计 `1,621`）；全仓更新为 **已复核等价 `3,613`／简化近似 `811`／缺失 `0`／上游未定义 `0`**（合计 `4,424`）；整包 8 维签字维持 **`0/37`**。

## 3. Headless Chrome CDP 探针与全量门禁实测（2026-11-23）

- **上游 Chrome CDP 实跑探针（`verifier-u05-b` `#86` 原始存档，位于 `docs/operations/evidence-raw/review-u05-7b/`）**：
  - `verifier-probe-u05-7b.mjs`（`docs/operations/evidence-raw/review-u05-7b/verifier-probe-u05-7b.mjs`，`sha256: c99a5fc70c039de4aaef4927f85b573a1f9f0ff59efd143be5a462f71121d082`）
  - `verifier-u05-7b-cdp-results.json`（`docs/operations/evidence-raw/review-u05-7b/verifier-u05-7b-cdp-results.json`，`sha256: 3063e214ff42949553765bc6696f2c24361306e1d5172320fa627436f07d2b8a`）：`blockedRequestsCount = 0`，5 组探针全通过（`tianxianjiu` 独立酒池共存与伤后摸牌、`lingjiandai` 3 灵剑抽取、`yinmilichang` 临时潜行阶段初失效、非回合锻造失装禁重铸与 `jiguanyaoshu` 仅装备牌弃置、`kongxin` + `sadengjinhuan` 额外的闪 + `jinjilengdong_equip1` 摸牌先于翻面）。
- **主线 Stage A & Stage B 实测**：
  - 严格编译（`RUSTFLAGS='-D warnings'`）：`wasm32-unknown-unknown` `engine-native-wasm` 与 5 原生 crate `--all-targets --no-run` 零警告通过。
  - 原生与全仓单测（`CARGO_INCREMENTAL=0 --offline`）：4-crate（`44` 块、`3,145/0/6`，其中 `g1_shahai` `426/0/0`〔含 `u05_b7b_swd_cards` `25/0/0`〕、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`、`nc08` `142/0/0`、`nc08_upstream_a_k` `141/0/0`、`nc08_upstream_l_z` `209/0/0`）、`content-native-loader`（`12` 块、`48/0/0`）、`conformance`（`19/19` + `3` 块 `4/0/0`）、`desktop-client`（`4` 块、`20/0/1`）、`--workspace --no-fail-fast`（`178` 块、`8,452/0/8`）。
  - Web 与跨端一致：独立 `wasm-pack` 构建 `native-pkg` 与 `wasm-pkg` 后，`web-client` `typecheck`、`test`（`540/0`）、`smoke:native`（`83/0`）与 `build`（`244` 模块，随后恢复 `dist/`）全过；`check-native-wasm-cross-target.py` 轨迹双通过（`native_wasm_match=true`）；`engine-cli` vs `wasm-hash.mjs` 双种子一致（`4/42 = 8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`、`2/1 = 6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`、`M42 = 1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`、`chain=ok`、`skill-vm ok=true`）。
  - 仓库脚本门禁：`unittest discover -s scripts`（`207` 项 `OK, skipped=1`）、`test_check_source_lines.py`（`2` 项 `OK`）、`check-source-lines.py`（`<= 2,000` 总行、Rust 非测试 `<= 1,000` 有效行）、`check-native-layering.py`（`0` 上行引用）、`recount_g0.py`（`37` 源 `缺失=0`）与 `preflight.py` 全过。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)。
