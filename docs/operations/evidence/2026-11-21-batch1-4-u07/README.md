# Batch 1.4（U07 玩点论杀）规则修复与独立复核摘要

- **基线 HEAD**：`08ad0b37a447a21f515da955204923bdaf3e6aa8`（Batch 1.3 `U08 英雄杀` 已发布，CI Run `44735674711` Attempt 1 仅 `perf-macos-latest` `start-hash-p50` 宿主噪声失败、Attempt 2 `10/10 success`，两行已并入 `ci-runs.md`）＋ 合入 `swarm/author-u07-l10` `63c0411ddf3f0189395f224cb571a050c5d8588e`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁（`9_097_112_637_251_845_257` / `11_413_175_519_070_831_972`）保持不变
- **正式包与前端哈希同步**：`packages/extensions/wudian-upstream-noname/assets/cards.json` 补齐红桃 4 与方片 4 实体【调兵遣将】（`diaobingqianjiang`，`kind: "wuzhong"`，`deferred: []`，总张数 `14 -> 16`），`package.npack.json` 与 `apps/web-client/src/engine/bundledNativePackages.ts:347-355` 同步 blake3 `contentHash` 为 `50f279c395f6206136b74c2d3dbca89efdb2a04a4d0e533f8fd63c207db33692`
- **独立复核员**：`5a16b8ee-86c7-497d-97bf-3ba0a35aa54b`（未参与 Batch 1.4 代码实现；三轮核对与 Chrome CDP 实跑比对全部通过：**`14 / 14 PASS / APPROVE`**）
- **清单变更**：全仓 `643 / 3,781 / 0` -> **`652 已复核等价 / 3,772 简化近似 / 0 缺失 / 0 上游未定义`**（`U07` 由 `27 / 9 / 0` 升为 **`36 / 0 / 0`**；`+9` 项来自原先被撤回为 `简化近似` 的 `U07` 条目）；整包八项签署维持 **`0/37`**（队列 1 仍余 `L29`/`L10`/`U01` 与剩余来源实跑/修复及队列 2–11）

---

## 1. Round 1–3 修复与对齐要点

1. **`U07` 第一轮基线修复（接续 `u07_equivalence.rs`）**：
   - 补齐 `fuchou` 杀入弃牌堆与 `fuchou_marked` 按来源去重、`fuchou2` `phaseAfter` 全局触发及真实出杀/拒杀失血分支（`kernel/sha/resume.rs:19-107`、`wudian_status.rs:122-228`）。
   - `zhucheng2` 支持 `he` 区任意多选 `num = zhucheng.len()`、取消不回杀、多目标独立结算（`asks_wudian.rs:163-181`、`wudian_cards.rs:147-205,678-690`）。
   - `youzhan` 按 `shaBefore` / `shaMiss` 逐目标循环并持续追问虚拟【诱敌深入】，禁止额外增伤；`jinyan` 覆盖使用/打出【杀】、濒死与黑锦囊禁用；`choudu` 补齐 `he >= 2` 与有效目标；`diaobingqianjiang` 补入实体包牌并重写 `WudianDiaobingCard` 流程。
2. **Round 2 独立复核 6 项修复（`u07_round2_fixes.rs` `u07_r2_fix1..6`）**：
   - `sha_card_use_data`（`content/host/step.rs:44-68`）：`WudianFuchou2`、`WudianYouzhan` 与实体 `youdishenru` 反击【杀】统一保留 `Fire` / `Thunder` / `Normal` 杀属性。
   - 虚拟 `youdishenru` 无懈与清理：经 `TargetChoice` 与 `intrinsic.rs:212-235,344-350` 发 `CardUsed(Youdishenru)`（含无懈与反无懈窗，`snap/validation.rs:146-147` 支持挂起恢复），并在 `resolution.rs:781-796` 按真实出栈清理 `yd_pending_source`/`yd_pending_sha_frame`/`virtual_diaobing_use`。
   - 实体 `youdishenru` 反噬持卡与青龙刀复位：`WudianYouzhan` 保留 `self.card`（`Some(yd)` vs 虚拟 `None`），青龙追杀第二杀清理 `asking_fuchou`/`fuchou_card`/`yd_tried`/`yd_cancelled`（`kernel/sha.rs:404-405`、`kernel/sha/resume.rs:450-451`）。
   - `jinyan_blocks_black_trick`（`kernel/combat_rules.rs:355-365`）受 `!catalog.is_tieji_sealed(core, player)` 门控（非锁定技被铁骑封印时解除黑锦囊封锁且禁用黑锦囊转【杀】）。
   - `choudu`/`diaobingqianjiang` 完整对齐：`wudian_choudu_targets` 排除 `jileng_active` 与 `eval_card_use_canceled`（`wudian_diaobing.rs:406-434`）；`GameMode::Versus` 新增 `wudian_diaobing_dir`（`known_prompts.rs:622`）并扩展进程内 `PlayerCellView::team: Option<u8>`（`view.rs:68`、`views.rs:98`）供 AI 公开队别判断；`play_card_targets.rs:128-145` 严格校验 `selectTarget: -1` 全集/空集；用 `NO_FRAME` `NullifyWindowMachine`（`nullify.rs:268-337`）替代伪 `SkillToggled` 并在双出口清理 `diaobing_wuxie_active`。
   - `diyjuntun` 重铸非装备从牌堆顶抽取非装备牌（`wudian_cards.rs:10-58`）。
