# Batch L11-3（L11 太虚幻境 40 祝福＋38 `_buff_txhj_*` 触发＋10 衍生辅助技共 88 项收口）独立复核与集成摘要

- **基线 HEAD**：`9c29b98996558302cadebec39c59ba8787cab86b`（Batch L01-Rules 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45017015012` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l11-b3` `ea623f255..91c68342c73984d0e7052a290f65bd701dbe0b37`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-rules-2` 首轮审查（`#188` 提出 7 项规则差异＋真实上游 Chrome CDP 探针）与 `verifier-ui` 次轮独立复核（`#201` 确认全部 7 项修复达成 **`APPROVE`**）
- **清单变更**：全仓 `1,939 / 2,485 / 0` -> **`2,027 已复核等价 / 2,397 简化近似 / 0 缺失 / 0 上游未定义`**（`L11 太虚幻境` 由 `192 / 744 / 0` 升为 **`280 / 656 / 0`**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与回归修复

1. **词汇表、钩子拆分与视图权限协同**（与 `L01` / `U01` / `L11-2` / `L25` 共存）：
   - `vocabulary/host/{behavior,behavior_list,keys,metadata,tag_decode,tags}.rs` 与 `crates/engine-native/src/lib.rs`：保留 `L01` 标签 `860..=864`，并入 `L11` 标签 `733..=735`（`L11BuffsA`、`L11BuffsB`、`L11BuffHelpersA`），`HostBehavior::ALL` 增至 **`593`**。
   - `catalog/hooks.rs`、`content/hooks.rs`、`content/host.rs`、`activation.rs`、`filtering.rs`、`step.rs`、`u08_batch_d_hooks.rs`：沿用既有拆分结构，将 `ContentHooks::on_judge_settled` 接入 `catalog/hooks.rs` 并在 `host.rs` / `u08_batch_d_hooks.rs` 保留 `host_judge_settled` 与 `on_card_use_after_l11_b3`。
   - `rvm_bridge/modifiers/d34_unrespondable.rs` 与 `d34_flags.rs`：保留既有的独立拆分模块 `d34_unrespondable.rs` 以满足 Rust 非测试有效行 `<= 1,000` 上限，将 `apply_card_unrespondable_with_nature` 落入该模块并经 `d34_flags.rs` 导出。
   - `kernel/judge.rs` 与 `kernel/judge_apply.rs`：保留 `judge_rewrite.rs` 既有导出并裁去 `judge_apply.rs` 中重复的三处改判辅助函数。
   - `engine/views.rs`：合并 `has_kapaikeshi` 观手视口与 `U01` `xingchong_card` 公开标记判断。
2. **帧出栈清理、装备/响应/拆顺兼容修复**：
   - `resolution/frame_hooks.rs`：合入 `l11_hand_dismantle_card:{c}`、`l11_hand_snatch_card:{c}`、`l11_equip_from_hand:{card}` 与 `l11_sha_dodge_ok:{popped_id}:` 清理（移除重复拆出的 `frame_retire.rs`），保留 `L11-2` 在 `vm_card_history || txhj_xuanwuSkill2` 下的 `l11_discard_non_hand:{c}` 清理与 `U01`/`L29` 清理。
   - `kernel/equipment.rs`：保留 `u01_equip_from_table` 与 `is_equip_slot_enabled` 门控，对有效手牌入装写入 `l11_equip_from_hand:{card}`。
   - `kernel/sha/{respond,resume}.rs`：在 `resolve_shan_card` 与 `asking_sadeng_shan` 中保留 `emit_card_responded_as` 并补充 `l11_sha_dodge_ok:{fid}:{shan}` 记录。
   - `kernel/tricks.rs`（回归修复 **`u01_ddd_fixes_1::part5_recheck::recheck_12_dddshilie`**）：在 `apply_dismantle_card` 与 `apply_snatch_card` 中对从手牌取走的卡牌保持使用 `ctx.take_hand_card(target, c)`（触发 `on_hand_card_removed` 清除 `dddshilie_shown_card` / `xingchong_card`），同时保留 `txhj_kapaikeshi:choose_card` 选牌分支及 `L11-2` `from_equip && vm_card_history` 下无条件写入 `l11_discard_non_hand:{c}`。

---

## 2. 独立复核 88 项升级实体清单（`#188` + `#201` `APPROVE`）

