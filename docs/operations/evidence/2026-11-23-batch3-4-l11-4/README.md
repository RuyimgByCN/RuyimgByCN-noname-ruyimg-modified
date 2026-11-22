# Batch L11-4（L11 太虚幻境 33 `L11BuffsC` + 37 `L11BuffsD` + 10 `L11BuffHelpersB` 共 +80→360/936）独立复核与集成摘要

- **基线 HEAD**：`0f514af299519ba8325711136b49e447df2608d2`（Batch U01-3 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45048457551` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l11-b4` `swarm/conv-l11-b3..c3248c4565231770f8c5c574de098193ad96447a`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-l11`（`#22` **`APPROVE`**，含 Round 3 inline judge / yield 对齐校验与 Headless Chrome CDP）
- **清单变更**：全仓 `2,047 / 2,377 / 0` -> **`2,127 已复核等价 / 2,297 简化近似 / 0 缺失 / 0 上游未定义`**（`L11 太虚幻境` 由 `280 / 656 / 0` 升为 **`360 / 576 / 0`**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与架构行数拆分

1. **宿主标签、提示词与生命周期合并体协同**（与 `L01` / `U01-3` / `L11-3` / `L25` / `L27` 共存）：
   - `vocabulary/host/*` & `engine-native/src/lib.rs`：保留 `L01` 标签 `860..=864`，在其后追加 `L11BuffsC`（`736`）、`L11BuffsD`（`737`）与 `L11BuffHelpersB`（`738`），同步 `HostBehavior::ALL = 596`。
   - `archive/known_prompts_l11_b4.rs`：独立扩展提示词表保持既有索引不漂移。
   - `resolution.rs` & `resolution/frame_hooks.rs`：因 `main` 已将原 `resolution/frame_retire.rs` 并入 `frame_hooks.rs`，移除重复 `frame_retire.rs` 并把 L11-4 收尾清理键（`l11_jianiang_stacks`、`l11_recover_source:`、`sha_used_jiu:`、`qilin_fuchen_used:`、`txhj_taiji_rem:`、`vm_va_recover_bonus`）并入 `frame_hooks.rs`，保留 `notify_new_skills(..., out)` 与 `res.active_actor` 追踪。
   - `machine.rs` / `kernel/nullify.rs` / `kernel/tricks.rs` / `kernel/sha/{step,respond,armor,resume}.rs` / `kernel/judge_apply.rs`：合并 `u01_recover_preapplied` + `note_recover_source`（维持 `kernel.rs::commit_recover` 发 `LogEvent::HpRecovered`）、`clear_stateplayer` + `nullify_last`/`prev_wuxie`、`is_tongyu` 揭示日志 + `shierqicetui_excludes`、`try_handle_u01_respond_skill` + `sha/respond.rs`（接入 `resume::guanshi_need`）+ `sha_tianxian_done:{fid}` 让步清理；并将 `draw_judge_card_with_elo`（空牌堆早退保护）与 `Ctx::inline_judge_suit_rank` / `Ctx::settle_inline_judge_card`、`ganglie_post_judge:{fid}` 让步统一合入 `events.rs`、`events/turn.rs`、`phase/judge.rs`、`commit_up37.rs`、`gang_lie.rs`、`luo_shen.rs`、`zj_leiji.rs` 与全部分散 `HostStep` 模块。
   - `l27_batch_b*.rs` & `l27_fixes_b.rs`：规避 `342` 问询号碰撞，分配 `ASK_TWMAHOU_RENIGONG_CONFIRM = 343`（`CANCELLABLE_ASKS: [u32; 20]`），导出 `mahou_max_x` / `finish_mahou_cast` / `finish_mahou_cast_stage` / `finish_renigong_draw`，并将 L11 三工之护与 `twfengji`/`twzhouzu` 修复（`HostStage::SelectCard` 防舞台键碰撞）兼容整合。
2. **Rust 有效行数上限合规拆分**：
   - 将 `crates/engine-native-content/src/content/host/step/skills_damage.rs` 尾部的 `CaochuanCard` 与 `D3Moyan` 按职责拆入 `crates/engine-native-content/src/content/host/step/skills_damage_extra.rs`（`mod skills_damage_extra;`），保持 `skills_damage.rs` 原有前 `920` 行（`YxsFanji`、`MbcmQingzheng`、`U06`、`LinghunzhihuoCard`）及现有文档行号锚点完全不变且低于 `1,000` 有效行上限。

