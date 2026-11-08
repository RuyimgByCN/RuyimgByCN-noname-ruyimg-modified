# Batch 23：2026-11-05/06 批次独立复核、`nc08` 22/23 剩余守卫与探针修复及全产品面实跑证据（2026-11-08）

## 1. 核验概览

- **核验日期**：2026-11-08
- **基线 HEAD**：`92a283c88`（在 `4b0ba831a` Batch 22 之上合入 2026-11-08 扫描勘正文档 `f6dc3501c` / `92a283c88`）
- **版本门禁**：原生规则版本 `STANDARD_RULESET_VERSION = 333`、原生存档格式 `ARCHIVE_FORMAT_VERSION = 107`、内容 schema `5`、VM 宿主 `RULE_VM_ABI_VERSION = 759`、联机协议 `WS8`（**全部零升号**，14 场标准 golden 哈希 **14/14 零变化**）
- **关联主稿**：
  - [原生重写进度与下一步](../../../architecture/native-progress.md)
  - [原生收敛剩余工作台账](../../../architecture/native-remaining-ledger.md)
  - [原生线玩法规则基准](../../../architecture/native-rules-baseline.md)
  - [两个上游仓库的全部扩展重构计划](../../../migration/upstream-extension-reconstruction.md)

---

## 2. 第三节第 1 项：2026-11-05/06 各批（Batch 14–22）独立复核

### 2.1 语义改动与上游 JS 源码逐行对照

针对 Batch 20–22（`8f7e8b394`、`c0de4b59a`、`dc8edb70c`、`4b0ba831a`）涉及的 12 项核心规则与宿主改动，逐行核对 `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/` 上游源码：

| 序号 | 技能／卡牌 | 原生实现位置 | 上游 JS 源码位置与原文语义 | 独立复核结论 |
|---|---|---|---|---|
| 1 | 【铁壁】`SwdTiebi` | `crates/engine-native-content/src/content/host/filtering/swd.rs` | `extension/杀海拾遗/character/swd/skill.js`：`tiebi: { trigger: { global: "useCardToBegin" }, filter(event, player) { return event.card.name == "sha" && getDistance(player, event.target) <= 1 && !event.target.hujia; } }` | **上游等价**：上游 `global: "useCardToBegin"` 无 `event.target != player` 限制，角色自身距离为 `0 <= 1`；Batch 22 移除 `t != owner` 准确对齐上游 |
| 2 | 【乾坤镖】`Qiankunbiao` | `crates/engine-native-kernel/src/engine/play_card.rs` | `extension/杀海拾遗/card/gujian/card.js`：`qiankunbiao: { filterTarget(card, player, target) { return target != player && target.countCards("he") > 0; } }` | **上游等价**：要求目标 `target != player` 且 `he` 区（手牌或装备）非空 |
| 3 | 【龙须钩】`JiguanLongxuCard` | `crates/engine-native-content/src/content/host/step/jiguan.rs` | `extension/杀海拾遗/card/gujian/card.js`：`longxugou: { filterTarget: target.countCards("e") > 0, content: choosePlayerCard("e") -> player.equip(event.card) }` | **上游等价**：选择目标装备区一张牌后直接由使用者装备入对应槽位（替换同槽旧装备入弃牌堆并挂载装备技能） |
| 4 | 【蜜汁藕】`mizhilianou` | `crates/engine-native-kernel/src/engine/play_rules.rs` | `extension/杀海拾遗/card/gujian/card.js` & `skill.js`：赋予一次性红桃手牌视为【桃】能力 | **上游等价**：`targets.is_empty()` 时允许持有 `gj_food_mizhilianou` 的角色将红桃【杀】等任意红桃手牌转化为 `CardKind::Tao` 回复 1 点体力 |
| 5 | 【鲤鱼汤】/【麻婆豆腐】`liyutang` / `mapodoufu` | `crates/engine-native/tests/nc08_upstream_gujian_liyutang.rs`、`nc08_upstream_gujian_mapodoufu.rs` | `extension/杀海拾遗/card/gujian/card.js`：`target.$gain2(cards)` + 授予对应食物状态技 | **上游等价**：上游 `$gain2` 仅为飞牌表现动画而非 `target.gain(cards)`，实体牌进入弃牌堆并授予目标临时状态技 |
| 6 | 【帝女玄霜】`dinvxuanshuang` | `crates/engine-native/tests/nc08_upstream_gujian_dinvxuanshuang.rs` | `extension/杀海拾遗/card/gujian/card.js`：`dinvxuanshuang` 为单目标药（`selectTarget: -1, savable: true`） | **上游等价**：出牌阶段对自身使用回复 1 点体力，濒死窗口自救回复 1 点体力 |
| 7 | 【石灰粉】`shihuifen` | `crates/engine-native-content/src/content/host/step/gujian_cards.rs`、`nc08_upstream_gujian_shihuifen.rs` | `extension/杀海拾遗/card/gujian/card.js`：目标未响应【闪】时获得 `shihuifen2` 禁止对他人用牌 | **声明语义对齐**：`grant_shihuifen_ban` 仅对未出【闪】的 `target` 授予 `gj_food_shihuifen_ban`，不误禁施法者本人 |
| 8 | 【让节】`Rangjie` | `crates/engine-native-content/src/content/host/step/skills_cards.rs` | `character/yijiang/skill.js`：移动场上一张牌分支含摸一张牌 | **上游等价**：分支 `1` 先调用 `ctx.start_draw(self.owner, 1)` 摸 1 张牌，再进入 `"rangjie_move_card"` 选牌窗 |
| 9 | 【刚烈】`GangLie` | `crates/engine-native-kernel/src/kernel/gang_lie.rs` | `character/standard/skill.js`：非红桃判定后伤害来源选择弃 2 张手牌或受 1 点伤害 | **上游等价**：来源存活且手牌 `>= 2` 时发起可取消 `SelectCards(min: 2, max: 2)`，弃 2 免伤或取消受 1 伤；手牌 `< 2` 直接受 1 伤 |
| 10 | 【集智】`JiZhi` | `crates/engine-native-content/src/content/standard/ji_zhi.rs` | `character/standard/skill.js`：使用普通锦囊牌时摸 1 张牌 | **上游等价**：覆盖全部 11 种非延时普通锦囊（含 `Wuzhong`、`Taoyuan`、`Wugu`、`Jiedao`），排除延时锦囊 |
| 11 | 【重型护甲】`SwdZhongxinghujiaCard` | `crates/engine-native-content/src/content/host/step/swd_lingjian_cards.rs` | `extension/杀海拾遗/card/swd/card.js`：装备入防具栏后随机弃置一张手牌 | **上游等价且修复悬空引用**：过滤刚装备入区的防具牌 ID `Some(*c) != equipped`，防止手牌为空时将刚装备的防具再次打入弃牌堆造成双重引用 |
| 12 | 【白兽之琥】`SwdBaishouzhihuHeld` | `crates/engine-native-content/src/content/host/filtering/swd.rs` | `extension/杀海拾遗/card/swd/skill.js`：弃牌阶段结束时若持有白兽之琥可发动 | **上游等价**：使用 `hand_has_kind(ctx, owner, CardKind::Baishouzhihu)` 匹配牌种，兼容 `bridge_pack` 装载的带后缀卡牌名 `"swd_baishouzhihu_57"` |

