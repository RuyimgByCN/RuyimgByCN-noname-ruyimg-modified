# Batch P3-A 证据摘要：U05 杀海拾遗全部 31 项非冻结条目等价收口（保留 3 项墙钟 / Decision 40 边界）

- **日期与基线**：`2026-11-24` · 基于主线 `e8244d6ed8f14ad499a5bdf82949bafb67b354b7`（Batch `Q4-remaining` 已提交并发布，CI Run [`45303299654`](../ci-runs.md) Attempt 1 `10/10 success`），合并 `swarm/p3-u05-closure`（`e8b71bfc4..7d574912ecf29d8ef158b34388c7827271fd0c72`，`verifier-u05-b` `#231 APPROVE`）。
- **核心不变量**：`STANDARD_RULESET_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动；整包 8 维签字维持 **`0/37`**。

## 1. 接入修复、模块拆分与回归

- **SWD 遗留 `10` 技 + `8` 将闭合**：
  - [swd_card_effects.rs](../../../../crates/engine-native-content/src/content/host/step/swd_card_effects.rs)、[swd_cards.rs](../../../../crates/engine-native-content/src/content/host/step/swd_cards.rs) 与 [swd_equipment_skills.rs](../../../../crates/engine-native-content/src/content/host/step/swd_equipment_skills.rs)：`yiluan`/`xingdian` 基于确定性 RNG 与公共态度选目标；`longfan`/`touzhi` 对齐敌对态度与阵亡免收武器。
  - [filtering/swd.rs](../../../../crates/engine-native-content/src/content/host/filtering/swd.rs)、[swd_status.rs](../../../../crates/engine-native-content/src/content/host/step/swd_status.rs) 与 [metadata/swd.rs](../../../../crates/engine-native-kernel/src/vocabulary/host/metadata/swd.rs)：`pojian`/`anlianying` 接入完整 `loseEnd` 失牌；`jqimou` 补全摸牌后可选 `chooseToUse`；`lanzhi` 改为 `optional: true` + 公共态度 AI；`xuanyuan_ai` 接入公共信息 AI 评分（[extension_choices_swd.rs](../../../../crates/engine-native/src/ai/extension_choices_swd.rs)）。
  - [resume.rs](../../../../crates/engine-native-kernel/src/kernel/sha/resume.rs#L903-L908)：`resolve_yueren` 红判回收【杀】、落 `yueren2` 每回合上限，并按 `!is_tieji_sealed && !is_rechanyuan_sealed_name && !is_xj_fengyin_sealed` 封印门控（拆分封印查询至 [catalog_queries_seals.rs](../../../../crates/engine-native-kernel/src/catalog_queries_seals.rs) 保 Rust 非测试 `<= 1,000` 有效行）。
  - [swd_damage_combos.rs](../../../../crates/engine-native-content/src/content/host/step/swd_damage_combos.rs) 与 [asks_swd.rs](../../../../crates/engine-native-content/src/content/host/asks_swd.rs)：`swd_yijue_recover_confirm` 挂接目标玩家选项供 AI 评估公共态度，`yijue_target` 优先敌方/满手牌伤友，闭合 `swd_jiangwu`（连同 `swd_anka`/`swd_huanyuanzhi`/`swd_jiliang`/`swd_kama`/`swd_lanyin`/`swd_qi`/`swd_weida`）。
- **阵法 / 运筹 / 子包配置与炉石武将**：
  - [swd_zhenfa_cards.rs](../../../../crates/engine-native-content/src/content/host/step/swd_zhenfa_cards.rs)：`qixingzhen` 对齐真实围攻/联攻弃牌与虚拟【杀】。
  - [shahai_config.rs](../../../../crates/content-native-loader/src/shahai_config.rs) 与 [locked_content.rs](../../../../crates/content-native-loader/src/locked_content.rs)：`U05:configs:{gwent,mtg,yunchou,zhenfa}` 真实按配置开关过滤对应子包卡牌与武将。
  - [character.json](../../../../packages/extensions/shahai-yunchou-upstream-noname/assets/character.json) 与 [u05_b1_yunchou.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b1_yunchou.rs)：`diy_caiwenji` 悲歌分离至 `ycn_beige`（`host:u05_yunchou_char`）同时保留经典 `beige`。
  - [group_f.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b6b_hearth/group_f.rs) 与 [extension_choices_u05_6b.rs](../../../../crates/engine-native/src/ai/extension_choices_u05_6b.rs)：闭合 `stone_chenshui` 及 `hs_{alleria,jaina,kcthun,tgolem,wuther,xsylvanas}`（含 `xinwuyan` `exact_base_name` 归一化与非锦囊过滤、`xinliegong`、`rejieming`）。
- **新增真实事件与 AI 回归**：[part7_p3a_batch1.rs](../../../../crates/engine-native/tests/u05_b1_fixes/part7_p3a_batch1.rs)、[part9_p3a_batch1.rs](../../../../crates/engine-native/tests/u05_b6b_hearth/part9_p3a_batch1.rs) 与 [part6_p3a_batch2.rs](../../../../crates/engine-native/tests/u05_b7d_swd_skills/part6_p3a_batch2.rs)。

## 2. 上游 Chrome CDP 探针与清单边界（`+31` 升级；`3` 项维持近似）

- **上游洁净 Chrome CDP 原始产物**（`verifier-u05-b` `#231` 3 轮核验，`blockedRequestsCount: 0`；未入库目录，仅登记路径与 SHA-256）：
  - `docs/operations/evidence-raw/review-p3-u05/verifier-probe-p3-u05.mjs` (`sha256: 7d96d27d3a93bd22944266560bdf518eaa4100b110b6fe81fd597bfbb5b10fee`)
  - `docs/operations/evidence-raw/review-p3-u05/verifier-p3-u05-cdp-results.json` (`sha256: 67f570d95b3f5503d3d74ae5c848267c462478dcfdd6f1ba92470b438aa75d00`)
  - `docs/operations/evidence-raw/review-p3-u05/verifier-probe-p3-u05-r2.mjs` (`sha256: dbf41420224769dc89dfff2d9b0d4685ea93d8b2d774130422bce4682baff7f4`)
  - `docs/operations/evidence-raw/review-p3-u05/verifier-p3-u05-r2-cdp-results.json` (`sha256: f7e378a4fa66c38f1de5d8453ccb45082cb4079846d1ce0cf9e868ece45b5f7f`)
