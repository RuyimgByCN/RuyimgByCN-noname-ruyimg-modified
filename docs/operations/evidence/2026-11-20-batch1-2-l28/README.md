# Batch 1.2（L28 雷霆万钧）规则修复与独立复核摘要

- **日期**：2026-11-20
- **基线提交**：`c119a64408e9556f58c415ddd0c5eceb1f11fdf0`（合并 `swarm/author-l28` `d50723fe23c05edfbe5fe43e6bd874c4b553a565`）
- **冻结版本门禁**：`RULES_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`（均零变更；14 项标准 golden 哈希全部保持不变）
- **独立复核 agent**：`571d828a-de8d-4c1a-b9cc-e06219f5006b`（未参与实现；三轮复审直至 **0 FAIL**，结合无头 Chrome 上游 `noname-liuli` 19 步真实事件探针与源码逐行对照）

## 1. 清单变化汇总

| 来源 | 修改前（等价／近似／缺失） | 修改后（等价／近似／缺失） | 净变化 | 说明 |
| --- | ---: | ---: | ---: | --- |
| L28 雷霆万钧 | 50 / 6 / 0 | 54 / 2 / 0 | +4 | `th_buxu`、`th_tianshu`、`th_yeyan`、`cards:th_taipingyaoshu` 升为 `已复核等价`；另 15 项重审通过；`th_zhengjing`、`thunder.css` 维持 `简化近似` |
| **全仓 4,424 项** | **632 / 3,792 / 0** | **636 / 3,788 / 0** | **+4** | 整包八项签署维持 **0/37** |

## 2. 本批修复的核心规则语义

1. **`th_buxu`（补叙）**：改为按玩家所选缺失六经类别依次搜索牌堆与弃牌堆（`step/l28_batch_a_ext.rs:130-296`），命中则置入 `th_liujing` 并累加 `th_buxu_used_turn`；两堆均无时不再占位、不添典并退还本次发动次数（已弃代价不退）。
2. **`th_chengye`（承业）**：在 `resolution.rs:831-833` 与 `l28_batch_a_ext.rs:58-128` 区分他人实体牌使用与延时/装备弃牌（排除真实转化牌，同时通过 `u.use_as == Some(catalog.card_kind(c))` 正确识别物理火/雷杀），接入死亡/换将/失技 `onremove` 退堆清理（`on_skill_removed_l28`）与 AI 选牌。
3. **`th_tianshu`（天书）与 `th_taipingyaoshu`（太平要术）**：`th_taipingyaoshu` 设为 `.with_off_deck()` 入 `detached_cards`；`th_tianshu` 改为 `phaseUseBegin` 钩子（`play_begin_tianshu`），先选目标再弃牌（皆可无损取消），从牌堆/弃牌堆/未出牌区取出实体交目标并于未废除栏位装备；`th_taipingyaoshu` 补齐 `unequip2` 手限与离场门控（`d34_flags.rs:136-163`、`equipment.rs`）、外层事件结束串行先摸 2 再判 `hp > 1` 失 1 血（`l28_batch_a_ext.rs:487-562`）及排除八卦判定（`is_bagua_armor`）。
4. **`th_yeyan`（业炎）与 `th_shen_zhouyu`（神周瑜 `pro_qinyin`）**：`th_yeyan`（`l28_batch_b_asks.rs:368-530`）按先弃 4 不同花色手牌、仅 `strengthen_skill == "forbid"` 时先失 3 体力、再按座次依次结算火伤的顺序对齐上游，并支持单目标 2/3 火与两目标 2+1 分配；`th_shen_zhouyu` 技能表改为 `[th_yeyan, pro_qinyin]`，通过 `Ctx::to_discard` / `monkey.rs` / `l28_batch_a_ext.rs` 统一记录同回合进弃牌堆红黑牌（含转化牌/响应牌/判定牌），支持全员回/失 1 血与减 1 上限重置业炎（禁用强化时退化经典 `qinyin`）。
5. **`th_bushi`、`th_caohua`、`th_tiansuan*`、`th_gushe`**：`th_bushi` 修复默认花色序 `(0, 3, 2, 1)`、全 24 排列、`s0` 无限酒杀、`effective_card_suit` 判断及成为多目标 `s2` 非首位目标时按目标触发 `s3` 摸牌；`th_caohua` 修复 `th_caiyi` `[1,1,1,2]` 选项池与强制弃牌及 `th_guili` 经 `turn_handoff`（`kernel/events/turn.rs`）插入完整额外回合；`th_tiansuan*` 修复加权签池、取牌区（`e` vs `he`）、`onremove` 退签与下签 `damageBegin3 +1` 优先于上签 `damageBegin4` 封顶；`th_gushe` 与 `rejici` 饶舌标记严格隔离。
6. **`th_jingxie`（精械）与 `th_shengong`（神工 18 件专属装备）**：`th_jingxie` 修复 6 牌白名单、`!is_jingxie_rewritten` 跨席禁再强化、`rewrite_lanyinjia` 手牌当闪使用/打出、青釭剑穿防具仅限杀类（`d34_l28_mods.rs:40-82`）及 `(target, "l28_vdmg_kind")` 使强化藤甲识别虚拟【杀】/【南蛮】/【万箭】；`th_shengong` 补齐助锻 1/2/3 备选、全部 18 件 `off_deck` 专属装备技能（`step/l28_shengong_equips.rs`、`d34_l28_mods.rs`、`extension_choices_l28.rs`）、八卦阵闪避补发 `EventData::ShaDodged`（`kernel/sha/armor.rs`）、黑光铠全伤害锦囊表、照骨镜 `CardKind::Jiedao`（`kernel/intrinsic.rs` + `kernel/tricks.rs` 按 `EquipSlot::Weapon` 判定）与虚拟 `"huogong"`、太公阴符重铸发 `CardsLost` 不发 `CardsDiscarded`、五行鹤翎扇保留物理火/雷杀判定。
7. **维持 `简化近似` 项（2 项）**：`th_zhengjing` 已修 `count=0` 不添经不摸牌与任意非空子集置经/剩余归己，仅实时 DOM 切水果小游戏得分与斗地主概率早停依裁决 40 维持 `0..=5` `ChoiceSpec` 计数抽象；`thunder.css` 为纯前端 CSS 渲染面，归属队列第 5 项界面阶段。