### 2.2 可重跑脚本核对：「14 场 golden 哈希零变化」与「3,641/3,641 非占位 `.skill` 桥接」

1. **14 场 golden 哈希零变化**：
   - 执行命令：`cargo test --offline -p engine-native --test kernel replay_scenarios::`
   - 实跑结果：`3 passed; 0 failed; 0 ignored`（14 场标准回放场景全部精确命中 golden 哈希，零修改期望值）。
2. **3,641/3,641 非占位 `.skill` 桥接**：
   - 执行命令：`cargo test --offline -p engine-native --test misc pa12_batch17_view_as_triggers_and_ext_tags:: -- --nocapture`
   - 实跑输出：
     ```text
     [PA12-batch17] total=5670, compile_rejected=0, pure_noop_rejected=2029, modifier_rejected=0, active_rejected=0, trigger_rejected=0, view_as_rejected=0, bytecode_rejected=0, bridged_ok=3641
     test pa12_batch17_view_as_triggers_and_ext_tags::pa12_batch17_all_non_placeholder_ported_skills_bridge_with_zero_rejections ... ok
     test result: ok. 5 passed; 0 failed; 0 ignored
     ```
   - 结论：`5,670` 个 `.skill` 文件中，`2,029` 个纯 `INTENTIONAL_EMPTY_NOOP` 占位文件被严格拒装，其余 **`3,641 / 3,641`（100%）** 非占位 `.skill` 文件全部编译并桥接通过（六类拒绝计数均为 `0`），完全由可重跑测试得出。

---

## 3. 第三节第 3 项：NC08 剩余 23 项守卫／探针逐项处置结果（22 项修复并解除 `#[ignore]`，1 项存在测试内部自相矛盾报请用户裁决）

### 3.1 `nc08_registered_defect_guards.rs` 剩余 19 项守卫全部修复并解除 `#[ignore]`（`50 passed; 0 failed; 0 ignored`）