---

## 2. 独立复核 80 项升级清单（`#22`）

- **升格为 `已复核等价` (80 项)**（`reviewer: "verifier-l11"`，`test_name: "l11_taixu_b4"`，`review_date: "2026-11-23"`）：
  - **Group C (`L11BuffsC`, 33 项)**：`buff_txhj_*` 与 `_buff_txhj_*` 对应技能覆盖 `cebeibuyu`、`duocejiyong`、`fangzhibaowu`、`gongtaizhizhi`、`gongzhibaowu`、`haoshidebaochao`/`baochang`、`luanjifengshi`、`shenfuhuaxie`、`suikuiliejia`、`taoyuanyishi`、`xiuluodebeimin`/`lianmin`、`yiguzuoqi`、`_buff_txhj_jufuxionghao` 及 `bowenqiangzhi` / `fenchengluetu` / `fuchendangmo` / `majundegaizao` / `shangjiangdelvli` / `shenhanxueyin` / `wolongmice` / `xianhujiqu`。
  - **Group D (`L11BuffsD`, 37 项)**：`buff_txhj_*` 与 `_buff_txhj_*` 对应技能覆盖 `gusoejueyan`/`guosejueyan`、`hannu`、`hengxingwuji`、`huangtianyili`、`huangtianzhimin`、`huangtianzhinu`、`huangtianzhizu`、`jijianchangge`、`luoshenjuezhang`、`mingyundezhiyi`、`mowangdejianiang`、`pofuchenzhou`、`shenyizhidao`、`shierqicejin`、`shierqicetui`、`tianxiawushuang`、`ximengzhili` 与 `tiangongdeyouhu` / `digongdeyouhu` / `renigongdeyouhu`（其中 `huangtianyili`/`huangtianzhizu`/`huangtianzhimin` 含 `round3_fixes.rs` 判定重写与红牌弃置摸牌实测）。
  - **衍生辅助技 (`L11BuffHelpersB`, 10 项)**：`taoyuanyishiX`、`txhj_jianiang_clear`、`txhj_jufuxionghao`、`txhj_luoshen`、`txhj_luoshenjuezhang`、`txhj_mowangdejianiang`、`txhj_pofuchenzhou`、`txhj_shierqicejin`、`txhj_shierqicetui`、`txhj_taoyuanyishi`。
- **回归测试入口**：`crates/engine-native/tests/l11_taixu_b4_fixes.rs`（`buffs_c`、`buffs_d`、`common`、`round3_fixes`，经 `l_rounds` 运行，共 `16` 项真实事件链回归测试，`0` `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l11-4/`）

- `42195180514615fd24c4565c89eb9bfd349bf73e04140d36a20c280331410127` — `docs/operations/evidence-raw/review-l11-4/probe-l11-4.mjs`
- `c4d6d157fb14140aa6b560d1ab3f22beb9e375f42ae27ead0a9e59843bee407e` — `docs/operations/evidence-raw/review-l11-4/probe-l11-4.json`
- `a2188fd10030b1a4f4528e2987bf02431fb62273033e098801f844784b829065` — `docs/operations/evidence-raw/review-l11-4/probe-l11-4-v2.mjs`
- `84f312771c62df046777cf91d4b916b600378cd55acc092989dad86d678e4bb5` — `docs/operations/evidence-raw/review-l11-4/probe-l11-4-v2.json`

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`, `engine-native-wasm`, `content-native-loader`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`44` 组、**`2,856` passed / `0` failed / `6` ignored**（含 `l_rounds` **`251/0/0`**、`g1` **`395/0/1`**、`kernel` **`435/0/0`**、`engine-native-wasm` `26` 组 **`89/0/0`**）
   - `content-native-loader` offline：`12` 组、**`48` passed / `0` failed / `0` ignored**
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py`（`native_wasm_match: true`）& `cross-target-hash.sh`（`start_4_42=8fe8316a...` / `start_2_1=6fb9d95a...` / `manifest_4_42=11214341...`）：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,163` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client run test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`recount_g0.py` 与 `preflight.py` 全过（最后一步提交前复验见发布日志）。
