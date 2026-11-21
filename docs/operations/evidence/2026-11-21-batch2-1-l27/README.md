# Batch Q2-L27（L27 选将美化）规则修复与独立复核摘要

- **基线 HEAD**：`d9e1c9a1fcea4f19f1555bedb416aee8b51fc9b8`（Batch UI-2 已发布，CI Run `44899269373` Attempt 1 `10/10 success`，已记入 `docs/operations/evidence/ci-runs.md`）
- **合入来源**：`swarm/conv-q2` `af6ef4899..f3b59ecf68392d610bba4fe1577ad9b74ad2f669`（`b5479d47a`、`771d2a2fc`、`f3b59ecf6`）
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁保持不变
- **独立复核员**：`verifier-rules`（未参与 Batch Q2-L27 代码实现；经三轮逐项源码比对与上游 headless Chrome CDP 实跑验证，在 `#47` 给出 **`APPROVE`**，`12/12` 规则相关条目 `0 discrepancies`）
- **清单变更**：全仓 `677 / 3,747 / 0` -> **`689 已复核等价 / 3,735 简化近似 / 0 缺失 / 0 上游未定义`**（`L27 选将美化` 由 `41 / 13 / 0` 升为 **`53 / 1 / 0`**：6 项 `skills` + 6 项 `ui_hooks` 升为 `已复核等价`；`L27:surfaces:e.css` 保持 `简化近似` 归 UI 线）；整包八项签署维持 **`0/37`**

---

## 1. 集成冲突协调（与 Batch 1.5 `L29 龙血玄黄` 并存）

1. `crates/engine-native-content/src/content/host/step/skills_damage.rs`：保留 `MbcmQingzheng`（L29）先弃发 `CardsLost` 后 `Flow::Yield` 再选目标花色及 `source: Some(owner)` 伤害；隔离并行 `L27MbcmQingzheng`（tag `654`）以保留 `:4259` 数组条件不造成伤害、在 `JiefenTargetDiscard` 缺花色兜底与 `CompareTargetPick` 设置 `l27_mbcmqingzheng_resolved`，并按 `771d2a2fc` 为 `L27MbcmQingzheng` 发出 `EventData::CardsDiscarded`。
2. `crates/engine-native-content/src/content/host/asks_mtg.rs`：修正补丁行偏移导致的 `CompareTargetPick` `allow_cancel` 误放行，恢复 `"mbcmqingzheng_target_suit"` 为 `allow_cancel: false`（与 `skills_damage.rs` 及快照恢复校验对齐），同时保持 `"mbcmqingzheng_suit"` 的 `allow_cancel: true`（`direct: true`）。
3. `crates/engine-native-kernel/src/kernel/combat_rules.rs`：保留 Batch 1.5 `is_trick_kind` 与 `baiban`/`jin` 逻辑，并在 `dying_tao_cards` 与 `response_kind_cards_core` 入口补齐 `blocks_all_cards` 检查。
4. `crates/engine-native-content/src/content/host/filtering/common.rs`、`docs/architecture/native-phase-filter-registry.tsv` 与 `scripts/{test_report_phase_filters.py,test_report_phase_arms.py}`：将并行匹配的 `HostBehavior::MbcmQingzheng | HostBehavior::L27MbcmQingzheng` 拆为两条独立分支，消除前置审计脚本漏记；同步登记第 39 行相位过滤（`Prepare:5 / Play:9 / begin=true:22`）及第 29 个相位钉死臂（`OK:24`），并修正历史证据过期行号引用（`21` 处）。

---

## 2. 独立复核 12 项升级清单（`#47` `APPROVE`）

