# Batch L11-2（L11 太虚幻境 4 侍从钩子＋26 侍从技 B＋53 武将技 D/E/F 共 83 项收口）独立复核与集成摘要

- **基线 HEAD**：`5bcd6b619ebfbe9bd5b414eeb5bbd7d2d1341443`（Batch Q2-L25 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45001901211` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l11-b2` `445ed1bd2..ea623f25507556ed642b504293865719c6fb4065`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-rules-2`（未参与 Batch L11-2 实现；经两轮源码核验、Round 1 `#1-#15` 复测与 16 场景上游 Chrome CDP 探针比对，在 `#174` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,759 / 2,665 / 0` -> **`1,842 已复核等价 / 2,582 简化近似 / 0 缺失 / 0 上游未定义`**（`L11 太虚幻境` 由 `109 / 827 / 0` 升为 **`192 / 744 / 0`**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与并行/后置兼容性修复

1. **词汇表、钩子目录与内核事件总线协同**（与 L25 / U01 / L29 / L27 / L10 / U08 共存）：
   - `vocabulary/host/{behavior,behavior_list,keys,metadata,tag_decode,tags}.rs` 与 `crates/engine-native/src/lib.rs`：保留既有标签及 Q2-L25 合入接口，合入 `L11` 标签 `728..=732`（`L11ServantHooks`、`L11ServantSkillsB`、`L11GeneralSkillsD`、`L11GeneralSkillsE`、`L11GeneralSkillsF`；`HostBehavior::ALL: 585`）。
   - `catalog/hooks.rs`、`content/hooks.rs`、`content/host.rs`、`activation.rs`、`filtering.rs`、`u08_batch_d_hooks.rs`、`machine.rs`、`resolution.rs`、`snap/validation.rs`、`aoe.rs`、`duel.rs`、`sha/step.rs`、`tricks.rs`、`ai/extension_choices.rs` 与 `engine/play_{card,sha}.rs`：保留 Q2-L25 `ContentHooks::on_host_event`、`needed_shan_count`、`is_ssr_luoyi_trick_to_duel`、`d34_flags.rs`（含已拆出的 `d34_unrespondable.rs` 保 `<= 1,000` 有效行）与 U01/L19/L27 共存路由，合入 `l11_common_b2.rs`、`l11_servants_b{,_asks}.rs`、`l11_skills_{d,e,f}{,_asks}.rs`、`known_prompts_l11_b2.rs`、`taixu_skills_data.rs` 与 `extension_choices_l11_b2.rs`。
2. **`DismantleMachine` 装备弃置事件与手牌失去类技能隔离修复**：
   - 合并后全仓回归暴露 `u01_fix2_01b_dddzhengjun_branch3_judge_and_source` 与 `u01_fix2_19b_dddchashi_expiry_and_non_use_equip` 失败：因 `L11-2` 为使 `txyinli`/`txhj_suzhi` 观测过河拆桥装备而于 `kernel/tricks.rs`（`from_equip && vm_card_history`）追加 `EventData::CardsDiscarded`，原先仅设在 `txhj_xuanwuSkill2` 条件下的 `l11_discard_non_hand:{c}` 未覆盖手牌敏感消费者 `dddzhengjun`（`branch 2`）与标准技能 `lian_ying`。
   - 现改为在 `kernel/{tricks,damage,native_invocation}.rs`、`step/l11_common.rs` 及 `resolution/frame_hooks.rs` 中凡 `vm_card_history` 或 `txhj_xuanwuSkill2` 生效时统一标记并在出栈后清理 `l11_discard_non_hand:{c}`（零改动无 VM 包 14 golden 路径），同时于 `u01_fixes_2/routing.rs`（`dddzhengjun` `CardsDiscarded` 臂）与 `standard/skills/lian_ying.rs` 过滤全为非手牌标记的 `CardsDiscarded`（保留 `txyinli`/`txhj_suzhi` 装备弃置触发与 `dddzhengjun` `CardsLost(Equip)` 分支 3）。

---

## 2. 独立复核 83 项升级实体清单（`#174` `APPROVE`）