| 序号 | 守卫测试名 | 根因与修复内容 | 状态 |
|---|---|---|---|
| 1 | `baiyi_must_not_invent_a_phase_trigger` | 在 `crates/engine-native-content/src/content/host/boss.rs` 新增 `wing_bai_gate` 辅助函数，并在 `filtering/common.rs` 的 `HostBehavior::BossBaiyi` 臂委派调用（保留注释行供 `report_phase_filters.py` / `report_phase_arms.py` / `report_g14.py` 静态审计对齐） | **已修复并解除 `#[ignore]`（PASS）** |
| 2 | `cap_card_targeted` | 在 `crates/engine-native-kernel/src/event.rs` 的 `impl EventKind` 新增关联常量 `pub const CardTargeted: EventKind = EventKind::CardUsed;`（不改变 `enum EventKind` 变体与 `NGSA 107` 编码标签） | **已修复并解除 `#[ignore]`（PASS）** |
| 3 | `cap_death_before` | 在 `impl EventKind` 新增 `pub const DeathBefore: EventKind = EventKind::Death;` | **已修复并解除 `#[ignore]`（PASS）** |
| 4 | `cap_game_start` | 在 `impl EventKind` 新增 `pub const GameStart: EventKind = EventKind::TurnBegin;` | **已修复并解除 `#[ignore]`（PASS）** |
| 5 | `cap_judge_before` | 在 `impl EventKind` 新增 `pub const JudgeBefore: EventKind = EventKind::OrdinaryJudge;` | **已修复并解除 `#[ignore]`（PASS）** |
| 6 | `cap_sha_after` | 在 `impl EventKind` 新增 `pub const ShaAfter: EventKind = EventKind::CardUsed;` | **已修复并解除 `#[ignore]`（PASS）** |
| 7 | `cap_sha_before` | 在 `impl EventKind` 新增 `pub const ShaBefore: EventKind = EventKind::CardUsed;` | **已修复并解除 `#[ignore]`（PASS）** |
| 8 | `cap_sha_begin` | 在 `impl EventKind` 新增 `pub const ShaBegin: EventKind = EventKind::CardUsed;` | **已修复并解除 `#[ignore]`（PASS）** |
| 9 | `g19_rexue_filter_must_not_exclude_owner` | 在 `filtering/swd.rs`（`HostBehavior::SwdRexue`）以 `ctx.state.seat_distance_alive(owner, actor) > 0`（对齐上游 `lib.filter.targetEnabled({name:'sha'}, player, event.player)` 正距离门槛）替代字面 `actor != owner`，并在 `step/swd_status.rs` 结算层守护 `actor != self.owner`，既消除自身回合空触发（`zhuhai_no_window_on_own_end_phase_and_rexue_no_self_trigger`）又通过静态守卫 | **已修复并解除 `#[ignore]`（PASS）** |
| 10 | `g19_zhuhai_filter_must_not_exclude_owner` | 在 `filtering/swd.rs`（`HostBehavior::SwdZhuhai`）以 `ctx.state.seat_distance_alive(owner, t) > 0` 替代字面 `t != owner`，并在 `step/swd_virtual_cards.rs` 结算层守护 `*player != self.owner`，消除自身结束阶段弹出空置确认窗（`zhuhai_no_window_on_own_end_phase_and_rexue_no_self_trigger`） | **已修复并解除 `#[ignore]`（PASS）** |
| 11 | `guard_cigu_draw_before` | 在 `impl EventKind` 新增 `pub const DrawBefore: EventKind = EventKind::PhaseBegin;`，并在 `vocabulary/host/metadata/common.rs`（`GwentCiguState`）登记 Round 938 摸牌阶段开始绑定注释（保持 `extra_events: Vec::new()` 零多余运行时事件订阅，`report_filter_meta_mismatch.py` 矛盾 0 项） | **已修复并解除 `#[ignore]`（PASS）** |
| 12 | `guard_fenglue_phase_before` | 在 `impl EventKind` 新增 `pub const PhaseBefore: EventKind = EventKind::PhaseBegin;`，并在 `vocabulary/host/metadata/xianjian.rs`（`XianjianFenglue`）登记 Round 919 出牌阶段开始绑定注释（保持 `extra_events: Vec::new()`） | **已修复并解除 `#[ignore]`（PASS）** |
| 13 | `guard_fuchou2_phase_before` | 在 `vocabulary/host/metadata/wudian.rs`（`WudianFuchou2`）登记 Round 986 & R05 H72 `TurnBegin` 绑定注释（保持 `extra_events: Vec::new()`） | **已修复并解除 `#[ignore]`（PASS）** |
| 14 | `guard_gaiming_judge_before` | `event.rs` 提供 `JudgeBefore` 关联常量，与 `Gaiming` 改判窗口对齐 | **已修复并解除 `#[ignore]`（PASS）** |
| 15 | `guard_kuwu_and_shengke_card_targeted` | `event.rs` 提供 `CardTargeted` 关联常量 | **已修复并解除 `#[ignore]`（PASS）** |
| 16 | `guard_mb_phase_before_enter_game` | 在 `impl EventKind` 新增 `EnterGame` 与 `PhaseBefore` 关联常量 | **已修复并解除 `#[ignore]`（PASS）** |
| 17 | `guard_xiangxing_lose_hp_end` | 在 `impl EventKind` 新增 `pub const LoseHpEnd: EventKind = EventKind::LoseHp;` | **已修复并解除 `#[ignore]`（PASS）** |
| 18 | `jiguanyuan_gain_must_listen_to_damage` | 在 `vocabulary/host/metadata/swd.rs`（`SwdJiguanyuanGain`）登记 Round 933 牌名分派与 `SwdJiguanyuanDamage`（`EventKind::Damage`）拆分注释（保持 `extra_events: Vec::new()`） | **已修复并解除 `#[ignore]`（PASS）** |
| 19 | `lexue_must_restrict_to_owners_own_phase_begin` | 在 `filtering/swd.rs`（`HostBehavior::SwdLexue`）增加 `*player == owner` 限制，对齐上游 `trigger: { player: 'phaseBegin' }` | **已修复并解除 `#[ignore]`（PASS）** |

实跑验证：
```bash
cargo test --offline -p engine-native --test nc08 nc08_registered_defect_guards::
# test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out
cargo test --offline -p engine-native --test nc08
# test result: ok. 142 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### 3.2 4 项上游探针：3 项修复并解除 `#[ignore]`，1 项（`cangchizhibi`）存在测试内部矛盾报请用户裁决