3. **Round 3 时序与失牌/弃牌修复（`u07_round2_fixes.rs` `u07_r3_*`）**：
   - `WudianDiaobingCard` 选牌替换阶段在 `JiefenTargetDiscard` 退让先结算 `CardsLost` 后再把亮牌置入目标手牌并推进下一目标无懈与 `content` 后段。
   - `WudianYouzhan` 在 `SelectCard` 弃牌后先经 `TargetChoice` 退让结算 `CardsDiscarded`（如 `WudianDuoqi`）后再发出虚拟 `CardUsed(Youdishenru)`。
   - `WudianDiyjuntun` 装备重铸调用 `discard_owned_card_once` 清除宣告槽、记失牌、发出手牌/装备 `CardsLost` 及单条 `CardDiscarded` 日志，并经 `DrawSettle` 退让后摸牌。

---

## 2. 独立复核 14 项逐案表（Round 3 `APPROVE`）

> 审查范围：`9` 项原 `简化近似` 升级为 `已复核等价` ＋ `5` 项既有 `已复核等价` 抽样复审（其余 `22` 项未受改动的 `U07` 武将/技能/卡牌沿用 `2026-11-12` `u07-verifier` `已复核等价` 并受 `g1_wudian` 全量回归覆盖）。

| # | 清单 ID (`U07`) | 类型 | 类别 | 上游位置 | 核对要点与原生回归 | 复核结论 |
|---:|---|---|---|---|---|---|
| 1 | `choudu` | 升级 | `skills` | `character/skill.js:586-615` | `he>=2`、`wudian_choudu_targets`（排 `jileng_active`/免疫）、先弃结算再发虚拟 `Wuzhong`；`u07_r2_fix5_*` | `PASS` -> **已复核等价** |
| 2 | `diaobingqianjiang` | 升级 | `cards` | `card/card.js:4-168` | 16 张正式包、`wudian_diaobing_dir`、`team` AI、strict `selectTarget:-1`、`NullifyWindowMachine`、`CardsLost` 先结算；`u07_r3_diaobing_*` | `PASS` -> **已复核等价** |
| 3 | `fuchou` | 升级 | `skills` | `character/skill.js:508-547` | 杀入弃牌堆、`fuchou_marked` 去重、多目标仅取消自己；`u07_fuchou_*` | `PASS` -> **已复核等价** |
| 4 | `fuchou2` | 升级 | `skills` | `character/skill.js:548-585` | `phaseAfter` 全局逐源、真实出杀选目标与火/雷属性保留、拒杀失血；`u07_fuchou2_*` + `u07_r2_fix1_*` | `PASS` -> **已复核等价** |
| 5 | `youzhan` | 升级 | `skills` | `character/skill.js:422-447` | 逐目标循环、火/雷杀属性、`TargetChoice` 先结算弃牌后再虚拟 `Youdishenru`；`u07_youzhan_*` + `u07_r2_fix1..2` + `u07_r3_youzhan_*` | `PASS` -> **已复核等价** |
| 6 | `youdishenru` (card) | 升级 | `cards` | `card/card.js:169-219` | 无懈/反无懈窗、火/雷属性、0 杀/取消/闪后反噬与退栈清理；`u07_r2_fix2..3_*` | `PASS` -> **已复核等价** |
| 7 | `youdishenru` (skill) | 升级 | `skills` | `card/skill.js:5-15` | 反噬保留实体/虚拟 `self.card` 区分、青龙二杀复位状态；`u07_youzhan_*` + `u07_r2_fix3_*` | `PASS` -> **已复核等价** |
| 8 | `zhucheng2` | 升级 | `skills` | `character/skill.js:218-251` | `he` 任意多选 `num=zhucheng.len()`、取消不回杀、多目标独立；`u07_zhucheng2_*` | `PASS` -> **已复核等价** |
| 9 | `jinyan` | 升级 | `skills` | `character/skill.js:448-507` | 使用/打出杀、濒死/已伤禁黑锦囊、`is_tieji_sealed` 解锁且禁黑锦囊转杀；`u07_jinyan_*` + `u07_r2_fix4_*` | `PASS` -> **已复核等价** |
| 10 | `g_youdishenru` | 抽样 | `skills` | `card/skill.js:16-36` | `youdishenru_use_options` 受 `jinyan`/铁骑制约及实体触发；`u07_youdishenru_*` + `u07_r2_fix3_*` | `PASS`（保持已复核等价） |
| 11 | `wdb_xizhenxihong` | 抽样 | `generals` | `character/character.js:42-48` | 蜀男 4 血 `[jinyan, fuchou]` 元数据与技能绑定；`u07_wdb_xizhenxihong_*` | `PASS`（保持已复核等价） |
| 12 | `diyjuntun` | 抽样 | `skills` | `character/skill.js:776-811` | `discard_owned_card_once` 装备重铸发 `CardsLost`/单日志、`DrawSettle` 与非装备摸牌；`u07_r2_fix6_*` + `u07_r3_diyjuntun_*` | `PASS`（保持已复核等价） |
| 13 | `zhucheng` | 抽样 | `skills` | `character/skill.js:160-217` | 回合结束置【筑】牌、专属牌堆往返存读档；`nc06_skill_pile_restore.rs` | `PASS`（保持已复核等价） |
| 14 | `kangyin2` | 抽样 | `skills` | `character/skill.js:391-407` | 亢音本回合无距离/杀次数 +1 与回合末清理；`u07_wdb_liuzan_*` | `PASS`（保持已复核等价） |