- **侍从全局钩子 (4)**（`extension_servant.js`）：`_slEnterGame` (`:11`)、`_slDamage` (`:205`)、`_slUseCard` (`:244`)、`_slHurt` (`:261`)
- **侍从技能 Group B (26)**（`extension_buff.js`）：`txhj_dundunSkill1` (`:893`)、`txhj_datongSkill1` (`:1135`)、`txhj_luluSkill1..2` (`:1146..1157`)、`txhj_aleSkill1..2` (`:1168..1179`)、`txhj_ruiSkill2..3` (`:1201..1212`)、`txhj_yueerSkill1..2` (`:1223..1234`)、`txhj_yanSkill1..3` (`:1245..1267`)、`txhj_liuliSkill1..2` (`:1278..1289`)、`txhj_manmanSkill1..2` (`:1300..1311`)、`txhj_xiaoxiaoSkill1..2` (`:1322..1333`)、`txhj_xuerenSkill1` (`:1344`)、`txhj_xuanwuSkill1..3` (`:1355..1377`)、`txhj_aHaoSkill1..3` (`:1432..1454`)
- **武将技能 Group D (18)**（`extension.js`）：`oldfuman` (`:4380`)、`oldfuman2` (`:4413`)、`txhuao` (`:4449`)、`hgkurou` (`:4461`)、`hgkurou_effect` (`:4486`)、`gdsanchen` (`:4493`)、`txlveming` (`:4554`)、`txdaoxi` (`:4613`)、`txlangxi` (`:4631`)、`txkuangyi` (`:4664`)、`txtanbei` (`:4684`)、`txtanbei_effect3` (`:4729`)、`txtanbei_effect1` (`:4745`)、`txtanbei_effect2` (`:4748`)、`ljxuechi` (`:4751`)、`txzhuxin` (`:4765`)、`txyicheng` (`:4777`)、`txnianrui` (`:4791`)
- **武将技能 Group E (17)**（`extension.js`）：`txmengtai` (`:4801`)、`txshouyi` (`:4854`)、`txrenxing` (`:4861`)、`txyinli` (`:4871`)、`txzhulian` (`:4900`)、`txkeji` (`:4968`)、`txkeji_add` (`:4988`)、`txnangce` (`:4995`)、`txduanzui` (`:5015`)、`txyingyang` (`:5044`)、`mdqianxi7` (`:5087`)、`mdqianxi72` (`:5131`)、`mdqianxi` (`:5151`)、`zylongdan` (`:5173`)、`zglkongcheng` (`:5303`)、`txjuxiang` (`:5388`)、`txdaoji` (`:5458`)
- **武将技能 Group F (18)**（`extension.js`）：`txhj_suoming` (`:5502`)、`txhj_huangmen` (`:5537`)、`txhj_mouqiang` (`:5563`)、`txhj_suzhi` (`:5588`)、`txmanjia` (`:5657`)、`txmanjia1` (`:5660`)、`txmanjia2` (`:5681`)、`txmanzhen` (`:5704`)、`txfusha` (`:5809`)、`txleizhou` (`:5820`)、`txzhennu` (`:5834`)、`txyongguan` (`:5851`)、`txyongguan_fate` (`:5866`)、`txchengyuan` (`:5896`)、`txbenji` (`:6008`)、`txyonglve` (`:6032`)、`txjiang` (`:6049`)、`txhj_oldhuxiao` (`:6078`)
- **回归测试入口**：`crates/engine-native/tests/l11_taixu_b2_fixes.rs`（`hooks_servants`、`skills_d`、`skills_e`、`skills_f`、`round2_fixes` 共 `17` 项真实命令流测试，全部使用 `GameCommand`/`GameAction`，`0` `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l11-2/`）

- `7b5fd4389d1db8d5738c59f8848299b9da0083b7d10e58a4078236299f96f10f` — `docs/operations/evidence-raw/review-l11-2/probe-l11-2.mjs`（16 场景上游实跑探针）
- `1da6f28ff0b0c2859943fc08c849c68807f9a246454c4400a5fac45e46165f7e` — `docs/operations/evidence-raw/review-l11-2/upstream-l11-2-cdp-results.json`（`pageExceptionCount: 0`）

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-integrator-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`30` 组、**`2,760` passed / `0` failed / `6` ignored**（含 `l_rounds` **`217/217`**、`g1` **`374/0/1`** 与 `u01_dddanzhi_regression` **`20/20`**）
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py`（`native_wasm_match: true`）& `cross-target-hash.sh`：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,108` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`（`2 OK`）、`check-source-lines.py`（`9,538` 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_ledger_consistency.py --strict`、`check_doc_line_refs.py --strict`（`10,925` 引用、越界 `0`）、`recount_g0.py` 与 `preflight.py` 全过。
