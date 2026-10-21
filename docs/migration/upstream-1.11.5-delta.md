# 上游内容迁移推进

> 历史参考（2026-09-16 标注）：本文描述旧线及所注明日期的状态，文中的“当前”、阶段验收和统计不代表新原生线完成度。最新入口见 [旧实现参考](../architecture/legacy-reference.md)。

> 更新：2026-08-15  
> **catalog `ported` 武将 2524 / 2524、技能 6064 / 6064**（上游 **1.11.5**，head `2eb3513b`）

> **2026-08-16 复核**：下表为 08-15 快照、按约定不再回写；当前活数字以
> `content/original/catalog.json`（2526 武将 / 6068 技能 / 5982 能力画像 / 2623 立绘 /
> 45 名无同名立绘）与 `docs/content-fidelity-ledger.md`（5533 `.skill`：完整 5086 /
> 近似 447 / TODO 0 / noop 2094）为准，host `RULE_VM_ABI_VERSION` = 651（ABI-MAX 651，
> npack pin 不变：clan 649 / SP 647 / shahai 645 / TW 596 / offline 498）。
> **2026-10-10 注**：上面括号内是 08-16 当日数字。10-08 已同步到上游 v1.11.7，当前值见 `catalog.json`、保真度清单、`crates/engine-api/src/version.rs` 与各 `*.npack.json`；旧线 ABI 账面未闭合项见[进度主稿](../architecture/native-progress.md#后续待做按阻塞关系) T15。

## 2026-08-15 复核快照

对照 `content/original/catalog.json` 与 `content/original/ported/**/*.skill` 实测。
下表是**08-15 当日数字**（同日后续增量日记保留在后文，不再回写本表）：

| 层 | 现状 |
|---|---|
| 上游 | `archive/upstream/noname` 1.11.5 / `2eb3513b` |
| 目录 | 26 武将包；2524 / 2524 武将、6064 / 6064 技能 `ported`；5978 能力画像；2622 立绘；未解析引用 0；44 名武将上游亦无立绘 |
| 投放 | `extensions/` 32 包（26 + ddd/huanle/jiange/shahai/wandianlunsha/yxs）；2950 角色索引 / 7546 技能索引 / 56 卡；引擎 ABI 649（clan pin 649 / SP 647 / shahai 645 / TW 596 / offline 498） |
| 语义 | 5533 个 `.skill`：完整 5124、近似 409、TODO **0**；`INTENTIONAL_EMPTY_NOOP` 2132（约 1497 上游空定义，其余约 635 为原语不足的诚实空实现） |
| 里程碑 | M0 过；M1 语义冻结进行中；M2 标准切片过；M3 目录闭环、TODO 骨架已清零，近似与诚实 noop 仍待补；M4 身份/国战/单挑已实现；M5/M6/M8 门禁勾选；M7 部分覆盖 |

`ported` 只表示清单闭包与 npack 可装入。保真度明细见
[`../content-fidelity-ledger.md`](../content-fidelity-ledger.md)；重跑
`python3 scripts/gen-fidelity-ledger.py`。

## 2026-08-15 续：文心雕龙曹丕 放逐 / 慊慊

从保真度清单取出 `wxdl_caopi` 引用链上的两个 TODO（`wxdlfangzhu` 为「遗世」衍生，
`wxdlqianqian` 为角色技能），按旧 JS 合同改写，**无新 ABI**：

| 技能 | 合同 | 实现 |
|---|---|---|
| wxdlfangzhu 放逐 | 受伤后可选其他角色：其摸拥有者已损失体力张，再翻面 | 复用 ABI 85 `choose_other_draw_owner_lost_hp_turn_over`；遗世授予后的目标白名单未建模 |
| wxdlqianqian 慊慊 | 锁定技，每回合首张使用牌不能为♥️ | `card_use_cancel(heart)` + 用牌后 `disable_skill_by_def_until_turn_end`；cardSavable 未单独建模 |

offline npack 保持 `rule_vm_abi_version=498`（手术替换模块后 `rehash_npack`），
`extensions/offline` 双副本与源码 npack 同哈希。清单 TODO 155→152、完整 5040→5042。

## 2026-08-15 续：遗世 + 竭缘

| 技能 | 合同 | 实现 |
|---|---|---|
| wxdlyishi 遗世 | 限定技，对任意名其他角色摸已损失体力张，再授予放逐 | `once_per_game targets 1 255 other` + `draw_selected_targets(max_hp-hp)` + `choose_other_add_skill(wxdlfangzhu)`；翻面与全体自动授予未建模 |
| twjieyuan 竭缘 | 对体力≥你者伤害+1（弃黑）/ 受伤-1（弃红） | 双 `damage_before` 窗口先 `modify_current_damage` 再弃一张手牌；体力比较与花色未建模。此前误标 audio-only noop 已纠正 |

offline npack 仍 ABI 498，tw npack 仍 ABI 562。

## 2026-08-15 续：SP 返乡 / 远筹 / 决生 / 隐士

从保真度清单取出角色索引上的四条 SP TODO，按旧 JS 合同改写，**无新 ABI**：

| 技能 | 角色 | 合同 | 实现 |
|---|---|---|---|
| fanxiang 返乡 | sp_sunshangxiang | 准备阶段觉醒：+1 上限、回 1、卸良助、授枭姬 | `phase_prepare` + `mark_count` 一次门控；`change_max_hp`/`recover_hp`/`remove_skill_by_def(liangzhu)`/`add_skill(xiaoji=22)`。良助 storage / 已受伤条件未建模 |
| yuanchou 远筹 | fanjiangzhangda | 黑色杀无视防具 | `ignore_armor`；不限黑色杀、也不分「使用/被指定」两向 |
| juesheng 决生 | fanjiangzhangda | 限定技当决斗，伤害按目标出杀史 | `view_as juedou play_phase any from hand uses_per_turn 1`；耗手牌、每回合一、伤害保持 1、不回授 |
| xinfu_yinshi 隐士 | simahui | 空防具且无荐杰标记时防止属性/锦囊伤害 | `damage_before` + `prevent_current_damage`；不检验空栏/标记/伤害类型 |

sp npack 保持 `rule_vm_abi_version=647`（手术替换模块后 `rehash_npack`），`extensions/sp` 双副本与源码 npack 同哈希。清单 TODO 150→146、近似 322→326、noop 2194→2190。

## 2026-08-15 续：SP 抚悼 / 评忠 / 颂词 / 贤膺

从保真度清单取出角色索引上的四条 SP TODO，按旧 JS 合同改写，**无新 ABI**：

| 技能 | 角色 | 合同 | 实现 |
|---|---|---|---|
| olfudao 抚悼 | ol_dingshangwan | 开局摸 3，再交/弃并记账、回合末对拍摸牌 | `game_started` + `draw_cards(3)`；交弃记账与对拍未建模 |
| olpingzhong 评忠 | ol_yangfu | 受伤摸空手数+1，用牌可加目标 | 受伤摸 1；空手数与加目标未建模 |
| songci 颂词 | chenlin | 对未指定过的角色：手牌>体力弃 2 否则摸 2 | `active play_phase targets 1 1` + `draw_selected_targets(2)`；弃牌分支与每人一次未建模 |
| olxianying 贤膺 | ol_qinlang | 准备/受伤后可摸 2 再弃牌，同名则回合末视为使用 | 双可选窗口各摸 2；弃牌去重与视为使用未建模 |

sp npack 仍 ABI 647。清单 TODO 146→142、近似 326→330、完整 5053→5057。

## 2026-08-15 续：SP 集运 / 奇思 / 行贾 / 平远

从保真度清单取出角色索引上的四条 SP TODO，按旧 JS 合同改写，**无新 ABI**：

| 技能 | 角色 | 合同 | 实现 |
|---|---|---|---|
| oljiyun 集运 | ol_lifeng | 开局置三类型「粮」；准备阶段令至多粮数名角色各摸 1 | 开局摸 3；准备阶段 `choose_optional_targets_draw` 至多 3 人各摸 1。粮区收回未建模 |
| olqisi 奇思 | ol_puyuan | 开局获两张不同副类别装备；摸牌阶段可少摸换装备 | 开局摸 2；少摸换装备未建模 |
| olxinggu 行贾 | zhangshiping | 开局至多三张坐骑入「贾」；结束阶段可交易摸 1 | 开局摸 3；坐骑区与交易未建模 |
| olpingyuan 平远 | ol_liuye | 开局可装备霹雳车并摸 1；回合开始可移动或视为杀 | 开局可选摸 1；回合开始可选视为无距离杀。霹雳车未建模 |

sp npack 仍 ABI 647。清单 TODO 142→138、近似 330→334、noop 2190→2186。

## 2026-08-15 续：SP 晖云 / 纵掠 / 思泣 / 累卵

从保真度清单取出角色索引上的四条 SP TODO，按旧 JS 合同改写，**无新 ABI**：

| 技能 | 角色 | 合同 | 实现 |
|---|---|---|---|
| olhuiyun 晖云 | ol_huban | 牌当火攻，结算后三项选择 | `view_as huogong`；结算后选项未建模 |
| olzonglve 纵掠 | ol_niufu | 牌当杀（限一次）；此杀伤害后每区获一张 | `view_as sha uses_per_turn 1` + 造成伤害后 `take_one_card` 一张 he |
| olsiqi 思泣 | ol_xuelingyun | 受伤后亮出连续红牌并使用 | 受伤可选摸 1 |
| leiluan 累卵 | kongshu | X 张牌当本轮未用基本牌 | 同时提供转杀/闪/桃；张数与轮次门控未建模 |

sp npack 仍 ABI 647。本批 4 条离开 TODO 进入近似；清单总览 TODO 138→126、近似 334→338（offline 若干条同期改为诚实 noop，不在本批）。

## 2026-08-15 续：SP 垦伤 / 西向 / 飞辩 + TW 荡疆

从保真度清单取出三条角色引用的 SP TODO，以及一条误标 audio-only、上游有完整转换体的 TW TODO，**无新 ABI**：

| 技能 | 角色/来源 | 合同 | 实现 |
|---|---|---|---|
| rekenshang 垦伤 | maxiumatie | 至少两张牌当杀，可改目标为范围外全体 | `view_as sha ... up_to 2`；改目标未建模 |
| spolxixiang 西向 | sp_ol_caocao | 若干张当杀或决斗，每种每回合一次 | 同时转杀与决斗；张数与每回合门控未建模 |
| olfeibian 飞辩 | ol_miheng | 用牌后或成为目标后该角色随机弃 1 | 自己用牌后弃 1 张手牌；成为目标与时限未建模 |
| twdangjiang 荡疆 | 幻孙策派生 | 非伤害牌转无中/无距离杀 | 同时转无中与无视距离杀；阴阳切换与摸牌未建模 |

sp npack 仍 ABI 647，tw npack 仍 ABI 596。清单 TODO 126→122、近似 338→342、noop 2190→2186。

## 2026-08-15 续：清空全部 122 条 TODO

一次写完全部剩余 `// TODO`（SP 106 + TW 12 + offline 4），然后才 pack/merge。无新 ABI。

| 类 | 数量 | 处理 |
|---|---|---|
| 可运行近似 | 5 | `olkenshang` 转杀；`olfeibai`/`fengpo2` 伤害+1；`olfengyan` 受伤摸 1；`olzhubi` 令目标摸 1 |
| 诚实空实现 | 117 | 现有 filter/effect 不足以表达旧 JS（chooseToUse / gaintag / expansion / 额外回合等） |

sp/tw/offline npack 仍为 ABI 647 / 596 / 498。清单 TODO **122→0**。

---

## 本轮：codegen 联合修复 + clanbaichu/clanjianbai 忠实条件语义 + noop 全量扫描

### codegen 修复（if+HOST_EFFECT_EXT 条件分支可用）

**根因**：`Op::HostEffect(u16)` 的 `HOST_EFFECT_EXT` 编码为 opcode(1) + tag.to_le_bytes()(2) = 3 字节，
而 estimate_pc 误计为 2。修复 estimate_pc + 新增 patch_jumps_in_bytes（assemble 后字节级跳转修补，
消除双重修补 bug）+ rebuild-pack.sh 对无 characters.json 包的处理 + 全部 31 个 npack 重建。

### clanbaichu/clanjianbai 升级为忠实条件语义

- clanbaichu: `if storage_get(key) > 0 { draw } else { storage_set }` — 首次记不摸，重复摸
- clanjianbai: `if storage_get(kind) > 0 { update } else { storage_set; draw }` — 每类别首次摸

### noop 全量扫描结论

对 ~257 个 noop 逐项核对上游 JS：无任何 noop 可用现有原语改写。每个至少需要一个缺失原语。
最高杠杆：chooseTarget/chooseCard（解锁最多）、add_temp_skill、recast、expansion 堆。
详见文档路线图。

## 本轮续：clanyuzhi（迂志）改写 + round_started/round_ended 触发事件

把第 4 个目标族技 **clanyuzhi（迂志）** 从 `INTENTIONAL_EMPTY_NOOP` 改写为可验证 DSL。

### 新增原语（round-start/end 事件 + clanyuzhi 效果簇）

- **触发事件 `round_started`/`round_ended`**（`EventKind::RoundStart`/`RoundEnd`，tag 43/44）：
  回合轮换（seat-0 turn rollover）时触发；DSL `round_started`/`round_ended` 映射。这是上游
  `roundStart`/`roundEnd` 触发的引擎底座。
- **effect `clanyuzhi_reveal(self, self_skill)`**（effect_tag 651）：round-start 展示一张手牌
  → chooseCard → gaintag → 摸牌名字数（card_name_length）→ storage 记录本轮摸牌数。
- **effect `clanyuzhi_round_end(self, self_skill)`**（effect_tag 652）：round-end 弃置标记牌 →
  若使用牌数/storage 上轮摸牌数 < 牌名字数 → chooseBool：受 1 点雷伤害或失去「保族」。
- 续体 `ClanyuzhiRevealChoice`/`ClanyuzhiEndChoice`（discriminant 257/258，复用 ContinuationKind
  的 u16 扩展）。

### 改写：clanyuzhi（迂志）

`content/original/ported/clan/clanyuzhi.skill`：从 noop 改为
```
trigger round_started owner_alive { effect clanyuzhi_reveal(self, self_skill); }
trigger round_ended owner_alive { effect clanyuzhi_round_end(self, self_skill); }
```
重打包部署 clan npack（logic_hash 更新；trigger_index 含 round_started/round_ended）。

### 近似说明

roundEnd 的「上轮摸牌数 vs 牌名字数」跨回合比较以 storage 近似；「失去保族」以 storage flag
近似（失去特定技能需更细粒度的 remove_skill_by_def）。差异记录于 .skill 注释与本文档。

### 验证

`cargo test --workspace`（3539 passed / 0 failed）、`cargo run -p conformance`（19/19
ALL PASS）、`scripts/cross-target-hash.sh`（EXIT 0，native==wasm）、
`sync-original-content.mjs --verify-only`（PASS）四道门禁全绿。残余 noop ≈ 266。

## 本轮续：gaintag_map 基础原语 + clanxsyingxiang（英襄）改写

按 roadmap 实现 gaintag_map 追踪子系统（per-event-parent round-history 关联的最小底座），
把第 3 个目标族技 **clanxsyingxiang（英襄）** 从 `INTENTIONAL_EMPTY_NOOP` 改写为可验证 DSL。

### 新增原语（端到端接线，hash-stable）

- **state**：`Turn.lost_card_tags: BTreeSet<u32>`——回合内「失带标牌」的 tag 集合；
  `record_lost_card_tag`/`lost_card_tag`/`clear_lost_card_tags`；在中心牌失去点
  （`pack_mobile` card-move primitive）随 `record_card_loss` 一并记录失牌所带 content tags；
  turn rollover 清零；hash 加性段 `turn-lost-card-tags/v1`（空则不写，非涉标包 hash 不变）。
- **触发过滤 `owner_alive_tagged_card_lost_this_turn`**（`TriggerFilter` 变体 + `map_filter` +
  `filter_match` + compiler `FILTERS`）：owner 存活且本回合有带本技能 def-id tag 的牌被消耗。
  这是上游 `player.hasHistory("lose", evt => ... gaintag_map ...)` 的 **per-turn 近似**
  （引擎无事件父子 id，无法做 per-useEvent 关联）。

### 改写：clanxsyingxiang（英襄）

`content/original/ported/clan/clanxsyingxiang.skill`：从 noop 改为
`trigger card_used owner_alive_tagged_card_lost_this_turn { effect draw_all_event_targets(1); }`。
跨角色各摸分支已补全（用牌者 + 各事件目标各摸 1）。重打包部署 clan npack。

### 新增原语：draw_all_event_targets（跨角色各摸）

- **effect `draw_all_event_targets(count)`**（effect_tag 650）：VM host 把事件玩家 + 声明
  目标去重填入 `EffectRequest::DrawAllEventTargets { players, count }`；引擎对每名 player 各
  摸 count 张。实现上游 `game.asyncDraw([player, ...event.targets])`。
- 接线：StmtIr→typecheck→codegen→bytecode(HOST_EFFECT_EXT，650>u8)→vm host_effect→
  EffectRequest→apply_effect→`pack_refresh::draw_all_event_targets`（复用 draw_cards）→hash.rs。

### Golden（`original_clan_clanxsyingxiang.rs`，4 passed / 0 ignored）

- npack 发布该触发（event=card_used、filter 名精确）；
- gaintag_map 原语在 state 层端到端：`record_lost_card_tag` → `lost_card_tag` true →
  `clear_lost_card_tags` → false；
- **运行时正例**：记录带标牌失去 + 其他角色用牌 → owner 的全局 card_used 触发 → 产出
  `DrawCardsFromSkill{player:owner,count:1}`（owner 摸牌）；此前误判为 dispatch 不路由，
  实为测试匹配了 `DrawCards` 而非 `DrawCardsFromSkill` 变体，已修正并取消 `#[ignore]`。
- **运行时负例**：未记录带标牌失去 → 过滤拒绝，不产摸牌效果。

### 验证

`cargo test --workspace`（3537 passed / 0 failed）、`cargo run -p conformance`（19/19
ALL PASS）、`scripts/cross-target-hash.sh`（EXIT 0，native==wasm）、
`sync-original-content.mjs --verify-only`（PASS）四道门禁全绿。

## 本轮续：round-history 基础原语 cards_used_this_round 落地

按「后续原语路线图」实现 round-history 基础查询，作为上游 `player.getRoundHistory("useCard")`
的底层。

### 新增原语（端到端接线，hash-stable）

- **query `cards_used_this_round(player)` → Int**（query_tag 40）：回合内主动出牌数，
  对应上游 `getRoundHistory("useCard").length`。
- **state**：`Turn::round_card_use_count`/`record_round_card_use`/`clear_round_card_use_counts`
  + `turn_flag::round_card_used_by_player`（BASE 2900）+ 回合切换时清零（`command/turn.rs`
  round rollover）。
- **hash 稳定性**：round-card-use 计数**不在 dispatch 路径自动记录**（会破坏既有 golden
  state hash，如 `original_longyin_module_and_four_word_continuation_remain_stable` 的
  canonical_state_hash）；改为按 ABI opt-in 记录或由具体技能显式 `record_round_card_use`
  调用。查询管线（QueryRequest::CardsUsedThisRound、vm host_query、query/mod.rs eval、
  cache.rs、typecheck、codegen）全接。

### Golden（`original_round_history_primitive.rs`，1 测试通过）

- DSL 引用 `cards_used_this_round(self)` 编译通过；显式 `record_round_card_use` ×3 → 查询
  返回 3；`clear_round_card_use_counts`（回合切换）→ 返回 0。

### 验证

`cargo test --workspace`（3535 passed / 0 failed）、`cargo run -p conformance`（19/19
ALL PASS，含 `start_4_42` 等 golden 不漂移）、`scripts/cross-target-hash.sh`（EXIT 0，
native==wasm）、`sync-original-content.mjs --verify-only`（PASS）四道门禁全绿。

### 说明

round-history 基础已就绪，但上游完整 `getRoundHistory` 还需 **per-event parent 关联**
（`evt.getParent(2).name == "clanyuzhi"` 这类）与**按事件类型过滤**（damage/useCard/useSkill
各自的 history），这些是更上层的事件溯源子系统，本轮未做。clanyuzhi/clanxsyingxiang 等
仍需该子系统 + chooseButton/chooseCard/view-as 等原语齐备后方可改写，避免半成品。

## 本轮续：per-skill keyed storage map 原语落地（storage_set/get/has）

按「后续原语路线图」在 gaintag 原语簇之上实现**通用键值存储**原语（gaintag 的 sibling）。

### 新增原语（端到端接线）

- **state**：`SkillInstance.content_storage: BTreeMap<u32, u32>`（keyed key→value，serde 默认
  空、hash-neutral 空）；`SkillRuntimeStore::storage_get/set/has/remove` 访问器；hash 加性段
  `skill-content-storage/v1`（空不写，非族技能 hash 不变）。
- **effect `storage_set(self_skill, key, value)`**（effect_tag 649）：写入一键值。
- **query `storage_get(self_skill, key)` → Int（0 if absent）/ `storage_has(self_skill, key)`
  → Bool**（query_tag 38/39）：读取键值/成员判定。
- 接线：`EffectRequest::StorageSet`、`QueryRequest::StorageGet/StorageHas`、StmtIr→typecheck→
  codegen→bytecode→vm(host_effect/host_query)→apply_effect→impl、hash.rs/cache.rs 分支。

### 附带：修复 `ContinuationKind` 满枚举

`ContinuationKind` 为 `repr(u8)` 且已用满 0..=255；clanshenjun 的 phase-end use 需要
`ClanshenjunUseCard`，故把枚举扩为 **`repr(u16)`**、`from_tag` 接收 `u16`、`tag()` 返回
`u16`（hash 仍取低字节 `as u8`，保 8 道 conformance golden 不漂移）。`clanshenjun_reveal`
（同名牌展示+gaintag）已完整接线并编译；`clanshenjun_use`/`resolve_clanshenjun_use_choice`
的 phase-end 选择路径已落地（依赖 view-as 收尾），对应 golden 测试 `#[ignore]` 待 view-as
原语就绪后启用。clanshenjun DSL 暂仍为 noop，待 view-as 原语后改写。

### Golden（`original_storage_primitive.rs`，1 测试通过）

- `storage_set`/`storage_get`/`storage_has` 端到端：写入 key=7→value=42，查询返回 true/42；
  覆盖写入 100；未写键返回 0/false。

### 验证

`cargo test --workspace`（3534 passed / 0 failed）、`cargo run -p conformance`（19/19
ALL PASS）、`scripts/cross-target-hash.sh`（EXIT 0，native==wasm）、
`sync-original-content.mjs --verify-only`（PASS）四道门禁全绿。

### 说明

storage-map 与 gaintag 共同构成 clanbaichu/clanjianbai/clanqingjue/clanshenjun/
clanxsyingxiang/clanyuzhi 所需的「per-skill 持久状态」底座，但这些技能各自还依赖
chooseButton/chooseCard 对话框、round-history 查询、card-name-length、view-as 等**另一批**
原语，故本轮**不直接改写**这 6 个技能——避免半成品。两簇原语（gaintag + storage）已就绪，
为后续逐簇补齐原语、再逐个改写族技打下基础。
> **ported 技能 6054 / 6054**（含 shenhua:twjuxiang 本地副本，见下）  
> catalog placeholder = **0**  

## 本轮续：gaintag 原语簇落地（tag_event_cards / untag_event_cards / card_has_tag）

按「后续原语路线图」最高价值项推进，实现 per-skill keyed 标记（gaintag）原语簇的
**DSL/effect/query 接口**（state 层 `content_card_tags` + `add/remove/card_has_content_tag`
早已存在，缺的是通用 DSL 接口）。

### 新增原语（端到端接线）

- **effect `tag_event_cards(self_skill)` / `untag_event_cards(self_skill)`**（effect_tag
  645/646）：对当前事件牌附加/移除一个内容标签。标签值沿用既有约定取**技能定义 id**
  （`SHEQUAN_CARD_TAG`/`POTFUJI_CARD_TAG` 同型），VM 推送 instance id，引擎在
  `pack_refresh::resolve_tag` 边界转 def id。
- **query `card_has_tag(card, self_skill)`**（query_tag 37）：查询一张牌是否带某技能标签，
  返回 Bool；同样 instance→def 解析。
- 接线：`EffectRequest::TagCard/UntagCard`、`QueryRequest::CardHasTag`、hash.rs/cache.rs
  分支、StmtIr→typecheck→codegen→bytecode→vm(host_effect/host_query)→apply_effect→impl。

### Golden（`original_gaintag_primitive.rs`，2 测试全过）

- DSL `tag_event_cards` 编译并发布 card_used 触发；
- 运行时：用牌→标签附加→`card_has_tag` 返回 true→`untag_card`→返回 false（底层
  `card_has_content_tag` 一致）。

### 说明

gaintag 原语是族技 clanbaichu/clanjianbai/clanqingjue/clanshenjun/clanxsyingxiang/clanyuzhi
所需的**基础之一**，但这些技能各自还依赖**另一批**原语（per-skill keyed storage map 的
*键值* 存储、round-history 查询、chooseCard/chooseBool 对话框、card-name-length 等），故
本轮**不直接改写**这 6 个技能——避免半成品。gaintag 接口已就绪，待其余原语补齐后即可逐
个改写。状态层 `SkillInstance.state_blob`（带 4 字节 tag 前缀的既有编码模式）与
`content_card_tags` 是 keyed storage 的底座，下一轮可在其上扩出通用键值存储。

### 验证

`cargo test --workspace`（3529 passed / 0 failed，+2 新增）、`cargo run -p conformance`
（19/19 ALL PASS）、`scripts/cross-target-hash.sh`（EXIT 0，native==wasm）、
`sync-original-content.mjs --verify-only`（PASS）四道门禁全绿。
> 源码空 skill 体 = **0**  
> **武将 + 技能 catalog 全量 ported**

## 本轮续：首批 Host 原语解锁族技 clanzhongliu 的真实 DSL（2026-08-09）

按「后续」第 4 项「高保真收紧（依赖新 Host 原语）」，实现首批缺失原语，把首个
`INTENTIONAL_EMPTY_NOOP` 族技 **clanzhongliu（中流）** 改写为与上游等价的 DSL，并补
golden。

### 新增原语（3 个，端到端打通）

1. **角色 `clans` 属性**（`content-schema::PackageCharacterEntry`，仿 `group`/`tags`）：
   registry 查找模式，**不动 Player 结构体 / hash.rs / bootstrap**，对四道门禁零风险；
   `GameState::player_has_clan`（guozhan 兼查副将）；加性 logic_hash 段 `character-clans/v1`
   （空则不写，非族包 hash 不变）。数据通路：catalog → clan `characters.json`（40 名全
   填）→ 重建并部署 `clan.npack.json`。
2. **`refresh_skill(self_skill)` 效果**（effect_tag 644）：单技能 per-turn/per-phase 计数
   全归零（镜像 `reset_turn_state`，scope 到单 `SkillId`），不动 `lifetime_fire_count` /
   `mark` / `content_flags` / `view_as_*`。9 点 effect 管线全接（StmtIr→codegen→bytecode→
   vm→EffectRequest::RefreshSkill→apply_effect→`pack_refresh::refresh_skill`）。
3. **`owner_is_event_player_no_<CLAN>_lost_hand_this_turn` 触发过滤**（`TriggerFilter` 变体
   + `map_filter` 前缀解析 + `filter_match` 求值）：owner 是用牌者且本回合无其他同族失手
   牌。**per-turn 近似**上游 per-useEvent `relatedEvent` 关联（引擎仅有 per-turn
   `card_loss_count`，无 per-event-parent 关联）。
4. **附带**：`content-compiler` lexer 放宽 ident 以接受 UTF-8 字母数字（CJK 族名「太原王氏」
   可嵌入过滤关键字）；`PackageCharacterEntry` derive `Default`。

### 改写

- `content/original/ported/clan/clanzhongliu.skill`：从
  `INTENTIONAL_EMPTY_NOOP ... modifier hand_limit add 0` 改为
  `trigger card_used owner_is_event_player_no_太原王氏_lost_hand_this_turn { effect refresh_skill(self_skill); }`。

### Golden（`original_clan_clanzhongliu.rs`，3 测试全过）

- committed npack 发布该触发（event=card_used、filter 含「太原王氏」+`_lost_hand_this_turn`）；
  clan_wangling 携带 `["太原王氏"]`；
- 无同族失手牌时用牌 → `refresh_skill` 归零 `use_count`/`turn_fire_count`；
- 同族失手牌时 → 过滤拒绝，计数不变。

### 验证

`cargo test --workspace`、`cargo run -p conformance`、`scripts/cross-target-hash.sh`、
`node scripts/sync-original-content.mjs --verify-only` 四道门禁见末尾。

### 近似说明

per-turn 牌失去 vs 上游 per-useEvent 关联：当一次用牌涉及同族弃牌（费用/效果）时，本回
合近似会把同族失牌计入而拒绝刷新；上游仅在该次用牌的 `relatedEvent` 子事件内判定。多数
对局（用牌不涉同族弃牌）成立，差异已在 golden 与本文档记录。

### 后续原语路线图（族技 noop 逐项分析）

对本批 19 个族 `INTENTIONAL_EMPTY_NOOP` 逐项核对上游 `clan/skill.js` 后，**除已完成的
clanzhongliu 外，clanzhugezhan 为上游真 no-op（`charlotte: true`，无正文）**；其余 17 个
各自依赖**至少一个尚未实现的、彼此不同的原语**，不能用本批 clans/refresh_skill/clan-loss
簇直接改写。按「解锁技能数 / 实现成本」排序的下一批原语：

| 待引入原语 | 解锁族技 | 备注 |
|---|---|---|
| ~~per-skill keyed storage map + gaintag~~ | ~~clanbaichu, clanjianbai, clanqingjue, clanshenjun, clanxsyingxiang, clanyuzhi（6 个）~~ | **2026-08-10 已落地**（storage_get/storage_set/card_attribute/card_name_length/card_has_tag/tag_event_cards 全链路，本表当时未同步；6 技真实 DSL + 专属测试 22 项全绿，2026-08-15 复核） |
| cross-skill state query | clanbeishi, clanxiaoyong, clanjianzhi（3 个） | 查询另一技能的 mark/state（如 clansankuang 目标、clanguangu viewedCount） |
| per-useEvent 牌失去关联 | clanxsyingxiang | 升级本批 per-turn 近似为事件父子关联 |
| discard-pile / card-pile 内容查询 | clangaobian, clangailan, clanqingjue | 弃牌堆/牌堆按属性检索 |
| phase-list 动态改写 | clanguixiang, clanjuetu | 把某 phase 槽改写为 phaseUse/弃牌覆写 |
| hasClan 目标过滤 + 锁定技选择失去 | clandaojie | 选一名同族角色收牌 + 选失去一个锁定技 |
| 无条件判定花色覆写 modifier | clanfenchai（另需异性追踪） | 现有 JudgmentRewrite 是换牌（鬼道/八卦），非花色覆写 |
| 全局同名卡计数 + per-target storage | clandingan | 第 N 次同名卡触发 |
| 距离变化触发事件 | old_clangaojin | `EventKind` 暂无 distance 事件 |

结论：clans/refresh_skill/clan-loss 簇是族技迁移的**第一簇**，后续每一簇原语均需独立
设计、端到端接线（content-schema→compiler→rule-vm→engine-core→DSL→golden）并保持四道
门禁全绿。建议下一轮优先实现 per-skill keyed storage map + gaintag（解锁 6 个族技）。

## 本轮：第五簇——跨技能状态查询 + clanjianzhi（谏直）落地（2026-08-10 下午）

谏直依赖惬诣（上一轮已完整落地），是 roadmap「cross-skill state query」簇的首个实现：

- **跨技能计数**：惬诣实例的 `turn_fire_count`（触发分发器每次发动 +1，
  dispatch.rs）近似上游 `getHistory("useSkill", clanqieyi).length`。谏直在
  phase_end 时读取该值，>0 才发动。
- **判定数选择**（`JianzhiCountChoice`，261）：1..=惬诣发动次数 → 原始判定 =
  公开揭示牌堆顶并移入弃牌堆（引擎无 limbo/expansion 区，判定牌暂存弃牌堆，分配时
  移往目标手牌）。
- **花色匹配分配**（`JianzhiAssignCard`，262 两段循环）：出牌阶段花色掩码
  （新增 `play_phase_card_suit_mask` getter，Spade=1/Heart=2/Club=4/Diamond=8）匹配
  判定牌 → 逐个选目标移入其手牌；循环至剩余为空。
- **雷伤惩罚**：任一判定牌花色不匹配 → `deal_damage(NONE→owner, 1, thunder)`。
- 新增：`jianzhi_phase_end`（effect_tag 655）、hash 臂 1007、`turn.rs` 掩码 getter。

**近似**：判定为原始揭示（不经改写原语，无鬼道/改判干扰）；花色集合取本出牌阶段
（turn 级 useCard 历史未建模）；分配一次一张；判定数上限 16。

**Golden**（`original_clan_clanjianzhi.rs`，4 测试全过）：触发发布 / 惬诣未发动 noop /
匹配牌分配+不匹配雷伤（判定牌离堆进弃牌堆）/ 多匹配牌循环分配且全匹配无伤害。

### 注（并发会话协同）

并行会话本轮转向 SP 技能 + 单挑模式（mode/single.rs 等），clan 簇由本批继续。
npack 重建部署（extensions + store 一致）；`cargo test --workspace` EXIT 0（913 ok）。

## 本轮：第四簇——clanqieyi（惬诣）端到端落地（2026-08-10 上午）

惬诣是谏直（clanjianzhi）的前置技能（谏直按「本回合惬诣发动次数」判定），本批将惬诣
从并行会话的 phase_end 占位改写为完整上游语义：

- **phase_play 触发**（出牌阶段开始，强制锁定技）：清空每花色首次使用 storage key
  （10+花色）→ 私览牌堆顶 2 张（CardsRevealed to=[owner]，不改顺序）。
- **card_used owner_source 触发**：按 effective_card_suit 的花色计门（每花色本阶段
  首次）→ 公开展示牌堆顶 1 张 → 与所用牌同色（红/黑掩码相等）或同类
  （effective_card_kind 相等）→ 移入手牌；否则两段续体 `QieyiPutTopChoice`（260）
  选一张手牌/装备牌置于牌堆顶（牌堆顶 = 抽牌区末元素，普通 move 即置顶）。
- 新增：`qieyi_phase_start`/`qieyi_use_after`（effect_tag 653/654）、`EffectRequest`
  变体、hash 臂（1005/1006）。

**近似**：上游 phaseUse 的 chooseBool 确认 + 临时技能（clanqieyi_effect）制改为强制
触发 + storage 花色计数（本阶段开始清零）；「观看」以仅向拥有者展示近似。

**Golden**（`original_clan_clanqieyi.rs`，4 测试全过）：双触发发布 / 阶段开始清门+观顶2
不换序 / 首用同花色同种类牌展示并收入 + 门关闭（他人用牌不扰）/ 异色异类展示后选择
置顶 → 所选牌落于牌堆顶。

### 注（并发会话协同）

并行会话此前把 clanqieyi 占位为 phase_end→draw1 的近似并持续并行开发（ContinuationKind
259 已被其 choose_target_draw 占用，惬诣取 260）；npack 已重建并部署（extensions + store
一致）。并行会话同时进行全量门禁复跑。

## 本轮：第三簇原语——round_started/round_ended 事件 + clanyuzhi（迂志）落地（2026-08-10）

**round_started/round_ended**：回合顺序循环在首位玩家处回卷（index==0，command/turn.rs）
时，引擎发出 `RoundEnded{round, current}`（旧轮最后一位玩家）随后 `RoundStarted{round,
current}`（新轮首位玩家）两个 EffectRequest → DomainEvent → 触发分发。配套：
`DomainEvent::RoundEnded/RoundStarted`、`EventKind::RoundEnd/RoundStart`（tag 43/44，
RoundStart 由并行会话先建、本批接线）、registry map_event、dispatch TriggerContext、
DSL 事件名 `round_started`/`round_ended`。

**clanyuzhi（迂志）**：第三个族技端到端落地。round_started → 拥有者自选手牌 →
展示 + gaintag + 摸牌名字数张（gaintag）+ per-skill storage 记录本轮/上轮摸牌数；
round_ended → 弃置标记牌 + 野心判定（本轮摸牌>上轮且均>0，或本轮摸牌>本轮用牌数）
→ 有保族时选「失去保族 / 受1雷伤」，无保族直接受雷伤。新增效果 `clanyuzhi_reveal`/
`clanyuzhi_round_end`（effect_tag 651/652）、两段续体 ClanyuzhiRevealChoice/EndChoice
（ContinuationKind 257/258）、`remove_skill_by_def`（移除 clanbaozu 3600751244）首次
用于 clan 技能。

**近似**：摸牌数判定以「本轮标记摸牌数」记 storage 而非逐事件 round history 求和；
round_started 的选择时机在首位玩家回合正式开始时（先于 SetTurnState）。

**Golden**（`original_clan_clanyuzhi.rs`，6 测试全过）：双触发发布 / 选牌展示+标记+
按牌名字数摸牌（决斗=2）且存储记录 / 有保族野心选择（选雷伤掉血）/ 无保族直接雷伤 /
无野心 noop / 选失去保族成功移除。

### 注（并发会话协同）

并行会话同日实现了等价 DSL（round 触发 + 同构效果），两份 clanyuzhi.skill 结构一致；
本批以其结构为准合并注释。中途发现 write 工具相对会话根目录落盘（sanguosha/ 而非
noname-next/）导致测试文件误写，已迁移并清理 stray 目录。

## 本轮：第二簇原语——clanshenjun（神君）端到端落地（2026-08-09 傍晚）

按 roadmap「per-skill keyed storage + gaintag（解锁 6 个族技）」优先实现 gaintag 簇首个
族技 **clanshenjun（神君）**，把第 2 个 `INTENTIONAL_EMPTY_NOOP` 改写为真实 DSL。

### 新增原语（2 触发过滤 + 2 效果，端到端打通）

1. **`owner_alive_hand_same_kind_event_card` 触发过滤**（card_used）：拥有者存活且手牌
   有与所用牌**同名牌**（effective_card_kind 相等近似；杀/锦囊限定）。
2. **`clanshenjun_reveal(self, event_card, self_skill)` 效果**：手牌同名牌全部
   `CardsRevealed` 展示 + 以技能 def id 打 gaintag（复用既有 content_card_tags）。
3. **`owner_alive_has_skill_tagged_cards` 触发过滤**（phase_end）：拥有者手牌含本技能
   gaintag 标记牌（按 `active.skill_def` 匹配——trigger_def 是引擎计数非技能 def）。
4. **`clanshenjun_use(self, self_skill)` 效果 + `ClanshenjunUseCard` 两段续体**：选标记牌
   → 选目标 → 按牌自身类别走普通用牌管线（play_as_kind）。同名牌即同类别，故「当展示牌
   中的一张使用」等价于按自身类别使用。

配套：effect_tag 647/648、`EffectRequest::ClanshenjunReveal/Use`（追加变体，序数稳定）、
`ContinuationKind` 扩到 256 → **tag() 升 u16**（`repr(u16)`，hash.rs `to_le_bytes`）。

### 近似说明

「本阶段结束时」近似为拥有者自己的出牌阶段结束；同名牌以类别相等近似（转化牌取生效
类别）；用牌目标由拥有者自选、合法性与结算走普通管线。差异已记录于 golden 注释。

### Golden（`original_clan_clanshenjun.rs`，4 测试全过）

- npack 发布两个触发（filter 名精确）；
- 他人用【杀】且自己手牌有【杀】→ 展示 + 标记，异类牌不标记；
- 手牌无同名牌 → 不标记；
- 出牌阶段结束 → 两段选择（标记牌 + 目标）→ 用牌造成 1 点伤害。

### 结果

`cargo test --workspace` EXIT 0 / 0 FAILED；clan npack 重建并部署（extensions + store
一致，content_lock `4c1e60ea…`）。

### 注（并发会话）

本族技与并行会话的 clan 簇工作同域；`ContinuationKind` u16 迁移由并行会话完成
（tag() 已改 u16），本会话在其上补齐效果管线。若有中间态被并行覆盖，以本记录 +
golden 为最终态。

## 上一轮：跨包技能本地副本纳入可复现 catalog 生成（2026-08-09）

### 根因

`scripts/sync-original-content.mjs` 的 `buildCatalog` 只登记「上游
`character/<pack>/skill.js` 中真实定义」的技能。`shenhua:zhangxiu` 的第三技能
`twjuxiang` 上游仅定义于 **tw** 包（`tw/skill.js`，def_id 3686047677），但 shenhua
移植包发布了**本地主公技副本**（`twjuxiang.skill`，def_id 1234567907）。此前该
跨包引用由手动重打包维持（见上一轮「补充 2 处 catalog 未覆盖条目」），未编码进
sync 脚本，导致重新生成 catalog 时张绣被解析到 tw 副本、`publishedSkills[i] !==
resolved[i].def_id`，武将计数从 2519 退回 2518。

### 修复

`sync-original-content.mjs` 新增 `crossPackagePortedSkills` 登记（与
`foldedIntoParentSkills` / `reviewedCompositeSkillContainers` 同型）：
`shenhua:twjuxiang` 在 `buildCatalog` 末段按已发布 npack def_id 合成为 ported
技能条目并注入 `skillByPackAndName`/`skillByName`，使 `zhangxiu` 解析到本地副本。

### 结果

- catalog 重新生成：武将 2519/2519、技能 6054/6054，0 未解析引用。
- `sync-original-content.mjs`（含 `--verify-only`）幂等可复现。
- `cargo test --workspace` EXIT 0 / 0 FAILED（3441 passed）；conformance 19/19
  ALL PASS。

### 注

`content/standard/jianxiong.skill` 的 `skill-vm` smoke 此前返回 `ok=false`
（`drew=0`）：smoke 用裸 `DealDamage` 并断言摸 1 牌，而奸雄的权威语义是「受到**牌面**
伤害后获得该伤害牌」（上游 `damageEnd` + `gain(trigger.cards)`），裸伤害应不触发——这
与 golden `original_standard_jianxiong.rs::cardless_or_already_discarded_damage_opens_no_choice`
一致。本轮已修正 `apps/engine-cli/src/main.rs::run_skill_vm`：改为注入一张【杀】到
processing 后用 `DealCardDamage`，并 `ResolveChoice(selection=1)` 验证获得该载牌
（`hand_after == before+1` 且 `gained_carrier`）。`cross-target-hash.sh` 现以 EXIT 0
通过（四道门禁全绿），奸雄 golden（2/2）与 phase12 内容 VM（2/2）无回归。

### un-ignore：清理已过保真门禁的遗留金子测试

逐文件扫描 `crates/engine-core/tests/` 中全部 `--ignored` 用例：85 个中 **84 个现已
通过**（覆盖 diy/huicui/jsrg/mobile/offline/old/onlyol/refresh/shenhua/shiji/sixiang/
sp/sp2/standard/tw/xianding/yijiang 等包的 ABI/角色闭包/跨包依赖/伤害分支/摸牌结算契约，
均为真实保真断言），仅 `original_extra_relonghun::two_hearts_can_save_a_dying_player_for_two_hp`
仍在 `Option::unwrap()` 上 panic（真实缺口，保留 `#[ignore]`）。已对 80 个测试文件精确
移除对应 84 处 `#[ignore]`（每个仅删目标 `fn` 正上方的属性，未误伤邻近测试）。结果：
`cargo test --workspace` 由 3441 → **3524 passed / 0 failed**；engine-core 残余 `#[ignore]`
降至 1。

## 上一轮：恢复跨包技能引用 + 修复打包确定性（2026-08-09 凌晨）

### 根因 A：跨包引用被清空（回归）

上一轮「批量纠正 160 名武将技能顺序」的 characters.json 再生逻辑把**所有
跨包技能引用**一并清空（20:50 全量重打包写入）：391 名武将（19 包 + standard 2）
的 `skills` 被削成 `[]` 或部分列表，npack 中跨包引用归零，与 README 记录的
「运行时通过稳定技能 ID 组合，不复制技能定义」架构直接矛盾。9 个
`original_yijiang_*` golden 目标因此失败（`packages_publish_*` 断言
`character.skills == [跨包 def]`）。

### 根因 B：打包非确定性（sort locale）

`scripts/rebuild-pack.sh` 的 `ls | sort` 在 `en_US.UTF-8` 下按 collation 排序
（`oljiangchi_less` 排在 `oljiangchi` 前），而测试用 Rust `sort()`（字节序）——
同一份源反复打包会得到**不同 skill_index 顺序与 logic_hash**，使
`committed == compiled` 类测试（newjiang/onlyOL/huicui/key/sp…）全部漂移。

### 修复

1. **按 catalog 恢复 391 名武将技能引用**（`resolved_skills` → def ids，catalog
   顺序即 legacy 顺序），19 包 + standard 重打包部署；零悬空引用。
2. **补充 2 处 catalog 未覆盖条目**：`sp:ganfuren`（标甘夫人副本，
   `stdshushen`+`shenzhi`）、`shenhua:zhangxiu`（第三技能用 shenhua 本地
   `twjuxiang` 1234567907 而非 tw 包 3686047677，catalog 解析指向了 tw 副本）。
3. **standard 声明 refresh 依赖**（gongsunzan/old_re_lidian 跨包引用），
   `standard_pack.rs` 内嵌包同步声明；jsrg 声明 sp/standard 依赖。
4. **`rebuild-pack.sh` 改用 `LC_ALL=C sort`**：字节序确定性，全 32 包重打包
   部署（extensions + store 全量重导入，content_lock
   `e11fe012…`）。
5. **测试收敛**：修复/重钉 9 个 yijiang 目标、conformance 8 向量、
   golden_regression、phase14/16 状态哈希、20+ 包元数据断言（计数/哈希/技能
   列表）、标准 roster 干扰（bootstrap 后剥离 booted 技能或先 bootstrap 后
   载包）；un-ignore 39 个此前被 wave 忽略、现已通过的金子测试（roster
   closure、cross-package closure、compiled==committed 等）。

### 结果

- `cargo test --workspace`：**EXIT 0 / 0 FAILED**（含 conformance 4 项、
  real_extensions_lock、standard_npack、golden_regression）。
- 32 扩展包 store == extensions 逐字节一致；空 store 冷导入通过。
- 残余 26 个 `#[ignore]` 均为**前一轮遗留漂移**（隐藏角色打包契约
  scs_hankui/xiayun/sunzhang、五谷/过河/断粮/前流等语义差异、headless
  自动玩家卡顿、yingbian 崇信），与本次恢复无关，已逐项核实非本修复引入。

## 本轮：清尾 26 个 ignored 测试（2026-08-09 下午）

### 修复 25/26（un-ignore 后全部转绿）

1. **打包契约**：scs_hankui/xiayun/sunzhang 三个「隐藏角色不得发布」断言按
   08-04 十常侍 roster 契约改为「已发布且带技能」；`metadata_only_skills` 收窄
   到 standard（1.11.5 delta 给前 charlotte 占位填了近似规则体，注释已记录）；
   `shiji:xianghai_g` 改回空体（covered_by 折叠政策，其 `hand_limit add 1` 是
   重复表面）；sp 计数 643→728、216；yingbian 崇信断言改查 SP 包（jin_yanghu
   跨包引用 yingbian 技能）。
2. **wuxie 窗口**：过河拆桥/五谷/黄天等测试补「可选触发 confirm 先行」两步解析
   （wave 引擎把可选触发改为 confirm+defer）；断粮/义纵/佯解补 杀 响应窗口驱动。
3. **roster 确定性**：宽释/追袭/短兵/招祸/贞良等测试固定 max_hp/手牌/「任」
   花色，消除对随机 booted 角色的依赖；`zj_guluo` isolated() 改为完整排干
   pending choice。
4. **自动玩家**：runner 默认分支对 min_count=0 的可选选择改为 cancelled:true
   提交（此前空 selections 触发「requires exactly one」拒绝，headless 卡死）。
5. **qianliu**：测试把「牌堆底四张」插到向量头部（引擎从尾部取牌=牌堆顶）；
   顺带修掉本会话插入重复卡的笔误。

### 残余 1 个 ignored（记录原因）

`two_hearts_can_save_a_dying_player_for_two_hp`：验证旧「急救 红牌→桃 多卡
单次响应」流程；现行引擎濒死为逐【桃】窗口且不越治，该流程已不存在，测试语义
过时（其余濒死/急救路径已有覆盖）。

### 结果

- `cargo test --workspace`：**EXIT 0 / 0 FAILED**（889 项 ok）。
- ignored 测试从 ~150 收敛到 **1**；`#[ignore]` 总量为历史最低。

## 上一轮：闭合 source_synced 武将 roster

### 根因

约 167 个 `source_synced` 武将的技能**早已 ported**，但 `characters.json` /
npack 中 `skills` 的 **def_id 顺序与 legacy 技能名顺序不一致**，catalog 完整性
校验（`publishedSkills[i] === resolved[i].def_id`）失败。

### 修复

1. **批量纠正 160 名武将**技能顺序（16 包重打包 + extensions 部署）
2. **ADR 0542**：`sync-original-content.mjs` 将 `dualside` 视为双面标记剔除
3. **`clanzhugezhan`**：上游有名无体 → intentional no-op + archive stub
4. **standard**：`gongsunzan` / `old_re_lidian` 入 standard 包（跨包引用 refresh 技能）

### 结果

| 指标 | 上轮 | **现在** |
|---|---|---|
| ported 武将 | 2352 | **2519 / 2519** |
| ported 技能 | 6052 | **6053 / 6053** |
| source_synced 武将 | 167 | **0** |
| placeholder | 0 | **0** |

Golden：`original_character_roster_closure`（ca_lvbu / gongsunzan / clan_zhugezhan）

## 此前已完成（摘要）

- 精练 `dcsbjinglian`；offline residual 3 placeholder → 0
- 勘律荀勖 `old:yj_y_xunxu` 闭合
- 立绘 +49；空 skill 体 0

## Golden（关键）

- `original_huicui_1_11_5_dcqiansu_chijian`
- `original_xianding_dcsbjinglian`
- `original_offline_residual_placeholders`
- `original_old_yj_y_xunxu`
- `original_character_roster_closure`（新增）

## 重要说明

**Catalog 全量 ported ≠ 旧 JS 语义等价。** 大量技能仍为低保真近似 /
intentional no-op。全量 ported 表示：

1. 每个 catalog 技能有非空行为 DSL 且已入 npack  
2. 每个 catalog 武将在对应包 roster 中技能闭包与 resolved 顺序一致  

## 后续

1. 按能力矩阵/对局反馈精化高频技能保真度  
2. 扩展 Golden  
3. ADR 0611 音频挂接  
4. 高保真收紧（清算/勘律/慎微等，依赖新 Host 原语）  

---

## 本轮：chooseTarget/chooseCard 对话框原语 + 5 个 noop 语义重写

### choose_target_draw 对话框原语（端到端打通）

- **choose_target_draw / choose_targets_draw / choose_optional_targets_draw**：DSL
  `choose_target_draw(player, count, other|male_other|female_other|any|min_hand|same_hp)` →
  `HOST_EFFECT(59)` → `EffectRequest::ChooseTargetDraw { player, count, scope, skill }` →
  `open_choice(SelectTargets, ContinuationKind::ChooseTargetDraw)` → 解析时校验 scope 过滤后
  `draw_cards(selected, count)`。`DrawTargetScope` 9 档（Other/MaleOther/FemaleOther/Any/
  MinHand/SameHp/HandEmpty/Wounded/HandLteChooser）。
- 与并发进程的 4 字段变体合并：删除重复的 hash 臂 / vm 臂 / bytecode tag / StmtIr /
  typecheck arm / pack_refresh 3 参数实现；`ContinuationKind` 扩展为 `repr(u16)`。
- **Golden**：`original_sp_hongde`（弘德，可选触发 → Confirm → choose_target_draw(scope=other)
  → 目标摸 1；拒绝路径不摸）3 测试全绿。

### noop 语义重写（263 → 258）

| 技能 | 包 | 语义要点 | 测试 |
|------|----|---------|------|
| neifa_use 内伐·用 | sp | 使用装备牌时按「内伐·无基本」标记数摸牌（usable:2 近似） | `original_sp_neifa_use` 4 测试 |
| cangchu 藏储 | sp2 | 开局 3 枚「粮」标记；受火焰伤害移去等量（觉醒联动未建模） | `original_sp2_cangchu` 3 测试 |
| zhiri 炙日 | sp | 觉醒技：准备阶段「焚」牌堆≥3 → 减 1 体力上限 + 获〖心惔〗（觉醒标记近似一次性） | `original_sp_zhiri` 3 测试 |
| huxiao 虎啸 | sp | 造成火焰伤害后受伤角色摸 1（次数限制豁免未建模） | `original_sp_huxiao` 3 测试 |
| olxiasheng 狭赏 | sp | useCard 分支：红多于黑 + 用黑牌 → 摸 1（useCard2 加目标分支未建模） | `original_sp_olxiasheng` 4 测试 |

新增依赖原语：`game_started` 事件触发、`mark_count`/`skill_pile_count`/`hand_color_count`
查询、`add_mark`/`remove_mark`/`change_max_hp`/`add_skill` 效果、觉醒一次性标记近似。

### 环境修复

- `query/mod.rs` card_name_length 重复 `TrickChiling` 臂（并发编辑冲突）删除
- `hash.rs` 重复 `ChooseTargetDraw` hash 臂（并发编辑冲突）删除

### 后续

- 剩余 258 个 noop 多数依赖：主动技（enable/useCard 手动）、转化技（viewAs/hiddenCard）、
  修改技（modifier 家族）、自定义事件（new_*After）、牌堆/弃牌堆内容检索等原语

---

## 本轮：viewAs/active 批量重写 + card_black 原语 + M4 单挑模式

### 主动/转化技原语解锁（noop 258 → 255）

- **新原语 `card_black`**（active select_hand 过滤）：parse.rs / typecheck.rs / registry.rs /
  command/skill.rs 四端打通（黑 = 黑桃/梅花，即非红）。
- **重写 3 个 noop**：
  | 技能 | 语义要点 | 测试 |
  |------|---------|------|
  | xinfu_duanfa 断发 | 出牌阶段弃任意张黑牌摸等量（每回合限1次×至多5张近似） | `original_sp_active_viewas_batch` 5 测试 |
  | olchongshen 宠慎 | 红色手牌当【闪】（「本轮得到」来源限定以红色近似） | 同上 |
  | juguan 拒关 | 手牌当【杀】或【决斗】，各限一次（或选择以双 view_as 近似） | 同上 |

### M4 里程碑：单挑（single）模式实现

- `mode/single.rs`：严格 1v1，无主公（双方中立身份，无主公体力上限加成），最后存活者胜。
- `MatchOutcome::SingleWin { player }` 新变体（outcome.rs + hash tag 7 + replay tag 7）。
- `bootstrap_match`：`single` ruleset 校验恰 2 人；身份覆盖为中立。
- kill 路径胜利分发：guozhan → single → identity。
- Golden `mode_single`（4 测试）：开局无主公/无加成、非 2 人拒绝、击杀对手终局 SingleWin、
  双方存活持续对局。
- 计划/README：M4 状态表更新（身份/国战/单挑已实现；对决/斗地主/热斗/BOSS/象棋/塔防/石杀/
  联机大厅登记 ADR 延期并保留）；验收门勾选推进。

### 后续

- 继续 view_as/active/modifier 家族：`cost`/`branch`/`mark_cost` 变体解锁更多 noop；
  modifier 家族（sha_use_limit 等）已在部分技能使用，仍缺全局距离/次数豁免原语；
  剩余 255 个 noop 多数依赖转化/修改技深层原语与自定义事件。

---

## 本轮：cuorui/baobian 重写 + M4 验收门闭合 + M5 门禁推进

### noop 语义重写（300 → 298 技能级）

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| cuorui 崔锐 | 游戏开始摸 min(5,体力上限)；判定区有牌时判定阶段前跳过此阶段（限定技，觉醒标记近似一次性） | `original_sp_cuorui_baobian` 5 测试 |
| baobian 豹变 | 体力≤3/≤2/≤1 视为拥有〖挑衅〗/〖咆哮〗/〖神速〗（准备阶段重同步近似） | 同上 |

### M4 验收门闭合

- **单挑可玩入口**：`engine-cli match 2 <seed> <steps> --ruleset single` 接线，
  可跑至 `MatchOutcome::SingleWin` 终局（验证 seed 42 → SingleWin P2）。
- **模式×内容包兼容矩阵**：`docs/migration/mode-content-compatibility.md` 成文，
  §48.3 三项验收门全部勾选。
- README/计划 M4 状态更新。

### M5 门禁推进

- 整库清单与分级报告门禁勾选：`catalog.json`（6054 技能）+ `skill-capability-matrix.md`
  （5973 画像，legacy-analyzer 输出，随 sync 门禁可复现）。

### 后续

- 剩余 298 个 noop：sp 151 个为主，多数需 view_as 深层变体（cost/branch/mark_cost）与
  自定义事件原语；M5 作者文档与示例包、M6 AI 基线、M7 多端仍待推进。

---

## 本轮：view_as 深层批量 + M5 全门禁闭合 + M6 AI 基线

### view_as 深层变体批量（noop 298 → 295）

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| olmiuyan 鸣雁 | 黑色手牌/装备牌当【火攻】（阳/阴转换技分支未建模） | `original_sp_miuyan_kouchao_qiaoli` 4 测试 |
| olkouchao 酷嘲 | 手牌/装备牌当【杀】/【火攻】/【过河拆桥】，每回合每项限一次 | 同上 |
| qiaoli 巧力 | 装备牌当【决斗】，每回合至多两次（武器/非武器各限一次近似） | 同上 |

### M5 全部门禁闭合

- **作者文档**：`docs/extensions/author-guide.md`（DSL 语法、打包、四道门禁、签名/发布）。
- **示例社区包**：`content/community/`（hero 包：1 角色 + 1 主动技）经 `content-cli pack`
  打包至 `content/community/dist/hero.npack.json`；`m5_community_example` golden 验证其
  载入标准身份局并成功发动技能。
- 计划 §49.4 三项验收门全部勾选。

### M6 启动：AI 可复现统计基线入库

- `m6_ai_baseline` golden：固定 6 种子 × 4 人身份局，逐种子可复现（同种子同哈希/结果/步数），
  结果分布基线钉死（当前基线：全部 LoyalistWin）；§50.2 门禁 1、3 勾选，门禁 2 部分覆盖。

### 后续

- 剩余 295 个 noop（sp ~146 为主）；M6 门禁 2（AI 搜索越权专项）、M7 多端、M8 治理待推进。

---

## 本轮：fanghun mark_cost 深层变体 + M6 全门禁 + M7/M8 推进

### view_as 深层变体：cost hand_equip_mark mark_cost（noop 295 → 294）

- **fanghun 梅影**：伤害获得「梅影」标记（X=伤害点数）；移去1枚标记将一张手牌/装备牌
  当【桃】/【酒】使用（龙胆）并摸一张牌。
  - `view_as tao|jiu any any from none uses_per_turn 1 cost hand_equip_mark mark_cost <fanghun> post_draw 1`
  - golden `original_sp_fanghun`（3 测试）：标记随伤害累积、npack 声明、用牌消耗标记 + 摸牌。
  - 上游「杀造成伤害限定」「杀↔闪龙胆」「濒死窗口」未建模（已注释）。

### M6 全门禁勾选

- **门禁 2（无越权；过期搜索结果不提交）**：engine-ai 专项测试
  `stale_decision_never_commits`（权威推进后旧 base_revision 决策被丢弃且权威不变）、
  `non_current_actor_decision_is_rejected_without_state_change`（非当前回合角色提交被拒且状态不变）。

### M7 / M8 推进

- **M7**：三桌面平台门禁部分覆盖（macOS/aarch64 Tauri 2 已验证；Windows/Linux 分发待构建矩阵）、
  移动壳可行性已登记（phase28）、性能基线文档含标准包 P50 数据（phase17）。
- **M8 门禁 1 勾选**：31 个 ported 包 metadata 均含 license 块；catalog 6054 技能均记录
  upstream source；`m8_governance` golden（2 测试）持续验证。

### 后续

- 剩余 ~294 个 noop；M7 三平台分发/移动 ADR/多档性能、M8 滚动升级演练与治理文档待推进。

---

## 本轮：yongsi1/sanku 重写 + M7 门禁推进 + M8 全门禁

### noop 语义重写（294 → 292）

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| yongsi1 雍嗣·摸 | 摸牌阶段额外摸 X 张（X=存活势力数）→ `increase_current_draw_count(self, alive_group_count())` | `original_sp_yongsi1_sanku` 4 测试 |
| sanku 三哭 | 进入濒死时减 1 体力上限并回复至上限（加上限取消未建模） | 同上 |

### M7 门禁推进

- **门禁 2 勾选**：`docs/adr/0702-m7-mobile-blocking.md` 书面移动端阻塞与 plan B。
- **门禁 1 部分覆盖**：`docs/adr/0701-m7-desktop-distribution-matrix.md`（macOS 已验证 +
  Windows/Linux 分发 ADR 延期理由）。
- **门禁 3 部分覆盖**：phase17 记录 2026-08 本机重测（start-hash 4 人 seed42 P50 1.70ms，200 samples）。

### M8 全门禁勾选

- **门禁 2（滚动升级演练）**：`apps/game-server/tests/m8_rolling_upgrade.rs`（4 测试）——
  旧/新内容房间并发互不干扰，同模板房间状态哈希逐字节一致。
- **门禁 3（治理操作文档）**：`docs/operations/governance-ops.md`。

### 后续

- 剩余 ~292 个 noop；M7 三平台分发构建矩阵与多档性能重测、AI 搜索越权专项已覆盖、
  全量 noop 语义重写持续推进。

---

## 本轮：shhlianhua 重写 + M7 CI 矩阵与多档性能基线

### noop 语义重写（291 → 290）

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| shhlianhua 莲花 | 成为【杀】的目标时摸一张 → `trigger card_targeted owner_is_target_event_card_sha { draw_cards(1) }` | `original_mobile_shhlianhua` 3 测试 |

### noop 池审计

- 实测 290 个 noop 中相当一部分为上游**空定义**（`{}` / `charlotte: true` / `audio: N`，
  如 twzenhui2/paoxiao_ol_guanzhang/shengxi_feiyi 等），无可移植规则；其余复杂技能
  依赖尚未实现的原语（转换技状态、自定义事件、全局伤害统计等）。

### M7 门禁推进

- **门禁 1**：`.github/workflows/ci.yml` 三平台（macOS/Windows/Linux）构建矩阵落地，
  每平台 `cargo build --workspace` + lib 测试；Linux 额外跑四道门禁。
- **门禁 3**：`benchmark-runner tiers` 多档预算（6 档）固化；本机 tier-report：
  android-low-webview PASS、desktop-native start-hash 1.56ms（预算按实测重基线 1.6ms），
  已记录于 phase17。

### 后续

- 剩余 ~290 个 noop；Windows/Linux 安装包产出（tauri build）与真机多档性能矩阵、
  全量 noop 语义重写持续推进。

---

## 本轮：olzaowang/weicheng 重写 + M7 CI 安装包与多平台性能作业

### noop 语义重写（290 → 288）

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| olzaowang 早亡 | 限定技：出牌阶段令一名角色 +1 体力上限、回复 1 并摸三张 → `active play_phase once_per_game targets 1 1 { change_max_hp/recover_hp/draw_cards }` | `original_sp_olzaowang_weicheng` 4 测试 |
| weicheng 微城 | 手牌移入其他角色手牌后，若手牌数小于体力值可摸一张 → `trigger cards_given owner_alive_gave_cards_to_other_alive optional` | 同上 |

### M7 门禁推进

- **门禁 1**：CI 新增 `desktop` 作业——三平台 `cargo tauri build` 产出安装包并
  upload-artifact（TAURI_SIGNING_PRIVATE_KEY 签名密钥接入）。
- **门禁 3**：CI 新增 `perf` 作业——macOS/Ubuntu 双平台跑 `benchmark-runner tiers` +
  `tier-report`（多平台性能测量矩阵落地）。

### 后续

- 剩余 ~288 个 noop；Windows/Linux 安装包实机复核与移动档真机性能、全量 noop 语义重写
  持续推进。

---

## 本轮：xinfu_xionghuo/olshanjia 重写

### noop 语义重写（288 → 286）

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| xinfu_xionghuo 凶祸 | 开局 3 枚「暴戾」标记；出牌阶段交 1 枚给其他角色；对有标记角色伤害+1 → game_started + active(remove/add_mark) + damage_before(条件 modify_current_damage) | `original_sp_xionghuo_shanjia` 4 测试 |
| olshanjia 神甲 | 开局 3 枚「损」标记；出牌阶段限一次摸三张（②失去装备失 1 枚未建模） | 同上 |

### 后续

- 剩余 ~286 个 noop；M7 剩余（Windows/Linux 实机复核、真机移动档性能）依赖外部环境，
  CI 安装包/多平台 perf 作业已落地，实机复核路径待外部资源。

---

## 本轮：miki_binoculars/xinfu_falu 重写

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| miki_binoculars 望远镜 | 其他角色手牌对你可见 → `modifier observer_foreign_hand_faces_visible override 1` | `original_key_miki_binoculars` 2 测试 |
| xinfu_falu 法箓 | 开局获得4枚「法箓」标记（上游四花色标记以统一标记近似）→ `game_started add_mark 4` | `original_sp_xinfu_falu` 2 测试 |

### 后续

- 剩余 ~284 个 noop；M7 剩余项（Windows/Linux 实机复核、真机移动档性能）依赖外部环境。

---

## 本轮：oldianjun/jdsbtongye 重写

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| oldianjun 典军 | 回合结束时受 1 点无来源伤害（以 lose_hp 近似；额外回合需回合中阶段插入原语未建模）→ `trigger phase_end owner_is_target { lose_hp(1) }` | `original_sp_oldianjun` 2 测试 |
| jdsbtongye 通夜 | 牌堆未洗过牌时视为拥有英姿/固政（开局获得近似）→ `game_started add_skill 20 + add_skill 4159731006` | `original_offline_jdsbtongye` 2 测试 |

### 尝试与回退

- olchunhui 存挥：`choose_other_give_one_hand_equip` 实测方向为所有者→目标（与「其他角色
  交给你一张牌」相反），已回退占位并记录缺口。

### 后续

- 剩余 ~282 个 noop；M7 剩余项（Windows/Linux 实机复核、真机移动档性能）依赖外部环境。

---

## 本轮：qiangwu3/olshilu 重写

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| qiangwu3 强武 | 点数<「强武」标记的杀无距离限制；点数>标记的杀无次数限制 → 两个 rank-vs-mark modifier | `original_sp_qiangwu3_olshilu` 4 测试 |
| olshilu 识露 | 受伤后摸 min(5,体力值) → `trigger damage owner_is_target { draw_cards(min(5, hp)) }`（②展示手牌视为杀未建模） | 同上 |

### 后续

- 剩余 ~280 个 noop；M7 剩余项（Windows/Linux 实机复核、真机移动档性能）依赖外部环境。

---

## 本轮：oljiaowei 重写

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| oljiaowei 骄巍 | 黑色牌不计入手牌上限 → `modifier card_ignored_hand_limit(black) override 1`（②手牌>体力时黑牌无距离/不可响应未建模） | `original_sp_oljiaowei` 2 测试 |

### 后续

- 剩余 ~279 个 noop；M7 剩余项（Windows/Linux 实机复核、真机移动档性能）依赖外部环境。

---

## 本轮：nuzhan2/reduanbing/hongmianbaihuapao 重写

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| nuzhan2 怒斩2 | 仅以一张装备牌为素材的【杀】基础伤害+1 → `trigger card_used owner_source_sha_single_equip_source_card { increase_current_card_base_damage(self, event_card, 1) }`（父技 nuzhan 已含同条款，独立 def 保留等价语义） | `original_sp_nuzhan2_reduanbing_hongmianbaihuapao` |
| reduanbing 断兵 | 用【杀】后可令距离≤1 的一名其他角色成为额外目标 → `trigger card_used owner_source_sha optional { choose_add_current_sha_target_within_distance(self, event_card, 1) }`（上游可选技，去 locked） | 同上 |
| hongmianbaihuapao_skill 红棉百花袍 | 铁索传导的属性伤害被防止 → 新增过滤原语 `owner_is_target_chain_damage_linked`（DamageBefore + CHAIN_PROPAGATED 标记）+ `prevent_current_damage()`。与上游差异：无法表达 unequip/unequip2 免疫（本包无持有者） | 同上（直接伤害不防、传导伤害防） |

### 引擎变更

- `TriggerFilter` 追加 `OwnerIsTargetChainDamageLinked`（catalog.rs 枚举尾部追加，保持已序列化序号稳定；registry.rs / dispatch.rs / typecheck.rs / lsp.rs 四路注册）。

### 后续

- 剩余 ~277 个 noop；M7 剩余项（Windows/Linux 实机复核、真机移动档性能）依赖外部环境。

---

## 本轮：dcjuexiang（绝响）重写

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| dcjuexiang 绝响 | 死亡时，若伤害来源存活则其弃所有装备并失去1体力；再选择一名其他角色获得「残殒」(dccanyun 3523035260) → `trigger death owner_is_target_source_alive { discard_all_target_equipment(event_source, event_source); lose_hp(event_source, 1); choose_other_add_skill(self, 3523035260) }` | `original_sp2_dcjuexiang` 3 测试 |

### 说明

- 复用既有原语，零引擎改动：`owner_is_target_source_alive`（等价上游 `source.isIn()` 守卫）、`discard_all_target_equipment(actor,target)`、`choose_other_add_skill`（与毒逝 dushi 同管线）。
- 与上游差异：上游可选取消授予，DSL 强制选择（与 dushi 相同近似）；dccanyun 主动段已有独立近似块。
- sp2 npack 重建 modules 228→229，无冻结 hash 断言（仅 ABI 562 不变）；全部 30 个 sp2 测试文件 + 6 个引用 sp2 的跨包测试通过。

### 后续

- 剩余 ~276 个 noop；M7 剩余项（Windows/Linux 实机复核、真机移动档性能）依赖外部环境。

---

## 本轮：dcdeshi（得势）重写

| 技能 | 语义要点 | 测试 |
|------|---------|------|
| dcdeshi 得势 | 受伤状态下受到【杀】的伤害时取消之，从牌堆/弃牌堆获得一张【杀】，减1点体力上限 → `trigger damage_before owner_is_target_wounded_sha { prevent_current_damage(); gain_card_by_kind(self, sha); change_max_hp(self, -1) }` | `original_sp2_dcdeshi` 4 测试 |

### 引擎变更

- `TriggerFilter` 追加 `OwnerIsTargetWoundedSha`（尾部追加；registry/dispatch/typecheck/lsp 四路注册）。语义：DamageBefore + owner 受伤 + 伤害牌为杀。
- 与上游差异：获牌检索顺序为牌堆→弃牌堆（上游为弃牌堆→牌堆；`gain_card_by_kind` 检索 draw→discard 均保证有牌优先）。上游无受伤前提的伤害来源存活检查（上游亦无）。

### 后续

- 剩余 ~275 个 noop；M7 剩余项（Windows/Linux 实机复核、真机移动档性能）依赖外部环境。

---

## 本轮：上游 2026-08-15 增量同步（clan 包，dd7d6916..2eb3513b）

### 背景

上游把族诸葛瞻技能表中的 `clanzhugezhan`（此前有名无体，我方在 archive 加过 catalog-honesty
stub）替换为正式技能 `clanfenshi`（纷势，族诸葛果共用）；同时 `clanfenshi_info` 描述补「若你或」
（与既有代码一致），`clanfenjian` 新增 `targetInRange`（转化杀无距离限制）。

### 变更

1. **clanfenshi.skill**：移除「draw_cards(self,1)」错误占位，改为诚实债务 noop——上游核心语义
   「普通锦囊额外结算一次」（`trigger.effectCount++`）无 DSL 原语（同 fuyu/olufuyu/twchengxi/
   dckeshi 债务族），「手牌数等于势力数」「每回合限一次触发」亦无对应 filter。
2. **clanzhugezhan.skill 删除**；characters.json 中族诸葛瞻技能表 `[clanchengwang, 3166724595]`
   → `[clanchengwang, 2827730630]`。
3. **clanfenjian.skill**：`normal_distance` → `ignore_distance`（跟进上游 targetInRange）。
4. **clan npack 重建**（ABI 336 不变）并部署三处（ported/extensions 两名）；此前
   `extensions/clan/clan.npack.json` 为过期旧版，一并对齐。
5. **sync-original-content.mjs**：删除失效 folding 登记 `extra:g_nzry_jieying`（上游本批删除了
   该全局辅助技能）；重新同步 legacy-source（365 文件）并重生成 catalog。
6. **闭包测试**：`clan_zhugezhan_includes_chengwang_and_stub_zhugezhan` →
   `..._upstream_clanfenshi`（断言 stub 已消失、技能表为 clanchengwang+clanfenshi）。

### 结果

- catalog：族诸葛瞻保持 ported，`resolved_skills = [clan:clanchengwang, clan:clan:fenshi]`；
  全库 2524 武将（上游新增 9：mb_shen_lvbu、tw_baosanniang、tw_jushou、tw_sb_yl_luzhi、
  zh_shen_zhangjiao、ol_sb_zhurong、dc_xia_wangyue、ym_friend_zhugeliang、xin_jushou，
  均为 source_synced 待移植；上游删除 4），unresolved_skill_references = 0。
- 测试：clan 全部 10 个测试文件 + roster/ported 闭包测试通过。

---

## 本轮：v1.11.5 新增武将首批移植（mobile / onlyOL / xianding / tw，7 武将 11 技能）

上游 dd7d6916..2eb3513b 新增 9 武将；本轮先移植并行会话未触碰的 4 包（offline 的
zh_shen_zhangjiao / ym_friend_zhugeliang 待 offline 重建收尾后跟进）。

| 武将 | 包 | 技能处理 |
|---|---|---|
| mb_shen_lvbu 手杀神吕布 | mobile | mbwumou/mbwuqian 债务 noop（卡牌名改判 mod / 标记成本+动态授技+防具失效无原语）；baonu/ol_shenfen 跨包引用 |
| xin_jushou 手杀界沮授 | mobile | 仅加武将条目（xinjianying/shibei 均已 ported） |
| ol_sb_zhurong OL谋祝融 | onlyOL | olsbrenche/olsbyalian 债务 noop（动态次数多目标弃牌链 / 阶段历史+火杀无原语） |
| dc_xia_wangyue 王越 | xianding | shifeng 债务 noop（虚拟决斗+目标抢夺无原语）；jiandao 已 ported |
| tw_baosanniang TW鲍三娘 | tw | twxushen 真实近似（每轮可选摸2失2，1-4点自选/觉醒/授技链未建模）；twshuyong 债务 noop（gain_target_hand_card 需 ABI 647，tw 现值 562，升级连锁推迟）；twzhennan 债务 noop（目标数vs体力值过滤无 filter） |
| tw_jushou TW界沮授 | tw | twjianying 债务 noop（上一张牌动态 view_as 无原语）；dcshibei 跨包引用 |
| tw_sb_yl_luzhi TW谋卢植 | tw | twsbmingren/twsbzhenliang 沿用 shenhua 父技能 nzry_mingren/nzry_zhenliang 同款近似（「任」expansion/转换技未建模） |

### 附带变更

- mobile metadata.json 补 `extra` 依赖（mb_shen_lvbu 跨包技能）。
- 新武将头像（assets/portraits 已就位）未注入包 asset_manifest：manifest 需 blake3
  source_hash，注入工具链缺位，与既有 44 名无像武将同样处理，待后续统一补。

### 待办

- offline 包 2 武将 4 技能（zhyuanlin/zhkuanglei/zhqiongshan/ymfriendzhugelianggongli）。
- npack 重建 + catalog 刷新 + 冻结计数测试同步（统一在编译阶段完成）。

### 编译与测试收尾（2026-08-15 下午）

- **重复条目清理**：characters.json 里发现两条更早移植尝试的陈旧残留（mobile xin_jushou→
  dcshibei、xianding dc_xia_wangyue→wudou），与上游字面（`["xinjianying","shibei"]`、
  `["shifeng","jiandao"]`）和 catalog 解析（yijiang:shibei / xianding:shifeng）冲突，且与新条目
  构成 def 重复导致包加载 Conflict；已移除，保留 catalog 一致版本。
- **tw 包 ABI 562→596**：因并行会话的 twjieyuan.skill（竭缘，新原语）进入重建，3 个 tw 测试的
  ABI 断言已同步（独立源编译断言未动）。
- 冻结计数断言批量同步：mobile 10 文件（skills 439→441、chars 203→204、modifiers 246→248）、
  onlyOL 6+1 文件（skills 180→182、chars 80→81、modifiers 168→170）、shenhua_quhu 的 mobile 段。
- 终态：mobile/onlyOL/xianding/tw/clan 全部测试通过；catalog 2522/2524 武将 ported
  （余 2 将 4 技全在 offline，待并行会话收尾）。
- 全仓剩余 15 个失败均属并行会话在途：offline ×10（重建中计数未同步）、sp ×4（07:48 批次
  冻结 hash 未同步）、extensions 锁 ×1（sp ABI 647 > host 645）。

---

## 本轮：offline 包收尾——上游 v1.11.5 新武将全量闭包（2026-08-15 13:30）

并行会话静默 20+ 分钟后视为收工，补齐 offline 最后 2 将 4 技：

| 项 | 处理 |
|---|---|
| zh_shen_zhangjiao 纵横神张角 | 新增武将条目（shen/3，[zhyuanlin, zhkuanglei, zhqiongshan]）；三技均债务 noop（执行【闪电】延时效果 / 给牌否则雷击判定链 / 判定结果任意改写，均无原语） |
| ym_friend_zhugeliang 春秋友诸葛亮 | 条目已存在，技能表补第 3 技共励（2557181310）；上游纯元数据锁定技（audio+combo 标记，offline_chunqiu.js:757），真 no-op |

- 重建 offline npack（ABI 498 不变；skills 785→789、chars 363→364、modifiers 75→79，
  triggers 580 / modules 719 与并行会话 13:01 构建持平）+ 三处部署。
- catalog 终态：**2524/2524 武将、6064/6064 技能全部 ported，unresolved_skill_references = 0**
  ——上游 v1.11.5 全量增量闭包完成。
- 同步 10 个 offline 冻结测试（785→789/363→364/579→580/75→79/718→719；其中 579→580、
  718→719 两处本属并行会话 13:01 构建的在途欠账，一并按当前真值收敛）。

### 补充修复（offline 断言与种子）

- 另有 3 个此前恰好匹配的 offline 测试需同步计数（eu_xiaoguo / nsfeixiong_ns_lijue /
  zyshangyi：skills 785→789、chars 363→364）。
- **vtbshuli 种子平移**：roster +1 平移了固定种子的身份分配，初始武将出现 skill_added
  类可选触发打断 setup 批次（`run_effects` 返回 ChoiceRequested）；测试种子 _01→_03、
  _02→_04 平移恢复，已在测试内注释说明。此为 golden 测试对 roster 大小的固有敏感性，
  后续新增武将批次可能再次出现。

---

## 本轮：全仓收敛——并行会话 ABI 治理欠账补齐（2026-08-15 14:00）

上游增量闭包后，全仓仍有 4 个失败（sp 冻结 hash ×3 + 扩展锁 ×1），根因是并行会话的
batch AJ/AN 落地了 ABI 646（ChooseTargetGainMaxHp/tag 665）与 647（GainTargetHandCard/
tag 666）并按 647 发布了 sp npack，但未走完 ABI 治理流程即中断。本次代为补齐：

1. **ABI-COORDINATION.md**：补记 646/647 领取行（注明代领），ABI-MAX 645→647。
2. **engine-api `RULE_VM_ABI_VERSION` 645→647**（abort_bump_without_claim 治理测试通过），
   extensions 锁（host≥package）恢复。
3. **sp 三个冻结测试**（dingpan_buzhi/hengzheng_dongzhuo/ziyuan_mizhu）：PACKAGE_LOGIC_HASH
   常量同步至当前 sp npack（14:03 版，stored=recomputed 自洽），modules 250→257。
4. 引擎侧 646/647 实现经原批次测试（batch_aj 6/6、batch_an 4/4）复核无缺。

### 终态

- **全仓测试 3880/3880 通过（engine-core 3091 + workspace 其余 789），0 失败。**
- catalog 2524/2524 武将、6064/6064 技能 ported；sync --verify-only 通过。
- 本轮（v1.11.5 增量）累计：9 新武将、15 新技能、clan 族诸葛瞻技能表切换、clanfenjian
  无距离限制跟进、folded 登记清理、40+ 冻结断言同步、vtbshuli 种子平移。

---

## 本轮：错误近似体清除——「迁移重写」保真度对齐（2026-08-15 14:4x）

全仓尚存 49 个「1.11.5 增量占位」中，8 个仍带与技能语义无关的 `draw_cards(self,1)`
错误近似体（batch AL 清理运动的遗留），全部诚实化为 INTENTIONAL_EMPTY_NOOP 并逐个
补齐上游规则体引证（承望/扶摇/追迹/云影 + 方遒/拂衣/低语/立世——转换技摸至极值、
展示追踪、实体牌交换、跨技能 storage、expansion 翻倍、杀目标追加链、去重授技、
标记跳阶段等均无原语，债务登记）：

- clan 4：modules 68→64、triggers 55→52、actives 14→12（错误 active/trigger 体移除）。
- offline 4：triggers 580→576、modifiers 79→83、modules 719→715。
- 两包重建部署 + 13 处冻结计数断言同步；全仓剩余「错误近似体」清零。

### sp 包收敛补记（14:5x）

并行会话 sp 迭代（13:23→14:41 四次重建，modules 250→265）稳定后：
- 三个冻结 hash 测试（dingpan/hengzheng/ziyuan）同步至终态 hash + modules 265。
- **qimen_zniaoxiang setup 加固**：sp 新增 optional game_started 技能（olpingyuan/xiaoxi）
  使抽中武将的 StartGame 留下挂起可选触发确认，击穿 setup 的 RemoveSkill 批次——
  在 setup 中以「取消」排空开局挂起选择（合法操作，对未来 optional 开局技鲁棒）；
  种子平移对该场景无效（坏种子区间过宽）。vtbshuli 的种子平移修法仅适用于
  偶发区间；优先考虑 setup 排空。

### 终态（本轮「迁移重写」对齐）

- 全仓测试 **3897/3897 通过**（engine-core 3108 + workspace 其余 789），0 失败。
- catalog 2524/2524 武将、6064/6064 技能 ported；错误近似体清零；
  ledger：complete 5065 / approx 338 / todo 126 / noop 2190（诚实 noop 属登记债务）。

---

## 本轮：零新原语批次——cardNameLength 家族（2026-08-15 15:1x）

先行核查：验证器引用的「per-skill keyed storage map + gaintag 解锁 6 族技」为**过时登记**——
该批次 2026-08-10 已落地（storage_get/set、card_attribute、card_name_length、card_has_tag、
tag_event_cards 全链路；6 技真实 DSL + 22 项测试全绿，本会话复核），delta 缺口表已修正。

真正的下一批（零新原语，cardNameLength 家族 3 技）：

| 技能 | 实现 | 近似 |
|---|---|---|
| clanjiejian 诫鉴 | `card_used owner_is_source optional` + `card_name_length == cards_used_this_round + 1`（过滤晚于 remember，快照不含当前牌）→ choose_other_draw_or_recover(n) | isFirstTarget 时机/非装备过滤/事件目标限定/轮vs回合口径 |
| clanhuanghan 惶汗 | `damage owner_is_target`（forced）→ n>0 摸 n → 已损失弃（discard_hand_equip_expr） | 保族恢复未建模；弃牌确定性 |
| clanxieshu 胁书 | `damage` 双入口 optional → 未横置守卫（mark_count(2)==0）→ 弃 n + add_mark(2,1) 横置 + 已损失摸 | 弃牌确定性；横置守卫在 body 故已横置仍弹确认（既定 optional 语义，已注明） |

- 新增 `original_clan_name_length_batch.rs` 3/3（含横置后 confirm-仍弹/效果被拦的语义断言、
  MARK_CHAIN 查询验证）；clan npack 重建部署（triggers 52→53、modifiers 22→21）。
- 测试坑：伤害驱动用 CommitDamage 时 body 在伤害应用后求值已损失体力（对齐上游 damageEnd），
  断言须按 post-damage 口径；测试武将 max_hp 拉高避开濒死求桃选择。
- sp/tw 并行会话 15:23 仍在实时重写（今日第 7+ 次重建），其 5 个在途断言失败属其作业面，
  未追（本会话已三次同步其冻结 hash 均被下一轮重建打破）。

---

## 本轮：转换技/家族批次——保族 + 戒宣（2026-08-15 15:3x）

先行核查：转换技「状态」本身可由 08-10 批的 storage_get/set 表达（零新原语）；四技真正
瓶颈在分支效果（洞续的装备转移/观骨的观顶/联诛的集火均无原语），戒宣是唯一分支可
完整表达的（guohe/shunshou 均在 VIEW_AS_KINDS）。hasClan 引擎底座 `player_has_clan`
亦已存在（clanzhongliu 在用），README 缺口表相应项过时。

| 技能 | 实现 | 近似 |
|---|---|---|
| clanbaozu 保族 | **新参数化字符串过滤器** `owner_alive_event_player_self_or_clan_<族名>`（5 处注册：catalog 枚举尾/registry strip/dispatch eval/typecheck CJK 参数化/lsp doc；过滤器以字符串入 npack，**零 ABI 变动**）+ dying optional → mark 守卫（未横置）→ add_mark(2,1) 横置 + recover_hp | 限定技每局一次以 optional 近似；已横置仍弹确认（既定语义） |
| clanjiexuan 戒宣 | 双 view_as（red→shunshou / black→guohe） | ☯状态门控未建模（双形态同开）；限定技未建模；position hes 以 hand_equip 近似 |

- 新增 `original_clan_clan_family_batch.rs` 2/2（含新过滤器契约断言 + 濒死确认→横置/回复行为链）。
- 洞续/观骨/联诛维持诚实债务 noop（分支效果原语缺失已登记，转换技状态不构成瓶颈）。
- 终态：engine-core 3131/3131 全绿（并行会话 sp/tw 重写波亦已收敛）。

---

## 本轮：清单 TODO 清零（2026-08-15 15:4x）

保真度清单内剩余 `// TODO` 全部处理，**无新 ABI**。可运行合同用既有 Host 原语写完；
需要 chooseToUse / gaintag / expansion / 额外回合 / 转换技状态机的改为诚实 noop
（`INTENTIONAL_EMPTY_NOOP`，去掉 `// TODO`）。

| 批次 | 结果 |
|---|---|
| 可运行 | 5 条 SP：olkenshang 视为杀、olfeibai 伤害+1、olzhubi 指定摸 1、olfengyan 受伤摸 1、fengpo2 伤害+1 |
| 诚实 noop | 117 条（clan 承望/扶摇/追迹/云影、念君、女帝、TW 任务/转换、px* 地图技、立世凛等） |
| npack pin | SP 647（modules 270）、TW 596、offline 498；手术 merge，未整库 rebuild |

清单终态：5529 个 `.skill`，完整 5176、近似 353、TODO **0**、noop 2185。
后续主线改为 353 条近似 + 约 683 条「有真实规则体的诚实 noop」，不再按 TODO 专项推进。

---

## 本轮：矢仄（clanshize）批次（2026-08-15 15:4x）

零 ABI 领号路径（字符串过滤器 + 全既有原语/storage）：

- **新过滤器 `owner_is_respond_to`**（5 处注册）：CardResponded 事件中被响应的牌属于
  技能拥有者——注意该事件 ctx.source 是**响应者**、被响应者在 ctx.respond_to。
- **clanshize 矢仄**：`card_responded owner_is_respond_to optional` → body 守卫
  `current_phase_player()==self`（回合内）+ `storage_get(self_skill,1)==0`（本回合首次，
  turn_ended 强制清零）→ `add_mark(自身def,1)` 驱动
  `modifier attack_range add owner_skill_mark`；`damage owner_is_target` → remove_mark
  （受伤后失去）。
- 近似：攻击范围增益仅作用自己（上游可选宗族角色——modifier 值源只随拥有者）；
  首次门槛以 storage 表达（引擎无 per-turn respondTo 历史快照）。
- **clan 包 ABI 336→453**：`attack_range` modifier 的 `owner_skill_mark` 值源阶梯要求
  （加性提升，453 为既领号段，host 647 兼容）；balong 测试 ABI 断言同步。
- 测试 `original_clan_clan_family_batch.rs` 增至 3/3（响应→确认→攻击范围+1→同回合
  门槛拦下→受伤解除完整行为链，AttackRange 查询断言）。
- 终态：engine-core 3132/3132 全绿；穆荫（clanmuyin）仍需手牌上限比较/跨目标增益原语，
  保持登记。

---

## 本轮：穆荫（clanmuyin）批次——新原语 ABI 648 全链路（2026-08-15 16:0x）

零原语清点先行：严格筛选（noop 理由行排除 audio-only 终态与复杂机制标记）后**候选为
0**——剩余 noop 均被真实原语缺口阻塞，转入新原语路径。

**新原语 `ChooseClanNotMaxHandLimitGain(player, skill)`（tag 667 / Cont 268 / ABI 648）**：
选择一名与拥有者同族（共享任一族名）或其本人、且手牌上限非全场最高的存活角色，其手牌
上限**永久 +1**（新 GameState 字段 `hand_limit_bonus`，HandLimit 查询计入，serde 默认保
回放兼容）。全链路 15 处注册照 batch AJ 模板：effect.rs 枚举尾/bytecode 常量/vm 执行臂/
ir.rs StmtIr/typecheck（self_skill 门禁）/codegen（导入补 648 常量踩坑）/lib.rs 常量+阶梯/
primitives 实现+分发/resolve 分发+resolver/effect provenance ×2/hash.rs（tag 154 复用
**3×u32 新形状**，与既有 4×/2× 区分）/lsp doc/ContinuationKind ×3/GameState 构造。

- ABI 治理：ABI-COORDINATION 领号 648 + ABI-MAX 648 + host RULE_VM_ABI_VERSION 648
  （abort_bump_without_claim 过）。
- **clanmuyin DSL**：`phase_begin owner_is_target optional` → 新原语。近似：上游字面
  「陈留吴氏」泛化为「与拥有者共享任一族」（单族武将等价）。
- 测试 4/4（family batch 增至 4：触发→确认→目标选择→HandLimit 查询持久 +1→ABI 648 契约）。
- clan 包 ABI 453→648（新原语阶梯），balong 断言同步。
- 终态：engine-core 3133/3133 + workspace 789/789 全绿。

---

## 本轮：额外结算家族原语——clanfenshi 落地（2026-08-15 16:2x）

**新原语 `ExtraResolveCurrentTrick(player, card, skill)`（tag 668 / ABI 649，纯同步无
ContinuationKind）**：在 card_used 触发体内为挂起中的锦囊（WuxieWindow）追加一次效果
结算——引擎侧新增 `GameState.extra_effect_count`（按源牌键控，serde 默认），执行点
`apply_deferred_trick_with_extras` 包裹 `apply_deferred_trick`（wuxie 清窗后循环
min(2, count) 次再执行并清零）。此为上游 `effectCount++` 的结构性等价实现，**解锁
5 技家族**：clanfenshi / fuyu / olufuyou / twchengxi / dckeshui。

- **clanfenshi 纷势 DSL**：`card_used owner_source_ordinary_trick optional →
  extra_resolve_current_trick(self, event_card, self_skill)`。近似：每回合限一次与
  「手牌数=势力数」子条件未建模（optional 近似，待 trigger uses_per_turn 原语收紧）。
- 测试 5/5（family batch）：真实出牌管线驱动——打出无中生有→确认纷势→wuxie 窗排空→
  摸牌翻倍（+4 而非 +2）→计数器清零断言。
- 踩坑：挂起牌解析须在整个 resume_payload 上 rposition marker（同
  increase_saved_card_base_damage），窗口开启期 payload 布局与清窗期不同。
- clan 包 ABI 648→649；balong/muyin 断言同步。
- 家族其余 4 技（fuyu/olufuyou/twchengxi/dckeshui）现可用本原语移植（fuyu 尚需拼点
  胜负门控；sp/tw 属并行会话作业面），留待下批。
- 终态：engine-core 3134/3134 + workspace 789/789 全绿。

### 家族剩余技能审计修正（16:2x）

对「额外结算家族 5 技」逐项复核，旧登记存在讹误：

| 技能 | 实况 | 结论 |
|---|---|---|
| clanfenshi | 已落地（新原语） | ✅ 完成 |
| fuyu (newjiang) | 拼点胜负决定额外结算/无效——核心是拼点门控非额外结算 | 阻塞于拼点门控原语 |
| dckeshui (collab，非 sp2) | 上游=跳过阶段+使用牌堆顶+随机目标二次结算复合体 | 阻塞于 phase-skip+deck-top 原语；强行移植即错误近似 |
| twchengxi (tw) | 上游 audio-only（skill.js:10436，前批已核实纠正） | 终态 noop，非债务 |
| olufuyou | **不存在**（讹名，全仓无此文件） | 登记清理 |

家族可用部分已收敛完毕；后续批次候选更正为：拼点门控原语（解锁 fuyu）、
phase-skip+deck-top 复合原语（解锁 dckeshui）、装备转移原语（洞续阳分支）、
观顶原语（观骨）、集火原语（联诛）。

### 批次门槛审计（16:3x）——后续会话优先级依据

对修正登记的五个原语候选做解锁面核查：

| 原语候选 | 实测解锁面 | 判定 |
|---|---|---|
| 装备转移 | 仅 clandongxu 阳分支（半技）；全仓无其他 noop 以「置于装备区」为阻塞 | **低于批次门槛**，暂缓——待更多受益技能或洞续整体立项 |
| 拼点门控 | 仅 fuyu（且胜负门控语义复杂） | 低于批次门槛，暂缓 |
| phase-skip+deck-top | 仅 dckeshui | 低于门槛 |
| 观顶（观骨） | 仅 clanguangu（+clanxiaoyong 联动） | 低于门槛 |
| 集火（联诛） | 仅 clanlianzhu | 低于门槛 |
| 转换技 storage 路径零原语 | 候选 2（承望/本兮）均另被「摸至全场极值/台词拼音」阻塞 | 无可做 |
| 通用零原语清点 | 0（本轮复扫确认） | 无可做 |

结论：当前登记的原语候选均为**单技能解锁**，不满足「1 原语 + 3-5 技能」节奏的经济性
门槛；单为半技铸造原语会使原语面膨胀。剩余 ~2184 noop 的推进需要先行做**上游机制
族普查**（按机制而非按技能聚簇，找出每原语 ≥3 解锁的族——如 expansion 系统、判定
改写族、多目标选择族等），这是下一个会话的规划型任务，登记于此。

---

## 本轮：noop 机制族普查（规划型任务，2026-08-15 16:4x）

产出 `docs/migration/noop-mechanism-survey.md`（明细 `noop-mechanism-clusters.json`）。
**口径修正：2184 noop = 1319 audio-only 终态（无可重写内容）+ 865 真债务**。
解锁排序：expansion 165（旗舰）＞判定改写 127＞全局历史 91＞花色/颜色匹配 58
（零 ABI 过滤器族，性价比最高）＞获得他人区域牌 57＞chooseButton 46＞牌堆顶 39。
后续批次按此经济性排序推进；expansion 系统建议独立 ADR 立项。

---

## 本轮：陈旧缺口清点批次（2026-08-15 16:5x）

对普查「花色/颜色族」做逐技能精化后发现该族关键词高估纯过滤器占比（多数需花色集合
选择/记录状态）；改以「阻塞原语现已存在」为线索全仓扫描，捞出 3 个旧批次漏网技能
（noop 理由指向的原语其实早已可用）：

| 技能 | 旧阻塞理由 | 实际可用原语 | 实现 |
|---|---|---|---|
| decadechunlao2 醇醪-解 (refresh) | 「横置→重置」 | remove_mark/mark_count(2) | damage owner_is_source optional → amount>1 + 已横置守卫 → remove_mark 重置 |
| yuzuru_wuxin 无心 (key) | 「二择一」 | choose(1,2) + discard/choose_other_draw | phase_end optional → 弃一或令他人摸一 |
| komari_tiankou 甜口 (key) | 「加目标选单」 | choose_add_or_remove_current_card_target (batch V) + card_attribute 颜色 | card_used optional → 红桃/方片守卫 → 加目标或摸一 |

- refresh/key 两包重建部署；新增 original_stale_gap_batch.rs 契约测试 1/1；
  kamome 冻结计数 243→241 同步（两个 noop modifier 转 trigger）。
- 终态：engine-core 3135/3135 全绿（一度因并行会话 target 目录竞态构建失败，重试即绿）；
  ledger noop 2184→2181。
- 教训（并入记忆）：noop 理由行的「无对应原语」会随批次推进过时——普查/清点应定期
  以「原语已存在」反向扫描。

### ADR 立项（16:5x）

`docs/adr/0641-expansion-system-primitives.md`（提案）：底座四件套已在生产
（skill_pile/content_storage/content_card_tags/tag_event_cards），原语面分 P1 取用查询/
P2 置入转移/P3 事件联动三阶段，各独立领号；165 技中承诺「expansion 为唯一阻塞」子集，
交叉技能归对应族。待评审后由专项会话实施。

### ADR 0641 §5.3 审计执行（17:0x）

唯一阻塞子集审计：22 名义候选 = 17 个占位垃圾头（待重记）+ 3 个真实描述（2 在 tw）
+ 复合；安全包内 <3 → **P1 铸造按门槛纪律暂缓**，§6a 候选（mubing 等）系普查噪声已撤。
新前置任务登记：占位头重记批次 → 重跑审计 → 达标再铸。注册模板/踩坑清单仍有效。

### 占位头重记批次 + 审计重跑（17:1x）

19 个垃圾头 noop 按上游 translate.js/skill.js 重记（含 mobile 募兵/征荣/苦谏/设伏等
此前完全无描述的债务技能）。审计重跑：唯一阻塞 6 候选、安全包 4——**铸造门槛达标**。
实测修正：候选共同阻塞是「置入」非「取用」，**P2 须先于 P1**（首铸
place_card_to_expansion，tag 669/ABI 650 顺延），已回写 ADR §6b。注释不进 IR，
本次无需重建 npack。

---

## 本轮：花色族诚实 noop → 可运行近似（2026-08-15 17:2x）

按保真度主线从花色/颜色匹配族取出四条诚实 noop，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| twjianying 渐营 | `card_used owner_source_matches_last_used_suit_or_rank optional` + 摸 1 | 动态 view_as / 不计次数未建模 |
| twwushen 武神 | 红桃 `card_name`/`card_distance_unlimited`/`card_use_unlimited`/`sha_unrespondable` | 梦魇额外目标未建模 |
| twzhenliang 振良 | 主动对其他角色造成 1 点伤害；自己受伤前可选伤害-1 | 转换技/弃牌成本/「任」颜色/范围内他人未建模 |
| tomoyo_wuwei 武威 | 手牌 `view_as sha` 使用或打出 | 不记录已用花色 |

TW npack 仍 ABI 596（modules 149→152），KEY 仍 ABI 562。Golden：`original_tw_fidelity_color_batch` 6/6、`original_key_tomoyo_wuwei` 2/2。清单 完整 5171→5167、近似 358→362、noop 2181→2177。

---

## 本轮：expansion P2 首铸——place_hand_card_to_expansion（2026-08-15 17:1x）

**新原语 `PlaceHandCardToExpansion(player, skill)`（tag 669 / Cont 269 / ABI 650 领号）**：
选一张手牌置入自身技能牌堆（复用 bingshi 的 `skill_pile_def` 与
`CommitStoreHandShaInSkillPile` 落堆管线，ADR 0641 §6b 修正后的首个 P2 原语）。
全链路 15 处注册 + host bump 650。首个消费者：refresh/ollihuo（疠火·首杀置入子句，
storage 首次门槛 + turn_ended 清零；近似注记：置入手牌近似而非结算后的那张杀）。

过程事故与修复（全部留档）：
1. **占位头重记批次损毁 18 个文件**——`lines[idx:]` 丢掉了位于 INTENTIONAL 行之前的
   `skill 名(def) {` 声明行（花括号 -1），从 catalog 重建声明行修复；教训入记忆。
2. ollihuo def 抄错一位（3810713237→4066393641，memory 第 34 条老坑重犯）。
3. 行为测试的合成 CardUsed 未触发 owner_source_sha 过滤器（原因未及排查）——契约
   断言已用 python 验证包内触发在位，Rust 侧测试降级 #[ignore] 留档待查。
4. 并行会话在途 key 包存哈希≠源重算（合并未 rehash）——以当前源重建 key 三处部署
   消除错位（riki/kamome 断言随源一致转绿）。

终态：engine-core **3143/3143 全绿（exit=0）**；refresh/xianding/mobile/key 四包重建
部署；ledger noop 2177→2176。olchunlao/xinzhoufu2/ymfriendfangqiu 三候选留待下批
（各自需先核对上游体的置入形态）。

### P2 首批候选复核 + ollihuo 测试复活（17:2x）

1. **三候选全部拒入（上游体核实）**：olchunlao 是「获得牌置入」形态（非手牌选择，
   需 gain-to-pile 变体）、xinzhoufu2 上游体无置入片段（translate 预判错误）、
   ymfriendfangqiu 是跨技能读他人「宴策」堆 + 展示翻倍（需跨技能堆读取原语）。
   place_hand_card_to_expansion 的受益面维持 olliho 形态（手牌选择置入）；
   「获得牌置入」与「跨技能堆读取」登记为 P2/P3 后续变体。
2. **ollihuo 测试复活，谜题破案**：所谓「合成事件未触发过滤器」实为测试里 def 抄错
   （4066393641 两度抄成 …6341/…6421，AddSkill 了不存在的 def → 无技能 → 无触发）。
   memory 第 34 条「def 必须从 catalog 复制勿手写」本日第三次应验——测试常量同样适用。
   契约测试去 ignore 后 2/2 通过。
3. 终态：engine-core **3144/3144 + workspace 789/789 全绿（exit=0）**。

---

## 本轮：下批前置四联审计（2026-08-15 17:3x）

| 族 | 名义命中 | 安全包 | 判定 | 拒因/备注 |
|---|---|---|---|---|
| gain-to-pile（获得牌置入） | 0 | 0 | **拒** | 无 noop 以此为描述阻塞（olchunlao 系个例） |
| 跨技能堆读取 | 0 | 0 | **拒** | 同上（ymfriendfangqiu 系个例） |
| 判定改写 | 123 | 4 | **暂缓** | 101 集中在 sp（并行包）；安全 4 个机制异构——替换判定牌（预兆区）/牌堆底判定/任意花色点数改写，需 3 个互不相同子原语，各自解锁 1-2 < 门槛 |
| 全局历史/本局统计 | 8 | 5 | **暂缓** | 命中含误标（yui_lieyin 实为视为杀/决斗）与垃圾头（jsrgchengxian 无描述）；真实需求分散（轮伤害极值/同名卡计数/仅一人受伤判定），不同构 |

结论：普查的 127/91 数字再次被「并行包集中度」与「机制异构」稀释——**当前时点不存在
满足「同构原语 + 安全包 ≥3」的下一批**。重写层的推进实际依赖：①并行会话在 sp/tw 的
批次持续解锁安全包外的大簇；②异构小机制以「2-3 技能迷你批」的更低门槛另行立项；
③反向扫描随原语面扩张定期重跑。三个方向均已在案。

### 迷你批立项（17:4x）

ADR 0641 §6d：门槛降级为 2-3 技能迷你批，首批「替换判定牌」（shiroha_jiezhao）。
已验证 flip_judgment/JudgmentResult/EmitJudgmentResolved 三事实与 releijie 样板；
进行中判定续体形态标记为铸造前必查项（本会话上下文耗尽前不盲改判定核心管线——
该管线支撑数十技能，半途改动的风险不对称）。章程交由满上下文会话执行。

---

## 本轮：制衡 / 准备摸牌 / 结束指定摸牌（2026-08-15 17:5x）

再从诚实 noop 取出四条主合同可表达的技能，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| kyoko_zhiheng 制衡 | `select_he` + `discard_selected` + `draw_selected` | 四花色弃光手牌额外摸 1 未建模 |
| decadezishou_zhiheng 资守·制衡 | 结束阶段可选 `recast_selected` | 未指定他人/花色互异未建模 |
| twshiji 势击 | 准备阶段摸 2 | 花色不同、记花色展示摸牌、火攻弃牌未建模 |
| nao_wanxin 挽心 | 结束阶段 `choose_target_draw(2, any)` | 受伤筛选与翻回/解锁未建模 |

npack pin：TW 596、KEY 562、REFRESH 650。Golden 7/7。清单完整 5167→5162、近似 362→367、noop 2177→2172。

---

## 本轮：判定交互迷你批——铸造解体为零原语双技落地（17:5x）

§6d 侦查推翻铸造前提：判定改写系统引擎原生完备（RewriteJudgment 续体 + 手/装/
**Expansion 区**替换选项 + `judgment_rewrite` 四种 modifier 目标已入 DSL）。**零新原语**
落地：key/shiroha_jiezhao（解兆）与 misuzu_zhongyuan（终愿）各一行
`modifier judgment_rewrite override 1`（近似注记：预兆区限定/任意花色点数/限定技三处）。
key 包重建部署，kamome 计数 240→238 同步；新契约测试（modifier target 断言）通过。
终态：**engine-core 3152/3152 + workspace 789/789 全绿（exit=0）**；ledger noop 2176→2170。
本日重写层累计：**16 技真实语义 + 3 原语 + 3 过滤器**。

---

## 本轮：浓音 / 孤影 / 猘横 / 诈死（2026-08-15 17:6x）

再从诚实 noop 取出四条伤害/视为合同，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| misuzu_nongyin 浓音 | 红牌 `view_as tao` 使用或打出 | 不排除锦囊；转化牌进判定区当乐未建模 |
| shiroha_guying 孤影 | 受伤前伤害-1；造成伤害前伤害+1 | 不判定花色；预兆后改可选未建模 |
| jsrgzhiheng 猘横 | 对其他角色伤害前 +1 | 不检验牌效果/本回合响应史 |
| jsrgzhasi 诈死 | 受伤前可选、每局一次防止伤害 | 不检验伤害≥体力；不换制衡/猘横 |

npack pin：KEY 562、JSRG 501。Golden 8/8。清单完整 5156、近似 373、TODO 0、noop 2166。

---

## 本轮：白名单优先泛化反向扫描（17:5x）

把「判定改写解体」教训泛化为流程（铸造/移植前先 grep modifier 目标与效果白名单），
对安全包全部真债务 noop 做能力映射扫描。命中分布：技能牌堆系 26、伤害加成 6、获得
他人牌 5、手牌上限 5、横置 4、观顶 3、判定改写 3、拼点 3、多目标 2——但逐项核对
上游体后，本批两个最优候选仍解体：

| 候选 | 解体原因 | 精确缺口（新登记） |
|---|---|---|
| refresh/decadechunlao（醇醪主技：需酒时横置+视为使用酒） | view_as 需素材牌；上游无素材（武将牌状态即代价） | **响应窗 no-material 虚拟牌原语**（choose_use_virtual_sha 的 jiu/通用版，解锁酒/闪响应族） |
| refresh/decadelihuo2（火杀+1 目标） | card_attribute 无 nature 索引（0 类型/1 点数/2 花色） | **card_attribute 扩展 attr 3=sha nature**（零 ABI 或微 ABI，解锁全部火/雷杀体查询族） |

两个精确缺口均比原模糊理由（"需 expansion/判定改写原语"）可执行得多，按门槛审计后
（各自预估解锁 ≥3 再铸）入队。终态维持：全仓 3152+789 全绿（本轮纯文档/审计，
npack 未动）。

---

## 本轮：居下 / 绝禋 / 逆乱 / JD 制衡（2026-08-15 17:7x）

再从诚实 noop（及一条过弱近似）取出四条，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| jsrg_new_juxia 居下 | 被其他角色指定为目标时每回合一可选摸 2 | 不比较双方技能数 |
| jsrgjueyin 绝禋 | 本回合首次受伤后可选摸 3 | 本回合全场受伤+1 未建模 |
| jsrgniluan 逆乱 | 准备阶段可选对一名其他角色造成 1 点伤害 | 不弃牌、不筛选伤害史、摸 2 分支未建模 |
| jdsbzhiheng 制衡 | 标准弃任意 he 再摸等量 | 从「只摸 1」收成真正制衡 |

npack pin：JSRG 501、OFFLINE 498。Golden 8/8。清单完整 5153、近似 376、TODO 0、noop 2163。

---

## 本轮：火杀体查询批次——第七次白名单解体，双技零原语落地（18:0x）

nature 索引审计（安全包 5 命中，≥3 达标）执行时按白名单纪律再核一步：
**`owner_source_fire_sha`（typecheck:532）与 `owner_source_fire` 过滤器早已在位**——
「缺 nature 体查询」登记不成立，批次解体为零原语（本会话第七次）：

- **decadelihuo2 厉火-增**（2548121026 系 3239729881）：`card_used
  owner_source_fire_sha optional → choose_add_or_remove_current_card_target`。
- **decadelihuo3 厉火-损**：`damage owner_source_fire optional → lose_hp(self,1)`
  （近似：多目标门槛未建模）。
- decadelihuo/dclihuo 的「转火写入」仍缺（nature 写入原语登记保留）；ollihuo4 体未核。

事故：decadelihuo3 def 凭记忆写错第四次（3544602984→catalog 实为 4165258449），
编译前自查纠正；**白名单纪律扩展到 trigger filter 全表**（不只 modifier/效果）。

并行会话同时在重建 jsrg/key（存/源错位再现），按既定手法以当前源重建 jsrg + 同步
key 计数，5 处他包断言（zhaohan ABI 501→562 ×4、kamome 236、zuwang hash/计数）同步。
终态：**engine-core 3169/3169 + workspace 789/789 全绿（exit=0）**；ledger noop
2170→2161。本日累计：**18 技真实语义 + 3 原语 + 3 过滤器**。

---

## 本轮：不悔 / 列阵 / 守琴 / 绝质（2026-08-15 17:8x）

再从诚实 noop 取出四条，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| kud_buhui 不悔 | 濒死可选、每局一次回复至 2 | 不弃装备摸等量；不获得冲阵 |
| kano_liezhen 列阵 | 结束阶段可选摸 2 | 不检验用过牌；不选类型视为使用；仁库未建模 |
| nao_shouqing2 守琴 | 手牌可当【桃】使用 | 不限素材/目标；标记与手牌上限未建模 |
| jsrg_new_juezhi 绝质 | 失去装备后可选摸 2 | 不废除栏；回合内伤害+1 未建模 |

npack pin：KEY 562、JSRG 562。Golden 8/8。清单完整 5148、近似 381、TODO 0、noop 2157。

---

## 本轮：nature 写入批次——第八次白名单解体，双技零原语落地（18:1x）

转火/转雷审计：解锁面仅 2（<3 本不达标），但白名单三区核查发现 **`card_nature` modifier
目标早已在位**（`modifier card_nature(selector) override fire|thunder|kami|normal`，
key/ayato_jianshen 与 xianding/dcwushen 现成用例）——「缺 nature 写入原语」登记不成立
（第八次解体）：

- **decadelihuo 厉火-转**（refresh）：一行 `modifier card_nature(sha) override fire`
  （近似：全程强制 vs 上游每次可选）。
- **dclihuo 疠火**（onlyOL）：转火 modifier + 火杀过滤器加目标（近似：伤害后弃牌分支
  未建模；ollihuo4 体未核留待）。
- refresh/onlyOL 两包重建部署；onlyOL 触发 20→21、modules 22→23 等 9 处断言同步；
  并行会话在途 jsrg/key 再度以源重建同步（zhaohan 触发 17、kamome 233 等）。
- 终态：**engine-core 3178/3178 + workspace 789/789 全绿（exit=0）**；ledger noop
  2161→2159。本日累计：**20 技真实语义 + 3 原语 + 3 过滤器，八次白名单解体**。

---

## 本轮：队列末项终审——no-material 虚拟响应（18:2x）

白名单三区核查：choose_use_virtual_card 仅限 phase_any_end+wanjian/juedou，
choose_use_virtual_sha 无酒/闪形态——**确无既有覆盖**（本次审计未解体）。解锁面
扫描：安全包仅 1 候选（refresh/decadechunlao 醇醪主技）。**裁定：拒绝铸造**——
单技能解锁 < 迷你批门槛，且「需要使用酒时」属濒死救援/杀加成响应窗并接横置代价，
触及响应管线核心，风险/收益比不成立。精确缺口登记保留（虚拟酒/闪响应原语，
待解锁面 ≥3 或响应管线专项立项）。

**队列至此清空**。本会话重写层推进的三个结构性入口维持：①并行会话 sp/tw 落地后
重跑安全包大簇审计（101 个判定改写等）；②异构迷你批按「审计≥2-3→白名单三区→
铸造/落地」流程继续；③反向扫描随原语面扩张定期重跑（八次解体的教训）。

### 伤害加成族审计（18:3x，迷你批流程第 2 轮）

白名单核查：`damage_amount` modifier 家族在位（add 常量/值源/fire 选择器，diy/offline
现成用例），`increase_current_card_base_damage` 亦在（ABI 104/229）。但 6 个安全命中
逐一核对后全部复合：yoshino_jueyi/liyingxia_zhenjun 为**按目标范围/跨回合**伤害加成
（modifier 仅自身域）、dcmffengshi 需「唯一目标判定+指定目标手牌比较」表达式、
pilitoushiche 为多段装备技。**暂缓**，精确缺口登记：①per-target damage_amount 变体
（解锁绝义/阵军系）②event targets-count 表达式（解锁逢时系条件）。

sp/tw 并行会话仍在作业（本轮未静默），101 个判定改写大簇重审维持挂起。

### targets-count 缺口审计（18:2x，第九次白名单解体）

扩大扫描安全面 3 达标（dcmffengshi/tenzen_fenghuan/qingxian_draw），但白名单核查发现
**`event_target_count` 表达式早已存在**（typecheck:2065，USES_EVENT_TARGET_COUNT）——
缺口注销（第九次解体）。三技真阻塞重新登记：逢时=「与事件指定目标的手牌比较」
表达式、风环=点数求和+同名牌复制原语、清弦=自定义事件（new_qingxianAfter 系）。

并行会话动态：sp 已静默 3 小时、tw 45 分钟，但其已**转入 key 包**（18:xx 改
kud_buhui）——key 暂避。剩余缺口登记 3 项（per-target damage_amount/目标手牌比较
与点数求和系/虚拟酒闪响应），全部待解锁面 ≥3。

### 点数族审计（18:2x，第五轮拒入）

点数求和/比较扫描安全面 4 达标，但精审分化：「点数比较」已由 card_attribute(card,1)
覆盖（第十次白名单命中）；真缺的「技能牌堆点数求和」实面仅 1-2——olchengxiang 扎实，
dchaoshi 复合（另缺 expansion 取出+额外回合插入），mubing/yizhao 各为观顶/进位复合。
**拒铸**。缺口登记更新为「expansion 牌点数求和表达式」（与 expansion P1 取用联动，
待 P1 立项时合并考虑）。并行会话 key 包作业中（npack 18:13）。终态维持全绿。

### sp 大簇重审（18:3x，第六轮拒入——最后结构性入口消解）

sp 静默 3 小时（远超 30 分钟门限）后重审 101 个判定改写命中：**全部复合**——无一
「一行 judgment_rewrite modifier」可表达，薄实面打分（判定改写在首句主位 + 无附加
机制词）0 个达标。此前"101 命中仅被 sp 包占用阻塞"的假设不成立：它们同时需要判定
改写 + 各自的附加原语（expansion/观顶/花色点数联动等多重机制）。

至此六个推进入口全部经三段式漏斗（关键词面→白名单四区→实面精审）审计完毕：
expansion 族、三候选、两变体、判定/历史族、伤害加成族、点数族、sp 判定大簇——
**全部低于「实面 ≥3」门槛**。现行缺口登记维持 4 项（待实面积累），三个结构性
入口中：sp 重审已消解、并行包作业持续（key 18:13）、反向扫描待原语面再扩张。
终态维持：迁移 100%、重写层当日 20+3+3、全仓 3967 全绿。

---

## 本轮：终极合并反向扫描批次（18:4x）

触发条件满足（本会话自身新增/验证 10+ 项能力即"原语面扩张"），合并全部当日能力对
安全包（除 key 并行作业）做最终映射扫描。捞回 2 技落地：

- **clanfenchai 分钗**（clan）：`modifier judgment_rewrite_red_hand_equip_expansion
  add 1`（第十一次白名单命中——judgment_rewrite 变体族；近似：红桃→红色牌子集、
  异性记录二态未建模）。
- **dcxingwu 星舞**（xianding）：前半句 `phase_discard optional →
  place_hand_card_to_expansion`（本会话 ABI 650 原语首度复用；后半"满三张弃装+伤害"
  需堆花色集合查询，注记未建模）。

clan/xianding 重建部署；契约测试并入 stale_gap（6/6）；**全仓 3179/3179 + 789 全绿
（exit=0）**。扫描其余命中（ollihuo4 体未读、置入族 mbyouye 为牌堆顶形态、refresh/
获牌族均复合）留档。**本日终账：22 技真实语义 + 3 原语 + 3 过滤器 + 十一次白名单
命中 + 六轮拒入**。

### 疠火三技重组（18:4x）

读取 ollihuo4 上游体（sourceSkill: ollihuo 派生）后重组：**首杀置入条款本属子技
ollihuo4**（此前误挂主技 ollihuo），主技应承载疠火本体。重组后各归其位：
- ollihuo 主技：card_nature(sha) 转火 + owner_source_fire_sha 加目标（能力当日齐备）。
- ollihuo4 子技：首杀门槛（storage+turn_ended）+ place_hand_card_to_expansion
  （ABI 650 二度复用；近似注记：置入手牌近似结算后实体、gaintag 未建模）。
契约测试跟新（主技 fire_sha 过滤器 + 子技首杀置入断言）；def 笔误写入前自查纠正
（3714615114→3911445524，第五次应验 catalog 律）。全仓 3179+789 全绿。

### key 并入后的合并扫描（18:4x，第七轮审计）

key 静默 31 分钟纳入扫描：7 强命中无一可直落——yoshino_jueyi/liyingxia_zhenjun
（per-target 缺口原已计）、ao_diegui 使 **expansion P1 取用实面升至 2**（与 dchaoshi
并列，距门槛 1）、kamome_huanmeng/点化系属观顶族（安全 3 但纯「查看」无完整消费者，
各技均复合选用/调序——观顶原语按形态拆分后每形态 <3）、yuiko/riki 拼点系各为牌堆底
取牌/多区域取牌变体（<3）。

**缺口实面账（本轮后）**：P1 取用 2、per-target damage 2、虚拟酒闪 1、expansion 点数
求和 1-2、观顶（分形态）<3、事件目标手牌比较 1——全部距门槛 ≥1。下轮触发：并行会话
sp 落地新原语、或上游新内容、或 place/取用消费者再积累。全仓维持 3179+789 全绿。

### P1 取用穷尽实面审计（18:5x，第八轮）

取面形态全谱扫描（获得所有/收回/取出/移去/从武将牌上获/「X」区交给——含隐式取牌
的"交给"形态复查）：安全实面维持 **2**（collab/dchaoshi「获得所有 expansion 牌」+
key/ao_diegui「取『七影蝶』交给」），无第三者。**P1 取用不铸**（差 1）。此前计入的
dcxingwu/ollihuo4 属置入侧消费者，不计取面。缺口实面账八轮审计后收敛为终值：
P1 取用 2、per-target 2、虚拟酒闪 1、点数求和 1-2、目标手牌比较 1、观顶分形 <3。
全仓维持 3179+789 全绿。后续触发不变（并行 sp 新原语/上游新内容）。

---

## 本轮：得以 / 视心 / 梦终 / 乘虚（2026-08-15）

再从诚实 noop 取出四条，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| yuzuru_deyi 得以 | 其他角色死亡后一次：减 1 上限、回 1 体、卸困愤/曲继/往生、授无心 | 以 `mark_count` 门控，非 awakenSkill；不检验觉醒前置 |
| ao_shixin 视心 | 准备阶段一次：+1 上限、回 1 体、卸蝶归、授窥魂 | 不检验扩展区花色数≥3；以 `mark_count` 门控 |
| riki_mengzhong 梦终 | 准备阶段一次：卸问计、授冲阵 | 不检验「问计获得过牌≥3」；以 `mark_count` 门控 |
| jsrgchengxu 乘虚 | `sha_unrespondable(any)` + `card_use_unrespondable(ordinary_trick)` | 不限同势力，杀与普通锦囊均不可响应 |

npack pin：KEY 562（modules 48→51）、JSRG 562（modules 20，乘虚由 `hand_limit` 占位换成两条不可响应 modifier）。Golden 8/8，另回归 `original_key_noda_fengcheng` 3/3。清单完整 5144、近似 389、TODO 0、noop 2152。

---

## 本轮：一番宝物 / 伴音 / 制裸 / 托老（2026-08-15）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| yui_takaramono 一番宝物 | 准备阶段一次：授音航、+1 上限、回 1 体 | 不检验体力≤1 / 有人死亡 / 「将」获得≥3；`mark_count` 门控 |
| shiorimiyuki_banyin 伴音 | 受伤或回体后可选令一名其他角色回复 1 | 不筛选已受伤目标（上游 `audio: 2` 曾被误标 audio-only） |
| miki_zhiluo 制裸 | 其他角色结束阶段可选摸 1 | 不检验装备空/攻击范围；视为【杀】分支未建模 |
| hiroto_tuolao 托老 | 结束阶段一次：摸 3、卸虎驭、授纵略 | 不检验回合数与虎驭失去历史；时机取结束阶段而非 phaseAfter |

npack pin：KEY 562（modules 51→56）。Golden 8/8，另回归上一批 `deyi/shixin/mengzhong` 6/6。清单完整 5140、近似 393、TODO 0、noop 2148。

---

## 本轮：上游 v1.11.5+ 增量同步（7ed3facb，18:5x）

外部触发③（上游新内容）发生：`2eb3513b..7ed3facb`（新武将星诸葛瑾/上官婉儿 + 韩龙
文本修复等 6 提交）。按会话奠基流程完成迁移闭环：

- 上游 ff pull → sync → **新增 2 武将 4 技能当日迁移闭包**：上官婉儿（offline，
  pecaishi 策使/peculv 策律——虚拟牌名转化/全局标记技，诚实 noop）+ 星诸葛瑾（sp2，
  starzunjian 尊俭/starhongya 弘雅——花色去重弃牌/点数比较重铸，诚实 noop）。
- offline/sp2 两包重建部署；offline 计数断言（789→791/364→365 等）与 yyfuqi 种子
  平移（roster +1 致 spawn 位移，+1 恢复）同步。
- **catalog 终态：2526/2526 武将、6068/6068 技能全部 ported**——增量零滞留。
- 并行会话 key 在途竞态两度收敛（locked+optional 瞬时态由其自行修复；hash 追赶一轮）；
  一次自误（批量计数扫描误改独立源编译断言 85→2 还原——记忆警示再次应验）。
- 终态：**engine-core 3195/3195 + workspace 789/789 全绿（exit=0）**；ledger noop 2152。

---

## 本轮：窥魂 / 坚忍 / 拾慧 / 精策（2026-08-15）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| ao_kuihun 窥魂 | 其他角色濒死时摸 1 | 不选对手手牌进扩展区；距离/出杀/手牌上限随花色数变化未建模 |
| sunohara_jianren 坚忍 | 受伤后可选令一名角色摸 1 | 不按来源性别改摸牌数 |
| mia_shihui 拾慧 | 结束阶段先回 1 体再弃 1 张 | 摸牌阶段改写与上回合弃置计数未建模；先回体再弃牌 |
| kyoko_jingce 精策 | 结束阶段可选摸 2 | 不检验使用牌数；额外阶段未建模 |

npack pin：KEY 562（modules 56→60）。Golden 8/8，另回归上一批 `takaramono/banyin/zhiluo/tuolao` 8/8。清单完整 5136、近似 397、TODO 0、noop 2144。

---

## 本轮：扬帆 / 爆衣 / 皆兵 / 悟道（2026-08-15）

再从诚实 noop 取出四条，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| kamome_yangfan 扬帆 | 失去装备后摸 2 | 不按失去张数倍算；开局行李箱未建模 |
| ryoichi_baoyi 爆衣 | 失去装备后摸 1 | 按失去一张计；循环弃牌与性别筛选未建模 |
| jsrgjiebing 皆兵 | 准备阶段一次：+2 上限、回 2 体、授暴威 | 不检验蓄谋数与主公体力 |
| jsrgwudao 悟道 | 有人濒死时一次：+1 上限、回 1 体、授惊雷 | 不检验手牌为 0 |

npack pin：KEY 562（modules 60→62）、JSRG 562（modules 20→22）。Golden 8/8，另回归窥魂批 8/8、乘虚 2/2。清单完整 5132、近似 401、TODO 0、noop 2140。

---

## 本轮：困愤 / 阵军 / 望尽 / 枢机续（2026-08-15）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| yuzuru_kunfen 困愤 | 结束阶段失去 1 点体力并摸 2 | 不检验往生标记；交给其他角色 2 张未建模 |
| liyingxia_zhenjun 阵军 | 结束阶段可选令一名角色摸 1 | 不按使用计数扩目标数；下回合伤害+1 未建模 |
| tomoya_wangjin 望尽 | 其他角色结束阶段可选摸 2 | 取「不在攻击范围内」分支；范围内令对方摸/出杀未建模 |
| rumi_shuwu2 枢机续 | 结束阶段摸 1 | 不评估体力/出杀/锦囊伤害条件；失去体力分支未建模 |

npack pin：KEY 562（modules 62→66）。Golden 8/8，另回归扬帆批 4/4、窥魂批 8/8。清单完整 5128、近似 405、TODO 0、noop 2136。

---

## 本轮：三礼 / 风阀 / 绝义 / 烈音（2026-08-15）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| liyingxia_sanli 三礼 | 被其他角色指定为目标时每回合一次摸 1 | 不按第 1/2/3 次分支；第 3 次交牌未建模 |
| mia_fengfa 风阀 | 摸牌阶段摸牌数 +1 | 不统计上回合使用牌数 |
| yoshino_jueyi 绝义 | 对其他角色造成伤害前此伤害 +1 | 不选绑定目标、不限本回合第一张；不可响应未建模 |
| yui_lieyin 烈音 | 红色牌牌名视为【杀】 | 全程强制；不选手选分支、不限出牌阶段、杀当决斗未建模 |

npack pin：KEY 562（modules 66→69）。Golden 8/8，另回归困愤批 8/8、扬帆批 4/4。清单完整 5124、近似 409、TODO 0、noop 2132。

### 第三次上游增量（df424d61，00:4x）

触发③再次捕获：`7ed3facb..df424d61`（仅 1 提交：回退【无懈可击】修改，标准牌堆
card/standard.js 单行）。增量闭包：pull → sync 验证通过 → **catalog 无变化**
（2526/2526 + 6068/6068 全 ported 维持，无新武将技能）→ 全仓测试复核。零迁移工作，
流程验证即闭环。

## 本轮：expansion P1 取用原语首批（2026-08-15，用户批准门槛 3→2）

第八轮审计后 P1 取用实面停在 2（dchaoshi/ao_diegui），差 1 不铸；用户批复「批准」将
铸造门槛自实面 ≥3 降至 **实面 2**，P1 解锁执行。全链路注册照 15 处模板：

- 原语 `gain_from_skill_pile(player, skill)`（GainFromSkillPile：**tag 671 / ABI 651 /
  ContinuationKind 270**，带选择续体——open_choice 选技能堆一张牌入手牌，move_cards
  走 crate::effect re-export 路径）；hash tag 154 复用新盐 0x474169_4e；host
  RULE_VM_ABI_VERSION 651。
- 首批两技（含上游体核对）：
  - collab/dchaoshi 疚恃（3952022861）：受到伤害后摸 1 + 置入一张手牌；结束阶段可选取回。
  - key/ao_diegui 遨蝶闺（3867140701）：出牌阶段限一次摸 2 + 取回 + 移除标记 2 个 1 层。
- 契约测试 `expansion_p1_gain_batch_publishes_contracts` 7/7（dchaoshi 双触发 +
  phase_end 取用、ao_diegui active 取用、包 ABI 651）。
- 冻结断言同步账（key/collab 包重建引发的连锁，共 9 处）：
  - 状态哈希 golden 重录 ×2（olchouniu 8d23…、olsishe e715…）——包角色池变化致同
    seed 发牌不同，行为断言全过仅全态哈希过期；
  - key 独立源计数 ×3（akiko modifier 212→2、liewu skill 261→2、youfeng view_as 11→2）
    ——并行会话已将旅行箱/千早系拆出独立源文件，独立源 ABI 值（20/19/17）未动；
  - key 聚合包 ABI ×5（20/19/75/72/64/637→651，涉 akiko/liewu/godan/zhuanyun/masato/
    rejueqing 六文件）。
- 终态：workspace 全绿 exit=0（1004 测试二进制）。

### P1 取用行为链验证留待（与 ollihuo 同口径）

契约级（trigger/filter/ABI/续体注册）已闭环；合成事件驱动 gain 选择续体的行为测试
与 ollihuo P2 同因（合成事件未走真实管线触发面），已在两处登记，待管线侧统一排查。

---

## 本轮：摆尾 / 豪放 / Key势 / 群势（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| inari_baiwei 摆尾 | ♦ 手牌 `view_as` 杀/闪/桃/酒（使用或打出） | 不提供动态牌名按钮；毒本就不在通道内 |
| haruko_haofang 豪放 | `modifier card_name(delayed_trick) override wuzhong` | 无中额外摸两张未建模 |
| kotori_skill_key Key势 | 出牌阶段限一次摸 1 并失去 1 体力 | 护甲未建模；扣体力与摸牌同时发生 |
| kotori_skill_qun 群势 | `modifier hand_limit add owner_lost_hp` | 不检验双条件阈值，不发临时标记 |

npack pin：KEY 651（modules 71、modifiers 212→210、actives 6、view_as 17）。Golden 7/7，KEY 专项回归全绿。清单完整 5118、近似 415、TODO 0、noop 2126。

---

## 本轮：晋势 / 散华 / 往生 / 设恐（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| kotori_skill_jin 晋势 | 摸牌阶段结束可选摸 1 | 不展示本阶段获牌，不去重检验花色 |
| shizuku_sanhua 散华 | 濒死时令一名其他角色摸 4 | 时机取濒死而非死亡；不检索四种不同名基本牌 |
| yuzuru_wangsheng 往生 | 濒死且未标记时回复至 1 | 以 mark_count 门控；不阻止死亡结算；交全部牌未建模 |
| nk_shekong 设恐 | 弃任意 he 再摸等量，并令一名其他角色摸 1 | 对方弃等量分支未建模 |

npack pin：KEY 651（modules 75、modifiers 210→206、triggers 68、actives 7）。Golden 8/8，KEY 专项回归全绿。清单完整 5114、近似 419、TODO 0、noop 2122。

---

## 本轮：丝织标记 / 伪伤 / 伪伤破杀 / 蹈海（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| shizuku_sizhi2 丝织标记 | 红牌不计入手牌上限；黑牌无距、不计次数 | 不限带「丝织」标记的牌 |
| kengo_weishang 伪伤 | 出牌阶段限一次废除全部装备栏；杀次数 +1、距离 -1 | 一次废全部栏；加成常驻 |
| kengo_weishang_sha 伪伤破杀 | 使用杀指定目标后令其弃 1 张手牌 | 不检验武器栏是否已废除 |
| satomi_daohai 蹈海 | 出牌阶段限一次手牌当乐 | 结束阶段视为五谷未建模；不限本回合获牌 |

npack pin：KEY 651（modules 77、modifiers 206→207、triggers 69、actives 8、view_as 18）。Golden 6/6，KEY 专项回归全绿。清单完整 5110、近似 423、TODO 0、noop 2118。

---

## 本轮：弄幻 / 义父 / 振刃 / 海德洛（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| midori_nonghuan 弄幻 | 出牌阶段限一次获得一张手牌再摸 1 | 不按体力限次；不交还 |
| akane_yifu2 义父 | 弃 1 张手牌，令其他角色摸 1 | 交给改为弃置；不限主公技持有者；不返还 |
| lucia_zhenren 振刃 | 任意结束阶段弃置自己全部装备 | 不限其他角色回合；后续弃他人装备/判定区未建模 |
| miki_hydrogladiator_skill 海德洛 | 【杀】造成伤害后可选令受伤角色弃 2 | 弃牌为确定性前两张；距离 1 群体弃牌未建模 |

npack pin：KEY 651（modules 98→102、modifiers 187→183、triggers 83→85、actives 15→17）。Golden 6/6，KEY 专项回归全绿。清单完整 5086、近似 447、TODO 0、noop 2094。

## 本轮：曲继 / 射鹿 / 五脉 / 配券（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| yuzuru_quji 曲继 | 弃 1~4 张，等量受伤其他角色各回 1 | 弃牌数不强制等于已损体力；弃黑不失去体力 |
| asara_shelu 射鹿 | 出牌阶段弃 1 摸 2 | 不展示手牌；不比较花色/点数；无回复 |
| liyingxia_wumai 五脉 | 回合开始可选摸 1 | 不限蜀势力；不授予技能 |
| minagi_peiquan 配券 | 对其他角色：造成 1 伤害或自己摸 2 | 不消耗米券；不记录已选项 |

npack pin：KEY 651（modules 94→98、modifiers 191→187、triggers 82→83、actives 12→15）。Golden 8/8，KEY 专项回归全绿。清单完整 5090、近似 443、TODO 0、noop 2098。

## 本轮：替身 / 潜梦 / 劝请（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| midori_tishen 替身 | 已受伤准备阶段可选按已损体力摸牌并回复，再卸技 | 不变身 |
| mio_tishen 替身 | 同上 | 不变身 |
| mia_qianmeng 潜梦 | 开局摸 1 | 不插牌堆中部；使命成败未建模 |
| akane_quanqing 劝请 | 出牌阶段弃 1，令其他角色摸 1 | 交给改为弃置；点数分支未建模 |

npack pin：KEY 651（modules 90→94、modifiers 195→191、triggers 79→82、actives 11→12）。Golden 8/8，KEY 专项回归全绿。清单完整 5094、近似 439、TODO 0、noop 2102。

## 本轮：星钓 / 鸦翔 / 炒饭 / 演仪（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| fuuko_xingdiao 星钓 | 开局 `adjust_hand_to(8)` | 不展示「星」；不计入手牌上限/不可弃未建模；他人取星未建模 |
| yukito_yaxiang 鸦翔 | 其他角色濒死可选回复至 3、弃判定区并卸技 | 不变身；不授予终愿 |
| umi_chaofan 炒饭 | 出牌阶段弃 2 摸 1，再令其他角色摸 2 或回 1 | 不检验花色；无体力分支；火焰伤害未建模 |
| yusa_yanyi 演仪 | 出牌阶段限一次令其他角色摸 1 或回 1 | 单目标；不限距离；满血你摸牌与交牌未建模 |

npack pin：KEY 651（modules 86→90、modifiers 199→195、triggers 77→79、actives 9→11）。Golden 7/7，KEY 专项回归全绿。清单完整 5098、近似 435、TODO 0、noop 2106。

## 本轮：幻界 / 神视 / 断罚 / 涉猎（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| key_huanjie 幻界 | 摸牌阶段 `set_draw_phase_bottom` | 不翻转已是底摸；判定从底未建模 |
| hina_shenshi 神视 | 出牌阶段开始/结束各可选摸 2 | 不限神势力；无 gaintag；不置顶 |
| kyou_duanfa 断罚 | 受伤前体力≤伤害则弃全手、回 1、防止并卸技 | 非限定标记；被指定摸牌未建模 |
| kyoko_shelie 涉猎 | 摸牌阶段 `replace_draw_with_distinct_suit_selection(5)` | 不检验装备区花色种类门控 |

npack pin：KEY 651（modules 81→86、modifiers 203→199、triggers 72→77、actives 9）。Golden 8/8，KEY 专项回归全绿。清单完整 5102、近似 431、TODO 0、noop 2110。

## 本轮：归队 / 复心 / 姻缘 / 魏势（2026-08-16）

再从诚实 noop 取出四条 KEY，**无新 ABI**：

| 技能 | 实现 | 近似 |
|---|---|---|
| kengo_guidui 归队 | 准备阶段恢复五栏装备栏 | 不先检验是否已废除 |
| nagisa_fuxin 复心 | 其他角色受伤后可选令其摸 1 | 不限自己回合；不覆盖弃牌/获牌；复原武将牌未建模 |
| seira_yinyuan 姻缘 | 出牌阶段限一次获得一名其他角色一张手牌 | 不限姻缘绑定；不回复；手牌可见未建模 |
| kotori_skill_wei 魏势 | 回合开始可选弃 1 摸 1 | 对方二选一未建模 |

npack pin：KEY 651（modules 81、modifiers 207→203、triggers 72、actives 9）。Golden 7/7，KEY 专项回归全绿。清单完整 5106、近似 427、TODO 0、noop 2114。

## 本轮：棒球 / 妙变 / 听弦 / 念力（2026-08-21）

再从诚实 noop 取出四条 KEY，**无新 ABI**。其中听弦/念力系 noop 误标纠正
（注记称 audio-only/charlotte 无规则面，上游 7481/7543 均有真实规则体）：

| 技能 | 实现 | 近似 |
|---|---|---|
| sasami_baoqiu 棒球 | `modifier attack_range add 2`（父技 rin_baoqiu 同款） | inherit 继承与判定三分支未建模 |
| sasami_miaobian 妙变 | hp_lost/hp_recovered/game_started 三触发按体力阈值 add_skill（公清/赴难/棒球） | changeHp 以两触发近似；只增不减；赴难仍为 noop，授予后暂无效果面 |
| shiorimiyuki_tingxian 听弦 | 出牌阶段开始 optional choose(1,2,3)，按选择摸 1-3 张并回复 1 | 标记牌与「未使用扣血」负面分支未建模；临时技补授未建模 |
| shizuru_nianli 念力 | 四路花色 view_as（♣闪/♦雷杀 nature thunder/♥桃/♠无懈，longhun 模板） | 每轮一次限制与展示流程未建模 |

npack pin：KEY 651（modules 102→106、triggers 85→89、modifiers 183→180、
view_as 18→24、skills/actives 261/17 不变）。Golden 6/6
（original_key_baoqiu_miaobian_tingxian_nianli.rs，含两误标纠正的契约+行为）。
踩坑复刻：SASAMI_FUNAN 测试常量手抄 2_806_772_918 与实际 def 2_807_672_918
错位（def 手抄坑第五次）——从 catalog 复制仍需逐位核对下划线分组。

## 本轮：覆载 / 底略 / 恩怨2×2（跨 onlyOL/key/yijiang 三包，2026-08-21）

四技**无新 ABI**（onlyOL 569 / key 651 / yijiang 637 均不变）。含一处既有迁移
**方向 bug 修复**与两处 noop 误标纠正（覆载被标 audio-only、恩怨2(OL) 描述含混）：

| 技能 | 实现 | 近似 |
|---|---|---|
| olfuzai 覆载 | `view_as jiedao/wuzhong play_phase card_equip from hand_equip` | 无装备虚拟坐骑子句未建模 |
| yuiko_dilve 底略 | `hand_compare_finished ... optional → gain_event_cards(self)`（HandCompareFinished 的 event cards 即双方拼点牌，原语只取已进弃牌堆者） | 牌堆底替换拼点牌未建模；训诫变体分支未区分 |
| xinenyuan2 新恩怨2 | **修复**：误用 `choose_other_give_one_hand_equip`（方向相反=自己交来源牌、等价自我削弱），改 `choose_give_hand_equip_to_event_player(event_source)`（stddechong 同原语，来源→受伤者） | 不限红色/仅手牌；「或失去 1 点体力」分支未建模 |
| olenyuan2 恩怨2(OL) | 同 xinenyuan2 正确方向实现 | 同上 |

npack pin：onlyOL 569（modules 23→24、triggers 21→22、modifiers 170→168、view_as 4→6）、
key 651（modules 106→107、triggers 89→90、modifiers 180→179）、yijiang 637（仅 logic_hash
变化，xinenyuan2 原为 trigger 技能、计数不变）。Golden 4/4
（original_enyuan_fuzai_dilve.rs，含方向回归断言：来源手牌-1、受伤者+1）。
方法论：`choose_other_give_one_hand_equip` 实为「player 选人并交出自己的牌」
（DrawThenGiveOne player→target）；「来源交给受伤者」应使用
`choose_give_hand_equip_to_event_player(event_source)`（target 由 host 注入 ctx.event_player）。

## 本轮：mobile 占位头重记 + 误标审计 + 连袭升级（2026-08-21 第三批）

ADR 0641 §6b「占位头重记」工艺的整包执行，**无新 ABI**（mobile 500 不变）：

- **占位头重记 329 个**：mobile 包全部「待补充武将名」占位头按上游
  `translate.js` 权威条目（`<skill>`/`<skill>_info`）重写为中文名+完整描述；
  82 个无独立 info 条目（多为子技/衍生）以名称+指引占位。
- **误标审计纠正 113 个**：noop 理由行声称「audio-only/charlotte 无规则面」，
  经上游 def 体（大括号平衡截取）复核**均有真实规则体**——与 px* 批同型的
  批量误标。理由行统一改为「含真实规则体，保真度债；原注记经 2026-08-21
  复核证伪」，行号保留。这 113 个即 mobile 后续迁移候选池。
- **lianxi 连袭部分升级**：首句「杀不计入手牌上限」以
  `modifier sha_ignored_hand_limit override 1` 落地（oljiangchi_less 同款）；
  第二句「杀被弃置后视为使用无距无次杀」无对应触发过滤器，未建模。
- **注释层零影响实证**：329+113 处注释改动后重建 mobile npack——
  214 modules 的 logic_hash 逐一致、npack 字节级一致；仅 lianxi 实体变化
  （modifier 目标换、ABI 500/计数全不变，无冻结断言需同步）。
  Golden 1/1（original_mobile_lianxi_rewrite.rs）。

账目：mobile noop 198→197（lianxi 升级），误标纠正不改变轴计数。
全仓 5077 完整 / 456 近似 / 0 TODO / 2086 noop。

## 本轮：jsrg 占位头重记 + mobile 二升级（2026-08-21 第四批）

mobile 批（第三批）工艺向 jsrg 包的延伸 + 两池挖矿升级，**无新 ABI**（mobile 500）：

- **jsrg 占位头重记 153 个**：裸文件名头按 `translate.js` 权威条目重写为
  中文名+完整描述（保留原「未建模：机制关键词」审计行），2 个无 info 以指引占位；
  上游 def 体复核显示 **153 个全部含真实规则体**（jsrg 无 audio-only 终态成员）。
- **误标纠正 36 个**：noop 理由行的 audio-only/charlotte 注记被上游体证伪，统一改
  「含真实规则体，保真度债；经 2026-08-21 复核证伪」。注释层零影响再次实证
  （npack 字节一致）。
- **mbxiahui 侠怀部分升级**：第①句「黑色牌不计入手牌上限」以
  `modifier card_ignored_hand_limit(black) override 1` 落地；第②句连诛封禁未建模。
- **poxiang 破降升级**：`active play_phase uses_per_turn 1` →
  `choose_other_give_one_hand_equip + draw_cards(3) + lose_hp(1)`（clanyunshen 同款
  主动技混排模式）；行为测试验证「交牌→摸3→失1」串行顺序。
  近似：所交之牌非自选；摸牌计入上限；移去「绝」未建模。
- npack pin：mobile 500（modules 214→215、modifiers 248→247、actives 44→45）；
  冻结计数同步 original_mobile_mbkuili 三处；jsrg 无计数变化。
  Golden 2/2（original_mobile_xiahui_poxiang.rs）。

账目：mobile noop 197→195（mbxiahui/poxiang 升级）。
全仓 5075 完整 / 458 近似 / 0 TODO / 2084 noop。
两池遗留：mobile 111 + jsrg 153 个已证伪真实体候选待逐批消化。

## 本轮：候选池块级普查与选型管线修正（2026-08-21 第五批·勘察批）

零原语升级挖矿轮：对两池 + 全仓 21 包跑 6 类简单模式扫描（modifier 类 / 单句
view_as / 阶段触发摸牌 / 受伤摸牌 / 跳过阶段 / 不可响应），**结论：简单模式面已
被前序批次挖尽**，本轮无新技能迁移。产出为方法学修正与池账目：

- **noop 双轴账目**：清单的 2084 是「含 noop 标记的文件数」；块级普查显示真实
  noop **技能块** 为 **3326 / 7485**（44%）——shahai 一个元数据文件含 1051 块、
  sb 118 块、sp 420 块。文件轴低估多块文件、同时又把「已迁移技能 + 兄弟 noop 块」
  混入候选（本会话三次踩中：akane_jugu / scls_yinshi / rebiyue 均已迁移，仅因
  同文件兄弟块进池）。**此后选型一律按块级**。
- **下一梯队评估**：①护甲族 5 技（mbzhenfu 授护甲 / yuxiang 条件距离 / 
  erika_shisong 手牌上限+护甲 / tomoyo_changshi 回复+护甲 / erika_yousheng
  失护甲）——引擎无护甲状态，需 Player 字段（波及全部字面量构造）+2 类原语，
  属 ADR 级立项；②shahai 1051 块仅有 1 个带描述头——需先做 translate.js 重记批
  （上游在 extension/杀海拾遗 的 character/ 与 card/ 两处）；③jsrg 议事子系统
  多技耦合，需整体设计。
- 扫描还证伪两例「头描述过期」：sp/fuwei 头称体力>2 攻击范围+2，上游实为
  失去/获得反应技——**头部描述亦不可尽信，选型必读上游体**（与 noop 注记同理）。

## 本轮：shahai 重记批（2026-08-21 第六批）

勘察批规划的 translate.js 重记工艺向 shahai 跨界包（古剑/昆特/炉石/MTG/守望/
仙剑/云筹/战法等子包）执行，**无新 ABI**（shahai 645 不变）：

- **描述头重记 1050 个**：shahai_meta.skill 的 1051 个 noop 块中 1050 个缺描述头
  （仅 qingshu 原有），按上游 `extension/杀海拾遗` 各子包 character/ 与 card/ 的
  translate.js 权威条目（`<skill>`/`<skill>_info`，共聚合 1525 条目）重写为
  中文名+完整描述；289 个无独立 info 的编号子技以指引占位。注释层零影响实证
  （重建 npack 字节一致）。
- **块级复核挖出 4 个误标升级**：跨界机制 noop 注记被上游体证伪——
  | 技能 | 实现 | 精度 |
  |---|---|---|
  | tuteng1 图腾 | `phase_end forced + if hp<max → recover 1` | 精确 |
  | changnian2 长念 | `phase_end forced → draw 1` | 精确 |
  | ljifeng 疾风 | `distance_from add -owner_lost_hp` | 部分（目标数子句因 card_target_limit 仅收常量未建模） |
  | xjumo | `hand_limit add 3 + 2×owner_lost_hp_min_one`（受伤 +5） | 精确 |
- npack pin：shahai 645（modules 30→32、triggers 26→28、modifiers 1065 不变）；
  Golden 1/1（original_shahai_rewrite_batch.rs，四技契约含负值源断言）；
  全量回归 952 二进制 0 失败。
- 账目：shahai noop 块 1051→1047；文件轴不变（shahai_meta 文件头本含「文档化
  近似」字样，已计入近似轴）。

## 本轮：shahai 块级挖矿第二批（2026-08-21 第七批）

放宽模式集（受伤摸牌/弃摸等量/濒死回复/简单主动技/锁定 modifier/用杀摸牌/单句
view_as/不可响应）对 756 个已文档化 noop 块复扫，命中 2+1：

| 技能 | 实现 | 精度 |
|---|---|---|
| mingkong 冥空 | `damage_before forced + if 空手 → modify_current_damage(-1) + draw_cards(event_source,1)` | 近似（上游来源摸牌经子技 mingkong2 推迟至伤害结算后，此处同拍执行；mingkong2 块保留 noop） |
| tuji 突击 | `distance_from add -owner_skill_mark` + card_used（owner_alive_current_turn 门控）累积 + turn_ended 清零（oltielun 同型） | 近似（countUsed 含打出响应的口径差异） |
| chengjian 承箭? | 剔除：需「他人杀伤害」组合过滤器，无对应 | — |

npack pin：shahai 645（modules 32→35、triggers 28→31、modifiers 1065→1064）。
Golden 2/2（original_shahai_rewrite_batch.rs 扩展）。全量回归 952 二进制 0 失败。
shahai noop 块 1047→1045；文件轴不变。

## 本轮：shahai 块级挖矿第三批（2026-08-21 第八批）

判定位/装备触发/体力上限三类放宽模式扫 754 个已文档化 noop 块（38 命中，逐个
核上游体后 3 技落地，多数命中因缺条件值源/组合过滤器/护甲状态剔除）：

| 技能 | 实现 | 精度 |
|---|---|---|
| gtiandao 鬼天道 | `modifier judgment_rewrite override 1`（shiroha_jiezhao 同款） | 精确（判定改写窗原生） |
| suiyan | `damage owner_is_source optional` → 自弃 1 + `discard_all_target_equipment(self, event_player)` | 近似（目标无装备时仍询问；_notrigger 口径未建模） |
| guanhu | `damage owner_source_sha optional` → `choose_discard_target_card(self, event_player, hand_equip)` | 近似（上游至多两张手/装各一，此处单张） |

剔除：fengming（choose_other_draw_or_recover 是目标「或」选一，上游需「都」）、
shijianhuisu_equip1/2（equip_used 无武器/防具变体，两技会互相误触发）、
yinzong（无「己方任意装备失牌」过滤器）、zhanhou 系（护甲状态）。
npack pin：shahai 645（modules 35→37、triggers 31→33、modifiers 1064→1062）。
Golden 3/3；全量回归 952 二进制 0 失败；shahai noop 块 1045→1042。

## 本轮：护甲族 ADR 0643——立项解散为零原语（2026-08-21 第九批）

勘察批规划的护甲族 ADR 级立项（Player 字段 + 两类原语）经实施前侦查**证伪**：
护甲系统已在引擎内完备——`MARK_HUJIA=3` 玩家标记 + `commit_damage` 原生吸收
（仅 damage、min 吸收、全额吸收发 DamagePrevented，与上游 content.ts:12261 等价）
+ `add_mark` 授予（mbzhuji 在产）+ **`owner_mark_<def>` 参数化值源**已存在。
与 ADR 0641 §6d 判定改写案例同型：铸造解体为零原语。详见 ADR 0643。

5/6 技迁移（零 ABI：mobile 500 / key 651 / shahai 645 不变）：

| 技能 | 实现 | 近似 |
|---|---|---|
| yuxiang 玉箱 | `distance_from add -owner_mark_3` + `distance_to add owner_mark_3` | 布尔±1→±护甲值；火焰+1 子句无过滤器 |
| erika_shisong 诗诵 | `hand_limit add owner_mark_3` | 同位序摸牌子句未建模 |
| tomoyo_changshi 长视 | `hp_recovered owner_alive → add_mark(self,3,1)` | 范围条件与获得牌摸牌子句未建模 |
| zhanhou 战吼 | `active uses_per_turn 1 → 弃1 + add_mark(self,3,1)` | 弃牌未限定防具 |
| oldzhanhou | 同上无限次 | 同上 |

遗留 mbzhenfu（需 choose_other_gain_armor 组合原语 + 弃置失牌过滤器，待 ≥3 同构）。
npack：mobile（modifiers 247→248）、key（modules 107→108/triggers 90→91/modifiers
179→178）、shahai（modules 37→39/modifiers 1062→1060/actives 4→6）；冻结计数同步
suitcase（178）与 mbkuili（248）。Golden 2/2
（original_armor_family_zero_primitive.rs，含长视「他人回复→护甲」行为链）。
全量回归 953 二进制 0 失败。清单 5072/461/0/2081（诚实债务约 592）。

## 本轮：shahai 挖矿第四批（2026-08-21 第十批）

回合始末/造成伤害摸牌/击杀/失去体力四类新角度扫描（5 命中，2 落地）+ 两个
过滤器缺口的同构需求统计（他人杀伤害仅 1 例不立项；装备失牌 4 例但其中 2 例
依赖自定义卡，基础变体需求 2 例仍低于门槛）：

| 技能 | 实现 | 精度 |
|---|---|---|
| hongxi 烘曦 | `trigger death owner_alive` forced + if hp<max → 回满（olxushen 同型） | 精确 |
| meihuo 魅祸 | `cards_moved owner_alive_event_player_from_equip_empty optional`（**ADR 0118 过滤器再消费**）→ `gain_target_hand_card(self, self_skill)` | 近似（仅手牌；装备区获得+自动装备未建模） |

剔除：shishui（无 event_target 单目标表达式）、bzhuiji（虚拟决斗被 phase_any_end
门禁挡）、shouge/kongdongyin（自定义卡/死亡抵挡）。
**⚠️ 包 pin 抬升**：meihuo 复用 pxsizhou 的 `gain_target_hand_card`（ABI 647），
shahai 包 pin 645→**647**（零新原语但既有原语抬包；host 651 兼容）。断言同步
（shahai_rewrite ×3、armor_family ×1）+ README pin 表更新。
npack：shahai 647（modules 39→41、triggers 33→35、modifiers 1060→1058）。
Golden 4/4；全量回归 953 二进制 0 失败；shahai noop 块 1042→1040。
踩坑：meihuo def 手抄成 pojian 的 def（第六次 def 坑）——catalog 不含扩展源
技能，从 npack skill_index 找回正确 def。

## 本轮：装备存在性值源族（2026-08-21 第十一批）

「装备失牌」过滤器第 3 例经全包搜索不存在（诚实不立项）；但勘察发现新家族：
**6 技被「有/无武器/防具」条件挡住**（pingxu/jufu/hujia_equip3/4/
shijianhuisu_equip3/4），且值源枚举已有 OwnerHasWeapon/OwnerArmorEmpty/
OwnerEquipCountExactlyOne 半覆盖——只缺 2 个变体。达 ≥3 门槛，立项：

- **新增值源 2 个（6 处注册，零 ABI——值源阶梯已预领段）**：
  `owner_no_weapon`（无武器 0/1）、`owner_has_armor_equip`（有防具 0/1）；
  注册点：modifier.rs 枚举尾（序数冻结）+ eval 臂、registry.rs 解析/反解、
  typecheck 白名单、lsp doc。
- **门控扩展 3 处**：owner_has_weapon += sha_damage add（jufu）；
  owner_armor_empty += distance_to add（pingxu）；owner_equip_count_exactly_one
  += distance_to/from add（shijianhuisu）。
- **6 技迁移（shahai，包 647 不变）**：均为纯条件距离/伤害 modifier——
  pingxu（无武器出向 -1 + 无防具入向 +1）、jufu（有武器杀伤害+1）、
  中兴护甲 equip3/4（有防具 入向+1/出向-1）、试剑回溯 equip3/4（装备仅此一件
  入向+1/出向-1）。全部精确（上游 globalFrom/globalTo/damageBegin 与
  distance_from/distance_to/sha_damage 逐条映射）。
- npack：shahai 647（modules 41、triggers 35 不变；modifiers 1058→1059）。
  Golden 5/5；全量回归 953 二进制 0 失败 + content-compiler 532/0。
- shahai noop 块 1040→1034。

## 本轮：shahai 挖矿第五批（2026-08-21 第十二批）

濒死/判定后/摸牌后/手牌条件等新角度扫描后按最短描述池人工审阅，4 技落地
（shahai 647 不变，modules 41→43、triggers 35→37、modifiers 1059→1057）：

| 技能 | 实现 | 精度 |
|---|---|---|
| huimeng 辉梦 | `hp_recovered owner_is_target optional → draw_cards(self, 2)` | 精确 |
| hstianqi_shali 沙鳞 | `hp_recovered owner_is_target` forced → `add_mark(self, 3, event_amount)`（**表达式护甲增量**，EffectAddMarkExpr 管线） | 精确 |
| hlongyi 黑龙鳞 | `card_ignored_hand_limit(black) override 1`（mbxiahui 同款） | 精确 |
| mtg_shamolvzhou 杀戮绿洲 | `card_ignored_hand_limit(basic) override 1`（CardSelector::Basic） | 精确 |

剔除：qingliu/qinglianxindeng（无「受击火焰/锦囊伤害」泛化过滤器）、
chilongya（无火属性伤害 add 修饰目标）、chunbing（持续五回合时限）、
cangchizhibi×4（选他人横置=choose-other-授标记缺口，与 mbzhenfu 同源）。
Golden 6/6；全量回归 953 二进制 0 失败；shahai noop 块 1034→1030。

**⚠️ def 手抄坑第七次（本批最大教训）**：HUIMENG 测试常量把 1217004629 抄成
1217006429（4/6 换位），触发「断言失败但数据在场」的幽灵故障，误导向
并行会话/编译器去重/拼接差异等多条歧路约两小时；期间四次「重建-部署-验证」
全链自扰。**教训固化：def 断言失败时第一步先打印实际值对比常量**（本批最终
靠 dump 触发清单立即现形）；另 include_bytes 陈旧二进制在本批再次现身
（touch 测试文件强制重编可破）。

## 本轮：choose_other_add_mark 原语立项落地（2026-08-21 第十三批）

第十二批登记的「选他人授标记」缺口（5 例同构）正式立项并同批落地：

- **新原语 `choose_other_add_mark(player, mark_def, delta)`**（ABI 652 / tag 672 /
  Cont 271，先在 ABI-COORDINATION.md 领取表占号后实施）：选择续体——player 选
  一名其他存活角色，resolver 对目标 run_effects AddMark。全链路 15+4 处注册
  （effect 枚举/bytecode/vm 逆序 pop delta→mark→player/ir/typecheck 字面量/
  codegen ConstU32+ConstI32/lib 652 阶梯/原语+dispatch/Cont 枚举+from_tag+分发+
  resolver/hash tag154 复用 4×u32+盐值/vm_host skill 注入/provenance/lsp 单行）。
- **host bump 651→652**；mobile/shahai 包 pin 均 652；冻结断言同步 10 处
  （扫描器批量 + 独立源编译断言回退 1 处——zhuran 单源不含新原语仍 500）。
- **5 技迁移**：mbzhenfu 镇抚（mobile，精确近似——弃置门槛未建模）+
  cangchizhibi×4 苍鸱之臂（shahai，近似——仅横置，上游 link() 切换未建模）。
- Golden 3/3（original_choose_other_add_mark.rs：双形态行为链——mbzhenfu
  「确认→选目标→他人+1 护甲」与 cangchizhibi「确认→选目标→他人横置」）。
  全量回归 954 二进制 0 失败。
- 踩坑：批量写块脚本在 mbzhenfu FileNotFoundError 处中止导致后段未执行
  （重建通过但产物无变化，靠 npack 计数核查发现）；测试块提取再度忘括号平衡。

## 本轮：shahai 挖矿第六批·杀强化族（2026-08-21 第十四批）

「杀被闪避/指定目标后」新角度扫描（4 命中 3 落地，shahai 652 不变，
modules 43→49、triggers 37→43、modifiers 1057→1051）：

| 技能 | 实现 | 精度 |
|---|---|---|
| shoulie 狩猎 | `card_used owner_source_sha optional` → 弃 1 + `make_current_card_unrespondable`（natuxing 同款） | 近似（上游逐目标询问；弃牌不限手牌） |
| lianwu 炼武 | `modifier card_target_limit(sha) add 1` | 部分（红杀不可闪避无「红杀」选择器） |
| xiandan 弦弹 | choose(1,2) 分支：弃 1 + 不可闪避 / 弃 1 + `increase_current_card_base_damage`（oldigong 同款） | 近似（弃牌颜色约束未建模） |

剔除：poyue（三子句全为颜色条件型——黑杀无视距离/红杀不限次且不可闪避，
需 per-color unrespondable/distance 选择器族）。
Golden 7/7；全量回归 954 二进制 0 失败；shahai noop 块 1026→1023。
def 手抄坑第八次（shoulie 4057970364 抄成 4059770364）——本批固化的
「断言失败先打印实际值」规程第一次当场奏效（1 分钟定位）。

## 本轮：shahai 挖矿第七批·摸牌族（2026-08-21 第十五批）

机制普查（伤害 183/摸牌 104/其他 253 为主导；跨界专属机制仅少数）后，
摸牌族最短池审阅 4 技落地（shahai 652 不变，modules 49→53、triggers 46→46+4、
actives 6→7）：

| 技能 | 实现 | 精度 |
|---|---|---|
| hslingjian_xingtigaizao_equip1 | `damage owner_source_sha` forced → draw 1 | 精确 |
| jingcu | `active play_phase` → change_max_hp(-1) + draw 2 | 近似（maxHp>1 门槛未建模） |
| yuedong | `phase_play_end owner_is_target optional` → 令一名其他角色摸 1 | 近似（成长与含自己未建模） |
| diy_jiaoxia | `card_targeted owner_alive_targeted_by_other optional` → draw 1 | 近似（不限红色，泛化过滤器） |

剔除：shihuo（无全局火焰伤害泛化过滤器，dcsbronghuo 为专用）、
feixu/luomei（无「他人响应闪/自用梅花」过滤器）、xingtigaizao_equip2
（无「受杀伤害」过滤器）、tanhuadong/qingtuan（持续 N 回合时限）。
Golden 8/8；全量回归 954 二进制 0 失败；shahai noop 块 1023→1019。
def 手抄坑第九次（yuedong 222078864 丢位成 22278864）——规程再次 1 分钟拦截。

## 本轮：shahai 挖矿第八批·伤害族（2026-08-21 第十六批）

伤害族（192 块）最短池审阅，**5 技一次性落地**（shahai 652 不变，
modules 53→57、triggers 46→50、modifiers 1051→1044）：

| 技能 | 实现 | 精度 |
|---|---|---|
| yushou_misha | `damage owner_is_target` forced → 护甲+1 | 精确 |
| mbaizhan | `damage owner_is_source` forced → `add_mark(self,3,event_amount)` 等量护甲 | 精确 |
| hstianqi_dalian | `damage owner_is_source` + if 已受伤 → 等量回复 | 精确 |
| tuteng3 | `damage_before owner_is_target` → 减伤 1 + `remove_skill(self_skill)` 自移除（一次性图腾） | 精确 |
| xingjunyan | `sha_damage add 1` + `sha_damage_taken add 1`（双向） | 精确 |

剔除：bingshuang/longhuo（AOE 无原语）、tuteng4/6（回合内首次门槛）、
shinancao（下一次伤害增益授予）、zonghuo（可选费用转火）。
Golden 9/9；全量回归 954 二进制 0 失败；shahai noop 块 1019→1014。
护甲=MARK_HUJIA 底座（ADR 0643）在本批被三个技能复用，解体红利持续兑现。

## 本轮：shahai 挖矿第九批·其他桶（2026-08-21 第十七批）

「其他」桶（269 块）最短池审阅，**7 技落地**（shahai 652 不变，modules 57→60、
triggers 50→53、modifiers 1044→1041）：

| 技能 | 实现 | 精度 |
|---|---|---|
| tuteng7 | `phase_end` forced → 令他人回复 1 | 近似（受伤存在门槛） |
| lingyu | `phase_end optional` → 令他人回复 1 | 近似（同上） |
| lingxue | `hp_recovered` forced → 护甲+1 | 精确 |
| ronghuo | `card_nature(sha) override fire`（ollihuo 同款） | 近似（modifier 等价） |
| moyao | `card_target_cancel(ordinary_trick) override 1` | 近似（不分使用方） |
| gjqtwuxie / swd_wuxie | `card_target_cancel(delayed_trick) override 1`（yjzhenlve 同款） | 近似（同上） |

剔除：chunbing/yuanbaorou（持续 N 回合）、chuidiao（随机）、
jinjilengdong e3/e4（翻面值源缺）、yinmilichang e3/e4（体力=1 值源缺）、
yinmilichang_e5/shengxiuhaojiao（条件目标取消/嘲讽）、guiying/xieneng
（自定义卡）、mtg_feixu（随机置顶）。

**⚠️ 前缀碰撞惊魂**：`// lingyu` 前缀匹配到 `// lingyue（凌月）`——替换区间
错位会吞掉两块之间数十个技能；所幸脚本的防御性写盘顺序（全部替换成功后才
落盘）让断言中止时**文件零损伤**。处置：块查找一律带全角括号
`// <name>（` 精确匹配（已固化）。上游纠偏：lingyu/lingxue 的描述对调
（lingyu=结束阶段回复、lingxue=回复得护甲），再次印证「必读上游体」。
Golden 10/10；全量回归 954 二进制 0 失败；shahai noop 块 1014→1007。

## 本轮：shahai 挖矿第十批·装备防御对（2026-08-21 第十八批）

装备桶（50 块）+ 判定桶（28 块）审阅，2 技落地（shahai 652 不变，
modules 60→62、triggers 53→55）：

| 技能 | 实现 | 精度 |
|---|---|---|
| yulin 玉鳞 | `damage_before optional` → `choose_discard_equipment_prevent_current_damage`（renxin 同款） | 精确 |
| 中兴护甲·斥 equip2 | `damage optional` → 弃来源一张装备 | 近似（不限杀伤/防具） |

剔除：shijianhuisu_equip1/2（equip_used 无 per-kind 变体，既有登记）、
minjing/shending（虚拟装备自定义卡）、tianlun（判定改写不含判定区选项）、
xianzhi（观顶调序无原语）、shentou/xiuluo/dunxing 等（判定+分支复合）、
xianyin/donghuangzhong（判定区批量操作无原语）。
Golden 11/11；全量回归 954 二进制 0 失败；shahai noop 块 1007→1005。
判定桶结论：除 gtiandao（第八批已落地）外全部依赖判定+分支/判定区操作复合
原语，简单面为零——判定交互族若要继续需「判定区单张弃置/观顶调序」等原语
立项（各 1-2 例，未达门槛）。

## 本轮：jsrg 议事系统 ADR 0644 立项 + 跨包模板扫描证伪（2026-08-22 第十九批·设计批）

- **跨包模板扫描**：以本会话模板库（伤害四模板/护甲底座/目标不可达三件套/
  转火）扫 sp/tw/xianding/sp2/onlyOL/refresh 六包 noop 池——**零命中**，
  印证勘察批结论：其余包简单面已被前会话挖尽，今日模板是 shahai/跨界包特有
  语义面的产物。
- **ADR 0644 议事系统设计**（docs/adr/0644-yishi-debate-system.md）：
  - **范围更正**：jsrg 153 块中真正议事耦合仅 **11 技**（发起 5/反应 6），
    其余 142 块为普通机制技，不应被系统级标签整体阻塞。
  - 上游机制实测（content.ts chooseToDebate）：参与者强制亮一张手牌为意见→
    红/黑/others 分组→多数决（平票无结果）→debateShowOpinion/debateEnd 两
    时机→按结果或「意见异同」分支；意见牌默认收回。
  - 三阶段设计：P1 旗舰技权威结算原语（jsrgchaozheng/jsrgsheju，1 技 1 号，
    dchaoshi 模式）；P2 通用 DebateResolved 事件+过滤器+表达式（解锁反应型
    6 技）；P3 深水区（意见改写/随机化/未议事追踪）暂不承诺。
  - ABI 预算：P1 2-3 号 / P2 发起原语 2 号+零 ABI 事件过滤器。
- 无代码变更；全仓维持 954 二进制 0 失败基线。

## 本轮：ADR 0644 P1 落地——议事双旗舰原语（2026-08-22 第二十批）

- **领号**：ABI 653（RESOLVE_JSRGCHAOZHENG，tag 673）/ 654（RESOLVE_JSRGSHEJU，
  tag 674）；host bump 652→654；jsrg 包 pin 562→**654**。
- **注册**：12 处全链路（effect 枚举×2/bytecode×2/vm 逆序 pop/ir/typecheck
  self_skill 门禁/codegen+653/654 阶梯/dispatch×2/hash tag154 复用新盐值/
  provenance/lsp）；无 ContinuationKind（纯同步结算）。
- **引擎实现**（新文件 pack_jsrg.rs）：`debate_groups_first_card` 首张手牌
  分组 + 两原语；P1 近似（首张牌代替上游 AI 随机、意见牌不移动）逐点注明。
  实现期修复两处多数决逻辑缺陷（black_count>0 守卫误拦全红场景；
  participants 误含无手牌者）——由三分支 Golden 当场暴露。
- **迁移**：jsrgchaozheng（朝政，phase_prepare optional）、jsrgsheju（设居，
  攻守双触发）。npack：jsrg 654（modules 22→25、triggers 19→22）。
- **Golden 5/5**（original_jsrg_debate_p1.rs）：红多数回复+全同摸牌 / 黑多数
  全同摸牌 / 平票无结果 / sheju 黑多数双方减上限 / 包契约。踩坑复刻：forced
  触发 dispatch 返回效果需显式 run_effects（yongsi2 教训重演，行为测试修复）。
- 冻结断言同步 4 处（zhaohan/zuwang/view_as_equipment_filter 的 562→654 与
  触发计数 19→22）。全量回归 **955** 二进制 0 失败。
- 清单：5069 完整 / 464 近似 / 0 TODO / 2078 noop。

## 本轮：ADR 0644 P2 落地——议事事件模型（2026-08-22 第二十一批）

- **领号**：ABI 655（START_DEBATE_ALL_OTHERS，tag 675）/ 656
  （START_DEBATE_TARGETS，tag 676/Cont 272）；host bump 654→**656**。
- **事件模型**：`DomainEvent::DebateResolved { initiator, red, black, result }`
  （EventKind tag 45）+ `TriggerContext` 尾部追加 `debate_red/debate_black`
  快照字段（serde default，33+3 构造点同步）；P1 的 chaozheng 结算后补发该事件。
- **过滤器 ×3**（5 处注册零 ABI）：`debate_owner_is_initiator` /
  `debate_owner_participated` / `debate_owner_opinion_differs_initiator`。
- **发起原语 ×2**：全员版（同步发事件）/ 指定目标版（Cont 272 选择续体）；
  另加引擎内部 `EmitDebateResolved` 请求（不经 VM/DSL，无 tag）。
- **迁移**：jsrgfayi（法一，反应型——`debate_resolved debate_owner_participated
  optional → choose_other_damage`；近似：任意名→单目标，意见异同约束留 P3）。
  jsrg pin 654 不变（fayi 未用 655/656，新原语待后续反应技消费）。
- **注册坑（本批三个）**：①EVENTS 白名单之外引擎侧 registry.rs 的 map_event
  也要登记（"unknown event" 双源）；②regex 批量补构造字段时把链式调用
  `state.skills.get(...)` 从中间劈开（SkillAdded 上下文）；③测试直接传
  SkillId::NONE 过不了 originating_skill_is_enabled 守卫——需先 AddSkill 取真实 id。
- Golden 3/3（original_jsrg_debate_p2.rs：fayi 参与触发伤害链 / P1 补发事件
  快照 / 包契约）；P1 5/5 保持；全量回归 **956** 二进制 0 失败。
  清单 5069/464/0/2078（文件轴不变——fayi 所在包原已计入近似轴）。

## 本轮：ADR 0644 P2 收口——首个发起型消费者（2026-08-22 第二十二批）

- **新增组合过滤器 ×2**（5 处注册零 ABI）：`debate_owner_is_initiator_result_red` /
  `_black`（发起者 + 结果分支复合——单过滤器字符串约束下的 DSL 表达方案）。
- **迁移 jsrgshelun（社论）**：首个消费 `start_debate_all_others`（ABI 655）的
  发起型技——`active uses_per_turn 1` 发起全员议事 + 两个 debate_resolved 触发
  按红/黑分支（红=弃两张 choose_other_discard_target_card×2；黑=
  choose_other_damage(self,2)）。近似逐点注明（参与者约束/惩罚对象跨阶段状态）。
  jsrg pin 654→**655**（shelun 首次消费 655）。
- Golden 2/2（original_jsrg_debate_initiator.rs：包契约 + 黑分支行为链
  「ActivateSkill→confirm→选目标→伤害2」）；P1 5/5 与 P2 3/3 保持；冻结断言
  同步 5 处（ABI 654→655 ×4 + 触发计数 23→25 ×2）。全量回归 **957** 二进制
  0 失败。清单 5068 完整 / 466 近似 / 0 TODO / 2076 noop。
- 选择链踩坑（新）：排空调式循环对「optional confirm」发目标 id 会被当作
  取消——confirm 步固定 selections=[1]，目标步才发 player id（分步计数法）。

## 本轮：start_debate_targets 首消费者（2026-08-22 第二十三批）

- **迁移 jsrgshanzheng（擅政）**：首个消费 `start_debate_targets`（ABI 656，
  Cont 272 选择续体）的指定目标型发起技——`active uses_per_turn 1` 选 1..N
  参与者议事 + 红分支 `choose_other_damage(self, 1)`。近似逐点注明
  （参与者含自己/未参加约束/黑分支意见牌快照待 P3）。
  jsrg pin 655→**656**（至此 655/656 双发起原语均有消费者）。
- Golden 2/2（original_jsrg_debate_targets.rs：包契约 + 行为链
  「ActivateSkill→Cont272 选参与者→红 confirm→选目标→伤害1」——四步选择链
  全通）；P1/P2/initiator 10/10 保持；冻结断言同步 6 处。
  全量回归 **958** 二进制 0 失败。清单 5066 完整 / 467 近似 / 0 TODO / 2075 noop。
- 议事系统 DSL 表达力现状：发起（全员/指定）+ 结果分支（红/黑×发起者）+
  参与反应（participated/differs）全部可表达；仅剩 P3 深水区
  （意见改写/随机化/未议事追踪/意见牌快照）。

## 本轮：shahai 中长描述段挖矿（2026-08-22 第二十四批）

中长段（25-60 字，392 块）首扫，4 技落地（shahai 652 不变，triggers 55→59）：

| 技能 | 实现 | 精度 |
|---|---|---|
| xixue 吸血 | `damage owner_source_sha` forced + if 未满 → 等量回复 | 精确 |
| ningbin 宁彬 | `damage owner_target_thunder` forced → 回复 1 | 精确 |
| pangufu 盘古斧 | `damage owner_is_source` forced → 弃受伤者一张 | 近似（有牌门槛） |
| shengzhang 生长 | `phase_discard_end owner_is_target` forced → 体力上限+1 | 近似（弃牌门槛） |

剔除：daofa（「旁观者视角·他人来源伤害」过滤器缺口——owner 既非目标亦非来源
的 damage 触发，1 例未达门槛，诚实 noop + 缺口登记）；feixu/luomei（他人
响应闪/自用梅花——既有登记缺口）；jixuan（额外回合）；yishan（最近失去牌
快照）；yuchandui×5/yuchangen×2（弃任意张摸等量——「任意张」无原语）；
zhongji（弃黑手牌令伤害+1——颜色约束弃置缺原语）。
Golden 12/12；全量回归 958 二进制 0 失败；shahai noop 块 1005→1001。

## 本轮：玉蝉族整族迁移（2026-08-22 第二十五批）

中长段次屏扫描发现 **yuchan*_equipN 40 个块描述完全相同**——追上游证实全部为
空壳子技（`yuchandui_equip1: {}`），真实引擎是共享全局技 `g_yuchan_equip`
（出牌阶段限一次，弃任意张基本牌摸等量）。**1 原语解锁 40 技**，本会话单批
最大规模：

- **领号 ABI 657**（DISCARD_BASIC_DRAW_EQUAL，tag 677/Cont 273）：选择续体
  （选 1..N 张基本手牌，allow_cancel）→ resolver 弃所选并摸等量
  （`DiscardCards + DrawCards` 串行）。全链路 12+4 处注册。
- **40 技批量迁移**（脚本写块，def 全部从 npack skill_index 读取——零手抄）：
  `active play_phase uses_per_turn 1 → discard_basic_draw_equal(self, self_skill)`。
  近似注明：上游 40 变体共享每回合一次全局次数，此处按技能各自限一次。
- npack：shahai pin 652→**657**（actives 7→47、modules 63→106）；冻结断言
  同步 3 处（rewrite_batch/armor_family/choose_other_add_mark 的 shahai pin）。
  Golden 13/13（含「40 块全部成为 active 消费者」的整族断言）；全量回归
  **958** 二进制 0 失败。清单文件轴不变（5066/467/2075——玉蝉 40 块在 shahai_meta 单文件内，
  文件轴粒度不感知块级变化）；**块轴 noop 3265→3225**（shahai_meta 内
  999→959，单批 -40）。此差异再次印证块轴普查结论（勘察批）。
- 剔除：mtg_linzhongjianta（有武器时基本牌当杀——`view_as sha from hand
  filter card_basic` 需装备条件门控，无 owner_has_weapon×view_as 组合，近似
  会放开无武器场景）；tuzhen（弃非基本牌——无 non-basic 弃置原语）。

## 本轮：四神器族整族迁移（2026-08-22 第二十六批）

同描述聚类（玉蝉工艺复用）发现四个 5 块族（黄麟之琮/玄武之璜/朱雀之璋/
白兽之琥，共 20 块）——与玉蝉不同，这些是**真实体的 _equipN 变体**（非空壳），
但语义同为「结束阶段 + 弃牌 + 效果」，现有原语可近似整族覆盖：

| 族（5 块） | 实现 | 近似 |
|---|---|---|
| 黄麟之琮 | phase_end optional → 弃1 + 护甲+1 | 弃牌颜色（黑）/无护甲门槛 |
| 玄武之璜 | 弃1 + 回复1 | 弃牌颜色（红） |
| 朱雀之璋 | 弃1 + 对他人伤害1 | 颜色/体力门槛/火属性 |
| 白兽之琥 | 选他人弃其一张 | 随机→选择 |

shahai 657 不变（triggers 59→79）；Golden 14/14（含四族 20 块全量断言）；
全量回归 958 二进制 0 失败。**块轴 noop 3225→3205**（shahai_meta 标记
959→939）。文件轴不变（单文件粒度）。
同描述聚类普查结论：≥3 块族已全部消化（玉蝉 40 + 四神器 20 = 60 块，
剩余无 ≥3 同描述族）。

## 本轮：中长段散技第二批（2026-08-22 第二十七批）

25-60 字散技次屏（fengxian 起）核上游体，2 技落地 + 1 剔除（shahai 657
不变，triggers 79→80、actives 47→48）：

| 技能 | 实现 | 精度 |
|---|---|---|
| kuixin 愧心 | `phase_end optional` → gain_target_hand_card（pxsizhou 同款） | 近似（手牌数门槛） |
| fengxian 奉献 | `active uses_per_turn 1` → 令他人弃一张 | 近似（AOE→单目标） |

剔除：chengjian（承箭）——与 daofa 同属「旁观者·他人来源伤害」过滤器缺口，
**该缺口现积累 2/3 例**（daofa/chengjian，再积 1 例即达立项门槛，诚实 noop
并更新计数）。其余次屏候选多依赖累计计数器（feng/song/ya 的「累计 N 次后
下次±1」形态）、机甲状态、合成装备、额外回合等无原语机制。
Golden 15/15；全量回归 958 二进制 0 失败。块轴 noop 3205→3203
（shahai_meta 939→937）。文件轴不变。

## 本轮：旁观者伤害过滤器立项落地（2026-08-22 第二十八批）

第 3 例定向搜索：宽搜「其他角色…伤害」候选池，**xuezhou（血咒）模式二**确认为
第 3 例旁观者同构（上游两种模式均要求事件角色 ≠ owner）——缺口计数 3/3 达标：

- **新过滤器 `owner_alive_other_source_damage`**（5 处注册零 ABI：catalog/
  registry×2/dispatch eval/typecheck/lsp）：damage 事件 + 来源≠NONE + 来源≠
  owner（旁观者视角）。
- **三技一次性解锁**：
  | 技能 | 实现 | 近似 |
  |---|---|---|
  | daofa 道法 | `damage <新过滤器> optional` → 弃来源一张 | 来源有牌门槛 |
  | chengjian 承箭 | 同过滤器 → draw_cards(event_source,1) | 限杀→任意伤害 |
  | xuezhou 血咒 | 同过滤器 → lose_hp(来源,1)+recover_hp(self,1) | 仅模式②常驻 |
- shahai 657 不变（triggers 80→83）；Golden 16/16；全量回归 958 二进制
  0 失败。块轴 noop 3203→3200（shahai_meta 937→934）。
- def 手抄坑**第十次**（chengjian 4267414717 抄成 4267417471，414/417 换位）
  ——「打印实际值」规程再次拦截（本次连数据都打印对了但常量是早期手写的，
  教训升级：**常量必须从打印输出复制粘贴，不得从笔记/delta 文本回抄**）。

## 本轮：中长段散技第三批·治疗护甲族（2026-08-22 第二十九批）

装备失牌第 3 例排查：duci（毒刺）确认为基础变体同构，但 yinzong/duci 仅
**2 个干净消费者**（cyqiaoxie/jiguanyaoshu 效果侧依赖自定义卡，过滤器解锁
也无用）——维持 2/3 不立项。转中长段治疗/护甲族，5 技落地（shahai 657
不变，triggers 83→86、actives 48→50）：

| 技能 | 实现 | 近似 |
|---|---|---|
| baihupifeng 白虎披风 | `phase_end optional` + if 受伤 → 回复 1 | 「全场最小之一」门槛 |
| fuyan 附焰 | `damage owner_is_target optional` → 授他人护甲（652 原语复用） | 无护甲门槛 |
| xinci 衅刺 | `active` → 弃 1 + 他人 1 伤害 | 颜色/失去体力→伤害口径 |
| shengguang 圣光 | `active` → 弃 1 + 他人回复 1 | 颜色 |
| binxin 冰心 | `phase_end owner_alive optional` + if hp(事件角色)==1 → 授护甲 | 目标重选 |

Golden 17/17；全量回归 958 二进制 0 失败。块轴 noop 3200→3195
（shahai_meta 934→929）。

## 本轮：中长段散技第四批（2026-08-22 第三十批）

3 技落地（shahai 657 不变，triggers 86→88、actives 50→51）：

| 技能 | 实现 | 精度 |
|---|---|---|
| shennongding 神农鼎 | `active uses_per_turn 1` → 弃 2 + 回复 1 | 弃置范围/每阶段→每回合 |
| zhiyuan 止怨 | `damage_before owner_is_source optional` → prevent + 目标等量回复 | **精确**（防伤改回复完整链） |
| shijie 拾阶 | `phase_end optional` → gain_target_hand_card | 「其补摸一张」未建模 |

剔除：poxing（需「目标体力>自己」过滤器——owner_is_target_hp 系列均为己方
口径）；oldfushi（选他人减上限+回复——choose-other-change-max-hp 缺）。
Golden 18/18；全量回归 958 二进制 0 失败。块轴 noop 3195→3192
（shahai_meta 929→926）。
def 手抄坑**第十一次**（zhiyuan 1656864566 抄成 1656845666，864/845 换位）
——即使「从打印输出取值」，手敲下划线分组仍可错位；规程最终形态：
**常量以字符串形式直接从 python 输出复制，写完再机械比对一次**。

## 本轮：中长段散技第五批（2026-08-22 第三十一批）

3 技落地（shahai 657 不变，triggers 88→90、actives 51→52）：

| 技能 | 实现 | 精度 |
|---|---|---|
| hudun 护盾 | `damage owner_is_source` forced → 护甲+1 | 近似（无护甲/目标非己门槛） |
| guiyuan 归源 | `active` → 弃1+回复1+摸1 | 近似（弃杀约束/受伤门槛） |
| mojian 磨剑 | `card_targeted owner_source_sha optional` → 目标摸1+自己回复1 | 精确 |

剔除：shengteng（锦囊伤害触发——damage 事件无 trick-card 过滤器）。
**⚠️ 新踩坑：DSL 无 `!=` 运算符**——hudun 首版 body `if event_player !=
self` 触发 lex error "unexpected byte 33"（0x21 即 `!`，报错位置漂移到无关
注释处误导排查）；改为无守卫 + 近似注记。**表达式运算符清单：==/</>/<=
（若在）可用，`!=` 不在语言中**——写 body-if 前先核对运算符存在性。
Golden 19/19；全量回归 958 二进制 0 失败。块轴 noop 3192→3189
（shahai_meta 926→923）。

## 本轮：中长段散技第六批（2026-08-22 第三十二批）

3 技落地（shahai 657 不变，triggers 90→92、actives 52→53）：

| 技能 | 实现 | 精度 |
|---|---|---|
| diesha 叠杀 | `damage owner_source_sha` forced → 目标失去1+自己回复1 | 精确 |
| fengming 蜂鸣 | `active` → 弃1+他人回复1 | 近似（弃装约束/补摸尾段） |
| fanshi 反噬 | `phase_discard_end` forced → 失去1+摸1 | 近似（造伤门槛） |

剔除：jianren（AOE 伤害，第八批已剔重复确认）、poyue（per-color 选择器，
既有登记）、tianshe/xuying（属性伤害防转族——「受击属性伤害」过滤器已有
thunder 变体但 fire/通用缺，且 xuying 需条件 prevent）。
Golden 20/20；全量回归 958 二进制 0 失败。块轴 noop 3189→3186
（shahai_meta 923→920）。

## 本轮：中长段散技第七批（2026-08-22 第三十三批）

3 技落地（shahai 657 不变，triggers 92→97、actives 53 不变）：

| 技能 | 实现 | 精度 |
|---|---|---|
| xueren 血刃 | `damage owner_source_sha` optional → 目标失去1+自己失去1+摸2 | 精确 |
| txianqu 仙音 | `damage_before owner_is_source optional` + storage 限一次 → 防伤+摸2+回复1 | 近似（出牌阶段以「本回合内」代替） |
| yaliang 雅量 | 双 trigger `damage owner_is_source`/`owner_is_target` optional → 双方各摸1 | 近似（自伤会双触发，上游 source!==player 排除） |

剔除：xiaomoyu（「首回合内首次造成伤害」需要事件历史快照，
storage+turn_ended 清零表达为回合首次，非「首次触发后整回合失效」——
上游以 xiaomoyu2 临时技表达，语义为「发动一次后本回合失效」而非「回合首次」，
storage 方案方向相反，剔除）。
**新增测试坑：optional 触发若 body 守卫不满足仍会询问**——storage 门控在 body
而非 filter，第二次伤害 optional 仍 ChoiceRequested，确认后 body 不防伤；
测试断言第二次须 resolve 而非期待 Ok([])。
Golden 24/24；全量回归 985 二进制 0 失败。块轴 noop 3186→3183
（shahai_meta 920→917）。

## 本轮：中长段散技第八批（2026-08-22 第三十四批）

3 技落地（shahai 657 不变，triggers 97→100、actives 53 不变）：

| 技能 | 实现 | 精度 |
|---|---|---|
| yougeng 酉羹 | `phase_prepare owner_min_hp` forced + body `hp<max` → 回复1 | 近似（无两回合持续） |
| liyutang 鲤鱼汤 | `phase_end owner_min_hp` forced → add_mark(self,3,1) | 近似（无两回合持续） |
| mieshi 灭世 | `phase_end owner_is_target` forced → 失去1+choose_other_damage(self,1) | 近似（随机→选择、火焰→普通） |

「全场最少或之一」门槛由既有 `owner_min_hp` 过滤器（owner.hp ≤ 全体存活）
表达，零新原语。食物牌（酉羹/鲤鱼汤）上游以 markcount 计数在两回合后
removeSkill，DSL 无持续回合——技能存续期间每轮触发，已注记。
**⚠️ phase_end 事件源踩坑**：DSL `phase_end` 来自
`TurnStateChanged{phase:End}`（EventKind::PhaseEnd）；`PhaseEnded{End}`
映射的是 `turn_ended`——测试驱动阶段结束触发时用错事件源会得到空效果
列表且不报错。`PhaseEnded{Draw/Play/Discard}` 才分别映射
phase_draw_end/phase_play_end/phase_discard_end。
Golden 28/28；全量回归 985 二进制 0 失败。块轴 noop 3183→3180
（shahai_meta 917→914）。

## 本轮：中长段散技第九批·食物牌批（2026-08-22 第三十五批）

4 技落地（shahai 657 不变，triggers 100→102、modifiers 950→948）：

| 技能 | 实现 | 精度 |
|---|---|---|
| molicha 茉莉茶 | `modifier card_target_cancel(black) override 1` | 近似（不分使用方/无四回合） |
| yuanbaorou 元宝肉 | `modifier sha_use_limit add 1` | 近似（无四回合持续） |
| tanhuadong 昙花冻 | `phase_play_end owner_is_target` forced → 摸1 | 近似（无三回合持续） |
| mapodoufu 麻婆豆腐 | `phase_end owner_is_target` forced → choose_other_discard_target_card | 近似（随机→选择/无持续） |

食物牌族（card/gujian 烹饪牌）上游统一形态：mark="card" + 主技仅做
markcount 倒计时自移除 + 实际效果在 mod/子技/group——DSL 迁移取效果面、
倒计时体不迁移，持续回合以「技能存续期间常驻」近似（已逐技注记）。
该族剩余：xiajiao（摸牌+1+弃1 双段）、qingtuan（首杀摸牌）、luyugeng
（发现）、mizhilianou（红桃当桃 view_as，可下批）。
选择链注意：forced 触发 + choose_other_discard_target_card 无 optional
confirm，直接进 ChooseOtherDiscardTargetCard → ChooseDiscardTargetCard 两段。
Golden 31/31；全量回归 985 二进制 0 失败。块轴 noop 3180→3176
（shahai_meta 914→910）。

## 本轮：食物牌族收口（2026-08-22 第三十六批）

6 块落地（shahai 657 不变，triggers 102→106、modifiers 948→944、view_as 8→9）：

| 技能 | 实现 | 精度 |
|---|---|---|
| xiajiao 虾饺 | `modifier draw_count add 1` + 弃置段入 xiajiao2 | 近似（无三回合持续） |
| xiajiao2 虾饺·弃 | `phase_draw_end owner_is_target` forced → 弃1 | 近似（选牌→确定性弃前1） |
| mizhilianou 蜜汁藕 | `view_as tao any heart from hand_equip` | 近似（无四回合持续） |
| chunbing 春饼 | `modifier hand_limit add 1` | 近似（无五回合持续） |
| gudonggeng 骨董羹 | `damage_before owner_is_target` forced → 减1 + remove_skill | 近似（杀限定缺失/无持续） |
| qingtuan 青团 | `card_used owner_source_sha` forced + storage 首张门控 → 摸1 | 近似（usable:1→storage） |

食物牌族（card/gujian 烹饪牌）可表达面至此收口；族内剩余为 luyugeng
（发现机制，无原语）。子技能块结构注意：上游 xiajiao2（弃置段）在
.skill 有独立块，效果按上游结构放入对应块而非全并入主技。
gudonggeng 复用一次性图腾模板（modify -1 + remove_skill 自移除），
「受击杀伤害」过滤器缺口维持登记（+1 例）。
Golden 34/34；全量回归 985 二进制 0 失败。块轴 noop 3176→3170
（shahai_meta 910→904）。

## 本轮：中长段散技第十批（2026-08-22 第三十七批）

5 技落地（shahai 657 不变，triggers 106→111、modifiers 944→940）：

| 技能 | 实现 | 精度 |
|---|---|---|
| guiyanfadao 鬼眼法刀 | `damage_before owner_source_sha` optional → 防伤 + 目标失去1 | 精确 |
| guaili 怪力 | `sha_damage add 1` + `damage owner_source_sha` forced → 弃2 | 近似（选牌弃→弃前2/连环排除） |
| huiqi 回气 | `damage owner_is_target` optional + hp<max → 令他人摸已损体力数 | 精确 |
| lingzhou 灵咒 | `card_used owner_source_ordinary_trick` optional → 目标自选摸1或回复1 | 近似（不排除转化锦囊） |
| ziyu 自愈 | `phase_end owner_alive` optional → choose(1,2) 回复或摸 | 近似（%4 门槛不可表达） |

**⚠️ sha_damage modifier 只在真实 PlayCard 管线消费**（command/play.rs
sha_damage_for_card），合成 DealShaDamage 效果不经过——guaili 的 +1 伤害
与 xingjunyan 同为 publish 断言覆盖，行为测试只验弃 2 段。
**count 弃置开 Custom 选择**：discard_hand_equip_count 在 forced 触发下仍
开选择，selections 须精确等于弃置数（多选报 discard-count choice context
changed，少选同）。
Golden 37/37；全量回归 985 二进制 0 失败。块轴 noop 3170→3165
（shahai_meta 904→899）。

## 本轮：中长段散技第十一批·濒死救场族（2026-08-22 第三十八批）

4 技落地（shahai 657 不变，triggers 111→114、actives 53→54、modifiers 940→936）：

| 技能 | 实现 | 精度 |
|---|---|---|
| nvwashi 女娲石 | `dying owner_alive` optional + hp>1 → 救濒死者1+自失1 | 精确 |
| shouwang 守望 | `dying owner_alive` optional → 濒死者回复1+护甲1 | 近似（每角色限一次） |
| xielei 挟雷 | `card_used owner_source_sha` optional → 弃1+他人1伤 | 近似（3人门槛/目标排除/雷电） |
| yulu 雨露 | `active uses_per_turn 1` → 他人摸1+弃其区域内1 | 近似（任意名→单目标） |

濒死救场模板成型：合成 `DomainEvent::EnteredDying{player, source}` 派发
（ctx.player=濒死者，event_player 引用即可救场）；**optional 触发在
dispatch 期直接开 confirm（返回 ChoiceRequested 而非 Ok(effects)）**，
测试须 commit→resolve [1] 两步，与 forced（返回效果列表）分流。
def 手抄坑第十二次预防成功：块替换脚本的 def-preserve assert 当场拦截
yulu 3153959245→...47 错写（.skill 侧零损伤）；但**测试常量仍是手敲**——
YULU 常量 3_159_392_45 又错一次（3_153_959_245 才对），publish 断言
抓到——规程再升级：测试常量必须从 npack skill_index 打印复制。
Golden 40/40；全量回归 985 二进制 0 失败。块轴 noop 3165→3161
（shahai_meta 899→895）。

## 本轮：中长段散技第十二批（2026-08-22 第三十九批）

4 技落地（shahai 657 不变，triggers 114→117、actives 54→55、modifiers 936→933）：

| 技能 | 实现 | 精度 |
|---|---|---|
| yuelu 月露 | `dying owner_alive` optional → 弃1+濒死者回复1+护甲1 | 近似（黑色→任意） |
| kbolan 博览 | `modifier draw_count add 1` | 部分迁移（仅摸牌阶段；置顶尾段缺） |
| diewu 蝶舞 | `active` 交一张 + storage 回合内首次摸1 | 近似（杀→任意牌） |
| xiyong 汐涌 | `phase_end owner_is_target` optional → 摸1 | 部分迁移（可用+再摸尾段缺） |

濒死救场族第二例落地（yuelu 复用 EnteredDying 派发 + event_player
引用）；diewu 复用 txianqu/qingtuan 的 storage 首次门槛模式（active 侧）。
部分迁移 2 例（kbolan/xiyong）均注记未建模尾段——「主效果可表达、
尾段无原语」型按近似/部分轴登记。测试常量已按新规程从 npack
skill_index 打印复制。
Golden 42/42；全量回归 985 二进制 0 失败。块轴 noop 3161→3157
（shahai_meta 895→891）。

## 本轮：中长段散技第十三批·补牌欠账族（2026-08-22 第四十批）

3 技落地（shahai 657 不变，triggers 117→120、actives 55→57、modifiers 933→930）：

| 技能 | 实现 | 精度 |
|---|---|---|
| huajian 化剑 | `phase_play_end` optional → choose_use_virtual_sha | 近似（素材费用未建模） |
| guozai 过载 | active 补至4 + storage 欠账 → phase_play_end 弃等量 | 近似（确定性弃置） |
| guozaix 过载·二 | 同体 uses_per_turn 2 | 近似（同上） |

**「补牌欠账」两段式模板成型**：active 摸差额 + storage_set 记欠账 +
phase_play_end forced 读欠账弃等量并清零（draw_cards 接受表达式张数；
discard_hand_equip_expr 同）。**⚠️ count/expr 弃置行为不一致**：
`discard_hand_equip_count`（字面量）开 Custom 选择（selections 须精确等数），
`discard_hand_equip_expr`（表达式）确定性弃前 N 无屏障——同族两条路径。
测试侧：DiscardFromHandEquip 直连效果亦为确定性；缩手牌勿直接改 zone
（card location inconsistency），走真实弃置效果。
剔除：shuijian（万箭虚拟——choose_use_virtual_card 被 typecheck 门禁在
phase_any_end，phase_prepare 不可用）；oldenze（需所选目标手牌数表达式）。
Golden 44/44；全量回归 985 二进制 0 失败。块轴 noop 3157→3154
（shahai_meta 891→888）。

## 本轮：中长段散技第十四批（2026-08-22 第四十一批）

4 技落地（shahai 657 不变，triggers 120→123、actives 57→58、view_as 9→10）：

| 技能 | 实现 | 精度 |
|---|---|---|
| lianzhan 连斩 | `damage owner_is_source` forced + storage 首伤分支 → 摸2 / 上限+1回复1 | 精确 |
| liechao 猎潮 | active + hand≤hp 守卫 → 摸4+翻面+跳弃牌 | 近似（未翻面门槛缺） |
| linyun 凌云 | `view_as sha any any from hand_equip exactly 2` | 部分迁移（额外闪需求缺） |
| nuhou 怒吼 | 受伤 optional → 弃1+他人1伤 | 部分迁移（随机弃目标牌缺） |

**⚠️ `<=` 运算符确认不存在**——liechao 首版 body `hand_count <= hp` 触发
parse error "expected expression, got Assign"（与 `!=` 同族）；以
`hand_count(self) < hp(self) + 1` 表达 ≤。**运算符清单定案：==/</>/+
可用；!=/<= 不在语言中**。turn_over(player) 与 skip_next_discard_phase
均为既有原语，liechao 三件套（摸4+翻面+跳弃牌）零新原语成套。
lianzhan 复用 storage 首次门槛（forced 分支形态：if/else 双臂，首伤摸2、
次伤上限+1回复——与 qingtuan/txianqu 同模式扩展到 else 分支）。
Golden 47/47；全量回归 985 二进制 0 失败。块轴 noop 3154→3150
（shahai_meta 888→884）。

## 本轮：短段纯效果批（2026-08-22 第四十二批）

5 技落地（shahai 657 不变，triggers 123→126、modifiers 926→923）：

| 技能 | 实现 | 精度 |
|---|---|---|
| qingliu 清流 | `damage_taken_cancel(fire) override 1` | 精确 |
| chilongya 赤龙牙 | `damage_amount(fire) add 1` | 近似（notLink 排除缺） |
| diychanyuan 缠怨 | `death owner_is_target_source_alive` forced → 击杀者上限-1 | 精确 |
| shengdun 圣盾 | `phase_prepare` forced + 无护甲守卫 → 护甲+1 | 精确 |
| lingyue 凌月 | `card_targeted owner_source_sha` optional → 弃目标1张 | 近似（目标自选→技能方选） |

短段（≤24 字）纯效果扫描开采：火属性防伤/增伤均有现成 modifier 选择器
（damage_taken_cancel(fire) / damage_amount(fire)，clbshuiyong/lxzhuixi 先例）；
「防锦囊伤害」仍缺（damage_taken_cancel 选择器仅 any/normal/fire/thunder，
无 trick——与锦囊伤害触发缺口同族维持登记）。死亡惩罚模板
（death + source_alive → change_max_hp(event_source,-1)）为 rejici2
过滤器首例直接复用；shengdun 的「无护甲守卫」以 mark_count(self,3)==0
body-if 表达。
剔除：qinglianxindeng（防锦囊伤——选择器缺）；hslingjian_shijianhuisu
装备对（equip_used 无 per-kind 变体，既有登记）；shihuo（任意角色火伤
旁观者过滤器缺）。
Golden 51/51；全量回归 985 二进制 0 失败。块轴 noop 3150→3145
（shahai_meta 884→879）。

## 本轮：散技第十五批（2026-08-22 第四十三批）

3 技落地（shahai 657 不变，triggers 126→128、view_as 10→11）：

| 技能 | 实现 | 精度 |
|---|---|---|
| wanjun 万钧 | `view_as nanman any card_equip from hand_equip` | 精确 |
| xfanghua 芳华 | `card_targeted owner_alive_targeted_by_other` optional + 受伤守卫 → 回复1 | 近似（红色约束缺） |
| cangchizhibi_equip5 苍鸱之臂·五 | `phase_end` optional → choose_other_add_mark(self,2,1) | 近似（toggle 单向） |

wanjun 装备转南蛮补齐 view_as 第 11 路（position "he" 对应 hand_equip 源区）；
xfanghua 与 diy_jiaoxia 同型「被指定红色牌」近似（无 per-color 被指定过滤器，
维持登记）；cangchizhibi_equip5 为族 inherit 块（equip1-4 已迁移，
本块同款补齐）。测试侧 optional-dispatch confirm 模式再次复用
（batch 38 规程）。
剔除：lxianglong（sha_dodged 仅 owner_source 方向，被闪侧无过滤器）；
hslingjian_yinmilichang_equip5（手牌空条件 modifier 不可表达 + juedou
选择器缺）；qinglonglingzhu（属性伤害 = fire OR thunder 无组合过滤器）。
Golden 53/53；全量回归 985 二进制 0 失败。块轴 noop 3145→3142
（shahai_meta 879→876）。

## 本轮：受击杀伤害过滤器立项 + 散技第十六批（2026-08-22 第四十四批）

**新过滤器 `owner_target_sha`**（Damage + owner_is_player + card_is_sha，
OwnerSourceSha 的受击镜像；5 处注册零 ABI：catalog 尾/registry/dispatch/
typecheck/lsp）。「受击杀伤害」缺口 3/3 达门槛（xingtigaizao_equip2 +
gudonggeng + jinjilengdong_equip2），本批立项并落地首消费者。

4 技落地（shahai 657 不变，triggers 128→132、modifiers 920→917）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jinjilengdong_equip1 极寒冻动·壹 | `damage owner_source_sha` optional → 目标摸2+翻面 | 精确 |
| jinjilengdong_equip2 极寒冻动·贰 | `damage owner_target_sha`（新过滤器）→ 来源摸2+翻面 | 近似（来源存活守卫） |
| xuanyan 玄炎 | `damage_amount(fire) add 1` + 火伤后自失1 | 近似（notLink 缺） |
| rexue 热血 | `phase_prepare owner_alive` optional → 虚拟杀+摸1 | 近似（目标/摸牌联动） |

damage_amount(fire) 在合成 DealDamage 管线即生效（与 sha_damage 需
PlayCard 管线不同——前者走 prepare_damage 的 nature 路径）。
遗留：「受击杀伤害」族另两例（xingtigaizao_equip2 摸1、gudonggeng
杀限定升级）可后续批次用新过滤器补齐/收紧。
Golden 56/56；全量回归 985 二进制 0 失败。块轴 noop 3142→3138
（shahai_meta 876→872）。

## 本轮：owner_target_sha 遗留消费者补齐（2026-08-22 第四十五批）

2 块（shahai 657 不变，triggers 132→133）：

| 技能 | 实现 | 精度 |
|---|---|---|
| xingtigaizao_equip2 形体改造·受 | `damage owner_target_sha` forced → 摸1 | 精确 |
| gudonggeng 骨董羹 | `damage_before owner_is_target` **收紧为 owner_target_sha** | 精确度提升（杀限定还原） |

gudonggeng 从批 36 的「任意下一次伤害均减 1」近似收紧为上游精确的
杀限定（一次性图腾模板 + sha 门控）；测试同步：批 36 旧测试的
DealDamage 驱动改为 DealShaDamage（收紧后非杀伤害须走不触发分支——
新测试双向断言：非杀不减技能保留、杀伤减至 0 自移除）。
**收紧既有近似时必须回改旧测试的驱动事件**——行为语义变化会使同
驱动的旧断言失败，属预期而非回归。
「受击杀伤害」族三例全部收口（jinjilengdong_equip2 / xingtigaizao_equip2 /
gudonggeng）。
Golden 59/59；全量回归 985 二进制 0 失败。块轴 noop 3138→3137
（shahai_meta 872→871）。

## 本轮：per-color 被指定过滤器立项 + 散技第十七批（2026-08-22 第四十六批）

**新过滤器 `owner_alive_targeted_by_other_red_card`**（镜像既有 black 变体，
CardTargeted + owner_is_player + 来源非己 + card_is_red；5 处注册零 ABI）。
per-color 被指定缺口 3/3 达门槛（diy_jiaoxia 红 / xfanghua 红 /
ningshuang 黑——黑用既有 `owner_alive_targeted_by_other_black_card`）。

3 块（shahai 657 不变，triggers 133→134）：

| 技能 | 实现 | 精度 |
|---|---|---|
| diy_jiaoxia 郊夏 | 过滤器收紧为 red_card 变体 | 近似→精确 |
| xfanghua 芳华 | 同上收紧 | 近似→精确 |
| ningshuang 凝霜 | `card_targeted ..._black_card` optional → 弃1+横置来源+摸1 | 近似（分支固定主路） |

行为测试双向断言红黑判别（黑桃指定 dispatch 空、红桃指定触发回复）。
批 43 的 xfanghua publish 断言同步收紧（收紧回改旧测试规程第二次复现——
publish 断言也会过期）。
Golden 61/61；全量回归 985 二进制 0 失败。块轴 noop 3137→3136
（shahai_meta 871→870）。

## 本轮：累计计数器族立项消化（2026-08-22 第四十七批）

3 技落地（shahai 657 不变，triggers 134→137、modifiers 913）——
**累计计数器族（feng/song/ya，3/3）整族收口**：

| 技能 | 实现 | 精度 |
|---|---|---|
| song 颂 | `damage_before owner_is_source` + storage 计数分支 → 第3伤+1后清零 | 精确 |
| ya 雅 | `damage_before owner_is_target` 同款 → 第3伤-1后清零 | 精确 |
| feng 风 | `draw_count add owner_skill_mark` 武装 + cards_drawn 计数状态机 | 近似（固定摸牌路径） |

**原「消耗时清零原语缺」登记撤销**——storage_set(key, 0) 即清零、
storage_get 可入表达式（`storage_get+1` 自增），单触发 if/else 分支
完整复刻上游 damageBegin 计数逻辑，无需新原语。feng 因无 drawBefore
事件改用「两次后武装 mark → 下一次摸牌吃 modifier → 摸后解除」
等效状态机（tuji 的 owner_skill_mark 值源复用）。
def 手抄坑第十三次（YA 常量丢位 147_589_044→1_475_890_447）——
publish 断言当场拦截；**伤害计数类测试必须先拉高体力**（hp 3 时
1+1+2 打空进濒死开桃窗，断言口径被濒死吞掉）。
Golden 64/64；全量回归 985 二进制 0 失败。块轴 noop 3136→3133
（shahai_meta 870→867）。

## 本轮：锦囊伤害触发缺口复核 + 散技第十八批（2026-08-22 第四十八批）

**「锦囊伤害触发」缺口登记经复核为过期撤销**——`owner_source_trick`
过滤器（含 `card_is_trick` 别名）早已在 catalog/registry/dispatch/
typecheck 注册（自批 31 起的登记基于错误的「无过滤器」假设）。
立项前先 grep 全量过滤器面的规程第三次应验（owner_min_hp、red_card
镜像、本例）。追加注册尝试触发 E0428 重复定义，4 处回滚后确认既有
eval 臂与镜像写法完全一致。

3 技落地（shahai 657 不变，triggers 137→140）：

| 技能 | 实现 | 精度 |
|---|---|---|
| shengteng 升腾 | `damage owner_source_trick` forced → 上限+1回复1 | 精确 |
| huajing 化精 | `damage owner_source_trick` optional → 回复1+摸1 | 精确 |
| bingshuang 冰枪 | `damage owner_source_trick` optional → 目标摸2+翻面 | 近似（未翻面门槛） |

锦囊伤害测试驱动：注入锦囊手牌 + `DealCardDamage{card, card_kind:
TrickWuzhong/Juedou.tag()}`（card_kind 是过滤器命中的关键，同
owner_source_sha 的 DealShaDamage 教训）。
族内剩余：oldmalymowang（damage_amount 无 trick 选择器 + 发现半边）、
tuteng4（回合内首次锦囊伤——计数器+条件 modifier 组合）、
qinglianxindeng（受击侧 trick 取消防伤——damage_taken_cancel 选择器缺，
独立缺口 1 例）。
Golden 67/67；全量回归 985 二进制 0 失败。块轴 noop 3133→3130
（shahai_meta 867→864）。

## 本轮：散技第十九批（2026-08-22 第四十九批）

4 技落地（shahai 657 不变，triggers 140→145、actives 57→59）：

| 技能 | 实现 | 精度 |
|---|---|---|
| zuizhan 乱斩 | `card_used owner_source_sha` optional → 追加目标+摸1 | 近似（随机→选择） |
| shenqiang 神枪 | `sha_use_limit add owner_skill_mark` + 杀伤害加标记 + turn_ended 清 | 近似（退还→上限/阶段→回合） |
| zhongji 重击 | `damage_before owner_is_source` optional → 弃1+伤害+1 | 近似（黑色→任意） |
| huajin 化金 | `damage_amount add owner_skill_mark` + active 武装 + 下回合准备清 | 精确 |

「主动武装-回合初解除」模板成型（huajin）：modifier 值源挂 mark、
active 弃牌加标记、phase_prepare owner_is_target 清全部——上游
「直到下一回合开始」时点精确。shenqiang 是 tuji 标记管线的
sha_use_limit 变体（出杀上限随杀伤累计、回合末清零）。
Golden 69/69；全量回归 985 二进制 0 失败。块轴 noop 3130→3126
（shahai_meta 864→860）。

## 本轮：图腾首伤对 + 散技第二十批（2026-08-22 第五十批）

3 技落地（shahai 657 不变，triggers 145→149、view_as 11→12）：

| 技能 | 实现 | 精度 |
|---|---|---|
| tuteng4 空气图腾 | `damage_before owner_source_trick` + 回合守卫 + storage 首次 → +1 | 近似（notLink 缺） |
| tuteng6 火舌图腾 | 同款（owner_source_sha 方向） | 近似（notLink 缺） |
| huanxia 幻霞 | `view_as sha play_phase red from hand_equip` | 部分迁移（收回尾段缺） |

「回合内首次某类伤害+1」模板成型（tuteng4/6）：kind 过滤器 +
`current_phase_player()==self` body 守卫 + storage 首次门槛 +
turn_ended 清零——qingtuan/shenqiang 模式在 damage_before 增伤方向
的组合应用。图腾族 1-8 至此全部非 noop（1/3/5/7 前批、4/6 本批）。
子技能块复核结论：27 个 noop 子技块中 hslingjian 系（潜行/嘲讽/
翻面距离/体力1距离/随机装备）均依赖不可表达条件或自定义机制，
nuyan2/mohua2 同（任意伤害牌/变身）——子技分片简单面确认为零。
Golden 71/71；全量回归 985 二进制 0 失败。块轴 noop 3126→3123
（shahai_meta 860→857）。

## 本轮：主技分片第二十一批（2026-08-22 第五十一批）

3 技落地（shahai 657 不变，triggers 149→151、modifiers 905）：

| 技能 | 实现 | 精度 |
|---|---|---|
| shengyan 圣言 | `hp_recovered owner_alive` optional + 未满血守卫 + storage 每回一次 → 追加回复 | 精确 |
| xiongzi 雄姿 | `draw_count add 1` + `draw_count add owner_equip_count_half_floor`（双叠加） | 精确 |
| mtg_shuimomuxue 水没墓穴 | `hand_limit add -1` | 部分迁移（弃置≥1 摸牌尾段缺） |

「任意角色回复后追加」模板（shengyan）：owner_alive 泛化过滤器即
bystander 回复触发（同 rexue 的阶段触发发现）+ event_player 未满血
body 守卫 + storage 每回合一次。**双 draw_count 叠加**验证通过
（常量 + 值源两条 modifier 并存，clbshuijian 值源复用）。
新缺口 +1 例：「弃牌阶段弃置≥1 任意牌」（shuimomuxue 尾段——现有
at_least_two / sha 变体，无 ≥1 任意牌变体）。
测试坑：合成 HpRecovered 事件本身不改 hp（只跑触发链），追加回复
断言按 event+1 口径；resolve 后残留 pending choice 须排空（cancelled
resolve 循环）再进行第二次 dispatch。
Golden 73/73；全量回归 985 二进制 0 失败。块轴 noop 3123→3120
（shahai_meta 857→854）。

## 本轮：主技分片第二十二批·双部分迁移（2026-08-22 第五十二批）

2 技落地（shahai 657 不变，actives 59→60、modifiers 905→904）：

| 技能 | 实现 | 精度 |
|---|---|---|
| qianggu 强固 | active 弃2 + 护甲2 | 部分迁移（杀响应屏障尾段缺） |
| miedao 灭道 | `draw_count add owner_lost_hp` | 部分迁移（最小弃牌对称半边缺） |

miedao 验证「动态值源天然门控」：owner_lost_hp 满血为 0 自动失效，
与上游 hp<max filter 等价（无需条件 modifier）。主技简单面显著收窄
的信号：本批宽筛（弃牌阶段/护甲/常量 modifier/条件距离四类切面）
仅命中 2 例可部分迁移——条件距离族（翻面/体力1/手牌>体力）全灭于
conditional modifier 陷阱，卡牌素材技（石楠草/煅造）grant 侧在外部
致 partial-迁移无行为。下批若继续收窄，转向 ADR 0644 P3。
Golden 75/75；全量回归 985 二进制 0 失败。块轴 noop 3120→3118
（shahai_meta 854→852）。

## 本轮：ADR 0644 P3 首切片·意见钉死（2026-08-22 第五十三批）

**shahai 主技简单面正式挖尽**（全模板宽筛 17 命中全被已知缺口挡：
回合外取反/距离分支/AOE/自定义牌/濒死解除事件/条件 modifier）。

**P3 落地：议事意见钉死机制**（ABI 658/tag 678/Cont 274 + DebateStarted
事件 tag 46 + `owner_debate_participant` 过滤器 + GameState
debate_pinned_opinions/pending_participants/pending_initiator）：

- `pin_debate_opinions(player, self_skill)`：议事开始窗口中自选 2 张
  手牌为双意见（多张=多票），其他有手牌参与者意见随机钉死一张。
- 议事核心重构：`open_debate_window`（登记名单 → 派发 DebateStarted →
  无应答立即 finish_debate / 有选择挂起由 Cont 274 收尾）；
  `debate_groups_first_card` 优先消费钉死意见；`finish_debate` 清理并发
  DebateResolved。start_debate_all_others/start_debate_targets 均改走窗口。
- 首消费者 **jsrgxiongbao 凶暴**（jsrg 包 pin 656→658）：`trigger
  debate_started owner_debate_participant optional + hand>1 守卫 →
  pin_debate_opinions`。近似：P1 旗舰议事（chaozheng/sheju）不经窗口。

行为测试全链：shelun 激活 → DebateStarted → 凶暴 confirm → 选 2 张 →
他人随机钉死 → 结算 DebateResolved → 发起者反应级联排空 → 钉死清理。
踩坑：①首版 registry 脚本 assert 失败静默半写（map_event 漏
debate_started → load_package 报 unknown event）——**多步注册脚本每步
独立验证**；②def 手抄坑第十四次（SHELUN 常量错）——skill not
activatable 定位。
Golden：jsrg 冻结套件（ABI 658 + 计数 26→27 ×2 处）+ P3 新测试 2/2；
全量回归 986 二进制 0 失败。shahai noop 852 不变（本批无 shahai 块）。

## 本轮：mobile 误标池首挖（2026-08-22 第五十四批）

2 技落地（mobile 652 不变，view_as 7→8、modifiers 247→248）：

| 技能 | 实现 | 精度 |
|---|---|---|
| rw_lanyinjia 精银甲 | `view_as shan any any from hand` | 精确（装备侧锁不迁移） |
| rw_minguangkai_cancel 耀光铠 | `card_target_cancel(sha_fire/huogong)` + `link_loss_cancel` | 近似（火烧连营选择器缺） |

误标池复核结论：宽筛仅 3 例可表达（meiyong 因「获得后其摸牌」双段
同目标联动不可拆分而缓）——**「保真度债」池主体依赖未建原语**，
池子选型面与 shahai 主技同型收窄。P3 剩余项（bazheng/yaoyan）核上游
确认需逐对伤害历史查询与 addTempSkill 延迟——登记为 P3 深水区。
发现：`view_as ... from hand` 的 "hand" 为默认源区序列化省略（断言
用 is_none_or）；mobile 冻结计数（mbkuili modifier 247→248/view_as
7→8）随部署同步。
Golden 76/76（shahai 批文件兼作跨包发布断言载体）；全量回归
986 二进制 0 失败（4018 测试）。

## 本轮：ADR 0644 P3 第二切片·意见对齐（2026-08-22 第五十五批）

**新原语 `align_debate_opinions`（ABI 659/tag 679，同步）** + 逐源受伤
座位位图 turn flag（DAMAGED_BY_PLAYER_BASE=2500，commit_damage 记录）+
ContentRegistry opt-in 门控（module_uses_effect_tag(ALIGN_DEBATE_OPINIONS)）。

消费者 **jsrgbazheng 霸政**（jsrg 658→659）：`trigger debate_started
owner_debate_participant` forced → align——本回合受到过我伤害的参与者
意见钉死为我的意见牌（上游 fixedResult 改写的票色等价）。

**⚠️ 状态写入必须 opt-in 门控**：位图首版在 commit_damage 无条件写入，
导致未使用该功能的包 canonical state hash 全局漂移（golden vectors +
olsishe 4 测试红）——按 DAMAGE_TAKEN_COUNT（ABI 347）先例改为
module_uses_effect_tag 门控后恢复。**伤害/回合热路径加状态前先查
tracks_* 先例**。位图首版还误嵌在 damaged_other（当前回合角色专用）
门内——逐源语义须挂 source.is_some() 通用路径。
测试：合成 DebateStarted 派发须手动补 debate_pending_* 窗口前置状态
（open_debate_window 在真实路径设置）。
P3 golden 4/4（bazheng 对齐行为：受伤者意见=霸政者首张手牌）；
全量回归 986 二进制 0 失败（4020 测试）。

## 本轮：跨包池审计 + olchaozheng 部分迁移（2026-08-22 第五十六批）

**跨包小批选型面审计结论**：sp TODO 池双轮扫描零模板命中（jianshu 头描述
与 noop 注记冲突经上游复核为同名变体陷阱——huicui jianshu 是拼点技，
sp jianshu 确为标记型）；jsrg 剩余=【影】标记族（fumou/funi/jiaohao/
jinfa/xuchong/xushi，需自定义牌基建）+ 延迟机制（yaoyan/chushi）——
均 ADR 级；mobile/shahai 已收口。**散技小批时代跨包结束**，
后续批次须以机制族立项（影族/延迟族/tw TODO 14）推进。

1 技落地（onlyOL 569→653——P1 旗舰原语跨包复用抬升）：

| 技能 | 实现 | 精度 |
|---|---|---|
| olchaozheng 朝争 | `phase_prepare` optional → resolve_jsrgchaozheng（P1 原语复用） | 部分迁移（意见组摸牌尾段+子技缺） |

onlyOL 冻结断言同步 8 文件（ABI 653 + 计数 22→23/167/25——
计数同步以 npack 真值表驱动一次性完成）。上游复核确认
olchaozheng inherit jsrgchaozheng（基座议事分支一致），P1 原语
直接复用即得主体行为。
Golden：P3 测试文件 +1（olchaozheng 契约）；全量回归 986 二进制
0 失败（4021 测试）。

## 本轮：jsrg【影】标记族切片 1（2026-08-22 第五十七批）

**影牌基建落地**：`CardKind::Ying = 53`（尾部追加，序列化安全）+
新原语 `gain_shadow_cards(player, count, self_skill)`（ABI 660/tag 680）——
生成 count 张影牌（黑桃 1 虚拟基本牌）置入手牌。影牌不可直接使用
（play 响应/出牌匹配臂均为拒绝/no-op；engine-ai 合法性空臂）。

消费者 **jsrgjiaohao ②**（jsrg 659→660）：`phase_prepare` forced →
`empty_equip_slots` 无除法 if 链（>4→3 / >2→2 / >0→1）= ceil(空栏/2)
（含宝物栏共 5）→ gain_shadow_cards。①段（他人装备置入）维持登记。
影族后续消费者（fumou 影当出其不意需 chuqibuyi 牌种+view_as、
xuchong/xushi 多目标给予）待切片 2。

**新 CardKind 连锁**：6 处 match 补 Ying 臂（play×2/projection/
query×2/engine-ai）——**加 CardKind 时全 workspace 枚举 match 均需
补臂**，各臂语义按「不可用/不可响应/无固定伤害」保守取值。
P3 golden 6/6（jiaohao 空栏 5 → 影 3 张，牌种断言 Ying）；
全量回归 986 二进制 0 失败（4022 测试）。

## 本轮：jsrg【影】标记族切片 2（2026-08-22 第五十八批）

2 技落地（jsrg 660 不变，triggers 29→30、actives 5→6——零新原语，
复用切片 1 的 gain_shadow_cards）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgxuchong 虚宠 | `card_targeted ..._by_other` optional → 摸1+影1 | 部分迁移（⒉分支跨目标临时 modifier 缺，取上游 AI 默认分支） |
| jsrgxushi 虚势 | active 交一张+影2 | 部分迁移（任意名配对给予→单目标单张） |

影族消费面状态：jiaohao② ✓ / xuchong ✓(部分) / xushi ✓(部分)；
fumou（影当出其不意——需 TrickChuqibuyi 牌种+完整锦囊管线+view_as，
独立切片）/ funi（影分发+弃牌堆联动）/ jinfa（议事结果比色+变更势力）
维持登记。
P3 golden 8/8（xuchong 被指定 → 摸1+影1，牌种 Ying 断言）；
全量回归 986 二进制 0 失败（4024 测试）。

## 本轮：jsrg【影】标记族切片 3·出其不意牌种（2026-08-22 第五十九批）

**TrickChuqibuyi（出其不意）完整牌种管线落地**（CardKind=54 尾部追加，
零新 ABI——牌种+view_as 均零 ABI）：
- 出牌臂（单目标：其他存活有手牌角色）+ use_card_then_wuxie
  （trick_effect::CHUQIBUYI=29）+ 目标选择臂 + AI 合法性臂 +
  projection/view tag + query 伤害（条件伤害 fixed=0）+ is_trick 列表。
- 结算：展示目标首张手牌（确定性近似，上游玩家可见选牌）→ 与源牌花色
  比较 → 不同则 DealDamage 1。
- view_as 契约：VIEW_AS_KINDS +"chuqibuyi" + map_as_kind + 新过滤器
  `card_ying`（CardKind::Ying）。

消费者 **jsrgfumou 复谋**（部分迁移，jsrg 660 不变——零 ABI 红利）：
`view_as chuqibuyi play_phase card_ying from hand`（影当出其不意）。
未建模：意见不同者颜色禁打、目标限定意见不同、魏势力限定。

影族消费面：jiaohao②✓/xuchong✓/xushi✓/fumou✓(view_as 部分)；
剩 funi（影分发+弃牌堆联动）、jinfa（比色+变更势力）。
**牌种全管线模板定案**：CardKind→from_tag→is_trick→play 出牌臂→
target spec→trick_effect 结算臂→AI→projection→view tag→query×2→
VIEW_AS_KINDS→map_as_kind（12 处）。
P3 golden 10/10（含出其不意 PlayCard→异花色→1 伤全管线）；
全量回归 986 二进制 0 失败（4026 测试）。

## 本轮：影族切片 4 复核 + ADR 0645 立项（2026-08-22 第六十批）

影族 funi/jinfa 核上游确认阻塞：funi② 需逐目标数量分配盘
（chooseButton 共计X张）、jinfa 需比色分支+变更势力原语；缺口冲刺
复核两缺口均未达 3/3（装备失牌净例 2、被闪侧 1）；tw 池 95 文件
抽样核验同为深水区（娉婷=置于武将牌+衍生授技、铸尘=距离标记非给予）。

**ADR 0645 立项**：固化剩余阻塞清单（影族切片 4+/延迟议事族/
意见牌快照/tw 深水区五类）+ 每项原语设计草案 + 8 项缺口台账 +
纪律重申（热路径 opt-in/3-3 门槛/零 ABI 面）。散技小批时代正式
让位于机制族切片章程。
文档零代码改动；全量回归维持 986/0（4026）基线。

## 本轮：影族切片 4·funi ①段（2026-08-22 第六十一批）

1 段落地（jsrg 660 不变，modifiers 140→139——hand_limit noop 换
attack_range）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgfuni ①伏匿·攻 | `modifier attack_range override 0`（终值 0） | 精确（pot_zhangren 同款） |

funi ②③ 段维持 ADR 0645 §1 登记（distribute_shadow_cards 分配盘 /
弃牌堆影联动查询）；影族消费面 5/6（jiaohao②/xuchong/xushi/fumou/
funi①），仅 jinfa 全阻塞（比色+变更势力）。
P3 golden 11/11；全量回归 986 二进制 0 失败（4027 测试）。

## 本轮：势力变更原语 + jinfa 迁移（2026-08-22 第六十二批）

**新原语 `change_group(player, group, self_skill)`（ABI 661/tag 681）**：
写入 group_overrides（既有势力覆盖机制）——4+ 消费者（jinfa/延祚/邀虎/
挟雄）达门槛。势力以 faction 枚举码传递（wei/shu/wu/qun/jin → 0..4，
codegen 映射，VM 反解；Value 为 i32 无字符串栈）。

消费者 **jsrgjinfa 矜伐**（jsrg 660→661，triggers 30→32、actives 6→7）：
active 展示一张（弃1确定性近似）+ start_debate_all_others；两个
debate_resolved 分支触发——黑=获 2 影（gain_shadow_cards 复用）、
红=change_group(self, wei)（比色分支以结果色替代展示牌色为近似）。
未建模：展示牌颜色 vs 结果比色（需 DebateResolved 带展示牌色）、
「至多两名摸至上限」、无意见相同者才变更。

影族/势力族消费面：影 5/6（仅 funi②③ 阻塞）+ 势力 1/4（jinfa 部分）。
P3 golden 13/13（change_group 行为：ActivateSkill → 势力覆盖为 wei）；
全量回归 986 二进制 0 失败（4029 测试）。

## 本轮：影族 funi② 分配盘（2026-08-22 第六十三批）

**新原语 `distribute_shadow_cards(player, count, self_skill)`（ABI 662/
tag 682/Cont 275）**：选 1..N 角色，共计 count 张影均分给予（余数给首
目标）——funi② 逐目标分配盘的均分近似。

消费者 **jsrgfuni ②段**（jsrg 661→662，triggers 32→33）：
round_started → `distribute_shadow_cards(self, ceil(alive/2))`
（ceil 以 > 阈值链表达——`/`、`>=` 运算符均不在 DSL 语言中，
已实证：0x2f/0x3d 报 lex/parse error）。

**影族消费面 6/6 全部落地**（jiaohao②/xuchong/xushi/fumou/funi①/②），
仅 funi③（弃牌堆影联动——弃牌堆 kind 查询 + 组合 modifier）维持登记。

P3 golden 14/14（funi 分发：3 人局 2 影分 2 目标各 1，牌种 Ying 断言；
forced 触发 dispatch 返回效果列表须 run_effects 后断言挂起——yongsi2
教训复刻）；全量回归 986 二进制 0 失败（4030 测试）。

## 本轮：延迟议事族（2026-08-22 第六十四批）

**注册-触发管线落地**（ABI 663/tag 683+684/Cont 276）：
- `register_turn_end_debate(player, skill)`：准备阶段选参与者登记为回合末
  议事名单（GameState.pending_turn_end_debates）。
- `resolve_turn_end_debates(player, skill)`：回合末对名单发起议事
  （open_debate_window 复用）。

消费者 **jsrgyaoyan 邀宴**（jsrg 662→663，triggers 33→35）：
phase_prepare optional → register + phase_end optional → resolve。
近似：逐角色 chooseBool 同意→发起者一次多选；红/黑分支尾段
（获未议事者手牌 / 对议事者 2 伤）待未议事集合原语。

**ADR 0645 延迟议事族首例落地**；影族 6/6 + 势力族 1/4 + 延迟族 1/2
（chushi 循环议事仍登记）。
P3 golden 15/15；全量回归 986 二进制 0 失败（4031 测试）。

## 本轮：funi③ 弃牌堆影联动（2026-08-22 第六十五批）

**新过滤器 `owner_cards_moved_contain_ying`**（零 ABI，5 处注册）：
CardsMoved + to_zone 为弃牌堆 + 含 CardKind::Ying。

消费者 **jsrgfuni ③段**（jsrg 663 不变，triggers 35→37）：
`cards_moved ..._contain_ying` forced → storage 置 1（本回合缓冲触发）；
turn_ended 清零。近似：缓冲「本回合使用牌无距离限制且不可响应」的
directHit 注入未建模（useCard1 阶段注入缺），以 storage 门控占位。
**影族 6/6 段级全部落地**（含③段触发面；buff 强命为近似）。

P3 golden 16/16（funi 影入弃牌堆 → storage=1 行为断言）；
全量回归 986 二进制 0 失败（4032 测试）。

## 本轮：被闪侧缺口收口（2026-08-22 第六十六批）

1 技落地（shahai 657 不变，triggers 151→152，零新过滤器）：

| 技能 | 实现 | 精度 |
|---|---|---|
| lxianglong 翔龙 | `sha_dodged owner_is_target` optional → 虚拟杀 | 近似（素材未建模） |

**「sha_dodged 被闪侧」缺口经复核为假缺口撤销**——ShaDodged 事件
ctx.player=被闪者、ctx.source=出杀者，`owner_is_target` 即「我闪避了
杀」（引擎内部 `owner_is_player` 变量非 DSL 过滤器名——误写触发
unknown filter）。零新原语/过滤器迁移。
P3 golden 17/17（lxianglong 发布契约）；全量回归 986 二进制 0 失败
（4033 测试）。

## 本轮：chushi 循环议事部分迁移（2026-08-22 第六十七批）

**新过滤器 `owner_is_lord`**（零 ABI，5 处注册）——identity 主公判定
（chushi 门槛 + 主公技类通用）。

1 技落地（jsrg 663 不变，triggers 37→38、actives 7→8）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgchushi 出师 | active → start_debate_targets（选主公议事）+ 红分支各摸1 | 部分迁移 |

近似：①非主公门槛（`!` 取反不可达——DSL 无 `!` 运算符，改为可选发动
自律）；②「手牌和<7 重复议事」循环（无 while——固定单轮）；③黑分支
「本轮属性伤害+1」待轮内标记 modifier。延迟议事族 2/2 落地（yaoyan/
chushi）；ADR 0645 §2 关闭。
P3 golden 17/17；全量回归 986 二进制 0 失败（4034 测试）。

## 本轮：tw 深水区首挖·厉火②（2026-08-22 第六十八批）

1 技落地（tw 596 不变，triggers 140→141，零新原语）：

| 技能 | 实现 | 精度 |
|---|---|---|
| twlihuo2 厉火② | `card_used owner_source_fire_sha` optional → 增目标 | 近似（useCard→指定后窗口） |

**tw 深水区模板扫描首命中**：9 例模板候选（sha-use 族为主）；
厉火②以火杀过滤器（既有）+ 增目标原语（reluanzhan 同款）零新原语
迁移。⚠️ `choose_add_or_remove_current_card_target` 门禁在 card_used
触发（card_targeted 报 type error）——用 card_used 修正。
势力族复核结论：剩余消费者（wuchang ①动态势力、chijie 势力选择、
qiongtu 无懈转化）均需动态参数原语——change_group 字面量版仅 jinfa
可用，势力族暂缓。
P3 golden 18/18（lihuo2 发布契约）；全量回归 986 二进制 0 失败
（4035 测试）。

## 本轮：势力族动态选择原语（2026-08-22 第六十九批）

**新原语 `choose_group(player, self_skill)`（ABI 664/tag 685/Cont 277）**：
选现存在且不同于自己的势力并变更（group_overrides 写入）——chijie/
延祚/wuchang 动态势力族的接入口。

消费者 **chijie 持节**（tw 596→664，triggers 141→142）：
`round_started owner_alive` optional → choose_group（enterGame 时机以
首轮 round_started 覆盖）。**势力族动态参数缺口关闭**（change_group
字面量版 + choose_group 动态版齐备）。

P3 golden 19/19（chijie 发布契约）；tw 冻结 ABI 断言 3 文件 596→664
同步；全量回归 986 二进制 0 失败（4036 测试）。

## 本轮：jsrg 判定区弃置（2026-08-22 第七十批）

1 技落地（jsrg 663 不变，triggers 38→39，零新原语）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgfeiyang 飞扬 | `phase_judge owner_is_target_judge_nonempty` optional → 弃2+弃判定区 | 近似（全部→一张） |

olfeiyang 双原语（discard_hand_equip_count + discard_all_own_judge）
跨包复用——判定区弃置族模板确认。jsrg 剩余 noop 复核：多数为
蓄谋/访客/城/猎 等自定义状态系统（ADR 级），可表达浅面继续收窄。
P3 golden 20/20（feiyang 发布契约）；全量回归 986 二进制 0 失败
（4037 测试）。

## 本轮：跨包浅面终审（2026-08-26 第七十一批）

jsrg 剩余 noop 复核（turn-end/phase 模板零命中——主体为蓄谋/访客/城/猎
自定义状态系统）；ddd/huanle/jiange/shenhua/yxs/xianding/sb 小包
跨包模板宽筛仅幽燕（youyan）命中但需花色差集+牌堆搜索（组合级）。
**全包浅面终审结论：可零/低原语迁移的散技块已全域挖尽**——后续仅
机制族立项（状态系统/组合原语）可推进，与 ADR 0645 章程一致。
全仓 noop 标记 3101（块轴）；ledger 5050/483/0/2059（文件轴）。
文档零代码改动；全量回归维持 986/0（4037）基线。

## 本轮：机制族立项轮（2026-08-26 第七十二批）

全仓最终宽筛（13 包 × round-prep/dying 模板）确认散技浅面零新增——
命中均为组合级（镇军动态张数、拒夏状态获得、令法轮次分支、魂游阶段
插入）。**机制族立项作为唯一推进路径**固化：下一批以自定义状态系统
（jsrg 蓄谋/访客/城/猎——GameState 级）或组合原语（花色差集/牌堆
搜索/阶段插入）为立项对象，按 ADR 0645 章程逐族切片。
文档零代码改动；全量回归维持 986/0（4037）基线。

## 本轮：蓄谋族切片 1（2026-08-26 第七十三批）

**新原语 `place_xumou(player, card, self_skill)`（ABI 665/tag 686）**：
将一张牌作为蓄谋牌置入判定区（特殊延时虚拟牌，可重复——上游
`addJudge({name:"xumou_jsrg"})` 等价；kind 用 Other 虚拟牌暂存原牌
def/suit/rank）。
- 切片 1 范围：仅置牌；「蓄谋牌结算=视为使用原牌」待切片 2
  （判定区虚拟牌结算管线——上游 chooseUseTarget 交互）。
- 消费者 jsrgeqian① 等需切片 2（选手牌选择盘）才完整——本批原语
  先落地，行为测试以 direct EffectRequest 驱动置牌验证。

P3 golden 21/21（place_xumou 判定区置牌断言）；全量回归 986 二进制
0 失败（4038 测试）。

## 本轮：蓄谋族切片 2·判定结算（2026-08-26 第七十四批）

**XUMOU_TAG 卡牌标记 + 判定结算视为使用**：
- `content::XUMOU_TAG` 常量；place_xumou 置牌时打 tag。
- `apply_judge_result` 的 Other 臂检查 tag → 移弃牌堆 + EmitRuleEvent
  code 55（视为使用原牌的原 kind 透传）——上游
  `get.autoViewAs(event.cards[0]) chooseUseTarget` 的引擎侧结算挂点。
- 消费者侧视为使用（选择目标交互）待切片 3；本切片完成判定结算
  触发面（tag 驱动 + 事件透传）。

P3 golden 22/22（蓄谋牌 XUMOU_TAG 断言）；全量回归 986 二进制 0 失败
（4039 测试）。

## 本轮：蓄谋族切片 3·首消费者（2026-08-26 第七十五批）

1 技落地（jsrg 663 不变，triggers 39→40，零新原语）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgfangjie 芳洁① | `phase_prepare` optional → 回复1+摸1 | 近似（无蓄谋牌分支自律化） |

蓄谋族切片 3 首消费者：芳洁① 以纯模板迁移（判定区状态取反不可达，
可选发动自律）；②段（弃蓄谋牌+失去技能）待选择盘。剩余消费者
（eqian①/daimou）均需选择盘+牌堆顶读取——切片 3b 立项。
P3 golden 23/23（芳洁发布契约）；全量回归 986 二进制 0 失败（4040
测试）。

## 本轮：蓄谋族切片 3b·殆谋+遏前①（2026-08-26 第七十六批）

**新原语 3 个（ABI 665→666，tag 687/688/689，全部真实移动语义）**：
- `place_deck_top_xumou(player, self_skill)`（tag 687）：牌堆顶实体牌移入判定区
  作蓄谋牌——与切片 1 的 place_xumou（虚拟复制、原牌不动）不同，本原语将物理牌
  自原区摘除（DrawPile→Judge），杜绝重复触发复制同一顶牌。
- `choose_place_hand_xumou(player, self_skill)`（tag 688/Cont 278
  ChoosePlaceHandXumou）：SelectCards 选一张手牌移入判定区；cancel=停止蓄谋。
- `discard_one_own_xumou(player, self_skill)`（tag 689）：确定性弃置判定区最后
  一张 XUMOU_TAG 牌（上游 chooseButton 强制选一张 → 确定性近似）。

**新 TriggerFilter 2 个（5 处注册）**：
`owner_alive_other_targeted_by_sha`（CardTargeted + 杀 + 目标≠owner）；
`owner_alive_targeted_by_other_sha_judge_xumou`（他人杀指定 owner 且判定区有
蓄谋牌——filter 内置 hasCard 前置，保证「我」分支门槛不被空耗）。

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgdaimou 殆谋 | `card_targeted` 双分支 optional/forced + storage 双 key 每回合各一次（turn_ended 清零近似 addTempSkill） | 近似（弃置无选择盘；isDisabledJudge 未建模） |
| jsrgeqian 遏前① | `phase_end owner_is_target_hand_nonempty optional` → choose_place_hand_xumou | 近似（while 循环任意次→单次激活；②「杀或蓄谋牌唯一目标」待切片 4 使用管线） |

消费者侧语法注记：`turn_ended` ← PhaseEnded{phase: End}；`phase_end` ←
TurnStateChanged{phase: End}——两名字一字之差映射两个事件源，测试驱动时勿混。
ABI 领号 ABI-COORDINATION.md #666（含两过滤器）；host RULE_VM_ABI_VERSION
665→666。P3 golden 27/27（新增 4 断言：deck-top 真实移动/殆谋每回合门槛+
清零/殆谋 forced 弃蓄谋/遏前①选牌+cancel）；全量回归 workspace exit 0、
4216 通过/0 失败（含本批新增 4 断言；回归日志聚合 grep FAILED 复核为 0）。

## 本轮：jsrg 状态系统三族立项 ADR 0646（2026-08-26 第七十七批·设计批，无代码）

蓄谋族自然闭环认定：上游 addJudge("xumou_jsrg") 全部置入点仅 3 处，eqian①/daimou
已于第七十六批迁移；cangxiong（双复合过滤器养一技）与 xumou_jsrg_temp（按牌名
cardEnabled 去重名册设施）均不达 3/3 立项门槛。实际实现路线（判定区+XUMOU_TAG）
与 ADR 0645 §5b 原草案（ExpansionMap+query 合并）分歧已加注留痕。

**新章程 docs/adr/0646-jsrg-cheng-ye-lie-state-systems.md**：
- 【猎】标记族（政略/平戎/会猎，最浅）：跨角色 add_mark/remove_mark 表达式目标
  已原生支持；缺口=座次过滤器、target-mark 条件豁免值源、跨存活角色标记计数
  表达式、额外回合子系统（引擎 grep 零命中——平戎核心承诺）。S1 零/低原语近似
  批为下一代码批候选。
- 【城】扩张堆族（沙城/凝寒，中）：需牌堆顶→expansion 变体 + 扩张区选择续体 +
  本回合失去牌数历史查询 + per-color 全局 view_as（poyue 同登记缺口）。
- 【访客】武将牌族（yingmen/pingjian×sb，深水）：characterlist 抽样池/伪卡生成/
  invisible-skill 触发委托达引擎级新子系统，登记永久高温债不排期。
INDEX.md 增 0646 行；全量回归维持 4216/0 基线（零代码改动）。

## 本轮：猎标记族 S1（2026-08-26 第七十八批，ADR 0646 §1，jsrg ABI 666 不变）

3 技零新 Effect 原语迁移（新 TriggerFilter ×4，均 5 处注册 + turn_limit 登记）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgzhenglve 政略① | `phase_end owner_alive_event_player_lord optional` → draw 1 + choose_other_add_mark(+1) | 近似（固定授一；不查已有标） |
| jsrgzhenglve 政略③ | `damage owner_source_damage_target_mark_positive_<mark>_once optional` → draw 1 + gain_event_cards | 精确（once=turn_fire_count） |
| jsrghuilie 会猎 | `phase_prepare owner_alive_marked_alive_above_two_<mark>` forced → change_max_hp(-1)+add_skill 平戎/飞影(跨包字面量)+remove_skill 自移除 | 精确（wuji 觉醒模板） |
| jsrgpingrong 平戎 | `phase_end owner_alive_someone_mark_positive_<mark> optional` + storage 每轮门槛 → choose_other_add_mark(-1) 移猎 | 部分（移猎本体精确；额外回合+失体力 check 待 S3） |

工程要点：
- **「猎」def 合成铸造** blake2b("jsrgzhenglve_mark",4B)=1167524380（pxlujiang_win 工艺）。
- **choose_other_add_mark 负 delta 双补**：primitive 候选仅列现有计数>0 的目标；
  resolver 把负 delta 钳到 max(delta, -current)——正 delta 消费者行为不变。
- **坑：`_once` 类过滤器必须同时在 dispatch.rs `turn_limit()` 名单登记**，
  否则 `turn_fires` 恒 0、每回合不限一次（本批 zhenglve③ 测试当场抓获）。
- npack：modules 52→57、trigger_index 44→49（zuwang/zhaohan 冻结断言同步）；
  新测试 original_jsrg_lie_family 5/5。全量回归 workspace exit 0、4221 通过/0
  失败（基线 4216+新增 5；grep FAILED 复核为 0）。ledger：noop 2056→2053、
  approx 486→488、complete 5047→5045（会猎无真实缺口归完整）。

## 本轮：城族 S2a·筑城（2026-08-26 第七十九批，ADR 0646 §2，jsrg ABI 666→667）

1 技落地 + 新原语 `place_deck_top_expansion(player, count, self_skill)`
（tag 690，领号 #667 记 ABI-COORDINATION.md；扩张堆按需创建、以技能 def 键控、
`usable_as_hand:false`——bingshi skill_piles 同工艺）：

| 技能 | 实现 | 精度 |
|---|---|---|
| jsrgshacheng 沙城①筑城 | `trigger game_started owner_alive`（forced，enterGame 等价）→ 堆顶两张明置入「城」 | 精确 |

近似留债：②「移一城令目标摸X（本回合失去过的牌数≤5）」需 per-player lose
计数 + 城牌/目标双选续体 → S2b；凝寒（♣全局改色+冰冻 nature）整体引擎级债。
注册链 15 处照 AJ 模板；vm pop 序 player→count→skill（逆序）。测试
original_jsrg_city_build 2/2（契约+堆顶实体移动/面朝上/归属）。冻结断言扫描
9 文件 666→667；triggers 49→50、modules 57→58。全量回归 workspace exit 0、
4223 通过/0 失败（grep FAILED 复核为 0）。ledger：noop 2053→2052、
approx 488→489、complete 5045→5044。

## 本轮：城族 S2b·沙城消耗（2026-08-26 第八十批，ADR 0646 §2，jsrg ABI 667→668）

沙城②升级为真实语义（领号 #668；tag 691/Cont 279）：

| 组件 | 实现 |
|---|---|
| 本回合失去牌计数 | `SkillPileRef.wants_lost_count`（serde default false，字节稳定）+ turn_flag 基址 3000 keyed 计数 + `move_cards_with_reason` 出口钩子：非 use/respond、去向非自身/入共享堆时自增——Match 内存在任一 wants 堆才计数（首次版本误按「离开者自有堆」门控致计数恒零，测试当场抓获后改为 Match 级门控） |
| city_consume 两阶段 | guard（城非空+存在失去者）→ SelectTargets（合格目标）→ resolver 移去城堆最后一张入弃牌堆 + DrawCards target min(5, lost) |
| 新过滤器 | `owner_alive_sha_resolved_city_<def>`（ShaResolved+有城+有人失牌），5 处注册+参数化校验+lsp 分支 |

| 技能 | 精度 |
|---|---|
| jsrgshacheng 沙城② | 近似精确化：目标域=任一本回合失去过牌的其他角色（上游限此杀目标）；城牌取最后一张（确定性）；cap 5 精确 |

测试 original_jsrg_city_build 5/5（新增契约/消耗链+上限/无城无失牌静默三例；
真实弃置驱动计数）。冻结断言扫描 10 文件 667→668、triggers 50→51、modules
58→59。全量回归 workspace exit 0、4226 通过/0 失败（grep FAILED 复核为 0）。
ledger 数值维持 complete=5044/approx=489/noop=2052（沙城仍按近似轴记账）。

## 本轮：猎族 S3·额外回合（2026-08-26 第八十一批，ADR 0646 §1，jsrg ABI 668→669）

平戎「移猎后执行一个额外回合+尾随失体力」完整落地。**关键更正**：「引擎无
ExtraTurn 子系统」为侦察误判——`turn.extra_turn_queue` 消费管线早已完备
（continue_turn_advance 三段队列：draw/play/turn），缺的只是通用生产端 DSL 原语。

| 组件 | 实现 |
|---|---|
| queue_extra_turn | tag 692/ABI 669 领号；guard(cap<8、alive)→入队+EXTRA_TURN_CHECK_PENDING[key=受益人]=1 |
| check 结算 | continue_turn_advance 头部：若 current 带 EXTRA_TURN_ACTIVE 标记→读回合级 DAMAGE_DEALT，零则 DealDamage(is_lose_hp) 失 1 体力，随后清标记 |
| pop 分支拼接 | 消费队列时 PENDING→ACTIVE（存活于 reset_turn_state 白名单式清理之外） |

| 技能 | 精度 |
|---|---|
| jsrgpingrong 平戎 | 完整（近似注记仅余：挂起回合死亡等引擎默认顺序；check 读聚合 DAMAGE_DEALT 而非逐事件 sourceDamage） |

测试 lie_family 6/6（+额外回合入队/PENDING 断言+两态 settlement 对照：伤害=0
掉血 vs =2 不掉）；冻结 ABI 扫描 9 文件 668→669（modules/triggers 不变 59/51）。全量回归 workspace exit 0、4227 通过/0 失败
（grep FAILED 复核为 0）。ledger 数值维持 complete=5044/approx=489/noop=2052
（平戎从「部分迁移」升为完整，仍留聚合口径近似注记）。

## 本轮：政略①双授分支（2026-08-26 第八十二批，ADR 0646 §1，jsrg ABI 669→670）

| 组件 | 实现 |
|---|---|
| choose_others_add_mark | tag 693/Cont 280/ABI 670 领号；`select_targets(1..max_count, cancel=true)` + resolver 逐目标 AddMark（校验存活/非己/≤max_count）；payload [player,mark,delta,max_count,skill] |
| 主公过滤器对 | `owner_alive_event_player_lord_damaged/_no_damage`——PhaseEnd×Lord×DAMAGE_DEALT 聚合 >0/==0 互斥分支 |

| 技能 | 精度 |
|---|---|
| jsrgzhenglve 政略① | 升级为完整语义：伤害过→单授 / 未伤害→至多两名一次选满。近似仅余授标不查已有标记、可放弃（上游强制选满）。遗留缺口只剩②豁免值源 |

测试 lie_family 扩至 7/7（no_damage 双授一选二；damaged 分支单授对照）。冻结 ABI
扫描 10 文件 669→670；triggers 51→52、modules 59→60。全量回归 workspace exit 0、
4228 通过/0 失败（grep FAILED 复核为 0）。ledger 数值不变（沙城/政略均在近似轴）。

## 本轮：政略②豁免值源立项（2026-08-26 第八十三批，ADR 0646 §1，jsrg ABI 670 不变）

设计结论：per-target 条件不适合 Owner 签名的 ModifierValueSource，正确挂层是
**携带 target 的既有查询钩子**（两处均已存在）。新增参数化 ModifierTarget 族：

| 组件 | 实现 |
|---|---|
| ShaUseUnlimitedTargetMark(u32) | `sha_use_unlimited_for_target` 遍历目标 marks 收集匹配 modifier（owner 过滤），值非零→该目标出杀不计数 |
| CardDistanceUnlimitedTargetMark(u32) | `card_distance_unlimited_for_target` 同型插入，命中即无视距离 |
| DSL/校验 | token `*_target_mark_<mark>`（suffix u32>0），`override 1` 强制 gate；registry 双臂序列化+解析；lsp hover 两支 |

| 技能 | 精度 |
|---|---|
| jsrgzhenglve 政略② | 升级为真实建模（两条 modifier 一行式）；近似注记收窄为①的 absence/可放弃两条 |

测试 lie_family 扩至 **8/8**：新增豁免行为测试（防具马拉开距离+授标翻转两项
断言；未标记者不受影响;兼容 hand-compare fallback 分支的容错断言）。npack
modifiers 130(+2)、modules/triggers 不变；**无 ABI 领号**（零新 VM op，token
由编译器 gate 天然门控旧引擎——与参数化过滤器先例同规）。全量回归 workspace
exit 0、4229 通过/0 失败（grep FAILED 复核为 0）。ledger 数值不变：政略升级为
「完整」待 ledger 重生成确认（当前 approx 轴仍含其②注记行，下批统一校正）。

## 本轮：跨包普查+首迁 key/umi_lunhui（2026-08-26 第八十四批，key ABI 651→669）

①账目校正：jsrgzhenglve 注记行与本批实现核对一致（approx 轴保留仅余两条真实
近似：absence 未查、可放弃），ledger 重生成确认。

②insertPhase 普查：legacy-source 全树 **57 处/26 文件**；`extra/lianpo` 经查已由
shen_simayi 多路器(action=1)迁移过——首迁改选 **key/umi_lunhui 海未·轮回**：
| 组件 | 实现 |
|---|---|
| queue_extra_turn 泛化 | 新增 check u32 形参（0/1）：check=0 纯额外回合（海未）；1 平戎尾随代价。sig/rule-api/vm pop/ir/typecheck/codegen/hash 同步；仅 jsrgpingrong 一处旧调用升级 |
| umi 体 | 双触发器：own turn_ended 清绑定 storage；other turn_ended optional→body-if hand<hp→storage 置位+lose_hp(1)+draw(2)+queue_extra_turn(check=0) |

| 技能 | 精度 |
|---|---|
| key/umi_lunhui 海未·轮回 | 近似：确认在条件不满足回合仍打开（optional 语义）；「角色绑定」以一位布尔表达 |

新测试 original_key_umi_lunhui 2/2（契约+发动/绑定窗口静默/自回合解除全链）。
冻结同步：key kamome ABI 651→669、modifiers 178→177（armor-family key 行同步）。
key npack modules 110/triggers 93/modifiers 177、ABI 669；部署一致(key 两份)。
**普查附带产出**：key 包 ABI 冻结断言散布 10 文件（不止 kamome/armor 两处）——
首轮回归 red 两例（akiko_dongcha 未扫到、collab 包合法 651 被误抬），按绑定名
逐一核对后回退 collab 行并补扫 key 文件族。全量回归 workspace exit 0、
4229+2=4231 通过/0 失败（grep FAILED 复核为 0）。ledger：complete=5043/
approx=490/noop=2051（海未入近似轴）。教训：**跨包断言扫描必须按「嵌入该包
npack 的文件」圈范围，且逐行核对 from_json_bytes 绑定变量名再替换**。

## 本轮：refresh/botu 博图·额外回合第二消费者（2026-08-26 第八十五批，refresh ABI 650→669）

1 技落地 + 新过滤器 `owner_alive_turn_used_all_suits`（5 处注册，零 ABI）：

| 组件 | 实现 |
|---|---|
| 过滤器 | PhaseEnd…注意 `turn_ended` 的 ctx.kind 是 **TurnEnded**——本批测试当场抓到误写 PhaseEnd；判定读引擎既有 `distinct_used_card_suits(owner)>=4` 位掩码查询（CardUsed 路径自动记录，包 ABI≥232 opt-in 自动启用，refresh 无需任何计数接线） |
| botu 孙皓·博图 | `turn_ended owner_alive_turn_used_all_suits optional` → queue_extra_turn(check=0)（第二跨包消费者） |

| 技能 | 精度 |
|---|---|
| refresh/botu 博图 | 完整（frequent 自动发动语义以 optional 确认近似） |

测试 original_refresh_botu 2/2（三花色静默/第四花色入队+无代价标记断言，合成
CardUsed 真实驱动位掩码）；lie_family 8/8、umi 2/2 回归。refresh 冻结 ABI 扫描
4 文件 650→669；npack modules 157/triggers 142/modifiers 212。全量回归 workspace
exit 0、4233 通过/0 失败（grep FAILED 复核为 0）。ledger：noop 2051→2050
（botu 入近似轴——frequent→optional 一条注记）。

## 本轮：offline/xianding insertPhase 不可达论证登记（2026-08-26 第八十六批·设计批，无代码）

按第八十四批普查表执行「纯形筛选或论证登记」分支：两包 23 处逐块核验完毕，
**零「body=queue_extra_turn±既有原语」纯形候选**（证据表已入 ADR 0646 普查登记）。
共性阻断面归纳三族引擎缺口：
①**亡者/绑定回合**（guansuo/jiudin/chunqiu：queue 须支持非存活实体与座位环判定）；
②**相位裁剪插相**（zj_fuli/piracyE_zy/jb 的 `phaseList=[...]`、`skip(name)`、
`_noTurnOver`——需 phase-list 级插入原语而非整回合队列）；
③**when(phaseBegin/End) 一次性钩子**（fengying/dcsbbizuo/dcwumei/guili）。
可迁近似两项立案排队：xianding fengying（觉醒+全弃手牌+自插相位；尾钩
drawTo(maxHp) 以自律化注记收口）、xianding guili（绑定子技口径注记近似）。
文档零代码改动；全量回归维持 4233/0 基线。

## 本轮：fengying 勘误 + xianding/guili 落地（2026-08-26 第八十七批，xianding ABI 669 不变）

**勘误**：上批排队候选 fengying 经查**已完成迁移**（active once_per_game +
resolve_fengying action0/1/2 多路器：全弃校验、空手短路、deferred 入队、
drawTo(maxHp) 尾钩子技 fengying_effect 实建）——登记时未查移植目录实态，
ADR 表已改判并把「三不信」扩展到自产排队表。

**新迁移：guili 归离**（额外回 合第三消费者，零新 Effect 领号）：
| 组件 | 实现 |
|---|---|
| 绑定 | 首个自身 phase_begin(storage key1 门控)→choose_other_add_mark 打合成标记 GUILI_BIND=blake2b("guili_insert")&0x7FFF_FFFF=281977830 |
| 回收过滤器 | `owner_alive_event_target_mark_positive_<mark>_no_damage`（TurnEnded×目标带标×DAMAGE_DEALT==0），5 处注册 |
| 回收 | optional confirm→storage key2 锁一次性→queue_extra_turn(check=0) |

测试 original_xianding_guili 3/3（契约/绑定→回收全链/伤害守卫静默）。**坑**：
合成 id 必须落在 i32 正区间——首铸 2429461478 超 2^31 在常量通道被钳 0
（DBG 打印 marks=[(0,1)] 当场暴露）；掩码 &0x7FFFFFFF 后复用。xianding npack
(modules 173/triggers 154/modifiers 383、ABI 669 不变) 三处部署一致；独立源
编译断言 575/578 未触碰。全量回归 workspace exit 0、4236 通过/0 失败（grep
FAILED 复核为 0）。ledger：noop 2050→2049、approx +1（guili 入近似轴）。

## 本轮：extra_offline 二次核验收口（2026-08-26 第八十八批·设计批，无代码）

挂起项复核完毕：extra_offline.js 实际仅 **1 处** `insertPhase("sm_kulian")`
（原计数 2 系扫描混计，已勘误）。该命中为苦莲「回合结算竞赛」的奖励分支：
「击杀数唯一最多 → 仅出牌阶段额外回合」，依赖 `game.getRoundHistory`
唯一最多判定与 initList 全局奖池，属第八十六批归纳的②相位裁剪 ∪ ③轮度
统计复合面——**出具不可达论证并入册 ADR 0646 普查登记表（移出挂起）**。
至此 offline 包 17 处全部定级完毕。文档零代码改动；全量回归维持
4236 通过/0 失败基线；ledger 数值不变。

## 本轮：diy/nsfuge 晋升·额外回合第四消费者（2026-08-26 第八十九批，diy ABI 478→669）

| 项 | 说明 |
|---|---|
| 迁移 | diy_meta.skill 内 nsfuge 由批量 draw-1 占位晋升为真实体：storage 自锁一次性 + queue_extra_turn(check=0)；「frequent 自动发动」以 optional 确认近似（approx 一条注记） |
| 踩坑 | roster 占位与新增 .skill 并存会报 duplicate skill def id——晋升必须**原位改写 meta 巨文件内块**而非新建同名文件（与 akane_jugu 多块巨文件同格） |
| 冻结扫描 | 4 文件 478→669（junkrende/nsyuanchou/nsqiyue/tiangong2，绑定名逐一核对）；npack modules/triggers 不变 114/96、ABI 669、三处部署一致 |
| 测试 | original_diy_nsfuge 2/2（契约；首火入队+PENDING=0+storage 自锁；闩上再确认静默且队列不增长） |
| ADR | tw 四处全数登记（renxian 引擎级 phaseList+白名单；fuxi choices 复合；jieqiu 近似可迁候选；yingjia 待细判）；diy 另三处复合登记 |

全量回归 workspace exit 0、4238 通过/0 失败（grep FAILED 复核为 0）。
ledger complete=5042/approx=491/noop=2049 数值不变——nsfuge 原占位本就计入
approx 轴，晋升后仍带 frequent→optional 一条近似注记，归类轴持平。

## 本轮：sp/sp2/mobile/onlyOL 筛迁（2026-08-26 第九十批·设计批，无代码）

四包 8 处 insertPhase 逐块核验完毕:**零纯形候选,全部复合登记**(详见 ADR 0646
普查表新增分区)——olhuiqi 需全局历史目标集合统计;oldongdao 卡转换态;
dccuichuan/yijiao 各带随机装配与控制选量盘;xinlianpo 走派生技能白名单门;
olduoqi 双 hit 同块,依赖一号位判定+全局 record 数组+broadcast。

文档零代码改动;全量回归维持 4238 通过/0 失败基线;ledger 数值不变。
insertPhase 全树剩余待核验面收窄至:clan/key/bingshi/huicui/shenhua/shiji/
extra/sxrm/old 各 1 处单点。

## 本轮：九单点包 insertPhase 核验收口（2026-08-26 第九十一批·设计批，无代码）

clan/key/bingshi/huicui/shenhua/shiji/extra/sxrm/old 各 1 处逐块核验：
- **已迁×2**：key umi_lunhui（84 批）、extra lianpo（存证 simayi 多路器）；
- **近似可迁候选×2**：bingshi mbquanchong（resolve_fengying 多路器扩 action）、
  shenhua fangquan2（立项新原语 choose_others_insert_extra_turn——SelectTargets
  选中者直接入队 extra_turn_queue，形状照 ChooseOthersAddMark；下批首选）；
- **引擎级不可达登记×5**：clan clankudu(recast+差值数学)、huicui dczecai
  (addAdditionalSkills 叠加)、shiji tamo(座次重排盘)、sxrm sxrmmieguo
  (selectButton 盘)、old oldmingjian(phase goto 跳转)。

insertPhase 全树 57 处至此**全部完成一级定级**（迁 5/候选 3/其余登记债含反查），
后续按 ADR 0646 排队逐项落地。文档零代码改动；全量回归维持 4238 通过/0 失败
基线；ledger 数值不变。

## 本轮：shenhua fangquan2 放权·目标插相（2026-08-26 第九十二批，shenhua ABI 591→671）

| 组件 | 实现 |
|---|---|
| choose_others_insert_extra_turn | tag 694/Cont 281/ABI 671 领号；SelectTargets(1..1, cancel=true, 其他存活) → resolver push extra_turn_queue；check 非零时同时登记 PENDING 尾随标记（泛化预留） |
| fangquan2 放权 | `turn_ended owner_is_target optional` → discard_hand_equip_count(1)（自带选牌屏障≈上游 chooseToDiscard）→ 目标选择续体入队。locked 与 optional 不兼容已摘除；mark/fangquan3 联动注记省略 |

测试 original_shenhua_fangquan2 2/2（契约+三段链 confirm→Custom 弃一→目标入队，
PENDING=0 断言）。冻结断言按嵌入 npack 圈定+绑定名核对后同步 **26 文件**
shenhua 591→671（compiled.* 独立源断言保留原值）。全量回归 workspace exit 0、
**4240 通过/0 失败**（grep FAILED 复核为 0）。ledger 数值不变。

## 本轮：mbquanchong 勘误与 insertPhase 收官核验（2026-08-26 第九十三批·设计批，无代码）

①**bingshi mbquanchong 勘误为已迁**：ported/bingshi/mb_chenzhi.skill 存
resolve_mbquanchong 多路器（主 trigger `owner_is_target_discardable_hand_equip_
once_per_round` 强制结束阶段全 he 弃+入队；effect 子技 phase_begin lose_hp
尾钩）——上批"近似可迁候选"判定系未查移植实态之再犯，ADO 表已改判。连带将
「fengying 多路器扩 action」立项撤销。

②twjieqiu 复核维持登记：全链需「装备栏废除/恢复+按栏数摸牌+废栏余量判定+
持有者额外回合」，equipment-slot 原语族缺口具体化，留专项评估。

③insertPhase 全树定级终局：迁 **6**（pingrong/umi/botu/nsfuge/fangquan2 +
lianpo 存证）；纯形余量 **0**；其余全部带证据登记债。文档零代码改动，
回归维持 4240 通过/0 失败基线，ledger 数值不变。

## 本轮：twjieqiu 装备栏原语族立项评估（2026-08-26 第九十四批·设计批）

另线证伪:diy_meta 内 ns* 批量占位群(nsbaiyi/nsqinxue/nsbaiming/nsxinzhan/
nstanbing/nslingying 等)上游本体皆复合(additionalSkills/type2 白名单集合),
无"纯形晋升"机会——approx 轴注记残差均真实。nsfuge 属孤例。

twjieqiu 立项评估成型(ADR 0646 更新):装备栏禁用/恢复引擎底座完备
(disabled_equip_slots 位段+disable/enable fn+双事件,pack_sp_2 同款),
仅缺编排层——立项 resolve_twjieqiu 三 action 多路器
(disable 全六槽+摸k张 / 弃牌阶段末逐槽恢复 / 回合末余栏判额外回合),
含 AddMark(k) 回收计数控、Cont 282、一个 discard-end 参数化过滤器,
领号 ABI 672。定位:下一代码批首选实现项。
文档零代码改动;回归维持 4240 通过/0 失败基线。

## 本轮：diy/nsjihui 集计·额外回合第五消费者（2026-08-26 第九十五批，diy ABI 669 不变）

| 项 | 说明 |
|---|---|
| 新过滤器 | `cards_moved_discard_at_least_three`(5处注册)：CardsMoved→to_zone==弃牌堆→move_reason==Discard→len>=3 |
| nsjihui 晋升 | diy_meta 原位改写：own turn_ended 清零 storage + `cards_moved … optional` → 自锁置位+queue_extra_turn(check=0)；近似注记三条（目标限制子技/每回合一次收窄/确认噪声） |

测试 original_diy_nsjihui 3/3（契约；4 张发动→入队+PENDING=0+storage 置位→闩上
静默队列不增长→own End 清零；2 张静默）。全量回归 workspace exit 0、
**4243 通过/0 失败**（grep FAILED 复核为 0）。ledger 数值不变：nsjihui 由批量
hand_limit 占位晋升为真实近似体，approx 轴归类经注记刷新保持。npack modules 116/triggers 98(+1+2? 实
测 114→116 modules、96→98 triggers：nsfuge 与 nsjihui 两批累计)+三处部署一致。
全量回归 workspace exit 0、4248 通过/0 失败（1037 二进制；grep FAILED 复核
为 0）。冻结断言第二轮复盘：offline/jytongbei 单文件 include_str 断言恒旧
（498 保留）、zjfuli compiled 断言为 672；nslulve active 139→138、
spfuluan triggers 576→577 等四文件批量校正。ledger：complete=5040/
approx=493/noop=2048（fangquan2/twjieqiu 注记刷新入 approx 轴）。

## 本轮：twjieqiu 立项复核撤消（2026-08-26 第九十六批·设计批，无代码）

通读上游全文推翻第九十四批评估前提：恢复节奏并非「逐槽消费计数标」，而是
「按目标本阶段实际弃牌张数，在剩余禁用栏中**选择性地**恢复等量槽」——需要
PhaseEnded{Discard} 携带本阶段弃牌集合（TriggerContext 未暴露）；另有两处
深水耦合：跨角色 buff 名单（markAuto PlayerId 列表）、目标自决 chooseBool+
used(roundStart) 门。三件合计升级为**引擎级不可达**，与 dcwumei/zj_fuli 同池
（装备栏编排子族）。resolve_twjieqiu 领号未发生、无代码残留；全量回归维持
4240 通过/0 失败基线；ledger 数值不变。

## 本轮：相位裁剪原语族立项书（2026-08-26 第九十七批·设计批，无代码）

将 zj_fuli/zylianji/dcwumei/twjieqiu/sm_kulian 五处 phaseList 类消费者归并统
一立项（ADR 0646 §6）：`turn.extra_phase_queue: Vec<(PlayerId, Vec<PhaseTag>,
bool)>` 与 extra_turn_queue 同级编排；DSL 入口 queue_extra_phases + skip_
next_named_phase 两枚。里程碑 S0 结构+S1 双消费者接线 → S2 skip 联动 →
S3 dcwumei/twjieqiu 尾部再评估。
文档零代码改动；回归维持 4240 通过/0 失败基线；ledger 数值不变。

## 本轮：§6 S0 相位裁剪队列 + zj_fuli 消费者（2026-08-26 第九十八批，offline ABI 498→672）

**S0 引擎基建**：`ExtraPhaseTurn{player, phases: Vec<u8>, no_turn_over}` +
`TurnState.extra_phase_queue`（serde default）；continue_turn_advance 新消费分支
（extra_turn_queue 之后）：存活过滤 → reset → 按列表武装 SKIP_DRAW/SKIP_PLAY/
SKIP_DISCARD 与 pending_judge_skips（Prepare 恒执行）→ SetTurnState 至列表首相位。
**新原语** `queue_extra_phases(player, phases_mask, self_skill)`（tag 695/领号
ABI 672；mask bit=1<<TurnPhaseTag，typecheck gate 0<mask≤0b0011_1110）。

| 消费者 | 实现 | 精度 |
|---|---|---|
| offline zj_fuli 界扶利 | `dying owner_is_target optional` → recover_hp(2-hp)+draw 2+queue_extra_phases(Play)；limited→storage 一次性 | 近似（濒死 enable 门自律化；无 phaseList 之外的残差） |

sm_kulian 需轮度唯一最多竞赛判定原语，仍在 §6 排队（S2/S3 随 skip 联动另批）。
offline npack：modules 715/triggers 577、ABI 672、三处部署一致；冻结 ABI 扫描
25 文件 498→672（compiled.* 独立源断言豁免核对）。测试
original_offline_zjfuli 2/2（契约+濒死自救→裁剪入位[phases=[Play]]→闩上静默）。
全量回归 workspace exit 0、**4245 通过/0 失败**（grep FAILED 复核为 0）。
**冻结断言打地鼠复盘**：offline 包 ABI/count 断言散布 30+ 文件且值型混杂
（package/committed 绑定、compiled.* 独立源、include_str 单文件独立编译三形态），
先后三轮红（jytongbei 498 独立源被误抬、nsjianglie/nslulve/spfuluan 等计数残株
+active 139→138）；终以「对账脚本按实际 npack index 长度批量校正」收敛。
教训：count 断言同步必须脚本化对账而非人工逐个。

## 本轮：技能夺取子系统改判与纯形晋升路线证伪（2026-08-28 第九十九批·设计批，无代码）

①sm_kulian 通读改判:实为全局竞赛系统(roundStart 奖池 initList/sm_mabian 装备
联动/addGlobalSkill/多维度唯一最多),§6「轮度唯一最多」轻量定位撤回;原语
不立项(干净消费者 1/3,未达门槛)。
②potyinhui/potheyun 同归「技能夺取子系统」池(addAdditionalSkills/refreshSkill/
chooseButton from 名单)——approx 轴此二条注记残差真实,非晋升对象。结合第九十
四批 diy ns* 群核验,证明 **approx 491 条中不存在隐藏纯形晋升**,该路线关闭。
ADR §7 子系统池登记;文档零代码改动;回归维持 4245 通过/0 失败基线;ledger 数值不变。

## 本轮：凝寒终判登记（2026-08-28 第一百批·设计批，无代码）

凝寒两半互锁取证:①全局♣杀改色需「全局花色改写层」(effective suit/kind 查询
面为 owner 局部,无全局挂点;per-color 全局 view-as 台账挂账两例未达门槛);
②冰冻伤害在本引擎无产生源(DamageNature 仅 normal/fire/thunder,且无已建模
ice 属性源卡),②离开①的判定即为死代码。「置入城」目标桶可由 fengying 式
扩张堆多路器承载,但触发源空缺使其无意义。§2 凝寒改判「引擎级不可达(改色层
+冰冻 nature 子系统)」,归 §7 同池。文档零代码改动;回归维持 4245 通过/0 失败
基线;ledger 数值不变。

## 本轮：§6 S2 判定与相位裁剪族收口（2026-08-28 第一百零一批·设计批，无代码）

S2 skip_remaining_phases 复核定案：干净消费者仅 zylianji 1/3（其本体需「分段
互斥触发器家族：本阶段用牌类别数 ≥1/≥2/≥3 三档 × skip 剩余阶段 × 动态
phaseList 目标回合」三重耦合），未达 3/3 立项门槛——**撤项**，登记并入 §7
装备栏/竞赛编排池。**§6 相位裁剪原语族正式收口**（S0 队列基建 + S1 zj_fuli
消费者已交付；sm_kulian 移交 §7 竞赛子系统池）。

文档零代码改动；全量回归维持 4245 通过/0 失败基线；ledger 数值不变。

## 本轮：tw/twjieqiu 劫囚·装备栏编排首迁（2026-08-28 第一百零二批，tw ABI 664→673）

resolve_twjieqiu 两支多路器落地（领号 ABI 673/tag 696/Cont 282 TwjieqiuSelect；
新过滤器 `owner_alive_event_target_mark_positive_<mark>` 5 处注册——此前
_no_damage 结尾变体不受影响）：

| 段 | 实现 |
|---|---|
| sub0 主动（出牌阶段限一次） | SelectTargets 选目标 → `disable_equip_slots` 全五槽（引擎自动将现有装备入弃牌堆+EquipSlotDisabled 事件）→ 目标摸 k 张（k=禁用前装备区牌数）→ BUFF 标记（合成 blake2b("twjieqiu_buff")=385592780） |
| sub2 结算 | `turn_ended … optional` → 仍余禁用栏则 owner 入队纯额外回合（storage 每场一次锁） |

| 技能 | 精度 |
|---|---|
| tw/twjieqiu 劫囚 | 近似精确化：废除+摸k+BUFF 标+owner 额外回合四件全接；残差=「按弃牌量选择恢复槽类别」未建模（禁用持续到目标回合结束由 sub2 结算，仍不自动恢复） |

tw npack：modules 155→157、triggers 142→143、active 13→14、ABI 664→673、
三处部署一致。测试 original_tw_jieqiu_impl 3/3（契约+sub0 废除/摸k/标记+
sub2 入队）。冻结断言 tw-embed 3 文件 664→673（twxingshuai 的 committed/
tw_sunluban 的 tw/twdianyi），compiled.* 独立源断言（491 等）逐文件豁免。
全量回归见下。

## 本轮：政略②豁免勘误与猎族收敛确认（2026-08-28 第一百零三批·设计批，无代码）

收尾核验发现:「政略②豁免值源」早在**第八十三批**已落地(jsrg npack 含
card_distance_unlimited_target_mark_1167524380 / sha_use_unlimited_target_mark_
1167524380 两条 modifier,query 距离/次数双接线生效)——**政略①③②全部真实
建模,猎族可达面收敛完毕**。「政略②反向豁免场景」改判:与凝寒同因(无触发源,
引擎级不可达),撤销登记。ADR §1 遗留清单同步修正。
文档零代码改动;回归维持 4248 通过/0 失败基线;ledger 数值不变。

## 本轮：twjieqiu 恢复槽即时等量恢复（2026-08-28 第一百零四批，tw ABI 673 不变，零领号）

引擎级收口 twjieqiu 残差①:「按弃牌量恢复槽类别」以 **CardsMoved(reason=Discard)
路径即时恢复**实现——初始实现误埋于 `tracks_discard_phase_discard_count() &&
phase==Discard` 双重门内(靠 DBG 打印定位:钩子从未执行),重定位为独立顶层块:
BUFF 标记(385592780)目标的每次 he→弃牌堆移动,即时等量恢复禁用槽(位段直减,
确定性顺序)。状态收敛等价于上游 phaseDiscardEnd 一次性按弃牌量恢复——恢复只
影响槽可用性,不影响既成弃牌;**无需 PhaseEnded{Discard} 携带弃牌集合的引擎
扩展,零领号**。

测试 original_tw_jieqiu_impl 4/4(新增:废除 5 槽→弃 3→剩 2 禁用即时断言)。
全量回归 workspace exit 0、**4249 通过/0 失败**(grep FAILED 复核为 0)。
ledger 数值不变(twjieqiu 仍在近似轴)。

## 本轮：guili 近似注记核销（2026-08-28 第一百零五批·设计批，无代码）

三条近似注记逐条核验:
- ①「绑定态以标记在场表达」:**核销**——上游 storage 绑定为一对一关系,标记
  在场即等价建模,非残差;
- ③「整回合队列 vs 事件流内定向 next」:**核销**——queue_extra_turn 与上游
  insertPhase 语义一致,非残差;
- ②「storage key2 每场一次锁」:**保留**——上游按绑定唯一性天然限一次,本实现
  的 storage 锁与上游限一次等价(此条重审后亦为等价,但锁实现细节保守保留注记
  以便后续 twjieqiu 恢复盘落地时联动调整)。

guili.skill 注记文本已刷新(核销两条/保留一条);xianding npack 重建部署一致
(ABI 669/mods 383/triggers 154 不变——注释层零影响)。guili 套件 3/3、lie_family
8/8、tw_jieqiu_impl 4/4 回归绿。ledger 数值不变。

## 本轮：装备栏编排尾项终判登记（2026-08-28 第一百零六批·设计批，无代码）

twjieqiu 恢复槽类别选择盘残差关闭登记:即时等量恢复已覆盖状态语义(禁用槽
数量收敛一致),剩余差异仅为「玩家偏好表达」;实现需 PhaseEnded{Discard} 事件
扩展+逐张交互盘+领号 674,投入/收益比失衡——除非未来 ≥3 处同类消费者重开。
dcwumei 动态重排单独挂起,并入 §6 相位裁剪原语族 S3 追踪。
§7 装备栏编排子族状态:twjieqiu 全链收口(废除+摸k+BUFF 标+即时恢复+owner
额外回合);dcwumei 单独挂起。文档零代码改动;回归维持 4249 通过/0 失败基线;
ledger 数值不变(complete=5040/approx=493/noop=2048)。

## 本轮：dcwumei 领号级立项书·§6 S3 终判（2026-08-28 第一百零七批·设计批，无代码）

xianding dcwumei 无昧通读后确认本体为**全场体力快照/回滚**（wake 子技
phaseBegin 记录全场 hp → phaseEnd 全员 changeHp 回滚），叠加 target 插相、
owner 二段插相（phaseList 重演）、nexts 队列操纵、pushHandler 全局历史栈
注入、_finished/untrigger 深度事件流控制。依赖「选择性状态快照/恢复」
设施（引擎 Transaction journal 已有基础，业务层部分回滚=新原语族），
规模多批级。S3 终判：dcwumei 与 twjieqiu 同池挂起，待「状态快照/恢复」
设施立项后一并实现。文档零代码改动；回归维持 4249 通过/0 失败基线；
ledger 数值不变。

## 本轮：ADR 0646 登记表全池收口审计（2026-08-28 第一百零八批·设计批，无代码）

登记表终局化:全池逐条核对,消除全部「挂起/待/候选」字样(9 处措辞改写 +
§6 S3 与 §7 重复条目合并为单一追踪入口)——每条均落四类终局态之一:
**已落地**(猎族S1/城族S2a·S2b/相位裁剪S0·S1/guili/nsfuge/umi/botu/fangquan2/
zj_fuli 等 13 技)、**已终判不可达**(凝寒两半互锁、chunqiu 技能互换、
shiji tamo 座次盘、sxrmmieguo selectButton 盘、old mingjian 相位跳转、
访客武将牌堆、dcwumei 快照回滚、twjieqiu 恢复类别选择盘、政略②反向、
sm_kulian 竞赛判定)、**已撤项**(§6 S2 skip_remaining_phases、
fengying 多路器扩 action[mbquanchong 已迁])、**登记债终态**
(§7 三池:访客/技能夺取·竞赛/装备栏编排尾部,多批级)。
回归维持 4249 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验启动 + oljuhun 纠错（2026-08-28 第一百一十批，collab ABI 651 不变）

approx 轴逐条核验启动（493 条抽样）：首例 **collab/oljuhun 拘魂** 纠错——上游
`oljuhun = { audio: 5 }` 为纯音频壳无规则面（扑克组合玩法仅存在于 info 文本，
JS 未实装），此前「出牌阶段开始摸两张」为无上游依据的发明（三不信违例）。
回退为诚实 audio-only 终态（INTENTIONAL_EMPTY_NOOP + hand_limit 0 格式）。
collab npack 重建部署（ABI 651 不变、triggers -1）；ledger complete +1/
noop +1（oljuhun 从无据发明归位 audio-only 终态轴）。collab 系套件 14 测试绿。
核验方法论沉淀：approx 抽样先查上游 def 体是否实为 audio 壳——此类为「误升
approx」而非「真残差」，回退即纠错。

## 本轮：approx 抽样核验·四例维持(2026-08-28 第一百一十一批·核验批，无代码)

collab/newjiang 四例上游对照核验，全部判「真实残差，维持近似」：
| 条目 | 上游本体 | 维持理由 |
|---|---|---|
| collab natufu | useCardAfter 锦囊后：natuxing 同名集合+isMinHp+isDamaged 条件分支→回复/火伤 | 条件分支+火伤属性复合 |
| collab dchaoshi | expansion 点数之和≥100 门→获所有牌+插相 | 点数求和原语+插相（插相已有 queue_extra_phases，但门槛缺失为主门）|
| newjiang fjzhitu | 等距追加目标（与你距离=当前目标距离的其他角色）| 等距追加目标原语缺失 |
| newjiang mpsijiu | 本回合他人获得牌后结束阶段摸一+观看一名其他角色手牌 | 观看手牌省略（残差小，已注记）|

approx 核验进度：1 纠错(oljuhun)+4 维持=5/493。零代码改动；回归维持 4248 通过
/0 失败基线；ledger 数值不变。

## 本轮：approx 抽样核验·四例维持（2026-08-28 第一百一十二批·核验批，无代码）

collab olweiyang/olhaizhu 经查实为**完整迁移**（active targets/match_selected_
card_count 与 cards_moved black-to-discard+双触发器全链），非核验对象——
ledger approx 抽样名单存在陈旧引用，实际以移植目录现状为准。

offline peduzhang/ymqianling 两例上游对照核验，均判「真实残差，维持近似」：
| 条目 | 上游本体 | 维持理由 |
|---|---|---|
| offline peduzhang 独仗 | 黑色牌唯一目标摸一+获凛+手牌上限+凛数 | 凛系统+黑色/唯一过滤缺 |
| offline ymqianling 潜聆 | 展示手牌+与黑色手牌数相同者交换 | 展示/比色/交换复合 |

approx 核验进度：累计 9/493（1 纠错+8 维持）。零代码改动；回归维持 4249 通过
/0 失败基线；ledger 数值不变。

## 本轮：approx 抽样核验·四例(2026-08-28 第一百一十三批，offline ABI 672 不变)

offline/collab 四例上游对照核验：
| 条目 | 判定 |
|---|---|
| offline zybishi 必失 | **纠错**：原移植「自己摸1」方向写反——上游为「他人对你出杀→该杀对你无效+使用者摸1」，已按上游方向修正（draw_cards(event_source)）；「牌目标取消(all_excluded)」由引擎既有目标取消事件面承接注记 |
| collab dczuandai 钻袋 | 维持：牌堆搜索+use 联动缺失 |
| collab mbzhengpeng 正鹏 | 维持：多条件判定+动态摸牌数+跨类型检索 |
| offline zyshangyi 尚义 | **等价核销**：移植 reveal_hand_to_player+choose_optional_discard_target_card 与上游 viewHandcards+chooseCardButton+discard 完全等价（原注记「旧 JS 仅语义合同」即等价声明）|

approx 核验进度：13/493(1 纠错 oljuhun+1 纠错 zybishi+2 等价核销+9 维持)。
offline npack 三处部署一致(ABI 672 不变)。全量回归 workspace exit 0、
**4249 通过/0 失败**。过程红两例均为 zybishi 方向修正的预期涟漪:同包
olchouniu/olsishe 的 canonical_state_hash golden 随同 seed 状态序列变化——
按 batch AO 先例以失败输出 left 值重录(非 bug)。

## 本轮：approx 抽样核验·collab 四例维持（2026-08-28 第一百一十四批·核验批，无代码）

collab 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由(残差真实) |
|---|---|
| dclbjiuxian 酒仙 | 「仅限多目标锦囊」筛选与 jiu 无次数修饰缺失 |
| dcrende 仁德(会玩) | 两张/任意基本牌/同 phase 不重复目标三缺 |
| dcshentui 神推 | 点数与攻击范围比较的双分支(强命+不计次/伤害+1)缺 |
| natuheng 恒 | 手牌构成门控与每轮限一次未精确建模 |

approx 核验进度:17/493(2 纠错+2 等价核销+13 维持)。零代码改动;回归维持
4249 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·那兔系三例维持（2026-08-28 第一百一十六批·核验批，无代码）

collab 那兔系三例上游对照核验,全部判「真实残差,维持近似」:
| 条目 | 维持理由 |
|---|---|
| natulie 烈 | 递增 X(发动次数上限)+「弃牌后对一名角色造成1点伤害」分支缺 |
| natuxing 兴 | 兴区/锁定兴 gaintag 系统缺(开局各锦囊一张、伤害解锁) |
| natuyi 义 | 颜色禁手+分配牌+gaintag 联动缺 |

approx 核验进度:16/493(2 纠错+2 等价核销+12 维持)。零代码改动;回归维持
4249 通过/0 失败基线;ledger 数值不变。

## 本轮：gaintag 区系统立项评估 + approx 核验三例维持（2026-08-28 第一百一十七批·设计批，无代码）

①gaintag 消费者清单核验:跨 12+ 包(collab 41 处那兔系为最大簇),**达 3/3
门槛**,ADR 0646 新增 §8 立项评估——能力分解:①per-card 标签(既有)
②hasCards-with-gaintag 查询(缺)③组合判定(缺,引擎级);S1 ①②先行,
S2 ③单独评估;领号预估 ABI 674。

②approx 核验三例维持(newjiang dcqiaolve 弃牌代价/dcvjiang 交牌分支/xvzhi
交换)——上游皆 chooseTarget/cost_data/交换复合面。

approx 核验进度:19/493。文档零代码改动;回归维持 4249 通过/0 失败基线;
ledger 数值不变。

## 本轮：§8 S1 gaintag-absent 过滤器 + natuxing 门控接线（2026-08-28 第一百一十八批·代码批，collab ABI 651 不变，零领号）

**S1 交付**:参数化过滤器 `owner_source_hand_gaintag_absent_<tag>`
(OwnerSourceHandGaintagAbsent(u32),5 处注册)——持有者用牌时若手牌不含
该 gaintag 卡则触发。消费者:natuxing 兴的 unrespondable 触发器由无条件
收紧为「手中无锁定兴 tag」门控(兴 tag=合成 blake2b("natuxing")=297116519)。

测试 original_collab_natuxing_gate 2/2:门关(手牌含兴 tag→policy 不置位)/
门开(无兴 tag→policy & 2 置位)两场景;断言以 SkillInstance.card_use_policy
(公开化)为可观察状态。gaintag 区系统 §8 原语族 S1 完成,S2 组合判定单独评估。

全量回归 workspace exit 0、**4250 通过/0 失败**(1037 二进制)。ledger 数值不变。

## 本轮：§6 S2 落地·skip_remaining_phases + zylianji 连计（2026-08-28 第一百一十九批，offline ABI 672→674）

撤项重判后落地:skip_remaining_phases 单原语虽消费者仅 1/3,但 zylianji 主体
可由既有原语组合近似闭环——「分段互斥触发器」以单 positive 过滤器近似,
「动态 phaseList 目标回合」以 queue_extra_phases(仅 Play)+目标自选近似。

| 组件 | 实现 |
|---|---|
| skip_remaining_phases | tag 697/ABI 674 领号;本回合剩余跳过 mask 置位阶段(Draw=4/Play=8/Discard=16),置 SKIP_DRAW/PLAY/DISCARD flags(既有推进流程自动跳过) |
| 新过滤器 | `owner_alive_play_end_used_categories_positive`(5处注册):PhasePlayEnd + play_card_category_history 位掩码 popcount>0(CardUsed 路径自动记录) |
| zylianji 连计 | `phase_play_end … optional` → choose_other_draw_or_recover(self,1,0,1) → skip_remaining_phases(20) → queue_extra_phases(8) |

| 技能 | 精度 |
|---|---|
| offline zylianji 连计 | 近似精确化:三段递进(摸1/受伤回复/≥3 跳+目标回合)合并为单次(摸1给任一角色+跳剩余+仅 Play 额外回合);「受伤回复」与「≥3 才跳」档位注记省略 |

测试 original_offline_zylianji 2/2(契约+三段链:confirm→摸1选目标→skip flags
置位→裁剪回合入位[phases=[Play]])。offline npack 三处部署一致(ABI 674/
modules 715/triggers 577 不变)。冻结断言 offline-embed 25 文件 672→674
(绑定名核对+compiled/include_str 豁免)。全量回归见下。

## 本轮：approx 抽样核验·五例维持（2026-08-28 第一百二十批·核验批，无代码）

collab/newjiang/offline 五例上游对照核验,全部判「真实残差,维持/占位维持」:
| 条目 | 维持理由 |
|---|---|
| collab mbshezi 蛇子 | 区域选择(手牌/装备/判定)缺 |
| newjiang mbguli 孤骊 | 装备牌弃置+摸至上限+未伤失体三分支 |
| offline cajingong 进攻 | 视为使用【杀】(含子技/失去体力分支)缺 |
| offline calianji 连计 | 目标摸1+VCard 伤害牌盘复合 |
| offline cacuixi 催袭 | 动态目标集(hp<player)+伤害复合 |

approx 核验进度:24/493(累计 2 纠错+2 等价核销+20 维持)。零代码改动;回归
维持 4250 通过/0 失败基线;ledger 数值不变。

## 本轮：offline cajujun 夺爵近似晋升 + count 断言对账（2026-08-28 第一百二十一批，offline ABI 674 不变）

**cajujun 夺爵近似晋升**（原 hand_limit 占位 → 真实体）：限定技 storage 锁 +
`hp_recovered owner_is_target optional` → recover_hp(2-hp 差值) +
draw_cards(max_hp-hp) 表达式组合（locked 与 optional 互斥已摘除 locked）。
近似残差=「本局不能回复体力」norecover 子技未建模（需 recover-cancel 修饰符）。

**count 断言对账复盘**：offline 包 active_index 实际 138→137（cajujun 由
active 占位晋升为 trigger 体）、triggers 577→578、modules 715 不变；
offline-embed 测试文件中 active/trigger 计数断言散布 7+ 文件（nslulve/
spfuluan/tyxianshou/tybenxiang/zyshangyi/zj_cuijian/tyzhuan/ylygqiangxi/
yjxuepin）逐个收敛——教训重申：**count 断言同步必须脚本化对账**（按实际
npack index 长度批量校正），人工打地鼠在本包规模必漏。

collab/newjiang/offline approx 核验五例维持（mbshezi/mbguli/cajingong/
calianji/cacuixi）；全量回归 workspace exit 0、4252 通过/0 失败
（1039 二进制）；ledger complete=5040/approx=493/todo=0/noop=2049。

## 本轮：approx 抽样核验·四例维持（2026-08-28 第一百二十二批·核验批，无代码）

clan 三例+bingshi old_pottuntian 上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| clanbaozu 保族 | 全球濒死+同族判定+横置+回复复合 |
| clanxieshu 协庶 | damage 双向+弃牌+横置+按伤摸复合 |
| clanshize 施泽 | 全局 useCard/respond+宗族范围增益（值源只随技能）|
| old_pottuntian 蓄田 | 蓄力系统（countCharge）+目标 chooseCard 置田缺 |

approx 核验进度:29/493(2 纠错+2 等价核销+25 维持)。零代码改动;回归维持
4249 通过/0 失败基线;ledger 数值不变。

## 本轮：§8 S1 状态回写（2026-08-28 第一百二十批·文档批，无代码）

核验收口:§8 S1「hasCards-with-gaintag 查询/过滤器」已于第一百一十八批落地
——参数化过滤器 `owner_source_hand_gaintag_absent_<tag>`(5 处注册,零领号)
+ natuxing 兴门接线(unrespondable 触发器从无条件收紧为 gaintag-absent 门控,
兴 tag=297116519)+ 测试 original_collab_natuxing_gate 2/2。ADR §8 状态行
同步为「S1 已落地收口,S2 组合判定单独评估」。回归维持 4250 通过/0 失败
基线;ledger 数值不变。

## 本轮：approx 抽样核验·collab 四例维持（2026-08-28 第一百二十二批·核验批，无代码）

collab 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| dcsantou 三头 | 体力条件分支（≥3 按 useSkill 历史改失去体力）缺——无条件 prevent 已覆盖主防御面 |
| olanle 乐 | 浮沉标记系统+动态技能附加（addAdditionalSkill/unmarkAuto）缺 |
| oldigong 帝功 | 使用牌计数器（4 次皮肤切换）+非伤害牌分支+红色判定摸1缺 |
| oldingxi 锭玺 | 跨回合状态追踪+弃牌堆选牌使用+expansion 存储缺 |

approx 核验进度:28/493(2 纠错+2 等价核销+24 维持)。零代码改动;回归维持
4249 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·huicui 五例（2026-08-28 第一百二十四批·核验批，无代码）

| 条目 | 判定 |
|---|---|
| dctujue 突戮 | 维持:觉醒+濒死+chooseTarget 复合 |
| dcyijia 移驾 | 维持:global damageEnd+inRange 计算+moveCard 复合 |
| zhanwan 斩腕 | 维持:cxliushi2 技能联动+lose history 遍历复合 |
| dczhongji 忠继 | **可晋升**:useCard+手牌<maxHp→draw min(20, maxHp-hand)——条件与动态摸数均可表达式表达 |
| zhuangrong 庄荣 | **可晋升**:global phaseEnd+hp==1\|\|hand==1→loseMaxHp+recover 差值——均可用现有表达式组合 |

approx 核验进度:29/493(本批+5)。零代码改动;回归维持 4252 通过/0 失败基线;
ledger 数值不变。下批=dczhongji/zhuangrong 双晋升实现批。

## 本轮：huicui dczhongji 忠继·近似晋升（2026-08-28 第一百二十五批·代码批，huicui ABI 487 不变）

**dczhongji 忠继近似晋升**（原「无条件摸1」→ 精确）：
filter `owner_is_target_hand_below_max_hp`（手牌<maxHp，引擎既有）+
`draw_cards(self, min(20, max_hp(self) - hand_count(self)))`（min 表达式,
引擎既有）——**零新原语、零领号**。

测试 original_huicui_dczhongji 1/1(契约:ABI 487+filter 断言)。huicui 系
4 二进制 12 测试全绿。全量回归 workspace exit 0、4252 通过/0 失败
(1039 二进制);ledger 数值不变。

approx 核验进度更新:本批核验+晋升累计 29/493(111 批 4+112 批 4+113 批 4+
114 批 4+116 批 3+117 批 3+120 批 5+本批含 dczhongji/zhuangrong 判定)。
zhuangrong 维持近似(逻辑或条件表达式缺口登记)。

## 本轮：approx 抽样核验·五例维持（2026-08-28 第一百二十六批·核验批，无代码）

key 三例+shiji 两例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| key akane_quanqing 劝请 | 点数分支与弃牌/伤害未建模 |
| key akane_yifu2 义父·实装 | 交给改弃置；不限主公技持有者；不返还手牌 |
| key asara_shelu 射鹿 | 不展示对方手牌；不比较花色/点数；回复分支未建模 |
| shiji chuhai2 除害·续 | 按空装备栏从牌堆取装备原语缺 |
| shiji fubi 缚辔 | 选择关联角色原语缺（fubi2 加成为无条件）|

approx 核验进度:**33/493**(2 纠错+2 等价核销+29 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：offline dragxueji 血祭近似晋升（2026-08-28 第一百二十七批，offline ABI 674 不变）

原「摸 1 占位」(与上游完全无关的发明)晋升为真实体:
`active play_phase uses_per_turn 1 { choose_other_damage(self, 1) }`——
选一名其他角色造成 1 点伤害。近似:①多目标(1..已损体力)收窄为单目标
(ChooseOtherDamage 单选);②火焰属性未建模(普伤替代)。

测试 original_offline_dragxueji 2/2(契约+ActivateSkill 命令驱动全链:
victim hp-1 断言)。offline npack 三处部署一致(ABI 674/active 137 不变)。
全量回归见下。

## 本轮：approx 抽样核验·yijiang 三例+sxrm 一例维持（2026-08-28 第一百三十批·核验批，无代码）

yijiang 三例+sxrm 一例上游对照核验,全部判「真实残差,维持近似」:
| 条目 | 维持理由 |
|---|---|
| yijiang reanjian 暗箭 | useCardToPlayered 距离判定(customArgs 注入)缺 |
| yijiang reduodao 夺刀 | 武器区牌夺取(武器限定)缺 |
| yijiang qinwang 勤王 | 蜀势力角色轮询+主公技门缺 |
| sxrm sxrmjiehuo | 无运行时行为(audio 壳,包 README 可表达性清单已注明) |

approx 核验进度:37/493(2 纠错+2 等价核销+33 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·五例维持（2026-08-28 第一百三十一批·核验批，无代码）

key 两例+yijiang 三例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| key erika_shisong 诗诵 | 同位序同类型摸牌（跨回合 useCard 历史对照）缺；护甲上限(owner_mark_3)已建模 |
| key fuuko_xingdiao 星钓 | 「星」展示/不计上限/不可弃/每轮不再摸至缺；adjust_hand_to(8) 已建模 |
| yijiang jieyue 竭节 | 弃牌选目标交互分支缺；gain_all_skill_pile_cards 已近似保留 |
| yijiang qianxi2 潜袭续 | 花色 storage 动态禁用需引擎层（占位已注记）|
| yijiang oldzhenlie 镇烈 | judgment_rewrite 牌堆顶来源与弃原判牌未建模 |

approx 核验进度:44/493(2 纠错+2 等价核销+40 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·collab 六例维持（2026-08-28 第一百三十四批·核验批，无代码）

collab 六例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| oldianbu 点簿 | 十三张与扑克组合未建模 |
| olfengzhu 逢主 | 目标选择+义父标记存储+改名广播缺（locked 不可含 optional 的引擎约束）|
| olliangzi 梁姿 | 无次数限制与势力计数未建模 |
| olsanou 击倒 | 击倒 debuff（取消出牌阶段+多层事件监听）缺 |
| olsuizhong 岁终 | 敌方循环弃牌（DSL 无循环）+首次伤害过滤器缺 |
| olweidai 魏代 | 势力门控与压上限/变更势力未建模 |

approx 核验进度:37→43/493(2 纠错+2 等价核销+39 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·offline 五例维持（2026-08-28 第一百三十五批·核验批，无代码）

offline 占位群五例上游对照核验，全部维持:
| 条目 | 判定 |
|---|---|
| hm_houying 侯应 | 维持占位——黑牌≥2 计数 filter 缺(视为杀可用 choose_use_virtual_sha)|
| hm_qianhu 千户 | 维持占位——红牌≥2 计数 filter 缺(视为决斗可用 choose_use_virtual_card)|
| hm_qianjun 千军 | 维持占位——swapSeat 座次交换引擎缺 |
| chunqiubi_skill | 注册名待查(changan 文件无同名块)|
| dragshiyao | 维持占位(随机毒牌生成,134 批已验)|

**新立项候选**:「黑/红牌计数 filter」×2(消费者 hm_houying/hm_qianhu,若加
zj_cuijian 系同类则达 3/3)——虚拟杀/决斗原语既有,零引擎新设施。
approx 核验进度:44→49/493。零代码改动;回归维持 4252 通过/0 失败基线;
ledger 数值不变。

## 本轮：颜色计数 filter 不立项判定（2026-08-28 第一百三十六批·核验批，无代码）

zj_cuijian 摧坚核验:已迁(reveal_target_hand_gain_shan_armor_or_draw_two 专用
原语),非「黑/红牌计数」同类。颜色计数 filter 消费者最终 **2/3**
(hm_houying 黑≥2 视为杀/hm_qianhu 红≥2 视为决斗),未达门槛——**不立项,
维持登记**(两占位维持)。若后续出现第三处同类消费者(黑/红牌计数+视为
杀/决斗),可重开立项(虚拟杀/决斗原语既有,仅缺计数过滤器)。

零代码改动;回归维持 4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·huicui 四例维持（2026-08-28 第一百三十八批·核验批，无代码）

huicui 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| anyong 暗恿 | source==currentPhase 条件+chooseToDiscard 弃牌代价缺 |
| dcdeshao 德劭 | 黑色牌条件（get.color=="black"）缺 |
| dcfanshi 反噬 | 觉醒+choices 列表逻辑（4-仓储计数）缺 |
| chijian 持剑 | 虚拟装备占槽/攻击范围/死亡转移缺（ignore_armor+sha_use_limit 已表达主面）|

approx 核验进度:45/493(2 纠错+2 等价核销+41 维持)。零代码改动;回归维持
4249 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·huicui 四例维持（2026-08-28 第一百二十五批·核验批，无代码）

huicui 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| dcfudou 复豆 | 颜色判定分支(红→双方各摸/黑分支语义未明)——当前实现方向与上游 red 分支不一致，修正方向不定 |
| dcgangjian 刚坚 | 无伤害历史条件+动作未明 |
| dcguangshi 光逝 | 全场 dcjizhong 标总数动态摸数+loseHp |
| dcguyin 鬼引 | gameDraw/loseAfter/cardsDiscardAfter 多事件+num 函数重写复合 |

approx 核验进度:49/493(2 纠错+2 等价核销+45 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·huicui 四例维持（2026-08-28 第一百二十六批·核验批，无代码）

huicui 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| dcjinjin 金金 | 朝冲标数动态+storage 重置复合 |
| dcjizhong 集众 | chooseBool 二选一（获三张牌或得信众标）分支 |
| dclongyuan 龙渊 | yizan_use 标≥3 门控觉醒+draw2+recover |
| dcmoshou 陌收 | 黑色牌+跨角色 mark 数动态摸数（maxHp-mark 数）复合 |

approx 核验进度:53/493(2 纠错+2 等价核销+49 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·huicui 四例维持（2026-08-28 第一百二十七批·核验批，无代码）

huicui 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| dcmouli 谋利 | checkAllHistory useSkill 过滤+觉醒+跨技授予(dczifu)复合 |
| dcqingren 情人 | useSkill 历史计数动态摸数（yizan_use 次数）缺 |
| dcqinqing 亲亲 | 主公座次判定(seatNum==1)+chooseTarget+弃牌复合 |
| dcqinshen 亲身 | getNum() 动态计算（引擎无对应表达式）缺 |

approx 核验进度:57/493(2 纠错+2 等价核销+53 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·huicui 四例维持（2026-08-28 第一百二十八批·核验批，无代码）

huicui 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| dcxingbang 兴邦 | 目标交互（draw+强制交 he+获手牌）+循环比牌复合 |
| dcyongdi 用地 | 觉醒+isMinMaxHp/isMinHp 复合判定+gainMaxHp+recover |
| dcyuguan 御冠 | 已损体力判定（getDamagedHp>0）+loseMaxHp+后续——近似体已有 loseMaxHp 主体 |
| dczhuguo 主国 | loseAfter/cardsGotoPile 多事件→牌堆底摸 2（bottom 摸牌缺）|

approx 核验进度:57/493(2 纠错+2 等价核销+53 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·huicui 四例维持（2026-08-28 第一百二十九批·核验批，无代码）

huicui 四例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| dczhuoli 拙略 | useCard 历史>hp 或 gain 总数比较+条件 gainMaxHp 多条件统计复合 |
| dczifu 自缚 | 手牌<min(5,maxHp) 复合条件缺（逻辑或/复合条件 filter 缺口同族）|
| liedan 烈胆 | player.die() 死亡自触发+mark>4 门缺 |
| yaopei 要佩 | huguan 技能门+目标弃牌花色集合统计复合 |

approx 核验进度:57/493(2 纠错+2 等价核销+53 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：approx 抽样核验·五例维持（2026-08-28 第一百三十批·核验批，无代码）

huicui 三例+jsrg 两例上游对照核验，全部判「真实残差，维持近似」:
| 条目 | 维持理由 |
|---|---|
| huicui dcfanhuo 反火 | hasSkill dcxingbang 门+awaken+addTempSkill mark 复合 |
| huicui shiwu 师武 | VCard 决斗 canUse+useCard 决斗+夺剑/死后授剑复合 |
| huicui zhengding 正定 | useCard/respond 非当前阶段角色+gainMaxHp+recover(非当前阶段条件 filter 缺)|
| jsrg jsrg_new_juezhi 绝质 | 废除对应栏+回合内伤害+1 缺(draw2 主体已保留)|
| jsrg jsrg_new_juxia 居下 | 不比较双方技能数(已注记)|

approx 核验进度:61/493(2 纠错+2 等价核销+57 维持)。零代码改动;回归维持
4252 通过/0 失败基线;ledger 数值不变。

## 本轮：extra 龙魂族四例纠错·遗留单臂移档收敛（2026-08-28 第一百三十九批）

extra 包 approx 抽样命中龙魂族四条（longhun1-4），上游对照（legacy-source/
character/extra/skill.js longhun 及 group 四子技能、relonghun 全块）核验后
判「纠错」而非维持：

1. 注记失实：四文件近似注记均称「花色条件未建模」，但四个技能体内均已带
   花色过滤（`view_as tao play_phase heart` 等）——花色条件实际已建模。
2. 被组技能取代：基础龙魂已在 `longhun.skill` 六臂高保真实现——tao/sha
   (nature fire) play_phase+response 双窗口、wuxie any、shan response，
   全部 `owner_hp_min_one_cards`（=上游 `selectCard() => max(1, hp)` 动态
   张数）、`hand_equip_expansion` 区（=上游 position "hes"）。四单臂文件为
   迁移早期遗留，语义是组技能的真子集（缺 response 窗与动态张数）。
3. 武将面：神赵云挂载 `relonghun`（up_to 2 same_suit + 火属性 +
   multi_card_bonus damage_1/recover_1/discard_current_hand_equip），与上游
   relonghun group 子技能逐条对应（relonghun_num：二张杀/桃 baseDamage++；
   relonghun_discard：二张闪/无懈弃当前回合角色一牌）；行为测试
   original_extra_relonghun 6/6 既有覆盖。

处置：四文件移入 `content/original/ported/extra/archive/`（rebuild-pack 的
非递归 glob 不装入 npack，保留磁盘追溯，附 README 交代取代链）；extra
npack 重建并四处同步（content 75→71 skills、extensions/extra×2、
.npack-store extra 槽撤销重装，logic_hash f0c4301e…，ABI 240 与
character_index 不变）；gen-fidelity-ledger.py 排除 archive/ 子目录并修正
扫描范围说明；ledger 重生成 **5529 文件 complete=5037/approx=492/noop=2049**
（四遗留此前被计 complete，approx 数值不变）。

既有问题备案：.npack-store 缓存自 08-07 未随批刷新，official/clan@0.1.0
槽位元数据与 extensions 不一致（旧缓存 88 技 vs 现池 87 技），整集校验在
clan 处报错属既有状态，非本批引入；本批仅重装 extra 槽不扩面。权威部署面
extensions/ 与 content/ 保持一致；real_extensions_lock（32 包全新安装+激活
锁+标准对局启动）绿。

回归：engine-core 3466 通过/0 失败（双遍确认）；workspace 全量 **4255 通过/
0 失败/1 忽略**（高于 4252 基线，增量来自并行会话新测试）。测试面加验：
original_extra_wuhun（logic_hash committed vs 重编译一致性）21/21、
original_ported_character_closure 1/1、content_lock_extensions 2/2。

approx 核验进度:+4 纠错(龙魂族)→49/493(4 纠错+2 等价核销+43 维持)。

## 本轮：extra 折叠遗留六件移档收敛+shiji 四例核验（2026-08-28 第一百四十批）

extra 包 approx 抽样命中 dawu3/kuangfeng3/qixing2/g_drlt_zhiti/g_nzry_jieying
及同族 duorui_clear，共六件，全部判「纠错·移档」（同为 sync 脚本
`foldedIntoParentSkills` 表登记的被取代遗留，无武将挂载、无授予引用、无测试
引用，catalog 以 covered_by 指向父体）：

| 条目 | 取代关系（上游对照） |
|---|---|
| dawu3/kuangfeng3 | 上游为「dawu2 标记+dawu3 addTempSkill 执行件」拆分；本库 qixing.skill 的 dawu2/kuangfeng2（damage_taken_cancel 防非雷伤/火伤+1 modifier）直挂目标等价建模；二者 draw-1 假体为早期占位 |
| qixing2 | 摸牌阶段摸 1 占位；被 qixing 的 resolve_qixing(self,0) 交换臂取代（同事件双触发冗余） |
| g_drlt_zhiti | 无条件手牌上限 -1 占位；父体 drlt_zhiti（drlt_duorui.skill）已有条件版 `global_hand_limit add -queried_player_wounded_in_owner_attack_range_other` |
| g_nzry_jieying | 上游 1.11.5 已删除该全局助手（sync 表仅存注释、catalog 无条目）；父体 nzry_jieying（nzry_longnu.skill）为权威实现；本文件为孤儿 |
| duorui_clear | 上游为 sourceSkill 清理语义（无规则行为）；本文件误带 optional draw-1 假体 |

处置：六文件移入 extra/archive/（README 增补取代链）；extra npack 重建
四处同步（71→65 skills，ABI 240 与 character_index 不变，store 槽重装
logic_hash 58e92191…）；sync --verify-only 通过（catalog 与归档后状态一致，
covered_by 折叠条目不依赖本地 .skill 文件）。

shiji 抽样四例核验（对照 legacy-source/character/shiji/skill.js）：
| 条目 | 判定 |
|---|---|
| chuhai2 除害·续 | 维持+注记纠错——原注记把上游条件误写为「判定区已废除」，实为「拼点关联角色存储+空装备栏+按副类别取装备装备」，三者均无原语，维持摸 1 近似 |
| refubi 缚辔·新 | 维持+注记纠错——增益偏差如实补记：上游为每回合准备阶段二选一（杀次数 +1/有标记他人手牌上限 +3），近似把 +3 挂拥有者自身、开局用赠牌原语代标记 |
| lingce 令策 | 维持——牌名集合（奇正相生/智囊/dinghan storage）filter 缺，且上游无每轮一次限制 |
| luanchou 乱宠 | 维持——双目标姻标记清除重发+共患伤害转移（damageBegin4 改 target）均无原语 |

ledger 重生成：**5523 文件 complete=5031/approx=492/noop=2049**（六遗留此前
被计 complete；chuhai2/refubi 注记文本精确化，类属不变）。回归：目标测试
（qixing/relonghun/wuhun/closure/扩展锁）全绿；workspace 全量 **4255 通过/
0 失败/1 忽略**（与一百三十九批基线持平）。既有备案不变：.npack-store 整集
激活仍卡 clan@0.1.0 既有漂移（extra 槽自身已一致）。

后续审计项：`foldedIntoParentSkills` 表内其余仍有本地 .skill 文件的折叠条目
（extra:renjie2、yijiang:sidi2、shenhua:baonue2/songwei2/bazhen_bagua/
wansha2、sp:shanjia2/hanyong3/fengpo2/qiangwu3/nuzhan2、sp2:langmie_damage、
standard:kongcheng1/jijiang1/jijiang3、key:noda_xunxin2/chihaya_liewu2/
hinata_qiulve_clear、mobile:mobilejingce_count/beizhu_draw、refresh:
hanzhan_gain、shiji:xianghai_g/qinzheng_count 等）可按同规格逐包核验假体与
挂载状态后移档收敛。

approx 核验进度:+4(shiji 维持)→53/493(4 纠错+2 等价核销+47 维持)。

## 本轮：折叠表全量审计·五包 23 文件三向处置（2026-08-28 第一百四十一批）

对 `foldedIntoParentSkills` 表（43 对）做机械化三查（本地文件存在/def_id 全仓
引用/characters.json 挂载）：23 对本就无本地文件（含前两批归档的 7 件），
**20 个在池文件**逐一核对 bodies 后三向处置——

**保留 5（显式政策注记）**：shiji:xianghai_g（空壳，「仅保留 skill_index 元数据
条目」政策）、mobile:mobilejingce_count 与 sp:shanjia2 与 sp2:langmie_damage
（INTENTIONAL_EMPTY_NOOP，2026-08-21 复核债注）、sp:nuzhan2（「保留等价
语义」等价副本政策，父技 nuzhan 同条款）。

**修正 3（按其自身「空定义保留稳定 ID」政策清假体）**：shenhua:bazhen_bagua、
yijiang:chunlao2、yijiang:chunlao2_old——注释均称暂留空定义，体内却带 draw-1
假体；改为真空壳 `{}`，语义由父技承载。

**归档 15**：shiji:qinzheng_count（注释自称无规则行为却带 optional draw-1）、
mobile:beizhu_draw（权威在 ResolveBeizhu，本地假体若挂载将双份结算）、
yijiang 七件（sidi2/qiangzhi_draw/lihuo2/lihuo3/reshangshi_2nd/yonglve2/yanyu2，
占位/近似假体）、shenhua 三件（baonue2/songwei2 无条件摸牌近似、wansha2
hand_limit+1 与完杀语义无关）、sp 三件（hanyong3/fengpo2 占位、qiangwu3
冗余双 modifier——父技 qiangwu.skill 含同款双臂）。各包 archive/README 增补
取代链。

部署：五包（shiji 178→177、mobile 441→440、yijiang 288→281、shenhua
204→201、sp 730→727）npack 重建四处同步，ABI 全部不变、character_index 不
变；store 五槽重装（logic_hash 逐槽核验一致）；sync --verify-only 通过。

测试对账（冻结断言按新值收敛）：37 文件 skill_index 计数、trigger/modifier
深层计数（mobile trigger 171→170、shenhua trigger 161→158 + modifier 57→56、
yijiang trigger 223→215）、modules 计数（mobile 216→215、shenhua 186→183、
sp 270→268×3）、sp PACKAGE_LOGIC_HASH×3 重写、refresh_relongyin 跨包
canonical_state_hash 按 batch AO 惯例重录（continuation_payload 行为断言
先验绿，仅内容派生哈希变化）；退役 original_sp_fengpo2.rs（其两测锁定
fengpo2 固定 +1 占位，非上游「按出杀计数加伤」语义；父技覆盖见
original_sp_fengpo_mayunlu）。

回归：engine-core **3464 通过/0 失败/1 忽略**；workspace 全量 **4253 通过/
0 失败/1 忽略**（-2 为退役的 fengpo2 两测，其余与基线持平）。既有备案不变：.npack-store 整集激活仍卡 clan 漂移。

**审计方法论沉淀**：折叠文件去留以「显式政策注记」为准——空壳元数据
（xianghai_g 型）、诚实 noop（shanjia2 型）、等价副本（nuzhan2 型）留池；
无政策注记的假体/占位一律归档。include_str 按文件名嵌入的测试 def_id 扫描
抓不到，处置前须按文件名补扫（fengpo2 教训）。

## 本轮：shiji 五件+sxrm 一件抽样·覆海原语晋升（2026-08-28 第一百四十二批）

shiji 五件+sxrm 一件上游对照核验；一处零 ABI 晋升落地：

**晋升：scfuhai 覆海（神孙策）**——原注记「锦囊直击待通用 directHit 原语」
已陈旧：`make_current_card_unrespondable`（natuxing 批所建，bytecode 220）
早已在位。换用通用原语（此前为 sha 专用 `make_current_sha_target_unrespondable`），
锦囊指定有标记角色时直击生效。注记同步纠错：标记名实为「应霸」
（yingba_mark=3219705664，由 yingba 设置）非「平定」；上游每回合前两次摸牌
限次与 forced 语义维持注记。新增行为测试 ×2（original_shiji_shen_sunce）：
锦囊（决斗）指定有标记角色 → policy 位 2 置位 + 摸 1（confirm 选择链经
ResolveChoice）；无标记/无目标 → 静默。3/3 绿。

**维持+注记纠错：tspowei 破威**——原注记称上游「有标记目标失去 1 点体力」，
上游 remove 子技实为**移除全部笃烈标记**；使命技状态机/标记散布移除/二选一
分支/神助授予均无原语，维持占位并如实补记。

**维持 4**：
| 条目 | 维持理由 |
|---|---|
| shiji fubi 缚辔 | 条件手牌上限（关联角色存活门）无值源；开局赠牌代标记（同 refubi 类） |
| shiji mbyongjue 勇决 | 需无 not_target 约束的 sha_resolved 变体过滤器——单消费者不达铸臂门槛，维持注记（候选登记） |
| shiji xinmouli 谋立·新 | 「备」expansion 区选择未建模（访客族/牌堆债同池） |
| sxrm sxrmjiehuo 劫火 | 包级诚实登记骨架（使命技状态机需伤害替换+失败判定；README 可表达性清单在案） |

审计发现：**yingbian/sb 两包 approx 实为零条**（此前派单名单过时）——两包
全部技能为 complete 或诚实 noop。未验 approx 面收敛为 shiji 剩余 2 件
（jianzhan/mjchenshi_target）+散包。

部署：shiji npack 重建（177 skills 不变，仅 scfuhai module 码更新）四处同步，
store 槽重装；计数/触发/模块断言全不变，无冻结哈希绑定 shiji。

回归：engine-core **3466 通过/0 失败/1 忽略**（+2 为新增覆海行为测试）；
workspace 全量 **4255 通过/0 失败/1 忽略**（与基线持平，+2 即新增两测）。ledger 数值不变（scfuhai/tspowei 均保持 approx 类属）。

## 本轮：§7 技能夺取设施 S0 落地·音洄增益臂接线（2026-08-28 第一百四十三批）

**领号 ABI 675**（ABI-COORDINATION.md 领取表+ABI-MAX 已更新，host
RULE_VM_ABI_VERSION 674→675，abort_bump_without_claim 绿）：

- **CHOOSE_GAIN_OTHER_SKILL**（tag 698/Cont 283）：`choose_gain_other_skill(player, self_skill)`——两段选择链：段0 选一名其他存活角色（仅列有可获得技能者）→ 段1 从其可获得技能名单（def 元数据 `locked==false && non_gainable==false`，def 升序去重）选一个 `AddSkill` 获得之。分段共用 Cont 283（payload stage 位）。全链 15 处：ir.rs/typecheck.rs/codegen.rs（含 ADDITIVE_RULE_VM_ABI_VERSION_675 阶梯头）/lsp.rs/bytecode.rs/vm.rs/rule-api effect.rs/engine-core state(ContinuationKind)/primitives(pack_bingshi_1)/resolve_choice/effect 重导出/hash.rs（digest 154+盐 0x4347_4F53）/engine-api version/ABI-COORDINATION。
- **消费者接线：bingshi potyinhui 音洄**（原「每轮首次准备阶段摸两张」近似）→ `phase_prepare once_per_round optional { choose_gain_other_skill(self, self_skill) }`——每轮真实获得一名其他角色一个可获得技能。残差：上游「先失去因此获得的技能」臂需获得技能追踪（设施 S1，与选择性状态快照同池），维持注记。
- **行为测试**（original_bingshi_potyinhui_gain，2/2 绿）：三段确认链（触发 confirm→选目标→名单选技）后 owner 获得目标技能；locked+non_gainable 技能不入名单；每轮一次闩。
- **部署与对账**：bingshi npack ABI 570→675 四处同步+store 槽重装（logic_hash 逐槽核验）；技能数 101 不变。27 文件 ABI 冻结断言 570→675 脚本化对账；original_bingshi_pot_yuji 单文件源夹具（pot_yuji.skill，不含新原语）的独立编译 ABI 断言按其真实值回缀 570——include_str 单文件夹具与 npack 整包断言必须区分（四层对账新条目）。

ADR 0646 §7 池状态回写：chooseButton-from-名单设施 S0 已落地（领号 675），
S1（失去已获得技能+获得追踪，待选择性状态快照设施）与 sm_kulian 竞赛系统
维持登记债。

回归：engine-core **3468 通过/0 失败/1 忽略**（+2 新增音洄测试）；workspace
全量 **4257 通过/0 失败/1 忽略**（与基线持平，+2 即新增两测）。

## 本轮：§7 技能夺取设施 S1 落地·音洄升级为完整上游语义（2026-08-28 第一百四十四批）

**领号 ABI 676**（领取表+ABI-MAX 已更新，host 675→676）：

- **LOSE_GAINED_SKILL**（tag 699/Cont 284）：`lose_gained_skill(player, self_skill)`——从本设施获得清单选一个技能 RemoveSkill 并出清登记；清单空静默；可取消。配套 GameState 新字段 `facility_gained_skills: BTreeMap<u32, Vec<u32>>`（#[serde(default)]，玩家 raw → 设施获得 def 列表）+ S0 段1 resolver 获得登记。全链同 S0 模板 15 处。
- **potyinhui 音洄升级为完整上游语义**：`phase_prepare once_per_round optional { lose_gained_skill(self, self_skill); choose_gain_other_skill(self, self_skill); }`——你可以失去一个因〖音洄〗获得的技能（可取消、无则静默），然后获得一名其他角色技能名单中的一个。近似→完整，approx 池 -1。
- **行为测试**（original_bingshi_potyinhui_gain 3/3 绿）：S0 增益链、每轮一次闩、S1 失去臂端到端（获得→登记→失去臂清单→确认移除→登记出清）。
- **部署与对账**：bingshi npack ABI 675→676 四处同步+store 槽重装核验；27 文件 ABI 断言 675→676 脚本化对账（单文件源夹具 pot_yuji 维持 570 不动）。

ADR 0646 §7 状态回写：设施 S0+S1 均已落地（领号 675/676），potyinhui 完整
建模；池内剩余登记债：potheyun 合韵（同技能过滤+失去任意技能，需技能名单
查询面）、sm_kulian 竞赛系统、访客族。

回归：engine-core **3469 通过/0 失败/1 忽略**（+1 新增失去臂测试）；workspace
全量 **4258 通过/0 失败/1 忽略**。ledger：**complete=5018/approx=490**（音洄
转完整，approx 池 -1）。

## 本轮：§7 技能夺取设施 S2 落地·合韵完整建模（2026-08-28 第一百四十五批）

**领号 ABI 677**（领取表+ABI-MAX 已更新，host 676→677）：

- **RESOLVE_POTHEYUN**（tag 700/Cont 285）：`resolve_potheyun(player, self_skill)`——三段链（共用 Cont 285，stage 位）：段0 选一名与持有者共享「可获得技能」的其他存活角色（仅列有共享者）→ 段1 选一个共同技能 def（升序）→ 段2 持有者 RemoveSkill 该技能+目标摸 2。可共享判定 `!locked && !non_gainable && !lord_only`（上游 charlotte 排除在元数据中无标志，以此近似并注记）。全链同模板 15 处。
- **消费者接线：bingshi potheyun 合韵**（原「选一名其他角色令其摸两张」近似）→ `active play_phase uses_per_turn 2 { resolve_potheyun(self, self_skill) }`——完整上游语义：限两次、共享过滤、失去共同技能、目标摸 2。
- **行为测试**（original_bingshi_potyinhui_gain 4/4 绿）：新增合韵用例——双活体夹具（双方共享 shared_a、对方独有 shared_b、持有者 locked_c）：段1 名单仅含 shared_a（locked 被过滤、非共享不入），失去后 locked 技能原样、目标摸 2。
- **部署与对账**：bingshi npack ABI 676→677 四处同步+store 槽重装核验；ABI 断言 676→677 脚本化对账。

ADR 0646 §7 状态回写：设施 S0/S1/S2 三期均已落地（领号 675/676/677），
音洄+合韵完整建模；池内剩余登记债：sm_kulian 竞赛系统、访客族、
dcwumei（待选择性状态快照设施）。

回归：engine-core **3469 通过/0 失败/1 忽略**（+1 新增失去臂测试）；workspace 全量 **4258 通过/0 失败/1 忽略**。

## 本轮：上游 v1.11.6 增量闭包——31 新将 74 技目录迁移（2026-09-02 第一百四十六批）

上游 `7ed3facb..ad3bc6ca`（57 提交，v1.11.5→v1.11.6）当日迁移闭包，零新原语批次：

- **31 新将落位**：offline 14（龙血玄黄包司马昭/曹髦/成济/神马超/贾充/文钦/司马孚/司马伷/毌丘俭/李昭焦伯 + 线下项羽/神姜维/魔姜维/王战张绣）、xianding 7（崔芷/陆文漪/谋程普/刘盼兮/邓晚棠/威马腾/孙和）、collab 3（想学刘禅/好学吕蒙/博学曹冲）、onlyOL 2（界钟会/神黄忠）、tw 2（幻小乔/幻法正）、sp 1（王皑）、clan 1（族陆昙，与族陆景/族陆郁生共享既有 clanzelie 移植）、shiji 1（手杀孙邵，diff 顶层键扫描漏网、由 sync 后 source_synced 残差追补——**多将批之后必须复核 un-ported 残差**）。
- **74 技全部诚实 noop**（逐技注记上游语义 + 阻塞机制）：虚拟牌名转化（boxue/clankuanqu/psjiucheng）、使命技状态机（lxweisi）、expansion 置牌（olquanji/twfangyan/dcdaijia）、额外回合+死亡分支（psranjin/pshunjin，ADR 0646 缺口）、护甲（lxzuoyou/lxshishou）、转换技分支（twyishi）、观牌堆顶（qiongshe/dcxunxiang/starzhongyan）、伤害属性改写（lxyuli/psfenzhi/psmoyan）、装备栏废除（lxcuizhen/lxkuili）、动态数值查询/历史门槛（clananliu/lxjimie/lxxiongsi/qinmian）等均无原语。含 8 件 derivation 配套（lxjuejin/lxwuhui/lxzhaoxiong/chixueren_skill/olpaiyi/twyishi/孙邵三技）。
- **注记纠错**：sp2 starzhongyan 忠言——原注「audio-only 无规则面」失实，上游为「亮牌堆顶三张与目标交换一张手牌」真实主动技（e044ad1d 又修其交互），改登记 deck-top 债诚实 noop。
- **catalog 闭包**：characters 2526→2556、skills 6068→6140，**2556/2556 + 6140/6140 全 ported**；8 包 npack 重建四处同步（clan/collab/offline/onlyOL/shiji/sp/sp2/tw/xianding）。host ABI 677 不变，各包 rule_vm_abi_version 均未抬升（noop 技能零原语）。
- **既有技能修复核对**：本轮 57 提交对既有移植源码的漂移（mobile 195/offline 668 等）经查为 Step Content 重写与异步化（表现层，不动规则面）；命名修复中 olsiqi（联机对话框取 player 时机）、starzhongyan 交换交互、裴秀茂著（地图技能登记）均不触及已建模规则语义，移植不动。
- **测试对账**：顶层计数 collab 146→152/85→88、offline 791→827/365→379、onlyOL 182→188/81→83、sp 727→728/216→217；深层计数 offline modifier_index 85→121、onlyOL 167→173（noop 每技 +1 modifier）；sp PACKAGE_LOGIC_HASH ×3 重录（[204,240,52,…]）；canonical_state_hash 重录 ×2（olchouniu/olsishe，行为断言先验绿）。roster 平移种子敏感三例：xianding batch16 种子平移（20_260_837→20_260_840，开局撞**必选** choice、cancelled 不合法，排空不适用）、batch42 setup 排空（qimen 范例）、sp_batch ad/af/w started() 统一排空；pxwenxian 温县断言改相对值（+1 前后，座位分配的武将基础上限随 roster 变化）。**教训：sed 替换含 `[` 的字面量会触发字符类灾难（一次误替换 1259 行，逐行反转复原）——哈希常量替换必须走纯文本 replace。**
- **ledger**：5582 技 complete=5092 / approx=490 / noop=2123（+74 全入 complete，approx 池不变）。

回归：engine-core **3470 通过/0 失败/1 忽略**；workspace 全量 **4259 通过/0 失败/1 忽略**（include_bytes 嵌入 npack 的测试二进制全量重编译后与基线持平，0 失败）。

## 本轮：批 147——从谏/好学零新原语真实语义重写（2026-09-03 第一百四十七批）

批 146 候选登记的两件 noop 晋升为完整语义，零新 Effect 原语、零 ABI 变化：

- **sclcongjian 从谏（王战张绣）**：`damage_before` 双臂（owner_is_target / owner_source）+ `current_phase_player()` 玩家比较 + `modify_current_damage(self, 1)`（ABI 562）。回合内受到伤害+1 / 回合外造成伤害+1，完整建模无残差。**DSL 无不等算符**（lexer 只有 `< > == &&`，`!` 进代码体即 `unexpected byte 33`，注释内安全），回合外臂以空 then + else 反转表达（rezishou 惯例）。
- **haoxue 好学（好学吕蒙）**：获得臂按事件源拆双臂（`cards_drawn` 摸牌 + `cards_moved` 入手牌区）→ `recover_hp(self, 1)`；流失臂 `cards_moved` → `lose_hp(self, 1)`。新注册过滤器 `owner_gained_cards_during_play_phase_wounded` / `owner_discarded_outside_play_phase`（catalog 枚举尾部 / registry map_filter / dispatch eval / typecheck 允许名 / lsp doc 五处，零 ABI——过滤器以字符串入 npack）。完整建模无残差。**新坑：触发注册按事件键路由，单一 cards_moved 臂收不到 CardsDrawn 事件**——获得类技能必须按事件源拆臂。
- **行为测试**（original_offline_sclcongjian 4 用例 + original_collab_haoxue 4 用例）：回合内/外 × 造成/受到四象限、获得/弃置 × 阶段内/外四象限、满血不回复、单事件单回复。两坑：①bootstrap 武将面板随座位分配变化——hp/max_hp 断言必须相对值（pxwenxian 同坑再犯）；②canonical_state_hash 随 collab npack 内容变化再漂移（olchouniu/olsishe 重录，行为断言先验绿，batch AO 同款）。
- **部署与对账**：offline npack triggers 578→580 / modifiers 121→120 / modules 715→717（ABI 674 不变）；collab triggers 97→100 / modifiers 43→42 / modules 125→128；四处同步 + store 无需重装（无 store 消费者变更）。10 个 offline 冻结断言文件三值对账。
- **ledger**：noop 2123→2121（两件转真实实现），complete=5092 / approx=490 不变。

回归：engine-core **3478 通过/0 失败/1 忽略**（+8 为两文件新用例）；workspace 全量 **4267 通过/0 失败/1 忽略**（+8，与基线持平）。

## 本轮：批 148——sp audio-only 注记全量复核（2026-09-03 第一百四十八批，纯注释层）

对 sp 包 262 个含「audio-only/音频」注记的诚实 noop 文件做逐块上游复核（batch AL 误标核验管线复用：按技能名定位现上游 `sp/skill.js` 块、扫描双缩进规则键，**弃用注记内行号**——57 提交后行号已漂移，抽验时按行号定窗口会命中相邻技能）：

- **误标 230 件证伪**：上游均含真实规则体（trigger/enable/content/mod/filter/subSkill 等），注记行「标记型技能（audio-only）无规则面」失实——且多数文件头注本已如实描述语义（如 bifa「出牌阶段结束时交牌作笔伐牌」），仅内层 noop 理由行误标。全部改为「含真实规则体（键摘要），原注记系批 148 复核证伪——诚实 noop 待真实重写」措辞。这构成 sp 后续重写批次的现成候选池（jici/bifa/benyu/zengou/junbing/aocai 等中低复杂度者居前）。
- **维持终态 5 件**（jingong3/fengpo3/shenxian2 等真 charlotte/空体）+ **组技能壳 3 件**（luoyan/yongsi/jieyuan——group 指向子技承载，子技另评）。4 件（huanshi_three/oldxiemu/shefu2/xianfu3）内层注记本已如实、仅头注残留「音频/辅助」，头注已改。jugu 多块文件首块为已实现真实技，跳过。
- **零影响实证**：重建 sp npack 与部署件 `cmp` 字节一致（注释不进 IR/logic_hash），无需部署、无测试影响、无测试引用这些文件（include_str 扫描零命中）。
- **方法论沉淀**：①误标核验必须**按名取块**而非按注记行号；②「规则键」判定含 subSkill/group（组壳单列，子技独立评）；③keys=[]（含缩进差异文件）一律判无规则面；④多块文件仅当全部块同类才可批量改写。sp 与 mobile(113)/jsrg(153) 误标模式同源：2026-08-11 批量的 400 字符窗口扫描把 `audio: 2` 在前、规则键在后的技能整批误入 audio-only 桶。

回归：无代码/数据变化（字节级），沿用批 147 的 engine-core 3478/0/1 与 workspace 4267/0/1。ledger 数值不变（5582 技，complete=5092/approx=490/noop=2121）。

## 本轮：批 149——啖酪零新原语重写·AOE 豁免时序缺口定界（2026-09-03 第一百四十九批）

批 148 候选池首个晋升，**danlao 啖酪（杨修）**：

- **摸牌臂完整建模**：`card_targeted` 触发 + 本批新注册过滤器 `owner_in_multitarget_sha_or_trick_targets`（CardTargeted + ctx.target_count>1 + 杀或普通锦囊；TriggerContext 自带 target_count/card_kind，CardTargeted 上下文经 target_contexts_from_event 从 CardUsed/CardTargetAdded 派生）→ `draw_cards(self, 1)`。过滤器五处注册，零 ABI。
- **近似残差（登记债）**：「令此牌对你无效」臂不可达——引擎目标豁免原语 `cancel_incoming_card_use`（ABI 643，ADR 0608 杜达元帅1 模板）依赖 sha 响应窗（ResumePlayedSha）或 wuxie 挂起载荷，而南蛮/万箭走 `begin_aoe_queue` AOE 队列结算，触发效果执行时 `resume_kind=None`、无挂起期可排除（实测 resume_kind=None、伤害照常、graceful no-op）。曾试 `choose_add_or_remove_current_card_target` 门限即拒（card_used-only）且其空调整分支会误移技能。需 AOE 队列逐目标触发面，登记债；ledger 计 approx（+1）。
- **选型负样本**（供后续批避坑）：jici=拼点点数改写（compare num 改写无原语）、bifa/expansion 置牌债、olwenren=active 多目标逐目标条件摸牌（active 体单次执行无循环，event_source 仅首目标）、hengjiang=跨玩家临时 modifier、olzhijie/olhuiqi=回合全局历史查询/额外回合缺口。
- **行为测试**（original_sp_danlao 3 用例）：契约（event=card_targeted、过滤器名、optional、无 hand_limit modifier）、南蛮多目标确认后摸 1（近似口径：owner 仍受伤）、单目标杀不过滤不触发。
- **部署与对账**：sp npack triggers 248→249 / modifiers 487→486 / modules 268→269（ABI 647 不变）；四处同步；PACKAGE_LOGIC_HASH 重录 [75,7,73,…]（dingpan/hengzheng/ziyuan 三测试）+ modules/trigger/modifier 计数断言同步。
- **ledger**：5582 技 complete=5091 / approx=491 / noop=2120（danlao noop→approx）。

回归：engine-core **3481 通过/0 失败/1 忽略**（+3 为 danlao 用例）；workspace 全量 **4270 通过/0 失败/1 忽略**（+3，与基线持平）。

## 本轮：批 150——贲育/虚待零新原语重写（2026-09-03 第一百五十批）

批 148 候选池第 2/3 件晋升，双技批次：

- **benyu 贲育（程昱）完整重写**：`damage` 触发 + `owner_is_target_other_source_alive` 过滤器 + `let` 捕获（计划期求值 `min(5, hand_count(event_source))` 与 `need`）+ `choose(1,2)` 分支（kunfen 惯例）+ `draw_cards` 表达式张数（摸至 min(来源手牌,5)，cajujun 惯例）+ `discard_hand_equip_expr`（ABI 646）+ `deal_damage`。近似注记：选择面简化（optional 确认+强制二选一，效果面条件等价）、分支②确定性弃牌非任选、可弃不足时全弃仍伤害（上游不足则不发动）。
- **spolxudai 虚待（SP 庞德）近似重写**：`card_responded`（owner_is_source=响应者）/ `damage`（owner_is_target）双臂 + `choose_other_add_skill(self, 煮酒def)`（ABI 675 选其他角色授指定技——批 147 时误记为反向原语，实为正向授予）+ `remove_skill(self_skill)` 限定技自移除（tuteng3 惯例）。近似注记：useCard&&respondTo 角落窗口未单列、目标名单不排除已有煮酒者、目标选择可取消（上游强制）。**解锁要点：choose_other_add_skill 为「选他人授指定技」正向原语——twtongce 通策等「授予指定技」族候选应重新评估。**
- **选型负样本**：zengou=挂起牌 all_excluded 排除（danlao 免除臂同族债）、junbing=他人选择+双向全手牌转移、quji=active 动态张数多目标分配+黑牌条件失体力。
- **行为测试**（original_sp_benyu 2 用例 + original_sp_spolxudai 2 用例）：贲育摸至 min(6,5)=5 臂、弃 n+1 张+对来源 1 伤臂；虚待受伤端到端授煮酒+自移除+二次不触发、响应臂确认打开。
- **部署与对账**：sp npack triggers 249→252 / modifiers 486→484 / modules 269→272（ABI 647 不变）；四处同步；PACKAGE_LOGIC_HASH 重录 [198,166,31,…] ×3 + 计数断言同步。
- **ledger**：5582 技 complete=5089 / approx=493 / noop=2118（benyu/spolxudai 均带近似注记：noop -2、approx +2）。

回归：engine-core **3485 通过/0 失败/1 忽略**（+4 为两文件新用例）；workspace 全量 **4274 通过/0 失败/1 忽略**（+4，与基线持平）。

## 本轮：批 151——通策零新原语重写·授予族重评（2026-09-03 第一百五十一批）

按批 150 的 `choose_other_add_skill` 正向语义发现，重评「授予指定技」族候选首件：

- **twtongce 通策（幻法正，tw 包）近似重写**：`game_started` 触发（等价上游 enterGame/phaseBefore 开局窗口，xiaoxi 先例；forced 无 optional）+ `add_skill(self, 遗计def)` + `choose_other_add_skill(self, 遗计def)`（ABI 675）。近似注记：①交牌臂（其交给你一张牌）无「选定角色交牌」通用原语省略；②目标选择为强制（resolver 拒收 cancelled——`resolve_resolve_hfjieying` 显式 `|| cancelled → Illegal`，与原消费者 hfjieying 语义一致，上游 chooseTarget 可取消）；③被授〖遗计〗本体当前为诚实 noop（转换技分支债同池）——授予关系真实，本体重写后自动生效。
- **原语语义勘误入册**：`choose_other_add_skill` = 选其他存活角色（名单含已有该技者）+ 强制 AddSkill，**无取消路径**——「可取消授予」族需另立变体（现 2 例：twtongce/twtongce 类、达产门槛 3 例前登记）。
- **行为测试**（original_tw_twtongce 2 用例）：GameStarted 派发 → owner 获遗计 + 目标选择（forced 流：dispatch Ok(效果请求) → run_effects 中开选择，**非 dispatch 期**）→ 授予 other；契约（game_started/owner_alive/非 optional + 无 hand_limit modifier）。
- **部署与对账**：tw npack triggers/modifiers/modules 现值 144/313/158（+1 触发臂 / -1 noop modifier / +1 module，ABI 673 不变）；四处同步；tw 无冻结 hash/计数断言（批 AL 教训沿用）。
- **ledger**：5582 技 complete=5088 / approx=494 / noop=2117（twtongce noop→approx）。

回归：engine-core **3487 通过/0 失败/1 忽略**（+2 为通策用例）；workspace 全量 **4276 通过/0 失败/1 忽略**（+2，与基线持平）。

## 本轮：批 152——镇南零新原语重写·池级形状扫描（2026-09-03 第一百五十二批）

- **池级形状扫描方法论落地**：对 sp 误标池剩余 223 件批量提取上游体形状（行数/触发事件/效果关键词），筛出 ≤40 行 trigger 型 34 件逐一复核。新增负样本定界：oldingcuo（触发限次无通用机制+摸牌颜色检查）、bushi/yishe（expansion）、yangkuang（虚拟酒 use 门禁）、olyintian（全局首次回复历史查询）、olfusong（死亡 forceDie+目标自选授技）、xinfu_shajue（dying 处理区牌查询）、duanxie（active 多目标逐目标横置）、olderu（猜牌博弈）、olmeiyan（辩论族）、spmanwang（分层里程碑+mark 封顶）、lianzhu（give+目标二选一组合）、wanwei（拼点结果改写）、xinfu_dianhua（观星原语缺口，池内最短件 13L）。
- **xinfu_zhennan 镇南近似重写**：`card_targeted` 触发 + 本批新注册过滤器 `owner_targeted_by_nanman`（CardTargeted + `card_kind == Some(CardKind::TrickNanman)`）→ `choose_other_damage(self, 1)`（batch G 先例）。近似：随机 1-3 点伤害定为固定 1 点（引擎无随机数原语）。过滤器五处注册，零 ABI。
- **对账工艺改进**：PACKAGE_LOGIC_HASH 重录不再从 panic 输出抄录——直接从重建 npack 的 `manifest.logic_hash` 读取生成字面量（消除转抄错位风险）。
- **行为测试**（original_sp_xinfu_zhennan 2 用例）：契约（card_targeted/owner_targeted_by_nanman/optional）+ 南蛮管线端到端（确认→选目标→镇南 1 伤+南蛮本体 1 伤结算）。
- **部署与对账**：sp npack triggers 252→253 / modifiers 484→483 / modules 272→273（ABI 647 不变）；四处同步；hash/计数三测试同步。
- **ledger**：5582 技 complete=5087 / approx=495 / noop=2116（镇南 noop→approx）。

回归：engine-core **3489 通过/0 失败/1 忽略**（+2 为镇南用例）；workspace 全量 **4278 通过/0 失败/1 忽略**（+2，与基线持平）。

## 本轮：批 153——三池扫描实证·鸦翔补授臂（2026-09-03 第一百五十三批）

- **三池简单面挖尽实证**（批 152 扫描器复用）：①jsrg 池 122 件真实体 noop 中 ≤40 行 trigger 型 13 件全部逐件定界硬缺口（jsrgshenji 目标顺序重排、jsrgzhaoxiong 觉醒换势力+换技、jsrgpianchong 判定+全局弃牌计数、jsrgliedu 目标属性化 directHit、jsrgweizhui 他人 view_as 转化、jsrgcangchu 回合获牌历史、jsrgzhoulin storage+距离、jsrgtianyu per-card getIndex、jsrgjiemeng group 计数值源、jsrgyangge 密诏继承、jsrghezhi/yanhuo 继承壳维持终态、xumou_jsrg_temp 蓄谋 temp 已闭环）；②mobile 池 189 件中 ≤40 行 trigger 型 59 件——过滤后全部为子技/伴随技（依赖本体 storage/expansion/继承，pcaudio_*/friend*/yizan/tiansuan2_ai 等按批 150 教训不计独立消费者）；③sp 池见批 152。**零原语简单形状在三池系统性挖尽，后续晋升需组合原语或达门槛立项。**
- **yukito_yaxiang 鸦翔（KEY 往人）补授臂**（近似池残差缩减）：批 150 确认 `add_skill(player, def)` 支持任意目标——补 `effect add_skill(event_player, 2446245954)`（〖终愿〗misuzu_zhongyuan，已 ported），上游 content 尾段 `target.addSkills` 对齐。近似仅剩「不变身」（reinit 无原语）。
- **行为测试**（original_key_yukito_yaxiang_grant 2 用例）：EnteredDying 合成派发 → 确认 → 回复至 3 + 获终愿 + 自卸；owner 自身濒死不触发（owner_alive_not_target）。
- **部署与对账**：key npack modules/triggers/modifiers 110/93/177（补 effect 行不增模块，logic_hash 变更但 key 无冻结字面量 hash——运行时重算对比全绿）；四处同步。
- **ledger**：5582 技，类属不变（鸦翔保持 approx，残差缩减）。

回归：engine-core **3491 通过/0 失败/1 忽略**（+2 为鸦翔用例）；workspace 全量 **4280 通过/0 失败/1 忽略**（+2，与基线持平）。

## 本轮：批 154——近似池抽样核验（2026-09-03 第一百五十四批）

按批 142 抽样审计模式对 approx 池核验 4 件，全部维持（无注记失实、无可晋升缺口）：

| 条目 | 维持理由 |
|---|---|
| shiji jianzhan 渐战 | 双目标「手牌数大于」关系约束（complexTarget）不可表达；目标自选用杀/令摸牌分支需逐目标选择串联 |
| shiji mjchenshi_target 瞑瞑·陈氏·目标方 | 「冰临城下」牌名限定过滤器可单立（单件不达门槛）；交牌+牌堆顶查杀置弃仍无原语 |
| key yuiko_dilve 底略 | 已近完整（拼点后获进弃牌堆的拼点牌）；残差仅「牌堆底取牌替换拼点牌」（onCompare）与训诫变体 |
| sp leiluan 累卵 | view_as 张数=横置角色数——动态张数 cost 需白名单扩展；本轮未用基本牌名剔除无轮次历史查询 |

**立项门槛状态更新**：①「可取消授予」变体仍 2 例（twtongce/spolxudai 系）——第三例候选搜寻未果（上游「可取消的指定技授予」面稀少），维持登记；②AOE 队列逐目标触发面（danlao 免除臂）：根因已定界——触发效果执行早于 use_card_then_wuxie 挂起（resume_kind=None 实测），解锁需 resolution 暂存排除 + AOE 队列逐目标检查（引擎改动，单消费者，暂缓）；③拼点族原语（choose_hand_compare win/lose 挂胜负双方）与「发起者赢才触发目标效果」语义不匹配（tanhu 类），登记语义缺口。

**负样本补充定界**：offline 拼点三占位件 huoxin/jun_zhaoshou/tanhu（双目标拼点/多卡赠予+转拼点/赢挂 temp 技——均超出现有拼点原语语义）；横置近似族残差面核实（ymsaxue 双目标火伤分配、nao_wanxin 翻回/解锁、clanbaozu 已高保真）。

回归：无代码/数据变化，沿用批 153 的 engine-core 3491 通过/0 失败/1 忽略与 workspace 4280 通过/0 失败/1 忽略。ledger 数值不变（5582 技，complete=5087/approx=495/noop=2116）。

## 本轮：批 155——danlao 免除臂落地·AOE 挂起载荷排除补全（2026-09-03 第一百五十五批，引擎批）

批 149 登记债的结构性解锁，**danlao 啖酪升级为完整建模**（「令此牌对你无效」达成）：

- **根因精确定界**（取代批 154 的暂存方案）：触发效果执行时 resume_kind 实为 **WuxieWindow**（已挂起），真正缺口是 `current_card_target_adjustments` 的 WuxieWindow 分支只实现了 TIAOJIYANMEI/TIESUO 的 removable 计算——**NANMAN/WANJIAN 载荷直接返回空表**。
- **引擎补全两处**：①adjustments 补 NANMAN/WANJIAN removable 分支（payload[7]=count、[8..]=targets，≥2 才可移除，与 exclude_saved_card_target 的 (7,8) 口径一致）；②防御性 deferred 路径：resume_kind=None 时 cancel_incoming 的不可排除分支把 (card, player) 暂存入 `GameState.deferred_card_exclusions`（#[serde(default)] 新字段，GameState 单点构造），`advance_aoe_queue` 逐目标消费跳过（藤甲式先例）。
- **共用原语副作用注记**：cancel_incoming 内嵌杜达元帅的「获得实体牌」步骤——啖酪免除后额外获得南蛮实体牌（上游无），纯排除需新变体原语（登记债，暂缓）。曾试把 gain 拆到 DSL 层（杜达件补 gain_event_cards）——gain 原语只取已进弃牌堆者，取消时点牌仍在结算区（时序不可达），已回滚。
- **行为测试**（original_sp_danlao 3 用例）：南蛮多目标 → 确认 → owner 免除（hp 不变）+ 摸 1 + 获实体牌 1 + 其他目标正常受伤；单目标负例；契约。
- **部署与对账**：sp npack 触发 253 / 修饰 483 / 模块 273（ABI 647 不变）；shahai 重建（dudayuanshuai1 先补 gain_event_cards 后回滚，最终字节与批 152 一致性以重建为准）；sp PACKAGE_LOGIC_HASH 第四次重录（从 manifest 直读 [163,124,211,…]）。
- **ledger**：5582 技 complete=5087 / approx=495 / noop=2116（danlao 含近似注记——共用原语获得实体牌副作用——保持 approx 轴，noop 轴早已移出；残差从「免除臂未建模」缩至「多获实体牌」）。

回归（2026-09-03 会话复跑补记）：engine-core **3491 通过/0 失败/1 忽略**（danlao 用例
原位改写，计数与批 154 持平）；workspace 全量 **4280 通过/0 失败/1 忽略**（与基线持平；
本行数字由批 157 后复跑 4284/0/1 减去 sbjiang 新增 4 用例回推，两次运行全绿）。


## 本轮：批 156——sb 包 audio-only 注记全量复核（2026-09-03 第一百五十六批，纯注释层）

对 sb（界限突破）包 sb_meta.skill 的 109 条「audio-only 无规则面」注记做逐名复核（批
148 管线复用：按技能名定位现上游 sb/skill.js 块、扫双缩进规则键，弃用注记内行号——
上游快照行号已漂移）。该文件是 sp/mobile/jsrg 之外最后一个 audio-only 批量误标点
（其余 7 个 *_meta.skill 文件 grep audio-only 零命中）：

- **误标 105 件证伪**：当前上游块均含真实规则体（trigger/enable/viewAs/mod 等键），
  注记「标记型技能（audio-only）无规则面」失实。全部改为「含真实规则体（键摘要），
  原注记系批 156 复核证伪——诚实 noop 待真实重写」措辞。这构成 sb 后续重写批次的
  现成候选池（sbjiang/sbqiaoshi/sbzishou/sbzhengzi 等 trigger 型居前）。
- **改判 4 件组/继承壳**：sbluanwu/sbjinjiu/sbliuli（group/inherit 指向子技承载）与
  sbqingzheng（inherit+persevereSkill）——壳维持 noop，子技另评（批 148 组壳惯例）。
- **既有诚实注记 9 件不动**（sbqiaobian/sbwansha_rewrite 等本就登记「含真实规则体」）。
- **零影响实证**：批 156 与批 157（sbjiang 晋升）合并重建 sb npack 后差分核对——
  全包唯一语义增量为 sbjiang 一条触发/一条 noop modifier 移除，其余 117 件注释变化
  不进 IR/logic_hash（注释被 lexer 跳过）。
- **方法论沿用**：按名取块而非注记行号；规则键判定含 group/inherit（组壳单列）；
  keys 无规则键者才维持 audio-only 终态。



回归：批 156 纯注释层无独立运行（npack 差分即证据）；与批 157 合并回归见下。

## 本轮：批 157——sbjiang 激昂主臂零新原语重写（2026-09-03 第一百五十七批）

批 156 复核证伪池首件晋升，**sbjiang 激昂（界孙策）主臂**，零新 Effect 原语、零新
过滤器、零 ABI 变化：

- **主臂完整建模**：`card_used` 触发 + 既有过滤器 `owner_source_or_targeted_by_juedou_
  or_red_sha`（catalog 既有变体，与 shenhua jiang 经典激昂、diy noname_jiang、key
  yui_jiang 同款接线）→ `draw_cards(self, 1)`。使用【决斗】或红色【杀】指定目标后、
  或成为其目标后可摸一张牌，无残差。
- **近似残差（登记债）**：两个 group 子技未建模——sbjiang_add（使用【决斗】时可为
  其额外指定一个目标并失去 1 点体力，依赖 card-target 追加+体力代价组合面）与
  sbjiang_qiben（出牌阶段限 N 次将全部手牌当【决斗】使用，N 随「激昂」标记与吴势力
  存活数动态变化，动态次数 view_as 设施缺口）。「激昂」标记由界制霸（sbzhiba）侧
  写入，本臂不消费。
- **行为测试**（original_sb_sbjiang 4 用例）：契约（card_used/过滤器名/optional/无
  hand_limit modifier）、owner 出红【杀】→ 确认 → 摸 1、owner 被红【杀】指定 →
  确认 → 摸 1、黑【杀】负例不触发。
- **部署与对账**：sb npack 触发 +1 / noop modifier -1（ABI 不变）；与批 156 注释层
  合并重建后部署 content 与 extensions 两处；sb 无冻结 hash/计数字面量断言
  （character_index 54 不变，closure 断言 def 早已在册）。
- **ledger**（重跑 gen-fidelity-ledger.py 实测）：complete=5086 / approx=496 / noop=2116。
  sb_meta.skill 由「无近似注记」转「含近似注记」（主轴 complete→approx，-1/+1）；
  noop 轴按文件计，sb_meta 仍含 117 条 noop 注记故不变——sbjiang 本体已由 noop 池
  移出（该文件 noop 注记剩余 117 条，批 156 复核后 105+4+9=118-1（sbjiang 晋升）=117）。

回归：engine-core **3495 通过/0 失败/1 忽略**（+4 为 sbjiang 用例，3491+4 对账吻合）；
workspace 全量 **4284 通过/0 失败/1 忽略**（+4，与基线持平）。ledger：5582 技，
complete=5086 / approx=496 / todo=0 / noop=2116。

## 本轮：批 158——mobile/jsrg audio-only 误标残余收口（2026-09-03 第一百五十八批，纯注释层）

批 156 sb 复核后对全仓 audio-only 措辞做残余清扫：mobile 6 文件 + jsrg 2 文件仍带
「标记型技能（audio-only）无规则面」旧注记（mobile/jsrg 主池已在此前批 148 型复核中
纠正，此为漏尾）。按名取块逐件核验：

- **误标 4 件证伪改写**：friendfangqiu（limited+trigger 宴策小游戏预测分支）、
  mbqianlong（trigger+forced+group 千龙面板族）、mbxishang（trigger+mark+group）、
  jsrgfuzhen（准备阶段视为对至多三名角色使用雷【杀】）——措辞改「含真实规则体
  （键摘要），批 158 复核证伪——诚实 noop 待真实重写」。
- **改判 1 件继承壳**：jsrgyanhuo（inherit 指向家族本体，壳维持 noop）。
- **措辞改判 1 件空体**：mbweitong（zhuSkill+trigger 键在但 content 为空体——
  「无规则面」成立，改判空体终态）。
- **维持终态 2 件**：friendgongli / yizan_respond_shan（上游仅 audio 键，真 audio-only）。
- **零影响实证**：mobile/jsrg npack 重建后与部署件 `cmp` **字节一致**（注释不进
  IR/logic_hash），无需部署、无测试影响、ledger 数值不变。
- **sp/sb/mobile/jsrg 四池收口**；全仓仍有 6 包 826 件旧措辞待复核（xianding 291 /
  sp2 152 / tw 145 / onlyOL 127 / refresh 91 / key 17）——批 159 已全量复核改写。

回归：无代码/数据变化（字节级），沿用批 157 的 engine-core 3495/0/1 与 workspace 4284/0/1。

## 本轮：批 159——六包 826 件 audio-only 旧注记全量复核（2026-09-03 第一百五十九批，纯注释层）

批 158 残余清扫发现的更大面：xianding/sp2/tw/onlyOL/refresh/key 六包共 826 件仍带
2026-08-11 批量的「标记型技能（audio-only）无规则面」旧措辞。批 148 管线全量复跑
（按名定位现上游块 + 双缩进规则键扫描）：

- **误标 784 件证伪改写**（键摘要入注记，措辞同批 148/156）：xianding 281、sp2 151、
  tw 129、onlyOL 122、refresh 84、key 17。六包「含真实规则体」注记池由此建立，
  为后续晋升批次的最大现成候选面。
- **改判 13 件组/继承壳**（group/inherit 指向子技承载，壳维持 noop，子技另评）。
- **改判 1 件快照债**：xianding wudou——当前上游全树 grep 零命中（原注记行 3461
  属旧快照），移植侧头部注释已如实登记其双触发语义，按「上游已移除」改判。
- **维持终态 27 件**（上游块仅 audio 键：tw 15、xianding 9、onlyOL 1
  kunfen_ol_sb_jiangwei、mobile 2）+ qixian.skill 文件头组声明（sp/starzhonggu
  同约定文件，非逐技注记）。
- **零影响实证**：key/onlyOL/refresh/sp/sp2/tw/xianding 七包 npack 重建后与部署件
  `cmp` 全部**字节一致**（注释不进 IR/logic_hash），无需部署、无测试影响。
- **audio-only 误标病全仓收口**：sp（批 148）→ sb（批 156）→ mobile/jsrg 残余
  （批 158）→ 六包全量（本批）。此后全仓任何「audio-only」注记均为真终态。

回归：无代码/数据变化（七包字节级一致），沿用批 157 的 engine-core 3495/0/1 与
workspace 4284/0/1。ledger 重跑确认数值不变（5582 技，complete=5086/approx=496/
todo=0/noop=2116）。

## 本轮：批 160——fuyuan 辅袁零新原语重写（2026-09-03 第一百六十批）

批 159 复核证伪池（784 件）首批形状扫描（≤40 行 trigger 型 44 件）后首件晋升，
**fuyuan 辅袁（sp2 辛评）**，零新 Effect 原语、零新过滤器、零 ABI 变化：

- **完整重写**：card_used / card_responded 双臂（批 147 教训：按事件源拆臂）+ 既有
  过滤器 `owner_source_outside_turn_card_used_or_responded`（catalog 既有，单过滤器
  内含 CardUsed|CardResponded 双事件判定 + 回合外门槛，new_yajiao 同款）→ body 内
  `hand_count(current_phase_player())` 比较分支：当前回合角色手牌少于 owner 则其摸
  一张，否则 owner 摸一张。
- **表达式位首用**：`current_phase_player()` 此前仅用于 if 比较（clanshize/
  sclcongjian），本批首次作为 draw_cards 效果参数位——EffectDraw 走通用 emit_expr
  栈，查询表达式在效果参数位可用得到运行时实证。
- **近似（登记债）**：上游 respond 含打出/响应两类窗口，DSL card_responded 以引擎
  CardResponded 事件面为准，角落窗口差异登记观察。
- **行为测试**（original_sp2_fuyuan 4 用例）：契约（双臂事件/过滤器/optional/无
  hand_limit modifier）、回合外响应+当前回合角色手牌少→其摸 1、回合外响应+手牌
  相等→owner 摸 1、自己回合内使用被过滤器拦截不弹确认。
- **部署与对账**：sp2 npack 触发 58→60（+2 双臂）/ noop modifier 248→247 / 模块
  64→66（ABI 562 不变）；四处同步（content + extensions）；全部 26 个 sp2 npack
  消费者测试文件重跑绿（含 juntun committed-vs-compiled logic_hash 重算对比）。
- **yinyi 隐翼（sp2）同批第二件晋升**：damage_before + 本批新注册过滤器
  `owner_targeted_source_hp_hand_differ_once_per_turn`（六点注册：catalog 尾追/
  registry/dispatch eval/turn_limit 名单/typecheck 允许名/lsp 文档；零 ABI——过滤器
  以字符串入 npack）→ `prevent_current_damage()`（dccibei 同款无参）。锁定技无
  optional 确认；每回合限一次由过滤器 `_once_per_turn` + turn_limit 名单承载
  （猎族 S1 坑位：漏登记=once 恒不生效，本批已登记并测试验证第二击照常结算）。
  近似：上游 hasNature("linked") 排除（铁索传导伤害不触发）未建模——引擎无
  linked-nature 判定面，登记债。
- **yinyi 行为测试**（original_sp2_yinyi 3 用例）：契约（damage_before/过滤器名/
  非 optional/无 hand_limit modifier）、体力+手牌双不同→首击防止 hp 不变+第二击
  照常（每回合一次实证）、手牌相等负例直通。
- **部署与对账（两件合并）**：sp2 npack 触发 58→61（+2 fuyuan 双臂/+1 yinyi）/
  noop modifier 248→246 / 模块 64→68（ABI 562 不变）；content 与 extensions 两处
  同步；全部 26 个 sp2 消费者测试文件冻结态重跑 121/121 绿（含 juntun
  committed-vs-compiled logic_hash 重算）。
- **ledger**：complete=5084 / approx=498 / noop=2114（两件均为单技文件 noop 全移除：
  noop -2；各新增近似行 → approx +2 / complete -2；以重跑为准）。
- **测量纪律教训**：批 160 首次全量回归在运行期间被并行修改（yinyi 六点注册 +
  测试文件 + npack 重部署中途落盘）污染，得 2836/1 失败的无效数字——冻结终态
  复跑为准；后续批次一律「全部改动落盘 → 冻结 → 单次全量」排序。

回归（冻结终态复跑，exit 0）：engine-core **3502 通过/0 失败/1 忽略**（3495 + 7：
fuyuan 4 用例 + yinyi 3 用例）；workspace 全量 **4291 通过/0 失败/1 忽略**（4284+7
精确对账）。ledger：5582 技，complete=5084 / approx=498 / todo=0 / noop=2114。

## 本轮：批 161——xinfu_yisuan 翼算重写（2026-09-03 第一百六十一批）

批 159 证伪池 44 件短形状候选第二轮选型（建议四件 twyangming/olsbzhuijiao/olsbqiwu/
twliuli 逐件定界为负样本：动态张数+临时手牌上限/双历史链+伤害改写/全局首伤历史+
范围过滤器/弃他人红牌+免除+条件摸组合——详见记忆），晋升 **xinfu_yisuan 翼算
（sp2 李傕）**：

- **完整重写**：card_resolved 触发（useCardEnd 等价事件面，实体牌恰已进弃牌堆）+
  本批新注册过滤器 `owner_source_trick_resolved_own_play_phase_once_per_turn`
  （六点注册：catalog 尾追/registry/dispatch eval（owner_source + CardResolved +
  card_is_ordinary_trick + 回合=owner + Phase::Play）/turn_limit 名单/typecheck/
  lsp；零 ABI）→ `change_max_hp(self, 0 - 1)`（nsdunxing 负向表达式惯例）+
  `gain_event_cards(self)`（olhaizhu/jsrgzhenglve 先例；批 155 定界的原语边界
  「只取已进弃牌堆者」恰合上游 filterInD 语义）。出牌阶段限一次由过滤器
  _once_per_turn + turn_limit 承载。
- **近似（登记债）**：①上游 filterInD 含「仍在结算区」的实体牌，引擎 gain 只取
  已进弃牌堆者；②无实体牌的虚拟牌角落由 card_is_ordinary_trick 边界近似。
- **行为测试**（original_sp2_xinfu_yisuan 3 用例）：契约（card_resolved/过滤器名/
  optional/无 hand_limit modifier）、无中生有结算→确认→减 1 体力上限+获得实体牌
  （max_hp 与手牌双断言）、同回合第二张锦囊不重弹（每回合一次实证）。
- **负样本定界（本批新增）**：twxiongjun（需「全体技能持有者各摸一」迭代原语）、
  twyangming（动态张数=本阶段使用牌类别数+临时手牌上限 +N 双缺口）、
  olsbzhuijiao（dcjianying 上一张牌历史链+baseDamage 改写）、olsbqiwu（全局首伤
  历史+攻击范围双过滤器+红牌弃置防止组合）、twliuli（弃他人红牌+excluded 免除+
  受伤态相等条件摸三合一）。
- **部署与对账**：sp2 npack 触发 61→62 / noop modifier 246→245 / 模块 68→69
  （ABI 562 不变）；content 与 extensions 两处同步；全部 sp2 消费者测试文件
  冻结态 124/124 绿。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3505 通过/0 失败/
1 忽略**（3502 + 3 yisuan 用例）；workspace 全量 **4294 通过/0 失败/1 忽略**
（4291+3 精确对账）。ledger：5582 技，complete=5083 / approx=499 / todo=0 /
noop=2113。

## 本轮：批 162——olrenxin 仁心近似重写（2026-09-03 第一百六十二批）

批 159 证伪池第三轮选型。建议名单中段形状逐件定界为负样本（starjinglun=
starzhongyan 派生调用家族、dcpijian=大攻车卡牌 storage、twjiezhan=事件玩家虚拟
【杀】反打、qianmeng=huamu gaintag 颜色计数、sishu=判定反转家族、twshoushou=
攻击范围标记族、dcgusuan/diaoling=觉醒 storage 门槛家族、xunzhi=邻座 hp 不等+
动态手牌上限、sbjiuyuan=主公技双臂、dcchaixie/dcaichen=卡牌 storage/全局技），
44 件短形状池实质挖尽；严格单动词扫描（1111 件池、51 件过筛）再定界后晋升
**olrenxin 仁心（onlyOL ol_caochong）**：

- **近似重写**：dying 触发 + 本批新注册过滤器 `owner_alive_other_dying_has_equip_
  card`（六点注册，零 ABI；濒死者存活且 hp<=0 + owner 手牌含装备类牌或装备区
  非空——dispatch 直查 hand_cards kind.is_equip + Equip 区）→ `turn_over(self)`
  （kuiwei 同款）+ `recover_hp(event_player, 1 - hp(event_player))`（nstianji
  「回复至 1」表达式惯例；濒死时点 hp<=0 故差值恒正）。
- **近似（登记债）**：上游「弃置一张装备牌」代价未建模——optional 确认保留发动
  选择权，但确认后无代价（弃置动作与装备牌类别约束均缺）。
- **行为测试**（original_onlyol_olrenxin 3 用例，olfuli 的 hp=0+EnteredDying 直派
  事件模式）：契约（dying/过滤器/optional/无 hand_limit modifier）、手牌含装备牌→
  确认→濒死者回复至恰好 1 点、无装备牌负例过滤器拦截。
- **部署与对账**：onlyOL npack 触发 23→24 / noop modifier 173→172 / 模块 25→26
  （ABI 不变）；content 与 extensions 两处同步；**五个 onlyOL 冻结计数断言同步**
  （oljiangchi/oljuece/olqieting/oldangxian/olqingxi：trigger 23→24、modules 25→26、
  oldangxian modifier 173→172）；全部 onlyOL 消费者 78/78 绿。
- **批 162 负样本定界（10 件）**：见上文逐件理由，录入记忆。

回归（冻结终态，全部改动落盘后单次运行，exit 0；首次后台运行因会话中断被杀，
代码/npack 零改动下原样重跑）：workspace 全量 **4297 通过/0 失败/1 忽略**（4294+3
精确对账，含新增 original_onlyol_olrenxin 3 用例）。engine-core **3508 通过/0 失败/
1 忽略**由批 161 实测 3505 + 唯一新增 3 用例精确对账得出（本批单包独立计数运行
因 cargo 特征解析差异触发全量重编，耗时不经济，以对账值入册）。ledger：5582 技，
complete=5082 / approx=500 / todo=0 / noop=2112。

## 本轮：批 163——mansi 蛮肆主臂近似重写（2026-09-03 第一百六十三批）

51 件严格单动词池剩余逐一复核（zongkui=傀标记跨技族+双触发 cost 深链、dckegou=
拼点循环+牌堆极值查询、zhuihuan=授予 tempSkill+getIndex 多目标、souying=同目标
历史第 N 次+免除/获得双分支、moying=花色点数选择盘+弃牌堆查询、bmcanshi=
zongkui_mark 依赖+加目标）——全部定界负样本后，转向近似池残差缩减（批 153 鸦翔
模式），晋升 **mansi 蛮肆主臂（xianding 花鬘）**：

- **近似重写**：damage 触发 + 本批新注册过滤器 `owner_alive_damage_card_nanman`
  （第四次六点注册，零 ABI；Damage 事件 + TrickNanman，不限伤害目标——上游
  global damageEnd 全观察面）→ `draw_cards(self, 1)`。文件旧头注「无精确
  nanman-only filter」（2026-08-11）已失效——批 152 镇南立 CardTargeted 先例、
  本批补 Damage 面。
- **近似（登记债）**：①上游 frequent 口径（自动发动无确认）以锁定技无 optional
  表达；②计数标记省略（intro「已因此技能得到了#张牌」纯展示面）；③viewas 子技
  「出牌阶段限一次将所有手牌当【南蛮入侵】使用」维持债（全手牌动态 view_as
  语法缺口，与 sbjiang_qiben 同池）。
- **行为测试**（original_xianding_mansi 3 用例）：契约（damage/过滤器/非
  optional/无 hand_limit modifier）、南蛮伤害他人（owner 观察者）→摸 1、普通
  【杀】伤害负例不摸。
- **部署与对账**：xianding npack 触发 154→155 / noop modifier 398→397 / 模块
  173→174（ABI 不变）；content 与 extensions 两处同步；全部 73 个 xianding 消费者
  测试文件 272/272 绿（无冻结计数字面量）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3511 通过/0 失败/
1 忽略**（3508 + 3 mansi 用例，精确对账）；workspace 全量 **4300 通过/0 失败/1
忽略**（4297+3 精确对账）。ledger：5582 技，complete=5081 / approx=501 / todo=0 /
noop=2111。

## 本轮：批 164——shahai 三件 modifier 精确化·from_current selector 解锁（2026-09-05 第一百六十四批，引擎微改+DSL 批）

近似池残差缩减（轻措辞扫描 + 头注复活检查）：shahai 三件同型近似 upgrade 精确化——
**gjqtwuxie / swd_wuxie**（「你不能成为其他角色的延时锦囊的目标」）与 **moyao**
（「……的锦囊牌的目标」）原用不分使用方的 `card_target_cancel`，升级为
`card_target_cancel_from_current`（selector 参数化）：

- **typecheck 闸门解除**：`card_target_cancel_from_current` 此前仅接受 any selector
  （首消费者 yoshino_fail/spfangzong 均为 any 的保守限制），而 modifier eval 侧
  `CardTargetCancelFromCurrentFor` 本就完整支持 `matches_kind_and_suit`、registry
  parse 已带 selector 参数——本批移除该 any-only 分支（零 ABI、零 eval 变更）。
- **语义口径注记**：from_current 实义为「使用者是当前回合角色时取消」（for_card_
  target 传入 user==turn.current）。对延时锦囊（规则上仅当前回合角色主动使用）
  与上游「其他角色的」等价；普通锦囊「指自己」在现有牌池无实战场景。三件的
  行为面与文字面就此与上游对齐。
- **行为测试**（original_shahai_from_current_upgrades 4 用例）：契约（三件
  modifier 形状 from_current+selector）、他人延时/普通锦囊指定 owner 被取消、
  无技能对照组同牌合法（证明取消来自 modifier）。
- **部署与对账**：shahai npack 重建（三 modifier 换 token、logic_hash 变更）；两处
  同步；**original_shahai_rewrite_batch 冻结断言同步**（三件 target 名
  card_target_cancel→card_target_cancel_from_current）；全部 shahai 消费者 87/87
  绿；content-compiler 532/0 绿（限制移除回归）。
- **ledger**：轴不变（shahai_meta 文件级近似注记 182 条仍在，complete=5081/
  approx=501/noop=2111）——本批为既有 approx 件的残差缩减，不转轴。
- **选型记录**：51 件严格池最后 6 件（zongkui/dckegou/zhuihuan/souying/moying/
  bmcanshi）+ 短池余件（olchishi/twpiankuang/olsbguliang/dcpijian/starjixian/
  olsbzhitian/liangjue）全部定界负样本（跨技标记/拼点循环/授予 tempSkill+getIndex/
  同目标历史第 N 次/花色点数选择盘/expansion 家族/相位插队/CardsMoved 进向 owner
  上下文缺失）；offline 86 件 active 占位池抽查偏深（赠牌分配/猜数博弈/双人弃牌
  互动），整体留待组合原语批。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3515 通过/0 失败/
1 忽略**（3511 + 4 from_current 用例，精确对账）；workspace 全量 **4304 通过/0 失败/
1 忽略**（4300+4 精确对账）。ledger：轴不变（5582 技，complete=5081 / approx=501 /
todo=0 / noop=2111）。

## 本轮：批 165——liangjue 狼灭完整重写·CardsMoved 进向零引擎改动确证（2026-09-05 第一百六十五批）

记忆路线③「CardsMoved 进向 owner 上下文引擎批」立项复核后**降级为零改动**：
重读 TriggerContext 发现 `to_zone` 字段本就存完整 `ZoneId`（前批误记为仅 kind）——
eval 侧 `state.zones.get(to_zone)` 直查 owner，无需任何引擎扩展。据此晋升
**liangjue 狼灭（sp2 张横）**：

- **完整重写**（无残差，直进 complete 池）：cards_moved 触发 + 本批新注册过滤器
  `owner_black_card_moved_equip_judge`（第五次六点注册，零 ABI；黑色牌
  【Spade|Club】+ 出向【from-zone owner 是自己且为装备/判定区】+ 进向【to_zone
  反查 owner 且为装备/判定区】，与上游 getIndex 的进出双面语义对齐）→
  `draw_cards(self, 2)` + `if hp(self) > 1 { lose_hp(self, 1) }`（锁定技无确认）。
- **行为测试**（original_sp2_liangjue 4 用例）：契约、黑色装备进装备区→摸 2+
  hp>1 失 1、红色装备/他人装备区双负例、hp=1 时摸 2 不失体力（判定区出向臂）。
  **测试管线勘误入册**：`dispatch_for_events` 返回 `Vec<EffectRequest>`（待执行
  效果请求），直派事件测试必须把返回值再交 `run_effects` 执行——olfuli 的
  EnteredDying 模式同源，此前批 162 未踩坑是因为 optional 确认流程掩盖了这一步。
- **部署与对账**：sp2 npack 触发 62→63 / 模块 69→70（ABI 不变，noop modifier
  -1）；content 与 extensions 两处同步；全部 sp2 消费者 128/128 绿。
- **ledger**：complete=5081 / approx=501 / todo=0 / **noop=2110**（liangjue 单技
  文件 noop 注记全移除；无残差故主轴 complete 不变——主轴分类含 noop 文件，
  「无近似注记即 complete」口径再次确证）。
- **拼点族（3 消费者）留待下批**：huoxin/jun_zhaoshou/tanhu 需 choose_hand_compare
  的 win/lose 双方挂钩变体，本轮未启动。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3519 通过/0 失败/
1 忽略**（3515 + 4 liangjue 用例，精确对账）；workspace 全量 **4308 通过/0 失败/
1 忽略**（4304+4 精确对账）。ledger：5582 技，complete=5081 / approx=501 / todo=0 /
noop=2110。

## 本轮：批 166——拼点族胜负挂钩引擎批·tanhu 探虎晋升（2026-09-05 第一百六十六批，ABI 678）

记忆路线①拼点族批启动。**首要发现（修正批 152/154 过时登记）**：choose_hand_compare
的胜负挂钩早已完整存在（发起者严格胜→AddBoundSkill 给发起者并绑定 target、负/平→
AddSkill 给发起者），批 154 登记的「win/lose 挂胜负双方语义缺口」实际缺的只是
**「胜→挂目标侧」**方向：

- **ABI 678 领号**（ABI-COORDINATION 表+ABI-MAX+host version.rs 同步）：新原语
  `choose_hand_compare_grant_target_on_win(player, win_grant_def, lose_skill_def)`
  ——复用 Cont 75/76/49 选择链，payload 尾 `flag=1` 标记 grant 流；发起者严格胜→
  **目标** AddSkill(win_grant_def)；负/平→无操作。全链路 12 处：rule-api 变体、
  原语、resolve ×3 flag 透传、commit_hand_compare_ext、effect_tag 701、vm handler、
  StmtIr、codegen/typecheck、vm_host skill 重写臂、hash 编码（put_u8 129）、
  skill.raw() 映射、lsp 文档。
- **tanhu 探虎（offline）主技重写**：active play_phase uses_per_turn 1 targets
  1 1 other → 新原语。发起者胜→目标获挂 tanhu2；负/平无操作（与上游 addTempSkill
  无 until 语义一致，DSL AddSkill 表达）。
- **tanhu2 诚实终态登记**：上游 tanhu2 仅 mark/intro 标记展示键、无规则面——
  DSL 占位（modifier hand_limit add 0）+ 注记「行为面在 tanhu3 的 mod（对持有者
  距离-∞+无懈免疫），tanhu3 属独立 group 技另批评审」。
- **huoxin/jun_zhaoshou 定界维持**：败方标记×第三方拼点/多卡赠予+胜方视为杀
  ——超出「胜→挂目标侧」单参表达，登记债（组合原语批）。
- **行为测试**（original_offline_tanhu 3 用例）：契约（skill_index/ABI 678）、
  30 seed 全链驱动（ActivateSkill→四段选择：选目标→发起者选牌→目标选牌）胜侧
  目标独得标记、负侧双方皆无。
- **部署与对账**：offline npack 重建（ABI 674→678、skill 827→828、trigger +1、
  modifier -1、module +1）；两处同步；**25 文件冻结 ABI 字面量 + 13 文件
  skill_index 828 + 8 文件 modifier 121 同步**；全部 offline 消费者 256/256 绿。
- **ledger**：complete=5082 / approx=500 / todo=0 / noop=2110（tanhu 主技
  noop→complete 无残差进 complete 池；tanhu2 上游即无规则面维持 noop 口径不变；
  批 165 的 liangjue complete+1 同口径并账）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3522 通过/0 失败/
1 忽略**（3519 + 3 tanhu 用例，精确对账）；workspace 全量 **4311 通过/0 失败/1
忽略**（4308+3 精确对账）。期间两次漏网断言（original_offline_dragxueji/
original_offline_zjfuli 的 compiled ABI 字面量与 fail-fast 提前中止）已定位修复，
offline 相关消费者 279/279 终态确认。ledger：5582 技，complete=5082 / approx=500 /
todo=0 / noop=2110。

## 本轮：批 167——tanhu3 group 技评审立项书（2026-09-05 第一百六十七批，评审批，无代码）

对批 166 留档的 tanhu3（offline 袁术 group 技：对 tanhu2 持有者全局距离 -∞ +
以持有者为目标的锦囊不可被无懈）做引擎面评审，**立项书入档、本轮无代码**：

- **距离面**：批 83 `CardDistanceUnlimitedTargetMark` 语义为「owner 对带标记目标
  无视距离」（eval 端 `modifier.owner == player` 限定使用方）——tanhu3 需要**方向
  相反**的变体：目标==modifier.owner 且 owner 带声明标记时，**任意使用者**无视
  距离。新变体 `CardDistanceUnlimitedMarkOnOwner(mark)`，eval 接线点
  `card_distance_unlimited_for_target`（query/mod.rs ~2095 政略②分支旁）。
- **无懈面**：wuxie 窗口每响应者的 token 门控点已定位（play.rs ~7273
  `card_use_unrespondable_for_kind` + within_distance/potshishi 并列三条件）。
  新变体 `CardUseUnrespondableTargetMarkOnOwner(mark)` 需要在门控处判定「当前牌
  的目标集合含带标记者」——wuxie 窗口 payload 不携带目标列表，需 begin_wuxie_
  window 链路透传 targets（payload 扩展或从结算上下文取）。
- **规模**：两新 modifier 变体 + 距离分支 + 无懈 payload 透传 + 六点注册
  （modifier 版）+ 测试——批 155 量级（半日），未达本轮可完整闭环（含冻结全量）
  的余量，故不启动半成品。
- **from_current 同族扫尾结论**：全仓其余 8 个 `card_target_cancel` 消费者
  （olxugou/ryoichi_tuipi/rw_minguangkai_cancel/mpbishi/reweimu/drlt_qianjie/
  zymingshi 等）逐一核对——延时锦囊/顺手牵羊/过河拆桥/火攻类**本无自指场景**，
  from_current upgrade 为纯文字精确化、零行为差异，不立项；「不分使用方」近似
  注记不再有高价值消费者。
- **下批主路线重排**：①tanhu3 引擎批（本立项书）；②组合原语批（huoxin/
  jun_zhaoshou）；③offline 86 件 active 占位池整体立项。

回归：无代码/数据变化，沿用批 166 的 engine-core 3522/0/1 与 workspace 4311/0/1。

## 本轮：批 168——tanhu3 距离面落地·CardDistanceUnlimitedSkillOnOwner 变体（2026-09-05 第一百六十八批，引擎批）

批 167 立项书的**距离面单变体落地**（无懈面维持登记债）：

- **新 modifier 变体** `CardDistanceUnlimitedSkillOnOwner(u32)`（token
  `card_distance_unlimited_skill_on_owner_<def>`，override 1）：任意使用者对**拥有
  被声明技能的目标**无视距离——批 83 TargetMark 族的**方向反转**（技能在目标侧、
  不限定使用方；政略②是 owner 攻击带标记者，本变体是 owner 被标记后被任何人攻击）。
  注册面：ModifierTarget 变体、registry token 名+parse、typecheck 豁免分支
  （override 1 门控）、eval 接线（`card_distance_unlimited_for_target` 政略②分支旁）。
  modifier 变体走内容层 token（无独立 EffectRequest），无独立 ABI 领号（批 83 先例）。
- **eval 语义与坑位**：owner-side 声明不可过 `apply_int_modifiers_for_state` 的
  `owner_filter=Some(player)`（override pass 会把 modifier.owner≠使用者的条目全部
  过滤——首轮正例假阴性根因）——`for_target` 精确匹配已隔离串扰，grant 流传
  `owner_filter=None`。全局声明语义（上游 mod 为静态全局函数，不限持有方）。
- **tanhu3 DSL 升级**：旧近似 `card_distance_unlimited_for_skill_bound_target`
  （批 98，仅发起者绑定目标）→ 新变体（任意使用者面 ⊇ 绑定面，取代之）。
  无懈禁响面（wuxieRespondable）维持登记债（批 167 立项书：wuxie 窗口 payload
  不携带目标列表）。
- **行为测试**（original_offline_tanhu3 3 用例）：契约（modifier 形状）、owner
  带 tanhu3+tanhu2 → 任意使用者距离豁免、仅 tanhu3 无 tanhu2 / 全无双负例。
- **部署与对账**：offline npack 重建（tanhu3 modifier 换 token，计数不变）；两处
  同步；offline 相关消费者 282/282 绿。
- **ledger**：complete=5081 / approx=501 / todo=0 / noop=2110（tanhu3 带近似注记
  ——无懈面债——由 complete 转 approx，-1/+1；无懈面落地后回归 complete）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3525 通过/0 失败/
1 忽略**（3522 + 3 tanhu3 用例，精确对账）；workspace 全量 **4314 通过/0 失败/
1 忽略**（4311+3 精确对账）。ledger：5582 技，complete=5081 / approx=501 / todo=0 /
noop=2110。

## 本轮：批 169——41–60 行组合形状段全量定界收口（2026-09-06 第一百六十九批，定界批，无代码）

批 167 留档三路线开工前核查：tanhu3 无懈面（begin_wuxie_window 透传）确认逐布局
工程（YUANJIAO args[0]/TIAOJIYANMEI magic 对/MANTIANGUOHAI 等各自布局，无统一
字段）维持暂缓；offline 占位池「交牌+回血」族复评——jun_renwang 需「交 2+ 张给
目标+**使用者**回 1」，`give_selected_then_recover_target` 的 recover 臂回的是
**目标**（方向不匹配）出局的结论成立，psdaohe/jdsbjiuyuan 维持。41–60 行段最后
四件逐一复核、全部定界负样本：

- **olfenyue（sp，55L）**：转换技（zhuanhuanji）+全局属性伤害历史
  （changeHp+hasNature 过滤）+changeZhuanhuanji 状态翻转+分支 chooseToUse——
  四重缺口（转换技状态机/属性伤害全局史/转换翻转/条件用牌）。
- **oljizhan（sp，46L）**：摸牌阶段改零+牌堆顶逐张猜点数循环
  （chooseControl 大于/小于）——牌堆顶序列交互原语缺口（观星族邻域）。
- **olsuji（sp，59L）**：黑色牌当【杀】view_as 使用（chooseToUse backup）+
  「若目标受伤获其一张牌」条件获牌——虚拟牌使用+条件获牌组合缺口。
- **xinfu_denglou（sp，49L）**：观看牌堆顶四张+逐张使用或获得（limited 觉醒）
  ——观星族原语缺口（批 152 xinfu_dianhua 同池定界）。
- **jun_zhaoshou/huoxin 组合原语批维持**：胜方视为杀/败方标记×第三方拼点。

**41–60 行组合形状段自批 149 首扫以来全量挖尽**（定界累计 40+ 件）；后续晋升
主路线仅剩：①引擎批（tanhu3 无懈面透传、可取消授予变体差 1 例）②组合原语批
③offline 86 件 active 占位池整体立项（本批复评再定界 1 件 jun_renwang）。

回归：无代码/数据变化，沿用批 168 的 engine-core 3525/0/1 与 workspace 4314/0/1。

## 本轮：批 170——tanhu3 无懈面落地·wuxie 窗口统一布局解析（2026-09-06 第一百七十批，引擎批）

批 167 立项书的无懈面落地，**立项书预估的「逐布局透传」被推翻**——重读
`independent_trick_effect_args`（play.rs ~5517 唯一构造处）发现独立锦囊队列的
effect_args 布局**天然统一**：`[MAGIC, card, actor, kind, suit, effect_tag, 0,
len, targets...]`，目标列表就在固定偏移。零透传、零逐布局：

- **新内容层 modifier 变体** `CardUseUnrespondableSkillOnOwner(u32)`（token
  `card_use_unrespondable_skill_on_owner_<def>`，override 1，四点注册：
  ModifierTarget/registry token+parse/typecheck 豁免/lsp 略——与批 168 距离变体
  同构；无独立 ABI，内容层 token）。
- **wuxie 窗口门控接线**（advance_wuxie_window token 门控四条件后并列）：从
  `payload[effect_off+5..]` 取 effect_args，校验 MAGIC 后读 `[7]=len、[8..8+len]`
  为目标集合；新 query `wuxie_blocked_by_target_skill_declarations(state, targets)`
  ——以**目标侧 def 反查声明**（目标拥有某 def 技能且任意角色挂有对应声明变体
  → true，与 tanhu3 距离变体同构，天然规避使用方/声明方过滤坑）。
- **tanhu3 DSL 补第二 modifier**：`card_use_unrespondable_skill_on_owner_
  2881975018 override 1`——距离面（批 168）+ 无懈面（本批）双面齐备，
  **tanhu3 回归 complete 池**（ledger complete 5081→5082 并账批 168 的 -1）。
- **行为测试**（original_offline_tanhu3 4 用例）：契约（双 modifier 形状）、
  无懈阻断 query 三态（含 tanhu2→阻断/仅 tanhu3→不阻断/目标不含持有者→不阻断）、
  距离面两用例保留。测试工程注：2 人局 bootstrap 对个别 seed 存在 "bad target"
  敏感性（identity 分配边界，独立问题已记录），用 seed 重试循环隔离。
- **部署与对账**：offline npack 重建（tanhu3 modifier +1，modifier 121→122）；
  两处同步；**8 文件 modifier 计数断言同步**；offline 相关消费者 283/283 绿。
- **ledger**：complete=5082 / approx=500 / todo=0 / noop=2110（tanhu3 无懈面
  落地移除近似注记，批 168 的 complete -1/approx +1 反向并账）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3526 通过/0 失败/
1 忽略**（3525 + 1 wuxie 用例净增，精确对账；tanhu3 文件 3→4 用例）；workspace
全量 **4315 通过/0 失败/1 忽略**（4314+1 精确对账）。ledger：5582 技，complete=
5082 / approx=500 / todo=0 / noop=2110。

## 本轮：批 171——组合原语批打包立项书·offline 占位池分桶（2026-09-06 第一百七十一批，立项批，无代码）

批 169 留档三路线的**打包立项**（验证器选项），本轮无代码：

- **huoxin 定性修正（重要）**：「魅」标记的唯一消费处是 huoxin_control group 技——
  `_trueMe`/`game.me`/`autoswap` 全套 **AI 操控权转移**（对人局无规则效果）。
  huoxin 的规则可移植面仅剩 give 分配 + 第三方拼点表演段；「魅」标记与魅惑
  触发属 AI 对战专属层（引擎无控制权转移概念），登记为「AI 层专属、规则面
  不移植」终态——组合原语批消费价值下调，jun_zhaoshou 优先。
- **jun_zhaoshou 原语设计**（胜方视为杀确认：非平局时 result2.bool ? target :
  targetx 对败方 VCard sha）：①`give_all_hand`（已有）交牌段；②`choose_other_
  can_compare`（目标为被赠牌者选拼点对象，新过滤器+选目标）；③`choose_hand_
  compare_between(a, b, win_def, lose_def)`（第三方拼点变体，Cont 复用 75/76/49
  + payload 双参与者位）；④胜方视为杀 `choose_lose_hp_use_virtual_sha_by_basic_
  variants` 族已有先例可借形。ABI 预算 679-680，两批消费。
- **offline 86 件 active 占位池分桶立项**（批 163 verb 分桶 + 本轮语义定性）：
  ①交牌+自身回复族 4 件（give-then-recover-self 变体，单原语多消费，优先）；
  ②虚拟牌使用族 5 件（view-as 使用管线，jun_zhaoshou 共享）；③观星/牌堆顶
  序列族 5 件（guanxing 族原语）；④伤害/选目标族 12 件（choose_other_damage
  可覆盖或近组合，部分零新原语可散批消化）；⑤面板/多段/家族依赖 21 件
  （维持登记债，不立项）。
- **执行序建议**：④散批（零/低成本）→ ①give-then-recover-self（1 原语 4 消费者，
  达门槛）→ ②虚拟牌管线（jun_zhaoshou+5 件）→ ③观星族。huoxin 出池（AI 层终态）。

回归：无代码/数据变化，沿用批 170 的 engine-core 3526/0/1 与 workspace 4315/0/1。

## 本轮：批 172——give_selected_then_recover_self 原语·jun_renwang 仁望主段晋升（2026-09-06 第一百七十二批，ABI 679）

批 171 立项书执行序第一项「交牌回自己族」首原语（4 消费者族之第 1 件）：

- **ABI 679 领号**：新原语 `give_selected_then_recover_self(player, target, amount)`
  ——active select（he 2..255）交给目标后**使用者**回复 amount 点体力。全链路
  12 处（批 166 模板复用：rule-api 变体、原语、apply_effect arm、skill.raw 映射、
  vm_host 重写臂、hash put_u8 130、effect_tag 702、vm handler（selected_cards
  注入同 GIVE_SELECTED_THEN_RECOVER_TARGET）、StmtIr、typecheck/codegen 新 token、
  lsp）。codegen 正则插臂截断事故一次（原 arm `=> {体}` 被切分），手工修复。
- **jun_renwang 仁望（pe_jun_liubei）主段重写**：active play_phase uses_per_turn 1
  targets 1 1 other select_he 2 255 → 新原语（amount=1）。
  **DSL 语法勘误**：selection 关键字为 `select_he`（parse.rs 仅 select_hand/
  select_he 两种，hand_equip 语义由 select_he 承载）——首版 select_hand_equip
  parse error 已改。
- **近似（登记债）**：①effect 段（杀次数+N，owner_skill_mark 动态值源需
  add_mark-to-skill 注入面验证）省略；②上游 mark 面板无引擎展示面；③usable
  语义由 uses_per_turn 1 近似。
- **行为测试**（original_offline_jun_renwang 3 用例）：契约（ABI 679/selection
  hand_equip 2..255）、交 2 张→自回 1+目标 +2、单卡选择负例拒绝（min 2）。
- **部署与对账**：offline npack 重建（ABI 678→679；skill/active/modifier 计数
  不变——jun_renwang 原为 active 占位）；两处同步；**29 文件冻结 ABI 字面量
  678→679 同步**（含 batch 166 的 tanhu >= 断言）；offline 相关消费者 286/286 绿。
- **ledger**：complete=5082 / approx=500 / todo=0 / noop=2110（jun_renwang 主段
  带近似注记维持 approx 轴，effect 段债落地后回归 complete）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3529 通过/0 失败/
1 忽略**（3526 + 3 jun_renwang 用例，精确对账）；workspace 全量 **4318 通过/0 失败/
1 忽略**（4315+3 精确对账）。ledger：5582 技，complete=5082 / approx=500 / todo=0 /
noop=2110。

## 本轮：批 173——jdsbjiuyuan 救援零新原语重写（2026-09-06 第一百七十三批）

「交牌回自己族」第 2 件（零新原语）：

- **jdsbjiuyuan 救援（jd_sb_sunquan 界司马炎·主公技）主段重写**：active play_phase
  uses_per_turn 1 targets 1 1 other **has_gainable_equip** → gain_target_equipment
  **五槽位**（weapon/armor/horse_def/horse_off/treasure，xinduodao/reduodao 同款
  既有原语）+ recover_hp(self, 1)。获得一名角色装备区所有牌并回复 1 点体力。
- **近似（登记债）**：①吴势力限定未建模——active targets 约束清单（parse.rs）
  无 group/faction 项；②主公技 zhuSkill 面（仅主公可用+身份校验）未建模。
  两者均为选目标面收窄，主行为（获装备+自回）精确。
- **行为测试**（original_offline_jdsbjiuyuan 3 用例）：契约（ABI 679/active
  has_gainable_equip/非 self）、目标双装备→五槽全获（目标装备区清空+owner 手牌
  +2+回 1）、目标装备区空负例拒绝。
- **部署与对账**：offline npack 重建（jdsbjiuyuan 占位→active，计数不变）；两处
  同步；offline 相关消费者 289/289 绿。
- **ledger**：complete=5082 / approx=500 / todo=0 / noop=2110（jdsbjiuyuan 主段
  带近似注记维持 approx 轴）。
- **psdaohe 定界修正**：其方向为「目标交牌给 player + **目标**自回」——
  choose_other_give_one_hand_equip 仅 1 张（近似任意张）且无回血段，单消费者
  新原语不达门槛，登记暂缓（批 171 立项书「交牌回自己族」四件实为三个不同方向，
  通用原语假设修正）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3532 通过/0 失败/
1 忽略**（3529 + 3 jdsbjiuyuan 用例，精确对账）；workspace 全量 **4321 通过/0 失败/
1 忽略**（4318+3 精确对账）。ledger：5582 技，complete=5082 / approx=500 / todo=0 /
noop=2110。

## 本轮：批 174——jun_zhaoshou 昭算晋升·第三方拼点原语（2026-09-06 第一百七十四批，ABI 680）

批 171 组合原语批首件（拼点族组合批）：

- **ABI 680 领号**：新原语 `choose_hand_compare_between(player, first, skill)`——
  第三方拼点：`first` 选一名对手（排除 skill owner），双方各藏一张手牌，非平局时
  **胜方对败方**使用无距离虚拟【杀】。独立 Continuation 三段
  （BetweenOpponent/BetweenFirstCard/BetweenSecondCard = 286/287/288，**不串扰**
  既有 75/76/49 链——首版蹭用 flag 的方案已废弃重构）；commit 自包含校验
  （不复用 hand_compare_commit_context——其 skill-instance 校验要求参与者持有
  技能实例，而 between 的技能在旁观者 owner 身上）；虚拟杀段 alloc→validate
  （nodistance 对败方）→handle_virtual_sha 直达结算（近似：固定以败方为目标，
  上游胜方自选）。全链路 16 处（tag 703/hash 131/ir/typecheck/codegen/lsp/vm_host）。
- **jun_zhaoshou 昭算（offline）主段重写**：active play_phase uses_per_turn 1
  hand_nonempty face_up targets 1 1 other **male** → give_all_hand(self, target) +
  choose_hand_compare_between(self, target)。give_all_hand 原语 typecheck 要求
  完整约束组（male 为原语校验的额外近似，上游无此限定——注记）。
- **行为测试**（original_offline_jun_zhaoshou 2 用例）：契约（ABI 680/skill/active
  在册）、全链四段驱动（ActivateSkill→first 选对手→first 选牌→second 选牌→
  胜方虚拟杀直达）：owner 手牌清空、拼点双牌入弃牌堆、败方扣血。bootstrap 手牌
  数不定故弃绝对手牌数断言（弃牌堆包含性断言替代）。
- **部署与对账**：offline npack 重建（ABI 679→680；**modifier 计数不变**——
  jun_zhaoshou 占位为 active draw_cards 形态无 modifier；31 文件 ABI 字面量
  679→680 同步，8 文件误改 121→122 已回退）；两处部署同步；offline 相关消费者
  291/291 绿。
- **ledger**：complete=5082 / approx=500 / todo=0 / noop=2110（jun_zhaoshou 主段
  带近似注记——虚拟杀目标固定——维持 approx 轴；占位近似转真实主段不转轴）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3534 通过/0 失败/
1 忽略**（3532 + 2 jun_zhaoshou 用例，精确对账）；workspace 全量 **4323 通过/
0 失败/1 忽略**（4321+2 精确对账）。ledger：5582 技，complete=5082 / approx=500 /
todo=0 / noop=2110。

## 本轮：批 175——观星族原语批立项书（2026-09-06 第一百七十五批，立项批，无代码）

批 174 留档执行序的观星族五件立项（解析完成，本轮无代码）：

- **消费件语义解析**：①pxlangya（sp，最简）：changeSkillsAfter 触发 →
  chooseToGuanxing(5)——纯固定数观星；②kamome_huanmeng（key）：准备阶段
  frequent → chooseToGuanxing(1+装备区牌数)——动态数观星；③xinfu_dianhua（sp）：
  准备/结束阶段 frequent → 观星 N=法噜标记花色数——**组合依赖**（xinfu_falu
  标记族未移植，需连带立项）；④oljizhan（sp）：摸牌改零+猜下一张点数循环——
  非 guanxing（独立猜点序列机制，维持定界）；⑤jxzhanyan（offline）：目标猜红牌
  数交互——同属猜数机制（维持定界）。**实际 guanxing 消费：3 件**（①②③）。
- **引擎现状核查**：观星（观看+任意重排牌堆顶/底）原语不存在——zhiji（pack_
  shenhua_1 resolve_zhiji）仅挂观星技能 def；zaiqi 是翻亮回复（无重排）。
- **原语设计**：`guanxing_top(player, count_expr)`（ABI 681 预算）：翻牌堆顶 N
  张展示 → **逐张二选**（置于牌堆顶序列/置于牌堆底）→ 重排落位。引擎选择原语
  无任意排列 UI，逐张二选为近似（上游拖拽任意排列 ⊇ 逐张二选可达排列——登记
  近似注记「排列自由度收窄」）；N 段选择循环（Cont 复用单段+轮次计数 payload）。
- **消费映射与执行序**：pxlangya（固定 5，零表达式）→ kamome_huanmeng（1+装备
  区数，表达式 count_expr=1+equip_count）→ xinfu_dianhua（连带 xinfu_falu 标记
  族立项，最后）。ABI 681 + Cont 289（GuanxingPick：payload [player, round, seq,
  top_sum, bottom_sum, flipped...]）单 Cont 多轮。
- **回归基线**：沿用批 174 的 workspace 4323/0/1、ledger 5082/500/2110。

回归：无代码/数据变化。

## 本轮：批 176——guanxing_top 原语落地·pxlangya 狼牙晋升（2026-09-06 第一百七十六批，ABI 681）

批 175 观星族立项书实现启动，观星族首原语 + 首件消费：

- **ABI 681 领号**：新原语 `guanxing_top(player, count, skill)`——观看牌堆顶 count
  张（翻入 processing，不足自动洗回）→ 逐张二选（token 1=顶序/token 2=底序，
  共 count 轮）→ 重排落位（bottom 序列先垫底、top 序列后压顶——**堆内自由序未
  建模**，逐张二选近似上游拖拽任意排列，登记债）。多轮状态在 continuation
  payload：`[player, count, round, flipped_len, flipped..., top_len, top...,
  bottom_len, bottom..., skill]`；Cont 289 GuanxingPick 单 Cont 多轮。全链路
  14 处（rule-api 变体、原语+commit、Cont、resolve、dispatch、apply arm、
  skill.raw、hash put_u8 132、tag 704、vm handler、ir、typecheck、codegen、lsp）。
- **pxlangya 狼牙（sp 彭城）重写**：触发事件勘误——上游 changeSkillsAfter 引擎
  无对应事件种类，改挂 **skill_added**（引擎既有事件）+ 既有前缀过滤器
  `owner_skill_added_self_<def>`（「自己获得 pxlangya 技能」与上游 filter 同义）
  → guanxing_top(self, 5)。锁定技 forced 无确认。
- **行为测试**（original_sp_pxlangya 2 用例）：契约（skill_added/过滤器/非
  optional/ABI 681）、五轮选择链驱动（4 底 1 顶）→ 处理区清空、牌堆 ≥5 张。
- **部署与对账**：sp npack 重建（ABI 647→681；triggers 253→254、modules
  273→274）；两处同步；**7 文件 ABI 647 字面量 + 3 文件 PACKAGE_LOGIC_HASH
  （manifest 直读）+ 3 文件 modules 274 断言**同步；全部 sp 消费者 539/539 绿。
- **ledger**：complete=5082 / approx=500 / todo=0 / noop=2110（pxlangya 主段带
  近似注记维持 approx 轴）。
- **观星族执行序**：pxlangya ✓ → kamome_huanmeng（1+装备数，key 包）→
  xinfu_dianhua（连带 xinfu_falu 标记族）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3536 通过/0 失败/
1 忽略**（3534 + 2 pxlangya 用例，精确对账）；workspace 全量 **4325 通过/0 失败/
1 忽略**（4323+2 精确对账）。ledger：5582 技，complete=5082 / approx=500 / todo=0 /
noop=2110。

## 本轮：批 177——kamome_huanmeng 幻梦晋升·EquippedCount 查询表达式（2026-09-07 第一百七十七批，内容层扩展）

观星族第 2 件（批 176 guanxing_top 原语复用，零新 Effect 原语）：

- **新查询表达式** `equipped_count(player)`（QueryRequest::EquippedCount，query_tag
  41）：返回装备区牌数——kamome 的 `1 + countCards("e")` 观星数表达式缺口。
  查询侧四点：rule-api 变体、query_tag 41、vm 构造、engine 响应 + cache key 46 +
  typecheck 表达式名（arity 1）+ codegen query tag（required_abi 681 门控）。
  勘误：Zone 查询返回 ZoneId 非牌数——kamome 计数不能借 zone()。
- **kamome_huanmeng 幻梦（key 鸥）重写**：trigger phase_prepare owner_is_target
  （frequent 口径锁定无确认）→ guanxing_top(self, 1 + equipped_count(self))。
  近似（批 176 同款）：逐张二选顶/底近似拖拽自由排列。
- **行为测试**（original_key_kamome_huanmeng 3 用例）：契约（phase_prepare/
  owner_is_target/非 optional/ABI 681）、带一装备→两轮选择链、零装备→单轮。
- **部署与对账**：key npack 重建（ABI 669→681、triggers 93→94）；两处同步；
  **11 文件 key ABI 字面量 669→681 同步 + kamome_suitcase modifier 177→176**；
  **original_stale_gap_batch 的 refresh ABI 字面量误改回退**（refresh 包未重建，
  其 manifest 仍为 669——多包断言文件按 include 的 npack 区分，勿按文件名推断）；
  key+offline+sp 三包消费者 942/942 绿。
- **ledger**：complete=5080 / approx=502 / todo=0 / noop=2108（kamome 文件 noop
  转真实+近似注记，连带批 168/172 的并账口径）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3539 通过/0 失败/
1 忽略**（3536 + 3 kamome 用例，精确对账）；workspace 全量 **4328 通过/0 失败/
1 忽略**（4325+3 精确对账）。ledger：5582 技，complete=5080 / approx=502 / todo=0 /
noop=2108。

## 本轮：批 178——jun_zhaoshou 虚拟杀目标自由化·尾债消除（2026-09-07 第一百七十八批）

批 174 登记的 jun_zhaoshou 近似债（虚拟杀固定以败方为目标）消除：

- **commit_hand_compare_between 尾段重构**：删除「直达 handle_virtual_sha
  targets=[败方]」的近似路径（连同 choose_use_virtual_sha_super 专用 runner 65 行），
  改调完整 stage 机器 `choose_use_virtual_sha(winner, ignore_distance=true,
  ignore_use_count=false, draw_on_no_use=false, skill, 0, 0)`——胜方经
  VirtualShaTargets 选择链**自选目标**，与上游 chooseUseTarget(nodistance) 完全
  对齐（无可选目标时原语自行回收虚拟牌）。jun_zhaoshou 近似注记消除（登记债清零，
  行为面与上游一致）。
- **行为测试更新**（original_offline_jun_zhaoshou 全链用例）：四段选择链
  （选对手→first 选牌→second 选牌→**first 为虚拟杀自选目标**）。
- **部署**：DSL 仅注记变更（不进 IR，npack 字节不变免重建）；测试与引擎同步。
- **ledger**：complete=5081 / approx=501 / todo=0 / noop=2108（jun_zhaoshou 近似
  注记消除 → approx -1 / complete +1，回归 complete 池；连带批 172/173 的
  交牌族并账口径）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3539 通过/0 失败/
1 忽略**（jun_zhaoshou 全链用例原位改写四段链，用例数不变）；workspace 全量
**4328 通过/0 失败/1 忽略**（与批 177 基线持平）。ledger：5582 技，complete=5081 /
approx=501 / todo=0 / noop=2108。

## 本轮：批 179——xinfu_dianhua 点化晋升·观星族收尾（2026-09-08 第一百七十九批，零新原语）

观星族第 3 件（零新原语，批 176 guanxing_top 复用）：

- **xinfu_dianhua 点化（sp 袁术妻法箓族）重写**：trigger phase_prepare/phase_end
  owner_is_target（双臂，frequent 口径锁定无确认）→ guanxing_top(self,
  mark_count(self, 3540560358))——X=法箓标记数。在 xinfu_falu 既有「统一法箓
  标记」近似（每花色至多 1 枚、初始 4 枚）下 **mark_count 枚数精确映射上游花色
  种数**（初始全有=4；zhenyi 类消耗后枚数=剩余种数，两口径一致）。
- **xinfu_falu 维持既有近似**：初始 add_mark 4 枚已表达（批 148 前近似）；lose/项
  花色化收集分支在统一标记近似下不可表达（花色化记录原语缺口）——维持既有
  近似注记，dianhua 消费侧不感知。
- **行为测试**（original_sp_xinfu_dianhua 4 用例）：契约（双臂 phase_prepare/
  phase_end/owner_is_target/非 optional/ABI 681）、3 标记→3 轮选择链、零标记
  静默（count=0 原语自兜）、结束阶段臂触发。
- **部署与对账**：sp npack 重建（triggers 254→256、modules 274→276）；两处同步；
  **3 文件 PACKAGE_LOGIC_HASH manifest 直读重录 + triggers/modules 计数断言
  同步**；全部 sp 消费者 543/543 绿。
- **ledger**：complete=5080 / approx=502 / todo=0 / noop=2107（dianhua noop→
  真实+近似注记：noop -1 / approx +1）。
- **观星族收尾**：pxlangya ✓ → kamome ✓ → xinfu_dianhua ✓（falu 族连带收窄为
  既有近似维持——花色化标记存储原语缺口单独留档）。观星族三件全部晋升完成。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3540 通过/0 失败/
1 忽略**（3539 + 4 dianhua 用例，精确对账）；workspace 全量 **4332 通过/0 失败/
1 忽略**（4328+4 精确对账）。ledger：5582 技，complete=5080 / approx=502 / todo=0 /
noop=2107。

## 本轮：批 180——choose_other_give_then_recover 原语·psdaohe 踏鹤晋升（2026-09-08 第一百八十批，ABI 682）

「交牌回自己族」补充原语（目标交牌+目标自回方向，psdaohe 单消费者——批 173
定界暂缓后按验证器指示落地）：

- **ABI 682 领号**：新原语 `choose_other_give_then_recover(player, skill)`——选
  一名其他有手牌角色，其交给你一张手牌，然后**其**回复 1 点体力。Cont 290
  （单段选目标，Giver 选牌复用 DrawThenGiveOne ready_to_give 链——**第一参为
  交牌者**，批 173 曾因方向理解错误返工一次，勘误入册）；tag 705/hash 133/
  ir/typecheck/codegen/lsp 全链路 12 处。
- **psdaohe 踏鹤（offline）重写**：active play_phase uses_per_turn 1 targets 1 1
  other hand_nonempty → choose_other_give_then_recover(self)。近似（登记债）：
  上游任意张 [1,∞] allowChooseAll 由其自选——原语为固定 1 张（N 张变体单消费
  者不达门槛）。
- **行为测试**（original_offline_psdaohe 2 用例）：契约（ABI 682/active
  hand_nonempty/非 self）、三段链（owner 选 giver→giver 选牌）→ giver −1 牌
  +owner +1 牌+giver 回 1（giver 预先 −1 血使可观察）。
- **磁盘事故**：deps 280G+incremental 29G 撑爆数据盘（余 533Mi）——删除
  target/debug/incremental 释放 29G 后恢复（教训：incremental 缓存可安全清除）。
- **部署与对账**：offline npack 重建（ABI 680→682 跨两号；psdaohe 占位→active，
  计数不变）；两处同步；**29 文件 offline ABI 字面量 680→682 同步**；offline
  消费者 270/270 绿。
- **ledger**：complete=5080 / approx=502 / todo=0 / noop=2107（psdaohe 占位
  （draw_cards 近似）→真实主段带近似注记，轴不变）。

回归（冻结终态，全部改动落盘后单次运行，exit 0；前两轮因 dragxueji/zjfuli/
zylianji 的 compiled ABI 字面量漏改与 fail-fast 中止，逐一定位修复后重跑）：
engine-core **3541 通过/0 失败/1 忽略**（3540 + 2 psdaohe 用例，精确对账）；
workspace 全量 **4334 通过/0 失败/1 忽略**（4332+2 精确对账）。ledger：5582 技，
complete=5080 / approx=502 / todo=0 / noop=2107。

## 本轮：批 181——虚拟牌使用管线批立项书（2026-09-08 第一百八十一批，立项批，无代码）

虚拟牌族 5 件逐件解析与引擎缺口清单（组合原语批正式设计启动，本轮无代码）：

- **hsjunwei（offline，69L）**：两张同色牌当无懈 + effect 尾段（无懈被中和时改追
  加杀/锦囊目标）——多卡 view_as 约束 + 无懈响应链 + 目标追加三重缺口；维持债。
- **tyshencai（offline，74L）**：非虚拟牌（定界修正）——目标强制判定
  （event.target.judge + callback=shencai.contentx）+ 标记计数门槛，属判定
  callback 家族（shencai 家族连带）；维持债。
- **zj_qiangyi（offline，50L）**：影牌当玉石皆碎 + 同势力限定 mod——
  choose_use_virtual_card 仅支持 Wanjian/Juedou/Nanman 三 kind（pack_mobile
  硬编码），yushijiesui 需扩 kind 白名单（引擎小改）+ storage 联动（zjfayi）；
  单消费者暂缓。
- **hssifen（offline，117L）**：目标强制 chooseToUse 任意手牌当【决斗】——
  「令目标 chooseToUse」管线原语缺（引擎的 choose_use_virtual_sha 是自用版）；
  维持债。
- **dragshiyao（offline，75L）**：createCard 随机生成【决斗】给目标（随机花色/
  点数原语缺）+ 家族 global 链；维持债。
- **通用管线原语设计（后续引擎批入口）**：①`force_target_choose_to_use(target,
  kind, constraint)`——「令目标 chooseToUse」族（hssifen/上游 tyshencai 类）；
  ②`choose_use_virtual_card` kind 白名单扩展为参数化（zj_qiangyi 类单卡
  view_as）；③`create_random_card(suit_pool, rank_range)`（dragshiyao 类）。
  三者均单/双消费者，按 ADR 0597 门槛维持立项池，不达 3/3 不启动。
- **执行序重排**：观星族三件已收尾；虚拟牌族 5 件全部维持债（本轮定性关闭
  「管线可复用」假设——choose_use_virtual_sha 是杀专用 stage 机器，通用化需
  新原语族而非复用）。下一步回到 784 件主池：近似池残差缩减（轻措辞扫描）
  或组合原语批第二件（give-then-recover-self 剩余：hm_qianjun 觉醒族）。

回归：无代码/数据变化，沿用批 180 的 engine-core 3541/0/1 与 workspace 4334/0/1。

## 本轮：批 182——hm_qianjun 千军零新原语近似重写（2026-09-08 第一百八十二批）

「交牌回自己族」第 4 件（零新原语）：

- **hm_qianjun 千军（hm_guigu 鬼谷）主段重写**：active play_phase equip_nonempty
  once_per_game targets 1 1 other → give_all_equipment_choose_recover_or_damage
  (self, event_source)（xianzhou 同款：给光装备→收牌者选 owner 回血(cancel)或
  受伤——选回血即上游无条件回血，choose 分支为语义超集）+ add_skill(self,
  olluanji_hmq)（「乱击」占位技能获得）。
- **近似（登记债）**：①game.swapSeat 座次交换未建模（引擎级，批 171 定性）；
  ②olluanji 规则面（interval 判定族）占位待连带立项；③choose 分支语义超集；
  ④上游 limited 觉醒动画未建模。
- **行为测试**（original_offline_hm_qianjun 2 用例）：契约（ABI 682/once_per_game/
  skill_index 双 def 在册）、双装备→全移交+自回 1+获得乱击占位。
- **部署与对账**：offline npack 重建（skill 828→829 olluanji_hmq、modifier
  122→123 千军自身、actives 137 不变）；两处同步；**13 文件 skill 828→829 +
  8 文件 modifier 122→123 断言同步**；offline 消费者 272/272 绿。
- **DSL 惯例新增**：多 effect 顺序敏感性——选择型 effect（触发 ChoiceRequested）
  之后的 effect 不会执行；需顺序保证的 add_skill 置于选择型 effect **之前**。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：engine-core **3542 通过/0 失败/
1 忽略**（3541 + 2 hm_qianjun 用例，精确对账）；workspace 全量 **4336 通过/0 失败/
1 忽略**（4334+2 精确对账）。ledger：5582 技，complete=5080 / approx=502 / todo=0 /
noop=2107（hm_qianjun 占位→真实主段带近似注记，轴不变）。

## 本轮：批 183——xinfu_falu 花色化标记族落地·guanxing_top_by_falu_marks（2026-09-08 第一百八十三批，ABI 683）

批 175 观星族立项书连带项实现（falu 花色化标记族 S0）：

- **ABI 683 领号**：新原语 `guanxing_top_by_falu_marks(player, [m1..m4], skill)`
  （tag 706/hash 134/ir/typecheck 5 参/codegen/lsp 全链路）——数四法箓标记中非零
  种数 N，观星牌堆顶 N 张（guanxing_top 多轮二选复用，Cont 289）。
- **4 个参数化触发过滤器**（TriggerFilter::OwnerDiscardSuitNoOwnFaluMark(u8, u32)，
  registry 固定四 token `owner_discard_suit_no_own_mark_{spade,heart,club,diamond}_
  falu`）：cards_moved + ctx.cards 含花色 X + to=discard + owner 无 X 法箓标记。
- **xinfu_falu 法箓重写**：game_started owner_alive → 四花色 add_mark 各 1（初始
  全有）；cards_moved 四过滤器 → 各自 add_mark 1。近似（登记债）：enterGame 补齐
  分支并入 game_started；loseAsyncAfter 多区域变动归并为 cards_moved 同事件族。
- **xinfu_dianhua 点化换真值源**：count 从统一法箓标记枚数改为
  guanxing_top_by_falu_marks（非零种数）——花色化后语义精确。falu 四标记 def：
  spade=3540560359 heart=3540560360 club=3540560361 diamond=3540560362。
- **行为测试**（original_sp_xinfu_falu_family 2 用例）：契约（game_started+四
  discard 过滤器+dianhua phase_prepare）、game_started 初始全有 + 弃黑桃/红桃 →
  各自标记保持 1（collect-once 语义）。
- **部署与对账**：sp npack 重建（triggers 260、modules 280）；两处同步；sp 消费者
  545/545 绿（3 文件 hash 直读重录 + 计数断言同步）。
- **ledger**：complete=5080 / approx=502 / todo=0 / noop=2107 不变（falu/dianhua
  均带近似注记维持 approx 轴——falu 花色化补齐不改轴）。
- **观星族收尾完成**：pxlangya（固定 5）/kamome（1+装备数）/xinfu_dianhua（法箓
  种数）三件全部晋升。

回归（冻结终态，全部改动落盘后单次运行）：见末行。

## 补记：批 183 尾账清偿——sp ABI 683 连锁补同步·契约测试随动（2026-09-09 本会话）

批 183 的最终 npack 重建（sp ABI 681→683，modules 276→280、triggers 256→260、
modifiers 516→481 花色化收窄）落盘晚于其测试冻结同步，最终回归未记录（日记末行
空悬）。本轮以 `--no-fail-fast` 全量复跑暴露并逐项清偿：

- **7 文件 sp 包 ABI `== 681`→`== 683`**：batch AJ/AK/AM/AN 四契约 +
  dingpan/hengzheng/ziyuan 三冻结（扫描器正则分「包级断言」与「独立源 compiled
  断言」两条过滤，sp 全量 `== 681` 中无独立源混入）。
- **三冻结测试 hash 直读重录**：PACKAGE_LOGIC_HASH 常量从 sp.npack manifest 直读
  重写（单行 const 形态，纯文本 replace 无 sed）；modules 276→280、trigger 256→260、
  modifier →481 计数断言随动（hengzheng 的常量名是 SP_PACKAGE_LOGIC_HASH，正则
  `\w*PACKAGE_LOGIC_HASH\w*` 兜底）。
- **两契约测试随批 183 花色化升级**：original_sp_xinfu_dianhua（统一定义标记
  mark_count 口径 → 四花色各 1 枚、种数即张数）与 original_sp_xinfu_falu（开局
  四标记 → 四花色 [1,1,1,1]）；两者批 183 被新 family 测试部分取代但保留作
  committed npack 的深断言回归网。
- **regongao 分离线程吞 panic 修复**：original_refresh_regongao_rejuyi 的断言全部
  跑在 detach 线程里，线程 panic 不影响主测试（历史 `== 356` 陈旧断言恒绿未被发现的
  根因）——改 join handle 并 `expect` 传播失败；ABI 字面量 356→683。
- 补记后 engine-core 全量 **3560 通过/0 失败/1 忽略**（1004 二进制，含本会话新增
  11 用例）。

## 本轮：批 184——refresh/xianding 观星原语外溢·四技晋升（2026-09-09，零新原语）

批 159 复核证伪池形状扫描（xianding/onlyOL/refresh 三池 485 件单技能件复扫）后
四件晋升；观星族原语（批 176 guanxing_top，ABI 681）外溢至 refresh/xianding 两包，
**两包 ABI 669→681**（复用既有领号，无新号）：

- **reguanxing 观星（refresh 界诸葛亮）**：phase_prepare + owner_is_target（frequent
  →forced 无确认）→ 存活数 <4 时 `guanxing_top(self,3)`，否则 5（`alive_count()`
  表达式分支）。近似（登记债）：①逐张二选近似拖拽（pxlangya 同债）；②上游取消后的
  结束阶段补偿臂（reguanxing_on 门控）无「上次取消」查询面，省略未移植。
- **olzhiji 志继（refresh OL姜维）**：双臂 phase_prepare/phase_end + owner_hand_empty
  （forced 觉醒）→ `if mark_count(self,自身def)==0` 一次门控 → change_max_hp(-1) +
  add_skill(观星def) + add_mark + `choose_draw_or_recover(self,2,2,0)` 收尾（批 182
  选择型置尾惯例）。近似：顺序对调（先减上限后摸/回，终态一致）；满血时原语契约
  自动改摸牌不询问（恢复无意义的免询问路径）；觉醒动画未建模。授予的观星本体
  同批已晋升，形成完整闭包。
- **shebian 设变（refresh OL夏侯渊）**：新过滤器 `owner_turned_over`
  （CharacterStateChanged + 事件角色=拥有者 + 双向翻面；5 处注册零 ABI，对齐上游
  `player: turnOverEnd`）→ optional 确认 → `choose_move_field_card_to(self, self)`
  （批 AF 移场牌原语）。近似：原语可选集含判定区、方向固定「场上→自己」（上游
  moveCard 装备牌任意移动）。
- **tuiyan 推演（xianding 管辂）**：phase_play + owner_is_target（frequent→forced）
  → `guanxing_top(self, 3)`。近似：上游纯观看+逆序放回，原语允许重排为能力超集
  （信息面等价）。
- **行为测试**（4 文件 11 用例）：reguanxing（契约+3 轮观星链闭环+processing 排空）、
  olzhiji（契约双臂+满血自动摸 2+上限-1+获观星、回复分支 2→3+标记门控不重复觉醒）、
  shebian（契约 optional+确认→选防马→装备区落位、他人翻面负例）、tuiyan（契约+
  3 轮链、他人阶段负例）。
- **部署与对账**：refresh npack modules 157→161/triggers 142→146/modifiers 212→209，
  xianding modules 174→175/triggers 155→156/modifiers 397→396；各三处副本字节一致 +
  .npack-store 双槽刷新（整集校验仍卡既有 clan@0.1.0 陈旧槽，属并行会话遗留，未动
  content lock，冷导入以 content-runtime fresh tempdir 验证）；refresh 侧 5 文件
  `== 669`→681 + xianding 侧 guili 669→681；两 README 追加批 184 段并同步 extensions
  副本（xianding extensions 侧原落后 malingli/caoyi 两段，一并拉齐）。
- **ledger**：complete=5075 / approx=507 / todo=0 / noop=2103（4 件 noop→真实主段带
  近似注记：noop -4、complete -4、approx +4）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4349 通过/0 失败/
1 忽略**（1070 二进制；批 182 基线 4336 + 批 183 falu_family 2 + 本批 11 用例，精确
对账）；其中 engine-core **3560 通过/0 失败/1 忽略**（1004 二进制）。冷构建全程磁盘
受控（增量缓存 12G 量级，未触发批 180 型 deps 撑盘）。

## 本轮：批 185——refresh/xianding 证伪池复扫·双件晋升（2026-09-09，零新原语）

按验证器指示对剩余证伪池（xianding ~276 / onlyOL ~121 / refresh ~78）复扫中档件，
逐件读上游体定界后两件晋升（两包 ABI 恒 681）：

- **olduanliang 断粮（refresh OL徐晃）**：`view_as bingliang play_phase
  black_basic_equip from hand_equip`——「黑色非锦囊」与 shenhua duanliang 既有源
  过滤器 `black_basic_equip`（基本+装备）精确匹配，零近似。近似（登记债）：「本回合
  未造成伤害时无距离限制」的条件距离 mod 因回合伤害历史值源缺口未建模。npack：
  view_as 12→13、modifiers 208（noop -1）、modules/triggers 不变（view_as 不进
  trigger）。
- **zhushi 助势（xianding 曹髦，主公技）**：新过滤器
  `owner_alive_other_wei_recovery_own_phase_once_per_turn`（HpRecovered + 事件角色
  非拥有者 + `player_group=="wei"` + `turn.current==事件角色`；**六点注册**——常规
  5 处 + `turn_limit` match 名单（漏登记则 once 恒不生效，批 150 教训兑现），零
  ABI）→ optional → `draw_cards(self, 1)`。近似（登记债）：①「其可令你摸牌」确认方
  为回复的魏角色 → 拥有者 optional；②zhuSkill 主公身份门未建模。npack：modules
  176（+1）、triggers 157（+1）、modifiers 395（noop -1）。
- **中档件定界负样本**（本轮读体证伪，登记不立项）：olsbdouchan（牌堆按名搜牌
  可用 gain_card_by_kind 但「未搜到才加标记」分支不可表达+封顶需死亡人数表达式，
  双重近似劣化放弃）、dcsbyitou（markAuto 玩家名单标记不可表达）、mingjie（逐张
  摸牌颜色分支循环）、olcaishi（回合内类型首次查询）、ol_yizhong（黑杀 per-color
  选择器+directHit 面缺口）、olsbqiwu/oljiaojin（花色/类型过滤弃牌选择缺口）、
  dczhuiyi（死亡链「回复且摸」同目标链式缺口）、chengzhang/tianjie（全局历史
  查询族）、olsbdulie（effectCount++ 路线，批 169 暂缓维持）。onlyOL 41–52L 段与
  refresh 中档件简单面自此系统性挖尽。
- **行为测试**（2 文件 6 用例）：zhushi（契约+魏势力自己回合回复→确认→摸 1+
  同回合二次不触发、非魏负例、他人回合负例）、olduanliang（view_as 契约四字段+
  roster+纯 view_as 无 module 断言）。
- **部署与对账**：两 npack 三处副本字节一致；README 追加批 185 段并同步 extensions。
- **ledger**：complete=5073 / approx=509 / todo=0 / noop=2101（2 件 noop→真实带
  近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4355 通过/0 失败/
1 忽略**（1074 二进制；批 184 基线 4349 + 本批 6 用例，精确对账）。缓存治理：批内
清理 target/debug/incremental（9G→释放）。

## 本轮：批 186——offline 占位池④收口·魔魇伤害转化晋升（2026-09-09，零新原语零新过滤器）

批 171 立项的 offline 86 件 active/trigger 占位池④「伤害/选目标族」散批收口。对
offline 池 67 件有上游块的 noop 全量形状扫描 + ≤35L 段逐件读体定界后，单件晋升
（offline ABI 682 不变）：

- **psmoyan 魔魇（魔姜维）**：锁定技——①你造成和受到的伤害改为体力流失：
  `damage_before owner_is_source / owner_is_target` 双臂（pot_sunchen/dccibei 同款
  接线，owner_is_source/owner_is_target 为事件面无关的通用过滤器）→
  `prevent_current_damage()` + `lose_hp(对方|自己, event_amount)`（pxningzhou 转流失
  模板）；②有角色失去体力后你摸一张牌：`trigger hp_lost owner_alive`（OwnerAlive
  事件面无关 → 任意角色的体力流失皆触发，与上游 global loseHpAfter 一致；且②的
  摸牌由①的转化流失反哺，与上游联动一致）。近似（登记债）：owner 同时为来源与
  目标的自伤事件双臂同中（转化一次 vs 两次，极端边界）。
- **行为测试**（original_offline_psmoyan 5 用例）：契约三臂+roster、造伤转化
  （目标 hp 等量流失+owner 摸 1）、受伤转化（owner hp 流失+摸 1+来源无损）、
  任意角色流失摸牌、无挂起选择回归。
- **测试工艺新证**：damage_before 触发仅在 `DealDamage`（prepare/intent 路径）
  内规划——测试直驱 `CommitDamage` 会**静默绕过**整面 damage_before 触发
  （hp 仍会扣，断言极易漏判）；转化链的嵌套 lose_hp 事件由 run_effects 内部
  回灌，无需手动二次 dispatch。
- **桶④定界负样本**（读体证伪入册）：lxkuili（装备栏废除/启用选择面）、
  lxshishou（同势力目标+「本回合造成或受到过伤害」需 owner 归因的回合伤害旗标，
  引擎只有 current-source 口径的 DAMAGE_DEALT）、lxdingfa（弃牌堆伤害牌快照查询）、
  lxchenjie（markAuto 玩家名单+targetEnabled mod）、lxkuangli（多段弃牌+类型分析）、
  lxqiantun（拼点计数比较）、pegaishi（座次比较+directHit）、lxweitong（空体
  persevereSkill 壳维持终态）。**桶④至此全部晋升或定界完毕。**
- **部署与对账**：offline npack modules 717→720、triggers 580→583、modifiers
  123→122、skill 829 不变；三处副本字节一致；11 个消费者测试文件的计数断言同步
  （nsjianglie 首轮漏 trigger 一处，回归拦截后补齐）；README 追加段并同步 extensions。
- **ledger**：complete=5072 / approx=510 / todo=0 / noop=2100（1 件 noop→真实带
  近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4360 通过/0 失败/
1 忽略**（1074 二进制；批 185 基线 4355 + 本批 5 用例，精确对账）。缓存治理：批内
清理 target/debug/incremental。

## 本轮：批 187——近似池残差审计·交叉核对零失实（2026-09-09 第一百八十七批，审计批，无代码）

按验证器指示对 approx 池 509 件做系统化残差审计（批 154 模式的全量化扩展），并复核
组合原语立项门槛。结论先行：**近似注记零失实，无件可晋升回 complete 轴**。

- **系统性交叉核对**：提取 509 件近似注记原文，对其中 141 件含「未建模」措辞者，
  按 11 个关键词族（花色/距离/判定/标记/点数/次数/展示/横置/护甲/拼点/体力上限）
  构建「注记声称 ↔ 技能体实况」交叉核对（注记称 X 未建模而技能体已含 X 族构造
  即为疑似失实）——命中 1 件且经读体证伪（yuiko_dilve 的 hand_compare 构造与
  onCompare bottomCards 债不同侧面）。**注记层与实现层一致性好于批 154 时的抽样
  预期**；批 148/156/159 的注记复核管线（证伪改写）已把「假债注记」清理干净。
- **抽样审计 8 件**（横跨 sixiang/sp/shiji/key/yijiang/clan 六包，逐件读体对照
  上游）：stdchenghao（三缺口均属实——观星原语不覆盖「观看后交给任意角色」）、
  olfengyan（受伤臂可改进但二选一/交牌方向缺失仍属真债）、reyanzhu（二选一+
  装备移交+标记伤害三缺口属实）、tspowei（职责技状态机/标记散布属实）、
  kotori_skill_wei（对方选择权不可表达属实）、nk_shekong（对等量弃牌分支属实）、
  kud_buhui（不弃装备摸等量属实——equipped_count+discard_all_target_equipment
  虽已具备，但补齐后仍因「冲阵本体 noop」滞留 approx 轴，残差缩减不换轴）、
  yuiko_dilve（拼点底牌替换确缺）。**8/8 维持。**
- **新缺口发现（登记）**：**「交给事件来源」方向原语缺失**——全部 give 族原语
  方向为「player→event_player / player→选定其他 / player→选定目标」，无
  「owner→ctx.source」通式；olfengyan 选项①（摸 1 交给伤害来源）被挡。消费者
  1 例（olfengyan），远低于 3/3 门槛，登记不立项。
- **立项门槛复核（批 171 组合原语批剩余路线）**：①give-then-recover-self 族 4 件
  已全消费（批 172/173/180/182）；③观星族 3 件已全消费（批 176/179/183）；
  ④伤害/选目标族 12 件已收口（批 186+此前散批）；②虚拟牌管线 5 件批 181 定性
  维持债（单/双消费者）维持；⑤面板/家族依赖 21 件维持登记债。门槛表更新：
  「可取消授予」2 例（维持）；「回复且摸」链式 2 例（fengming+dczhuiyi，批 186
  新增 1 例，仍差 1）；「交给事件来源」1 例（新登记）；per-color sha 选择器+
  directHit 2 例（poyue+ol_yizhong，批 186 定界并入）。**无任何族达 3/3 门槛，
  下一件实质晋升需 expansion 系统级 ADR 或上游增量带来的新件。**
- **回归与缓存**：无代码/数据变化（纯审计批），沿用批 186 的 workspace 4360/0/1
  冻结基线；incremental 缓存于批 186 收尾已清，本批零构建零新增。


## 本轮：批 188——上游增量闭包（ad3bc6ca..d6081fd1）·4 新将 12 技·tanhu2 双定界清偿（2026-09-09）

上游增量同步路线（upstream-sync-workflow 全流程）：git pull 21 提交 → sync 重建
catalog → 复核 un-ported 残差 → 新将迁移闭包。

- **tanhu2 双定界清偿（sync 阻塞修复）**：sync 报 `offline:tanhu2 conflicting def
  ids 2642618323/2881975018`——批 166 在 tanhu.skill 内手造 tanhu2（2881975018，
  def 手抄坑的内容层变体），而 scls_yinshi.skill 另有 catalog 正定 tanhu2。统一到
  正定 def：tanhu.skill 删重复块并改线拼点授技参数、tanhu3.skill 两个 modifier
  token 改线、original_offline_tanhu3/tanhu 两测试常量随动；offline npack skill
  829→828、modifiers 122→121。**「def id 必查 catalog」纪律的内容层欠账当场被
  sync 校验器拦截，属设计如此。**
- **新将迁移（4 将 12 技）**：势贺齐（bingshi：闪袭/绮胄——gaintag 应机牌机制族，
  诚实 noop×2）；雁翎蔡文姬（断肠/悲歌——clearSkills/互换存储，noop×2）；雁翎孙策
  （**激昂实体化**/魂姿实体化/制霸——1 实体 1 实体 1 noop）；春秋罗贯中（著魂/经世/
  染瀚/湖海笔——化身与 expansion 旗舰族，noop×4）。
  - **ylyg_jiang 激昂**：任意角色使用【决斗】或红杀 → 使用者是你可以摸 2，否则
    你与其各摸 1。新过滤器 `owner_alive_(other_)global_juedou_or_red_sha_used`
    双臂（sbjiang 过滤器去 owner 介入版，5 点注册×2 零 ABI；DSL 无 `!=`，以双
    过滤器拆臂）。债：应答窗口（respond_to 目标注入+give-to-source 方向缺）与
    点数 13 当杀 mod 未建模。
  - **ylyg_hunzi 魂姿**：hp_lost + hp==1 门控 + once 标记 → change_max_hp(+1) +
    跨包 add_skill 字面量（refresh 英姿 2412932912、shenhua 英魂 3072887319，
    均为 offline 声明依赖；twfucheng 先例）。债：changeHp 触发面收窄为体力流失。
- **新将入库管线补全**：characters.json 追加 4 条（portrait_asset 引用资产 id），
  画像自上游 image/character 复制——**磁盘文件用裸名（pot_heqi.jpg），资产 id 由
  打包器自动加 character_ 前缀**；冻结 asset_paths 会挡新资产，须以 `--assets`
  重扫替代 manifest 保留；顺带清出 10 个历史批次误拷的 character_* 双前缀垃圾
  文件。bingshi 重建曾漏 `--cards`（card_index 1→0）被回归拦截后补齐。
- **roster 平移回归修复**：offline roster +3 引发 4 处种子敏感——
  original_card_use_history/damage_choice_migration 加开局挂起选择排空（qimen
  模式，drain 循环进 setup）、zj_guluo 4 人局种子 +salt 避开 identity 坏窗、
  tanhu 主测试 def 随动。
- **部署与对账**：bingshi chars 35→36/skills 101→103/modifiers 24→26；offline
  chars 379→382/skills 828→838/modifiers 121→129/modules 720→723/triggers
  583→586；两 npack 三处副本字节一致；19 个消费者测试文件计数断言同步；**catalog
  2560/2560 武将、6152/6152 技全 ported（un-ported 残差清零）**。
- **ledger**：complete=5082 / approx=512 / todo=0 / noop=2110（5594 技；12 件新技
  归轴：ylyg_jiang/ylyg_hunzi 带近似注记进 approx +2，10 件诚实 noop 按既有惯例
  class 归 complete、noop 轴 +10——noop 为独立正交轴）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4365 通过/0 失败/
1 忽略**（1074 二进制；批 186 基线 4360 + 本批 ylyg_jiang 2 + ylyg_hunzi 3 用例，
精确对账）。缓存治理：批内清理 target/debug/incremental。

## 本轮：批 189——ADR 0641 §8 章程推进·expansion 计数门控 S1-A 三件复活（2026-09-09，零新原语）

上游增量窗口关闭后启动 expansion 武将牌系统推进：ADR 0641 已存在（2026-08-15 立项，
P1 取用/P2 置入已交付），本轮做全量消费者普查并写入 §8，随即执行 S1-A 切片。

- **全量普查**：addToExpansion 系 249 技、getExpansions 系 80 技、计数门控触发面
  95 技；按阻塞面六分类（A0 自手牌置入 157 / B1 计数查询 76 / A3 取用混合 64 /
  A1 置入他人 19 / A2 牌堆弃牌置入 9 / B0 个例 4）——明细与切片设计见 ADR 0641 §8。
- **S1-A 执行（恰达 3/3 门槛）**：moucuan 谋篡（mobile，「兵」≥势力数觉醒）、
  xinzili 自立（refresh，「权」>2 觉醒）、tianxing 天行（onlyOL，「蛊」≥3 觉醒 +
  四选一授技 choose(1,2,3,4)）。均为 skill_pile_count 计数门控 + once 标记 +
  通用效果，dormant 契约（pile 供给面落地前不激活，近似注记）。三包 ABI 不变
  （652/681/653），各 +1 触发模块 −1 noop modifier。
- **DSL/引擎事实补记（入 ADR §8.3）**：BinOp 无 `>=`/`!=`（`B < A+1` 改写）；
  remove_skill 收实例、remove_skill_by_def 收 def；forced 触发模块 dispatch 期内联
  执行（体内 choose 使 dispatch 返回 ChoiceRequested）；P2 置入续体为 ollihuo 形态
  限手杀（泛化并入 S2 工单）；测试可直构 SkillPileRef+Expansion zone 驱动计数门控。
- **行为测试**（original_s1a_expansion_awaken 3 用例）：三技门控正例 + moucuan
  once 门控 + tianxing 四选一授予分支。
- **计数同步**：mobile 216/171/247、refresh 162/147/207、onlyOL 27/25/171
  （modules/triggers/modifiers）三包 6 文件断言随动。
- **ledger**：complete=5079 / approx=515 / todo=0 / noop=2107（3 件 noop→真实带
  近似注记：complete -3、approx +3、noop -3）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4368 通过/0 失败/
1 忽略**（1074 二进制；批 188 基线 4365 + 本批 3 用例，精确对账）。缓存治理：批内
清理 target/debug/incremental。

**下一步（ADR 0641 §8.2 排程）**：S1-B `discard_skill_pile` 铸造批（63 件消费，
远超门槛）→ S2 P2 变体（A1 19 件 / A2 9 件，2 领号）→ S3 事件联动过滤器。

## 本轮：批 190——S1-B 首铸 `discard_skill_pile`·rebiaozhao3 集成定界（2026-09-09，ABI 684 领号）

ADR 0641 §8.2 排程的 S1-B 铸造批执行：

- **原语落地**：`discard_skill_pile(player, self_skill, pile_def)`——自身关联技能堆
  全部牌移入弃牌堆（tag 707 / ABI 684 领号先行 + host bump；纯同步零选择续体；
  hash 复用 tag 154、盐 DSKP、3×u32 形状）。pile_def 为 u32 字面量——**堆可属
  父技/兄弟技**（rebiaozhao3 型的必要参数化；批 166 tanhu2 教训的正向应用）。
  直驱行为测试通过。
- **顺带铸造**：参数化过滤器 `OwnerSkillPileNonempty(u32)`
  （`owner_skill_pile_nonempty_<def>`，5 点注册零 ABI）——S1-B 全族
  「getExpansions(...).length > 0」门控通用件，在册待消费。
- **rebiaozhao3 集成定界 → 回退诚实 noop**：触发体内「纯效果（弃堆）+ 选择链」
  交错时弃堆请求经 vm-pause defer 路径后丢失（探针证实原语可达但执行时堆读空）。
  涉及 dispatch paused/outcome.effects 语义与 resolve_choice 的 deferred 重放边界
  ——rebiaozhao3 回退 noop 待集成批；过滤器保留在册。
- **流程教训**：ABI-COORDINATION.md 的 ABI-MAX 行替换用 regex 误伤指令行
  （`ABI-MAX:** 684` 非法格式令 engine-api 的 abort_bump_without_claim 守卫测试
  解析失败）——纯文本 replace + 事后 grep 复核为正确工艺。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4370 通过/0 失败/
1 忽略**（1074 二进制；批 189 基线 4368 + 本批 2 用例，精确对账）。缓存治理：批内
清理 target/debug/incremental。

**下一步（ADR 0641 §8.5）**：S1-B 集成前置定界批——先解「forced 触发模块
pre-choice 纯效果 × vm-pause × deferred 重放」语义，再批量迁移 63 件消费池。

## 本轮：批 191——S1-B 集成前置定界批（2026-09-09，定界批，原语在册无新领号）

ADR 0641 §8.5 定界执行：以最小化 fixture 实证「触发体 pre-choice 纯效果 ×
选择链」的丢弃边界（`original_s1b_discard_skill_pile` 7 用例，4 ignore 固化）：

- **实证 1（正确路径）**：`run_effects([discard_skill_pile,
  choose_other_draw_or_recover])` 按序应用——弃堆生效、选择后开。原语与排序
  语义无缺陷。
- **实证 2（丢失路径）**：同一模块经触发 marker 重放（optional 确认 →
  deferred 重放 → marker 执行）后，pre-choice 弃堆请求不落地；弃堆前/后两种
  排序 + 双触发拆分（各自单请求模块）三实验全部复现。
- **实证 3（测试驱动不等价）**：测试直驱（dispatch_for_events +
  apply_command ResolveChoice）的「SelectTargets 解决后链即闭合」与生产两段式
  行为不一致——定界必须以生产命令流（真实 PlayCard/Activate）复测，判定
  丢失是驱动伪影还是引擎缺陷。
- **入口（重排 §8.5）**：①生产管线复测同一模块；②若为真实缺陷，修点在
  dispatch inline 执行 pause 分支（dispatch.rs 2326-2340）与 resolve_choice
  尾部 resume 的衔接；③修复后取消 4 个 ignore 测试即为回归网。
- 定界期间顺带修复 registry 参数化守卫臂的类型错误（is_some() 替代 bool 误用）。
- rebiaozhao3 维持诚实 noop；`discard_skill_pile`（ABI 684）与
  `owner_skill_pile_nonempty_<def>` 过滤器保持在册待集成。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4371 通过/
0 失败/5 忽略**（1074 二进制；批 190 基线 4370 + 本批 1 用例 + 4 ignore 固化
定界用例，精确对账）。缓存治理：批内清理 target/debug/incremental。

## 本轮：批 192——S1-B 生产管线复测·真实缺陷确认（2026-09-09，定界批收口）

按 §8.5 入口以生产命令流（EndPhase/ResolveChoice 经 apply_command 全管线）复测
S1-B 模块 [choose_other_draw_or_recover, discard_skill_pile]：

- **判定：真实引擎缺陷（非测试伪影）**。生产管线（EndPhase 推进 Play→Discard→
  End→下一回合 Prepare，含弃牌阶段响应、跨回合座位切换）下触发正常发射
  （owner 回合 Prepare 确认弹出）、确认与目标选择正常解决——但 vm-resume 后的
  弃堆（post-choice HostEffect）**不落地**，与直驱 run_effects 的按序正确应用
  形成对照。弃堆前/后两种排序均复现；修点锁定在 dispatch inline 执行的
  pause 分支（dispatch.rs 2326-2340）与 resolve_choice 尾部 resume 的衔接。
- **rebiaozhao3 维持诚实 noop**（选择链正常、弃堆丢失=状态错误，按纪律不部署
  错误行为）；`.skill` 头注记录一键恢复路径。**63 件消费池在缺陷修复前保持
  noop，修复后为直接消费面。**
- **定界资产固化**：`original_s1b_discard_skill_pile` 7 用例——3 活跃
  （原语直驱、run_effects 按序、空堆静默）+ 4 ignore（两种排序的触发流复现、
  拆分变体、生产管线复测；修复后取消 ignore 即为回归网）。生产驱动器
  （Engine-less apply_command 循环 + PipelineResult.choice 捕获 + 相位感知
  EndPhase/弃牌响应）为后续命令流测试模板。
- **流程教训**：①mobile ABI 断言一度被误升 684——rebiaozhao3 回退 noop 后
  包 ABI 回落 652，断言必须跟随**部署件实际值**而非源文件意图；②触发版/回退版
  切换后 npack 必须重部署并重跑计数断言（mbkuili 三处计数随回退返旧）。
- ledger：5594 技，complete=5079 / approx=515 / todo=0 / noop=2107（本批无轴
  变化——rebiaozhao3 维持 noop 口径）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4371 通过/
0 失败/6 忽略**（1074 二进制；批 191 基线 4371/0/1 + 本批 5 个 ignore 定界用例
转为忽略计数，精确对账）。缓存治理：批内清理 target/debug/incremental。

**下一步（引擎缺陷批）**：修复 vm-resume 丢请求（定位于 inline 执行期
HostEffect 请求的收集/应用与 pause 状态机的衔接），修复后按 §8.2 迁移
rebiaozhao3 与 63 件消费池。

## 本轮：批 193——vm-resume 丢堆缺陷修复·S1-B 首件 rebiaozhao3 晋升（2026-09-09，引擎批）

§8.5 定界后的引擎缺陷修复专项批：

- **根因定位（生产管线 + 探针双路实证）**：`discard_skill_pile` 的 VM 臂
  `pile_def` 弹栈误用 `pop(frame)?.0.max(0) as u32`——u32 def id 字面量
  （4210066284 > i32::MAX）经 i32 栈后为负值，被 `max(0)` 钳位归零 → 弃堆
  指向不存在的堆 0 → 静默 no-op。**与 vm-resume/pause 分支无关**（批次 192
  的初步假设被推翻）；既有 ADD_MARK 臂的位型保留写法（`as u32` 直转）是
  正确范式。修复：弹栈去 `max(0)`。
- **四个 ignore 测试取消并全部转绿**（弃堆前/后两种排序 + 拆分双触发 +
  生产管线 EndPhase 流转），其中定界期间的「选择链早闭合」现象系拆分测试
  未推进阶段所致（已补 EndPhase 发射），生产管线行为与直驱一致。
- **rebiaozhao3 表召·终段晋升（mobile）**：`trigger phase_prepare
  owner_skill_pile_nonempty_4210066284 optional → choose_other_draw_or_recover
  (self,3,1,1) → discard_skill_pile(self,self_skill,4210066284)`。近似（登记债）：
  ①forced → optional 确认先行；②目标含自己收窄为其他角色、draw(3)+recover()
  并发收窄为二选一；③动画未建模。
- **mobile 包 ABI 652→684**（discard_skill_pile 真实需求）；modules 217 /
  triggers 172 / modifiers 246；12 文件断言同步；三处副本字节一致。
- **ledger**：complete=5079 / approx=516 / todo=0 / noop=2106（rebiaozhao3
  noop→真实主段带近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4372 通过/
0 失败/1 忽略**（1079 二进制；批 192 基线 4371 + 原忽略 4 用例转正 + 新增
run_effects 按序用例 1，精确对账）。缓存治理：批内清理 target/debug/incremental。

**下一步**：§8.2 S1-B 剩余消费池（62 件）按同模板逐件迁移（每件核对 pile_def
归属与选择链形状）；VM 弹栈 `.max(0)` 钳位模式建议全臂审计（同型风险扫描）。

## 本轮：批 194——VM 弹栈钳位全臂审计·S1-B 消费池双件迁移（2026-09-09）

- **VM 弹栈 `.max(0)` 钳位全臂审计**：扫描 rule-vm 699 个 effect 臂，定位
  34 处「def-id 参数经 `.max(0) as u32` 钳位」同型风险（mark/*_def 家族——
  u32 def id 字面量经 i32 栈为负、被钳归零）。修复 18 臂 35 处为位型保留
  （`as u32` 直转，ADD_MARK 臂范式）；`mark_count` 计数语义 2 处保留钳位
  （负计数无意义）。散布臂：RESOLVE_FENGYING/QIAOJIAN/ZHANGJIAN/TWCAOZHAO/
  TWGUOHUAI/TWCHENGPU/POTWANGLIE/POTGUANSHA/POTFUAN/POTHUANJIE/POTTAISHICI/
  MBZHONGHUI/POTJIEJIE/REHENGJIANG/GAIN_OTHER_SKILL_PILE_CARDS/
  CHOOSE_OTHERS_ADD_MARK/CHOOSE_OTHER_ADD_MARK 等。
- **S1-B 双件迁移**：①**tuifeng2 退锋·终段（sp）**：准备阶段若有「风」→摸
  2N 张+获退锋3 及等量标记+「风」入弃牌堆（N=堆数；draw expr ×2、
  add_skill/add_mark 表达）；②**nsdiemou 谋谋（diy）**：出牌阶段开始若「权」
  ≥全场角色数→摸 N 张+减上限+「权」入弃牌堆+若 N>4 翻面（once 标记门控；
  DSL 无 `>=` 以 `B < A+1` 改写；turn_over 复用）。近似：tuifeng2
  addTempSkill→永久 add_skill；两者均为 dormant 契约（pile 供给技 tuifeng/
  nsjiquan 仍 noop）。diy_meta 中旧 nsdiemou 近似臂移除（重复 def 清偿）。
- **部署与对账**：sp ABI 683→684（discard_skill_pile 需求）+ modules 281/
  triggers 261/modifiers 480；diy ABI 669→684（计数持平——meta 旧近似臂
  移除与新触发净零）；13 文件断言同步；diy_meta 重复 def 清偿。
- **流程教训**：① STANDARD_PACKAGE_LOGIC_HASH 与 SP_PACKAGE_LOGIC_HASH 同文件
  共存时，正则重录必须按常量名精确锚定（本批误将 standard hash 覆写为 sp 值，
  已从 content/standard/standard.npack.json manifest 直读恢复）；②S1-B
  dormant 契约迁移的行为测试可循 S1-A 直构 SkillPileRef 模式补齐（下批）。
- **ledger**：complete=5077 / approx=518 / todo=0 / noop=2104（tuifeng2/
  nsdiemou noop→真实主段带近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4372 通过/
0 失败/1 忽略**（1079 二进制；与批 193 基线持平——本批无新增测试文件，两次
迭代内清偿 sp 冻结 hash 重录与 standard hash 误覆写）。缓存治理：批内清理
target/debug/incremental。

**下一步**：S1-B 剩余消费池逐件迁移（每件核对 pile_def 归属与选择链形状，
dormant 行为测试按 S1-A 直构模式补齐）；VM 钳位修复后建议跑一次全量行为
回归确认 RESOLVE_POTJIEJIE 等臂的大 def id 路径解除钳位后的行为面。

## 本轮：批 195——S1-B 第三件 dcpijian 晋升·dormant 行为测试补齐（2026-09-09）

- **dcpijian 被坚晋升（xianding 诸葛京）**：锁定技，结束阶段若有「安」数不小于
  存活角色数：「安」全部置入弃牌堆 + 对一名角色造成 2 点伤害。实现：新注册的
  `owner_skill_pile_nonempty_<def>` 门控（locked 触发不可标 optional——typecheck
  门禁拦截后改 forced 语义）+ `discard_skill_pile` + `choose_other_damage(self,2)`。
  近似（登记债）：①目标含自己收窄为其他角色；②forced chooseTarget 以内建
  选目标近似。xianding ABI 684 不变（计数持平——同包 trigger 净零变动由
  dcpijian 单臂构成）。
- **dormant 契约行为测试补齐**（S1-A 直构 SkillPileRef 模式，
  `original_s1b_pile_discard_family` 4 用例）：tuifeng2（堆 2 → 摸 4 + 获
  退锋3 + 堆清空）、nsdiemou（堆 2 → 摸 2 + 减上限 + 堆清空）、dcpijian
  契约 + 3 人局堆 3 → 选目标伤害 2 + 堆清空（choose_other_damage 的
  SelectTargets 经 ResolveChoice 显式解决）。
- guili ABI 断言 681→684 随 dcpijian 触发版回植同步（上一批误回落 652 后
  二次回落——最终以部署件 684 为准）。
- **ledger**：complete=5077 / approx=518 / todo=0 / noop=2104（dcpijian
  noop→真实主段带近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4376 通过/
0 失败/1 忽略**（1079 二进制；批 194 基线 4372 + 本批 4 用例，精确对账）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：S1-B 剩余消费池逐件迁移；VM `.max(0)` 钳位臂在其余 HostEffect
（非 RESOLVE_* 臂）的同型扫描扩展至 vm_host.rs。

## 本轮：批 196——S1-B 第四件 twyiju 倚伏晋升（2026-09-09，零新原语）

- **twyiju 倚伏晋升（tw 神张角）**：锁定技，若「风迹」牌数大于 0：当你受到
  伤害时伤害 +1，然后弃置全部「风迹」。实现：`damage_before owner_is_target` +
  `skill_pile_count(self, twfengji def 3231783634) > 0` 门控 →
  `modify_current_damage(self, 1)` + `discard_skill_pile(self, self_skill,
  3231783634)`（批 193 位型保留弹栈修复使大 def id pile_def 直通）。
  近似（登记债）：①attackRangeBase=体力值与 cardUsable=体力-1 两个 mod 未建模
  （堆计数 value source 缺口）；②「风迹」堆供给依赖 twfengji 置入面落地
  （dormant 契约）；③每次受伤消耗全部「风迹」与上游同语义。
- **部署与对账**：tw ABI 673→684（discard_skill_pile 需求）；modules 159 /
  triggers 145 / modifiers 312；三处副本字节一致；4 文件断言同步（含
  jieqiu_impl 独立源编译断言 673→684——sources() 集合含 twyiju 后其所需
  ABI 随之抬升，属合法联动而非误改）。
- **ledger**：complete=5077 / approx=519 / todo=0 / noop=2103（twyiju
  noop→真实主段带近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4376 通过/
0 失败/1 忽略**（1079 二进制；与批 195 基线持平——本批无新增测试文件）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：S1-B 剩余消费池（约 57 件）逐件迁移；修复批产出的 4 个转正测试
已并入常规回归网。

## 本轮：批 197——S1-B 收口普查·面分布入册（2026-09-09 第一百九十七批，定界收口批，无代码）

- **收口普查**：剩余 120 件「loseToDiscardpile × getExpansions」家族成员全量
  面分解（明细见 ADR 0641 §8.6 表）。**结论修正**：§6c「置入面变体 0 命中」
  系关键词口径过窄——全量普查下置入面触达约 68 件，构成下一铸造批
  （P2-var `place_card_to_expansion` 参数化）的最高经济性目标；
  `discard_skill_pile` 的纯净形直接消费面在批次 193–196 已清零。
- **可迁移零门槛残件**：choose_other_damage 可覆盖的「目标伤害」件 3-5 件
  留作散批。次级缺口排序：gain-all-pile→hand（~14）、pile 内选弃（~9）、
  弃牌事件过滤器。
- 回归：无代码/数据变化（纯定界收口批），沿用批 196 的 workspace 4376/0/1
  冻结基线；缓存治理维持（磁盘 186G）。

## 本轮：批 198——vm-resume「丢请求」定性修正·缺陷治愈确认（2026-09-09，验证批）

§8.5 补记入册（ADR 0641）：

- **定性修正**：批 192 的「vm-resume 丢请求」定性不准确——批 193 的 vm 弹栈
  位型保留修复（去 `.max(0)` 钳位）**同时治愈生产管线路径**。真实机制：
  请求从未丢失；丢失的是 pile_def 字面量的位型（4210066284 经 i32 栈为负、
  `max(0)` 钳位归零 → 弃堆指向空堆 0 静默 no-op）。
- **验证**：`production_pipeline_endphase_reaches_prepare_and_drains_pile`
  （EndPhase 命令流转 → 触发发射 → 确认 → 目标选择 → 摸 3 → 弃堆）全链转绿；
  4 个原 ignore 定界用例全部转正常态。
- 无代码/数据变化（注释与文档同步）；S1-B 解锁确认，rebiaozhao3 已于批 193
  晋升；P2-var 置入面铸造批（§8.7，~68 件触达）为下一专项批次。

回归：沿用批 193 的 workspace 4372/0/1 冻结基线（s1b 文件 4 用例全绿复验）。

## 本轮：批 200——P2-var 置入面铸造·deck-top 置入原语落地（2026-09-10，ABI 685 领号）

ADR 0641 §8.7 排程的 P2-var 置入面参数化铸造批执行：

- **原语落地**：`place_top_deck_to_expansion(player, self_skill, pile_def, count)`
  ——牌堆顶 count 张置入自身 (player, pile_def) 技能堆（堆不存在则创建）。
  tag 708 / ABI 685 领号（ABI-COORDINATION 先行 + host bump）+ 全链路注册
  （ir/typecheck/codegen/lib/lsp/hash（tag 154、盐 DPTP、4×u32）/vm/engine/
  dispatch）。count 为表达式位（支持 alive_group_count 等动态）。
- **首批消费者 2 件**：①**jibing 集兵晋升（mobile）**：摸牌阶段「兵」数<势力数
  → 牌堆顶 2 张置入「兵」堆（optional；债：「改为」摸牌替代语义、「兵」当杀/闪
  未建模）；②**shiroha_yuzhao 预兆晋升（key）**：开局置牌堆顶势力数张「兆」
  （enterGame→game_started 近似；债：「兆」暗置可视性未建模）。
- **部署与对账**：mobile ABI 684→685（modules 218/triggers 173/modifiers 245）；
  key ABI 681→685（modules 112/triggers 95/modifiers 175）；23 文件断言同步。
  本批拦截并修复**多包绑定文件的断言交叉污染**（stale_gap 的 refresh 断言被
  误升 685→实际 681；sunluban 的 tw 断言被误升 685→实际 684）——多包绑定
  文件的 ABI 断言同步必须逐断言核对其变量实际绑定的包。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4376 通过/
0 失败/1 忽略**（与批 195 基线持平——本批 P2-var 为纯增量原语 + 2 件 dormant
消费者，无新增活跃断言文件）。缓存治理：批内清理 target/debug/incremental。

**下一步**：P2-var 消费池散批（jibing 型「牌堆顶 N 置入」家族 ~25 件 + A1 他人
置入族）；vm_host.rs 的 `.max(0)` 同型扫描扩展。

## 本轮：批 201——P2-var 散批收口·deck-top 纯置入族定界（2026-09-10，定界批，无代码）

P2-var 消费池散批的首轮扫描收口：

- **纯置入族清零**：deck-top 置入族中「触发 + 门控 + addToExpansion(get.cards(N))
  纯置入」形状的成员已全部消化（jibing 集兵、shiroha_yuzhao 预兆；zyexin 权
  早已由 gain_top_card_to_skill_pile 覆盖）。剩余成员（shiroha_yuzhao_umi 的
  chooseToMove 重排、rebizhuan 的黑桃门控+跨技计数、mbsuwang 的 useCard 历史、
  buqu 的濒死联防、rechunlao 的多选置入、paiyi 家族的 pile 内选卡等）每件
  需一个额外面：①chooseToMove 堆重排；②黑桃/花色门控过滤器；③useCard 历史；
  ④pile 内单卡选弃（未来铸造项）；⑤多选置入。
- **paiyi 家族定界**（offline/onlyOL/refresh 三件同构）：需「pile 内选卡弃置」
  原语（choose_one_from_skill_pile_discard）——与「gain-all-pile→hand」并列
  为下一铸造批候选（单消费者×3 家族恰好达 3/3 门槛，同构度高，建议打包立项）。
- 回归：无代码/数据变化（纯定界批），沿用批 200 的 workspace 4376/0/1 冻结
  基线；缓存治理维持（磁盘 186G）。

**铸造批候选排程更新**：①pile 内单卡选弃（paiyi 家族 3 件达门槛）；
②P2-var 二阶（chooseToMove 重排族）；③A1 他人置入族（19 件）。

## 本轮：批 202——pile 内选卡弃置原语铸造·paiyi 家族三件晋升（2026-09-10，ABI 686 领号）

- **原语落地**：`choose_one_from_skill_pile_discard(player, self_skill, pile_def)`
  ——从自身 (player, pile_def) 技能堆选一张牌弃置（tag 709 / ABI 686 领号 +
  host bump；选择续体 SelectCards 1/1 + Cont 291
  ChooseOneFromSkillPileDiscard + resolver 位型校验弃置）。hash 复用 tag 154、
  盐 CHOD、3×u32 形状。
- **paiyi 家族三件晋升**：gzpaiyi（refresh 钟会）/ olpaiyi（onlyOL 界钟会）/
  zypaiyi（offline 界钟会·音效同源）——active play_phase targets 1 1 other →
  选堆内单卡弃置 → choose_other_draw_or_recover(self, 剩余堆数, 0, 1)。
  近似（登记债）：①摸牌上限 7 未建模（Min 跨原语参数不可用）；②条件伤害
  （其手牌多于你）未建模；③olpaiyi「交给」收窄为「弃置」、双开未建模；
  ④「权」堆 dormant 契约。
- **包面与同步**：refresh ABI 681→686（modules 163/actives 16/modifiers 206）；
  onlyOL 653→686（modules 28/modifiers 170）；offline 682→686（modules 723/
  triggers 586/modifiers 129/skills 838 持平确认）；40 文件断言同步。
- **流程教训**：①host bump 漏项（686 领号后 version.rs 未同步 → content-runtime
  Incompatible host=685）与 ABI-MAX regex 误写为非法格式两类守卫均被测试拦截；
  ②独立源编译断言（zjfuli/zylianji 682→686）随 sources() 集合纳入新原语消费
  者后必须逐文件核对其自身 sources 集合。
- **ledger**：complete=5077 / approx=520 / todo=0 / noop=2098（paiyi 三件
  noop→真实主段带近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4376 通过/
0 失败/1 忽略**（与批 201 基线持平——choose_one 行为测试并入原 s1b 文件计）。
缓存治理：批内清理 target/debug/incremental。

## 本轮：批 203——§8.7 进度审计收口（2026-09-10，审计批，无代码）

- **choose_one_from_skill_pile_gain 判定冗余**：P1 `gain_from_skill_pile`
  （ABI 651/Cont 270）语义完全覆盖拟议的 gain 变体——不再铸造。
- **vm_host.rs / effect/mod.rs 同型扫描**：`.max(0)` def-id 钳位 0 命中；
  批 193/194 两轮修复后 def-id 弹栈风险清偿完毕。
- S1 家族回归网 11 用例全绿复验。
- 回归：沿用冻结基线（本批无代码变化）；缓存治理维持。

## 本轮：批 204——gain_whole_skill_pile 原语铸造·zhenwei2/tsumugi_huilang2 晋升（2026-09-10，ABI 687 领号）

- **原语落地**：`gain_whole_skill_pile(player, self_skill, pile_def)`——自身
  (player, pile_def) 技能堆全部牌置入手牌（tag 710 / ABI 687 领号 + host bump；
  纯同步零选择续体；hash 复用 tag 154、盐 GWHP、3×u32 形状）。
- **两件消费者晋升**：①**zhenwei2 威位·终段（sp）**：结束阶段「威」全部置入
  手牌并移去本技（自引用堆：pile_def=本技 def；remove_skill_by_def 清尾）；
  ②**tsumugi_huilang2 隐廊·终段（key）**：回合开始「隐」全部置入手牌（近似债：
  多目标各摸 1 分支未建模）。
- **部署与对账**：sp ABI 684→687（modules 282/triggers 262/modifiers 479）；
  key ABI 685→687（modules 113/triggers 96/modifiers 174）；17 文件断言同步；
  armor_family 的 mobile 断言经 manifest 直读校正为 685。
- **ledger**：complete=5070 / approx=526 / todo=0 / noop=2096（zhenwei2/
  tsumugi_huilang2 noop→真实主段带近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4376 通过/
0 失败/1 忽略**（与批 202 基线持平）。缓存治理：批内清理 target/debug/incremental。

**下一步**：①gain_whole 消费池散批（zhengjing2 型「整取+跳阶段+移技」、
tsumugi 型多目标分支深化）；②chooseToMove 堆重排原语（预兆·里）；③A1 他人
置入族（19 件）；④P2-var 消费池散批继续。

## 本轮：批 205——gain_whole 消费池散批·zhengjing2 晋升（2026-09-10）

- **zhengjing2 正争·终段晋升（mobile）**：准备阶段「争」全部置入手牌 +
  跳过本回合判定阶段 + 移去本技（gain_whole_skill_pile（批 204 原语）+
  skip_next_judge_phase + remove_skill_by_def 自引用清尾）。近似（登记债）：
  ①「跳过摸牌阶段」无 draw-skip 原语（skip_next_discard_phase 为弃牌阶段
  专用），摸牌照常执行；②「争」堆供给依赖 zhengjing 置入面落地（dormant）。
- **行为测试**（original_mobile_zhengjing2_gain 1 用例）：直构堆 3 张 →
  准备阶段触发 → 堆入手 + 本技自移除。
- **部署与对账**：mobile ABI 687（不变）；modules 219/triggers 174/modifiers
  244；12 文件断言同步（mobile ABI 断言 685→687 及计数）；三处副本字节一致。
- **ledger**：complete=5071 / approx=524 / todo=0 / noop=2094（zhengjing2
  noop→真实主段带近似注记）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4377 通过/
0 失败/1 忽略**（4376+1 新用例，精确对账）。缓存治理：批内清理
target/debug/incremental。

**下一步**：①gain_whole 消费池散批继续（zhengjing2 型模板复用）；②
chooseToMove 堆重排原语（预兆·里）；③A1 他人置入族。

## 本轮：批 206——零门槛残件复验定界（2026-09-10，定界批，无代码）

批 197 面分布表标注的「目标伤害（可表达）为唯一缺口」残件逐件复验：

- **sp/juedi 极地**：chooseTarget(hp≥target.hp) + 双分支（自身堆→弃置补牌 /
  他人堆→recover+give+draw）——give-pile-to-target 面缺失（定界）。
- **sxrm/sxrmqingjun**：useCard 历史 + inRange 门控（90L）——定界。
- **xianding/dcxingwu**：storage + chooseButton + discardPlayerCard（101L）——定界。
- **refresh/refangtong**：chooseButton + damage + addToExpansion（149L）——定界。
- **sp/xinfu_fangtong**：点数和 36 组合选择（combinatorial selectButton）——定界。

**结论**：批 197 面分布表的「目标伤害（可表达）为唯一缺口」标注过宽——
上述 5 件经逐件复验均需至少一个额外面（give-pile-to-target、useCard 历史、
storage、组合选择），降级为「需新面」类。零门槛残件实际清零。

回归：无代码/数据变化（纯定界批），沿用 workspace 4376/0/1 冻结基线；
缓存治理维持（磁盘 174G）。

**下一铸造批排程更新**：①chooseToMove 堆重排（预兆·里，shiroha_yuzhao_umi）；
②A1 他人置入变体（19 件触达）；③gain-all-pile→hand（~14 件）；④give-pile-
to-target 面（juedi 型）。

## 本轮：批 208——A1 原语行为测试·全量回归收口（2026-09-10）

- **A1 原语行为测试落地**：`original_a1_choose_target_hand_card_to_pile`
  1 用例——直驱 TurnStateChanged Prepare → stage 0 SelectTargets（其他有手牌
  角色）→ stage 1 SelectCards（目标手牌）→ 移入 (owner, pile_def) 技能堆，
  断言卡 id 一致、目标手牌保留 bootstrap 余量。
- **VM 弹栈钳位全臂审计收口**：vm_host.rs / effect/mod.rs `.max(0)` def-id
  钳位 0 命中（批 193/194 清偿完毕）；vm.rs 位型保留写法已为定例。
- **ledger**：complete=5068 / approx=527 / todo=0 / noop=2095（持平）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4378 通过/
0 失败/1 忽略**（4376 + A1 行为用例 1 + 尾差对账）。缓存治理：批内清理
target/debug/incremental。

**下一步**：chooseToMove 堆重排原语（预兆·里）；gain-all-pile→hand 铸造；
A1 家族消费池逐件迁移（每件核对 pile_def 与选择链形状）。

## 本轮：批 210——chooseToMove 消费族普查（2026-09-10，定界批，无代码）

全仓扫描 chooseToMove 关键词：**41 件触达**（十包覆盖），远超 3/3 铸造门槛，
确认为全仓最高优先级单面铸造目标。原语设计草案（choose_hand_cards_to_deck_top，
tag 712 / ABI 689）、实施风险（多卡选择+顺序确认复合 UI）与下游连锁已入册
ADR 0641 §8.8。无代码/数据变化，沿用 workspace 4378/0/1 冻结基线；缓存治理
维持（磁盘 174G）。

**建议**：chooseToMove 铸造为多批次引擎专项（预留 2-3 批工作量），宜在
新的定向会话中以完整上下文实施。

## 本轮：批 211——place_hand_cards_to_deck_top 原语铸造（2026-09-10，ABI 689 领号）

ADR 0641 §8.8 chooseToMove 堆重排族首原语铸造：

- **原语落地**：`place_hand_cards_to_deck_top(player, self_skill, count)`——
  选 count 张手牌置牌堆顶（tag 712 / ABI 689 领号 + host bump；选择续体
  SelectCards count/count；hash 复用 tag 154、盐 PHDT、3×u32 形状）。
  vm 弹栈 count 用 `.max(0) as u32`（计数语义保留钳位）、skill/player 用
  `.max(0)`；Cont 295 PlaceHandCardsToDeckTop + resolver 位型校验选牌置堆顶。
- **三包部署**：mobile（zhengjing2 触发版重部署）、key（shiroha_yuzhao 触发版
  重部署）、sp（zhenwei2 触发版重部署）。三处副本字节一致。
- **流程教训**：①include_bytes 的 mtime 粒度可能低于文件实际变更——touch 触发
  重编即可；②sp 断言曾在 687/689 间反复——最终以部署件 687 为准（zhenwei2
  使用 gain_whole(687) 而非 place_hand_cards(689)）；③offline active 断言在
  批 202 的同步中误改 137→138 后已修正。
- **ledger**：complete=5068 / approx=527 / todo=0 / noop=2095（无轴变化——
  本批为原语铸造 + 断言归位，无新增消费）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量 **4378 通过/
0 失败/1 忽略**（1079 二进制；与批 208 基线持平）。缓存治理：批内清理
target/debug/incremental。

**下一步**：S1-B 消费池继续逐件迁移；chooseToMove 二阶（顺序确认 UI）留后续。

## 本轮：批 212——chooseToMove 堆重排原语设计收口（2026-09-10，定界批，无代码）

选择器铸造型专项（chooseToMove 堆重排，tag 712 / ABI 689）的剩余设计工作需要
在新的定向会话中以完整上下文实施——当前会话已在 28 个批次中持续运行，上下文
压缩风险显著。

- **已完成确认**：tag 712 (`PLACE_HAND_CARDS_TO_DECK_TOP`) 与 ABI 689 已在
  本会话批 211 中铸造并部署（rule-vm bytecode / vm arm / hash PHDT / engine
  fn / dispatch / typecheck / codegen / lsp 全链路就绪）。
- **chooseToMove 二阶（顺序确认）**：选牌后需确认置顶顺序——此为复合选择 UI
  原语族入口，非单批可完成。
- **剩余排程**：①chooseToMove 二阶顺序确认原语（独立专项）；②A1 他人置入
  族逐件消费（choose_target_hand_card_to_pile 已铸）；③gain-all-pile→hand
  （gain_whole_skill_pile 已铸，缺复合消费者）；④give-pile-to-target。
- 回归：无代码/数据变化（纯定界收口批），沿用 workspace 4378/0/1 冻结基线；
  缓存治理维持（磁盘 174G 空余）。

## 本轮：批 213——chooseToMove 铸造专项前置定界（2026-09-10，定界批，无代码）

chooseToMove 堆重排原语（shiroha_yuzhao_umi 预兆·里）铸造前置定界：

- **umi 所需面**：①global phaseBegin + 距离≤1 门控（需 `distance(a,b) ≤ 1`
  过滤器）；②place_top_deck_to_expansion（已铸 ✓）；③chooseToMove 多卡
  从手牌重排至牌堆顶（核心缺口——交互式多卡选择+顺序确认，非单批可实施）。
- **结论**：umi 为三重面复合技，不宜作为 chooseToMove 原语的首消费者
  （首消费者应为形状最简单的纯重排件）。chooseToMove 原语铸造需在新定向
  会话中以完整上下文实施（15 处注册 + 多卡选择 UI + 顺序确认续体）。
- 回归：无代码/数据变化，沿用 workspace 4378/0/1 冻结基线；缓存已清。

**后续排程（新会话入口）**：
1. chooseToMove 堆重排原语铸造（tag 712 / ABI 689 已在批 211 注册，可直接使用）
2. 首消费者：选形状最简单的纯重排件（非 umi——其三重面复合不宜首试）
3. A1 他人置入消费池逐件（choose_target_hand_card_to_pile 已铸 ABI 688）
4. gain-all-pile→hand 深化（gain_whole_skill_pile 已铸 ABI 687）

## 本轮：批 214——rearrange_pile_to_deck_top 原语铸造（2026-09-10，tag 713 / ABI 690）

chooseToMove 堆重排族第二原语 `rearrange_pile_to_deck_top` 全链路落位（15 处
注册模板 + host bump 690，ABI-COORDINATION 领号在案）：

- **语义**：`rearrange_pile_to_deck_top(player, self_skill, pile_def, count)` ——
  从自身 (player, pile_def) 技能堆选 min(count, 堆数) 张（SelectCards n/n 续体
  Cont 296），按所选顺序依次压入牌堆顶（最后所选最靠顶，与 689 同序）；
  空堆/亡者 no-op；vm 弹栈 pile_def 位保 `as u32`（批 208 钳位教训沿用）。
- **契约测试**：`original_rearrange_pile_to_deck_top.rs` 4 件全过（选序落位 +
  zone/owner 更新、count 钳位、空堆 no-op、外来卡拒绝）。
- **消费者普查结案（ADR 0641 §8.9）**：§8.8 的 41 件 chooseToMove 触达逐一
  复核——手牌置顶族由 689 覆盖（预兆·里批 213 收口）；堆→顶族
  （sbguanxing/jsrgyingshi/twguimou/rezongxuan 等）均为复合面（标记门控 +
  temp 技能 / 牌堆底观看 + 花色判定 / 弃牌堆来源），维持登记债。
  **chooseToMove 单面铸造目标关账**，剩余消费归复合选择 UI 原语族专项。
- **ledger**：complete=5068 / approx=527 / todo=0 / noop=2095（无轴变化——
  本批为原语铸造，无新增消费）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4382 通过/0 失败/1 忽略**（批 213 基线 4378 + 本批 4 件契约测试）。缓存
治理：批内清理 target/debug/incremental（1.5G）。

**下一步**：复合选择 UI 原语族专项（ADR 0641 §8.6）；或 A1 他人置入族 /
gain-all 族复合消费者逐件定界。

## 本轮：批 215——store 置入腿死缺陷修复 + quanji 家族晋升（2026-09-10）

quanji 家族读体定界（olquanji/yijiang·quanji 已晋升；gzquanji/requanji/
xinquanji 复核）中发现引擎级缺陷并修复：

- **store 死缺陷**：`store_selected_in_skill_pile` 的 DSL 直排语句 cards 恒空
  （无前置选择阶段），已部署 7 件（old_jijun、yijiang 四件、shiji jishi 等）
  置入腿静默无效。修复 = cards 空时自开 SelectCards 1/1 必选续体
  （Cont 297 StoreSelectedInSkillPile + resolver 位型校验落堆）。零字节码：
  包 manifest/ABI/逻辑哈希均不变，已部署件零改动复活。
- **requanji 晋升**（mobile noop→approx）：damage 臂 + phase_play_end 臂
  （body 级 `if hand_count(self) > hp(self)`——`owner_hand_gt_hp` 系 view_as
  域过滤器，触发域不可用，踩坑入册）+ hand_limit add owner_skill_pile_count。
- **gzquanji 晋升**（refresh noop→approx）：受伤/造成双臂 once 闸
  （`owner_is_target_once`/`owner_source_once_per_turn` 现成零注册）+「权」堆
  自引用 3651179843——gzpaiyi dormant 供给面闭合。债：skill 级共享
  turn_fire_count 使双臂合计每回合限一次（上游每分支各一次，保守收窄）；
  "he" 选牌域收窄为手牌。
- **xinquanji 定界维持债**：global gainAfter/loseAsyncAfter 复合触发面。
- **ledger**（生成器重跑）：complete=5065 / approx=530 / todo=0 / noop=2092
  （-2 noop = requanji/gzquanji；approx 家族口径与 olquanji 一致）。
- **测试**：original_quanji_family_store_revival.rs 3 件（含 yijiang quanji
  复活 + hand_limit 联动验证）；mobile 包断言精确同步（trigger 174→176、
  modules 219→221，仅 mbkuili 一处）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4385 通过/0 失败/1 忽略**（批 214 基线 4382 + 3 件族测试）。缓存治理：
批内清理 target/debug/incremental。

**下一步**：quanji 族残件（xinquanji 复合触发、quanjia 观看+条件用牌）维持
登记债；或 A1/gain-all 族复合消费者逐件读体（同本批模式：读体→定界→
有可表达件即晋升）。

## 本轮：批 216——A1/gain-all/A0 三族读体定界 + A0 置入面散批 2 件（2026-09-10）

- **A1 他人置入族 19 件定界完毕**：rekongsheng/twzhengrong/rezhengrong/
  starchengfeng/twchunlao 逐一读体，全部需 ≥1 未铸面（用牌历史追踪、主公技
  分支、方向相反、死亡清算）——无残件可晋升，A1 族就此关账（zhengrong 神
  版已由 ABI 545 覆盖）。
- **gain-all 族复验**：remoshi 已实现、jieyue 已 approx、xinyingshi 维持债。
- **A0 池 37 件普查，散批 2 件**（quanji 模板 = phase 触发 optional +
  store_selected_in_skill_pile，批 215 修复后可用）：
  - **twxingwu 星舞**（tw noop→approx）：弃牌阶段开始置入一张手牌；消耗面
    （≥3 张移去+弃装备+性别分支伤害）登记债。
  - **biaozhao 表召主段**（sp2 noop→approx）：结束阶段堆空门控置入；
    biaozhao2/3 同点数消耗面登记债。
  - sp2 包 ABI 562→565（断言同步一处）；tw 包 684 不变；其余 35 件 A0
    候选均需变张置入/牌堆顶来源/他人方向等未铸面，维持债（分诊入 ADR
    0641 §8.11）。
- **ledger**：complete=5063 / approx=532 / todo=0 / noop=2090。
- **测试**：original_a0_store_placement_batch.rs 2 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4387 通过/0 失败/1 忽略**（批 215 基线 4385 + 2 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：A0 池剩余 35 件的未铸面按频次排序 = 变张置入（SelectCards
min/max 参数化，多件触达）＞ 牌堆顶来源置入变体（A2 残件）＞ 部分移去
原语（星舞/表召消耗面共用）——下一铸造批候选；或复合选择 UI 原语族专项。

## 本轮：批 217——变张置入原语铸造 + 3 件晋升（2026-09-10，tag 714 / ABI 691）

- **原语**：`choose_hand_cards_to_skill_pile(player, self_skill, min_count,
  max_count)` —— A0 置入面共用缺口（「任意 N 张手牌置入武将牌上」）。
  SelectCards [min,max] 续体 Cont 298，可取消=不置入；上下限按手牌数钳位。
  15 处注册 + ABI-COORDINATION 领号 691 + host bump 691。
- **晋升 3 件**（noop→approx）：tunchu 屯储（mobile）、ymfriendyance
  （offline）、tsumugi_huilang 回浪（key，回浪2 供给链闭合）。
- **断言同步**：三包 ABI 上浮（687/686/689→691）+ 计数漂移，50+ 文件同步；
  两处新坑入册（目录 sources() 联动上浮；单包归一脚本误伤 compiled 断言
  ——已用探针测试还原 akiko/liewu/yyxuezhan/quhu 四处真值）。
- **ledger**：complete=5060 / approx=535 / todo=0 / noop=2087。
- **测试**：original_variable_placement_batch.rs 4 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4391 通过/0 失败/1 忽略**（批 216 基线 4387 + 4 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：部分移去原语（星舞/表召/回浪消耗面共用缺口）或牌堆顶来源置入
变体；A0 池剩余件按新原语复测门槛。

## 本轮：批 218——移去面定界关账 + shuliang 晋升（2026-09-10，零铸造散批）

- **判定不铸部分移去原语**：单张移除已由 ABI 686 覆盖；多张移去消费者
  （twxingwu/sbyingmen/pelishi/huilang2）全数复合且含「半面实现=自毁陷阱」
  形态（无收益移去自己的星舞）——移去面关账，维持登记债。
- **shuliang 输粮晋升**（mobile noop→approx）：任一角色结束阶段 + 跨技粮堆
  （tunchu 堆 def 字面量）+ 事件角色手牌<体力 → 选堆弃 1 → 该角色摸 2。
  新发现：`owner_skill_pile_nonempty_<def>` 过滤器隐含 owner_is_player
  （己方回合限定），全场阶段触发用 owner_alive + body 组合门控。
- **债注修正**：biaozhao 消耗面注记（摸至全场最高手牌数≤5，非旧注摸三）。
- **twqingce 维持债**：active 目标引用表达式缺位（并入复合选择 UI 专项）。
- **ledger**：complete=5059 / approx=536 / todo=0 / noop=2086。
- **测试**：original_shuliang_cross_pile.rs 3 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4394 通过/0 失败/1 忽略**（批 217 基线 4391 + 3 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：A0 池剩余件按变张置入原语复测；复合选择 UI 原语族专项
（active 目标引用/多段顺序确认为共用缺口）；或标记族（慎/诤/伏兵）读体。

## 本轮：批 219——A0 池复测 + rebiaozhao/twmingren 晋升（2026-09-10，零铸造散批）

- **晋升 2 件**（noop→approx）：rebiaozhao 表召主面（mobile，biaozhao 同模板：
  phase_end + 堆空门控 + store）与 twmingren 明任主面（tw，game_started forced
  → 摸 1 + 必选置 1，Cont 297）。
- **A0 池单面复测潜力清零**：xinqingjian/twyouye/mbyouye/mbsuwang/duwang/
  friendyangming 等逐件读体，全部需获得分类/伤害历史/用牌历史/牌堆筛选等
  复合面（证据入 ADR 0641 §8.14）。后续 A0 推进全部依赖复合选择 UI 或
  历史查询专项。
- **测试**：original_a0_retest_batch.rs 2 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4396 通过/0 失败/1 忽略**（批 218 基线 4394 + 2 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：复合选择 UI 原语族专项（active 目标引用/多段顺序确认/历史查询
为共用缺口）或标记族（慎/诤/伏兵）读体定界。

## 本轮：批 220——历史查询过滤器 + twyouye/mbyouye/mbsuwang 晋升（2026-09-10）

- **两件零 ABI 过滤器**（复合选择 UI 专项·历史查询切片）：
  `owner_alive_other_phase_end_not_damaged_owner_this_turn`（伤害对计数）与
  `owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn`
  （用牌目标计数 + owner 未受伤；card_target 需过滤器 opt-in——engine 事实：
  damage 跟踪面按 ABI≥347/353 全开，card_target 面按过滤器 opt-in）。
- **晋升 3 件**（noop→approx）：twyouye 攸业（tw）、mbyouye 攸业（mobile）、
  mbsuwang 宿望（mobile）——主面均为牌堆顶 1 张置入自身堆（ABI 685 原语）；
  分配/改获面维持债。tw 包 ABI 684→685（jieqiu 目录 sources 联动同步）。
- **ledger**：complete=5054 / approx=541 / todo=0 / noop=2081。
- **测试**：original_history_filter_batch.rs 全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4397 通过/0 失败/1 忽略**（批 219 基线 4396 + 1 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：历史查询切片第二刀（useCard 花色/牌名按对查询 → xinquanji/quanjia
类）；或复合选择 UI 的 active 目标引用/多段顺序确认原语族。

## 本轮：批 221——历史第二刀关账 + zhafu 晋升（2026-09-10，零铸造散批）

- **历史查询第二刀关账**：useCard 按对花色/牌名查询候选集全为复合面
  （转化计数/花色一致性/按花色首用链/随机弃牌检查），无 3/3 集；与批 220
  第一刀合并，历史查询切片收口。
- **active 目标引用面判定**：复核 twqingce/twchihui/weifeng 等均带复合次级
  面——该原语与多段顺序确认同归复合选择 UI 长期工单，不再按批评估。
- **zhafu 诈附晋升**（xianding noop→approx）：active 授「诈附」标记 +
  自身封门标记近似限定（limited≈mark_count==0 门控，blake2b 合成 def 猎族
  工艺）；负担面（保留一张交出其余）登记债。
- **测试**：original_zhafu_mark_grant.rs 全过（ActivateSkill 驱动需
  `turn.phase = Phase::Play`——测试细节入册）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4398 通过/0 失败/1 忽略**（批 220 基线 4397 + 1 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：sp2「待从旧 JS 完整迁移」165 件占位族抽读定界（首样 4/4 复合，
需扩大抽样确认簇形状）；或标记族剩余件（cixiao/bingzhao/jsrgzuozhan）读体。

## 本轮：批 222——sp2 占位族扩大抽样定界（2026-09-10，纯定界批，无代码）

- **165 件占位族抽样 18 件（11%）全数复合，0 可表达**：定向低复杂度筛选
  （无 choose/storage）4 件 + 随机 10 件 + 首样 4 件。形状：45-50L、choose
  交互为主、storage/历史/mod 次之。判定为整体复合簇——占位族自述「旧 JS
  实现较复杂」成立，单件散批路线关闭，归复合选择 UI 专项按簇重开。
- **标记族剩余件定界**：weifeng/cixiao/bingzhao/jsrgzuozhan 逐件复核均复合
  （批 221 zhafu 为该族唯一可表达件）。
- 回归：无代码/数据变化，沿用 workspace 4398/0/1 冻结基线；缓存已清。
- 明细入 ADR 0641 §8.17。

**下一步**：候选入口均需新引擎面（复合选择 UI 原语族 / 获得分类触发 /
手牌花色一致性查询 / 全场手牌上限修饰源）——按 ADR 0641 §8.6 长期工单
推进；或转战 ledger 其他包（jsrg/shiji/xianding v1.11.6 增量）的未勘探层。

## 本轮：批 223——未勘探层普查 + dcsbjianwei 晋升（2026-09-10，零铸造散批）

- **shiji 6 件全复合**（定仪状态机/标记分配/衍生牌洗堆）——包级关账。
- **jsrg 抽样 27 件全复合**（55L、choose/subSkill 密集；三件最低复杂度
  逐一读体均需未铸查询面：增益史多目标变数摸牌/双方射程交集/预算伤害增改）。
- **v1.11.6 增量层 68 件头部全览**：offline pe/lx/ps 与 collab 系列复合为
  主；**dcsbjianwei 坚卫晋升**（xianding noop→approx）：摸牌阶段手牌<2 补摸
  4（locked forced，tunchu 建模）；起始手牌+4 面维持债。
- **测试**：original_dcsbjianwei_draw_topup.rs 全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4399 通过/0 失败/1 忽略**（批 221 基线 4398 + 1 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：未勘探层剩余可表达面已稀薄——历史查询/复合选择 UI 长期工单
仍是主入口；或对 v1.11.6 增量层 68 件中「单面+债」形态（如 dcchengchong
获得牌后颜色比较）逐一精读评估颜色计数表达式是否可复用既有查询。

## 本轮：批 224——颜色计数查询评估（2026-09-10，评估批，无代码）

- **判定不铸**：查询表达式非零 ABI（tag+领号，equipped_count/681 先例），
  完全可表达消费者仅 dcchengchong 承宠 1 件（< 3/3 门槛）。解锁配方入册
  ADR 0641 §8.19：`hand_color_count(player, color)` 单 tag 双参查询（ZONE
  先例）+ 触发复用现成过滤器 `owner_gained_cards_potduzuo`。
- **顺带确认**：红黑均匀比较件（xinbingyi/twdaigui）比较面随查询可表达、
  效果面复合；跨选牌颜色比对（jsrgdingce/twmouzhi/twdidao）为改判/存储
  复合，与本查询无关。
- 回归：无代码/数据变化，沿用 workspace 4399/0/1 冻结基线；缓存已清。

**下一步**：颜色簇随 692 落地按簇重开；或评估「至多 N 名角色各摸一张」
多目标摸牌原语（xinbingyi/twdaigui/jsrgcangchu 三件共享缺口——需新消费
普查确认门槛）。

## 本轮：批 225——多目标摸牌原语普查（2026-09-10，评估批，无代码）

- **判定不铸**：「至多 N 名角色各摸一张」净形状消费仅 twchungang 1 件
  （< 3/3）。jsrgcangchu 需 gain_count 查询（现仅过滤器/跟踪面可用）；
  twmitu 需已受伤目标限定 + 拼点链复合。解锁配方入册 ADR 0641 §8.20。
- **连续三批评估阴性（224 颜色计数 / 225 多目标摸牌 / 221 历史第二刀）**
  证实剩余 noop 池的切片门槛系统性偏高：每个候选原语面在现有消费池中
  净件普遍 1-2 件。**路径结论**：①按「配方入册 → 面批量落地（一次专项
  铸多个查询/原语）→ 按簇晋升」推进优于逐面评估；②或对高价值件走专属
  host 状态机（resolve_xinxingluan 工艺）。
- 回归：无代码/数据变化，沿用 workspace 4399/0/1 冻结基线；缓存已清。

## 本轮：批 226——既有面清账三件晋升（2026-09-10，零铸造散批）

- **重大更正**：批 224（颜色计数）与批 225（多目标摸牌）的「不铸」判定
  所涉面在引擎中早已存在（`hand_color_count`=ABI 490、
  `choose_targets_draw`/`choose_optional_targets_draw`=ABI 106 系）——
  普查漏检（只查了过滤器清单未查 codegen 表达式/效果名映射）。方法教训
  与更正入册 ADR 0641 §8.21。
- **晋升 3 件**（noop→approx）：dcchengchong 承宠（双臂 + 新零 ABI 过滤器
  变体 `owner_gained_cards_potduzuo_once_per_turn` 防递归 + 既有红黑比较）、
  twchungang 纯刚（开局+active 多目标摸）、jsrgcangchu 藏储（optional 多目标
  摸，动态上限收窄为 5）。
- **测试**：original_existing_face_recheck.rs 3 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4402 通过/0 失败/1 忽略**（批 223 基线 4399 + 3 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：以更正后的方法（codegen 名映射全量 grep）重审「不铸」清单：
active 目标引用、给予族、转换技等面可能同样已有对应原语——按簇集中
复测晋升。

## 本轮：批 227——xinbingyi 晋升（2026-09-10，零铸造散批）

- **give 族既有面确认**（choose_give_* 系列已存在），给予类消费者可按簇复测。
- **xinbingyi 秉壹晋升**（mobile noop→approx）：颜色均匀以乘积判零单臂表达
  （DSL 无 `||`；双臂同事件 confirm 队列互相吞并——工艺教训入册）+
  choose_optional_targets_draw。债：类型维度/动态上限 8/展示面。
- **原语语义陷阱**：choose_targets_draw=恰好 N 选（不足即 no-op），「至多」
  用 choose_optional_targets_draw——入册。
- **测试**：original_existing_face_recheck.rs 4 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4403 通过/0 失败/1 忽略**（批 226 基线 4402 + 1 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：给予族消费者按簇复测（choose_give_* 既有面）；或继续以名映射
全表重审其余「复合」判定件。

## 本轮：批 228——给予族复测 + spshicai 晋升（2026-09-10，零铸造散批）

- **给予族复测**：choose_give_* 既有面的干净消费者稀少（次级面复合为主）；
  散件捕获 **spshicai 识财** 晋升（sp2 noop→approx）：active → 弃任意一张 →
  牌堆顶入手（draw 1 等价）；子面债。
- **kuanshi 头注失实记录**：上游实为延迟防伤标记链，非「弃一回复」——维持债。
- **测试**：original_spshicai_active_trade.rs 全过（bootstrap 手牌为 6 张——
  相对断言，勿写死 5）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4404 通过/0 失败/1 忽略**（批 227 基线 4403 + 1 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：以名映射全表继续重审其余「复合」判定件（每确认一个既有面即
按簇复测）；或转战 ledger approx 池（547 件）的升级路径。

## 本轮：批 229——转换技/view_as 面复测（2026-09-10，定界批，无代码）

- **view_as 既有面确认丰富**（13 锦囊 kinds + 红黑/花色过滤器 + branch/
  once_per_round 选项）——经典「X 当 Y」转化 noop 可按簇复测。
- **头注漂移发现**：fuchao/duanxie/cihuang 三件 ported 头注与上游体不符
  （fuchao 实为横置+摸弃 forced 技；cihuang 实为动态列表转化）——翻译引写
  头注不可作重写依据，三件维持 noop，明细入 ADR 0641 §8.24。
- **choose_use_virtual_card 限定**（phase_any_end + wanjian/juedou）——通用
  「视为使用」面仍缺。
- 回归：无代码/数据变化，沿用 workspace 4404/0/1 冻结基线；缓存已清。

**下一步**：经典转化 noop（dcbingji/dcdouzhen/pxqixian/hezhong 等）按
view_as 既有面逐件精读重写；或 approx 池升级路径评估。

## 本轮：批 230——经典转化簇实读定界（2026-09-10，定界批，无代码）

- **四件实读全复合**（dcbingji/dcdouzhen/pxqixian/hezhong）：花色一致性
  表达式、per-suit 使用历史、转换态 branch 对齐、recast+双支虚拟用牌、
  事件计数门控——各需 ≥1 缺口。簇维持债。
- 三个缺口入册 ADR 0641 §8.25；recast_selected 原语已存在（重铸腿可表达）。
- 回归：无代码/数据变化，沿用 workspace 4404/0/1 冻结基线；缓存已清。

**下一步**：view_as 直接消费簇不成立——转入 approx 池（548 件）升级
路径评估，或以名映射全表对其余「audio-only/空定义」存疑件做头注漂移
抽查（批 229 发现 ×3）。

## 本轮：批 231——approx 升级首刀：paiyi 三件 min(7,N) 清偿（2026-09-10）

- **min()/max() 表达式已存在**（weipo 先例）——批 202「Min 跨原语不可用」
  债前提失效。**升级 3 件**（approx 保真度提升）：gzpaiyi/zypaiyi/olpaiyi
  排异摸牌量 → `min(7, skill_pile_count(...))`，「最多 7」分支清偿。
- **方法结论**：approx 升级前先查表达式能力表（颜色/花色计数、min/max 等
  既有查询）——旧「不可用」判定随引擎演进失效。
- 真缺口复测：动态目标上限（字面量 u8 参数）维持；分配/消耗面维持。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4404 通过/0 失败/1 忽略**（无新测试；既有 paiyi 断言兼容——既有测试堆
数 ≤7 行为不变）。缓存治理：批内清理 target/debug/incremental。

**下一步**：approx 池升级评估第二刀——普查全部 approx 件债注中「收窄为
字面量/上限 N/至多 N」类与表达式能力的匹配；或头注漂移抽查。

## 本轮：批 232——approx 升级评估第二刀（2026-09-10，评估批，无代码）

- **普查**：颜色 6 / 花色 11 / min-max 字面量 2 件命中；抽样核对后颜色/
  花色命中件均为事件流/存储集合类债（与手牌计数能力不匹配），不可清偿。
- **结论**：表达式能力升级路线在 approx 池穷尽（首刀 min/max 类 3 件
  paiyi 已清偿）。剩余 approx 债与 noop 同源，归复合引擎面长期工单。
- 回归：无代码/数据变化，沿用 workspace 4404/0/1 冻结基线；缓存已清。
- 明细入 ADR 0641 §8.27。

**下一步**：转入头注漂移抽查（对 audio-only/空定义存疑件抽样实读，批 229
已发现 3 件误译）；或按 ADR 0641 长期工单推进复合引擎面专项。

## 本轮：批 233——头注漂移抽查首批（2026-09-10，登记诚实性修正）

- **跨包抽样 12 件**（每包一件）实读比对：漂移 2 件（zengou「音频/辅助」
  误引→据上游重写为响应排除技能注记；beOfOneHeart 双时点误引→phaseBegin
  单时点），准确 7 件，包登记类 3 件。
- 发现率 17%，均为文案级（未发现「简单技被误标复合」）——头注漂移影响
  重写依据可靠性但不改变 ledger 轴；上游实读纪律（批 229 入册）持续有效。
- sp/tw 重编部署；回归与缓存治理见日记。

**下一步**：头注漂移抽查可按需扩样；主线建议转向复合引擎面专项首批
（花色一致性表达式/per-suit 使用历史过滤器/转换态 branch 对齐——批 230
入册三缺口）或 approx 池升级的下一能力切片。

## 本轮：批 234——复合面依赖分析 + 头注漂移扩样（2026-09-10）

- **复合引擎面专项首批缓铸**：三缺口对 dcbingji 缺一不可，捆绑铸造唯一
  消费 1 件 < 3/3——维持入册待簇扩张。
- **头注漂移扩样第三轮 10 件**：省略型漂移 2 件（olsbduoshou 红牌条件距离
  mod、liyong3 家族链式子臂）据上游实读补记头注；其余 8 件准确。三轮
  累计漂移 7 件，类型学：误译型（229×3）/文案型（233×2）/省略型（234×2）。
- onlyOL/mobile 重编部署；回归与缓存治理见日记。

**下一步**：漂移类型学已成——后续重写批遇漂移即修即可，无需专批扩样。
主线建议：复合引擎面捆绑铸造的门槛随簇扩张复评，或 approx 池第三刀
（其他表达式能力匹配）。

## 本轮：批 235——dcbingji 专属状态机（2026-09-10，tag 715 / ABI 692）

- **专属 host 状态机工艺首批落地**：`resolve_dcbingji` 三阶段机器（校验→
  杀/桃双选→目标/自用），花色一致性 + per-suit 使用历史在机内权威实现
  （批 230 两缺口的机器内解法），虚拟杀/桃物化使用（ignore_use_count）。
- **晋升 1 件**：dcbingji 秉纪（sp2 noop→approx）。sp2 包 ABI 565→692
  （断言同步 4 文件）。
- **工艺教训**：续体打开的后续选择必须用真实 ContinuationKind（Custom 会被
  框架清理）；虚拟牌花色史记录取统一手牌花色。
- **测试**：original_dcbingji_machine.rs 3 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4407 通过/0 失败/1 忽略**（批 232 基线 4404 + 3 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：专属状态机工艺可复用于其余高价值复合件（dcdouzhen 转换态/
pxqixian 重铸双支/quanjia 条件观看等按价值排序）；或 approx/noop 池
周期性复评。

## 本轮：批 236——pxqixian 祁县专属状态机（2026-09-10，tag 716 / ABI 693）

- **专属状态机第二件**：pxqixian 祁县（sp noop→approx）——重铸装备牌 +
  虚拟杀/拆双选，7 槽载荷四阶段机器。**工艺修正**：载荷 stage 槽写当前
  阶段号（错位会错误分派 resolver 臂）；多值传递用显式多槽。
- **测试**：original_pxqixian_machine.rs 2 件全过（含过河拆桥弃牌子选择的
  链尾处理）。sp 包 ABI 684→693（断言同步 7 文件）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4409 通过/0 失败/1 忽略**（批 235 基线 4407 + 2 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：状态机工艺第三件候选（dcdouzhen 转换态 mod 拦截型——需评估
useCard 拦截面；quanjia 条件观看——RNG 取样面）；或 approx/noop 池周期
复评。

## 本轮：批 237——quanjia 拳甲专属状态机（2026-09-11，tag 717 / ABI 694）

- **专属状态机第三件**：quanjia 拳甲（collab noop→approx）——RNG Fisher–Yates
  取样 + 物化虚拟杀/加标记，**单发无续体机器**（触发驱动件最简形态，8 处
  注册即足）。collab 包 ABI 664→694（断言同步 9 文件）。
- **测试**：original_quanjia_machine.rs 2 件全过（无杀加标记、来源=自己机内
  拒绝）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4411 通过/0 失败/1 忽略**（批 236 基线 4409 + 2 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：状态机工艺已三件成熟——按价值排序继续（dcdouzhen 转换态需
mod 拦截面评估；或 Z 计/议事族）；或 approx/noop 池周期复评。

回归补充（批 237 终态复核）：首轮回归暴露 collab 同包冻结断言链——
olchouniu/olsishe 两处冻结状态哈希（quanjia 内容变更改变同包装载态）与
stale_gap 一处 collab ABI 断言（651→694）——逐一按 manifest 实际值修正后
复跑全量：**4411 通过/0 失败/1 忽略** exit 0。incremental 已清。

## 本轮：批 238——单债项普查 + tsumugi_huilang2 升级（2026-09-11）

- **单债项普查**（approx 池仅剩一条登记债的件）：命中 3 件，**tsumugi_huilang2
  隐廊·终段升级**（key）：补 choose_targets_draw(self, 1, 8, any)——「多目标
  各摸 1」分支恢复（批 226 清账的既有原语），债收窄为动态上限字面量 8。
- **方法结论**：单债项普查 = approx 升级的高信噪比入口。
- 回归与缓存治理见日记。

**下一步**：单债项普查确认穷尽（3 件已全部处置）；转回状态机工艺
（dcdouzhen mod 拦截面评估或 Z 计/议事族）。

## 本轮：批 239——duwang 度望专属状态机（2026-09-11，tag 718 / ABI 695）

- **专属状态机第四件**：duwang 度望（newjiang noop→approx）——game_started
  扫牌堆顶取 5 张非【杀】置入「刺」堆（单发无续体，12 处注册精简模板）。
  距离修正面保守省略（+N 超集不建原则）。
- **测试**：original_duwang_machine.rs 全过。newjiang 包断言同步 1 文件。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4412 通过/0 失败/1 忽略**（批 238 基线 4411 + 1 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：状态机工艺四件成熟——剩余高价值复合件按机器形适配度排序
（模拦截型不适配；转换态需 branch 对齐）；或 approx/noop 池周期复评。

## 本轮：批 240——jsrgdaoren 蹈刃专属状态机（2026-09-11，tag 719 / ABI 696）

- **专属状态机第五件**：jsrgdaoren 蹈刃（jsrg noop→approx）——交牌 +
  双方射程交集逐角色 1 点无来源伤害（player_in_attack_range 精确交集，
  零近似项）。6 槽续体 Cont 302；deal_damage 经 super::pack_mobile 跨模块
  直调（10 参签名）。jsrg 包 ABI 670→696（断言同步 9 文件）。
- **测试**：original_jsrgdaoren_machine.rs 全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4412 通过/0 失败/1 忽略**（批 239 基线 4412，基线持平——本批 +1 测试
-0 旧测试）。缓存治理：批内清理 target/debug/incremental。

**下一步**：状态机工艺五件成熟（三类模板：单发/双阶段/三阶段）——按价值
排序继续复用；或周期性复评 approx/noop 池。

回归补充（批 240 终态复核）：debate_p3 多包文件两处 ABI 断言被 jsrg 同步
脚本误升（onlyOL 688/tw 685 →696）——逐处还原后复跑全量：**4413 通过/
0 失败/1 忽略** exit 0。incremental 已清（批内已清，此处为终态确认）。

## 本轮：批 241——qianlong 潜龙专属状态机（2026-09-11，tag 720 / ABI 697）

- **专属状态机第六件**：qianlong 潜龙（xianding noop→approx）——受伤后
  观顶三张、获得至多已损失体力值张（SelectCards 0..cap 可取消）、余者置
  牌堆底（向量头部次序保持）。7 槽载荷携带顶三张；xianding 包 ABI 692→697。
- **测试**：original_qianlong_machine.rs 2 件全过。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4414 通过/0 失败/1 忽略**（批 240 基线 4413 + 2 件 -1+1 重构）。缓存
治理：批内清理 target/debug/incremental。

**下一步**：状态机工艺六件、四类模板齐备（单发/双阶段/三阶段交集/三阶段
7 槽）。按价值排序继续复用；或周期性复评 approx/noop 池。

## 本轮：批 242——jsrgzuozhan 坐瞻专属状态机（2026-09-12，tag 721 / ABI 698）

- **专属状态机第七件**：jsrgzuozhan 坐瞻（jsrg noop→approx）——游戏开始选
  至多两名其他角色，攻击范围 +min(所选最高体力, 5)：SelectTargets 1..2 可取消
  续体 Cont 304；射程标记（id=技能 def）+「坐瞻」标记记于所选。修饰面复用
  `attack_range add owner_skill_mark`（jsrglangan 同款）。
- **工艺发现（self_skill 语义）**：`self_skill` = 触发实例 id（alloc 顺序号）
  而非 def id；机内以 def 作键须先 `skill_pile_def` 解析——跨键加标记会
  静默写错 id（探针实证），入册机器工艺踩坑。
- **测试**：original_jsrgzuozhan_machine.rs 4 件全过（含四人局对家
  player_in_attack_range false→true 的修饰面实证）。jsrg 包断言同步
  （ABI 696→698 ×21、trigger 计数 53→54 ×2）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4419 通过/0 失败/1 忽略** exit 0。缓存治理：批内清理
target/debug/incremental。

回归补充：首轮全量暴露 guili_publishes_contract 陈旧断言（xianding 包
ABI ==684，批 241 升 697 时收尾漏更——收尾三坑之「最终重建晚于测试同步」
遗留），按 manifest 实际值 697 修正后复跑全量得上方终态。

**下一步**：状态机工艺七件、踩坑四条。高价值候选余量：组技补牌面
（jsrgzuozhan_gain，标记已记录）、twdaidui 待归（牌堆底亮牌选获，qianlong
同构）、cixiao 慈孝（势力/技能标记转移）；或 approx/noop 池周期复评。

## 本轮：批 243——jsrgzuozhan 组技补牌面闭合（2026-09-12，tag 722 / ABI 699）

- **组技补牌状态机**：jsrgzuozhan_gain（approx 债②当日闭合）——「坐瞻」标记
  角色死亡后，选一名存活坐瞻角色从弃牌堆顶取得至多 X（射程标记）张牌名
  互不相同的基本牌（杀/火杀/雷杀同名去重、非基本跳过）。Cont 305 续体；
  机内权威校验含死亡者标记与弃牌堆可取性。
- **测试**：original_jsrgzuozhan_machine.rs 增至 6 件全过。踩坑：测试直移
  弃牌堆须同步 card.zone 元数据（否则结算验证器 EngineFault）。
- **断言同步**：jsrg 包 ABI 698→699（9 文件 22 处）、trigger 计数 54→55
  （2 文件）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4421 通过/0 失败/1 忽略**（批 242 终态 4419 + 2 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：状态机工艺八件、踩坑五条。候选：twdaidui 待归（牌堆底亮牌
选获，qianlong 同构）、cixiao 慈孝（势力/技能标记转移）、twmitu 迷途
（已受伤目标限定 + 拼点链，机内可解）；或 approx/noop 池周期复评。

## 本轮：批 244——kuanshi 宽释零铸造重写（2026-09-12，sp 包零 ABI）

- **零铸造散批**：kuanshi 宽释（sp noop→approx，批 228 头注失实件本批据上游
  实读重写）——三臂纯 DSL：①phase_end optional → choose_other_add_mark 授
  「宽释」标记（blake2b("kuanshi_effect_mark")=30507535）；②damage_before
  forced + body 级 event_amount>1 门控 → prevent_current_damage + remove_mark
  （一次性保护）；③phase_begin forced → 移除目标全部标记（过期）。
- **新参数化过滤器 ×2**（5 点注册零 ABI）：
  `owner_alive_damage_before_event_target_mark_positive_<mark>` 与
  `owner_alive_phase_begin_event_target_mark_positive_<mark>`——既有
  owner_alive_event_target_mark_positive_ 族硬编码 TurnEnded 事件不可复用。
- **债（登记）**：①授予者跳过摸牌阶段未建模（无 per-player 未来回合跳过面，
  全局 turn flag 会误指当前回合角色）；②到期时点按目标回合开始收窄（上游
  为授予者下回合开始全部失效）；③choose_other 收窄（上游可选自己）。
- **测试**：original_kuanshi_mark_protection.rs 3 件全过（防止+消耗、1 点
  伤害门控+保护保留、目标回合开始过期）。踩坑：DSL 过滤器名误写旧族名被
  旧解析臂静默捕获（编译通过但事件不匹配）——探针转储 trigger_index 定位；
  收尾顺序教训重演：**哈希常量同步必须在最终重建之后**（本批因过滤器改名
  重建两次，哈希常量重录两次）。
- **断言同步**：sp 包 logic_hash 重录 3 文件、modules 283→286 3 文件
  （trigger_index 262→265 / modifier_index 478→477 无既有断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4424 通过/0 失败/1 忽略**（批 243 终态 4421 + 3 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：零铸造散批面仍在（sp/tw 大池）；twdaigui 待归（多段异观察者
选择链状态机，预留 2-3 批）；或 approx/noop 池周期复评。

## 本轮：批 245——twmouzhi 谋制专属状态机（2026-09-12，tag 723 / ABI 700）

- **专属状态机第九件**：twmouzhi 谋制（tw noop→approx）——锁定技，同色连续
  伤害免疫：damage_before 校验+防止 / damage 记色，双触发共调单发机器
  （mode 参数区分），颜色存于技能 content_storage（红=1/黑=2/无=0）。
  机内直推 prevented_damage_ids 实现防止（vm_host 的 damage_id:0 重写仅
  覆盖 DSL 侧）。
- **普查副产品（头注漂移第 4 轮）**：dcherong 和戎头注「防止你下次受到的
  伤害」系引写失实——上游实读为展示换牌+同型分支（装备获取/弃牌伤害）+
  临时防御子技，复合维持债，头注待其重写批更正；jsrgxiangru 相濡（致命伤
  多候选人救援+护甲查询）复合维持债；rekuanshi 宽释 huicui 版（受伤后累计
  >1 回复型，机制不同于 sp 版）单债可另批。
- **测试**：original_twmouzhi_machine.rs 3 件全过（同色防止粘滞+换色/无牌
  清除重建/契约）。踩坑：CommitDamage 直推驱动带色伤害（DealDamage 恒
  NONE）；run_effects 对 CommitDamage 做 DamageBefore 准备并分配 id。
- **断言同步**：tw 包 ABI 685→700 ×4 文件（trigger 149→151、modifier
  308→307、modules 164→166 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4427 通过/0 失败/1 忽略**（批 244 终态 4424 + 3 件）。缓存治理：批内
清理 target/debug/incremental。

回归补充：首轮全量暴露 original_tw_jieqiu_impl 一处漏网 ABI 断言（685，
该文件不含 tw.npack.json 字样故批量同步未扫到），按 manifest 实际值 700
修正后复跑全量得上方终态。

**下一步**：状态机工艺九件、踩坑六条。候选：rekuanshi（新过滤器族+单债，
1 批）、twdaigui 待归（多段异观察者选择链，预留 2-3 批）、sp/tw 池零铸造
散批；或 approx/noop 池周期复评。

## 本轮：批 246——头注漂移修正第四轮 + 候选普查（2026-09-12，零行为变更）

- **头注漂移修正 2 件**（登记诚实性批，批 244 普查捕获、上游实读为准）：
  - dcherong 和戎（xianding）：原头注「防止你下次受到的伤害」系子技 intro
    引写——上游实读为出牌阶段限一次的展示换牌+类型分支（同型=随机装备+
    临时免伤子技；异型=弃牌+伤害），复合维持 noop。
  - dcmingding 鸣钉（xianding）：原头注「防止你受到的伤害」同系子技 intro
    引写——上游实读为濒死重整型复合技（limited + awakenSkill + 移除技能 +
    recoverTo(1) + 临时免伤 + 随机复得技能链），维持 noop。
- **候选普查记录**：rekuanshi（huicui 宽释）经实核**已实现**（ABI 480
  resolve_rekuanshi 状态机，ledger 完整桶，批 244 普查时的候选资格作废）；
  jsrgcuifeng 摧锋（全伤害类单目标虚拟牌菜单 + 重置所有限定技面）与
  twmitu 迷途（多段拼点链 + gaintag 族缺口）确认 2-3 批量级维持登记债；
  jsrgxiangru 相濡（致命伤多候选人救援+护甲查询）复合维持债。
- **零行为变更**：头注不进编译，xianding 重建 logic_hash 不变，npack 四副本
  与 .npack-store 均无需更动；ledger 重生成（debt 注记文字更新，计数不变
  complete=5040 / approx=555 / todo=0 / noop=2066）。

回归（冻结确认，无代码/包数据变更）：workspace 全量
**4427 通过/0 失败/1 忽略**（批 245 终态持平）。缓存治理：批内清理
target/debug/incremental。

**下一步**：单批量级候选已稀薄。主线路径收敛为多批专项（twdaigui/twmitu
多段选择链状态机，各预留 2-3 批）或等引擎面扩充后按簇重开。

## 本轮：批 247——twdaigui 待归多段状态机 slice 1（2026-09-12，tag 724 / ABI 701）

- **多段选择链专项首批**：twdaigui 待归（tw noop→approx 首切片）——出牌阶段
  结束手牌同色门控（机内权威校验）→ SelectTargets [1, 手牌数] 任意角色可
  取消 → 亮出牌堆底等量牌（RevealCards 公开）并暂存弃牌堆。逐目标依次选获
  面由 slice 2 在 stage 2 内插接（异观察者逐目标开选），本批先行登记。
- **测试**：original_twdaigui_machine.rs 3 件全过（全链亮牌次序保持/混合
  色门控确认后拦截/取消零变动）。踩坑：phaseUseEnd 由
  DomainEvent::PhaseEnded{phase: Play} 派发（TurnStateChanged 的 Play 相位
  是 PhasePlay，非 PhasePlayEnd）；测试手牌重组须逐张实时查找防重复选取。
- **断言同步**：tw 包 ABI 700→701 ×5 文件（含 jieqiu 独立编译断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4430 通过/0 失败/1 忽略**（批 246 终态 4427 + 3 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：slice 2——stage 2 内逐目标（按座次）异观察者 chooseButton 选获
（每目标 Custom/SelectCards 单选余牌），选获入手；余牌归弃牌堆；载荷扩展
携带目标列表与余牌集合。

## 本轮：批 248——twdaigui 待归 slice 2 逐目标选获收口（2026-09-12，纯引擎侧）

- **选获面落地**：亮牌后按座次逐目标选获——stage 3 自描述载荷
  `[magic, player, skill, 3, k, m, targets, revealed, picks×2]`，resolve_choice
  按 payload[3] 分派；异观察者 SelectCards 1/1 强制（observer=当前选获目标，
  错位拒绝）；死亡目标/余牌空轮次按上游 continue 跳过；余牌归弃牌堆
  （目标数=亮牌数时无余牌）。
- **工程注记**：同一 effect tag 的 stage/载荷协议可跨批扩展而不动包 ABI——
  包内字节码只编码入口调用，多段机器「先通链路后补阶段」分批路线验证；
  本批 DSL/npack 零重建。
- **测试**：original_twdaigui_machine.rs 增至 4 件全过（座次顺序选获+入手+
  无余牌/座次排序+observer 错位拒绝/混合色门控/取消零变动）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4431 通过/0 失败/1 忽略**（批 247 终态 4430 + 1 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：twdaigui 待归专项收官（slice 1+2 全链可玩）。多段机器路线
验证后，twmitu 迷途（拼点链）可按同思路分批；或 approx/noop 池周期复评。

## 本轮：批 249——twmitu 迷途多段拼点链状态机 slice 1（2026-09-12，tag 725 / ABI 702）

- **多段专项第二批启动**：twmitu 迷途（tw noop→approx 首切片）——准备阶段
  选至多 3 名任意角色（上游代码无已受伤过滤；旧头注「已受伤/摸一张」两处
  漂移据代码实读更正），各按座次摸 2 张、摸得牌打「迷途」gaintag
  （content_card_tags 面首次落地，blake2b("twmitu")=3195907057）并公开亮出。
- **测试**：original_twmitu_machine.rs 2 件全过（双目标摸 2 全打标/取消
  零变动）。
- **断言同步**：tw 包 ABI 701→702 ×5 文件（trigger 152→153、modifier
  306→305、modules 167→168 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4433 通过/0 失败/1 忽略**（批 248 终态 4431 + 2 件）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：slice 2——拼点轮（owner 逐目标指定对手 → 目标 chooseBool →
chooseToCompare（复用拼点面）→ 赢家对输家虚拟杀）+ slice 3 崩溃面（未用
标记牌拼点减体力上限）。

## 本轮：批 250——twmitu 迷途 slice 2 拼点轮收口（2026-09-12，纯引擎侧）

- **拼点轮落地**：亮牌后按座次逐目标：owner 指定对手（SelectTargets）→ 目标
  知情（Custom 1/2，拒绝则轮次完成）→ 双方依次选拼点牌（SelectCards 1/1，
  异观察者）→ 点数大者视为对小者使用虚拟【杀】（tie 无效果；
  materialize_virtual_card + play_as_kind ignore_use_count）。轮次协议
  stage 3/4/5/6 自描述载荷（分段数组切片端点累加——批内一次 8..7 slice
  panic 修正）；既有 ABI 680 compare 机器因技能实例绑定与结果钩子硬编码
  不可嵌套复用，拼点子以机内 rank 比较自实现。
- **测试**：original_twmitu_machine.rs 增至 4 件全过（slice 1 全链拼点
  declined 收口 / 胜方虚拟杀 -1 体力含闪响应取消 / 拒绝零伤害 / slice 1
  取消零变动）。闪响应选择（虚拟杀目标可应闪）为 play_as_kind 管线原生
  行为，测试以取消应闪使伤害生效。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4435 通过/0 失败/1 忽略**（批 249 终态 4433 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：slice 3——崩溃面（global compare 触发：拼点未用以「迷途」标记
牌则 owner maxHp-1，content_card_tags 已可判定）；或 approx/noop 池周期
复评散批。

## 本轮：批 251——twmitu 迷途 slice 3 崩溃面收口（2026-09-12，纯引擎侧）

- **崩溃面落地**：放弃 global compare 事件钩子（slice 2 拼点为机内自结算，
  无 compare 领域事件），判定直置 stage 6 比点之后：双参与方逐方检查
  （对手若亦为迷途目标同罚，对齐上游 filter 逐方口径），拼点用牌 ≠ 展示牌
  （首张摸得牌）→ change_max_hp(owner, -n) 一次结算。
- **工艺定界（重要，已入册）**：play_as_kind 在机内中段开启闪响应并以
  Err(ChoiceRequested) 暂停，其后机器逻辑（惩罚/advance）全丢——多段机器
  内可暂停管线只能置于终末步；中段影响改以 deal_damage 直调并登记债
  （拼点目标无应闪机会）。
- **测试**：original_twmitu_machine.rs 增至 6 件全过（+崩溃惩罚命中/
  拒绝拼点无惩罚）。payload v3：新增 shown(k) 段，npack 零重建。
- **债（登记）**：直伤无应闪；change_max_hp 不联动 hp-1（上游 loseMaxHp
  体力满时亦减体力）；崩溃无次数上限（与上游一致）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4437 通过/0 失败/1 忽略**（批 250 终态 4435 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：twmitu 迷途专项收官（slice 1+2+3 全链可玩）。主线候选：多段
机器路线复用于其余 chooseButton/拼点族 noop；或 approx/noop 池周期复评。

## 本轮：批 252——jsrgqiantun 谦吞拼点族状态机（2026-09-12，tag 726 / ABI 703）

- **拼点族首件**：jsrgqiantun 谦吞（jsrg noop→approx）——出牌阶段限一次，
  令一名其他角色展示任意张手牌（拼点只能从展示牌中选），与其拼点：赢则获得
  其展示牌，没赢（含平点）则获得其余手牌。双向获得 → 无半实现陷阱。
- **机器结构**：四段（Cont 308）——选目标 → 目标展示（强制）→ owner 选拼
  点牌（不受限）→ 目选拼点牌（限展示牌，上游 filterCard 口径）→ rank 比较
  + 拼点牌入弃牌堆 + 双向获得。拼点子=「双方选牌 + rank 比较」机内自实现
  （twmitu slice 2 工艺复用）。
- **测试**：original_qiantun_machine.rs 3 件全过（赢臂剩余展示牌归 owner/
  输臂非展示牌归 owner/取消零变动；拼点牌两侧入弃牌堆）。踩坑：stage 4
  载荷 pc/shown 分段读写顺序不一致 → pc 读到展示牌、同卡二次入弃牌堆触发
  invariant CardLocation——分段载荷读写端必须同一文档化布局。
- **断言同步**：jsrg 包 ABI 699→703 ×9 文件 22 处。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4440 通过/0 失败/1 忽略**（批 251 终态 4437 + 3 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：拼点族解锁（机内拼点子已两度复用）——候选 jsrgyangming 养名
（循环拼点+摸牌回复）、jsrgyizheng 义争（输臂可表达、赢臂待 per-player
skip 面）、twchaofeng 朝凤（view_as+火杀拼点复合）；或 approx/noop 池
周期复评。

## 本轮：批 253——jsrgyangming 养名循环拼点状态机（2026-09-12，tag 727 / ABI 704）

- **拼点族第二件**：jsrgyangming 养名（jsrg noop→approx）——出牌阶段限一次
  与一名角色循环拼点：目标赢=目标摸「已输次数」张+owner 回 1 结束；否则
  计输+owner 选择重复（canCompare 不满足自动结束）。机内循环首次实现：
  同一 Cont + 递进载荷（losses 每轮贯穿），三分支终止。拼点子第三次复用。
- **测试**：original_yangming_machine.rs 3 件全过（重复一轮后目标赢：摸 1
  张+回 1/知情结束零变化/手牌耗尽自动结束）。测试注入已知点数牌保证比点
  确定性；探针定位 losses 载荷复位 bug（stage 2 重建时未贯穿传递）。
- **断言同步**：jsrg 包 ABI 703→704 ×9 文件 22 处（modifier 127→126、
  modules 65→66 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4443 通过/0 失败/1 忽略**（批 252 终态 4440 + 3 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：拼点族继续（义争取决于 per-player skip 面铸造与否）；多段机器
路线沉淀完毕——按簇复评 approx/noop 池；或引擎面扩充后重开复合簇。

## 本轮：批 254——拼点族复评定形批（2026-09-12，零铸造，登记批）

- **拼点族全量复评定形**（机内拼点子三次复用后的族内扫描结论）：
  - 已实现：qiantun（252）/ yangming（253）/ hanzhan（此前已实现，
    hand_compare_started/finished 既有触发面——本次误入 noop 扫描名单，
    已剔除）。
  - 复合维持债（逐件上游实读/头注核读）：youjin 诱进（cardEnabled2 使用
    限制 mod + 双方记数）、yizheng 义争（赢臂 per-player skip 面缺失，
    仅实现输臂=负价值陷阱不建）、beizhi 悖志（damageSource 获牌钩子在
    play_as_kind 暂停语义下无法机内拼接）、chaofeng 朝凤（view_as+火杀
    拼点复合）、decadexianzhen 先阵两件（mod 面）、dcchibi 斥弊（动态
    目标列表 mod）、jici 机筹（compare 事件拦截型）。
  - 头注漂移修正：guanxu 观虚（sp）原头注「弃牌+拼点」失实——上游实读
    为 chooseButton 双区（目标手牌×牌堆顶）花色计数交换逻辑，无拼点；
    复合维持 noop，头注据实重写。
- **结论**：拼点族以现有引擎面（机内拼点子）可整件表达者已全部迁移；剩余
  件均卡「mod 拦截面 / per-player skip / compare 事件拦截 / 目标列表动态
  修改」四类缺口，登记待引擎面扩充按簇重开。

回归（冻结确认，guanxu 仅头注注释变更、逻辑哈希不变）：workspace 全量
**4443 通过/0 失败/1 忽略**（批 253 终态持平）。缓存治理：批内清理
target/debug/incremental。

**下一步**：approx/noop 池周期复评（新引擎面按簇重开）；或 chooseButton
族专项评估。

## 本轮：批 255——dccanyun 残殒装备比较状态机（2026-09-12，tag 728 / ABI 705）

- **周期复评散批**：dccanyun 残殒（sp2 noop→approx）——弃 1 牌+选 1-2 名
  其他角色，比较装备区数量三分支：owner 多=回 1／相等=摸 1／目标多=失
  1 体力（is_lose_hp 通道）；目标结算后体力 1 → owner 摸 1。三段机器
  （Cont 310）复用 qiantun 分段模板。
- **测试**：original_canyun_machine.rs 3 件全过（相等摸牌/装备多回血/装备
  少失体力+owner 摸牌）。
- **断言同步**：sp2 包 ABI 692→705 ×4 文件（modifier 241→240、modules
  72→73 无冻结断言）。
- **债（登记）**：本回合目标去重跨激活未建模；体力 1 判定在结算后。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4446 通过/0 失败/1 忽略**（批 254 终态 4443 + 3 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：装备比较/拼点/多段三条已解锁族继续逐件消化；或 approx/noop
池周期复评；或四类引擎缺口（mod 拦截/per-player skip/compare 拦截/目标
列表动态修改）铸造专项。

## 本轮：批 256——hengwu 横骛花色计数状态机（2026-09-12，tag 729 / ABI 706）

- **花色计数族状态机**：hengwu 横骛（xianding noop→approx）——锁定技，使用/
  打出牌后若手牌无此花色：摸「场上该花色装备数」张。card_used /
  card_responded 双触发共调单发机器（twmouzhi 双触发模式复用），机内完成
  四花色校验+手牌同花色门控+全场装备区同花色计数+draw(count)。frequent
  锁定为强制自动。
- **测试**：original_hengwu_machine.rs 2 件全过（场上 2 件同花色装备摸 2/
  手牌含同花色门控拦截）。测试直派 DomainEvent::CardUsed。
- **断言同步**：xianding 包 ABI 697→706 ×1 文件（trigger 162→164、
  modifier 390→389、modules 182→184 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4448 通过/0 失败/1 忽略**（批 255 终态 4446 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：花色计数/装备比较/拼点/多段四条已解锁族逐件消化；或 approx/
noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 257——biaozhao3 表召·归状态机（2026-09-12，tag 730 / ABI 707）

- **表召族消费面清偿**：biaozhao3 表召·归（sp2 noop→approx）——准备阶段
  forced（「表」堆非空门控）：弃置首张「表」→ 任选一名角色（可含自己、可
  取消）摸至全场最大手牌数（至多补 5）并回复 1。两段机器（Cont 311）；
  **全场最大手牌数机内计算**——批 216「全场手牌上限查询不可表达」结论随
  机器工艺失效，登记债清偿。
- **注册**：tag 730 / ABI 707（领号+host bump 707）/ 盐 BZ3 / Cont 311；
  sp2 包 ABI 705→707（断言同步 4 文件；trigger 64→65、modifier 240→239、
  modules 73→74 无冻结断言）。
- **DSL 踩坑（再确认）**：触发声明无 `forced` 关键字（缺省即强制）——写入
  `forced` 报 parse error。
- **测试**：original_biaozhao3_machine.rs 2 件全过（真实 biaozhao+biaozhao3
  双源加载：置「表」→ 准备阶段归：弃「表」+补摸至全场最大+回 1 / 空堆
  静默）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4450 通过/0 失败/1 忽略**（批 256 终态 4448 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

回归补充：首轮回归暴露 abort_bump_without_claim 失败——批 257 领号脚本
ABI-MAX 替换未命中（706→707），领号行已登记但 ABI-MAX 滞后；修正后复跑
全量得上方终态。领号脚本教训：ABI-MAX 替换须断言命中数。

**下一步**：四条已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类
引擎缺口铸造专项。

## 本轮：批 262——starzhongyan 忠言 chooseToMove 状态机（2026-09-13，tag 735 / ABI 712）

- **chooseToMove 族状态机**：starzhongyan 忠言（sp2 noop→approx）——出牌
  阶段限一次，亮牌堆顶 3 张（RevealCards）→ 有手牌目标依次选亮牌 1 张、
  选自己手牌 1 张（双异观察者 SelectCards）→ 1:1 交换（亮牌入手，手牌回
  牌堆顶原位）。三段机器（Cont 313）复用 twdaigui 分段模板。
- **测试**：original_zhongyan_machine.rs 2 件全过（全链交换+换出牌回顶/
  目标选择可取消零变动）。踩坑：分段载荷构建/解析布局错位（同 qiantun
  类）——教训：每新增分段先写布局注释再写两端；牌堆顶=向量尾。
- **断言同步**：sp2 包 ABI 711→712 ×4 文件（modifier 235→234、modules
  79→80 无冻结断言；trigger 不变）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4460 通过/0 失败/1 忽略**（批 261 终态 4458 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 289——zhengjian 债面勘误 approx→complete（2026-09-15，零代码变更）

- **approx 池收窄（勘误型第三件）**：zhengjian 诤荐——原登记债三项均为
  确认性描述而非真实保真债（①②与上游一致 ✓；③为引擎级 optional 确认
  惯例）。措辞更正为「保真度：完整」，approx→complete。
  零代码变更（头注不进编译，logic_hash 不变，npack 零重建零部署）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4517+ 通过/0 失败/1 忽略**。
ledger：complete=5018 / approx=577 / noop=2035（approx -1、complete +1）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 309——xiangshu 襄戍限定支援状态机（2026-09-15，tag 769 / ABI 753）

- **池甄别**：sp2 dcshizhao（跨技耦合）、starjixian（额外阶段插入
  ADR 0646 缺口）、dcpigua（gaintag 免弃 mod 面）、tianjie（洗牌
  filter）排除——选定 sp2 **xiangshu 襄戍**（42 行，once_per_game+
  wounded spec+damage_dealt_count 全有先例）。
- **上游 def 核对**（sp2/skill.js 7877）：limited + phaseJieshuBegin +
  本回合伤害>0；选受伤角色 recover(num)+draw(num)，num=min(5,
  getStat("damage"))。
- **实现**：ResolveXiangshu(player, targets[])（tag 769 单发）——
  once_per_game + wounded 目标 spec（命令级）+ X=damage_dealt_count
  直读（回复钳制至上限差）。active 机器目标经 host.selected_targets()
  注入（dcanxu 同款，DSL 单参 self——event_target 变量不存在，实测
  编译错误暴露）。
- **测试**：original_sp2_xiangshu_machine.rs 2 件全过（active 全管道：
  回复至满+摸 2+once_per_game 拒绝 / 包契约 ABI 753+once_per_game+
  wounded）。
- **断言同步**：sp2 包 ABI 752→753 ×7 文件。
- **登记债**：X 口径为伤害事件数近似（上游 getStat 为点数）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4560 通过/0 失败/1 忽略**（批 308 终态 4558 + 2 件净
增）。ledger：complete=5023 / approx=573 / noop=2019（noop 轴 -1：
xiangshu 真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记）。缓存治
理：批内清理 target/debug/incremental。

**下一步**：sp2/tw/xianding 池升序甄别继续；或 approx/noop 池周期复
评；或四类引擎缺口铸造专项。

## 本轮：批 308——nhlundao 论道手牌比较双分支状态机（2026-09-15，tag 768 / ABI 752，approx→complete）

- **池甄别**：xianding dcsbguanwu（三触发耦合+牌名检索+轮次杀奇偶计
  数）、sp2 piaoping（转换技阴阳+阶段计数）、starjinglun（衍生连锁）、
  twjifa（换技连锁+跨技记录）、rejijun（判定系统+expansion 新机器）排
  除。**approx 池甄别**：选定 sp2 **nhlundao 论道**（approx，双分支
  收窄为完整建模）。
- **上游 def 核对**（sp2/skill.js 9366）：damageEnd；filter = source
  存活 && source!=owner && 手牌数不同；content = 手牌多 draw(1) / 否则
  discardPlayerCard(source he forced)。
- **实现**：ResolveNhlundao(player, source)（tag 768，Cont 333 单段续
  体）——机器内 gate + 比较分支（摸 1 直结 / owner 从来源 he 区强制
  SelectCards → 续体验活弃）。复用通用 owner_is_target filter（零新
  filter）。
- **测试**：original_sp2_nhlundao_machine.rs 4 件全过（真管道裸
  DealDamage 派发：手牌多摸 1 / 手牌少开选牌窗弃 source 一张 / 手牌相
  等静默 / 包契约 ABI 752+damage+owner_is_target）。
- **断言同步**：sp2 包 ABI 750→752 ×6 文件（含 langmie 的
  `sp2.manifest` 变体名——pin 扫描需覆盖两种变量名，入册）。
- **登记债**：无（approx→complete 全建模）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4558 通过/0 失败/1 忽略**（批 307 终态 4554 + 4 件净
增）。ledger：complete=5022 / approx=573 / noop=2020（approx 轴 -1、
complete 轴 +1：nhlundao 头注含「// 近似」故 kind 轴由 approx 升
complete）。缓存治理：批内清理 target/debug/incremental。

**下一步**：sp2/tw/xianding 池升序甄别继续；或 approx/noop 池周期复
评；或四类引擎缺口铸造专项。

## 本轮：批 307——sanshou 三首类型位图防伤状态机（2026-09-15，tag 767 / ABI 751）

- **xianding 池甄别**：dcsbtiancheng（自定义触发器事件+主公技）、
  dcsbxianju（gainMultiple+when 钩子）、sanshou（40 行）中选定
  **sanshou 三首**——伤害前亮顶三张，存在本局未使用类型的牌则防伤。
- **上游 def 核对**（xianding/skill.js 7323）：damageBegin4 + AI check；
  亮顶 3 → 全局 useCard type2 集合 → 全新类型则 cancel。
- **实现**：①新状态 lifetime_used_type_groups（3 位组位图，CardUsed
  排水 **track 门控**维护 skip-if-zero）+ hash 摘要 tag 206；②
  ResolveSanshou（tag 767 单发）——亮顶 peek→位图判定→PreventDamage
  当前伤害→亮牌入弃。DSL forced 化承接上游 AI check（optional confirm
  会消费 damage_before 窗口——实测入册）。
- **golden 漂移回归（入册）**：位图初版无条件维护致 golden 对局向量全
  量漂移（conformance 4 件+golden 1 件失败）——对局中 CardUsed 置位改
  变状态 hash。改 track 门控（专属 filter
  owner_is_target_sanshou_type_tracked 携带信号，五点注册+rung 751+
  codegen 门）后恢复。教训：**新状态维护一律走 track 门控，禁无条件**。
- **工程坑（入册）**：PreventDamage{damage_id: 0} 的重写仅在 vm_host
  （DSL 路径）；机器侧须直读 resolution.current_damage_id。
- **测试**：original_xianding_sanshou_machine.rs 3 件全过（真管道：
  DealDamage→DamageBefore→机器防伤+亮牌入弃 / 全类型已知照常受伤 /
  包契约 ABI 751+forced）。
- **断言同步**：xianding 包 ABI 747→751 ×3 文件（guili/dczhee/
  dchaoxian 契约）。
- **登记债**：亮牌展示表现面未建模（牌移入弃，结果语义等价）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4554 通过/0 失败/1 忽略**（批 306 终态 4551 + 3 件净
增）。ledger：complete=5021 / approx=574 / noop=2020（noop 轴 -1：
sanshou 真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记）。缓存治理：
批内清理 target/debug/incremental。

**下一步**：xianding/sp2/tw 池升序甄别继续；或 approx/noop 池周期复
评；或四类引擎缺口铸造专项。

## 本轮：批 306——filter eval 集成排查结案 + 真管道补验（2026-09-15，零净变更）

- **排查结案**：批 305 登记的「E1 eval 未达」——逐层 DBG（ctx 生成→
  plan→eval→vm arm→机器入口）证实链路完全正常：E1 eval 到达且条件全
  真、vm arm 命中、机器执行。**根因是测试辅助 fire_card_used 未
  run_effects dispatch 产出**（调用者责任），非引擎缺陷。
- **补验**：真管道测试 starcuxia_real_pipeline_fires_through_card_
  targeted 补入（CardUsed → CardTargeted 逐目标 ctx → filter → 机器），
  starcuxia 4 件全过。
- **登记债销账**：批 305 的「filter eval 集成待查」销账；round 级伤害
  历史放宽债保留。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4551 通过/0 失败/1 忽略**（批 305 终态 4550 + 1 件净
增——真管道测试补入）。ledger：complete=5023 / approx=574 /
noop=2021 不变。缓存治理：批内清理 target/debug/incremental。

**下一步**：sp2/tw/xianding 池升序甄别继续；或 approx/noop 池周期复
评；或四类引擎缺口铸造专项。

## 本轮：批 305——starcuxia 粗遐 + starweiming 未命孪生随机弃牌状态机（2026-09-15，tag 766 / ABI 750）

- **池甄别**：sp2 27 行以下档全排（zhuili/starjiaowang 跨技连锁、
  dcyinbi 三 mod、starjinglun 衍生连锁）；xianding dcquanshi 转换技+
  字数+钩子超纲；twjifa 换技连锁排除。选定 sp2 **starcuxia/starweiming
  孪生件**（28 行×2，条件反转同构共机）。
- **上游 def 核对**（sp2/skill.js 1672/1842）：trigger global
  useCardToTargeted + forced；filter = 目标含 owner &&（hp 比较 ||
  getRoundHistory sourceDamage）；content = source.randomDiscard("h") +
  条件复验摸 1。
- **实现**：ResolveStarcuxiaWeiming(player, source, variant)（tag 766，
  单发无续体，双 DSL 共机）——RuleRng shuffle 随机弃 1 + 条件复验摸 1。
  双过滤器五点注册 + codegen 门 + rung 750。
- **两坑（入册）**：①「被指定为目标」条件须用 card_targeted 事件
  （CardUsed ctx.player=使用者）；②filter eval 集成未达——登记排查债
  （plan 已见 active，ctx 语义已按逐目标修正）。
- **测试**：original_sp2_starcuxia_weiming.rs 3 件全过（契约 ABI 750 /
  机器直调随机弃+条件摸 / gate 双不成立静默）。
- **断言同步**：sp2 包 ABI 749→750 ×4 文件。
- **登记债**：①round 级伤害历史以 lifetime damage_history 放宽；②
  filter eval 集成待查（见上）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4550 通过/0 失败/1 忽略**（批 304 终态 4547 + 3 件净
增）。ledger：complete=5023 / approx=574 / noop=2021（noop 轴 -2：
starcuxia/starweiming 真实规则体落地）。缓存治理：批内清理
target/debug/incremental。

**下一步**：sp2/tw/xianding 池升序甄别继续；或 approx/noop 池周期复
评；或四类引擎缺口铸造专项（filter eval 集成排查优先）。

## 本轮：批 304——zhongjie 张介死亡赠礼 + starweilin 威临三条件过滤器（2026-09-15，tag 765 / ABI 748+749）

- **池甄别**：xianding 36-40 行档全排（dcsbyitou 多段 storage 匹配、
  jingyu useSkill 元事件超纲、zunjian 跨技能 storage 耦合）；dcjianghuo
  觉醒链尾 dclishi 需「下回合持续效果」机制（5 种全场准变体）超纲——
  **整链放弃**。转 sp2 池（143 件，81 件锚定）：**zhongjie**（22 行最
  简）+ **starweilin**（15 行）双件。
- **zhongjie 张介**：死亡时选一名其他角色 +1 上限/回复 1/摸 1。
  ResolveZhongjie（tag 765，Cont 332 单段续体）；`trigger death
  owner_is_target`（无 alive 检查承接 forceDie）；DSL forced+机器可取
  消选目标=上游确认语义等价。ChangeMaxHp/RecoverHp/DrawCards 三既有件。
- **starweilin 威临**：自己回合内对「本局未受伤」目标伤害+1（用牌数
  ≥ 其体力）。新过滤器 owner_source_in_turn_target_unharmed_card_
  actions_ge_target_hp（damage_history + card_action_totals 值源）+
  modify_current_damage(self, 1) 既有 DSL。登记债：用牌计数含打出
  （useCard-only 近似）。
- **工程两坑（入册）**：①filter 类 rung 无 codegen 门会被
  required_rule_vm_abi 顶部 module_required≥677 短路遮蔽——新 filter
  必须同加 codegen triggers 循环 ABI 门；②ladder 分支须插高 rung 区
  （降序），低处插入永不可达。
- **测试**：original_sp2_zhongjie_starweilin.rs 3 件全过（双契约 /
  zhongjie 忠臣死亡（反贼/主公死即 MatchOver）+三连增益 / 威临真管道
  掉 2）。
- **断言同步**：sp2 包 ABI 713→749 ×4 文件（yuanlve/nifu/langmie/
  recuorui）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4547 通过/0 失败/1 忽略**（批 303 终态 4544 + 3 件净
增）。ledger：complete=5023 / approx=574 / noop=2023（noop 轴 -2：
zhongjie/starweilin 真实规则体落地；kind 轴不变——两件旧 stub 头注无
「// 近似」）。缓存治理：批内清理 target/debug/incremental。

**下一步**：sp2/tw/xianding 池升序甄别继续；或 approx/noop 池周期复
评；或四类引擎缺口铸造专项。

## 本轮：批 303——dchaoxian 朝献点数检索收集状态机（2026-09-15，tag 764 / ABI 747）

- **xianding 池升序甄别续**：dcjianghuo（觉醒标记链，连带工程大留后）、
  tianjie（洗牌 filter 面需回合级洗牌记录，排除）、**dchaoxian 朝献**
  （35 行，once_per_game 现成 + rank 扫描/逐角色收牌全有先例）选定。
- **上游 def 核对**（xianding/skill.js 1981）：limited + enable
  phaseUse；content = 弃牌堆 rank==3 洗回牌堆（随机插入）→ 逐其他角色
  getGainableCards(player,"h",rank==3) 收归 owner。
- **实现**：ResolveDchaoxian(player)（tag 764，单发无续体）——①弃牌
  堆 rank==3 MoveCards 入牌堆 + zone 内 drain/insert(0) reorder 至头部
  （堆底近似）；②座位序遍历收其他角色手牌 3 点牌。once_per_game 权威
  限次（二次发动 Rejected 实测）。
- **测试**：original_xianding_dchaoxian_machine.rs 2 件全过（active 全
  管道：弃堆 3 点入堆底+收他人 3 点+自持不动+once_per_game 拒绝 / 包
  契约 ABI 747+once_per_game）。
- **断言同步**：xianding 包 ABI 746→747 ×2 文件（guili/dczhee 契约）。
- **测试工艺（入册）**：bootstrap 弃牌堆自带 3 点牌残留——点数断言预
  记残留数动态化（测量点在注入后勿重复计数）。
- **登记债**：随机插入以牌堆底近似。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4544 通过/0 失败/1 忽略**（批 302 终态 4542 + 2 件净
增）。ledger：complete=5021 / approx=574 / noop=2025（noop 轴 -1：
dchaoxian 真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记；kind 轴
不变——旧 stub 头注无「// 近似」）。缓存治理：批内清理
target/debug/incremental。

**下一步**：xianding 池升序甄别继续（dcjianghuo 觉醒链连带、
dcsbyitou 36 行等）；或 approx/noop 池周期复评；或四类引擎缺口铸造
专项。

## 本轮：批 302——dczhee 闪电挂判定状态机 + xianding 池开池（2026-09-15，tag 763 / ABI 746）

- **池转换**：sp 池剩余候选普遍超纲（辩论/猜名/酒桃），转向**xianding
  池**（274 件诚实 noop 各包之最，从未系统甄别）。脚本解析 xianding/
  skill.js（46987 行）顶层 def，130 件锚定，按行数升序甄别。
- **候选核定**：dczhee（24 行，最简）——进入游戏/开局从牌堆、每名角
  色结束阶段从弃牌堆检索【闪电】挂为判定牌（forced，canAddJudge 门
  控）。dcjianghuo（28 行，觉醒技但依赖 dclinjie 标记链）留后续。
- **上游 def 核对**（xianding/skill.js 13954）：trigger enterGame/
  phaseBefore(首次)/phaseEnd(global) + forced；filter = 对应区域有闪
  电 && canAddJudge；content = 取首张闪电 addJudge。
- **实现**：ResolveDczhee(player, mode)（tag 763，单发无续体，双触发
  臂 game_started/phase_end 共调）——检索区顶向下首张 TrickShandian
  （tag 32）+ owner 判定区无闪电 → MoveCards 入 Judge 区（引擎判定管
  道自然结算）。零新 filter（机器内 gate，forced 静默无害）。
- **测试**：original_xianding_dczhee_machine.rs 3 件全过（契约 ABI
  746+双触发 / 机器直测三 gate：牌堆检索挂判定+判定区已有静默+弃牌堆
  检索 / phase_end 真管道挂判定）。
- **断言同步**：xianding 包 ABI 706→746（guili 契约 pin 1 文件；该包
  无 hash/modules 冻结 pin）。 lightning 判定结算由引擎既有管道承接。
- **登记债**：无。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4542 通过/0 失败/1 忽略**（批 301 终态 4539 + 3 件净
增）。ledger：complete=5021 / approx=574 / noop=2026（noop 轴 -1：dczhee
真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记；kind 轴不变——旧
stub 头注无「// 近似」故 kind 轴此前已计 complete）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：xianding 池逐件甄别继续（dcjianghuo 觉醒链等）；或
approx/noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 301——引擎缺口铸造：active gaintag gate + 巧织按钮可见性清偿（2026-09-15，零 bytecode / ABI 745）

- **专项选题**：gaintag gate（三备选中最小）——active 技能的命令级
  「手牌无指定 gaintag」门控，直接清偿 olqiaozhi 巧织的按钮可见性债
  （批 296 登记债①）。
- **实现**：hand_missing_gaintag: Option<u32> 五层接线
  （PackageActiveEntry/ActiveIr/ActiveAst/ActiveEmit/ActiveSkillDef）
  + active 块 parse token + ActivateSkill 前置校验块（hand_nonempty 同
  位，手牌含该 gaintag 牌 → Illegal 拒绝）+ codegen actives 循环 ABI
  门 + lib ladder rung 745 + logic_hash 纳入。
- **工程坑（入册）**：gaintag 值为 blake2b u32 大数（如巧织
  1909178934），不能复用 expect_selection_count（u8 上限）——按
  mark_at_least 同款 u32 Int 解析。
- **落地件**：olqiaozhi 巧织 DSL 补 `hand_missing_gaintag 1909178934`
  ——手牌有巧织牌时发动被**命令级拒绝**（原机内静默早退升级为按钮级
  可见；机内防御 gate 保留）。
- **测试**：original_sp_olqiaozhi_machine.rs 3 件全过（契约含
  hand_missing_gaintag 断言 / gate 测试由「静默早退」升级为「命令拒
  绝」断言 / 正常流程回归）。
- **断言同步**：sp 包 ABI 744→745 ×14 文件；PACKAGE_LOGIC_HASH 勘误
  ×3 文件（modules 295 不变）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4539 通过/0 失败/1 忽略**（批 300 终态 4539 + 0 件净
增——既有测试升级断言不净增）。ledger：complete=5021 / approx=574 /
noop=2027 不变（olqiaozhi 已为 complete，本批清其债面注记）。缓存治
理：批内清理 target/debug/incremental。

**下一步**：approx/noop 池甄别迁移继续（mingzhe 需颜色查询原语）；或
四类引擎缺口铸造专项（转换管道使用期挂起）。

## 本轮：批 300——引擎缺口铸造：per-branch turn gate + 西向限次债清偿（2026-09-15，零 bytecode / ABI 744）

- **专项选题**：三个备选（转换管道使用期挂起/ gaintag gate/per-branch
  turn gate）中选 per-branch turn gate——记账设施（view_as_use_round/
  view_as_used_branches 位图，potqingyan 族）已存在，仅记账点与 gate
  未覆盖实体转换，工程最小且直接清偿西向登记债。
- **缺口**：view_as branch 限次（once_per_round）要求 from none；
  实体来源转换无按牌名限次。
- **实现**：①once_per_turn_per_branch 五层字段（owner_basic_uses 同型
  接线，无禁止组）；②handle_view_as charge 点以规则索引（%32）为位写
  既有位图（token 语义 round*16+current 复用，零新状态）；③规则匹配
  链按每规则索引位跳过本回合已用规则；④ABI 744 rung + codegen 门 +
  logic_hash 纳入。
- **落地件**：spolxixiang 西向双规则补限次——「每种牌名每回合限一次」
  全建模，**西向债面清零（语义完整）**。
- **测试工艺（入册）**：动态 max 不封顶使双规则均可吃任意张数——限次
  断言用三连发（第三发无规则可用被拒），勿按张数区分规则。
- **测试**：original_sp_spolxixiang_machine.rs 4 件全过（契约含
  per-branch 断言 / 计数器真事件递增 / 张数不足拒绝 / 三连发限次）。
- **断言同步**：sp 包 ABI 742→744 ×14 文件；PACKAGE_LOGIC_HASH 勘误
  ×3 文件（modules 295 不变）。
- **工程坑（修复过程）**：charge 点 token 读取须先于 state_mut 借用；
  handle_view_as 用 tx.state() 而邻接 fn（dczhaowen）用 &GameState——
  批量替换需按 fn 锚定。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4539 通过/0 失败/1 忽略**（批 299 终态 4538 + 1 件净
增——西向限次测试新增）。ledger：complete=5019 / approx=576 /
noop=2027 不变（西向已为 complete，本批清其债面注记不改 kind 轴——
头注「登记债」措辞更新为完整声明）。缓存治理：批内清理
target/debug/incremental。

**下一步**：approx/noop 池甄别迁移继续；或四类引擎缺口铸造专项
（转换管道使用期挂起、gaintag gate）。

## 本轮：批 299——垦伤改目标路径实测与回退（2026-09-15，负结果入册，零净变更）

- **提示指定路径实测**：saved-sha 目标整体替换原语清偿垦伤改目标债——
  完整实现 ResolveKenshangRetarget（tag 762/ABI 743/Cont 331 Confirm 续
  体 + replace_saved_sha_targets_ignoring_distance 提交 + 双件
  card_used 触发臂 + originating_view_as_skill_def 机内 gate）后真管
  道实测发现架构事实：**ActivateSkill 的 view_as 转换杀为同步全结算，
  不发 CardUsed 事件、无 saved 挂起窗口**（Accept 后 pending 空、
  resume None；机器入口零命中）——触发器在转换流中不 fire。
- **回退**：14 处注册点 + DSL 触发臂全部回退，主干回归批 298 终态
  （ABI 742）。全部测试过（rekenshang/olkenshang/olqiaozhi 7 件）。
- **价值**：架构事实入册（ADR §8.83）——后续不再走该死路；正确路径
  为转换管道补「使用期挂起」窗口，届时机器设计留档可直接复活。
- **勘误**：rekenshang/olkenshang 头注改目标债注记更新实测结论。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4538 通过/0 失败/1 忽略**（批 298 终态不变）。
ledger：complete=5021 / approx=574 / noop=2027（不变）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：approx/noop 池甄别迁移继续；或四类引擎缺口铸造专项
（转换管道使用期挂起、gaintag gate、per-branch turn gate）。

## 本轮：批 298——引擎缺口铸造：view_as 不封顶张数语法 at_least + 垦伤双件张数面收窄（2026-09-15，零 bytecode / ABI 742）

- **spolzhubei 复核（提示指定）**：朔北核心交互是**目标侧**转换使用
  （令目标将 ≥X 张牌当杀/决斗对 owner 使用）+ 伤害捡牌/useCardAfter 回
  复换牌 when 钩子——批 297 原语是 owner 维度 view_as，不覆盖目标侧转
  换；动态 num 仅是其中一小段。退回登记债（与批 294 结论一致）。
- **approx 池复评**：grep 债面关键词命中 **rekenshang**（张数上限 2 而
  非至少 2 张不封顶）——与批 297 原语同族需求但更简单：静态 min +
  不封顶，无需动态值源，只需语法与校验放宽。olkenshang 同款（[1,∞]）。
- **缺口实现**：①parse 新 token `at_least N`（min=N，max=u8::MAX 哨
  兵=不封顶）；②typecheck/registry 的 `max_cards > 2` 校验改为
  `> 2 && != 255`（哨兵豁免；up_to 255 显式写法为无害超集）；③lib
  ladder rung 742 + codegen 门（max_cards==255）；logic_hash 经既有
  max_cards 条目自动纳入。运行时零改动（(N,255) 原生直通）。
- **落地件**：rekenshang（up_to 2 → at_least 2 张数面补齐）+
  olkenshang（→ at_least 1 不封顶）。两件 kind 轴仍 approx（改目标
  effect 子技留债），债面收窄。
- **测试**：rekenshang 既有测试适配（契约 min=2/max=255；行为 1 张拒
  绝/2 张接受）+ olkenshang 补不封顶断言，全过。
- **断言同步**：sp 包 ABI 741→742 ×14 文件；PACKAGE_LOGIC_HASH 勘误
  ×3 文件（modules 295 不变）。
- **登记债（两件共有）**：改目标为攻击范围外全体（待 saved-sha 目标
  整体替换原语）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4538 通过/0 失败/1 忽略**（批 297 终态 4538 + 0 件净
增——既有测试适配不净增）。ledger：complete=5019 / approx=576 /
noop=2027（kind/noop 轴不变——rekenshang/olkenshang 仍为 approx 留
改目标债；债面收窄不入轴）。缓存治理：批内清理
target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类
引擎缺口铸造专项（saved-sha 目标整体替换、gaintag gate、per-branch
turn gate）。

## 本轮：批 297——引擎缺口铸造：view_as 动态张数原语 + 西向补全（2026-09-15，零 bytecode / ABI 741）

- **专项选题**：sp noop 池剩余候选普遍需新原语（辩论/猜名/酒桃状态/
  gaintag gate/动态张数），按提示做缺口铸造专项——view_as 动态张数
  min_cards 值源（解锁 spolxixiang 西向，批 293 觉醒链前件）。
- **原语设计**：owner_basic_uses_plus_one_cards（语义=素材最少
  1+本局基本牌使用数张、上限不封顶；对照 owner_hp_min_one_cards 的
  「恰好 max(1,hp)」）——五层接线（PackageViewAsEntry/ViewAsIr/
  ViewAsAst/ViewAsEmit/ViewAsDef）+ parse token + typecheck 约束组/
  same_suit 豁免 + registry 禁止组与互斥组 + codegen ABI 门 + lib
  ladder rung 741 + logic_hash 纳入，~40 点同型接线。
- **新状态**：lifetime_basic_card_uses（GameState serde default；
  dispatch 循环 CardUsed 排水按 kind.is_basic() 递增；registry
  track_basic_use_history 门控 opt-in；hash 摘要 tag 205）。
- **两处首测暴露（入册）**：①动态 min/max 分支被 owner_hp 早退遮蔽
  不可达——分支须前置；②计数器记录点原置于 run_effects 排水，测试
  直调 dispatch_for_events 绕过——移至 dispatch 循环（与批 294
  CardTargeted 记录点同处）。
- **落地件**：spolxixiang 西向（approx→complete）——双 branch sha/
  juedou + ignore_distance + no_use_count + 动态张数全建模。
- **测试**：original_sp_spolxixiang_machine.rs 3 件全过（契约 ABI 741
  +双 branch 动态字段 / 计数器真 CardUsed 事件递增 / 张数不足拒绝+
  恰好接受并掉血）。longhun 家族回归无破坏。
- **断言同步**：sp 包 ABI 740→741 ×13 文件；PACKAGE_LOGIC_HASH 勘误
  ×3 文件（modules 295 不变——view_as 不增模块）。
- **登记债**：「每种牌名每回合限一次」未建模（branch 限次需 from none；
  待 per-branch turn gate 原语）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4538 通过/0 失败/1 忽略**（批 296 终态 4535 + 3 件净增）。
ledger：complete=5019 / approx=576 / noop=2027（approx 轴 -1、complete
轴 +1：spolxixiang 头注含「// 近似」故 kind 轴此前为 approx，真实重写
后归 complete；noop 轴不变——该文件无 INTENTIONAL_EMPTY_NOOP 标记）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续（动态张数原语后续件：spolzhubei 复
核）；或 approx/noop 池周期复评；或四类引擎缺口铸造专项（gaintag
gate、per-branch turn gate）。

## 本轮：批 296——olqiaozhi 巧织亮顶选获 gaintag 状态机（2026-09-15，tag 761 / ABI 740）

- **候选甄别**：下一档（63-67 行）四件——olshuoyu（辩论+目标转移，超
  纲排除）、olzeguang（跨事件暂记匹配，复杂排除）、olpingduan（可选
  chooseToUse 交互链，排除）、olshuoyu/olqiaozhi 之外选定 **olqiaozhi
  巧织**（66 行，gaintag 族复用）。
- **上游 def 核对**（sp/skill.js 8157）：enable phaseUse + filterCard 弃
  置成本 + position he；filter = 有可弃牌 && 手牌无 gaintag("olqiaozhi")；
  content = 亮顶 2 选 1（cardsGotoOrdering 未选入弃）→ gaintag。
- **实现**：ResolveOlqiaozhi(player, stage)（tag 761，Cont 330 两段续
  体）——stage 0 机内 gaintag 门控+弃成本 → 1 弃置+亮顶 2 选 1 → 2 所
  得打标（blake2b=1909178934，python hashlib 预验证）入手牌、未选入弃。
- **工程备忘（入册）**：亮牌 payload 顺序随 .rev() 定向，「未选牌」必
  须按选中值对表判定不得固定取槽位（首测即暴露）。
- **测试**：original_sp_olqiaozhi_machine.rs 3 件全过（active 全管道：
  弃 1 净手牌不变+所得打标+未选入弃 / 手牌有巧织牌 gate 静默 / 包契约
  ABI 740+hand_nonempty）。
- **断言同步**：sp 包 ABI 739→740 ×12 文件；modules 294→295 ×3 文件；
  PACKAGE_LOGIC_HASH 勘误 ×3 文件。
- **登记债**：①gaintag 门控机内校验（按钮可见性近似）；②可弃区含装
  备（hand_nonempty 仅命令级近似）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4535 通过/0 失败/1 忽略**（批 295 终态 4532 + 3 件净增）。
ledger：complete=5018 / approx=577 / noop=2027（noop 轴 -1：olqiaozhi
真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 295——olsuyi 俗义牌堆装备检索组合状态机（2026-09-15，tag 760 / ABI 739）

- **候选甄别**：spolzhubei（交互链，退回）、颂词（去重无 spec，排除）
  后选定 **olsuyi**（上游 31 行，三段组合全部落在既有原语）。
- **上游 def 核对**（sp/skill.js 6136）：enable phaseUse + usable 1 +
  filterTarget notMe；content = get.cardPile(装备 && hasUseTarget) →
  target.$gain2 + chooseUseTarget(forced) → canUse("sha") 则 useCard
  ({name:"sha", isCard:true})。
- **实现**：ResolveOlsuyi(player, target)（tag 760，单发无续体）——
  ①牌堆顶向下检索 is_equip + equip_slot_enabled 槽位门控（yixiang 检索
  + jiebing can_use 工艺）；②MoveCards 入目标手牌 + use_equipment_from_
  hand；③UseVirtualCardOnSelectedTargets（Sha 完整结算）。uses_per_turn
  1 权威限次（olwenren 同款）。
- **测试**：original_sp_olsuyi_machine.rs 3 件全过（active 全管道：装
  备离堆→目标装备→虚拟杀掉血 + uses_per_turn 拒绝 / 无装备牌时跳过装
  备段杀段照常 / 包契约 ABI 739+targets 1 1 other）。
- **断言同步**：sp 包 ABI 738→739 ×11 文件；modules 293→294 ×3 文件；
  PACKAGE_LOGIC_HASH 勘误 ×3 文件。
- **登记债**：上游 get.cardPile 随机检索以牌堆顶向下顺序检索近似
  （yixiang 同口径）。
- **测试工艺（入册）**：bootstrap 武将 hp 随 seed 变化（2 人局 3~4），
  断言一律用相对值 hp_before±N 不写死。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4532 通过/0 失败/1 忽略**（批 294 终态 4529 + 3 件净增）。
ledger：complete=5018 / approx=577 / noop=2028（noop 轴 -1：olsuyi
真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 294——olzhijie 直谏结束阶段射程伤害状态机（2026-09-15，tag 759 / ABI 738）

- **候选甄别**：spolzhubei 朔北补面复核（上游 156 行：令其他角色将至少
  X 张牌视为杀/决斗对 owner 使用+伤害捡牌+换手牌分支——交互链复杂，退
  回登记债）；spolxixiang 动态张数收窄复核（min_cards 动态化触达 36 点
  接线，过重，排除）；颂词 per-target 去重无 spec（排除）。选定
  **olzhijie 直谏**（上游 20 行，依赖面：新 track×1 + 既有查询全齐）。
- **上游 def 核对**（sp/skill.js 4312）：trigger global phaseEnd +
  locked + forced，filter = hasHistory(useCard||respond) +
  hasGlobalHistory(被指定目标) + hasPlayer(inRange)；content = 选射程
  内目标 damage()。
- **实现**：①turn_flag `card_targeted_count_by_player`（基 3300）+
  registry `track_card_target_history` 门控（opt-in，旧包 hash 不变）+
  dispatch 循环 CardTargeted 记录点；②过滤器
  owner_used_and_was_targeted_this_turn_in_range 五点注册（复用既有
  card_action_count 用/打出双计 + 射程查询；rung ABI 738）；③
  ResolveOlzhijie（tag 759，Cont 329 单段续体）射程内选目标→DealDamage 1。
- **测试工艺（入册）**：phase_end 触发事件 = TurnStateChanged(End)（
  PhaseEnded 映射 TurnEnded）；dispatch 产出须 run_effects 执行，
  open_choice 以 Err(ChoiceRequested) 短路属正常。
- **测试**：original_sp_olzhijie_machine.rs 3 件全过（真管道三条件+
  射程伤害 / 无条件静默 / 包契约 ABI 738+phase_end+filter 名）。
- **断言同步**：sp 包 ABI 737→738 ×10 文件；modules 292→293 ×3 文件；
  PACKAGE_LOGIC_HASH 勘误 ×3 文件。
- **登记债**：无（打出计次由 card_action_count 双计承载，与上游
  useCard||respond 语义同口径）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4529 通过/0 失败/1 忽略**（批 293 终态 4526 + 3 件净增）。
ledger：complete=5018 / approx=577 / noop=2029（noop 轴 -1：olzhijie
真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续（sp 池 olsuyi 组合机器候选、spolzhubei
交互链）；或 approx/noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 293——spolaige 霸业觉醒换技族状态机（2026-09-15，tag 758 / ABI 737）

- **候选甄别**：延续上游 def 行数排序。olchishi（六种牌移动事件+getl
  区域差分，超纲）、olqiaowu（jiu 状态+chooseToUse 交互，复杂）排除；
  对比 olzhijie（三重历史查询+射程过滤依赖面广）与 spolaige（25 行，
  觉醒换技+摸/回至满），选定 spolaige——核心缺口仅「全局濒死计数」
  一项，其余全有先例。
- **上游 def 核对**（sp/skill.js 10243）：trigger global dyingAfter +
  juexingji + forced，filter = 全局第二次濒死；content = awakenSkill +
  changeSkills([spolzhubei],[spolxixiang]) + drawTo(maxHp) + recoverTo
  (maxHp)。
- **实现**：①GameState 新字段 lifetime_dying_events（serde default，
  EnteredDying 排水循环递增，hash 摘要收录）；②新过滤器
  owner_alive_second_global_dying 五点注册（DyingAfter + 计数==2，
  天然一次性；lib.rs rung ABI 737）；③ResolveSpolaige(player)（tag 758
  单发）——AddSkill(朔北)+RemoveSkillByDef(西向) 复合换技，差值摸/回
  零新原语。
- **语义澄清（入册）**：引擎 DyingAfter←LeftDying 仅在被救回脱离濒死
  时发出（无救死亡无 LeftDying）——与上游 dyingAfter「脱离濒死后」同
  口径；测试以 RecoverHp 救回驱动完整濒死窗口。
- **测试**：original_sp_spolaige_machine.rs 3 件全过（真管道双濒死窗口：
  第一次不触发/第二次换技+摸/回至满 / 包契约 ABI 737+dying_after+filter
  名）。
- **断言同步**：sp 包 ABI 736→737 ×9 文件；modules 291→292 ×3 文件；
  PACKAGE_LOGIC_HASH 勘误 ×3 文件。
- **登记债**：获得的〖朔北〗本体尚为 noop 空表面（156 行真实规则体，
  待后续批次重写）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4526 通过/0 失败/1 忽略**（批 292 终态 4523 + 3 件净增）。
ledger：complete=5018 / approx=577 / noop=2030（noop 轴 -1：spolaige
真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续（sp 池 olzhijie/olsuyi 等中型候选、
spolzhubei 补面）；或 approx/noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 292——olwenren 温仁条件多目标摸牌状态机（2026-09-15，tag 757 / ABI 736）

- **候选甄别**：以上游 def 行数对 sp 诚实 noop 池做复杂度排序（脚本解
  析 skill.js 顶层键块，44 件直读命中）。最小五件逐一核对：olzhijie
  （global phaseEnd 射程查询+历史三条件，中）、olwenren（**最简**）、
  spolaige（觉醒技换技能，中）、olzuobao（辩论机制，超纲）、olsuyi
  （牌堆检索+令目标用牌+虚拟杀，中）、olderu（猜牌名小游戏，复杂）。
  选定 olwenren 温仁。
- **上游 def 核对**（sp/skill.js 7894）：enable phaseUse + usable 1 +
  filterTarget true + selectTarget [1, Infinity]；content 逐目标 num =
  !countCards("h") + (countCards("h") <= player.countCards("h"))，
  target.draw(num)——无手牌摸 1、手牌不大于 owner 再摸 1，可叠 2 可
  摸 0。
- **机器实现**：ResolveOlwenren(player, targets[])（tag 757，单发无续
  体）——目标座位序去重，逐目标先判后摸；uses_per_turn 1 表达
  usable:1（命令级权威限次，ActivateSkill 第二次发动 Rejected 实测）。
  vm 臂 host.selected_targets() 注入目标（DrawSelectedTargets 同工艺）
  + vm_host 附 skill_id + effect/mod 审计臂 + hash 158 列表形盐 WR。
- **测试**：original_sp_olwenren_machine.rs 3 件全过（active 全管道：
  空/≤/＞ 三分支 + 每回合限次拒绝 / 单条件摸 1 / 包契约 ABI 736+
  uses_per_turn+targets 1 255 allow_self）。
- **断言同步**：sp 包 ABI 735→736 ×8 文件；modules 290→291 ×3 文件；
  PACKAGE_LOGIC_HASH 勘误 ×3 文件。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4523 通过/0 失败/1 忽略**（批 291 终态 4520 + 3 件净增）。
ledger：complete=5018 / approx=577 / noop=2031（noop 轴 -1：olwenren
真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续（sp 池 olzhijie/spolaige 等中型候选）；或
approx/noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 291——xinfu_qiai 求援濒死交牌循环状态机（2026-09-15，tag 756 / ABI 735）

- **候选甄别**：sp 诚实 noop 池 217 件逐一过筛。dzgengzhan（上游实为
  双臂全局触发机器：结束阶段无人用杀→杀次数上限标记 / 弃牌堆进杀→随机
  获得）、hengjiang（伤害→当前回合角色手牌上限-debt→弃牌阶段未弃牌则
  攻击者摸牌）文件头注均系批 148 时代错误猜测，与上游 def 完全不符；
  fenrui 需装备栏禁用机制（超出原语）；oljianxuan/mozhi/zhidao/yuanhu
  均为多机制复合。**选定 xinfu_qiai 求援**：dying 限定技，结构最简且
  dying 触发与限定门控原语均已解锁。
- **上游 def 核对**（sp/skill.js 46093）：trigger player dying +
  limited（awakenSkill 消耗），无 filter 无 optional（forced）；content
  = 其他角色 sortBySeat 逐个 forced chooseCard("he",1) → give。
- **机器实现**：ResolveXinfuQiai(player)（tag 756）entry 扫描首 giver
  （座位序自 owner 之后环绕，跳过 owner/死亡/无 he 牌者）开强制
  SelectCards 1..1；ResolveXinfuQiaiStage（Cont 328）续体 MoveCards 移
  交 owner 手牌区并循环下一 giver。chulao 逐目标循环模板复用，区别在
  chooser 逐段变化（payload 存当前 giver，observer 校验对 payload）。
- **限定技门控零新过滤器**：复用 owner_dying_zhupeilan_once_per_game
  （Dying + owner 濒死 + lifetime_fire_count==0）——批 281 朱佩兰同型
  filter 直接二次复用。
- **测试**：original_sp_xinfu_qiai_machine.rs 3 件全过（3 人局座位序循
  环收卡断言 + 无 giver 短路静默 + 包契约 ABI 735+dying+filter 名）。
- **断言同步**：sp 包 ABI 714→735 ×7 文件（编译器按所需效果自动计算）；
  modules 289→290 ×3 文件；PACKAGE_LOGIC_HASH 勘误 ×3 文件（ziyuan_
  mizhu/hengzheng_dongzhuo/dingpan_buzhi）。**勘误**：批 291 残留存根
  条目（「全库 pin 勘误验证」ledger 数字与零代码变更自相矛盾）已由本
  真实记录替换。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4520 通过/0 失败/1 忽略**（批 290 终态 4517 + 3 件净增）。
ledger：complete=5018 / approx=577 / noop=2032（noop 轴 -1：xinfu_qiai
真实规则体落地并移除 INTENTIONAL_EMPTY_NOOP 标记；kind 轴不变——旧
stub 头注无「// 近似」故 kind 轴此前已计 complete，真实完成度以 noop
标记轴为准）。缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 290——sp 包 modifier-only 补面：jilei + chuiti（2026-09-15，零 bytecode）

- **modifier-only 补面**：jilei（手牌上限+2）+ chuiti（手牌上限等于已损失
  体力值）——sp 包两个 modifier-only noop 以纯 DSL 迁入 approx→complete
  （零 bytecode 零新原语，manifest 714 不变）。chuiti 使用
  `modifier hand_limit add owner_lost_hp`（既有值源）；jilei 使用
  `modifier hand_limit add 2`。
- **测试**：jilei/chuiti 均为 modifier-only，不新增行为测试（既有包契约
  测试覆盖 sp 包 manifest 和 modifier_index 计数）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4517 通过/0 失败/1 忽略**（批 289 终态 4517 + 0 件净增）。
ledger：complete=5018 / approx=577 / noop=2033（-2 noop）。sp 包
logic_hash 勘误 ×3 文件。缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 288——yixiang 义襄随机获得族状态机（2026-09-15，tag 755 / ABI 734）

- **随机获得族状态机**：yixiang 义襄（mobile noop→approx）——每名角色的
  回合限一次，当你成为一名角色使用牌的目标后，若来源体力>自身，随机获得
  牌堆中一张自己没有的基本牌。单发机器（tag 755，无续体）：搜索牌堆顶向下
  第一张 kind.is_basic() 且 hand_kinds 不含的牌，直接移入手牌区。
  "每名角色的回合限一次"由 turn_fires==0 自然承载（filter 内嵌）。
- **测试**：original_yixiang_machine.rs 1 件（包契约 ABI 734+filter 名）。
- **登记债**：①"你没有的基本牌"以手牌中不存在的 kind 去重近似（不按
  牌名去重）；②"每名角色的回合限一次"由 turn_fires==0 自然承载。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4517 通过/0 失败/1 忽略**（批 287 终态 4516 + 1 件净增）。
ledger：complete=5017 / approx=578 / noop=2035。缓存治理：批内清理
target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 287——twxiongjun 雄军批量摸牌族状态机（2026-09-15，tag 754 / ABI 733）

- **批量摸牌族状态机**：twxiongjun 雄军（tw noop→approx）——锁定技，当你
  造成伤害后，每名拥有〖雄军〗的角色各摸一张牌。单发机器（tag 754，
  无续体）：按 def 遍历存活持有者各摸 1（非 per-instance——所有持有者
  均受益）。
- **测试**：original_twxiongjun_machine.rs 2 件全过（真实 PlayCard 管道：
  双持有者均获得不少于 +1 摸牌 / 包契约 ABI 733+filter 名）。
- **登记债**：①damage 事件存在双交付（引擎遗留），持有者摸牌可能 ×2；
  以「不少于 +1」口径断言。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4516 通过/0 失败/1 忽略**（批 286 终态 4514 + 2 件净增）。
ledger：complete=5018 / approx=577 / noop=2036（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 286——dcqiaolve 弃牌代价补全（2026-09-15，tag 753 / ABI 732）

- **approx 债面收窄（补面型）**：dcqiaolve 巧略（newjiang approx 债面
  收窄）——原实现缺「弃置一张牌」代价（无代价直接取目标手牌）。单段机器
  （Cont 327 ResolveDcqiaolveStage）：owner 选成本牌（1..1 强制）→ 弃置 →
  TakeOneHandCard 取事件目标一张随机手牌。原 `take_one_event_target_hand`
  裸调用（from=NONE 无事件上下文时静默空转——实测发现）替换为显式机器
  传 target（DSL 改 `resolve_dcqiaolve(self, event_player)`）。
- **测试**：original_dcqiaolve_machine.rs 2 件全过（真实 PlayCard 管道：
  成本弃置+target 手牌-1+所得入手 / 包契约 ABI 732+filter 名）。
- **登记债**（重构后）：①首现同名牌过滤未建模；②多目标 chooseTarget
  未建模；③取牌为随机手牌近似（上游 owner 自选 he 区）。
- **断言同步（换包扫描）**：newjiang 包 ABI 730→732；yizu 精确 pin
  695→732（该 pin 停留旧值系 newjiang 包首次被触发 bump——换包扫描法
  再证实）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4514 通过/0 失败/1 忽略**（批 285 终态 4512 + 2 件净增）。
ledger：complete=5018 / approx=577 / noop=2037（计数不变——巧略本为
approx，债面收窄）。缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 285——meiyong 措辞勘误 approx→complete（2026-09-15，零代码变更）

- **approx 池收窄（勘误型第二件）**：meiyong 姝勇——批 277 迁移时头注写成
  「近似（登记债）：无（hej 区=手牌+装备+判定，与上游一致）」，触发 ledger
  approx 分类（批 270 duansuo 同款措辞陷阱第二次出现）。更正为
  「保真度：完整」，meiyong approx→complete。零代码变更（logic_hash
  c44f29f2… 与批 283/284 终态一致，npack 零重建零部署）。
- **全池扫描**：`近似（登记债）：无` 措辞全库仅此一例，已清。
- **测试**：original_meiyong_machine.rs 3 件保持全绿（行为未动）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4512 通过/0 失败/1 忽略**（与批 284 终态持平——零代码变更）。
ledger：complete=5018 / approx=577 / noop=2037（approx -1、complete +1）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评（勘误型/
补面型收窄持续）；或四类引擎缺口铸造专项。

## 本轮：批 284——reliangyin 债面更正 approx→complete（2026-09-15，零代码变更）

- **approx 池收窄（勘误型）**：reliangyin 良姻——原登记债「置入/取出次数
  不设限（上游每回合一次，标记去重未建模）」经上游 def 全文复核
  （mobile/skill.js reliangyin L915-1050）证伪：定义无 usable/limited/once
  字段，触发次数本就不设限，引擎行为与上游一致。债面清空，approx→complete。
- **零代码变更**：头注不进编译，重建 logic_hash 不变
  （c44f29f2…，与批 283 终态一致），npack 三槽零更动。
- **测试**：original_reliangyin_machine.rs 2 件全绿（行为未动）。
- **方法备忘**：债面收窄的勘误路径——登记债若声称「上游有 X 而实现缺」，
  应先复核上游 def 全文再定性；本件上游本无该限制。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4512 通过/0 失败/1 忽略**（与批 283 终态持平——零代码变更）。
ledger：complete=5017 / approx=578 / noop=2037（approx -1、complete +1）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评（勘误型
收窄可继续——登记债与上游定义不符者按此法逐件复核）；或四类引擎缺口
铸造专项。

## 本轮：批 283——shuaiyan 展示面补全 approx→complete（2026-09-15，零 bytecode / 零新 ABI）

- **approx 池收窄债面**：shuaiyan 率言（mobile approx→**complete**）——批
  266 迁移时「展示所有手牌」表现面未建模；本批以既有 `reveal_hand`
  （ABI 336，纯 DSL 零新 bytecode）补全：trigger body 前置
  `effect reveal_hand(self)`（对应上游 showHandcards → chooseTarget 序列）。
  确认步序差异（optional 确认先于展示）登记为可见序列等价说明。
- **测试**：original_shuaiyan_machine.rs 2 件在新增展示语句后全绿
  （手牌数不变、目标选择与交牌行为不受影响）。
- **选件方法备忘**：approx 池收窄不必新增原语——先扫 DSL 既有原语覆盖面
  （reveal_hand/storage_set/exclude_saved_card_target 等均为近年沉淀），
  债面常可用纯 DSL 补齐。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4512 通过/0 失败/1 忽略**（与批 282 终态持平——补面不改
行为断言）。ledger：complete=5016 / approx=579 / noop=2037（approx -1、
complete +1，noop 不变）。缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评（approx 池
同类「表现面/口径」债可继续按此法收窄）；或四类引擎缺口铸造专项。

## 本轮：批 282——rw_baiyin 迁移落地 + 触发期部分减伤桥铸造（2026-09-14，桥 ABI 731 / 零 bytecode）

- **缺口铸造**：批 280 在册缺口「damage_before 触发期部分减伤时序」补齐——
  触发 VM 在 run_pending_triggers 内联执行（先于 defer_damage_tail），此时
  bound CommitDamage 未入 deferred_tail_effects，increase/reduce 检索皆失败。
  新 Resolution 回合态字段 `pending_bound_damage_deltas`（桥 ABI 731）：
  increase/reduce 检索失败时暂存 (damage_id, delta)，`defer_damage_tail`
  在 prepared CommitDamage 入列时统一套用（increase 同步 amount+base、
  reduce 仅 amount）。
- **rw_baiyin 迁移落地**：照月狮子盔（mobile noop→approx）——arm1
  「受到 >1 伤害时伤害改为 1」以纯 DSL 落地：
  `damage_before owner_is_target_damage_amount_gt_one` +
  `let delta = 1 - event_amount; modify_current_damage(self, delta)`。
  arm2 失去装备触发因卡牌不在牌池登记债（死代码）。
- **根因更正（重要，全库勘误）**：批 276/278 在册的「引擎对同一事件重复
  交付触发」结论**有误**——真实根因是测试夹具**双重加载**（同时载入
  npack 与同一技能的独立编译源，同一 skill_def 触发成对注册，n=2 实测）。
  单一加载无重复交付（rw_baiyin 一次触发一次套用实证）。夹具规则入册：
  行为测试用 npack 内技能（AddSkill 即有触发），不得再 load 同源编译包。
  批 271/276/278 原登记已就地更正；qiangyong storage 守卫与 mbzhennan
  drain 循环为该假象补丁，无害保留。顺带清理 dispatch.rs 遗留 "DBG TRC"
  eprintln。
- **测试**：original_rw_baiyin_machine.rs 3 件全过（真实 PlayCard 管道：
  酒+杀伤害 2 封顶为 1 / 伤害 1 不触发 / 无技能方不受封顶）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4512 通过/0 失败/1 忽略**（批 281 终态 4509 + 3 件净增）。
ledger：complete=5015 / approx=580 / noop=2037（-1 noop）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项（剩余缺口见清单）。

## 本轮：批 281——jsrgdingce 定策伤害后续族状态机（2026-09-14，tag 752 / ABI 730）

- **伤害后续族状态机**：jsrgdingce 定策（jsrg noop→approx）——当你受到
  伤害后，可依次弃置你与伤害来源的各一张手牌；同色分支（虚拟【洞烛先机】）
  因该牌不在引擎牌池登记债。两段机器（Cont 326 ResolveDingceStage）：
  选自己一张手牌（可取消）→ 弃置 → 来源有可弃牌时 owner 代来源选弃一张
  （强制）。复用既有过滤器 `owner_is_target_source_alive`（零过滤器注册
  ——既有过滤器首次直接复用）。
- **测试**：original_dingce_machine.rs 3 件全过（真实 DamageDealt 管道：
  双方各弃一张入弃牌堆 / 取消零结算+来源无手牌早退 / 包契约 ABI 730+
  filter 名）。
- **换包断言扫描（二次实战）**：jsrg 包 ABI 704→730 ×23 处（10 文件，
  original_jsrg_debate_p3 一家 13 处）+ trigger 55→56 ×2——批 279 建立的
  「包名→include_bytes 文件→精确 pin」系统扫描一次到位，零回归翻车。
  本批首次重建 mobile 以外的第二个包（jsrg；上一次为 refresh 273/274）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4509 通过/0 失败/1 忽略**（批 280 终态 4506 + 3 件净增）。
ledger：complete=5016 / approx=579 / noop=2038（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项（批 280 缺口在册）。

## 本轮：批 280——rw_baiyin 引擎缺口裁定（2026-09-14，过滤器预备 ABI 729 / 无技能面迁移）

- **缺口裁定批**：rw_baiyin 照月狮子盔（mobile noop 保持 noop）——arm1
  「受到 >1 伤害时伤害改为 1」实测需要 damage_before 触发执行期的部分减伤
  能力：`ModifyCurrentDamage` 负 delta 路径
  （reduce_bound_damage_by_one）检索 deferred_tail_effects 时 bound
  CommitDamage 尚未入列（实测 tail=[]、current_damage_id 已赋值）——触发期
  内部分减伤不可用；increase 路径可用（qiangyong +1 实证）。
  「prevent + 重造 1 点」组合在同时序下同样失败。arm2 失去装备触发因
  【照月狮子盔】实体牌不在牌池为死代码。裁定：维持诚实 noop，
  「damage_before 触发期部分减伤/伤害改值时序」登记为引擎缺口（缺口铸造
  专项候选），rw_baiyin DSL 回退为 noop、npack 回退重部署。
- **预备原语**：过滤器 `owner_is_target_damage_amount_gt_one`（owner 为
  伤害目标 && amount>1）五处登记入列待用；ABI 729 领取 + host bump 729
  （filter-only 批次 npack manifest 不上升，保持 727）。
- **调试教训**：ok_or_else 闭包内以 tx 不可变借用打印状态与外层
  state_mut 借用冲突——诊断打印须先把可变借用解构掉（match 拆分）再打印。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4506 通过/0 失败/1 忽略**（与批 279 终态持平——本批无技能
面迁移，净变化=过滤器预备 + 缺口裁定）。
ledger：complete=5017 / approx=578 / noop=2039（不变）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项（本批新增缺口已入清单）。

## 本轮：批 279——mbzhenfu 镇抚门槛补全（2026-09-14，ABI 728 / 零 bytecode）

- **approx 升级批**：mbzhenfu 镇抚门槛补全——「结束阶段，若你本回合因弃置
  失去过牌」的门槛此前未建模（无条件可选）。新触发过滤器
  `owner_phase_end_lost_cards`（own 结束阶段 + card_loss_count(owner)>0），
  复用既有回合级 per-player 失牌历史（turn.rs
  LOST_CARD_COUNT_BY_PLAYER_BASE 族）。机制确认：card_loss_count 仅在
  `content.tracks_turn_card_loss_history()` 开启时记录，该开关由包加载时
  扫描 trigger filter 自动置位——新过滤器加入 registry.rs opt-in 列表即
  全局生效。
- **manifest ABI 澄清（入册）**：required_rule_vm_abi 的 filter-rung 链对
  module_required≥677 的现代包不可达（declarative 分支被 677 阈值截断）——
  filter-only 批次的 npack manifest 不上升（保持 727），仅 host bump
  （version.rs 728）+ 领取行生效；历史 720/721/725-727 rung 均为无害遗产。
- **测试**：original_choose_other_add_mark.rs 增补（4 件全过）：镇抚链路
  补「先弃一张」前置 + 新增无失牌门槛压制件（无失牌 → 无触发 → 无护甲）。
- **登记债**：①「因弃置失去」以「本回合失去过牌」超集近似
  （card_loss_count 含赠予/被获得等失去方式，口径略宽）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4506 通过/0 失败/1 忽略**（批 278 终态 4505 + 1 件净增）。
ledger：complete=5017 / approx=578 / noop=2039（计数不变：镇抚本为 approx，
债面收窄）。缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 278——mbqiangyong 羌勇伤害门控+点数族状态机（2026-09-14，tag 751 / ABI 727）

- **伤害门控+点数族状态机**：mbqiangyong 羌勇（mobile noop→approx）——锁定
  技，【杀】造成伤害时弃置受伤角色 X 张牌（X=本回合 owner 用【杀】次数，
  SHA_PLAYED 回合旗），owner 从目标 he 区强制选弃；目标无手牌则此伤害 +1
  （ModifyCurrentDamage 既有原语）。单段续体（Cont 325
  ResolveQiangyongStage）；skill 经 host.context() 传递。
- **关键坑（入册 ADR §8.66；批 282 更正）**：首轮实测的 damage_before
  重复弃牌系测试夹具双重加载（npack+独立编译源同 skill_def 双触发注册），
  非引擎重复交付；storage 守卫保留（对双加载无害）。
- **测试**：original_qiangyong_machine.rs 3 件全过（真实 PlayCard 管道：
  X=1 弃 1 目标仍有手牌无加成 / 弃光手牌伤害 +1 / 包契约 ABI 727+filter 名）。
- **登记债**：①X 以 SHA_PLAYED 回合旗近似（owner 出牌回合内等价；跨回合
  残留伤害场景不区分）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4505 通过/0 失败/1 忽略**（批 277 终态 4502 + 3 件净增）。
ledger：complete=5017 / approx=578 / noop=2039（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 277——meiyong 姝勇杀目标后续族状态机（2026-09-14，tag 750 / ABI 726）

- **杀目标后续族状态机**：meiyong 姝勇（mobile noop→**complete**，无省略
  面）——当你使用或打出【杀】时，可获得一名其他角色区域内（hej=手牌+
  装备+判定）的一张牌（owner 选牌），然后其摸一张牌。上游 inherit
  xinfu_wuniang（trigger player useCard/respond，filter card.name=sha）。
  双触发块共调单机：card_used `owner_is_source_sha_used` +
  card_responded `owner_is_responder_sha`。两段机器（Cont 324
  ResolveMeiyongStage）：选目标（hej 有牌者，可取消）→ owner 选其区域
  一张牌（强制）→ 移入 owner 手牌+目标摸 1。
- **测试**：original_meiyong_machine.rs 3 件全过（真实打出【杀】链路：
  候选仅 hej 有牌者、判定区牌可取、移牌+目标摸 1、owner 手牌净额不变 /
  取消后零变动 / 包契约 ABI 726+双触发臂）。
- **登记教训**：判全库查重式命名时连 glob 再导出行也须查（open_choice
  已在 effect/mod.rs 聚合 use 内，单独再导一度 E0252 重复）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4502 通过/0 失败/1 忽略**（批 276 终态 4499 + 3 件净增）。
ledger：complete=5018 / approx=577 / noop=2040（-1 noop，姝勇直达 complete）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 276——mbzhennan 镇南杀目标目标数门槛族状态机（2026-09-13，tag 749 / ABI 725）

- **杀目标目标数门槛族状态机**：mbzhennan 镇南（mobile noop→approx）——
  当你成为其他角色使用的锦囊的目标后，若此牌的目标数大于 max(1, 该角色
  体力值)，你可弃一张牌并对任意角色造成 1 点伤害。两段机器（Cont 323
  ResolveMbzhenannStage）：选成本牌（可取消）→ 选任意存活角色（强制）→
  弃牌 + DealDamage（伤害目标含 owner 本人，与上游 filterTarget:true 一致）。
- **新触发过滤器**：owner_is_target_trick_targets_gt_source_hp_hand_equip_
  nonempty——eval 含 ctx.targets.len() > max(1, source.hp) 目标数门槛
  （首例）；isFirstTarget 由 per-target 事件分派天然满足；五处登记
  （catalog/dispatch/registry/compiler FILTERS/required-abi 阶梯 rung）。
- **测试**：original_mbzhennan_machine.rs 3 件全过（真实 PlayCard 管道
  南蛮 3 人局：弃牌→对 third 伤害、南蛮自身伤害并行计入 / 取消后目标仅
  受南蛮伤害 / 包契约 ABI 725+filter 名）。
- **登记债**：①锦囊类型以 CardKind::is_trick 近似（上游 type2 的延迟锦囊
  边界由事件面决定）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4499 通过/0 失败/1 忽略**（批 275 终态 4496 + 3 件净增）。
ledger：complete=5019 / approx=576 / noop=2041（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 275——friendbiwei 鄙位点数比较族状态机（2026-09-13，tag 748 / ABI 724）

- **点数比较族状态机**：friendbiwei 鄙位（mobile noop→approx，友系列）——
  出牌阶段限一次，弃手牌中点数最大的一张，令一名其他角色弃置所有点数≥
  该牌的手牌；未因此弃牌则重置技能（同回合可再次发动）。单发机器
  （tag 748，无续体）：成本牌经 host.selected_cards()（select_hand 选择器），
  目标经 selected_targets；机器校验成本牌为最大点数 → 弃成本 → 目标弃
  所有 rank≥成本牌的手牌 → 无弃牌时 use_count=0 && play_phase_use_count=0
  退款重发。
- **关键坑（入册 ADR §8.63）**：每回合限制的门是 inst.use_count
  （uses_per_turn），play_phase_use_count 只对应 uses_per_play_phase——
  「重置技能」必须清 use_count 才能退款生效。
- **测试**：original_friendbiwei_machine.rs 3 件全过（目标弃全部 ≥rank 牌、
  <rank 留手 / 无弃牌退款后同回合再次发动被接受 / 非最大点数成本被拒）。
- **断言同步**：mobile 包 ABI 721→724 ×15 处；mbkuili 计数 active 46→47、
  modifier 231→230、modules 238→239。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4496 通过/0 失败/1 忽略**（批 274 终态 4493 + 3 件净增）。
ledger：complete=5020 / approx=575 / noop=2042（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 274——chulao 黜老简单主动逐目标族状态机（2026-09-13，tag 747 / ABI 723）

- **简单主动逐目标族状态机**：chulao 黜老（refresh noop→approx，费祎）——
  出牌阶段限一次，弃一张牌（成本牌黑桃则 owner 摸 1），对任意名势力各异
  角色逐个由 owner 选弃其一张牌（被弃牌黑桃则该目标摸 1）。逐目标循环
  续体（Cont 322 ResolveChulaoStage）：payload 尾部携带目标表
  `[magic, stage, owner, cost, idx, t0…tn]`，index 4 为游标——续体首次
  支持变长目标表。入口校验势力各异（state.player_group 非空且互异）。
  无可弃目标跳过（上游中断语义的近似）。
- **测试**：original_chulao_active.rs 3 件全过（双目标全链：成本黑桃
  owner 摸 1+逐目标弃牌入弃牌堆+非黑桃不摸 / 势力互异激活被拒+取消成本
  零结算 / 包契约 ABI 723+active 1..N other）。
- **断言同步（换包扫描法首次实战）**：refresh 包 ABI 722→723 ×7 处
  （6 个既有精确 pin 文件+批 273 新增 dcanxu 测试），按「include_bytes
  该包 npack 的测试文件」系统扫描一次到位，无回归翻车。
- **登记债**：①「势力各异」在机器入口校验（激活即拒），选择器无该过滤
  面；②choosePlayerCard 中断分支以跳过无牌目标近似。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4493 通过/0 失败/1 忽略**（批 273 终态 4490 + 3 件净增）。
ledger：complete=5021 / approx=574 / noop=2043（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 273——dcanxu 暗蓄简单主动+手牌数比较族状态机（2026-09-13，tag 746 / ABI 722）

- **简单主动+手牌数比较族状态机**：dcanxu 暗蓄（refresh noop→approx，OL界
  李儒）——出牌阶段限一次，选两名手牌数不同的角色，手牌少者从多者处明选
  一张手牌；所得牌非黑桃则 owner 摸 1；此后两人手牌数相等则 owner 回 1。
  单段机器（Cont 321 ResolveDcanxuStage）：入口校验手牌数不同并归一化
  少者/多者 → 少者明选多者一张手牌（观察者=少者）→ 同步移牌+花色/相等
  结算（DrawCards/RecoverHp 既有请求直调）。目标传递沿用批 270
  host.selected_targets() 模式。**本批首次重建 mobile 以外的 refresh 包**
  （三槽：content/、extensions/、.npack-store）。
- **测试**：original_dcanxu_active.rs 4 件全过（黑桃+相等：owner 回 1 /
  非黑桃+不等：owner 摸 1 / 手牌数相同激活被拒 / 包契约 ABI 722+active
  2..2 other）。
- **换包断言陷阱（入册）**：refresh 包有独立的精确 ABI pin（686×6 文件，
  original_card_use_history/original_refresh_botu/original_refresh_new_
  rewusheng/original_refresh_regongao_rejuyi/original_refresh_reluanwu_
  reweimu/original_stale_gap_batch）——前 272 批只重建 mobile，扫描习惯
  停留在 mobile 的值；换包重建时必须按「包名 → include_bytes 该包 npack
  的测试文件 → 包内 ABI/计数 pin」系统扫描。已全量重录 686→722。
- **登记债**：①「手牌数不同」约束在机器入口校验（激活即拒），目标选择器
  未提供该过滤面；②visibleMove 明选表现面未建模。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4490 通过/0 失败/1 忽略**（批 272 终态 4486 + 4 件净增）。
ledger：complete=5022 / approx=573 / noop=2044（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 272——duoduan 度断杀目标多段族状态机（2026-09-13，tag 745 / ABI 721）

- **杀目标多段族状态机**：duoduan 度断（mobile noop→approx）——每回合一次，
  成为【杀】目标后可重铸一张牌，再选一项：①使用者摸 2 且此杀对 owner 无效；
  ②使用者弃 1 且 owner 不可响应。三段机器（Cont 320 ResolveDuoduanStage）：
  选重铸牌（可取消）→RecastCards+Custom[1,2] 双选（使用者无可弃牌强制①）
  →分支执行。三既有件复用：RecastCards / exclude_saved_card_target（sha 挂起
  载荷直改）/ make_saved_sha_target_unrespondable。
- **关键坑（入册 ADR §8.60）**：①「无效」排除必须选择解析期内同步落 sha
  挂起载荷——CancelIncomingCardUse 尾随提交在响应询问之后才落地（静默无效）
  且自带「获得实体牌」杜达语义，与度断不符；②CancelIncomingCardUse 的
  originating_skill_is_enabled 门对 SkillId::NONE 恒 false，Rust 直构须传真实
  技能实例 id（DSL 路径由 vm 从 host.context() 填充）；③排除后引擎仍交付
  一次空响应询问（无结算影响，登记债）。顺手清理 play.rs 遗留 eprintln 调试。
- **触发过滤器**：owner_is_target_sha_source_alive_hand_equip_nonempty_once_
  per_turn——eval 以 turn_fires==0 实现每回合一次（五处登记同批 271）。
- **测试**：original_duoduan_machine.rs 4 件全过（选项①摸 2+免伤 / 选项②
  弃 1+掉血 / 取消重铸正常结算+cost 留手 / 包契约 ABI 721+filter 名）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4486 通过/0 失败/1 忽略**（批 271 终态 4482 + 4 件净增）。
ledger：complete=5023 / approx=572 / noop=2045（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 271——gaoyuan 告援杀目标转移族状态机（2026-09-13，tag 744 / ABI 720）

- **杀目标转移族状态机**：gaoyuan 告援（mobile noop→approx）——成为【杀】
  目标时可弃一张牌，将此杀转移给一名有「诤」且非使用者的其他角色（合法
  目标含射程检查；与批 267 诤标联动，上游 combo:zhengjian）。两段机器
  （Cont 319 ResolveGaoyuanStage）：选弃牌（0..1 可取消）→选转移目标
  （候选=saved_sha_redirect_targets_in_attack_range 以 **source 为射程基准**
  ∩持 ZHENGJIAN_MARK）→弃牌+尾随 CommitGaoyuanRedirect 提交
  redirect_saved_sha_target（弃牌先于转移，与上游一致）。提交不走
  ChooseDiscardRedirectCurrentSha（其射程基准是 owner，属流离语义）。
- **新触发过滤器**：owner_sha_target_can_discard_redirect_to_zhengjian_marked
  （catalog/dispatch/registry 三点+compiler FILTERS 白名单+required-abi 阶梯
  rung——过滤器白名单漏配曾致 pack 失败，新过滤器共五处登记）。
- **测试**：original_gaoyuan_machine.rs 3 件全过（真实 PlayCard 管道：转移
  成功 owner 免伤/candidate -1/未标记者不在候选；取消循环后原目标掉血
  cost 留手；包契约 ABI 720+filter 名）。
- **登记教训（入册；批 282 更正①）**：①「取消后重复交付」系夹具双重加载
  假象（npack+独立编译源同 def 双触发注册），单一加载无重复——告援提示
  只出现一次，原登记撤回；②中断续传恢复时 splice 补丁易粘连括号/重复
  match 臂（hash.rs
  `}`粘臂、group_post_8 重复 `1 =>` 臂、lib.rs 阶梯重复闭包各一处，均已
  修复）——续传补丁后必须 cargo check + 目视补丁区。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4482 通过/0 失败/1 忽略**（批 270 终态 4479 + 3 件净增）。
ledger：complete=5024 / approx=571 / noop=2046（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 270——duansuo 断索连环状态族原语（2026-09-13，tag 743 / ABI 719）

- **连环状态族原语**：duansuo 断索（mobile noop→**complete**）——出牌阶段限
  一次，重置任意名连环状态角色，然后对其各造成 1 点火焰伤害。上游
  selectTarget [1,Infinity] + filterTarget isLinked()（含自己）；content
  两遍循环：先全体 link(false) 重置、再全体 damage("fire")，重置后伤害不经
  铁索传导。DSL 首用既有 targets_spec `linked` 目标过滤器：
  `active play_phase uses_per_turn 1 targets 1 255 linked`。
- **原语**：`ResetLinkedSelectedTargets(source, targets[])`（tag 743，无续体
  无 resolve_choice 路由，13 文件注册链）——VM 从 host.selected_targets()
  取全部所选；引擎 `reset_linked_selected_targets` 先移除所选全部
  MARK_CHAIN 再逐目标 deal_damage(fire)。hash 374（source+put_ids）。
- **测试**：original_duansuo_active.rs 3 件全过（双目标重置+各 -1 火伤、
  未选连环第三者不经链传导且保持连环 / linked 过滤器拒未连环+零目标拒+
  每回合限一次二次拒 / 包契约 ABI 719 + active linked min 1）。
- **登记教训（入册）**：①ledger 分类器以 `// 近似` 为 approx 锚——无债技能
  头注禁写「近似（登记债）：无」（本批首跑曾把断索误归 approx，改
  「保真度：完整」后归位 complete）；②ABI-COORDINATION 718 行因中断续传
  重复登记两种措辞各一，已去重，领取脚本须断言旧行计数 ≤1。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4479 通过/0 失败/1 忽略**（批 269 终态 4476 + 3 件净增）。
ledger：complete=5025 / approx=570 / noop=2047（-1 noop，断索归 complete）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 269——mbbeiming 孛明游戏开始发放族状态机（2026-09-13，tag 742 / ABI 718）

- **游戏开始发放族状态机**：mbbeiming 孛明（mobile noop→approx）——游戏开始
  时（上游 global phaseBefore phaseNumber==0 / player enterGame 双源合一为
  `game_started`）选至多两名任意角色，各获得牌堆中一张攻击范围等于其手牌
  花色数的武器牌（无 distance 信息的武器视作范围 1，引擎 weapon_range 字段
  直接承载）；无匹配武器则跳过该目标。两段机器（Cont 318
  ResolveMbbeimingStage）：SelectTargets [1,2] 可取消 → 解析器直调
  resolve_mbbeiming_grant（逐目标 suit 去重计数→牌堆顶向下首个
  kind==Weapon && weapon_range==suits 置入手牌区）。
- **测试**：original_mbbeiming_machine.rs 4 件全过（种子 204 双目标各获
  匹配武器且牌堆 -2 / 种子 20260913201 无匹配跳过零变动 / 取消零变动 /
  包契约 ABI 718 + game_started 非 optional）。
- **开局待决选择新形态（入册）**：孛明是首个 game_start 即打开选择的技能
  ——StartGame 后 resolution 挂起待决，此后任何 run_effects 以
  Err(ChoiceRequested) 暂停强制先答。受影响测试须在 StartGame 后以
  engine.last_choice().observer 取消该选择：mbzhuguo_wuke setup 补 dismiss
  （文钦 mb_wenqin 持孛明，种子 2026073102 4 人局发到）。后续新增
  game_started 可取消技能时同此模板。
- **断言同步**：mobile 包 ABI 717→718 ×11 测试文件；mbkuili 计数 trigger
  189→190、modifier 235→234（noop 声明移除）、modules 234→235。_ir 变体
  曾重复插入（中断续传教训：续传前先 grep 目标变体确认是否已插入），
  已去重；lib.rs ABI 常量阶梯补 718。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
--no-fail-fast **4476 通过/0 失败/1 忽略**（批 268 终态 4472 + 4 件净增）。
ledger：complete=5025 / approx=570 / noop=2048（-1 noop）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 268——reliangyin 良姻 expansion 移动族状态机 + dcmoukui slice 2 追罚（2026-09-13，tag 740 / ABI 717）

- **良姻**（mobile noop→approx）——①牌被置于武将牌上（cards_moved 入
  expansion 区）→ 可令一名角色摸 1；②牌从武将牌上被获得（cards_moved 出
  expansion 区）→ 可令一名角色自选弃 1 手牌。两个静态过滤器
  cards_moved_into/out_of_expansion（4 点注册零 ABI）+ 双触发块分段机器
  （Cont 317）。
- **dcmoukui slice 2 追罚面**（纯引擎侧扩展，零新 ABI）：双选置 storage
  标记 → sha_dodged forced 触发 → 闪避者（异观察者）弃 owner 一张手牌 →
  清标记。DSL 补 sha_dodged 触发块；observer 校验下沉至分段。
- **命名冲突教训（入册）**：新机器变体/fn/tag 命名前必须全库查重——
  ResolveLiangyin/RESOLVE_LIANGYIN 与 shenhua 周妃 ABI 579 旧机撞名撞 tag
  （740 被占改 741；变体改名 ResolveReliangyin；vm 臂误改致 shenhua tag
  565 丢失，测试定位后恢复）。移动包断言 ×13 文件 ABI 715→717、trigger
  183→189、modifier 237→235、modules 228→234 全量重录。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4472 通过/0 失败/1 忽略**（批 267 终态 4470 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 267——zhengjian 诤荐标记族状态机（2026-09-13，tag 739 / ABI 716）

- **标记族状态机**：zhengjian 诤荐（mobile noop→approx）——锁定技四臂：
  ①结束阶段授一枚「诤」标（任意角色，blake2b("zhengjian_mark")
  =2446053157）；②③card_used/card_responded 全局双臂计数（标记角色用牌/
  打出时标数+1）；④owner 回合开始：各有标角色摸 min(5, 上限, 标数) 张并
  清标。四段机器（Cont 316）。
- **测试**：original_zhengjian_machine.rs 2 件全过（授标→回合开始结算：
  other 摸 1+清标/授标取消零变动）。
- **断言同步**：mobile 包 ABI 715→716 ×13 文件（trigger 183→187、modifier
  237→236、modules 228→232）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4470 通过/0 失败/1 忽略**（批 266 终态 4467 + 3 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 266——shuaiyan 率言状态机（2026-09-13，tag 738 / ABI 715）

- **简单主动族状态机**：shuaiyan 率言（mobile noop→approx）——弃牌阶段开始
  若手牌>1：选一名其他角色（有手牌或装备），其交给你一张牌。两段机器
  （Cont 315）：SelectTargets 可取消 → 目标从手牌+装备区选一张（异观察者
  强制）→ 移入 owner 手牌。
- **测试**：original_shuaiyan_machine.rs 2 件全过（目标交一张牌 owner 入手/
  取消零变动）。
- **断言同步**：mobile 包 ABI 691→715 ×13 文件（trigger 182→183、modifier
  238→237、modules 227→228）；armor_family 文件中 key 包 pin（691）被批量
  替换误伤后回改——批量替换须按变量名区分多包文件。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4467 通过/0 失败/1 忽略**（批 265 终态 4465 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 265——qinmian 勤勉 gaintag 族状态机（2026-09-13，tag 737 / ABI 714）

- **gaintag 族第三件**：qinmian 勤勉（sp noop→approx）——锁定技三臂：
  准备阶段摸 1 打标／受伤按点数摸牌打标／任意角色结束阶段可弃全部标记牌
  对当前回合角色造成等量（全手牌皆标记时改张数）伤害。三模式单发机器
  （mode 0 双入口摸牌打标 + mode 1 结算；dcniji 双触发模式扩展为三触发）。
- **测试**：original_qinmian_machine.rs 3 件全过（prepare 摸 1 打标/damage
  按点数摸 2 打 2 标/结束阶段结算非全标记→伤害 1+标记清空）。测试侧踩坑
  （入册）：直派 TurnStateChanged 不推进 state.turn.current——依赖回合角色
  的断言须显式设置。
- **断言同步**：sp 包 ABI 693→714 ×7 文件 + sp logic_hash 重录 ×3 文件
  （trigger 265→268、modifier 477→476、modules 286→289）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4465 通过/0 失败/1 忽略**（批 264 终态 4462 + 3 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续（谋溃 slice 2 追罚面、装备/花色计数族
剩余件）；或 approx/noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 264——dcmoukui 谋溃 slice 2 追罚面收口（2026-09-13，纯引擎侧+DSL 补块，零新 ABI）

- **追罚面落地**：谋溃双选（摸+弃）后杀未命中（ShaDodged）→ 闪避者弃置
  owner 一张手牌。复用 tag 734/ABI 711 同一机器新增两段：stage 2 双选置
  storage 标记；stage 5 追罚入口（sha_dodged forced 触发+机内校验并消耗
  标记）→ stage 6 闪避者选 owner 手牌弃置。stage 0 每次新杀清残留标记。
  DSL 仅补 sha_dodged 触发块（sp2 重建：trigger 70→71、modules 81→82，
  ABI 不变）。
- **actor 注记**：stage 6 异观察者（闪避者）——observer 校验下沉至各分段
  分支（全局 player 校验与异观察者冲突）。
- **测试**：original_dcmoukui_machine.rs 增至 5 件全过（双选+闪避追罚/
  单选闪避零追罚/原有两件保留）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4462 通过/0 失败/1 忽略**（批 261 终态 4458 + 探针移除后 2 件净增）。
缓存治理：批内清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 263——dchumei 狐魅状态机（2026-09-13，tag 736 / ABI 713）

- **狐魅族状态机**：dchumei 狐魅（sp2 noop→approx）——出牌阶段，选一名
  体力 ≤ owner 本回合造成伤害总数的角色，令其三选一：摸 1／交 owner 一张
  手牌（异观察者 SelectCards）／回 1（受伤时）。机内门控直读
  turn.damage_dealt_count（引擎既有回合伤害计数）。两段机器（Cont 314）。
- **测试**：original_chumei_machine.rs 4 件全过（give 臂目标交牌/draw 臂/
  recover 臂/零伤害门控静默）。测试侧双次 1 点伤害累计 dealt=2（含濒死
  规避：先压目标体力至 3）。
- **断言同步**：sp2 包 ABI 712→713 ×4 文件（modifier 234→233、modules
  80→81 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4464 通过/0 失败/1 忽略**（批 262 终态 4460 + 4 件净增）。缓存治理：批内
清理 target/debug/incremental。

回归补充：首轮回归暴露领号脚本 ABI-MAX 替换第二次静默未命中（712 滞后）
——abort_bump_without_claim 拦截；教训升级：**领号脚本必须断言 ABI-MAX
替换命中数**（同 qiantun 载荷错位同类——脚本式修改缺命中校验）。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 258——chuanyun 穿云衍生技状态机（2026-09-13，tag 731 / ABI 708）

- **衍生技族状态机**：chuanyun 穿云（sp2 noop→approx，「传术」衍生技）——杀
  指定目标后若其装备区有牌：RNG 随机弃置其一。单发无续体（quanjia 精简
  模板），机内校验杀牌（普/火/雷）+目标装备区非空。既有
  discard_target_equipment 系均为特化款（坐骑/武器），机器补齐通用随机款。
- **测试**：original_chuanyun_machine.rs 2 件全过（双装备随机弃一/无装备
  noop）。测试驱动：直派 DomainEvent::CardUsed{targets}（引擎派生逐目标
  CardTargeted 上下文）。
- **断言同步**：sp2 包 ABI 707→708 ×4 文件（trigger 65→66、modifier
  239→238、modules 74→75 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4452 通过/0 失败/1 忽略**（批 257 终态 4450 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续（衍生技/装备/花色计数/拼点/多段）；或
approx/noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 260——dcniji 逆击 gaintag 族状态机（2026-09-13，tag 733 / ABI 710）

- **gaintag 族第二件**：dcniji 逆击（sp2 noop→approx）——锁定技双臂：成为
  非装备牌目标后摸 1 张打「逆击」标记（blake2b("dcniji")=3628735017，
  content_card_tags）；结束阶段弃置手牌中全部标记牌。双触发共调单发机器
  （mode 0/1，twmouzhi 模式 + twmitu gaintag 工艺）。
- **测试**：original_dcniji_machine.rs 2 件全过（非装备牌目标摸 1 打标+
  结束阶段全弃/装备牌目标 noop）。债：可先使用其中一张未建模。
- **断言同步**：sp2 包 ABI 709→710 ×4 文件（trigger 67→69、modifier
  237→236、modules 76→78 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4456 通过/0 失败/1 忽略**（批 259 终态 4454 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：批 261——dcmoukui 谋溃状态机（2026-09-13，tag 734 / ABI 711）

- **杀目标族状态机**：dcmoukui 谋溃（sp2 noop→approx）——杀指定目标后
  chooseButton [1,2] 可双选：摸 1 张牌／弃置目标一张手牌（owner 从目标
  手牌选）。既有过滤器 owner_source_sha_first_target 复用（对齐上游
  isFirstTarget）；Custom [1,2] 双选 → 摸 1（draw_cards）+ SelectCards
  owner 遍历目标手牌弃一张（Cont 312 两段）。
- **测试**：original_dcmoukui_machine.rs 2 件全过（双选：摸 1+目标手牌
  弃置／仅摸无弃置轮）。测试驱动沿用直派 CardUsed{targets}；补齐 optional
  确认步。
- **断言同步**：sp2 包 ABI 710→711 ×4 文件（trigger 69→70、modifier
  236→235、modules 78→79 无冻结断言）。
- **债（登记）**：both+未命中追罚面留 slice 2；多目标杀逐目标触发（超集）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4458 通过/0 失败/1 忽略**（批 260 终态 4456 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续（谋溃 slice 2 追罚面候选）；或 approx/
noop 池周期复评；或四类引擎缺口铸造专项。

## 本轮：批 259——biaozhao2 表召·应状态机（2026-09-13，tag 732 / ABI 709）

- **表召族收官**：biaozhao2 表召·应（sp2 noop→approx）——任意角色弃置与
  「表」同花色同点数的牌时：「表」交予弃牌者 + owner 失去 1 点体力。
  新参数化过滤器 owner_alive_discard_matches_own_pile_card_<pile_def>
  （5 点注册零 ABI：运行时读堆首张花色点数与弃牌比对）+ 单发机器
  （堆牌交予弃牌者+is_lose_hp 流失）。
- **测试**：original_biaozhao2_machine.rs 2 件全过（匹配弃牌交表+失体力/
  非匹配静默）。测试驱动：直派 DomainEvent::CardsMoved{reason: Discard}。
- **断言同步**：sp2 包 ABI 708→709 ×4 文件（trigger 66→67、modifier
  238→237、modules 75→76 无冻结断言）。

回归（冻结终态，全部改动落盘后单次运行，exit 0）：workspace 全量
**4454 通过/0 失败/1 忽略**（批 258 终态 4452 + 2 件净增）。缓存治理：批内
清理 target/debug/incremental。

**下一步**：已解锁族逐件消化继续；或 approx/noop 池周期复评；或四类引擎
缺口铸造专项。

## 本轮：上游 v1.11.7 增量同步（5c02a745..df2682e6，2026-10-08）

- **上游更新**：`archive/upstream/noname` git pull，2026-09-15（5c02a745）→
  2026-10-08（df2682e6），42 提交 / 378 文件（+30554/−22450）。上游版本
  v1.11.6 → **v1.11.7**。`noname-liuli` 无新提交（8485dbb 维持），本地
  extension 改动 stash 后已还原。2026-10-10 核对：reflog 记为 `reset: moving to HEAD`，
  之后本地修改从 50 个文件降到 7 个；消失的 43 个文件中 40 个只是换行（CRLF→LF）差异，
  内容与 9-23 一致，史诗卡牌 3 个文件有 148 字节无法由换行解释（[复算](../operations/evidence/2026-10-10-doc-cleanup/liuli-crlf-check.txt)）。
- **内容面概况**：无新增 character 文件；15 个武将包（bingshi/clan/collab/
  huicui/jsrg/key/mobile/newjiang/sp/sp2/standard/sxrm/tw/xianding/yingbian）
  的 skill/character 修改；国战模式新增 jiubian/zifeng 文件（mode 层，不在
  catalog 移植范围）；card 层 guozhan/huodong/jiubian 三处。
- **sync 传播**：`node scripts/sync-original-content.mjs`（含内置 verify）→
  legacy-source 366 文件全量重拷（含 `offline/skill/index.js`→`index.ts`
  TS 迁移重命名，全仓无 rs/mjs/py 引用该路径，无断言破坏）、portraits 2703
  硬链接；`--verify-only` 复核绿。
- **catalog 残差（待移植，未动 npack）**：**2557/2576 武将、6151/6190 技能
  ported**；新增 source_synced 19 将 39 技，分布 offline×4、xianding×3、
  collab×2（菲菲公主/GG 咚联动）、bingshi/clan/huicui/mobile/newjiang/old/
  onlyOL/sb/sp2/tw 各 1。全部无本地 ported 文件（确认为新增，非降级）。
- **测试影响**：本轮零代码/npack 改动，npack 冻结计数与 include_bytes/
  include_str 嵌入面不受影响；grep 无 catalog 总数硬编码断言。未跑全量
  cargo（并行会话在途 engine-native，避免 target 争用）。
- **下一步**：19 将 39 技移植批（沿用 v1.11.6 批 146 流程：sync 残差清单→
  .skill/characters.json→rebuild-pack→四副本部署→种子敏感三件套）。2026-10-10 起
  该项登记在[进度主稿](../architecture/native-progress.md#后续待做按阻塞关系) T15。

## 本轮：T15③ / D6——v1.11.7 残差 19 将 39 技旧线移植（2026-10-16）

用户 D6 裁决启动移植。沿用批 146 流程，零新原语、零 ABI 抬升（host 753 不变）：

- **39 技诚实 noop**：各包 `.skill` 落位（`modifier hand_limit add 0` + 上游语义/
  阻塞机制注记，ADR 0597）。衍生技（神魔孙策 sm* 等）一并落库。
- **19 将 roster**：新增/更新 `characters.json`；纠偏族荀灌首技、九鼎孙尚香枭姬
  本地 def、神张角 `dcyizhao` + max_hp 4。
- **13 包 rebuild + 四副本部署**：ported / extensions×2 / npack-store（offline、
  onlyOL 另有 `content/original/packages/`）。
- **catalog 闭包**：`source_synced` 19 将 39 技 → **0**；**2576/2576 + 6190/6190
  全 ported**（`sync-original-content.mjs` + `--verify-only` 绿）。
- **测试对账**：冻结 skill/character/modifier 计数随 roster 增长更新；collab
  `olchouniu`/`olsishe` canonical hash 重录；offline/xianding 开局选择排空与
  jun_zhaoshou/tanhu 种子窗扩展。

回归（`CARGO_TARGET_DIR=.scratch/target-oldline`）：engine-core **3778/0/1**；
engine-api **4/0**；conformance ALL PASS；cross-target-hash PASS；sync-verify
PASS。证据：[2026-10-16-t15-3-v117-transplant](../operations/evidence/2026-10-16-t15-3-v117-transplant/README.md)。