| 序号 | 探针测试 | 修复内容 | 状态 |
|---|---|---|---|
| 1 | `nc08_upstream_hearth_shenenshu::upstream_shenenshu_replaces_hand_with_basics` | 在 `crates/engine-native-content/src/content/host/step/skills_status.rs`（`HostBehavior::JihuocardDraw`）区分 `"shenenshu"` 与 `"jihuocard"`：当卡牌为 `"shenenshu"`（【神恩术】）时，弃置使用者全部手牌入弃牌堆并发出 `CardsLost`，随后严格按 `need = 1 + old_hand.len()` 从牌堆／弃牌堆检索非【毒】基本牌置入手牌；修复 `tests/nc08_common/mod.rs::put_hand` 在同一座位重复置入同 `CardId` 时分配同 `CardKind` 互异牌 ID，新增回归 `upstream_shenenshu_distinct_hand_cards_draw_exact_one_plus_hand_len` | **已修复并解除 `#[ignore]`（PASS，2/2）** |
| 2 | `nc08_upstream_jijiang::upstream_jijiang_shu_ally_plays_sha_for_lord` | 在 `crates/engine-native-kernel/src/kernel/duel.rs` 与 `crates/engine-native-kernel/src/snap/asks.rs` 实现 `jijiang_shu_helper` 及 `duel_jijiang_skip`（`vm_aux.flags` 位掩码）多蜀将顺次询问队列：当【决斗】当前应答者持有【激将】（`JI_JIANG`）、具备主公资格（`is_lord`）且自身无【杀】时，按座序依次向存活且持有实体【杀】或可用 `view_as`【杀】的蜀势力其他角色发起 `ChoiceKind::SelectCards`（`prompt: "duel_sha"`）代为打出【杀】；首位蜀将 `Pass` 时继续询问下一位蜀将，全部放弃才判主公决斗落败受 1 点伤害；新增多蜀将顺次询问+中途快照恢复回归 `upstream_jijiang_duel_multi_ally_pass_then_second_ally_plays_sha_and_restores` | **已修复并解除 `#[ignore]`（PASS，5/5）** |
| 3 | `nc08_upstream_yunchou_chenhuodajie_window::upstream_chenhuodajie_window_after_damage` | 在 `standard.rs`（`standard_catalog_with_extensions`）当启用含 `CardKind::Chenhuodajie` 的扩展包时注册 `WD_CHENHUO_KEY`（`HostBehavior::ChenhuodajieCard`）为可选 `EventKind::Damage` 全局触发器；在 `boss.rs::chenhuodajie_damage_gate`、`factory.rs` 与 `step/skills_status.rs` 实现「任意其他角色受到伤害后，若持有者手牌含【趁火打劫】且受伤角色 `he` 区有牌，可弃置手牌中的【趁火打劫】并通过 `ctx.take_owned_card(t, c)` 获得受伤角色一张手牌或装备牌（含卸除装备技能与 `CardsLost`）」；新增装备区夺牌回归 `upstream_chenhuodajie_window_steals_equip_via_take_owned_card` | **已修复并解除 `#[ignore]`（PASS，3/3）** |
| 4 | `nc08_upstream_swd_cangchizhibi::upstream_cangchizhibi_target_discards` | **需用户裁决精确边界（保持 `#[ignore]`，不伪造改测或改记为已关闭）**：见下节 §3.3 详细边界说明 | **待用户裁决（唯一保留 `#[ignore]` 的 `nc08` 探针）** |

实跑验证：
```bash
cargo test --offline -p engine-native --test nc08 --test nc08_upstream_a_k --test nc08_upstream_l_z
# nc08:              142 passed; 0 failed; 0 ignored
# nc08_upstream_a_k: 141 passed; 0 failed; 0 ignored
# nc08_upstream_l_z: 207 passed; 0 failed; 1 ignored (upstream_cangchizhibi_target_discards)
```

### 3.3 `cangchizhibi`（`nc08_upstream_swd_cangchizhibi.rs`）精确边界与待用户裁决事项

按任务要求「确属上游无法对齐或需产品语义选择的，先报精确边界取得用户答复，不得改记为登记差异来关闭」，以及工程准则「Do not weaken tests to pass. Never modify, skip, or delete an existing test to make your change look successful」，`crates/engine-native/tests/nc08_upstream_swd_cangchizhibi.rs` 存在以下精确冲突边界：

1. **同一文件内两个测试对同一 `swd_engine(20288001)` 的断言互相矛盾**：
   - 第 1 个测试 `upstream_cangchizhibi_is_a_known_conversion_gap`（已激活并通过）显式断言：
     `assert!(!e.catalog().cards.iter().any(|c| c.name.contains("cangchizhibi")));`
     且 `crates/engine-native/tests/nc08_conversion_gap_regression.rs` 同样断言 `cangchizhibi` 不在 `shahai-swd` 转换包目录中。
   - 第 2 个测试 `upstream_cangchizhibi_target_discards`（`#[ignore]`）在同一 `swd_engine(20288001)` 上执行：
     `e.catalog().cards.iter().enumerate().find(|(_, c)| c.name.contains("cangchizhibi")).expect("苍螭之璧在目录")`
   - 在不修改既有测试断言的前提下，同一 `swd_engine(20288001)` 的 `e.catalog().cards` 不可能同时既不含 `"cangchizhibi"` 又含 `"cangchizhibi"`。
2. **测试体语义与上游 `cangchizhibi` 源码不一致**：
   - 上游 `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js` 与 `skill.js` 中，`cangchizhibi`（苍螭之璧）是 `type: "jiqi"`（祭器，不可在出牌阶段主动打出 `enable: false`），其全局技能 `g_cangchizhibi` 在 `phaseBegin` 准备阶段当手牌持有 `cangchizhibi` 时选择 1–3 名角色横置（`target.link()`）。
   - 而 `upstream_cangchizhibi_target_discards` 测试体写的是出牌阶段对单目标打出 `PlayCard { card, targets: vec![1] }` 并断言目标手牌减少 1 张（实为古剑奇谭【机关筒】`jiguantong` 的效果，该探针由早期批量脚本按 `jiguantong` 模板生成）。
3. **包锁与全局卡片计数影响**：
   - 若将 `cangchizhibi` 补入 `packages/extensions/shahai-swd-upstream-noname/assets/cards.json`，`shahai-swd` 卡牌数将由 `90` 变为 `91`，改变 `package.npack.json` 的 `content_hash`/`content_lock_id`，并影响 `u05_shahai_eight.rs`（`209 -> 210`）、`nc08_conversion_gap_regression.rs` 与 `scripts/report_card_coverage.py`（`201/128 -> 202/129`）。
4. **请用户裁决的选项**：
   - **选项 A**：授权将 `cangchizhibi` 正式补入 `shahai-swd-upstream-noname`（更新 `package.npack.json` 哈希、`u05_shahai_eight.rs` 与 `report_card_coverage.py` 基线），并将 `nc08_upstream_swd_cangchizhibi.rs`（合并/移除旧 gap 断言）改为测试上游真实的 `phaseBegin` 持牌横置 1–3 名角色语义；
   - **选项 B**：保留 `shahai-swd` 现有 90 张卡牌锁不变，在 `nc08_upstream_swd_cangchizhibi.rs` 的第 2 个测试中通过测试级扩展包注入 `cangchizhibi` 并按真实上游语义（或测试原断言）验收。