| # | 清单 ID (`L27`) | 类别 | 原生落点与回归重点 | 复核结论 |
|---:|---|---|---|---|
| 1 | `xiefang` | `skills` | `l27_fixes.rs`、`skills_status.rs`、`core_ops_modifiers.rs`（`gameDrawEnd`/`dieEnd` 实时刷新，不与 `Phase::Draw` 重复） | **已复核等价** |
| 2 | `new_meibu` | `skills` | `skills_virtual_use.rs`、`l27_fixes.rs`、`filtering/common.rs`（`phaseUseEnd` 清理、虚拟转化） | **已复核等价** |
| 3 | `new_zhixi` | `skills` | `host.rs`、`skills_status.rs`、`combat_rules.rs`（0 起始计数、`blocks_all_cards` 含无懈/濒死求桃、`phaseUseEnd` 清理） | **已复核等价** |
| 4 | `clandianzhan` | `skills` | `l27_fixes.rs`（per-round 非取消花色掩码、重铸与横置/多目标过滤） | **已复核等价** |
| 5 | `reqimou` | `skills` | `l27_fixes.rs`、`d34_l11_mods.rs`、`extension_choices.rs`（全距不增攻击范围、选数后可取消、`LoseHp` 先于 `draw(X)`、AI 门控） | **已复核等价** |
| 6 | `bolan` | `skills` | `l27_fixes.rs`、`activation.rs`、`l27_batch_b_asks.rs`（`phaseUse` 动态池排除 trigger/已有技、多持有人选择） | **已复核等价** |
| 7 | `mbcmqingzheng` | `ui_hooks` | `skills_damage.rs`、`metadata/common.rs`、`asks_mtg.rs`（`L27MbcmQingzheng` tag `654` 与 L29 隔离、`direct:true`、`CardsDiscarded`、`:4259` 无伤） | **已复核等价** |
| 8 | `sbqingzheng` | `ui_hooks` | `l27_fixes_b.rs`、`l27_batch_b_asks.rs`、`extension_choices.rs`（严格 `3 - 治世` 花色、仅查目标手牌花色、独立加印） | **已复核等价** |
| 9 | `xingchong` | `ui_hooks` | `l27_fixes.rs`、`l27_fixes_b.rs`、`kernel.rs`、`tricks.rs`、`machine.rs`（`take_hand_card` + `CardsMoved` 覆盖顺手/弃/装备/响应） | **已复核等价** |
| 10 | `dcxiaoxi` | `ui_hooks` | `l27_fixes_b.rs`（距离/合法分支过滤、单分支自动及空牌堆虚拟【杀】） | **已复核等价** |
| 11 | `twfengji` | `ui_hooks` | `l27_fixes_b.rs`（`sha` 普通/火/雷同名归一化、仅牌堆检索、不足停抽） | **已复核等价** |
| 12 | `twzhouzu` | `ui_hooks` | `l27_fixes_b.rs`、`extension_choices.rs`（目标自选仅手牌 `'h'`、`Some(owner)` 1 点雷伤） | **已复核等价** |

---

## 3. Chrome CDP 上游原始证据索引（存 `evidence-raw/review-q2-l27/`）

- `715dec8b79e0a7fb4135346390967017c5c1866e27ddbf01340c71790c2fd058` — `docs/operations/evidence-raw/review-q2-l27/probe_l27_upstream.mjs`
- `1736ce50f321bf77f1e02732b6f154e2ab706e8600b64145a6082f8dba872997` — `docs/operations/evidence-raw/review-q2-l27/upstream-l27-cdp-results.json`
- `c852f9bdc322576d9371c69c292b562c2982d12b99133921154267045e3cf415` — `docs/operations/evidence-raw/review-q2-l27/verifier_l27_edge_cdp.mjs`
- `023cdbc8690af1bc42b98af9aa229ade500720981aab1da6a243aa3753dd8333` — `docs/operations/evidence-raw/review-q2-l27/verifier-l27-edge-cdp-results.json`（`runtimeExceptions: []`）

---

## 4. `main` 全量未过滤门禁实测记录

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-q2-final CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：exit `0`（已即时 `rm -rf` 清理）
   - `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`：exit `0`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader`：30 组输出，`2,586` passed / `0` failed / `6` ignored（其中 3 crate `2,538/0/6`，`l_rounds.rs` `116/0/0`，`content-native-loader` `48/0/0`）
   - `cargo run --offline -p conformance`：`passed=19`、`ALL PASS`
   - `cargo test --offline -p desktop-client`：`20` passed / `0` failed / `1` ignored
   - `python3 -B scripts/check-native-wasm-cross-target.py` & `bash scripts/cross-target-hash.sh`：`3` passed / `ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：首轮并发负载下 `nc10_ai_bounds::nc10_planner_is_invariant_under_deck_order` 曾因 `with_deadline_ms(50)` 触发单次波动，单独重跑全量未过滤 `cargo test --offline --workspace --no-fail-fast` 后 178 组输出 **`7,934` passed / `0` failed / `8` ignored**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `typecheck`：exit `0`；`pnpm test`：**`498/0`**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：
   - `python3 -B scripts/recount_g0.py`、`test_report_phase*.py`（`22 OK`）及 `python3 -B scripts/preflight.py`（全部 22 项，含 `test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`check_doc_line_refs.py --strict`、`check_evidence_budget.py`）全过。