## 3. 独立复核 21 项明细表（`571d828a-de8d-4c1a-b9cc-e06219f5006b`）

| 清单项 ID | 结论 | 核对分支、测试与上游对照依据 | 清单新状态 |
| --- | --- | --- | --- |
| `L28:skills:th_buxu` | **PASS** | `l28_batch_a_ext.rs:130-296`、`l28_batch_a_asks.rs:185-197`；上游 `s1_buxu_hit_via_dom_button_3` / `s1_buxu_miss_via_dom_button_4_refunds_stat`；`l28_batch16_*` | `已复核等价` |
| `L28:skills:th_tianshu` | **PASS** | `l28_batch_a_ext.rs:399-485`（`phaseUseBegin`、先目标后弃牌可取消、`off_deck` 取牌与替装）；上游 `s2_tianshu_and_tpys`；`l28_reviewer_reg_tianshu_chengye_tpys` | `已复核等价` |
| `L28:skills:th_yeyan` | **PASS** | `l28_batch_b_asks.rs:333-335,368-530`（`strengthen_skill==forbid` 才失 3 血、座次结算）；上游 `s3_yeyan_great_and_pro_qinyin`；`l28_reviewer_reg_caohua_shen_zhouyu` | `已复核等价` |
| `L28:cards:th_taipingyaoshu` | **PASS** | `decision34_closure.rs:290-297`、`equipment.rs`、`d34_flags.rs:136-163`、`l28_batch_a_ext.rs:487-562`；上游 `s2_tpys_*`；`l28_chengye_*`、`l28_reviewer_reg_tianshu_chengye_tpys` | `已复核等价` |
| `L28:skills:th_taipingyaoshu` | **PASS** | `d34_l11_mods.rs:800-834`、`d34_flags.rs:136-163`、`l28_batch_a_ext.rs:487-562`；上游 `s2_tpys_fire_prevented` / `qinggang_bypass` / `onLose_*` | `已复核等价` |
| `L28:skills:th_zhengjing2` | **PASS** | `l28_batch_a_asks.rs:565-578`、`l28_batch_a_ext.rs:41-47`、`kernel/intrinsic.rs`（收“经”跳判定与摸牌、`onremove` 退弃牌堆） | `已复核等价` |
| `L28:generals:th_caohua` | **PASS** | `l28_batch_a_asks.rs:259-403`、`kernel/events/turn.rs`（`th_caiyi` 触发与选项池、`th_guili` 额外回合）；上游 `s6_caohua_caiyi`；`l28_reviewer_reg_caohua_shen_zhouyu` | `已复核等价` |
| `L28:generals:th_mamidi` | **PASS** | `decision34_closure.rs:218`、`resolution.rs:831-833`、`l28_batch_a_ext.rs:26-237`（`th_chengye` 含物理火/雷杀排除转化、`onremove`）；上游 `s1_chengye_*`；`l28_chengye_*` | `已复核等价` |
| `L28:generals:th_shen_zhouyu` | **PASS** | `decision34_closure.rs:224`、`machine.rs`/`monkey.rs`/`d34_l28_mods.rs:272-296`、`l28_batch_b_asks.rs:339-363`；上游 `s3_pro_qinyin_reset_yeyan`；`l28_reg_pro_qinyin_all_discard_paths` | `已复核等价` |
| `L28:skills:th_bushi` | **PASS** | `l28_batch_a.rs:364-385,597-612`、`l28_batch_a_shengong.rs:19-118`（24 排列、`effective_card_suit`、多目标 `s2`）；`l28_reg_th_bushi_multitarget_s2_and_effective_suit` | `已复核等价` |
| `L28:skills:th_shengong` | **PASS** | `l28_batch_a_shengong.rs:120-373`、`l28_shengong_equips.rs`、`armor.rs`、`tricks.rs`、`d34_l28_mods.rs`；`l28_reg_th_shengong_*`（6 武器/6 防具/6 宝物与 Round2/3 回归） | `已复核等价` |
| `L28:skills:th_jingxie` | **PASS** | `l28_batch_b.rs:91-127,450-487`、`d34_l28_mods.rs:40-260,297-301`、`sha.rs`/`aoe.rs`/`resolution.rs`；`l28_reg_th_jingxie_*` 与 `l28_reg_reviewer_round2_fixes` | `已复核等价` |
| `L28:skills:th_tiansuan` | **PASS** | `l28_batch_b.rs:579-691`、`l28_batch_b_asks.rs:148-188`（加权签池与 `e`/`he` 取牌区）；上游 `s4_gushe_tiansuan_jingxie` / `s5_*`；`l28_reviewer_reg_tiansuan` | `已复核等价` |
| `L28:skills:th_tiansuan2` | **PASS** | `l28_batch_a_ext.rs:48-51`、`l28_batch_b.rs:579-609`、`death.rs`/`machine.rs`（离场/下回合清除命运签）；`l28_reviewer_reg_tiansuan` | `已复核等价` |
| `L28:skills:th_tiansuan2_0` | **PASS** | `d34_flags.rs:823-825`（上上签 `damageBegin4` 防全伤）；`l28_reviewer_reg_tiansuan` | `已复核等价` |
| `L28:skills:th_tiansuan2_1` | **PASS** | `d34_flags.rs:826-833`、`l28_batch_b.rs:237-240,414-419`（上签封顶 1 并摸等量牌）；`l28_reviewer_reg_tiansuan` | `已复核等价` |
| `L28:skills:th_tiansuan2_3` | **PASS** | `d34_l28_mods.rs:83-107`、`d34_flags.rs:640-650`（下签 `damageBegin3 +1` 且禁桃）；`l28_reviewer_reg_tiansuan` | `已复核等价` |
| `L28:skills:th_gushe` | **PASS** | `l28_batch_b_gushe.rs:9-260`（饶舌与 `rejici` 隔离、3 目标上限与 7 标记阵亡）；上游 `s4_gushe_tiansuan_jingxie`；`l28_reviewer_reg_bushi_gushe` | `已复核等价` |
| `L28:configs:hidePack` | **PASS** | `thunder.rs:62,156,201-204`（默认 `true` 解析与 `hides_exclusive_pack`）；`l28_thunder_eight_roster_card_and_configs` | `已复核等价` |
| `L28:skills:th_zhengjing` | **KEEP_APPROX** | `l28_batch_a_ext.rs:298-397`、`l28_batch_a_asks.rs:177-184`（0 张与任意子集已修；DOM 小游戏与斗地主概率早停依裁决 40 抽象 `0..=5`）；`l28_batch16_*` | `简化近似` |
| `L28:surfaces:thunder.css` | **KEEP_APPROX** | 纯前端样式与动画表（1,522 行），无规则态副作用，归队列第 5 项界面阶段 | `简化近似` |

