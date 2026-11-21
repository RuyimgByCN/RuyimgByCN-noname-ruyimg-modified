# Batch Q2-L19（L19 末日乱斗 9 项技能收口 → 69/69）规则修复与独立复核摘要

- **基线 HEAD**：`8a879f8e479406e3788011885eba2a16f0d96613`（Batch UI-5 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44944693459` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-q2-l19` `3686468ca..1b80982366cac7029f8f393f9c4679ef925eb994`（`c524c362f`、`57d9827dc`、`1b8098236`）
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁保持不变
- **独立复核员**：`verifier-rules-2`（未参与 Batch Q2-L19 代码实现；经三轮源码核验与上游 headless Chrome CDP 实跑比对，在 `#83` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,304 / 3,120 / 0` -> **`1,313 已复核等价 / 3,111 简化近似 / 0 缺失 / 0 上游未定义`**（`L19 末日乱斗` 由 `60 / 9 / 0` 升为 **`69 / 0 / 0`**，100% 单项等价）；整包八项签署维持 **`0/37`**

---

## 1. 集成冲突协调（与 Q2-L27 / Batch 1.6a 共存）

1. **`crates/engine-native-kernel/src/archive/known_prompts.rs`**：保留 `L27`/`L10` 全部已知提示词并追加 `"wujinmoshiRE1:confirm"` 与 `"wujinmoshiRE2:choose_skill"` 至表尾。
2. **`crates/engine-native-kernel/src/catalog.rs` 与 `crates/engine-native-content/src/content/{hooks.rs, host.rs}`**：将 `ContentHooks` 保留 L27/L10 钩子并组合引入 `on_recover_before` 与 `preserves_skills_on_death`（`on_round_start` 共用）。
3. **`crates/engine-native-content/src/content/host/step.rs`**：将 `l27_fixes::host_on_round_start` 与 `l19_fixes::host_on_round_start` 分别别名为 `host_on_round_start_l27` / `host_on_round_start_l19` 并在 `step::host_on_round_start` 顺序复合调用，确保 L27 轮标展示清理与 L19 `_relifeskill` 倒数复活互不覆盖。

---

## 2. 独立复核 9 项升级清单（`#83` `APPROVE`）

| # | 清单 ID (`L19`) | 类别 | 原生落点与回归重点 | 复核结论 |
|---:|---|---|---|---|
| 1 | `ganran2` | `skills` | `l19_batch_a.rs`、`decision34_l19_l20_l27.rs`（对齐注释禁用的 `enable:'phaseUse'`，Play 不可发动且无自动触发） | **已复核等价** |
| 2 | `_jiangshi2` | `skills` | `host.rs`、`l19_batch_a.rs`（`TurnBegin` pri 15 前置、`owner._tuizhi` 判定、先清 `_jiangshi_force_nj=0` 再置 `fz`、发 `Death` 不预清 0 `hp`、`Loyalist`+`fzjsNumber++`） | **已复核等价** |
| 3 | `_huzhu` | `skills` | `l19_fixes.rs`、`l19_batch_a*.rs`（`play_begin_l19`＋`huzhu_play_stamp` 每 `Phase::Play` 计次支持同回合两出牌阶段、二步取消返还） | **已复核等价** |
| 4 | `chanceIdentity10` | `skills` | `l19_batch_a.rs`、`l19_fixes.rs`（Play 禁用、`next_alive` 顺延 `alive > 10` 门控、`idx + 1` 非冲突槽位） | **已复核等价** |
| 5 | `chanceIdentity0` | `skills` | `l19_batch_a.rs`、`l19_fixes.rs`（同 `chanceIdentity10`，`idx + 1 == 1` 与 `chanceIdentity` 槽位彻底隔离） | **已复核等价** |
| 6 | `wujinmoshiRE1` | `skills` | `l19_batch_a.rs`、`known_prompts.rs`（移除濒死自救、敌死向 `game.me` 发 `confirm`/`choose_skill`、取消与 `Pass` 洁净放弃＋存档恢复） | **已复核等价** |
| 7 | `_wujinmoshiRecover` | `skills` | `l19_fixes.rs`（`on_recover_before` 合并单次 `HpRecovered` 且限 Seat 0；`RecoverE` 使用 `saturating_sub` 防负溢出；移除 `Phase::End` 误回血） | **已复核等价** |
| 8 | `_relifeskill` | `skills` | `kernel/death.rs`、`l19_fixes.rs`（保留自带技同时调用 `revoke_equip_skills` 撤销弃置装备技如丈八蛇矛；按座序复活摸牌、仅查 `zuan_hudun`） | **已复核等价** |
| 9 | `_guandieskill` | `skills` | `l19_batch_b*.rs`、`l19_fixes.rs`（主公无 `jingcheng` `Death pri -99`、`is_charlotte_skill` 豁免、单步 `zuanExp` 升级公式与合并 `hej` `CardDiscarded`） | **已复核等价** |

---

## 3. Chrome CDP 上游原始证据索引（存 `evidence-raw/review-q2-l19/`）

- `1f7a9c23f5cef4c46f00436945fa3747b80954f090dfc7b4106c9e436bad03fb` — `docs/operations/evidence-raw/review-q2-l19/probe-l19.mjs`
- `a0c29c5b55f5745e56f2ade5cc6ae835edc2c8ec4d5b889815a630b657560437` — `docs/operations/evidence-raw/review-q2-l19/upstream-l19-cdp-results.json`

---

## 4. `main` 全量未过滤门禁实测记录

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-l19 CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：exit `0`（已即时 `rm -rf` 清理）
   - `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm`（`-D warnings`）：exit `0`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader`：30 组测试输出，**`2,612` passed / `0` failed / `6` ignored**
   - `cargo run --offline -p conformance`：**19 / 19 (`ALL PASS`)**；`cargo test --offline -p conformance`：3 组、`4` passed / `0` failed / `0` ignored
   - `cargo test --offline -p desktop-client`：4 组、**`20` passed / `0` failed / `1` ignored**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：`native_wasm_match: true`（标准 `40`、扩展 `20`）；`bash scripts/cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：178 组测试输出，**`7,960` passed / `0` failed / `8` ignored**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`514` passed / `0` failed**
   - `pnpm --dir apps/web-client run build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`（`2 OK`）、`check-source-lines.py`（9,429 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_ledger_consistency.py --strict`（`台账与枚举一致`）、`recount_g0.py` 与 `preflight.py` 全过。