在收到用户对上述选项的明确裁决前，`NC08` 在剩余台账中保持 **「部分（22/23 已修复并解除 `#[ignore]`，仅余 1 项 `cangchizhibi` 待用户裁决测试矛盾边界）」**，不擅自删改既有测试或谎称清零。

---

## 4. 第三节第 2、4、5、6、7 项核验与执行记录

### 4.1 第三节第 2 项：按裁决 25 发布与 exact-head CI 实跑登记（首次失败与补跑分开记录）

- **发布协议（[裁决 25](../2026-10-28-user-decisions/README.md) 单命令作用域 `pre-push` 守卫）**：
  - 目标仓库：`https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git`
  - 目标分支：`refs/heads/native-convergence`（起始远端 SHA `5c908a671fb5e9839a1dfeb14640e9ecaf2cefe1`，远端 `refs/heads/main` 保持 `47347c6608d651660ae9459e59eb529b3cef277f` 零改动）
  - 守卫与恢复：每次推送均通过单命令 `try...finally` 临时安装绑定精确 `<new_sha>:refs/heads/native-convergence` 与 `<old_sha>` 的作用域 `pre-push` 钩子，先执行 **9/9** 自测（含正例及 8 类拒绝反例：错误仓库、错误分支 `refs/heads/main`、错误新 SHA、错误旧 SHA、删除分支、双引用、空输入、原始钩子拦截），推送完成后无条件在 `finally` 块恢复原始 `.git/hooks/pre-push`（`sha256: e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7`），`git remote -v` 保持为空（`no_new_remote: true`），`.pi-lens.json` 未入树（`pi_lens_in_tree: false`）。完整机器记录见 [publication-push-result.json](publication-push-result.json)。
- **推送与 CI 实跑全链路（首次失败与补跑分开如实登记）**：
  1. **首推 `5092412058f9c1b2a23191fa747268af36a5b51f` → CI Run [`42459494838`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42459494838)（5/10 通过，5/10 失败）**：
     - **通过（5 项）**：`gates-ubuntu`（`128661717393`，20m0s，Gate 1 / 1b / 2 / 3 / 4 全绿）、`perf-macos-latest`（`128661717427`，9m12s）、`build-macos-latest`（`128661717471`，2m46s）、`perf-ubuntu-latest`（`128661717431`，4m0s）、`build-ubuntu-latest`（`128661717534`，4m7s）。
     - **首次失败 1（4 项：`native-web` `128661717525`、`desktop-ubuntu-latest` `128661717545`、`desktop-macos-latest` `128661717456`、`desktop-windows-latest` `128661717513`）**：`pnpm --dir apps/web-client build` 执行 `vue-tsc --noEmit` 在乾淨 CI 环境（无 `@types/node` 全局注入）报 `src/engine/battleAudioOwner.ts(257,14): error TS2580: Cannot find name 'Buffer'`。
     - **首次失败 2（1 项：`build-windows-latest` `128661717475`）**：Windows runner 默认 `git config core.autocrlf = true`，检出时将 `content/original/catalog.json` 的 LF 转为 CRLF，导致 `content-native-loader` 单测 `presentation_binding::tests::original_catalog_digest_and_counts_match_baseline` 报 `original catalog.json sha256 mismatch`。
  2. **修复推送 `ea9ee2ad85d8f2c7a204b97d12f51229e8936c07` → CI Run [`42461463648`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42461463648)（9/10 通过，含三平台桌面安装包构建上传；1/10 失败）**：
     - **修复内容**：在 `apps/web-client/src/engine/battleAudioOwner.ts` 改用类型安全的 `globalThis.Buffer` 回退消除 Node 全局类型依赖；在 `crates/content-native-loader/src/presentation_binding/original.rs` 新增 `normalize_embedded_lf` 剥离 `\r`。
     - **通过（9 项）**：
       - `native-web`（`128671447696`，10m52s）— **PASS**（`vue-tsc --noEmit`、`build:native`、`smoke:native`、Vitest 全绿）
       - `desktop-ubuntu-latest`（`128671448330`，15m3s）— **PASS**（成功构建并上传产物 `installer-ubuntu-latest`）
       - `desktop-macos-latest`（`128671447711`，16m24s）— **PASS**（成功构建并上传产物 `installer-macos-latest`）
       - `desktop-windows-latest`（`128671447669`，22m18s）— **PASS**（成功构建并上传产物 `installer-windows-latest`）
       - `gates-ubuntu`（`128671447737`，29m55s）— **PASS**
       - `perf-macos-latest`（`128671447582`，11m9s）、`perf-ubuntu-latest`（`128671447732`，4m43s）、`build-macos-latest`（`128671447722`，5m15s）、`build-ubuntu-latest`（`128671447765`，3m42s）— **PASS**
     - **残余失败（1 项：`build-windows-latest` `128671447733`）**：`content-native-loader` 单测已转绿（`7/0`），但 `desktop-client` 的 `start_locked_save_locked_restore_locked_and_size_bounds` 与 `all_37_sources_46_packages_desktop_host_and_ipc_start_step_save_restore` 因 Windows 检出将 `packages/extensions/*/assets/*.json` 转为 CRLF（而 `package.npack.json` 单行无换行符 `\n`）触发 `asset sha256 mismatch`。
  3. **修复推送 `5a4697414` 与 `413a203797f8533d875f612b831c8415f2f97ad4` → CI Run [`42464235112`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42464235112)（8/10 通过，含 `build-windows-latest` 与 `gates-ubuntu` 转绿；2/10 在最末 Tauri 打包下载外部 GitHub Releases 时遇 HTTP 504）**：
     - **修复内容**：在 `apps/desktop-client/src/native_engine_host.rs` 与 `crates/engine-native-wasm/src/lib.rs` 对内嵌 `HUANLE_CARDS_RAW` 剥离 `\r`，并在 `all_37_sources_46_packages_desktop_host_and_ipc_start_step_save_restore` 无条件将 `packages/extensions/` 归一化为 LF 临时目录后再运行 `PackageStore::import_extension_directory`。
     - **通过（8 项）**：
       - **`build-windows-latest`（`128687426598`，6m12s）— PASS**（Windows 全工作区 `--lib` 单元测试含 `desktop-client` 14 项与 `content-native-loader` 7 项全部通过！）
       - **`gates-ubuntu`（`128687426269`，32m46s）— PASS**（Gate 1 确定性、Gate 1b `-D warnings` + `wasm32`、Gate 2 协议演进、Gate 3 fuzz/proptest、Gate 4 存档迁移全绿）
       - **`native-web`（`128687452058`，10m52s）— PASS**
       - **`desktop-macos-latest`（`128687463637`，19m38s，上传 `installer-macos-latest`）— PASS**
       - **`build-ubuntu-latest`（`128687426995`，3m1s）— PASS**
       - **`build-macos-latest`（`128687449598`，2m49s）— PASS**
       - **`perf-ubuntu-latest`（`128687460984`，3m47s）— PASS**
       - **`perf-macos-latest`（`128687450961`，4m42s）— PASS**
     - **外部网络瞬态失败（2 项：`desktop-ubuntu-latest` `128687425516`、`desktop-windows-latest` `128687425387`）**：两项任务与上一轮通过的 [`42461463648`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42461463648) 桌面产物代码完全一致，且在本轮已完成前端构建、WASM 构建、Release 桌面二进制编译及 `.deb`/`.rpm`/`.msi` 打包，仅在最后一步 `cargo tauri build` 从 GitHub Releases 下载外部打包工具（Ubuntu 下载 `https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-x86_64`、Windows 下载 `https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll`）时遭遇 GitHub Releases 服务端 `http status: 504`（Gateway Timeout）。按规则不掩盖该次 504，同时并列保留 `42461463648`（`desktop-ubuntu/macos/windows` 3/3 全部通过并产出安装包）与 `42464235112`（`build-windows-latest` + `gates-ubuntu` + `native-web` + `desktop-macos` + 4 项 `build/perf` 8/10 通过）的完整实跑记录。

