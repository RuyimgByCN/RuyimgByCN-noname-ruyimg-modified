# Batch Q2-L20+L22（L20 标记补充终 3 技 ＋ L22 段位制裁终 2 技 → 双源 100% 单项等价）规则修复与独立复核摘要

- **基线 HEAD**：`41b7e4dedeada725ecd08b3e4b3400285bd44470`（Batch UI-6 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44969821746` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-q2-l20-l22` `d9e1c9a1f..94f42d34e3c52677c07e489fd5f73346ae33c93f`（`86d565f3c`、`94f42d34e`）
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁保持不变
- **独立复核员**：`verifier-rules-2`（未参与 Batch Q2-L20+L22 代码实现；结合 6 项上游 headless Chrome CDP 实跑探针完成两轮审查，在 `#115` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,491 / 2,933 / 0` -> **`1,496 已复核等价 / 2,928 简化近似 / 0 缺失 / 0 上游未定义`**（`L20 标记补充` `92 / 3 / 0` -> **`95 / 0 / 0`**；`L22 段位制裁` `6 / 2 / 0` -> **`8 / 0 / 0`**；另刷新 4 项已复核等价条目的复核备注）；整包八项签署维持 **`0/37`**

---

## 1. 集成冲突协调与行数治理

1. **`crates/engine-native-content/src/content/host.rs`**：`kind_trigger_meta` 保留 `L19`（`L19DoomsdayBatchA`/`B`）与 `U01`（`D3Moyan`/`U01DddBatchA..D`）分支，并合入 `L20` `FzLiegong`/`L20MarkBatchA(wengua/wengua5)`/`L20MarkBatchB(fz_liegong/mibei/kuanshi/kuanshi2)` 运行期触发元数据。
2. **`crates/engine-native-kernel/src/state/core_ops.rs`**：`legal_targets` `Sha | FireSha | LeiSha` 采用 `self.in_attack_range(catalog, player, *t)` 统一对齐 `tspowei` 单向攻击范围覆盖且不污染 `computed_distance`。
3. **`crates/engine-native-kernel/src/kernel/events/vm_history.rs`**：新增 `events/vm_history.rs` 承接 `record_vm_history` 与 `record_dddanzhi_card` 并在 `events.rs` 原位导出，令 `events.rs` 保持低于 `1,000` 有效 Rust 行上限且 `0` 处文档行号越界。

---

## 2. 独立复核升级与复检清单（`#115` `APPROVE`）

| # | 清单 ID | 类别 | 原生落点与回归重点 | 动作与结论 |
|---:|---|---|---|---|
| 1 | `L20:skills:mibei` | `skills` | `l20_batch_b_mouyi.rs`、`host.rs`、`l20_batch_b.rs`（`use_as` 同名去重、排延时锦囊、deck-only `get.cardPile2` M1、`TurnEnded`/`TurnPhaseAfter` 过期不扣上限 M2；`l20_fix_mibei_*`） | **升级 → 已复核等价** |
| 2 | `L20:skills:fz_liegong` | `skills` | `host.rs`、`d34_l11_mods.rs`、`l20_batch_b.rs`（`(Normal, 0, true)`、`has_zhu_shouyue` 跨模式查主 F1、虚拟杀 `cid.unwrap_or(0)` 不可响应 F2；`l20_fix_fz_liegong_*`） | **升级 → 已复核等价** |
| 3 | `L20:skills:wengua` | `skills` | `l20_batch_a.rs`、`l20_batch_a_asks.rs`、`known_prompts.rs:259`（移除独立主动入口，对齐 `wengua5` `he` 与单牌牌堆不重洗 W1、`phaseUseEnd` 清零 W2、多持有者选人 W3；`l20_fix_wengua_*`） | **升级 → 已复核等价** |
| 4 | `L22:skills:_eloDraw` | `skills` | `elo.rs`、`zj_leiji.rs` 及全 19 处判定点（定制判定偏好、雷击 `BlackPositive` E1、19 处 `player.judge` 接线且保持非判定 peek 零漂移 E2；`l22_fix_elo_draw_*`） | **升级 → 已复核等价** |
| 5 | `L22:skills:_timeKiller` | `skills` | `elo.rs`（限 `chooseToUse`/`chooseToRespond`/`chooseToDiscard`，补齐 `jiedao`/`guanshi`/`qinglong` 等提示词白名单 TK1 且非目标询问零 RNG 漂移；`l22_fix_time_killer_*`） | **升级 → 已复核等价** |
| 6–9 | `L20`：`tspowei` / `kuanshi` / `kuanshi2` / `wengua5` | `skills` / `ui_hooks` | 修复 `dulie` 顺移与 `in_attack_range` 双向覆盖 T1、`kuanshi` 后置伤 `>1`/仅 `kuanshi2` 挂 `DamageBefore` 且可封 K1、`Prepare`/`Death` 清理不含 `Judge` K2 及 `wengua5` W1–W3 | **复检保持 → 已复核等价** |

---

## 3. Chrome CDP 上游原始证据索引（存 `evidence-raw/review-q2-l20-l22/`）

- `c85b85132a543cdd8a0fa0490acfa1d2ea7a707d0b0a1af01c47ffc7a5813106` — `docs/operations/evidence-raw/review-q2-l20-l22/probe-l20-l22.mjs`
- `9b53cc92703679a100fcfdc4472f8c8bf540cb675d6b68d52a4c7e2be5ff464c` — `docs/operations/evidence-raw/review-q2-l20-l22/upstream-l20-l22-cdp-results.json`

---

## 4. `main` 全量未过滤门禁实测记录

1. **Rust 编译与单元测试（`CARGO_INCREMENTAL=0`）**：
   - `RUSTFLAGS="-D warnings"` `cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`（临时 `/private/tmp/noname-target-q2-l20`，已清理）：exit `0`
   - `RUSTFLAGS="-D warnings"` `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm -p engine-native`：exit `0`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader`：30 组测试输出，**`2,670` passed / `0` failed / `6` ignored**（含 `l_rounds` **`156`** passed，含 14 项 `l20_l22_fixes::*`）
   - `cargo run --offline -p conformance`：**19 / 19 (`ALL PASS`)**；`cargo test --offline -p conformance`：3 组、`4` passed / `0` failed / `0` ignored
   - `cargo test --offline -p desktop-client`：4 组、**`20` passed / `0` failed / `1` ignored**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：`native_wasm_match: true`；`bash scripts/cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：178 组测试输出，**`8,018` passed / `0` failed / `8` ignored**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`520` passed / `0` failed**
   - `pnpm --dir apps/web-client run build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`（`2 OK`）、`check-source-lines.py`（9,459 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_doc_line_refs.py --strict`（`越界 0 处`）、`check_ledger_consistency.py --strict`（`台账与枚举一致`）、`recount_g0.py` 与 `preflight.py` 全过。
