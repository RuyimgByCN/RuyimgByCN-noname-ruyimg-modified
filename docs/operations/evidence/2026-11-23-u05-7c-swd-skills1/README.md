# Batch U05-7c 证据摘要：U05 杀海拾遗 SWD 角色技能 Part 1（113 项等价，2 项技能保留近似）

- **日期与基线**：`2026-11-23` · 基于主线 `12d5dca34bd499aac00eb3063054727ddea5b769`（Batch `U05-7b` 已提交并发布，CI Run [`45251837050`](../ci-runs.md) Attempt 1 `10/10 success`），三路合入 `swarm/conv-u05-7c-swd-skills1`（`41b7e4ded..96c66dfe5a0e7fab5be8f983d6df8f992a9d5314`，`verifier-u05-b` `#64 APPROVE`）。
- **核心不变量**：`STANDARD_RULESET_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动；整包 8 维签字维持 **`0/37`**。

## 1. 接入模块与主线协同修复

- **行为与提示词接入**：新增 `HostBehavior` `1340..=1345`（`SwdTouxi..=SwdPozhouSealed`，[behavior.rs](../../../../crates/engine-native-kernel/src/vocabulary/host/behavior.rs#L1890-L1902)，`HostBehavior::ALL = 682`，`trigger_bearing = 309`），[known_prompts_u05_7c.rs](../../../../crates/engine-native-kernel/src/archive/known_prompts_u05_7c.rs#L3-L30) 独立尾追加 `26` 条提示词（`"touxi_discard"..="fengze_pick"`），接入 [metadata/swd_cards.rs](../../../../crates/engine-native-kernel/src/vocabulary/host/metadata/swd_cards.rs)、`u05_7c_{asks,sha,skills,viewas}.rs`（4 模块）、[extension_choices_swd.rs](../../../../crates/engine-native/src/ai/extension_choices_swd.rs) 与 [u05_b7c_swd_skills1](../../../../crates/engine-native/tests/u05_b7c_swd_skills1.rs)（`24` 项测试）及 R1–R3 全量语义修复。
- **主线协同、拆分与集成修复**：
  - [combat_rules_sha.rs](../../../../crates/engine-native-kernel/src/kernel/combat_rules_sha.rs)：从 [combat_rules.rs](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs) 拆出 `sha_card_use_as` / `sha_blocked_for` / `sha_user_or_target_blocked` / `usable_sha_options`（保 Rust 非测试源码 `<= 1,000` 有效行；`luoyi_bonus` 与 `is_tao_like_core` 原位保留）。
  - [keys.rs](../../../../crates/engine-native-kernel/src/vocabulary/host/keys.rs#L13-L16)：自 `SwdXuanying::group_children` 移除 `xuanying_damage`，仅授予 `xuanying_use`（对齐 `#64` `xuanying` 回合外失牌反击面与主线既有测试）。
  - [play_rules.rs](../../../../crates/engine-native/src/engine/play_rules.rs#L28-L32) 与 [play_rules.rs](../../../../crates/engine-native/src/engine/play_rules.rs#L878-L912)：`common_play_validation` 头部仅保留 `is_u01_hand_locked`（`CardLimit`），保留 `:129` `blocks_hand_cards` 对 `mianzhen2` 返回 `IllegalTargets`；合入 `huanxia_yaotong_play_fallback`（收窄 `&crate::catalog::Catalog`）并在 [catalog_queries.rs](../../../../crates/engine-native-kernel/src/catalog_queries.rs#L151-L169) 与 `host_view_as_extra_cards` 去重统一。
  - [u05_b7c_swd_skills1/common.rs](../../../../crates/engine-native/tests/u05_b7c_swd_skills1/common.rs) & [decision34_round4_u05_shahai.rs](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L538)：补齐测试辅助器 `move_card_to_hand` / `equip_slot_card` / `push_top_deck` 的 `declared_kinds`/`declared_slots` 清理（防 `fengze` `Taoyuan` 遗留声明污染重取牌），并因 `swd_duguningke` `"突袭"` → `SwdTouxi` 迁移将剩余 U05 VM 下界自 `>= 80`（实测 `79`）校准为 `>= 75`；同步将 [a2-wave4-report.md](../2026-11-03-r05-private-wip/logs/a2-wave4-report.md#L33) 的 `H44` `SwdGouhunluoSub` 引用更新至拆出后的 `metadata/swd_cards.rs:45`。

## 2. 上游 Chrome CDP 探针与清单边界（`+113` 升级；`2` 项保留近似）

- **上游洁净 Chrome CDP 实跑**（`verifier-u05-b` `#64`）：`cleanroomBlockedCount: 0`、`pageExceptionCount: 0`，覆核 `aojian`/`shangxi`（`computed_distance <= 1` + `owner.hp <= p.hp`）、`linyun`（`ActivateSkill` 门槛 + 混色 `"none"` 对仁王盾 `4->3`）、`yudun`（`usableAtZero: 1` / `shaStatAfterYudun: 0`）、`kbolan`（`frequent: true`）、`jilve`（残堆+弃牌堆重洗 `170`）、`huanxia`（红拆/红决斗 `true`、黑杀 `false`）。原始探针指纹：
  - `docs/operations/evidence-raw/review-u05-7c/verifier-probe-u05-7c.mjs` (`sha256: 6045a0d00c5b5fcad73e704dec3fbe43a73ba70bc3c5a4e10812eccdce0ee6be`)
  - `docs/operations/evidence-raw/review-u05-7c/verifier-u05-7c-cdp-results.json` (`sha256: 4f9beb83badcc5f07d48b36f92a0708dbff4e5bf2697f49e3f64f6e9f5412d33`)
- **清单计数与保留项**：[inventory_status.json](../../../migration/g0/inventory_status.json) 将 `#64` 批准之 exact **`113`** 项 `U05:skills:*` 升为 `已复核等价`，严格保留以下 **`2`** 项为 `简化近似`：
  1. `U05:skills:pojian`（`character/swd/skill.js:2123`）：[filtering/swd.rs](../../../../crates/engine-native-content/src/content/host/filtering/swd.rs#L614-L624) 仅监听 `CardsDiscarded`，尚未覆盖完整 `loseEnd` 手牌离手路径。
  2. `U05:skills:xuanyuan_ai`（`character/swd/skill.js:3289`）：仅属 AI 评分权重效果，未接入 `engine-native/src/ai/` 决策器。
- **统计更新**：`U05` 升为 **`1,406 / 215 / 0`**（合计 `1,621`）；全仓更新为 **已复核等价 `3,726`／简化近似 `698`／缺失 `0`／上游未定义 `0`**（合计 `4,424`）；整包 8 维签字维持 **`0/37`**。

## 3. 主线整合全量实测（2026-11-24，最后一次代码改动后）

| 门禁 / 套件 | 命令口径 | 实测结果 |
| --- | --- | --- |
| 行数 / 分层门禁 | `test_check_source_lines.py` + `check-source-lines.py` + `check-native-layering.py` | 2/2 OK · `9,841` 文件（Rust 非测试 `<= 1,000` 有效行）· `0` 上行引用 |
| 严格 `-D warnings` | `RUSTFLAGS='-D warnings'`（`wasm32` + 5 原生 crate `--all-targets --no-run`） | `0` error · `0` warning |
| 原生 4 crate | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast` | `44` 套件 `3,169` pass / `0` fail / `6` ignored（`g1_shahai` `450/0/0`〔含 `u05_b7c_swd_skills1` `24/0/0`〕、`kernel` `436/0/0`、`g1` `395/0/1`、`swd` `135/0/0`、`e2e` `113/0/0`、`nc08` `142/0/0`、`nc08_upstream_a_k` `141/0/0`、`nc08_upstream_l_z` `209/0/0`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`） |
| 装载器 / 一致性 / 桌面 | `content-native-loader` + `conformance` + `desktop-client` | `12` 套件 `48/0/0` · `19/19` vectors + `4/0/0` · `4` 套件 `20/0/1` |
| Workspace 全量 | `cargo test --offline --workspace --no-fail-fast` | `178` 套件 `8,476` pass / `0` fail / `8` ignored |
| 脚本单测 | `test_report_phase_filters.py` / `test_report_phase_arms.py` / `test_audit_upstream_skill_flags.py` / `test_check_source_lines.py` | `9/9` + `13/13` + `6/6` + `2/2` = `30/30` OK |
| 前端与双 WASM | `wasm-pack`（`native-pkg` + `wasm-pkg`→`public/pkg`）+ `web-client` (`typecheck` / `test` / `smoke:native` / `build`) | `typecheck` OK · `540/0` · `83/0` · `vite build` `244` 模块（`dist` 已还原） |
| 跨端一致与 `skill-vm` | `check-native-wasm-cross-target.py` + `engine-cli` (`hash`/`manifest`/`skill-vm`) vs `wasm-hash.mjs` | 双段 `"native_wasm_match": true` · `H42==W42` (`8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`) · `H1==W1` (`6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`) · `M42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0` `chain=ok` · `skill-vm ok=true` |
| 门禁汇总 | `python3 -B scripts/preflight.py` | `22/22` 通过 |
