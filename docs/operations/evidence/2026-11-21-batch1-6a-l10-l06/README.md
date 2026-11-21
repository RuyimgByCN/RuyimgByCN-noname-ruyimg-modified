# Batch 1.6a（L06 wuxing ＋ L10 史诗卡牌）规则修复与独立复核摘要

- **基线 HEAD**：`e119f5518b32911ed45c3559881d97751966eddb`（Batch UI-4 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44929144693` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-q1-l10` `af6ef4899..b0a09c8a95a4358b44217f33ca90b5ab98ba690c`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁保持不变
- **独立复核员**：`verifier-rules-2`（未参与 Batch 1.6a 代码实现；经三轮源码对照与上游 headless Chrome CDP 实跑验证，在 `#70` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,182 / 3,242 / 0` -> **`1,186 已复核等价 / 3,238 简化近似 / 0 缺失 / 0 上游未定义`**（`L06 wuxing` 由 `2 / 1 / 0` 升为 **`3 / 0 / 0`**，`100%` 单项等价；`L10 史诗卡牌` 由 `8 / 9 / 0` 升为 **`11 / 6 / 0`**，4 项技能 `100%` 单项等价，UI 配置/钩子/样式 6 项保持 `简化近似`）；整包八项签署维持 **`0/37`**

---

## 1. 集成冲突协调（与 L29 / Q2-L27 / UI-1..4 共存）

1. **`crates/engine-native-kernel/src/kernel/combat_rules.rs`**：保留 L29 不含 `hand_banned` 语义与 L27 `blocks_all_cards` 濒死/响应检查，合并 `has_rechanyuan` 门控。
2. **`crates/engine-native-kernel/src/kernel/dying.rs`**：保留 L29 `mbcmjiushi_dying_skill` 濒死自救分支并同时整合 Batch 1.6a 抽出的 `dying_response_skills` 调度。
3. **`crates/engine-native-kernel/src/kernel/sha/step.rs` 与 `sha/respond.rs`**：将 L29 `mbcmfangzhu` 生效下的 `baiban` 与 `banned` 守卫完整迁入拆分后的 `sha/respond.rs`（`:306, :340`）。
4. **`HostBehavior` 与 AI 分派**：组合 L27 `L27MbcmQingzheng`（tag `654`）与 L10 `L10ScsCompanion`/`L10Rechanyuan`（tag `574..=575`），`HostBehavior::ALL` 更新为 **572** 项并双射通过；`crates/engine-native/src/ai/extension_choices.rs` 将 `plan_l10_choice_action` 与 `plan_l28_choice_action` 串联；修正 `crates/engine-native/src/lib.rs:60-61` 合并遗漏的花括号闭合；同步协调 `wuxing-upstream-liuli` 资产包与 `bundledNativePackages.ts`。

---

## 2. 独立复核 4 项升级清单（`#70` `APPROVE`）

| # | 清单 ID | 类别 | 原生落点与回归重点 | 复核结论 |
|---:|---|---|---|---|
| 1 | `L06:cards:wuxingpan` | `cards` | `crates/content-native-loader/src/wuxing.rs`、`wuxing.json` `0.3` 默认倍率、`WuxingNaturesRate` 内容锁隔离生克权重且排除 `wuxingpan` 自身赋属性、`tests/l06_wuxingpan_fixes.rs` | **已复核等价** |
| 2 | `L10:skills:reguhuo` | `skills` | `l10_reguhuo.rs`、动态 `filterCard`、`rechanyuan`、借刀双目标与 5 种响应上下文、`blocks_all_cards`（`new_zhixi`/`bingfeng2`）及 AI 猜牌 | **已复核等价** |
| 3 | `L10:skills:rangjie` | `skills` | `l10_epic_cards_fixes.rs`、`extension_choices_l10.rs`、`direct: true` `cancel2`＋场内移牌/摸牌及公有态 AI 选择 | **已复核等价** |
| 4 | `L10:skills:k_danggu` | `skills` | `l10_scs.rs` / `l10_scs_asks.rs` / `l10_scs_extra.rs` / `skills_recovery.rs` / `kernel/nullify.rs`：全 10 项 `scs*` 常侍伴生技（含 `scschihe` 无幻影加伤、`scsanruo` 反制无懈追踪 `nullify_stateplayer` 且零 schema 变更、`blocks_all_cards` 门控） | **已复核等价** |

---

## 3. Chrome CDP 上游原始证据索引（存 `evidence-raw/review-q1-l10/`）

- `8cd0ffc8136f8f6aa66a05e155669e3a29cbe351046dd401aa37a2749ee5ee48` — `docs/operations/evidence-raw/review-q1-l10/probe_l10_l06_upstream.mjs`
- `8fbf2aa0eebc09ccbc235d9c89c63a0bcc817946108665ae6898c8d793493cd9` — `docs/operations/evidence-raw/review-q1-l10/upstream-l10-l06-cdp-results.json`
- `f0940a2573e5f9bbf45b6093285069464d4c91ad0683458d915278f9ec28074b` — `docs/operations/evidence-raw/review-q1-l10/verifier_l10_r2_edge_cdp.mjs`
- `3cd4e177e5ba7ed2267e6b12bc6ebed311e499ef9282e44ce8c083de29af5352` — `docs/operations/evidence-raw/review-q1-l10/verifier-l10-r2-edge-cdp-results.json`

---

## 4. `main` 全量未过滤门禁实测记录

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-16a CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：exit `0`（已即时 `rm -rf` 清理）
   - `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`（`-D warnings`）：exit `0`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader`：30 组测试输出，**`2,600` passed / `0` failed / `6` ignored**
   - `cargo run --offline -p conformance`：**19 / 19 (`ALL PASS`)**；`cargo test --offline -p conformance`：3 组、`4` passed / `0` failed / `0` ignored
   - `cargo test --offline -p desktop-client`：4 组、**`20` passed / `0` failed / `1` ignored**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：`native_wasm_match: true`（`final_revision: 20`、`final_hash: 42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`bash scripts/cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：178 组测试输出，**`7,948` passed / `0` failed / `8` ignored**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`510` passed / `0` failed**
   - `pnpm --dir apps/web-client run build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`（`2 OK`）、`check-source-lines.py`（9,419 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_ledger_consistency.py --strict`（`台账与枚举一致`）、`recount_g0.py` 与 `preflight.py` 全过。
