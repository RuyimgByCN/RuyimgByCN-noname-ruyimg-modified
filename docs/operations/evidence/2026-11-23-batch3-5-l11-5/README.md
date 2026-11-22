# Batch L11-5（L11 太虚幻境系统/计数/武器覆写/领主与武将技共 +48→408/936）独立复核与集成摘要

- **基线 HEAD**：`6b8c847c6e5acbc878fd3fa4a333f53c5ed9298a`（Batch L11-4 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45056028805` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l11-b5` `c8247fb03e391e86ec7063868ed7e109e29a708a..3ca237e375e02f0f0652da6ab61892e85a994160`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-l11`（`#41` **`APPROVE`**，含 Round 2 校验与 Headless Chrome CDP）
- **清单变更**：全仓 `2,127 / 2,297 / 0` -> **`2,175 已复核等价 / 2,249 简化近似 / 0 缺失 / 0 上游未定义`**（`L11 太虚幻境` 由 `360 / 576 / 0` 升为 **`408 / 528 / 0`**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与主线对齐修复

1. **宿主标签、提示词与核心生命周期协同**（与 `L01` / `U01-3` / `L11-4` / `L25` / `L27` 共存）：
   - `vocabulary/host/*` & `engine-native/src/lib.rs`：保留 `L01` 标签 `860..=864`，在其后追加 `L11SystemHooks`（`739`）、`L11Weapons`（`740`）、`L11GeneralBossC`（`741`）与 `L11BossLingfeng`（`742`），同步 `HostBehavior::ALL = 600`。
   - `archive/known_prompts_l11_b5.rs`：独立扩展提示词表保持既有索引不漂移。
   - `resolution/frame_hooks.rs`：折叠 L11-5 清理键（`l11_loss_dedup:{fid}:{c}`、`l11_move_equip_evt:{card}`、`EventData::OrdinaryJudge` -> `l11_resolved_ordinary_judge:{invocation}`、`l11_carduse_lost:{popped_id}`、`sha_tianxian_done:{popped_id}`、`sha_longxiang_done:{popped_id}`、`sha_yueren_done:{popped_id}`），并移除冗余 `resolution/frame_retire.rs`。
   - `content/host/{activation,filtering,step}.rs`、`l27_batch_b_tw.rs`、`play_{card,equip}.rs`、`kernel.rs`、`judge_apply.rs`、`sha/{step,armor,respond,resume}.rs`、`events.rs`、`events/turn.rs`、`phase/judge.rs`：保留 `HEAD` 空牌堆/让步守卫、`txhj_mouduan` 装备栏门槛、`judge_apply` 先入弃牌堆再早退 `vm_card_history`、`sha` 响应/让步及 `L10`/`U01` 标志清理，并整合 L11-5 系统、武器与领主行为。
2. **合入后主线回归协调修复**：
   - `crates/content-native-loader/src/decision34_pack9.rs`：在未传扩展包回退链路中先行匹配 `"pyzhuren_diamond" -> HostBehavior::L25ZuanBatchE`，避免被 L11-5 `taixu_skills_data.rs` 新增的 `"pyzhuren_diamond" -> HostBehavior::L11Weapons` 遮蔽而破坏 `l25_zuan_fixes` 与 `l20_l22_fixes` `content_lock_id`。
   - `crates/engine-native/tests/l11_taixu_b5/round2_fixes.rs`（`3B-3` `txhj_zhuwei` × `zj_leiji`）：沿用主线 L11-4 已依上游 `mode/boss.js:7991-7992` 修复的 `zj_leiji` 判定归属（目标 `seat 1` 为判定者），将 `3B-3` 中 `txhj_zhuwei` 授权与手牌断言对齐至 `seat 1`。
   - `crates/engine-native-kernel/src/kernel/sha/step.rs`：在调用 `self.resolve_tieji(ctx)` 前保留显式 `if self.tieji_red.is_none()` 守卫，满足 `nc08_upstream_tieji::upstream_tieji_differences_are_fixed` 的源码固定差异断言。

---

## 2. 独立复核 48 项升级清单（`#41`）

- **升格为 `已复核等价` (48 项)**（`reviewer: "verifier-l11"`，`test_name: "l11_taixu_b5"`，`review_date: "2026-11-23"`）：
  - **系统/计数/装备栏/司道/破橹/神器同步 (`L11SystemHooks`, 15 项)**：`_txhj_skill_maxCards`、`_txhj_skill_Timelimit`、`_txhj_skill_addEffectStyle`、`_txhj_skill_gaincard`、`_txhj_skill_discard`、`_txhj_skill_usecard`、`_txhj_skill_damage`、`_txhj_skills_kill`、`_txhj_equip5`、`twsidao`、`twsidao_equip`、`polu`、`polu_damage`、`_txhj_shenqi`、`_txhj_XXXshenqi`。
  - **武器覆写技 (`L11Weapons`, 6 项)**：`hanbing_skill`、`cixiong_skill`、`guanshi_skill`、`sanjian_skill`、`yitianjian`、`pyzhuren_diamond`。
  - **L11/L28/L27 复用防具与三工 (`L28ThunderBatchA` 2 项 + `L27BeautifyBatchB` 3 项)**：`guofengyupao`、`wushuangfangtianji_skill`；`twzhouhu`、`twharvestinori`、`twzuhuo`（均沿 `l27_batch_b*` 共享 `buff_txhj_*_youhu`/`bihu` 别名复用）。
  - **将军与领主技能 (`L11GeneralBossC` 21 项 + `L11BossLingfeng` 1 项)**：`txhj_mouduan`、`olshanjia`、`boss_xuanlei`、`txhj_xuanlve`、`boss_baolian`、`boss_lingfeng`、`dz_gs_yinfu`、`boss_leili`、`boss_fengxing`、`boss_yushou`、`boss_xixing`、`boss_moyany`、`boss_danshu`、`boss_modao`、`boss_jueji`、`boss_mojian`、`boss_lianyu`、`boss_skonghun`、`txhj_zhuwei`、`boss_zhennu`、`txhj_chuhai`、`txhj_chuhai2`。
- **回归测试入口**：`crates/engine-native/tests/l11_taixu_b5_fixes.rs`（`system_weapons` 8 + `boss_generals` 5 + `round2_fixes` 6，经 `l_rounds` 运行，共 `19` 项真实事件链回归测试，`0` `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l11-5/`）

- `e1c518421212f11da9ad26b8528a5251fd2446dfa06869bd0cecb4d9a376c73f` — `docs/operations/evidence-raw/review-l11-5/probe-l11-5.mjs`
- `30b1f95559df5843d20c0ee85daf9cc648e334f7a23b45c3c14ce2f9e5de0094` — `docs/operations/evidence-raw/review-l11-5/probe-l11-5.json`

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`, `engine-native-wasm`, `content-native-loader`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`44` 组、**`2,875` passed / `0` failed / `6` ignored**（含 `l_rounds` **`270/0/0`**、`g1` **`395/0/1`**、`kernel` **`435/0/0`**、`engine-native-wasm` `29` 组 **`89/0/0`**）
   - `content-native-loader` offline：`12` 组、**`48` passed / `0` failed / `0` ignored**
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py`（`native_wasm_match: true`）& `cross-target-hash.sh`（`start_4_42=8fe8316a...` / `start_2_1=6fb9d95a...` / `manifest_4_42=11214341...`）：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,182` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client run test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`recount_g0.py` 与 `preflight.py` 全过（最后一步提交前复验见发布日志）。