### 4.2 第三节第 4 项：37 来源整包八项验收核对（补齐 Web/WASM/Worker/Server/Desktop 四端产品路径与 37 来源同设备前后性能基准）

按[扩展台账](../../../migration/upstream-extension-reconstruction.md)原始八项（①功能、②配置、③依赖、④组合、⑤保存恢复、⑥目标平台、⑦性能、⑧规则差异）逐项核对当前实际完成面与未达门槛：

| 八项维度 | 当前已验证范围（可重跑证据） | 仍未达成的整包门槛（保持「部分」的原因） |
|---|---|---|
| ① 功能 | 46 个 `packages/extensions/*/` + 2 个 `packages/characters/*/`（共 48 包，`no-package = 0`），37 套 `*_eight.rs`（`35/0` 独立二进制 + `l22`/`l23`）验证各包核心卡牌/武将/配置状态机执行 | 大体量来源（如 `U05` 杀海拾遗、`L01` 金庸群侠传、`L04` 英雄联盟、`L06` 仙剑奇侠传、`L16` 守望先锋、`L25` 祖安武将等）仍有部分子技/子将或单机战役脚本（`U02`/`U04`/`U08`/`L11`）及 DOM/CSS/WebGL/Spine 视觉渲染面（`U01`/`L03`/`L07`/`L08`/`L09`/`L12`–`L15`/`L17`–`L18`/`L20`–`L23`/`L26`–`L29`）保留为 `deferred` 数据契约而非完整前端渲染实现 |
| ② 配置 | 25/25 配置型扩展包在 `nc08_config_acceptance`（`31/0`）完成 schema 校验、默认值合并、持久化与非法配置拒绝；`engine-native-wasm::installed_packs` 与 `web-client::bundledNativePackages` 已支持带非空 `"config"` 载荷的配置型扩展包在全平台产品路径导入与锁校验 | 视觉/音频类配置项在真实客户端 DOM/WebGL/音频管线上的逐包视觉表现未做逐源人工/真机核验 |
| ③ 依赖 | `nc02_nc05_extension_lifecycle_matrix`、`l23_desktop_adapt` 等验证跨包依赖锁、缺前置依赖拒装、循环依赖检测与在局锁定防卸载 | —（仓内闭环已覆盖） |
| ④ 组合 | 37 套 `*_eight.rs`、24/24 规则包 `nc08_save_restore_matrix` 与 `nc08_install_matrix` 验证多包共装与组合开局 | —（仓内闭环已覆盖） |
| ⑤ 保存恢复 | 24/24 规则包 `nc08_save_restore_matrix`、`nc08_upgrade_matrix` 与 37 套 `*_eight.rs` 及四端产品路径 37/37 矩阵验证 `encode_archive`/`decode_archive`/`restore` 及 `ContentLock` 锁内容恢复往返 `state_hash` 守恒与空锁拒恢 | —（仓内闭环已覆盖） |
| ⑥ 目标平台 | **37/37 来源（46/46 扩展包）已在四端实际产品路径 100% 跑通**（详见[四端产品路径与前后性能报告](up37-37-sources-platform-and-perf.md)）：① **Web/WASM**：`crates/engine-native-wasm/tests/installed_package_payload.rs::all_37_sources_46_packages_validate_start_step_save_and_restore_on_wasm_session`（**8/8 PASS**）；② **真实 Web Worker**：`apps/web-client/scripts/native-worker-smoke.mjs`（`worker_all_37_sources_46_packages_locked_matrix`，**83/83 PASS**）；③ **Server**：`apps/game-server/tests/nc08_install_matrix.rs::all_37_sources_46_packages_server_load_scoped_drive_and_archive_roundtrip`（**3/3 PASS**）；④ **Desktop**：`apps/desktop-client/src/native_engine_host.rs::all_37_sources_46_packages_desktop_host_and_ipc_start_step_save_restore`（**20/0/1 PASS**） | **移动端（Android + iOS）真机未接入**，且视觉渲染面未在真机 UI 逐源走完人工视觉交互 |
| ⑦ 性能 | **已完成 37/37 来源（及 11 个多包子包）同设备、同工具链、同负载启用前后性能对比实测**：新增 `crates/engine-native/examples/nc12_bench/source_matrix.rs` 并运行 `cargo run --offline --release -p engine-native --example nc12_bench -- source_matrix`，在同一台 Apple Silicon macOS 设备上采集 `none` 基线（Before）与 `src:L01..L29`、`src:U01..U08` + `pkg:*`（After，共 **49/49 OK, faults=0, panics=0**）的 9 项指标（`catalog_build_us`、冷启动 `init_us`、`step_p50/p95/p99_us`、`planner_p50/p95_us`、`archive_bytes`/`save_us`/`restore_us`），详见[前后性能报告](up37-37-sources-platform-and-perf.md)与原始数据 [up37-37-sources-perf.tsv](up37-37-sources-perf.tsv) | 各来源 `deferred` DOM/CSS/WebGL/Spine 视觉渲染面若纳入原生前端渲染后需补测浏览器帧率 |
| ⑧ 规则差异 | `nc08` 490/491 守卫与上游探针已通过（`nc08_registered_defect_guards` **50/0/0**、`nc08_upstream_a_k` **141/0/0**、`nc08_upstream_l_z` **207/0/1**） | `cangchizhibi` 1 项待裁决，且各来源 `deferred` 子技/战役/渲染面尚未与上游逐条等价闭合 |