## 4. 独立复核记录的三项低风险内核边界

1. **`zhaogujing` 虚拟 `"huogong"`**：受冻结 `CardKind` / `engine-api::CardKindTag` 无 `Huogong` 变体约束，虚拟火攻于 `l28_shengong_equips.rs:160,242,286-299` 先经 `eval_card_target_canceled(..., CardKind::Linghunzhihuo, DamageNature::Fire)` 校验国风玉袍等免疫，再亮出目标 `hand.first()` 并询问弃同花色牌造成火伤（未开 `CardUsed`/无懈窗口或由目标自选亮牌；严于既有实体 `"huogong"` 在 `intrinsic_named_ext.rs:474-480` 的未核花色简化）。
2. **`play_card.rs:507` 手牌实体【借刀杀人】预检**：仍检查 `CardKind::Weapon`（未与 `kernel/tricks.rs:302-369` 同步为 `card_slot_of == EquipSlot::Weapon`）；对 `th_shengong` 6 件专属武器无影响（其自身 `CardKind` 即为 `Weapon`）。
3. **`(target, "l28_vdmg_kind")` 临时伤害来源键**：按受伤害目标键存并在 `Damage` 弹出删除，可隔离并行 AOE 压栈与铁索传导；仅同一目标在未弹出的虚拟杀/AOE `Damage` 帧内再嵌套触发另一个 `card: None` 伤害时才可能覆写。