---

## 3. Chrome CDP 上游原始证据索引（存 `evidence-raw/`）

- `9d34d333fbf53b6e0563faeb99d9c77d5ead419ac0b3d433db7f1486a26d630b` — `docs/operations/evidence-raw/review-batch1-4/probe_u07_upstream.mjs`
- `33c5aa41255a9c50d05668e7bd3068ebcbc9cbc10ab9e3c5d7e93718b69de8f` — `docs/operations/evidence-raw/review-batch1-4/u07_upstream_cdp_results.json`
- `cde98f4e8fd25730a529b53805d2376ea596a972289ed8fa55025a3e9e65a3c9` — `docs/operations/evidence-raw/review-batch1-4/probe_u07_r2_upstream.mjs`
- `74095edbea5a4c59c05413d15ecddd2dee98766fd9b91822609b41058b4772e1` — `docs/operations/evidence-raw/review-batch1-4/u07_r2_upstream_cdp_results.json`

---

## 4. `noname-next` 本地第五节未过滤门禁实测记录

1. **脚本与静态检查**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：2 tests OK（exit 0）
   - `python3 -B scripts/check-source-lines.py`：9,384 文件 <= 2,000 行、Rust 非测试有效行 <= 1,000（exit 0）
   - `python3 -B scripts/check-native-layering.py`：0 upward references <= 0（exit 0）
   - `python3 -B scripts/check_doc_line_refs.py --strict`：10,889 处检查、0 越界（exit 0）
   - `python3 -B scripts/recount_g0.py`：来源数=37、缺失=0（exit 0）
2. **Rust 编译与单元测试**：
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`：**2,509 passed; 0 failed; 6 ignored**（18 个测试结果集 / 15 个非零测试集；含 14 标准 golden 全通过、`g1_wudian` `145` passed / `u07_` `26` passed、`u01_dddanzhi_regression` `20` passed、`test_registry` `1` passed）
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-batch1-4-lint RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：零警告通过（exit 0，临时目录已清理）
   - `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`：exit 0
   - **首次 `--workspace` 失败与修复**：首轮 `cargo test --offline --workspace --no-fail-fast` 仅 `apps/game-server/tests/native_product_path.rs:87`（`t13_generals_package_loads_via_native_path`）1 项失败，系该测试仍断言旧值 `14` 张实体牌（`left: 16`, `right: 14`，因本批在 `wudian-upstream-noname` 补齐两张实体【调兵遣将】后为 16 张）。将 `native_product_path.rs:87,108-109` 更新为 `16` 后，`cargo test --offline -p game-server --test native_product_path` **10 passed; 0 failed**。
   - `cargo test --offline --workspace --no-fail-fast`（独立复跑）：**7,905 passed; 0 failed; 8 ignored**（178 个测试结果集 / 146 个非零测试集，exit 0）
   - `cargo run --offline -p conformance`：**19/19 (`passed=19`, `ALL PASS`)**（exit 0）
   - `cargo test --offline -p desktop-client`：**20 passed; 0 failed; 1 ignored**（exit 0）
3. **前端与跨端 WASM**：
   - `pnpm --dir apps/web-client run typecheck`：exit 0；`pnpm --dir apps/web-client test`：**489 passed; 0 failed**
   - `pnpm --dir apps/web-client run build:native`：exit 0；`pnpm --dir apps/web-client run smoke:native`：**83 PASS / 0 failed**（含 `worker_all_37_sources_46_packages_locked_matrix` `37/37, 46/46`）
   - `python3 -B scripts/check-native-wasm-cross-target.py`：标准（`final_revision: 40`）与扩展（`final_revision: 20`）跨端轨迹均匹配（`native_wasm_match: true`）
   - `bash scripts/cross-target-hash.sh`：`start_4_42` (`8fe8316...`) / `start_2_1` (`6fb9d95...`) Native 与 WASM 哈希一致，`ALL CROSS-TARGET CHECKS PASSED`
   - `pnpm --dir apps/web-client run build`：exit 0（随后 `git checkout -- apps/web-client/dist/` 还原）
4. **最终提交前检查**：`python3 -B scripts/preflight.py` 全量 22 项通过（exit 0）。