- **祝福定义 Group A/B (40)**（`extension_buff.js`）：`buff_txhj_aozhan`、`buff_txhj_yanhuo`、`buff_txhj_jinglei`、`buff_txhj_xihuo`、`buff_txhj_dinglei`、`buff_txhj_wuzhongshengsha`、`buff_txhj_wuzhongshengshan`、`buff_txhj_fengyushengcai`、`buff_txhj_jipodizhen`、`buff_txhj_wenhezhenlue`、`buff_txhj_wenhejibei`、`buff_txhj_yueyingdezhihui`、`buff_txhj_changbingguangren`、`buff_txhj_fuzu`、`buff_txhj_xiaojideyongwu`、`buff_txhj_xiongziyingfa`、`buff_txhj_wuxiangdexinxue`、`buff_txhj_juguzhidao`、`buff_txhj_pozhenzhifeng`、`buff_txhj_yanrenzhinu`、`buff_txhj_dunjiazhoufa`、`buff_txhj_ziyideyongzhan`、`buff_txhj_yanhuozhiren`、`buff_txhj_jingleizhiren`、`buff_txhj_yingjicuoshijia`、`buff_txhj_yingjicuoshiyi`、`buff_txhj_xihuozhikai`、`buff_txhj_dingleizhikai`、`buff_txhj_yueyingdejiqiao`、`buff_txhj_taipingzhoufa`、`buff_txhj_baijujianchu`、`buff_txhj_juejingzhice`、`buff_txhj_xumingshenyao`、`buff_txhj_wuweizhile`、`buff_txhj_yuanduandilue`、`buff_txhj_jiezizhidao`、`buff_txhj_huangtiannizhuan`、`buff_txhj_dongchazhiyan`、`buff_txhj_mingjingzhiyan`、`buff_txhj_juxiangzhifeng`
- **祝福触发 (38)**（`extension_skill.js`）：`_buff_txhj_yanhuo`、`_buff_txhj_jinglei`、`_buff_txhj_xihuo`、`_buff_txhj_dinglei`、`_buff_txhj_wuzhongshengsha`、`_buff_txhj_wuzhongshengshan`、`_buff_txhj_fengyushengcai`、`_buff_txhj_jipodizhen`、`_buff_txhj_wenhezhenlue`、`_buff_txhj_wenhejibei`、`_buff_txhj_yueyingdezhihui`、`_buff_txhj_changbingguangren`、`_buff_txhj_fuzu`、`_buff_txhj_xiaojideyongwu`、`_buff_txhj_xiongziyingfa`、`_buff_txhj_wuxiangdexinxue`、`_buff_txhj_pozhenzhifeng`、`_buff_txhj_yanrenzhinu`、`_buff_txhj_dunjiazhoufa`、`_buff_txhj_ziyideyongzhan`、`_buff_txhj_yanhuozhiren`、`_buff_txhj_jingleizhiren`、`_buff_txhj_yingjichuoshijia`、`_buff_txhj_yingjichuoshiyi`、`_buff_txhj_xihuozhikai`、`_buff_txhj_dingleizhikai`、`_buff_txhj_yueyingdejiqiao`、`_buff_txhj_taipingzhoufa`、`_buff_txhj_baijujianchu`、`_buff_txhj_juejingzhice`、`_buff_txhj_xumingshenyao`、`_buff_txhj_wuweizhile`、`_buff_txhj_yuanduandilue`、`_buff_txhj_jiezizhidao`、`_buff_txhj_huangtiannizhuan`、`_buff_txhj_dongchazhiyan`、`_buff_txhj_mingjingzhiyan`、`_buff_txhj_juxiangzhifeng`
- **祝福衍生辅助技 (10)**（`extension_skill.js`）：`txhj_kapaikeshi`、`txhj_wenhejibei`、`txhj_yueyingdezhihui`、`txhj_changbingguangren`、`txhj_xiongziyingfa`、`txhj_wuxiangdexinxue`、`txhj_pozhenzhifeng`、`txhj_wuweizhile`、`txhj_yuanduandilue`、`txhj_huangtiannizhuan`
- **回归测试入口**：`crates/engine-native/tests/l11_taixu_b3_fixes.rs`（`buffs_a`、`buffs_b`、`common`，通过 `l_rounds` 分组运行，共 12 项真实命令流回归测试，`0` `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l11-3/`）

- `708feb53f725b01cd7ca8e71e5a0c19106b906c094dc39fd2a39bfbd318e6e89` — `docs/operations/evidence-raw/review-l11-3/probe-l11-3.mjs`
- `779e7d7e8f347d3b2eeec24bb665990ac4ba7a1779c42642b0242e4b9f07e7c3` — `docs/operations/evidence-raw/review-l11-3/upstream-l11-3-cdp-results.json`（`pageExceptionCount: 0`）

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `4-crate` offline（`engine-native-kernel`, `engine-native-content`, `engine-native`, `engine-native-wasm`）：`44` 组、**`2,819` passed / `0` failed / `6` ignored**（含 `l_rounds` **`235/235`**、`g1` **`374/0/1`**）
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py` & `cross-target-hash.sh`：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,126` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-integrator-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client run test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`check_ledger_consistency.py --strict`、`check_doc_line_refs.py --strict`、`recount_g0.py` 与 `preflight.py` 全过。