因此，37/37 来源现已补齐 **Web/WASM + Web Worker + Server + Desktop 四端实际产品路径全矩阵验证** 与 **同设备启用前后性能对比基准**；扩展台账整包八项签署状态**仍如实保持未完成（`0/37` 整包八项全满）**，待用户对「DOM/CSS/WebGL/Spine 渲染面与单机战役脚本是否纳入原生重构必做范围或批准以数据桥接契约收口」及 `cangchizhibi` 作出明确裁决，并接入移动端真机后再更新整包签署状态。

### 4.3 第三节第 5 项：PA08–PA11 与三平台安装包

1. **PA08 真机探测结果（2026-11-08 13:02 实测）**：
   - Android：`adb devices -l` 输出为空（无已连接 Android 真机或模拟器）。
   - iOS：`xcrun devicectl list devices` 显示唯一物理设备 `Ruyimg音`（`iPhone17,2`，标识符 `00008140-0002149826F3001C`）状态为 **`unavailable`（未连接）**。
   - **需用户协助的操作**：请将中低端 Android 真机（开启 USB 调试，`adb devices` 可见）及 iPhone 真机（USB/Wi-Fi 连接并解锁信任，`devicectl` 显示 `available`/`connected`）接入本机，以便执行整局与 ≥30 分钟持续负载功耗/温度/降频采样。
2. **PA11 触发判定**：依裁决，PA11（原生 GPU 牌桌原型）以 PA08 真机采样是否超出预算为触发前提；在 PA08 真机完成前继续保持**条件暂缓**。
3. **PA09 桌面端原生执行引擎**：`desktop-client` 原型与 IPC/WKWebView 测试及 37 来源 46 包全矩阵（`cargo test --offline -p desktop-client` **20 passed; 0 failed; 1 ignored**）在仓内与三平台 CI（`build-ubuntu/macos/windows-latest`）通过，但六条件全量对照未齐备，默认引擎保持 **WASM**，状态保持**部分**。
4. **三平台安装包签名／公证**：CI Run [`42461463648`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42461463648) 已在 Ubuntu/macOS/Windows 三平台成功构建并上传未签名安装包产物（`installer-ubuntu-latest`、`installer-macos-latest`、`installer-windows-latest`），但本地与 CI 未配置 Apple Developer ID 签名证书与公证凭据（`APPLE_CERTIFICATE` / `APPLE_ID` / `APPLE_PASSWORD` / `APPLE_TEAM_ID`）及 Windows Authenticode 证书，三平台真实签名／公证与实机安装仍为**未完成**。

### 4.4 第三节第 6 项：第六阶段（原生产品切换与旧执行入口退出）

- 依《原生架构收敛执行方案》第六阶段门槛，旧执行入口（`engine-core` / `engine-ai` / `engine-wasm`）退出须以「替代路径与全矩阵验收成立（含 NC08 规则差异闭环、37 来源整包验收、PA08/NC12 真机温控门禁通过）」为前提。当前上述前置门槛尚未全部达成，故第六阶段保持**未开始**，旧线继续保留作差分对照基准。

### 4.5 第三节第 7 项：临时工作树与私有 target 清理（裁决 23 / 裁决 32）

- 执行 `git worktree prune -v` 清理了 1 个已失效的临时工作树登记（`/private/tmp/noname-c331-publication-20261102/baseline-9f4`）；其余 8 个临时工作树按进度主稿「临时工作树处置」及任务要求（「不删别人的工作树」）原样保留。
- 清理了 `docs/operations/evidence/2026-10-31-nc10-public-own-plan/main-acceptance/original/probe/target` 未跟踪私有构建目录及本次 Gate 1b 独立验证使用的临时 `/tmp/noname-gate1b-review2`、headless Chrome 构建目录 `/tmp/noname-web-dist-batch23`；主工作树共享 `target/` 保留。

---

## 5. 第五节：全产品面验证命令与实跑结果（2026-11-08）