## 5. 原始复核证据路径与 SHA-256（存于 `.gitignore` 目录 `docs/operations/evidence-raw/`）

- `docs/operations/evidence-raw/review-batch1-2/probe_l28_upstream.mjs`：`sha256: c89b814b27a74b7a59acf392fedeb54a62711da4ef25e284a89fba557bf06aca`
- `docs/operations/evidence-raw/review-batch1-2/l28_upstream_live.json`：`sha256: 92c2eb8f1c92c55878ab14ab04ada1e94636b8ae04d0fbcacc4685d0c5a3083a`（19 步上游真实事件探针，`staticCheck` ＋ `s1`–`s6`）

## 6. `noname-next` 本地第五节未过滤门禁实测记录

1. **脚本与静态检查**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：2 tests OK（exit 0）
   - `python3 -B scripts/check-source-lines.py`：9,379 文件 <= 2,000 行、Rust 非测试有效行 <= 1,000（exit 0）
   - `python3 -B scripts/check-native-layering.py`：0 upward references <= 0（exit 0）
   - `python3 -B scripts/check_doc_line_refs.py --strict`：10,788 处检查、0 越界（exit 0；同步修正 `2026-11-12-l28-thunder-review/README.md` 与 `2026-11-13-queue1-review/independent/other-sources/samples.md` 共 6 处因 `l28_batch_a*` 拆分移位行号）
   - `python3 -B scripts/recount_g0.py`：来源数 37、缺失 0（exit 0）
2. **Rust 编译与单元测试**：
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native`：**2477 passed; 0 failed; 6 ignored**（18 个测试集，含 14 标准 golden 全通过；其中 `tests/g1` `l28_` 20 passed）
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-batch1-2-lint RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：零警告通过（exit 0，临时目录已删除）
   - `cargo check --target wasm32-unknown-unknown -p engine-native-wasm`：exit 0
   - `cargo test --offline --workspace --no-fail-fast`：**7873 passed; 0 failed; 8 ignored**（178 个测试集，exit 0）
   - `cargo run -p conformance`：**19/19 ALL PASS**（exit 0）
   - `cargo test -p desktop-client`：**20 passed; 0 failed; 1 ignored**（4 个测试集，exit 0）
3. **前端与跨端 WASM**：
   - `pnpm --dir apps/web-client test`：**489 passed; 0 failed**；`pnpm --dir apps/web-client run typecheck`：exit 0
   - `pnpm --dir apps/web-client run build:native`：exit 0；`pnpm --dir apps/web-client run smoke:native`：**83 total / 0 failed**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：标准 7 模式与扩展跨端轨迹均匹配（`native_wasm_match: true`, `final_revision: 20`）
   - `bash scripts/cross-target-hash.sh`：`start_4_42` / `start_2_1` Native 与 WASM 哈希一致，`ALL CROSS-TARGET CHECKS PASSED`
   - `pnpm --dir apps/web-client run build`：exit 0（随后 `git checkout -- apps/web-client/dist/` 还原）
4. **最终提交前检查**：`python3 -B scripts/preflight.py` 全量通过（exit 0）。