- **台账升级与保留边界**：[inventory_status.json](../../../migration/g0/inventory_status.json) 仅将 `#231` 批准的 **`31`** 项（`qixingzhen`、`configs:{gwent,mtg,yunchou,zhenfa}`、`diy_caiwenji`、`hs_{alleria,jaina,kcthun,tgolem,wuther,xsylvanas}`、`swd_{anka,huanyuanzhi,jiangwu,jiliang,kama,lanyin,qi,weida}`、`stone_chenshui`、`yiluan`/`longfan`/`touzhi`/`pojian`/`xuanyuan_ai`/`anlianying`/`jqimou`/`xingdian`/`lanzhi`/`yueren`）升为 `已复核等价`；严格维持 `3` 项为 `简化近似`：
  1. `U05:skills:shixu`（`character/hearth/skill.js:4877`）与 `U05:generals:hs_nozdormu`（`character/hearth/character.js:215`）：墙钟确定性边界。
  2. `U05:skills:old_longfan`（`card/swd/skill.js:2403`）：Decision 40 实时 UI 小游戏边界（200ms `setInterval` 4 位数字 DOM 虎口机滚轮实时点击停止，见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md)）。
- **统计更新**：`U05` 升为 **`1,618 / 3 / 0`**（合计 `1,621`）；全局 **`4,035 / 389 / 0`**（缺失 `0`，余留 `389` = `L11` `382` + `U05` `3` + `L26` `1` + `L28` `1` + `U01` `1` + `U02` `1`；整包 8 维签字维持 **`0/37`**）。

## 3. 主线整合本地全量实测（2026-11-24）

| 门禁 / 套件 | 命令口径 | 实测结果 |
| --- | --- | --- |
| 行数 / 分层 / 标记 / 卡种 / host 键门禁 | `test_check_source_lines.py` + `check-source-lines.py` + `check-native-layering.py` + `check_card_kind_compare.py` + `test_audit_upstream_skill_flags.py` + `report_host_keys.py --check` + `recount_g0.py` | 2/2 OK · `9893` 文件 `<= 2,000` 行（Rust 非测试 `<= 1,000` 有效行）· `0` 上行引用 · `check_card_kind_compare` OK · 6/6 OK · `U05` host 键 `315`（`shahai-yunchou` `17`）· `来源数=37 缺失=0` |
| 严格 `-D warnings` | `RUSTFLAGS='-D warnings'`（`engine-native-wasm wasm32` + 5 原生 crate `--all-targets`） | `0` error · `0` warning |
| 原生 4 crate | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast` | `44` 块（`40` 非 doc + `4` doc）`3,228` pass / `0` fail / `6` ignored（`g1_shahai` `482/0/0`、`kernel` `436/0/0`、`misc` `428/0/0`、`g1` `395/0/1`、`l_rounds` `318/0/0`、`nc_rounds` `311/0/5`、`nc08_upstream_l_z` `209/0/0`、`nc08` `142/0/0`、`nc08_upstream_a_k` `141/0/0`、`swd` `135/0/0`、`e2e` `113/0/0`、`engine-native` lib `16/0/0`、`engine-native-kernel` lib `7/0/0`、`engine-native-content` lib `5/0/0`、`engine-native-wasm` `27/0/0` + 集成） |
| 装载器 / 一致性 / 桌面 | `content-native-loader` + `conformance` + `desktop-client` | `12` 块 `48/0/0` · `19/19` vectors（`ALL PASS`）+ `4/0/0` · `20/0/1` |
| Workspace 全量 | `cargo test --offline --workspace --no-fail-fast` | `178` 块（`152` 非 doc + `26` doc）`8,535` pass / `0` fail / `8` ignored |
| 前端与双 WASM | `wasm-pack`（`native-pkg` + `wasm-pkg`→`public/pkg`）+ `web-client` (`typecheck` / `test` / `smoke:native` / `build`) | `typecheck` OK · `540/0/0` · `83/0` · `vite build` `244` 模块（`dist` 已还原） |
| 跨端一致与 `skill-vm` | `check-native-wasm-cross-target.py` + `engine-cli` (`hash`/`manifest`/`skill-vm`) vs `wasm-hash.mjs` | `standard` 40 步 (`6a59d996f801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`, `native_wasm_match=true`) · `huanle` 20 步 (`42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`, `native_wasm_match=true`) · `H42==W42` (`8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`) · `H1==W1` (`6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`) · `M42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0` `chain=ok` · `skill-vm ok=true` |
| 门禁汇总 | `python3 -B scripts/preflight.py` | `22/22` 通过 |