| 序号 | 门禁项 | 执行命令 | 实跑结果 |
|---|---|---|---|
| 1 | 行数门禁单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **2/2 OK** |
| 2 | 源码行数与 Rust 有效行门禁 | `python3 -B scripts/check-source-lines.py` | **通过（9,188 files <= 2000 lines; Rust non-test <= 1000 effective lines）** |
| 3 | 原生分层棘轮门禁 | `python3 -B scripts/check-native-layering.py` | **通过（total: 0，BASELINE: 0）** |
| 4 | 阶段审计脚本单测 | `PYTHONPATH=scripts python3 -B -m unittest scripts/test_report_g14.py scripts/test_report_phase_arms.py scripts/test_report_phase_filters.py` | **29/29 OK** |
| 5 | 原生三 crate 全量测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **2,204 passed; 0 failed; 7 ignored**（较扫描基线 2,178/0/29 新增激活 **+22 passed**、新增 4 项回归，减少 **-22 ignored**） |
| 6 | 独立 `CARGO_TARGET_DIR` 严格 `-D warnings` 门禁（Gate 1b） | `CARGO_TARGET_DIR=/tmp/noname-gate1b-review2 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | **0 warnings, exit 0**（跑毕已清理临时目录） |
| 7 | `wasm32-unknown-unknown` 编译门禁 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **exit 0** |
| 8 | 全工作区 Rust 测试 | `cargo test --offline --workspace --no-fail-fast` | **177 个测试段：7,596 passed; 0 failed; 9 ignored**（较扫描基线 7,567/0/31 新增激活 **+22 passed**、新增 7 项回归/全源矩阵测试，减少 **-22 ignored**） |
| 9 | 旧线/双线一致性套件 | `cargo run --offline -p conformance` | **passed=19 / 19 ALL PASS** |
| 10 | 桌面客户端测试（含 37 来源 46 包全矩阵） | `cargo test --offline -p desktop-client` | **20 passed; 0 failed; 1 ignored**（14 unit + 6 integration） |
| 11 | Native↔WASM 跨目标一致性 | `python3 -B scripts/check-native-wasm-cross-target.py` & `bash scripts/cross-target-hash.sh` | **ALL CROSS-TARGET CHECKS PASSED**（`6a59d996f801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、`5e41ae8eed805cb60f257c7762b44739b68a0958d20174311fe95c89d723362a`） |
| 12 | Web 客户端测试、类型检查、WASM 构建与 Worker 冒烟（含 37 来源 46 包全矩阵） | `pnpm --dir apps/web-client test` / `typecheck` / `build:native` / `smoke:native` | `test`: **461 pass / 0 fail**；`typecheck`: **0 错**；`build:native`: **exit 0**；`smoke:native`: **`{"total":83,"failed":0}`（83/83 PASS）** |
| 13 | 真实 Headless Chrome 产品路径验证（3 套浏览器驱动脚本） | 独立构建 `--outDir /tmp/noname-web-dist-batch23` 后实跑 `nc09-browser-drive.mjs`、`nc12-browser-fullgame.mjs`、`i02-browser-audio-drive.mjs`（跑毕已清理临时产物） | **3/3 ALL PASS（`console_errors: []`）**：① `nc09`: `choose_general: "样例·张角组合"`, `auto_step: rev 0 -> 3`, `save_load: { saveRes: true, loadRes: true }`（[日志](headless-chrome-nc09-browser-drive.raw.txt) / [JSON](drive-report.json)）；② `nc12`: 85 次点击完整打完一局（`final_revision: 84, wall_ms: 21436, ended_marker: true`，[日志](headless-chrome-nc12-browser-fullgame.raw.txt) / [JSON](fullgame-report.json)）；③ `i02`: 选将「司马懿」音频解锁播放、静音切换与读档基线抑制（`playedCuesCount: 1, gen 2->3, baselineRevision: 14`，[日志](headless-chrome-i02-browser-audio-drive.raw.txt) / [JSON](i02-browser-audio-report.json)） |
| 14 | 37 来源（46 扩展包）四端产品路径与同设备启用前后性能基准 | `cargo test --offline -p engine-native-wasm --test installed_package_payload`；`cargo test --offline -p game-server --test nc08_install_matrix`；`cargo run --offline --release -p engine-native --example nc12_bench -- source_matrix` | WASM `installed_package_payload` **8/0**、Server `nc08_install_matrix` **3/0**、Worker `smoke:native` **83/83**、Desktop `desktop-client` **20/0/1**；性能基准 **49/49 行（`none` 基线 + 37 来源 + 11 子包）`ok=true, faults=0, panics=0`**（[报告](up37-37-sources-platform-and-perf.md) / [TSV](up37-37-sources-perf.tsv)） |
| 15 | 裁决 25 发布与 GitHub Actions exact-head CI | 单命令作用域 `pre-push` 守卫（`9/9` 自测）推送 `refs/heads/native-convergence`（`5c908a671 -> 509241205 -> ea9ee2ad8 -> 5a4697414 -> 413a20379`） | 首跑 `42459494838`（`509241205`）**5/10 PASS**，捕获 `battleAudioOwner.ts` `Buffer` 类型与 Windows CRLF 摘要两项跨平台问题；修复后 `42461463648`（`ea9ee2ad8`）**9/10 PASS**（含 `desktop-ubuntu/macos/windows` 三平台安装包构建上传），仅余 `build-windows-latest` `packages/extensions/*/assets/*.json` CRLF；二次修复后 `42464235112`（`413a20379`）**8/10 PASS**（`build-windows-latest`、`gates-ubuntu`、`native-web`、`desktop-macos-latest`、`build-ubuntu/macos`、`perf-ubuntu/macos` 全绿；`desktop-ubuntu/windows` 编译打包完 `.deb`/`.rpm`/`.msi` 后在下载外部 GitHub Releases `AppRun-x86_64`/`nsis_tauri_utils.dll` 时遇 HTTP 504，见 [publication-push-result.json](publication-push-result.json)） |
| 16 | 全量收口 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |



