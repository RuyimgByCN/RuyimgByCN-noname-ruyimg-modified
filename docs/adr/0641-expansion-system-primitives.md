# ADR 0641：expansion（武将牌上置牌）系统原语面立项

- 状态：**已接受（评审 2026-08-15，goal-verifier 指令）**
- 日期：2026-08-15
- 背景：noop 机制族普查（`docs/migration/noop-mechanism-survey.md`）显示 expansion 族
  解锁 165 技、居首；clan README 原语缺口表早有同名判断（成本最高/解锁最多）。
- 关联：ADR 0597（保真度账本）、batch 2026-08-10（storage_get/set/card_has_tag/
  tag_event_cards 全链路）、本会话 batch AQ（执行点包裹模式）。

## 1. 问题

上游「置于武将牌上」（expansion/标记牌/技能牌堆，如 clanjiyun 的「粮」、clanguangu 联动、
sp2/biaozhao2 等 165 技）在 DSL 中无完整表达：牌可进（已有 skill_pile 雏形）但缺
**取用/查看/按属性查询/展示**与**事件联动**。

## 2. 底座现状（全部已在生产，非新建）

| 底座 | 位置 | 能力 |
|---|---|---|
| `skill_pile_cards(player, skill_def)` + `QueryRequest::SkillPileCount` | engine-core state/query | 按技能置牌/计数（SkillPileCount DSL 查询已通） |
| `content_storage`（键控 u32） | SkillInstance | storage_get/set（batch 08-10） |
| `content_card_tags: BTreeMap<CardId, BTreeSet<u32>>` + `card_has_content_tag` | GameState | 牌上标记（gaintag 对应物） |
| `tag_event_cards` / `owner_alive_tagged_card_lost_this_turn` | batch 08-10 | 事件牌打标/失标查询 |

## 3. 原语面（分三阶段，每阶段独立领号、独立批次）

- **P1 取用与查询（预计解锁 ~60 技）**
  - `expansion_gain_card(player, self_skill)`：从自身 expansion 取一张入手（选择续体）
  - 查询 `expansion_count(player, skill_def)`、`expansion_has_name(player, skill_def, card_name)`
- **P2 置入与转移（~50 技）**
  - `place_hand_card_to_expansion(player, self_skill)`（选一张手牌置入）
  - `expansion_to_equip/置入装备区` 与 expansion 间转移（复用 ADR 中 choose 双段模式）
- **P3 事件联动（~55 技）**
  - 触发过滤器：`expansion_gained`/`expansion_lost` 事件 + 既有 card_tags 联动
  - 「expansion 中的牌被使用/获得时」历史快照（对齐 TriggerHistorySnapshot 模式）

切片内每批仍守「1 原语 + 3-5 技能」：P1 首批建议 sp2/mubing（粮）+ clanjiyun（粮）+
sp/tongji 族（按普查明细 noop-mechanism-clusters.json）。

## 4. ABI 与治理

- 每个新 Effect 原语独立领号（当前 ABI-MAX 649，预计 P1-P3 共 4-6 个号）；
  过滤器类走字符串 5 处注册（零 ABI）。
- 序列化兼容：skill_pile/content_card_tags/content_storage 均已在 GameState/SkillInstance
  序列化内，无回放风险；新查询只读。

## 5. 风险与开放问题

1. expansion 牌的可见性（上游默认公开/可隐藏）——P1 先公开口径，隐藏视图留 P3。
2. 「按牌名查询」需 card_name 注册表对齐（card_name_length 先例可复用）。
3. 165 技中约 1/3 与判定改写/历史族复合（普查交叉），本 ADR 只承诺其中
   「expansion 为唯一阻塞」的子集，交叉技能由对应族 ADR 覆盖。

## 6b. §5.3 唯一阻塞子集审计结果（2026-08-15 执行，修正 §6a）

按本 ADR §5.3 承诺执行的审计发现：名义 22 个「expansion 唯一阻塞」候选中，**17 个的
noop 文件头是占位垃圾**（「待补充武将名」「刷新包技能」——旧批量批次未写描述），须
逐个核对上游体后才能定性；**有真实描述仅 3 个**：twmingren（摸牌置入）、twqingce
（移去「荣」）、ymfriendfangqiu（展示牌）——其中 2 个在 tw（并行会话作业面）。
**安全包内达标的候选 <3，按批次门槛纪律 P1 铸造暂缓。**

§6a 工单作废部分：mubing 经核不是 expansion 技（观牌堆顶+点数换牌，普查关键词噪声）、
tongji 属复合阻塞——首批候选来自噪声数据，已撤。

### §6b 续：占位头重记批次已执行（17:1x）+ 审计重跑

19 个占位头已按上游 translate.js/skill.js 权威描述重记（6 个无 info 条目以名称+指引
占位）。重跑唯一阻塞审计：**6 候选，其中安全包 4 个**（xianding/xinzhoufu2、
refresh/ollihuo、onlyOL/olchunlao、offline/ymfriendfangqiu；tw×2 属并行会话）——
**铸造门槛（≥3）已达标**。

**顺序修正（实测）**：4 个候选的共同阻塞是「置入」（首张杀置于武将牌上/扣置手牌/
置酒），而非 §3-P1 的「取用」——**P2 置入必须先于 P1 铸造**（无置入则取用无对象）。
首个原语应为 `place_card_to_expansion(player, self_skill[, zone])`（选牌续体或
event_card 直置两种形态），领号仍按 §6a（tag 669/ABI 650 顺延）；注册模板与踩坑
清单不变。

### §6c：P2/P3 变体审计（17:3x）

gain-to-pile 与跨技能堆读取两变体经全仓 noop 描述审计均为 **0 命中**（olchunlao/
ymfriendfangqiu 属个例非族）——两变体从待铸清单移除，待出现 ≥3 同构需求再立项。

### §6d：异构迷你批章程（判定交互，2026-08-15 17:4x 立项）

门槛降级依据：四联审计后无同构族达标，改以「2-3 技能迷你批」推进异构小机制。

**首批：替换判定牌**（消费者 key/shiroha_jiezhao 4008645358「预兆」区替换判定牌；
备选 key/misuzu_zhongyuan 终愿「修改判定结果花色点数」——同判定交互面）。

**执行结果（17:5x）：铸造解体为零原语**——侦查发现判定改写系统引擎原生完备
（RewriteJudgment 续体按座次询问、选项含 ZoneKind::Expansion、`judgment_rewrite`
等四种 modifier 目标已暴露 DSL，前会话为鬼道系建造）。两技以
`modifier judgment_rewrite override 1` 直接落地（近似注记在 .skill 头：解兆未限预兆
区、终愿花色点数受限于所持牌/限定技未建模）。§6d 的 ReplaceJudgmentCardFromSkillPile
铸造取消——引擎已覆盖。

已验证事实（本会话）：
- `crate::command::judge::flip_judgment(tx, judged, JudgmentFavor)` 为独立判定入口，
  返回 `JudgmentResult{card, suit, rank}`（pack_mobile.rs releijie 是消费样板）；
- 独立判定后以 `EmitJudgmentResolved` 收尾（RuleNote code 236 先例）；
- **未验证（铸造前必查）**：进行中判定的续体形态（judging window 中判定牌存放于
  何处、何种 ContinuationKind），替换牌如何回流判定求值——这是本原语与 flip 的差异点。

原语草案：`ReplaceJudgmentCardFromSkillPile(player, self_skill)`（触发于判定窗口内）：
选自身技能堆一张牌替换当前判定牌，被换牌入弃牌堆。注册照 §6a 15 处模板（tag 670+
顺延 / ABI 651 / Cont 270）。

第二迷你批候选：牌堆底判定（key/key_huanjie 幻界，摸/判均改牌堆底——需 deck-bottom
draw/judge 双变体，规模 1-2 技能，暂缓）。

### 修正后的前置任务（P1 铸造前必做）

1. **占位头重记**：17 个垃圾头 noop 逐个读上游体重写描述（本身是一次独立小批次，
   顺带可能捞出反向扫描漏网）。
2. 重跑唯一阻塞审计（数据干净后）；若安全包内 ≥3 达标，按 §6a 注册清单执行
   （注册模板与踩坑清单仍然有效）。

## 6a. P1 首批工单（turnkey，供实施会话直接执行；候选集见 §6b 修正）

1. 领号：`EXPANSION_GAIN_CARD`（tag 669 / ABI 650，纯同步若取牌无选择；若带选择续体则
   ContinuationKind 269）+ 查询 `expansion_count(player, skill_def)`（零 ABI，走
   QueryRequest 新变体或复用 SkillPileCount 语义核对后可能零新增）。
2. 注册：照 batch AQ 模板 15 处（effect.rs 枚举尾/bytecode/vm/ir/typecheck self_skill
   门禁/codegen+import/lib 阶梯/primitives 实现+分发/resolve 分发+resolver/provenance
   ×2/hash tag 154 复用需新 payload 形状/lsp 单行/GameState 构造——本会话记忆
   upstream-sync-workflow.md 第 18/20 条含全部踩坑）。
3. 首批技能（各核对上游体后写 DSL + 近似注记）：sp2/mubing（粮置入取用）、
   clandongxu 关联的 clanjiyun（粮，08-10 已近似——按 P1 原语升级为真实取用）、
   sp/tongji 族 ≥1 技（sp 属并行会话作业面，先以 sp2/xianding 侧同类替换）。
4. 测试：契约（npack trigger/filter/ABI 650）+ 行为（置牌→取牌入手牌计数）；
   冻结断言同步（两包计数 + 任何包级 ABI 断言）。

## 6. 验收

- P1 落地标准：≥3 技真实 DSL + 行为测试 + 全仓绿；expansion 计数查询进 npack 契约。

## 7. P1 执行记录（2026-08-15）

第八轮实面审计后 P1 取用安全包内实面 2（dchaoshi/ao_diegui），用户批准门槛 3→2，
按 §6a 模板执行完毕：

- 原语 `gain_from_skill_pile`（GainFromSkillPile，tag 671 / ABI 651 / Cont 270，选择
  续体；与 §6a 预估的「EXPANSION_GAIN_CARD tag 669」不同号——669/670 已被 P2 置入与
  判定替换占用，P1 顺延领号）。
- 首批取 key/collab 安全包两技（§6a 候选 mubing/clanjiyun 属并行会话包面，未动）；
  契约 7/7、全仓 exit=0；行为链验证与 P2 ollihuo 同口径留待（见 delta 末节）。
- 执行坑补记：resolver 引用 skill_pile_def/move_cards 须走 `crate::effect::` re-export
  （primitives 模块私有）；lsp doc 必须单行字面量。

## 8. 章程推进：全量消费者普查与 S1-A 执行（2026-09-09 第一百八十九批）

上游增量窗口关闭后对本 ADR 的全面推进（全新普查取代 §6b 的 22 件口径）：

### 8.1 全量消费者普查（按上游体关键词，非注记）

- **addToExpansion 系：249 技**；**getExpansions 系：80 技**（单技可属多面）。
- **计数门控触发面（getExpansions(...).length 出现于 trigger filter）：95 技**。
- 阻塞面分类：

| 面 | 数量 | 代表 | 现状 |
|---|---|---|---|
| A0 自手牌→expansion | 157 | clanbeishi、dcbenxi | P2 原语已覆盖置入；多数卡在取用/联动面 |
| B1 计数/查询门控 | 76 | nsfuwei、ao_diegui | `skill_pile_count` 查询已通，配合通用效果即可复活 |
| A3 expansion→使用/获得混合 | 64 | dchaoshi、oldingxi | 部分被 P1 覆盖；混合面依赖 S1-B |
| A1 置入**他人**expansion | 19 | nsjiquan、rekongsheng | 变体原语待立（>3 门槛，P2-var） |
| A2 牌堆/弃牌→expansion | 9 | jsrgshacheng、shiroha_yuzhao | 变体原语待立（>3 门槛，P2-var） |
| B0 其他引用 | 4 | bushi | 个例 |

### 8.2 切片设计（取代 §3 三阶段）

- **S1-A（零新原语，本轮执行）**：计数门控 + 通用效果（觉醒/摸牌/授技）的 3 件——
  moucuan（mobile 谋篡）、xinzili（refresh 自立）、tianxing（onlyOL 天行）。
  恰达 3/3 门槛；置入供给未落地前为 dormant 契约（近似注记）。
- **S1-B（下一铸造批，63 件消费）**：`discard_skill_pile(player, self_skill)`——
  自身技能堆整体移入弃牌堆（rebiaozhao3 型「准备阶段弃全部→令目标摸3回1」的
  前置面）；复用 P1/P2 的 15 处注册模板；纯同步、预计零选择续体。63 件远超门槛。
- **S2（P2-var，达门槛）**：置入方向/来源变体——A1 他人 19 件、A2 牌堆/弃牌 9 件；
  需给 place 原语加 (target, source_zone) 参数化或新变体，领号 2 个。
- **S3（P3 事件联动）**：expansion_gained/lost 触发过滤器（字符串 5 点注册零 ABI）+
  TriggerHistorySnapshot 扩展；待 S1-B 落地后按实际消费者重测门槛。

### 8.3 S1-A 执行记录

- 三技 DSL + 行为测试（`original_s1a_expansion_awaken` 3 用例）：计数门控、once
  标记、四选一授技（choose(1,2,3,4) + if == 链）。
- **引擎事实补记**：①DSL BinOp 仅 `+ - * == < > &&`，无 `>=`/`!=`——`A>=B` 写作
  `B < A+1` 或双过滤器拆臂；②`remove_skill` 收存活实例表达式、`remove_skill_by_def`
  收 def 字面量；③forced 触发模块在 dispatch 期内联执行，体内 choose 使
  **dispatch 本身**返回 Err(ChoiceRequested)；④P2 置入续体为 ollihuo 形态
  （限「当前手牌【杀】」）——**P2 泛化（pile 卡种/来源参数化）并入 S2 工单**；
  ⑤测试可直接构造 SkillPileRef+Expansion zone 驱动计数门控。
- 包面：mobile/refresh/onlyOL 各 +1 触发模块 −1 noop modifier，ABI 均不变
  （652/681/653）；三处副本字节一致。

### §8.4 S1-B 首铸记录（2026-09-09 第一百九十批）与集成发现

- **原语落地**：`discard_skill_pile(player, self_skill, pile_def)`（tag 707 /
  ABI 684 / 纯同步零选择续体）——pile_def 为 u32 字面量（可指父技/兄弟技的堆，
  批 166 tanhu2 教训的参数化正用）；hash 复用 tag 154、盐 DSKP（3×u32 形状）；
  host 已 bump 684（领号先行）。直测通过：直驱 run_effects 正确清空堆入弃牌堆。
- **首件消费者 rebiaozhao3 集成受阻 → 回退诚实 noop**：触发体内「纯效果（弃堆）
  + 选择链（choose_other_draw_or_recover）」交错时，弃堆请求经 vm-pause
  defer 路径后**丢失**（探针证实原语可达但执行时堆读数为空/终态未清）。
  涉及 dispatch 2308-2340 的 paused/outcome.effects 语义与 resolve_choice 的
  deferred 重放边界——需独立定界批（§8.5）。
- **顺带产出**：参数化过滤器 `OwnerSkillPileNonempty(u32)`
  （`owner_skill_pile_nonempty_<def>`，5 点注册零 ABI）——S1-B 全族门控通用，
  已在册；上方误置的 rebiaozhao3 触发随回退移除，过滤器保留待集成批复用。

### §8.5 S1-B 集成前置定界批（下一批入口）

1. 定界「forced 触发模块：pre-choice 纯效果 × vm-pause × deferred_effects 重放」
   的确切语义（qinxue 型单选择链正常、discard+choose 型丢失——差异点在
   pause 时机与 effects 归属）。
2. 定界后再迁移 rebiaozhao3 及 63 件消费池（每件核对 pile_def 归属与选择链形状）。

### §8.5 定界批结果（2026-09-09 第一百九十一批）

最小化实证（`original_s1b_discard_skill_pile` 7 用例，4 ignore 编码已知缺陷）：

- **run_effects 直驱路径正确**：`run_effects([DiscardSkillPile,
  ChooseOtherDrawOrRecover])` 按序应用——弃堆先生效、选择后开 ✓（原语与排序
  语义均无缺陷）。
- **触发 marker 重放路径丢失**：同一模块经「optional 确认 → deferred 重放 →
  marker 执行」后，pre-choice 的弃堆请求**不落地**（A/B/C 三实验：弃堆前/后
  两种排序 + 单请求拆分，全部 pile 保持不变）。且测试发现「选择链在
  SelectTargets 解决后即闭合」与生产管线的两段式行为不一致——**测试直驱
  （dispatch_for_events + apply_command ResolveChoice）与生产命令管线不等价**，
  缺失的正是 vm-cont 恢复的管线内机制（`run_module_with_selected_inputs` 一族）。
- **结论与入口**：S1-B 消费者集成需要①先以生产管线（真实 PlayCard/Activate
  命令流）复测同一模块，判定丢失是测试驱动伪影还是引擎真实缺陷；②若为真实
  缺陷，修点在 dispatch inline 执行的 pause 分支（dispatch.rs 2326-2340）与
  resolve_choice 尾部 resume 的衔接。定界用例已固化为 ignore 测试（修复后
  取消忽略即为回归网）。
- 顺带：`owner_skill_pile_nonempty_<def>` 参数化过滤器（5 点零 ABI）与
  `discard_skill_pile` 原语（ABI 684）保持在册；rebiaozhao3 维持诚实 noop。

### §8.6 S1-B 消费池收口普查（2026-09-09 第一百九十七批，定界收口批，无代码）

对剩余 120 件「loseToDiscardpile × getExpansions」家族成员做全量面分解
（已迁 5 件除外），按缺失面分布：

| 主缺失面 | 件数 | 说明 |
|---|---|---|
| 置入面（另技 addToExpansion/置堆） | 34 | 与 A1/A2 变体同源——**主导缺口** |
| storage + 置入面 | 17 | 叠加 storage 联动 |
| 置入面 + 选目标 | 10 | 选目标已可表达（choose_other 家族） |
| storage + 置入面 + 选目标 | 7 | |
| gain-all-pile→hand 及组合 | ~14 | 堆整取入手原语缺口 |
| pile 内单/多卡选弃 | ~9 | 选弃原语缺口 |
| 目标伤害（可表达）为唯一缺口 | 3 | **已可迁移**（choose_other_damage） |
| 其他复合（storage/history/gaintag 等） | ~9 | |

- **结论修正**：`discard_skill_pile` 单原语的直接消费面（「门控 + 整弃 + 简单
  效果」纯净形）在批次 193–196 已迁移 5 件后**清零**；剩余家族的主导缺口是
  **置入面（placement）**——即 §6c 曾判定 0 命中的 A1/A2 变体在全量普查下
  实为 34+17+10+7 ≈ **68 件触达**，构成下一铸造批的最高经济性目标
  （`place_card_to_expansion` 参数化变体：source_zone + 目标堆，预计领号 1-2 个）。
- **可迁移零门槛残件**：「目标伤害（可表达）为唯一缺口」3 件
  （offline/zypaiyi、onlyOL/olpaiyi、refresh/gzpaiyi + sp/xinfu_fangtong、
  xianding/dcxingwu 等 choose_other_damage 覆盖件）留作后续散批。
- 次级缺口排序：gain-all-pile→hand（~14）、pile 内单/多卡选弃（~9）、
  弃牌事件过滤器（biaozhao2 型）。

### §8.7 下一铸造批排程（更新 §8.2）

1. **P2-var 置入面参数化**（~68 件触达）：`place_card_to_expansion(player,
   self_skill, pile_def[, source_zone])` 泛化变体，P2 既有 650 号臂的参数化
   重构或新领号。
2. **gain-all-pile→hand**（~14 件）：`gain_whole_skill_pile` 或
   `gain_from_skill_pile` 的 all 变体。
3. **pile 内单卡选弃**（~9 件）：`discard_one_from_skill_pile`。
4. 选目标/目标伤害件为纯零门槛散批，随批次穿插消化。

### §8.5 补记：缺陷修复验证（2026-09-09 第一百九十八批）

批 193 的 vm 弹栈位型保留修复（pop 去 `.max(0)` 钳位）**同时治愈生产管线
路径**：`production_pipeline_endphase_reaches_prepare_and_drains_pile`
（EndPhase 命令流转 → 触发发射 → 确认 → 目标选择 → 摸 3 → 弃堆）全链转绿。
§8.4 的「vm-resume 丢请求」定性修正为：**请求从未丢失，丢失的是 pile_def
字面量的位型**（钳位归零后弃堆指向空堆 0 而静默 no-op）。4 个 ignore 定界
用例已全部转正为常规回归网。S1-B 解锁，rebiaozhao3 已晋升（批 193）。
P2-var 置入面铸造批（§8.7，~68 件触达）为下一专项批次。

### §8.7 进度补记（2026-09-10 第二百零三批，审计收口批，无代码）

- **choose_one_from_skill_pile_gain 判定为冗余**：P1 的
  `gain_from_skill_pile(player, self_skill)`（ABI 651/Cont 270）已实现
  「从自身技能堆选一张入手」——与拟议的 gain 变体语义重合，不再铸造。
- **vm_host.rs 同型扫描**：`.max(0)` 钳位 0 命中；effect/mod.rs 同样 0。
  def-id 钳位风险在批 193/194 两轮修复后清偿完毕（vm.rs 内 18 臂 35 处
  位型保留 + mark_count 计数保留的口径不变）。
- S1 家族回归网（s1a/s1b/pile_discard_family 三文件 11 用例）全绿。

### §8.7 补记：四原语消费面收敛确认（2026-09-10 第二百零九批，定界收口批，无代码）

四原语（place_top_deck_to_expansion / discard_skill_pile / choose_one_from_
skill_pile_discard / gain_whole_skill_pile / choose_target_hand_card_to_pile）
铸造后对消费面做最终扫描：

- **deck-top 纯置入族**：仅剩 xianding/dcxialei（92L，多事件门控 + mark 计数
  分支）——需复合面，维持债。
- **gain-all 家族**：remoshi（65L，gaintag+装备区联动）、jieyue（138L，
  chooseCardTarget 双选）、xinyingshi（114L，多角色武将牌置入）——均需复合面，
  维持债。
- **结论**：四原语可直达的纯形状消费面已全部消化。剩余 expansion 家族成员
  每件需 ≥1 个组合面（gaintag 标记族 / history 查询 / storage 联动 / 复合
  选择 UI），按 ADR 0641 §8.6 维持登记债，不再有零门槛散批。
- 后续推进入口收敛为两个专项：①chooseToMove 堆重排原语（预兆·里）；
  ②复合选择 UI 原语族。均为多批次引擎工程。

### §8.8 chooseToMove 消费族普查（2026-09-10 第二百一十批，定界批，无代码）

全仓扫描「chooseToMove」关键词：**41 件消费**（覆盖 sp/sp2/tw/xianding/key/
mobile/offline/onlyOL/refresh/jsrg 十包），远超 3/3 铸造门槛——chooseToMove
（选 N 张手牌置牌堆顶可重排）为**全仓最高优先级单面铸造目标**。

- **原语设计草案**：`choose_hand_cards_to_deck_top(player, self_skill, count)`
  ——选 count 张手牌，按所选顺序置牌堆顶（两段选择续体：SelectCards count 张
  + 顺序确认），tag 712 / ABI 689 领号。
- **下游连锁**：铸造后可直接解锁预兆·里（key/shiroha_yuzhao_umi）、观星变体
  （sb/sbguanxing）、经纬（xianding/dcsbchengce）等 41 件触达面。
- **实施风险**：chooseToMove 为多卡选择+顺序确认复合 UI，需两段以上续体
  与顺序编码——建议独立专项会话实施（预留 2-3 批工作量）。
- **§8.6 修正**：置入面（~68 件）与 chooseToMove（41 件）为并行的两大
  铸造目标；其余面（give-pile-to-target、gain-all-pile→hand 已由批 204
  原语部分覆盖）维持登记债。

### §8.9 chooseToMove 堆重排原语铸造收口（2026-09-10 第二百一十四批）

`rearrange_pile_to_deck_top`（tag 713 / ABI 690 / Cont 296）全链路落位：15 处
注册模板齐备（effect.rs 变体 / bytecode / vm 弹栈〔pile_def 位保 `as u32`〕/
ir / typecheck / codegen / lib ABI 常量 / lsp / hash RPTD / engine fn / dispatch /
Cont 路由 / group_post_8 续体 / lsp 文档 / version.rs host bump 690）。语义：
从自身 (player, pile_def) 技能堆选 min(count, 堆数) 张，按所选顺序依次压入
牌堆顶（最后所选最靠顶，与 place_hand_cards_to_deck_top 同序）；空堆/亡者
no-op；选择越界（非堆内卡）Resolve 拒绝。

- **契约测试**：`original_rearrange_pile_to_deck_top.rs` 4 件（选序落位 /
  count 钳位 / 空堆 no-op / 外来卡拒绝）全过——合成 probe 技能直构（§8.8
  消费者均复合面，dormant 契约）。
- **§8.8 普查结案**：41 件 chooseToMove 触达逐一复核——手牌置顶族已由
  place_hand_cards_to_deck_top（ABI 689，批 211）覆盖（预兆·里批 213 收口）；
  堆→顶族（sbguanxing 观星变体、jsrgyingshi 鹰眎、twguimou 鬼谋、rezongxuan
  纵玄等）逐一复核均为复合面：标记计数门控（sbguanxingx）+ 旧堆弃置 +
  temp 技能门控 / 观看牌堆底 + 花色判定 / 弃牌堆来源（非技能堆）——按 §8.6
  维持登记债。**chooseToMove 单面铸造目标就此关账**：原语面（689 手牌 +
  690 技能堆）已齐备，剩余消费全部归入复合选择 UI 原语族专项。

### §8.10 store 置入腿死缺陷修复与 quanji 家族晋升（2026-09-10 第二百一十五批）

**缺陷**：`store_selected_in_skill_pile(player)` 为续体式置入原语（cards 由
前置选择供给），但 DSL 直排语句没有前置选择阶段——`host.selected_cards()`
恒空，置入腿在全部 7 个已部署消费件（old_jijun / yijiang kuangbi·quanji·
jieyue·chunlao·xiansi / shiji jishi）中**静默无效**（堆恒空、手牌上限源恒 0）。
探针实测定性后修复：cards 空时原语自开 SelectCards 1/1（必选、当前手牌、
不可取消），新续体 `StoreSelectedInSkillPile = 297`（resolver 位型校验 +
ensure_skill_pile_zone 落堆）。零字节码：tag/ABI/包 manifest 均不变（包
required ABI ≤ host 690 恒成立），已部署件 npack 零改动即复活。

**晋升**（quanji 家族，yijiang/quanji 模板）：
- **requanji**（mobile，noop→approx）：受伤臂 + 出牌阶段结束臂
  （body 级 `if hand_count(self) > hp(self)` 门控——`owner_hand_gt_hp` 仅
  view_as 域可用、非触发过滤器）+ hand_limit add owner_skill_pile_count。
- **gzquanji**（refresh，noop→approx）：受伤/造成双臂
  （`owner_is_target_once` / `owner_source_once_per_turn`，现成过滤器零注册）
  + hand_limit 源。「权」堆 = 本技自身堆 3651179843——**gzpaiyi（批 202）
  的 dormant 供给面就此闭合**。
- 债（两件同款）：getIndex 逐点拆分未建模；frequent 锁定为 optional 确认；
  gzquanji 每分支限一次收窄为 skill 级共享 turn_fire_count（双臂合计每回合
  一次，保守收窄）；gzquanji "he" 选牌域收窄为手牌；onremove 清堆未建模。

**契约测试**：`original_quanji_family_store_revival.rs` 3 件（requanji 双臂
契约+全链、gzquanji once 闸、yijiang quanji 复活+hand_limit 联动）。
**ledger**：complete=5065 / approx=530 / todo=0 / noop=2092（生成器重跑）。

### §8.11 A1/gain-all/A0 三族读体定界与 A0 置入面散批（2026-09-10 第二百一十六批）

按批指命对 A1 他人置入族 / gain-all 族逐件读体定界，兼查 A0 自手牌置入池：

- **A1 他人置入族（19 件）定界**：rekongsheng（扣置于其武将牌+用牌回收+死亡
  清算=复合）、twzhengrong/rezhengrong（每阶段第奇/偶数张用牌历史追踪）、
  starchengfeng（主公技+势力分支双源）、twchunlao（选他人牌置入**其**堆=
  方向相反+group 联动）——**无可表达残件**，维持复合登记债。zhengrong（神
  版）已由 ABI 545 专属 host fn 覆盖。
- **gain-all 族（remoshi/jieyue/xinyingshi）复验**：remoshi 已实现（locked 双
  触发 resolve_remoshi）、jieyue 已 approx（交互分支债在册）、xinyingshi
  （phaseUseBegin+damageSource+die 三事件+额外回合）维持债。无新增可表达件。
- **A0 自手牌置入池（批 215 store 修复解锁）散批 2 件**：37 件形状普查，
  晋升 **twxingwu 星舞**（tw：弃牌阶段开始置入一张手牌；消耗面≥3张移去+
  弃装备+性别伤害登记债）与 **biaozhao 表召主段**（sp2：结束阶段堆空门控
  置入一张手牌；biaozhao2/3 同点数消耗面登记债）——quanji 模板复用
  （phase 触发 optional + store_selected_in_skill_pile，批 215 Cont 297）。
  sp2 包 required ABI 562→565（trigger 体 if+store 组合所致，≤host 690），
  tw 包 684 不变。
- 其余 35 件 A0 候选分诊：变张置入（tunchu/xinfu_zhaoxin/xinqingjian）、
  牌堆顶来源（duwang/twshenyi/mbsuwang/mbfutu）、他人方向（twshijun/spdaizui/
  scschiyan）、条件授予（twpingting/jueyue）、回浪型多面（tsumugi_huilang）、
  给出型（mbsuwang/zhengjing）等——均需 ≥1 个未铸面，维持登记债。
- **契约测试**：`original_a0_store_placement_batch.rs` 2 件（twxingwu 连续
  置入、biaozhao 堆空门控二回合拦截）。
- **ledger**：complete=5063 / approx=532 / todo=0 / noop=2090（-2 noop =
  twxingwu/biaozhao）。

### §8.12 变张置入原语铸造（2026-09-10 第二百一十七批）

`choose_hand_cards_to_skill_pile(player, self_skill, min_count, max_count)`
（tag 714 / ABI 691 / Cont 298）15 处注册落位。语义：从自身手牌选
[min_count, max_count] 张置入自身技能堆；可取消=不置入（上游 chooseCard
bool=false 合法路径）；上下限以当前手牌数钳位；max==0 或空手 no-op；
取消选择不视为上下限违约。hash 盐 CHSP 4×u32。

- **消费 3/3 达门槛**：tunchu 屯储（mobile：摸牌阶段无粮门控 → 摸 2 +
  [1,手牌数] 置入；禁杀 mod 面与置入时点登记债）、ymfriendyance（offline：
  每轮开始摸 5 + 恰好 5 张置入 min=max=5；useCard 消耗面与本轮归还登记债）、
  tsumugi_huilang 回浪（key：回合结束 [1,N] 置入；he 域收窄 + 消耗面债——
  回浪2 本体（批 204）供给链闭合）。
- **断言同步经验**：本批波及 mobile/offline/key 三包（ABI 687/686/689→691），
  涉及 50+ 测试文件的 ABI 与计数断言。陷阱：①目录式 sources()（read_dir）
  的 compiled ABI 断言随新 .skill 落盘联动上浮；②单包文件粗粒度计数归一会
  误伤同文件 compiled 小包计数断言（akiko 2/liewu 2/yyxuezhan 2/quhu 3）——
  归一脚本必须限定「npack 绑定变量」且同文件多包时逐变量判定。
- **契约测试**：`original_variable_placement_batch.rs` 4 件（取消路径、
  双张置入、tunchu/ymfriendyance/huilang 消费链）。
- **ledger**：complete=5060 / approx=535 / todo=0 / noop=2087（-3 noop）。

### §8.13 移去消耗面定界关账与 shuliang 晋升（2026-09-10 第二百一十八批，零铸造散批）

按批指命评估「部分移去原语」铸造门槛，逐件读体后**判定不达 3/3、零铸造**：

- **单张移除已有原语**：`choose_one_from_skill_pile_discard`（ABI 686）完整
  覆盖移去一张形状，无需新原语。
- **多张移去消费者全数复合**：twxingwu（移去三张+弃装备+性别伤害——半面实现
  = 无收益自毁星舞，判定为陷阱面维持债）、sbyingmen 盈门（武将牌堆来源）、
  pelishi 立世（等量多目标+阶段跳过链）、tsumugi_huilang2（移去赋使用牌效果）。
- **晋升 1 件**：**shuliang 输粮**（mobile noop→approx）——任一角色结束阶段 +
  粮堆（tunchu 堆 3943350142，跨技 def 字面量）非空 + 事件角色手牌<体力 →
  ABI 686 选堆单卡弃置 + draw_cards(event_player, 2)。**触发器语义发现**：
  `owner_skill_pile_nonempty_<def>` 过滤器 eval 含 `owner_is_player`（己方回合
  限定），全场阶段触发须用 `owner_alive` + body 级组合门控。
- **债注修正**：biaozhao 消耗面注记更正（上游为「摸至全场最高手牌数≤5+回复1」，
  非旧注「摸三」；全场手牌上限查询不可表达维持债）。
- **twqingce 清侧维持债**：第二腿弃置目标区域牌需 active 目标引用表达式
  （`choose_discard_target_card` 需显式 target expr，active 选中目标无引用
  机制）——与复合选择 UI 专项合并。
- **契约测试**：`original_shuliang_cross_pile.rs` 3 件（跨技堆移除+event_player
  摸牌、他人回合触发、hand≥hp 门控拦截）。
- **ledger**：complete=5059 / approx=536 / todo=0 / noop=2086（-1 noop）。

### §8.14 A0 池变张置入复测（2026-09-10 第二百一十九批，零铸造散批）

以批 217 变张置入原语（691）与批 215 store 修复对 A0 池剩余件复测：

- **晋升 2 件**：**rebiaozhao 表召**（mobile noop→approx，主面与 biaozhao SP
  完全同形：phase_end + 堆空门控 + store；同点数面与准备阶段摸至场最高面
  维持债）与 **twmingren 明任**（tw noop→approx，game_started forced → 摸 1 +
  必选置 1；置换面/enterGame 时点维持债）。
- **复测维持债（读体证据）**：xinqingjian（gainAfter 非额定摸牌分类触发——
  DSL 无获得分类事件）、twyouye/mbyouye（按对手本回合伤害历史过滤——
  hasHistory("sourceDamage") 按对查询无对应过滤器/表达式）、mbsuwang
  （useCard 历史）、duwang（牌堆顶按牌种筛选）、friendyangming（阶段失去
  历史+用牌链）。A0 池剩余件全部需 ≥1 个历史/分类复合面，**单面复测
  潜力清零**。
- **契约测试**：`original_a0_retest_batch.rs` 2 件（rebiaozhao 堆空门控二次
  拦截、twmingren forced 全链）。
- **ledger**：以生成器输出为准（本批 -2 noop）。

### §8.15 历史查询过滤器铸造与 twyouye/mbyouye/mbsuwang 晋升（2026-09-10 第二百二十批）

复合选择 UI 专项的历史查询缺口首切片：引擎回合级历史面
（`damage_pair_count(source,target)` / `card_target_count(user,target)` /
`damage_taken_count(player)`）已存在，据此铸两件零 ABI 过滤器（5 点注册 +
registry 跟踪 opt-in）：

- `owner_alive_other_phase_end_not_damaged_owner_this_turn`——伤害对计数
  （跟踪面 ABI≥353 全开，无需 opt-in）；
- `owner_alive_other_phase_end_used_card_on_owner_undamaged_this_turn`——
  用牌目标计数（**需 `track_turn_card_target_counts` 过滤器 opt-in**，danshou
  先例）+ owner 本回合未受伤（跟踪面 ABI≥347 全开）。

**晋升 3 件**（noop→approx，主面 = place_top_deck_to_expansion ABI 685）：
twyouye 攸业（tw：堆<5 body 门控；分配面债）、mbyouye 攸业（mobile：同型；
分配面债）、mbsuwang 宿望（mobile：用牌+未受伤双门控；摸牌阶段改获面债）。
tw 包 ABI 684→685、目录式 sources 联动上浮（jieqiu 断言同步）。

**契约测试**：`original_history_filter_batch.rs`（伤害对门控、用牌计数开启、
受伤关闭、互不串扰）。
**ledger**：complete=5054 / approx=541 / todo=0 / noop=2081（-3 noop）。

### §8.16 历史第二刀定界关账与 zhafu 晋升（2026-09-10 第二百二十一批，零铸造散批）

- **历史查询第二刀判定不铸**：useCard 按对花色/牌名查询的候选消费集
  （pxjingzhao/x_dc_zhenyi/twxinshou/dcshigong/mbwusheng/dcbingji）逐件读体
  全为复合面（转化计数、手牌花色一致性查询、按花色首次记录链、随机弃牌
  检查）——无 3/3 集，与批 220 第一刀（3 件直接晋升）合并后**历史查询
  切片关账**。
- **active 目标引用面亦无 3/3 集**：twqingce/twchihui/weifeng/zhiren 等逐件
  复核均带复合次级面——active 目标引用原语与多段顺序确认同归复合选择 UI
  专项长期工单，不再按单批评估。
- **晋升 1 件**：**zhafu 诈附**（xianding noop→approx）——active 出牌阶段 +
  `mark_count(self, 1309895272)==0` 自身封门标记近似限定（limited 原语义为
  awakenSkill 移除技能；blake2b("zhafu_mark")，猎族工艺）→
  choose_other_add_mark 授「诈附」标记（blake2b("zhafu_effect_mark")
  =448647394）+ add_mark 封门。负担面（标记角色弃牌阶段保留一张、其余交给
  授予者）登记债。
- **契约测试**：`original_zhafu_mark_grant.rs`（授标+封门+二次拦截）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.17 sp2 占位族扩大抽样定界（2026-09-10 第二百二十二批，纯定界批，无代码）

对 sp2「待从旧 JS 技能完整迁移」占位族（165 件）扩大抽样读体：

- **抽样 18 件**：定向低复杂度 4 件（dcwujie/dcyinbi/dczuojian/dcwuyuan，
  按 trigger 无 choose/storage 筛选）+ 随机 10 件 + 首批 4 件
  （shunshi/xiying/jielie/dcyuzheng）。
- **判定：18/18 全复合，0 可表达**。形状分布：45-50L 主体、choose 交互
  16/18、storage/历史 4/18、mod 专属 1 件（dcyinbi——全场手牌上限取最大值
  修饰源缺失）、身份模式限定 1 件（dcwujie）。占位族自述「旧 JS 实现较复杂」
  与抽样一致，判定为**整体复合簇**，不再逐件单批重审；解锁路径归复合选择
  UI 原语族与专项读体（随引擎面扩充按簇重开）。
- **标记族剩余件顺带定界**：weifeng（每阶段首次伤害牌+目标域+weifeng2 互斥
  ——复合）、cixiao（势力/技能标记转移——复合）、bingzhao（主公技+势力
  声明——复合）、jsrgzuozhan（选人攻击范围取最高体力值——复合）。
- 回归：无代码/数据变化（纯定界批），沿用 workspace 4398/0/1 冻结基线；
  缓存治理维持（target/debug/incremental 已清）。

### §8.18 未勘探层普查与 dcsbjianwei 晋升（2026-09-10 第二百二十三批，零铸造散批）

按批指命普查 ledger 未勘探层（shiji/jsrg/v1.11.6 增量）：

- **shiji（6 件）全复合**：定仪 buff 状态机（mbdingyi/mbfubi/mbzuici）、
  四定义标记分配（mjdingyi）、衍生牌洗堆（tianzuo）；zjjuxiang 已有部分体。
- **jsrg（122 件）抽样 27 件全复合**：55L 主体、choose/subSkill/group/
  storage 密集；三件最低复杂度（jsrgcangchu 增益史多目标变数摸牌 /
  jsrgdaoren 双方射程交集伤害 / jsrgdangyi 主公技预算伤害增改）逐一复核
  均需未铸查询面。
- **v1.11.6 增量层（68 件）头部全览**：offline pe/lx/ps 系列（使命状态机/
  额外回合/座次势力查询）与 collab（质数判定/牌名档位）复合为主。
  **晋升 1 件**：**dcsbjianwei 坚卫**（xianding noop→approx）——摸牌阶段
  手牌<2 补摸 4（locked forced，tunchu 同款）；起始手牌+4 面
  （gameDrawBegin 数值改写）无原语维持债。
- **契约测试**：`original_dcsbjianwei_draw_topup.rs`（门控拦截 + 补 4）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.19 颜色计数表达式评估（2026-09-10 第二百二十四批，评估批，无代码）

按批指命评估「零 ABI 手牌红/黑计数表达式」切片：

- **非零 ABI 证实**：DSL 查询表达式经 query_tag（u8）编译进字节码
  （hand_count=tag 5 先例），新增查询 = tag 分配 + ABI 领号（equipped_count
  即绑定 ADDITIVE_RULE_VM_ABI_VERSION_681 先例）——预期 tag 716 / ABI 692
  （查询支持 (player, u8 kind) 双参，ZONE 先例，单 tag
  `hand_color_count(player, 0红|1黑)` 即可覆盖）。
- **消费集普查**：完全可表达仅 **dcchengchong 承宠** 1 件（触发面可复用
  现成过滤器 `owner_gained_cards_potduzuo`——CardsDrawn/CardsMoved 入自身
  手牌判定已注册；body 级红>黑比较缺查询）。同形件的红黑均匀比较
  （xinbingyi 秉壹/twdaigui 待归「所有手牌颜色相同」= 红计数==0 或黑计数==0）
  虽比较面可表达，但其效果面（至多 X 名角色各摸一张的多目标摸牌 / 牌堆底
  亮牌选获）复合——不构成完整晋升件。其余（jsrgdingce 双选牌颜色比对 /
  twmouzhi 上次受伤牌颜色存储 / twdidao 改判链）为跨选牌状态或改判复合。
- **判定：1 < 3/3 门槛，不铸**。解锁配方已入册（新查询 + ABI 692 领号 +
  15+ 处注册 + dcchengchong 触发复用 owner_gained_cards_potduzuo），
  与 dcchengchong 同簇件（dcdaijia 黛颊随机红牌等）随查询落地后按簇重开。
- 回归：无代码/数据变化（评估批），沿用 workspace 4399/0/1 冻结基线；
  缓存治理维持（target/debug/incremental 已清）。

### §8.20 多目标摸牌原语普查（2026-09-10 第二百二十五批，评估批，无代码）

普查「至多 N 名角色各摸一张」多目标摸牌原语消费集：

- **广谱普查 16 件头部 + 定向读体**：净形状仅 **twchungang 纯刚** 1 件
  （多触发 game_started/round_started/active + 至多两名各摸 1；摸得牌使用
  限制面债）。**jsrgcangchu 藏储**：门控可复用
  `owner_alive_hand_gained_this_turn_can_compare_other`（近似——上游为任意
  获得计数），但动态上限=回合获得牌数**无 DSL 表达式**（card_gain_count 仅
  过滤器/跟踪面可用，非查询表达式），各摸 1/2 条件量亦需近似——需再铸
  gain_count 查询才可表达。**twmitu 迷途**：限已受伤目标 + 拼点链复合。
  其余（dczimu/twxiongjun/jsrgrihui 的「全部持有者/判定区有牌者各摸」）
  为不同原语形状（持有者查询批量摸）。
- **判定：净件 1 < 3/3，不铸**。解锁配方入册：原语
  `choose_targets_draw(player, self_skill, max_targets, draw_each)`
  （SelectTargets 1..max + 逐目标 draw 续体）；jsrgcangchu 另需
  `gain_count(player)` 查询切片；twmitu 另需目标限定标志（仅已受伤）。
- 回归：无代码/数据变化（评估批），沿用 workspace 4399/0/1 冻结基线；
  缓存治理维持（target/debug/incremental 已清）。

### §8.21 既有面清账三件晋升与普查方法更正（2026-09-10 第二百二十六批，零铸造散批）

面批量落地专项首批启动即获重大发现——**两件判定「不铸」的面在引擎中早已存在**：

- `choose_targets_draw(player, count, target_count, scope)`（effect_tag 106，
  scope=other|male_other|female_other|any|min_hand|same_hp）及
  `choose_optional_targets_draw(player, count, min, max, scope)` 变体；
- `hand_color_count(player, red|black)` 查询（query_tag，ABI 490）。

**批 224/225 的「不铸」判定据此更正**：颜色计数与多目标摸排均为既有面。
更正后的真实缺口收窄为：①gain_count 表达式（jsrgcangchu 动态上限）仍缺；
②twmitu 的已受伤目标限定标志。**普查方法教训**：判定「不铸」前必须
grep codegen 的 expr/effect 名映射（typecheck 名称清单），仅查 catalog/
过滤器清单会漏检既有查询与效果面。

**晋升 3 件**（noop→approx，零铸造）：
- **dcchengchong 承宠**（xianding）：双臂 cards_moved/cards_drawn +
  新过滤器变体 `owner_gained_cards_potduzuo_once_per_turn`（零 ABI：Potduzuo
  增益判定 + turn_limit，近似上游递归守卫）+ body 级红>黑比较（既有查询）
  → 摸 1。债：每获牌可发收窄为每回合一次。
- **twchungang 纯刚**（tw）：game_started + active（uses_per_turn 1）→
  `choose_targets_draw(self, 1, 2, any)`。债：摸得牌使用限制面。
- **jsrgcangchu 藏储**（jsrg）：phase_end + 既有获得过滤器 optional →
  `choose_optional_targets_draw(self, 1, 1, 5, any)`。债：动态上限收窄为
  字面量 5、各摸 1/2 条件量固定 1。
- **契约测试**：`original_existing_face_recheck.rs` 3 件。
- **ledger**：以生成器输出为准（本批 -3 noop）。

### §8.22 更正后方法首轮复测：xinbingyi 晋升（2026-09-10 第二百二十七批，零铸造散批）

以批 226 更正后的普查方法（codegen 名映射全表 grep）首轮复测：

- **give 族既有面确认**：`choose_give_hand_equip_to_event_player` /
  `choose_other_give_one_hand_equip` / `choose_give_or_discard_hand` 等
  give 方向原语已存在（批 187「give 族无 owner→event_source 方向」结论
  部分过时，可按簇复测给予类消费者）。
- **晋升 1 件**：**xinbingyi 秉壹**（mobile noop→approx）——单臂 optional +
  **乘积判零**表达颜色均匀 OR（`hand_color_count(red) * hand_color_count(black)
  == 0` ⇔ 至少一色为 0；DSL 无 `||`，双臂同事件 confirm 队列会互相吞并
  选择——工艺教训）+ `choose_optional_targets_draw(self, 1, 1, 8, any)`。
  债：类型维度、动态上限收窄 8、展示面。
- **原语语义陷阱入册**：`choose_targets_draw` 为**恰好 N 选**（合格目标数
  < N 即静默 no-op、不可取消）；「至多」语义必须用
  `choose_optional_targets_draw(player, count, min, max, scope)`。
- **契约测试**：`original_existing_face_recheck.rs` 增至 4 件（混色门控 /
  单色选择摸牌）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.23 给予族复测与 spshicai 晋升（2026-09-10 第二百二十八批，零铸造散批）

以 choose_give_* 既有面按簇复测给予族消费者：

- **复测结论**：纯给予主面的干净消费者稀少——jsrgdaoren（给牌+双方射程
  交集伤害）、lxbeini（二选一分支）、rencheng（等量反馈）、psjiusheng
  （多目标链）均带复合次级面。**散件捕获 spshicai 识财**（sp2 noop→approx，
  非给予但同轮复测捕获）：active 出牌阶段 → discard_one_hand_or_equip(self)
  + draw_cards(self, 1)（弃任意一张、牌堆顶入手）；无 usable 按上游不限次。
  债：子面（令一名角色使用所得+摸一张）与 storage 上张所得限制。
- **头注失实记录**：kuanshi 宽释的 ported 头注「弃一张牌令一名角色回复
  1 点体力」与上游不符（实为结束阶段选目标的延迟防伤标记链）——该件
  维持债，头注待其重写批更正。
- **契约测试**：`original_spshicai_active_trade.rs`（active 驱动 + 弃一摸一
  + 弃牌堆落位）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.24 转换技/view_as 面复测与头注漂移发现（2026-09-10 第二百二十九批，定界批，无代码）

以名映射全表重审转换技族：

- **view_as 既有面确认丰富**：kinds 含 basic/sha/shan/tao/jiu + 13 种锦囊
  （guohe/shunshou/juedou/wuzhong/le/bingliang/huogong/jiedao/tiesuo/wuxie/
  wanjian/wugu/nanman/chuqibuyi）；filters 含 red/black/花色四色/card 系列/
  owner_hand_gt_hp；options 含 uses_per_turn/ignore_distance/once_per_round/
  branch（转换态）/mark_cost 等。经典「X 当 Y 使用」类 noop 可按此面成簇复测。
- **头注漂移发现（3 件）**：fuchao/duanxie/cihuang 的 ported 头注与上游体
  不符——上游实读：fuchao=横置当前回合角色+轮末横置者摸牌弃牌的 forced 技
  （非「黑牌当杀」）；cihuang=chooseButton 动态列表转化（非「黑牌当桃」）。
  **翻译引写头注不可作重写依据**，三件维持 noop，重写批须先补上游实读。
- **choose_use_virtual_card 限定**：仅 phase_any_end 触发可用、仅
  wanjian/juedou 两种——「视为使用」类通用面仍缺（jsrgcuifeng 单目标伤害类
  虚拟牌等维持债）。
- 回归：无代码/数据变化（定界批），沿用 workspace 4404/0/1 冻结基线；
  缓存治理维持（target/debug/incremental 已清）。

### §8.25 经典转化簇实读定界（2026-09-10 第二百三十批，定界批，无代码）

经典转化 noop 簇四件上游实读（防头注漂移流程）：

- **dcbingji 秉纪**（sp2 王威）：全手牌同花色判定（无花色一致性表达式）+
  本回合未用该花色（per-suit 历史 DSL 无过滤器）+ 虚拟杀/桃双选带 storage
  无限次 mod——复合维持债。
- **dcdouzhen 斗阵**（sp2 穆顺）：zhuanhuanji 转换态（☯ 标记奇偶）+ mod 式
  强制转化（红基本→无次限杀 / 黑基本→决斗得牌）——转换态 mod 与 view_as
  branch 语法尚无对齐实证，复合维持债。
- **pxqixian 祁县**（sp）：recast_selected 既有（重铸腿可表达）但后续
  sha/guohe 双支虚拟用牌（chooseButton 分支）+ chooseToUse 时点窗口复合——
  维持债。
- **hezhong 合纵**（sp）：lose/gain 事件计数门控 + 手牌唯一性——复合维持债。
- **结论**：经典转化簇经实读全数复合；view_as 既有面的直接消费簇尚未
  成立，需先解决「转换态 branch 语法 ↔ zhuanhuanji」「花色一致性表达式」
  「per-suit 使用历史过滤器」三个缺口。
- 回归：无代码/数据变化（定界批），沿用 workspace 4404/0/1 冻结基线；
  缓存治理维持（target/debug/incremental 已清）。

### §8.26 approx 池升级评估首刀：paiyi 三件 min(7,N) 清偿（2026-09-10 第二百三十一批）

approx 池升级路径评估首刀——**发现 min()/max() 表达式已存在**（weipo
`min(5, max_hp(self))` 既有先例），批 202 登记的「表达式内 Min 跨原语参数
不可用」债前提已失效：

- **升级 3 件**（approx 内保真度提升，轴不变）：gzpaiyi（refresh）/
  zypaiyi（offline）/ olpaiyi（onlyOL）的排异摸牌量由「剩余权数」升级为
  `min(7, skill_pile_count(...))`——上游「最多 7」分支清偿。
- **方法结论**：approx 升级评估须先查**表达式能力表**（min/max/颜色计数/
  花色计数等既有查询）再判定缺口真伪——批 202 时代的「不可用」结论随
  引擎演进失效。
- 其余已知升级缺口复测：xinbingyi/jsrgcangchu 动态上限（max 参数为字面量
  u8，表达式不可用——真缺口）；twyouye/mbyouye 分配面、twxingwu/biaozhao
  消耗面、spshicai 子面维持债。
- 回归：workspace 全量（终态见日记），缓存治理批内清理 incremental。

### §8.27 approx 升级评估第二刀（2026-09-10 第二百三十二批，评估批，无代码）

以表达式能力表（min/max/hand_color_count/hand_suit_count）对 approx 池债注
做模式匹配普查：

- **普查命中**：颜色类 6 件 / 花色类 11 件 / min-max 字面量类 2 件
  （jsrgcangchu、xinbingyi——真缺口，choose_optional_targets_draw 的 max
  为字面量 u8）。
- **抽样核对结论**：颜色/花色命中件（twzhenliang 弃牌与「任」颜色比对、
  jsrgfumou 跨目标颜色禁令、zhancai/zhenbian 花色集合记录）均为**事件流/
  存储集合类债**，与手牌计数能力（hand_color_count/hand_suit_count 只数
  当前手牌）不匹配——不可清偿。
- **结论**：表达式能力升级路线在 approx 池已穷尽（第一刀 min/max 类 3 件
  paiyi 已清偿；第二刀颜色/花色类无匹配）。剩余 approx 债与 noop 同源——
  归复合引擎面长期工单；升级路线图收缩为「随新引擎面按簇重开」。
- 回归：无代码/数据变化（评估批），沿用 workspace 4404/0/1 冻结基线；
  缓存治理维持（target/debug/incremental 已清）。

### §8.28 头注漂移抽查首批（2026-09-10 第二百三十三批，登记诚实性修正）

跨包抽样 12 件 noop（每包一件）实读上游比对头注：

- **漂移修正 2 件**：zengou（sp）首行「音频/辅助技能」系陈旧误引——上游实为
  「范围内角色【闪】被使用时可弃非基本牌或失 1 体力令其被排除
  （all_excluded）」，据实读重写头注（响应排除机制无 DSL 原语，诚实 noop
  维持）；beOfOneHeart（tw）「游戏开始时/准备阶段」双时点误引——上游实为
  phaseBegin 单时点 + oneHeart 技能信息过滤，头注更正。
- **准确件 7 件**（shunshi 占位诚实 / duwang / qianlong / olluanji inherit /
  mbsaojian 无独立效果 / shiina_qingshen / rekongsheng 复合描述与上游一致）；
  包登记类 3 件（sb_meta/qingshu/ddddiedang）为声明文件非技能。
- **抽查比例与发现率**：12 件中漂移 2 件（17%），均为头注文案级（未发现
  「实为简单技被误标复合」案例——批 229 的 3 件漂移亦为文案级误译）。
  头注漂移不改变 ledger 轴，但影响后续重写的依据可靠性——重写批必须
  上游实读（已在批 229 入册）。
- 回归：sp/tw 重编部署（头注注释变更不影响逻辑哈希——注释不进编译；
  若 logic_hash 有变以 manifest 为准同步），终态见日记；缓存治理批内
  清理 incremental。

### §8.29 复合引擎面依赖分析与头注漂移扩样（2026-09-10 第二百三十四批）

- **复合引擎面专项首批依赖分析**：批 230 入册三缺口对唯一整件消费者
  dcbingji **缺一不可**（花色一致性 filter 前半 + per-suit 历史 filter 后半
  + 虚拟杀/桃双选结算面），单/双面铸造均不解锁整件；捆绑铸造的唯一消费
  计 1 件 < 3/3——**专项首批判定缓铸**，三缺口维持入册待簇扩张。
- **头注漂移扩样（第三轮 10 件）**：文案级 0、**省略型漂移 2 件**——
  olsbduoshou（头注未描述契约：实为红牌条件距离 mod）与 liyong3（「无独立
  info 条目待补充」：实为 liyong 家族链式子臂，随父技重写批处理）——均据
  上游实读补记头注；其余 8 件头注准确。三轮累计漂移 7 件（含批 229 三件
  误译型 + 批 233 两件文案型 + 本批两件省略型），漂移类型学成型。
- 回归：onlyOL/mobile 重编部署（注释变更），终态见日记；缓存治理批内
  清理 incremental。

### §8.30 专属状态机工艺首批：dcbingji 秉纪（2026-09-10 第二百三十五批，tag 715 / ABI 692）

按批 234 备选路径对经典转化簇旗舰件 dcbingji 走专属 host 状态机工艺
（resolve_xinxingluan/ABI 476 先例）：

- **机器结构**：`resolve_dcbingji(player, skill, stage, kind)` 三阶段——
  ①校验（手牌非空+全同花色（effective_card_suit 逐张）+ 该花色本回合未用
  （used_card_suit_history 位测试））→ Custom 双选（杀/桃）→ ②桃自用 /
  杀选目标（其他存活角色宽口径）→ ③物化虚拟牌
  （materialize_virtual_card + play_as_kind ignore_use_count=true）+
  记录统一手牌花色至 used_card_suit_history。
- **注册**：tag 715 / ABI 692（领号+host bump）/ Cont 300
  ResolveDcbingjiStage / 盐 DCBJ；15 处注册模板 + vm_host ctx 注入。
- **工艺教训**：①Custom 续体打开的选择会被框架清理——**续体打开的后续
  选择必须用真实 ContinuationKind**（A1 先例同源）；②虚拟牌无花色——
  花色史记录取持有者统一手牌花色（虚拟使用不消耗手牌）。
- **晋升 1 件**：dcbingji 秉纪（sp2 noop→approx）。债：展示面、sha 目标
  宽口径（上游走通用 useCard 合法性）。
- **契约测试**：`original_dcbingji_machine.rs` 3 件（混色沉默 / 桃自用+
  花色史阻断再发动 / 杀目标链）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.31 专属状态机第二件：pxqixian 祁县（2026-09-10 第二百三十六批，tag 716 / ABI 693）

dcbingji 工艺复用第二件：**pxqixian 祁县**（sp noop→approx）——出牌阶段限
一次，重铸一张装备牌（手牌/装备区装备牌置入弃牌堆）并视为对一名其他角色
使用虚拟【杀】或【过河拆桥】。

- **机器结构**：`resolve_pxqixian(player, skill)` 入口 + `pxqixian_step`
  四阶段（牌种双选 → 重铸素材 SelectCards → 目标 SelectTargets → 素材入弃
  + materialize_virtual_card + play_as_kind(ignore_use_count)）。
  **载荷 7 槽**：[magic, player, skill, stage, kind, material, target]——
  多值传递需显式多槽（批 235 的 5 槽在双值阶段丢失 material 的教训）；
  **载荷 stage 槽必须写当前阶段号**（resolver 按槽分派，写下一阶段号会
  错位分派）。
- **注册**：tag 716 / ABI 693（领号+host bump）/ Cont 301
  ResolvePxqixianStage / 盐 PXQX；sp 包 ABI 684→693（断言同步 7 文件）。
- **契约测试**：`original_pxqixian_machine.rs` 2 件（无装备沉默——注意
  bootstrap 手牌可含装备牌、沉默测试勿与 uses_per_turn 共局；全链含过河
  拆桥弃牌子选择）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.32 专属状态机第三件：quanjia 拳甲（2026-09-11 第二百三十七批，tag 717 / ABI 694）

dcbingji/pxqixian 工艺复用第三件：**quanjia 拳甲**（collab noop→approx）——
一名角色（当前回合角色）对另一角色造成伤害后（来源非你且有手牌），随机观看
其 (2+标记数) 张手牌：有【杀】则对其使用虚拟【杀】，否则获得 1 枚标记。

- **机器结构**：`resolve_quanjia(player, skill, source, target)` **单发无续体**
  ——机内权威校验（来源非自己/目标非来源/来源为当前回合角色/存活有手牌）+
  RNG Fisher–Yates 取样（RuleRng from_state/gen_range/into_state 既有工艺）+
  物化虚拟杀 play_as_kind(ignore_use_count) 或 add_mark。**无续体机器是
  触发驱动件的最简形态**（确认后一次执行到底）。
- **注册**：tag 717 / ABI 694（领号+host bump 694）/ 盐 QJA；无 Cont——
  8 处注册即足（无续体类机器的精简模板）。collab 包 ABI 664→694。
- **契约测试**：`original_quanjia_machine.rs` 2 件（无杀加标记 / 来源=自己
  机内拒绝）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.33 单债项普查与 tsumugi_huilang2 升级（2026-09-11 第二百三十八批）

approx 池**单债项普查**（仅剩一条登记债的件最接近完全体）：命中 3 件——

- **tsumugi_huilang2 隐廊·终段升级**（key approx 保真度提升）：单债「多目标
  各摸 1 分支未建模」恰逢 choose_targets_draw 既有（批 226 清账）——补
  `choose_targets_draw(self, 1, 8, any)`，债收窄为动态上限字面量 8。
- dunyong_effect（遁勇子臂，授予型临时效果挂件）与 twlihuo2（近似为时点
  注记非功能缺口）不适用。
- **方法结论**：单债项普查是 approx 升级的高信噪比入口（3 件中 1 件可
  升级，且升级直接对齐既有原语）。
- 回归：key 重编部署（modules 115 不变——模块数按新计数核对），终态见
  日记；缓存治理批内清理 incremental。

### §8.34 专属状态机第四件：duwang 度望（2026-09-11 第二百三十九批，tag 718 / ABI 695）

状态机工艺第四件：**duwang 度望**（newjiang noop→approx）——游戏开始时，
将牌堆顶前 5 张非【杀】牌置于自身技能堆（「刺」）。

- **机器结构**：`resolve_duwang(player, skill)` 单发无续体（quanjia 精简
  模板，12 处注册）——牌堆顶（向量尾）向牌堆底扫描，按 CardKind != Sha
  取前 5 张，ensure_skill_pile_zone + move_cards 置入「刺」堆。
- **保守省略判定**：距离修正面（其他角色至你的距离 +min(1, 刺数)）——
  `owner_skill_pile_count` 修饰源只能给 +N（超集、强化对对手的限制），
  min(1,count) 无修饰源——按「保守收窄优先」原则省略登记债，不建超集面。
- **注册**：tag 718 / ABI 695（领号+host bump 695）/ 盐 DUW；newjiang 包
  ABI 695（断言同步 1 文件）。
- **契约测试**：`original_duwang_machine.rs`（5 张非杀落堆 + 无杀混入）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.35 专属状态机第五件：jsrgdaoren 蹈刃（2026-09-11 第二百四十批，tag 719 / ABI 696）

状态机工艺第五件：**jsrgdaoren 蹈刃**（jsrg noop→approx）——出牌阶段限一次，
将一张手牌交给一名其他角色，然后对处于你与其攻击范围交集内的所有角色各造成
1 点无来源伤害。

- **机器结构**：`resolve_jsrgdaoren(player, skill)` 入口 + `jsrgdaoren_step`
  三阶段（手牌 SelectCards → 目标 SelectTargets → 交牌（跨手区移动+owner
  改写）+ `player_in_attack_range` 交集逐角色 `deal_damage(NONE, p, 1)`）。
  载荷 6 槽：[magic, player, skill, stage, card, target]。
- **注册**：tag 719 / ABI 696（领号+host bump 696）/ 盐 JSDR / Cont 302
  ResolveJsrgdaorenStage；jsrg 包 ABI 670→696（断言同步 9 文件）。
- **零近似项**：交集用 player_in_attack_range 既有查询精确实现；伤害无来源
  与上游 damage("nocard") 同口径。
- **契约测试**：`original_jsrgdaoren_machine.rs`（交牌落区 + 三方血量非增）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.36 专属状态机第六件：qianlong 潜龙（2026-09-11 第二百四十一批，tag 720 / ABI 697）

状态机工艺第六件：**qianlong 潜龙**（xianding noop→approx）——受到伤害后，
观看牌堆顶三张牌，获得至多已损失体力值张牌，其余置于牌堆底。

- **机器结构**：`resolve_qianlong(player, skill)` 入口（校验存活 + 顶三张
  存在 + 已损失>0，SelectCards min 0 / max min(3, 已损失) 可取消）→
  `resolve_qianlong_stage(player, top3, selections)` 续体（所选入手 +
  未选按原次序压牌堆底向量头部）。载荷 7 槽携带顶三张牌 id。
- **注册**：tag 720 / ABI 697（领号+host bump 697）/ 盐 QLN / Cont 303
  ResolveQianlongStage；xianding 包 ABI 692→697。
- **契约测试**：`original_qianlong_machine.rs` 2 件（获得+置底落位次序、
  cap 超额拒绝）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.37 专属状态机第七件：jsrgzuozhan 坐瞻（2026-09-12 第二百四十二批，tag 721 / ABI 698）

状态机工艺第七件：**jsrgzuozhan 坐瞻**（jsrg noop→approx）——游戏开始时，选择
至多两名其他角色，攻击范围 +X（X = 所选角色最高体力值且至多 5）。

- **机器结构**：`resolve_jsrgzuozhan(player, skill)` 入口（校验存活 + 存在其他
  角色）→ SelectTargets 1..2 可取消（取消=不发动）→
  `resolve_jsrgzuozhan_stage(player, skill, selected)` 续体：射程标记
  min(所选最高体力, 5) 加于自身 +「坐瞻」标记记于所选角色（0x5A55_0002，
  对应上游 markAuto 目标集）。
- **工艺发现（self_skill 语义）**：DSL `self_skill` 经 HostSkill 压入的是
  **context.skill_id = 触发实例 id**（skills.alloc 顺序号），非 def id——
  forced 直驱（duwang）与 optional 确认链（本批）同源。机器内凡以 def 作键
  （marks/skill_pile）必须先经 `skill_pile_def(tx, player, skill)` 解析
  （duwang 事实同此，其测试按 def 断言能过系 pack 级键一致所致——本批
  jsrgzuozhan 以 owner_skill_mark 修饰源按 def 读标记，跨键即静默归零，
  探针实证 (8,4) 后修正）。**「机内加标记用 def 键必须过 skill_pile_def」
  入册为机器工艺第 4 条踩坑。**
- **修饰面复用**：`modifier attack_range add owner_skill_mark`（jsrglangan
  同款，正方向）——射程标记 id = 技能自身 def，既有修饰源精确消费，零新增。
- **注册**：tag 721 / ABI 698（领号+host bump 698）/ 盐 ZUZH / Cont 304
  ResolveJsrgzuozhanStage；jsrg 包 ABI 696→698（断言同步 9 文件 21 处 +
  trigger 计数 53→54 两文件）。
- **近似（登记债）**：①enterGame/phaseBefore 双时点以 game_started 单点近似；
  ②组技「坐瞻角色死亡后令存活坐瞻角色从弃牌堆获得至多 X 张不同名基本牌」
  面未建模（需死亡事件目标集合过滤 + 弃牌堆按牌名去重扫描链；坐瞻标记已在
  机内记录，后续批可复用）。
- **契约测试**：`original_jsrgzuozhan_machine.rs` 4 件（双目标标记+射程值、
  四人局对家距离 2 修饰面实证 player_in_attack_range 由 false 转 true、
  取消零标记、自选被 resolver 拒绝）。
- **ledger**：complete=5042 / approx=553 / todo=0 / noop=2068（-1 noop）。

### §8.38 坐瞻组技补牌面闭合（2026-09-12 第二百四十三批，tag 722 / ABI 699）

批 242 登记债②当日闭合：**jsrgzuozhan_gain 组技补牌状态机**（jsrgzuozhan
approx 保真度提升）——「坐瞻」标记角色死亡后（全局 dieAfter），令一名存活的
坐瞻角色从弃牌堆获得至多 X（射程标记数）张牌名各不相同的基本牌。

- **机器结构**：`resolve_jsrgzuozhan_gain(player, skill, event_player)` 入口
  （机内权威校验：owner 存活 / 死亡者带「坐瞻」标记 / 存活坐瞻角色存在 /
  弃牌堆有可取基本牌）→ SelectTargets 1..1（存活坐瞻角色）→
  `resolve_jsrgzuozhan_gain_stage` 续体：弃牌堆顶（向量尾）向堆底扫描取
  min(X, 堆内不同名基本牌数) 张入手。
- **牌名口径**：杀/火杀/雷杀同名（zuozhan_basic_name_group 四组：
  sha/tao/shan/jiu），与上游 card.name 去重对齐；非基本牌跳过。
- **注册**：tag 722 / ABI 699（领号+host bump 699）/ 盐 ZUZG / Cont 305
  ResolveJsrgzuozhanGainStage；jsrg 包 ABI 698→699（断言同步 9 文件 22 处 +
  trigger 计数 54→55 两文件）。
- **近似（登记债）**：optional 确认先于机内校验打开——未标记角色死亡也空开
  一次确认（quanjia 批 237 同款债，测试固化该语义）。
- **契约测试**：`original_jsrgzuozhan_machine.rs` 增至 6 件（死亡驱动全链：
  同名杀去重 + 锦囊跳过 + 配额钳位 + 死亡者标记保留 / 未标记死亡确认后
  空转）。测试侧踩坑：直移弃牌堆必须同步 card.zone 元数据，否则补牌结算
  时验证器 EngineFault("zone N card M mismatch")。
- **ledger**：以生成器输出为准（noop 轴不变，approx 债注收窄）。

### §8.39 专属状态机第九件：twmouzhi 谋制（2026-09-12 第二百四十五批，tag 723 / ABI 700）

状态机工艺第九件：**twmouzhi 谋制**（tw noop→approx）——锁定技。受伤时若此
牌颜色与上次受伤牌颜色相同则防止；每次受伤后记录颜色（无牌/无色清除）。

- **机器结构**：`resolve_twmouzhi(player, skill, mode, event_card)` **双触发
  共调单发机器**（quanjia 精简模板扩展）：mode 0（damage_before）校验事件牌
  有色 + storage 记录同色 → 直接推 `resolution.prevented_damage_ids`
  （即 prevent_current_damage 原语的机内等价，vm_host 对 DSL 侧的
  damage_id:0 重写不适用于机内直调）；mode 1（damage）写 storage
  （红=1/黑=2/清除=0，键=1）。
- **上游语义保真**：防止的伤害经 trigger.cancel() 不进入 history → 「上次
  受伤牌颜色」不变 → 连续同色伤害均被防止；机内存储口径与之一致。
- **注册**：tag 723 / ABI 700（领号+host bump 700）/ 盐 MOUZ / 无 Cont
  （单发精简）；tw 包 ABI 685→700（断言同步 4 文件；trigger 149→151、
  modifier 308→307、modules 164→166 无既有冻结断言）。
- **契约测试**：`original_twmouzhi_machine.rs` 3 件（同色防止+粘滞+换色
  生效/无牌清除后重建/契约注册）。测试侧驱动：直接推 `CommitDamage`
  （run_effects 对其做 DamageBefore 准备并注入 card 上下文；DealDamage
  效果恒 CardId::NONE 不适用于带色伤害）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.40 待归多段选择链状态机专项·slice 1（2026-09-12 第二百四十七批，tag 724 / ABI 701）

多段选择链专项首批启动：**twdaigui 待归 slice 1**（tw noop→approx 首切片）——
出牌阶段结束时手牌同色门控 → 选 [1, 手牌数] 名任意角色（可含自己、可取消）→
亮出牌堆底等量牌（slice 1 暂存弃牌堆；逐目标选获面由 slice 2 在 stage 2 内
插接，余牌归弃牌堆与上游一致）。

- **机器结构**：`resolve_twdaigui(player, skill, stage, count)`——stage 0
  （DSL 直调）：机内权威校验（存活+手牌非空+红黑同色，按花色逐一判色）→
  open SelectTargets [1, 手牌数]；stage 2（续体 Cont 306）：取牌堆底
  （向量头部）等量牌 → RevealCards 公开亮出 → move_cards 暂存弃牌堆。
  slice 2 的逐目标选获以「stage 2 内逐目标开选（异观察者）+ 载荷携带目标
  列表与余牌」扩展。
- **DSL 触发面**：`trigger phase_play_end owner_is_target optional`——
  phaseUseEnd 事件由 DomainEvent::PhaseEnded{phase: Play} 派发（非
  TurnStateChanged，后者 Play 相位映射 PhasePlay——测试驱动踩坑入册）。
- **注册**：tag 724 / ABI 701（领号+host bump 701）/ 盐 TWDG / Cont 306
  ResolveTwdaiguiStage；tw 包 ABI 700→701（断言同步 4 文件 + jieqiu 独立
  编译断言 1 处；trigger 151→152、modifier 307→306、modules 166→167 无
  冻结断言）。
- **契约测试**：`original_twdaigui_machine.rs` 3 件（同色全链：目标选择+
  牌堆底亮出暂存弃牌堆次序保持 / 红黑混合门控在确认后拦截 / 取消零变动）。
  测试侧手牌重组：原手牌移回牌堆底 + 按花色现场选取新入手牌（须逐张实时
  查找，快照式查找会重复选取同一张）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.40 续：待归 slice 2 逐目标选获收口（2026-09-12 第二百四十八批，纯引擎侧）

多段选择链专项第二切片当日收口：**twdaigui 待归选获面**（twdaigui approx
保真度补全）——亮牌后按座次逐目标选获，纯引擎侧扩展（stage/载荷协议演进，
tag 724 / ABI 701 不变，DSL 与 npack 零重建）。

- **轮次协议**：stage 3 载荷自描述——`[magic, player, skill, 3, k, m,
  targets(k), revealed(m), picks(2×r)]`（chooser/card 成对追加，card=0 表示
  跳过）。resolve_choice 按 payload[3] 分派 stage 0 链与 stage 3 轮。
- **异观察者选择**：每轮 SelectCards 1/1 强制，observer = 当前选获目标
  （resolver 校验 observer == targets[r]）；余牌池 = 亮出牌 − 已选；
  已死亡目标与余牌为空的轮次按上游 `continue` 跳过（picks 记 0）；
  全轮完成后余牌归弃牌堆（目标数=亮牌数时无余牌）。
- **契约测试**：`original_twdaigui_machine.rs` 增至 4 件（座次顺序逐目标
  选获+入手断言+无余牌 / 选择顺序与座次相反时选获轮仍按座次推进且
  observer 错位被拒 / 混合色门控 / 取消零变动）。
- **工程注记**：同一 effect tag 的 stage/载荷协议可跨批扩展而不动包 ABI——
  机器的权威状态在载荷与 GameState 中，包内字节码只编码入口调用；
  多段机器「先通链路后补阶段」的分批路线就此验证。
- **ledger**：以生成器输出为准（noop 轴不变，approx 债注收窄）。

### §8.41 迷途多段拼点链状态机专项·slice 1（2026-09-12 第二百四十九批，tag 725 / ABI 702）

多段专项第二批启动：**twmitu 迷途 slice 1**（tw noop→approx 首切片）——准备
阶段选至多 3 名角色，各摸 2 张、摸得牌打「迷途」标记并公开亮出。

- **机器结构**：`resolve_twmitu(player, skill, stage, count)`——stage 0
  （DSL 直调）→ SelectTargets [1,3] 任意存活角色可取消；stage 2（续体
  Cont 307，resolver 经 resolve_twmitu_stage）：各目标按座次
  `draw_cards(2)` → 摸得牌 `add_content_card_tag(迷途)`（gaintag 面首次
  经 content_card_tags 落地，blake2b("twmitu")=3195907057）→ RevealCards
  公开亮出。slice 2 将以 stage 3+ 轮次载荷扩展拼点链（复用 twdaigui
  自描述载荷模式）。
- **头注漂移修正**：本件旧头注「至多三名已受伤角色各摸一张牌」两处失实——
  上游 chooseTarget 无 filterTarget（不限已受伤），`target.draw(2)` 实为
  摸 2 张（非 1 张）。本实现按代码口径。
- **注册**：tag 725 / ABI 702（领号+host bump 702）/ 盐 TWMT / Cont 307
  ResolveTwmituStage；tw 包 ABI 701→702（断言同步 5 文件；trigger 152→153、
  modifier 306→305、modules 167→168 无冻结断言）。
- **契约测试**：`original_twmitu_machine.rs` 2 件（双目标各摸 2 全部打标/
  取消零变动）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.41 续：迷途 slice 2 拼点轮收口（2026-09-12 第二百五十批，纯引擎侧）

多段专项第三切片：**twmitu 迷途拼点轮**（twmitu approx 保真度补全）——亮牌
后按座次逐目标拼点：owner 指定对手 → 目标知情（Custom 1/2）→ 双方依次选拼
点牌 → 点数大者视为对小者使用虚拟【杀】（tie 无效果）。

- **轮次协议**：stage 3/4/5/6 自描述载荷（`[magic, player, skill, stage, k,
  done, targets(k), opponents(d), consents(d), pend]`，d 随阶段递增）——
  分段数组切片端点须累加前段长度（opponents_end 后再累 d_consents，批内
  一次 slice 8..7 panic 修正）；resolve_choice 按 payload[3] 分派。
- **异观察者四类actor**：owner（SelectTargets 选对手）/ 目标（Custom 知情
  + SelectCards 拼点牌）/ 对手（SelectCards 拼点牌），逐 resolve 校验
  observer 一致性。
- **既有面复用评估**：ABI 680 ChooseHandCompareBetween 的胜负结果钩子
  （胜方虚拟杀败方）与目标选择方均硬编码于其自身技能实例校验链，无法作为
  回调嵌套——拼点子在机内以 SelectCards+rank 比较自实现（rank 字段既有），
  虚拟杀复用 materialize_virtual_card + play_as_kind(ignore_use_count)。
- **契约测试**：`original_twmitu_machine.rs` 增至 4 件（slice 1 双目标全链
  + 拼点轮 declined 收口 / 胜方虚拟杀败方 -1 体力，含闪响应取消 / 拒绝拼点
  零伤害 / 取消零变动）。
- **ledger**：以生成器输出为准（noop 轴不变，approx 债注收窄）。

### §8.41 续：迷途 slice 3 崩溃面收口（2026-09-12 第二百五十一批，纯引擎侧）

多段专项第三切片：**twmitu 迷途崩溃面**（twmitu 全链可玩）——每个参与拼点
的「迷途」目标，若其拼点用牌不是自己的展示牌（首张摸得牌），owner 减 1 点
体力上限。

- **实现口径**：放弃 global compare 事件钩子方案——slice 2 的拼点在机内
  自结算（无 compare 领域事件），崩溃判定直接置于 stage 6 比点之后：遍历
  双参与方（对手若亦为「迷途」目标同样计罚，对应上游 filter 对
  [event.player, event.target] 逐方检查），used_card != shown[idx] 计罚，
  `change_max_hp(player, -penalty)` 一次结算。
- **工艺定界（重要）**：`play_as_kind` 会在机内中段开启闪响应选择并以
  Err(ChoiceRequested) 暂停——其后的机器逻辑（惩罚、advance）全部丢失
  （单目标测试不可见，多目标断链）。多段机器内可暂停管线（用牌/响应类）
  **只能置于机器终末步**；中段影响改以 deal_damage 直调并登记债（本批：
  目标无应闪机会）。
- **payload v3**：新增 shown(k) 段（首张摸得牌，stage 2 记录）——载荷布局
  演进为引擎内部协议，npack 零重建（DSL 入口不变）。
- **债（登记）**：①拼点用牌不可应闪（直伤结算）；②change_max_hp 仅钳位
  体力至新上限，上游 loseMaxHp 同步 hp-1（体力满时表现差异）；③崩溃面
  上限减少无次数上限（与上游一致 ✓）。
- **契约测试**：`original_twmitu_machine.rs` 增至 6 件（+崩溃惩罚命中/
  拒绝拼点无惩罚）。
- **ledger**：以生成器输出为准（noop 轴不变，approx 债注收窄）。

### §8.42 拼点族状态机首件：jsrgqiantun 谦吞（2026-09-12 第二百五十二批，tag 726 / ABI 703）

twmitu slice 2 验证的机内拼点能力首次复用：**jsrgqiantun 谦吞**（jsrg
noop→approx）——出牌阶段限一次，令一名其他角色展示任意张手牌（其拼点只能
从展示牌中选），与其拼点：赢则获得其展示的手牌，没赢（含平点）则获得其其余
手牌。双向获得 → 无半实现陷阱。

- **机器结构**：`resolve_qiantun(player, skill, stage)` 四段（Cont 308）——
  0=SelectTargets 其他有手牌角色（可取消）→ 2=目标展示 [1, 手牌数] 强制 →
  3=owner 选拼点牌（不受限）→ 4=目选拼点牌（手中有展示牌则限展示牌，上游
  filterCard 口径）→ 5=rank 比较 + 拼点牌入弃牌堆 + 双向获得。
- **注册**：tag 726 / ABI 703（领号+host bump 703）/ 盐 QTN / Cont 308
  ResolveQiantunStage；jsrg 包 ABI 699→703（断言同步 9 文件 22 处；
  modifier 128→127、modules 64→65 无既有冻结断言）。
- **契约测试**：`original_qiantun_machine.rs` 3 件（赢臂：剩余展示牌归
  owner、拼点牌两侧入弃牌堆 / 输臂：非展示牌归 owner / 取消零变动）。
- **工程注记**：拼点子=「双方各选一张手牌 + rank 比较」机内自实现，展示
  约束以选项过滤表达——chooseToCompare 的暗选同时性以顺序明选近似（登记）。
  载荷分段（header/target/shown/pend）两处读写顺序必须一致——本批 stage 4
  载荷 pc/shown 错位导致 pc 读到展示牌、同卡二次入弃牌堆触发 invariant
  （探针定位后修正）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.43 拼点族状态机第二件：jsrgyangming 养名（2026-09-12 第二百五十三批，tag 727 / ABI 704）

机内拼点子第三次复用：**jsrgyangming 养名**（jsrg noop→approx）——出牌阶段
限一次，与一名角色循环拼点：目标赢 → 目标摸「已输次数」张 + owner 回复
1 点体力结束；否则计一次输，owner 可选择重复（canCompare 不满足自动结束）。

- **机器结构**：`resolve_yangming(player, skill, stage)` 循环续体（Cont 309）
  ——stage 0=SelectTargets 可拼点目标（双方有手牌）→ 2=owner 选拼点牌 →
  3=目标选拼点牌 → 比点+拼点牌入弃牌堆 → 分支（目标赢：draw(losses)+
  recover 1 结束；否则 losses++ → 4=知情 Custom [1,2]）→ 重复回到 stage 2
  （losses 贯穿传递）。
- **首次实现引擎循环**：知情→重复→比点→知情…循环经「同一 Cont + 递进
  载荷」表达，终止条件三分支（目标赢/拒绝/canCompare 不满足）。循环体内
  状态（losses）随载荷每轮传递，不入 storage。
- **注册**：tag 727 / ABI 704（领号+host bump 704）/ 盐 YML / Cont 309
  ResolveYangmingStage；jsrg 包 ABI 703→704（断言同步 9 文件 22 处；
  modifier 127→126、modules 65→66 无既有冻结断言）。
- **契约测试**：`original_yangming_machine.rs` 3 件（重复一轮后目标赢：
  摸已输次数 1 张+回 1 / 知情结束零变化 / owner 手牌耗尽自动结束）。
  测试侧注入已知点数牌保证比点确定性（探针定位 losses 载荷复位 bug）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.44 装备比较族状态机：dccanyun 残殒（2026-09-12 第二百五十五批，tag 728 / ABI 705）

approx/noop 池周期复评散批：**dccanyun 残殒**（sp2 noop→approx）——出牌阶段
可弃置一张牌并选择 1-2 名其他角色，比较装备数：owner 多则其回复 1 体力，
相等则其摸 1 张牌，其多则其失去 1 体力；目标体力为 1 时 owner 摸一张牌。

- **机器结构**：`resolve_canyun(player, skill, stage)` 三段（Cont 310）——
  0=弃置一张手牌（SelectCards 强制费用）→ 2=SelectTargets [1,2] 其他角色
  （可取消）→ 3=逐目标比较装备区数量三分支（recover_hp / draw_cards /
  deal_damage is_lose_hp 体力流失通道）+ 体力 1 判定 owner 摸 1。
- **族定位**：装备数比较族（非拼点）——比较子为「区数量读取 + 三分支」
  （equip zone 长度直读，equipped_count 查询的机内等价）。
- **注册**：tag 728 / ABI 705（领号+host bump 705）/ 盐 CYY / Cont 310
  ResolveCanyunStage；sp2 包 ABI 692→705（断言同步 4 文件；modifier
  241→240、modules 72→73 无冻结断言）。
- **契约测试**：`original_canyun_machine.rs` 3 件（相等→目标摸 1 / owner
  装备多→目标回 1 / 目标装备多→失去 1 体力+体力 1 触发 owner 摸 1）。
- **债（登记）**：①「本回合未选过的角色」跨激活去重未建模（单次激活内
  天然互异）；②体力 1 判定在结算后（上游结算中触发序差）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.45 花色计数族状态机：hengwu 横骛（2026-09-12 第二百五十六批，tag 729 / ABI 706）

花色计数族状态机：**hengwu 横骛**（xianding noop→approx）——锁定技。使用/
打出牌后，若手牌无此花色且场上有同花色装备：摸「场上该花色装备数」张。

- **机器结构**：`resolve_hengwu(player, skill, event_card)` **双触发共调
  单发机器**（twmouzhi 双触发模式复用）——card_used / card_responded 两个
  触发块共调同一单发机器：机内校验事件牌四花色 → 手牌无此花色门控 →
  遍历全场装备区统计同花色装备数 → draw(count)。frequent 锁定为强制自动。
- **注册**：tag 729 / ABI 706（领号+host bump 706）/ 盐 HENW / 单发无续体；
  xianding 包 ABI 697→706（断言同步 1 文件；trigger 162→164、modifier
  390→389、modules 182→184 无冻结断言）。
- **契约测试**：`original_hengwu_machine.rs` 2 件（场上 2 件同花色装备 →
  用牌后摸 2 / 手牌含同花色门控拦截零摸牌）。测试驱动直派
  DomainEvent::CardUsed。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.46 表召族消费面清偿：biaozhao3 表召·归（2026-09-12 第二百五十七批，tag 730 / ABI 707）

批 216 登记债清偿：**biaozhao3 表召·归**（sp2 noop→approx）——准备阶段若
「表」堆有牌：弃置首张，令一名角色摸至全场最大手牌数（至多补 5）并回复
1 点体力。

- **机器结构**：`resolve_biaozhao3(player, skill, stage)` 两段（Cont 311）——
  0=机内校验「表」堆非空（跨技堆字面量 BIAOZHAO_PILE_DEF=3640810944，
  shuliang 先例）→ 弃置首张 → 机内计算全场最大手牌数（旧批 216「全场手牌
  上限查询不可表达」结论随机器工艺失效）→ SelectTargets 任意角色可取消 →
  2=摸 min(差值,5)+recover 1。
- **注册**：tag 730 / ABI 707（领号+host bump 707）/ 盐 BZ3 / Cont 311
  ResolveBiaozhao3Stage；sp2 包 ABI 705→707（断言同步 4 文件；trigger
  64→65、modifier 240→239、modules 73→74 无冻结断言）。
- **DSL 踩坑（再确认）**：触发声明无 `forced` 关键字（缺省即强制，
  `optional` 才是显式标注）——写入 `forced` 报 parse error。
- **契约测试**：`original_biaozhao3_machine.rs` 2 件（真实 biaozhao+biaozhao3
  双源加载：置「表」→ 准备阶段归：弃「表」+补摸至全场最大+回 1 / 空堆
  静默）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.47 衍生技族状态机：chuanyun 穿云（2026-09-13 第二百五十八批，tag 731 / ABI 708）

衍生技族状态机：**chuanyun 穿云**（sp2 noop→approx，「传术」衍生技）——杀
指定目标后若其装备区有牌：随机弃置其中一张。

- **机器结构**：`resolve_chuanyun(player, skill, event_player, event_card)`
  单发无续体（quanjia 精简模板）——机内校验：事件牌为杀（普/火/雷）+ 目标
  存活且装备区非空 → RuleRng gen_range 随机取装备一张 → 移入弃牌堆。
- **注册**：tag 731 / ABI 708（领号+host bump 708）/ 盐 CYN / 单发；
  sp2 包 ABI 707→708（断言同步 4 文件；trigger 65→66、modifier 239→238、
  modules 74→75 无冻结断言）。
- **既有面核查**：discard_target_equipment 系既有面均为特化款
  （gain_mount/draw_weapon_range），无「随机任一装备」通用款——机器补齐。
- **契约测试**：`original_chuanyun_machine.rs` 2 件（双装备随机弃一/
  无装备 noop）。测试驱动：直派 DomainEvent::CardUsed{targets}（引擎由其
  派生逐目标 CardTargeted 上下文）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.48 表召族收官：biaozhao2 表召·应（2026-09-13 第二百五十九批，tag 732 / ABI 709）

表召族收官件：**biaozhao2 表召·应**（sp2 noop→approx）——任意角色弃置与
「表」同花色同点数的牌时：将「表」交给弃牌者，owner 失去 1 点体力。

- **新参数化过滤器（5 点注册零 ABI）**：
  `owner_alive_discard_matches_own_pile_card_<pile_def>`——eval：cards_moved
  事件 → to_zone 为弃牌堆 → 「表」堆非空 → 弃牌中含与堆首张同花色同点数者
  （花色点数双匹配，运行时读堆牌状态）。
- **机器结构**：`resolve_biaozhao2(player, skill, event_player)` 单发无续体
  （quanjia 精简模板）——机内再校验堆非空 → 「表」交予弃牌者（event_player
  存活时入手；无弃牌者时入弃牌堆）→ `deal_damage(is_lose_hp)` owner 失
  1 体力。
- **注册**：tag 732 / ABI 709（领号+host bump 709）/ 盐 BZ2 / 单发；
  sp2 包 ABI 708→709（断言同步 4 文件；trigger 66→67、modifier 238→237、
  modules 75→76 无冻结断言）。
- **契约测试**：`original_biaozhao2_machine.rs` 2 件（匹配弃牌：「表」交予
  弃牌者+owner 失 1 体力+堆清空 / 非匹配弃牌静默）。测试驱动：直派
  DomainEvent::CardsMoved{reason: Discard}。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.49 gaintag 族状态机第二件：dcniji 逆击（2026-09-13 第二百六十批，tag 733 / ABI 710）

gaintag 族状态机第二件：**dcniji 逆击**（sp2 noop→approx）——锁定技双臂：
成为非装备牌目标后摸 1 张牌并打「逆击」标记；结束阶段弃置手牌中全部
「逆击」牌。

- **机器结构**：`resolve_dcniji(player, skill, mode, event_card)` 双触发共调
  单发（twmouzhi 模式 + twmitu gaintag 工艺）——mode 0（card_targeted）：
  机内校验事件牌非装备（equip_slot 为空）→ draw 1 → content_card_tags 打标
  （blake2b("dcniji")=3628735017）；mode 1（phase_end）：遍历手牌收集全部
  标记牌 → 移入弃牌堆。
- **注册**：tag 733 / ABI 710（领号+host bump 710）/ 盐 NJI / 单发；
  sp2 包 ABI 709→710（断言同步 4 文件；trigger 67→69、modifier 237→236、
  modules 76→78 无冻结断言）。
- **契约测试**：`original_dcniji_machine.rs` 2 件（非装备牌目标：摸 1 打标+
  结束阶段全弃 / 装备牌目标 noop）。
- **债（登记）**：「可先使用其中一张」未建模（结束阶段全部直接弃置）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.50 杀目标族状态机：dcmoukui 谋溃（2026-09-13 第二百六十一批，tag 734 / ABI 711）

杀目标族状态机：**dcmoukui 谋溃**（sp2 noop→approx）——杀指定目标后，
chooseButton [1,2] 可双选：摸 1 张牌／弃置目标一张手牌（owner 从目标手牌选）。

- **机器结构**：`resolve_dcmoukui(player, skill, stage, target)` 两段
  （Cont 312）——0（card_targeted owner_source_sha_first_target optional
  触发）→ Custom [1,2] 可双选 → 2=应用：选 1 摸 1；选 2 → SelectCards
  observer=owner 遍历目标手牌 → 3=弃置所选。
- **既有过滤器复用**：`owner_source_sha_first_target`（sha 首目标限定，
  对齐上游 isFirstTarget）。
- **注册**：tag 734 / ABI 711（领号+host bump 711）/ 盐 MKU / Cont 312
  ResolveDcmoukuiStage；sp2 包 ABI 710→711（断言同步 4 文件；trigger
  69→70、modifier 236→235、modules 78→79 无冻结断言）。
- **契约测试**：`original_dcmoukui_machine.rs` 2 件（双选：摸 1+目标手牌
  入弃牌堆 / 仅摸不弃：无弃置轮）。
- **债（登记）**：①both+未命中追罚面留 slice 2（ShaDodged 触发+追罚
  存储）；②多目标杀逐目标触发（上游 isFirstTarget 仅首目标）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.51 chooseToMove 族状态机：starzhongyan 忠言（2026-09-13 第二百六十二批，tag 735 / ABI 712）

chooseToMove 族状态机（多段选择链第四条专项线）：**starzhongyan 忠言**
（sp2 noop→approx）——出牌阶段限一次，亮出牌堆顶三张牌，一名有手牌的角色
以其中一张与其一张手牌交换。

- **机器结构**：`resolve_zhongyan(player, skill, stage)` 三段（Cont 313）——
  0=SelectTargets 有手牌任意角色（可含自己、可取消）→ 亮牌堆顶 3 张
  （RevealCards）→ 3=目标从亮牌选 1（异观察者强制）→ 4=目标选自己手牌
  1 张 → 交换（亮牌入手牌，手牌置回牌堆顶原亮牌位置）。
- **注册**：tag 735 / ABI 712（领号+host bump 712）/ 盐 ZZY / Cont 313
  ResolveZhongyanStage；sp2 包 ABI 711→712（断言同步 4 文件；modifier
  235→234、modules 79→80 无冻结断言；trigger 不变）。
- **契约测试**：`original_zhongyan_machine.rs` 2 件（全链：选目标→选亮牌→
  选手牌→1:1 交换+换出牌回牌堆顶 / 目标选择可取消零变动）。
- **踩坑（再入册）**：①分段载荷构建/解析端布局错位（stage 4 pick 与
  revealed 顺序）——与 qiantun 同类，教训：每新增分段先写布局注释再写
  两端；②牌堆顶=向量尾，亮牌取尾部（qianlong 先例）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.52 狐魅族状态机：dchumei 狐魅（2026-09-13 第二百六十三批，tag 736 / ABI 713）

**dchumei 狐魅**（sp2 noop→approx）——出牌阶段，选择一名体力值不超过 owner
本回合造成伤害总数的角色，令其执行一项：摸一张牌／交给 owner 一张手牌／
回复 1 点体力（仅目标受伤时可选）。

- **机器结构**：`resolve_chumei(player, skill, stage)` 两段（Cont 314）——
  0=机内门控（`turn.damage_dealt_count(owner) > 0` + 目标体力 ≤ 该值，
  引擎既有回合伤害计数直读）+ SelectTargets 1/1 可取消 → 2=Custom 三选一
  （选项动态过滤：give 需目标有手牌、recover 需目标受伤）→ 分支结算
  （draw_cards / give 子选择 / recover_hp）。
- **注册**：tag 736 / ABI 713（领号+host bump 713）/ 盐 CHM / Cont 314
  ResolveChumeiStage；sp2 包 ABI 712→713（断言同步 4 文件；modifier
  234→233、modules 80→81 无冻结断言）。
- **契约测试**：`original_chumei_machine.rs` 4 件（give 臂：目标交牌/
  draw 臂：目标摸 1/recover 臂：目标回 1/零伤害门控静默）。
- **债（登记）**：「每项限一次」收窄为每次发动限选其一（跨发动去重需
  回合级 storage，未建模）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.50 续：谋溃 slice 2 追罚面收口（2026-09-13 第二百六十四批，纯引擎侧零新 ABI）

批 261 登记债①当日清偿：**谋溃追罚面**——双选（摸+弃）后此杀未命中
（ShaDodged）→ 闪避者弃置 owner 一张手牌。

- **实现**：复用 tag 734/ABI 711 同一机器，新增两段——stage 2 双选时
  `storage_set(skill, FLAG, 1)`；stage 5 追罚入口（sha_dodged forced 触发，
  机内校验标记并消耗）→ stage 6 闪避者 SelectCards 弃 owner 手牌。
  stage 0 每次新杀清残留标记。零新 ABI、DSL 仅补一个触发块。
- **载荷/actor 注记**：stage 6 的 actor 为闪避者（payload[4]），observer
  校验下沉至各分段分支（全局 player 校验与异观察者冲突）。
- **契约测试**：`original_dcmoukui_machine.rs` 增至 5 件（双选+闪避→追罚
  命中 / 单选闪避零追罚 / 原有两件保留）。
- **ledger**：以生成器输出为准（noop 轴不变，approx 债注收窄）。

### §8.53 gaintag 族状态机第三件：qinmian 勤勉（2026-09-13 第二百六十五批，tag 737 / ABI 714）

gaintag 族第三件：**qinmian 勤勉**（sp noop→approx）——锁定技三臂：准备
阶段摸 1 张打「勤勉」标；受到伤害后按伤害点数摸牌打标；任意角色结束阶段
可弃手牌中全部标记牌并对当前回合角色造成等量伤害（手牌全部为标记牌时改
为张数）。

- **机器结构**：`resolve_qinmian(player, skill, mode, amount)` 双触发共调
  单发三模式（dcniji 模式复用）——mode 0 双入口（phase_prepare amount=1 /
  damage event_amount）draw+tag；mode 1（phase_end owner_alive optional 确
  认）弃全部标记牌 + deal_damage(current, isAll ? n : 1)。
- **注册**：tag 737 / ABI 714（领号+host bump 714）/ 盐 QMM / 单发；
  sp 包 ABI 693→714（断言同步 7 文件 + sp logic_hash 重录 3 文件；
  trigger 265→268、modifier 477→476、modules 286→289 同步）。
- **契约测试**：`original_qinmian_machine.rs` 3 件（prepare 摸 1 打标/
  damage 按点数摸 2 打 2 标/结束阶段结算：非标记牌存在→伤害 1+标记清空）。
  测试侧踩坑（入册）：**直派 TurnStateChanged 不推进 state.turn.current**——
  测试断言依赖回合角色时必须显式设置 `state.turn.current`。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.54 简单主动族状态机：shuaiyan 率言（2026-09-13 第二百六十六批，tag 738 / ABI 715；批 283 债面收窄 approx→complete）

**shuaiyan 率言**（mobile noop→approx）——弃牌阶段开始，若手牌数大于 1：
选择一名其他角色（有手牌或装备），其交给你一张牌（手牌或装备区）。

- **机器结构**：`resolve_shuaiyan(player, skill, stage)` 两段（Cont 315）——
  0=机内门控（存活 + 手牌>1 + 存在有牌候选）→ SelectTargets 1/1 可取消 →
  2=目标从手牌+装备区选一张（异观察者强制）→ 移入 owner 手牌。
- **注册**：tag 738 / ABI 715（领号+host bump 715）/ 盐 SHY / Cont 315
  ResolveShuaiyanStage；mobile 包 ABI 691→715（断言同步 13 文件：ABI ×12、
  trigger 182→183、modifier 238→237、modules 227→228）。
- **契约测试**：`original_shuaiyan_machine.rs` 2 件（目标交一张牌+owner
  入手 / 取消零变动）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

- **批 283 债面收窄（2026-09-15）**：「展示所有手牌」以既有 `reveal_hand`
  （ABI 336，纯 DSL 零新 bytecode）补全——trigger body 前置
  `effect reveal_hand(self)`，shuaiyan approx→complete；债面仅余确认步
  序（optional 确认先于展示，上游为展示后可取消选目标，可见序列等价）。

### §8.55 标记族状态机：zhengjian 诤荐（2026-09-13 第二百六十七批，tag 739 / ABI 716）

**zhengjian 诤荐**（mobile noop→approx）——锁定技四臂：①结束阶段选一名角色
授一枚「诤」标（blake2b("zhengjian_mark")=2446053157）；②③该角色用牌/
打出牌时其标数 +1（全局 card_used/card_responded 双臂机内计数）；④owner
回合开始：各有标角色摸 min(5, 体力上限, 标数) 张并清标。

- **机器结构**：`resolve_zhengjian(player, skill, stage, event_player)` 四段
  （Cont 316）——0=SelectTargets 任意角色可取消（授标）→ 1/2=计数臂
  （机内直读直加，无选择 inline）；3=结算臂（遍历有标角色 draw+清标）。
- **注册**：tag 739 / ABI 716（领号+host bump 716）/ 盐 ZJZJ / Cont 316
  ResolveZhengjianStage；mobile 包 ABI 715→716（断言同步 13 文件；
  trigger 183→187、modifier 237→236、modules 228→232）。
- **契约测试**：`original_zhengjian_machine.rs` 2 件（授标→回合开始结算：
  other 摸 1 + 清标 / 授标取消零变动）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.57 游戏开始发放族状态机：mbbeiming 孛明（2026-09-13 第二百六十九批，tag 742 / ABI 718）

**mbbeiming 孛明**（mobile noop→approx）——游戏开始时（global phaseBefore
phaseNumber==0 / player enterGame 双源合一为 game_started），选至多两名任意
角色，各获得牌堆中一张攻击范围等于其手牌花色数的武器牌；无 distance 信息
的武器视作范围 1（引擎 weapon_range 字段直接承载该值）；找不到匹配武器则
跳过该目标。

- **机器结构**：`resolve_mbbeiming(player, skill, stage)` 两段（Cont 318
  ResolveMbbeimingStage）——0=SelectTargets [1,2] 任意存活角色可取消 →
  2=解析器直调 `resolve_mbbeiming_grant`（逐目标 suit 去重计数 → 牌堆顶向下
  首个 `kind==Weapon && weapon_range==suits` 置入其手牌区，跳过无匹配者）。
- **注册**：tag 742 / ABI 718（领号+host bump 718）/ 盐 BBM 0x4242_4D /
  Cont 318；mobile 包 ABI 717→718（断言同步 12 处；trigger 189→190、
  modifier 235→234（noop 声明移除）、modules 234→235）。
- **契约测试**：`original_mbbeiming_machine.rs` 4 件（双目标各获匹配武器
  且牌堆 -2 / 无匹配种子跳过零变动 / 取消零变动 / 包契约 ABI 718 +
  game_started 非 optional）。
- **ledger**：以生成器输出为准（本批 -1 noop）。

### §8.58 连环状态族原语：duansuo 断索（2026-09-13 第二百七十批，tag 743 / ABI 719）

**duansuo 断索**（mobile noop→complete）——出牌阶段限一次，重置任意名处于
连环状态的角色，然后对这些角色各造成 1 点火焰伤害。上游 selectTarget
[1, Infinity] + filterTarget isLinked()（含自己）；content 两遍循环：先全体
link(false) 重置、再全体 damage("fire")——重置完成后伤害不经铁索传导。

- **原语结构**：`ResetLinkedSelectedTargets(source, targets[])`（tag 743，
  无续体）——VM 从 host.selected_targets() 取全部所选目标；引擎侧
  `reset_linked_selected_targets` 先移除所选全部 MARK_CHAIN
  （PlayerLinkedChanged 随之派发），再逐目标 deal_damage(nature=fire)。
- **DSL**：`active play_phase uses_per_turn 1 targets 1 255 linked`——
  linked 目标过滤器为既有 targets_spec 能力（批 270 首用），零选择器成本。
- **注册**：ABI 719（领号+host bump 719）/ tag 743 / hash 374（source +
  put_ids(targets) 形状）；无续体、无 resolve_choice 路由（13 文件注册链）。
- **契约测试**：`original_duansuo_active.rs` 3 件（双目标重置+各 -1 火伤、
  未选连环第三者不经链传导且保持连环 / linked 过滤器拒未连环、零目标拒、
  每回合限一次二次拒 / 包契约 ABI 719 + active linked min 1）。
- **登记债**：ABI-COORDINATION 718 行曾因中断续传重复登记（两种措辞各一），
  本批去重；领取脚本以后必须断言旧行计数 ≤1。

### §8.59 杀目标转移族状态机：gaoyuan 告援（2026-09-13 第二百七十一批，tag 744 / ABI 720）

**gaoyuan 告援**（mobile noop→approx）——成为【杀】的目标时，可弃置一张牌
将此【杀】转移给一名有「诤」且不是此【杀】使用者的其他角色（须为该杀的
合法目标，含射程检查；与批 267 zhengjian「诤」标联动，上游 combo:zhengjian）。

- **机器结构**：`ResolveGaoyuan(player, source, card)`（tag 744，Cont 319
  ResolveGaoyuanStage 两段）——0=选弃牌（0..1 可取消，手牌+装备区）→
  1=选转移目标（1..1 可取消，候选=`saved_sha_redirect_targets_in_attack_range
  (source 射程)`∩持 ZHENGJIAN_MARK）→ 2=弃牌 +
  尾随 `CommitGaoyuanRedirect`（引擎内部变体，无 bytecode tag）提交
  `redirect_saved_sha_target` + CardTargetAdded，弃牌先于转移与上游一致。
  候选复用 liuli 合法性管道、射程基准从 owner 改为 source（告援语义），
  故提交不走 ChooseDiscardRedirectCurrentSha。
- **触发过滤器**：`owner_sha_target_can_discard_redirect_to_zhengjian_marked`
  （catalog/dispatch/registry 三点注册零 ABI 门控）——liuli 过滤器变体：
  候选存在性=持诤标 ∩ source 射程 ∩ 未被取消 ∩ 非 owner/source。
- **注册**：ABI 720（领号+host bump 720）/ tag 744 / hash 154+盐 GY /
  Cont 319；compiler FILTERS 白名单与 required-rule-vm-abi 阶梯同步补
  rung（filter 白名单漏配曾致 pack 失败——新过滤器需同时登记三处）。
- **契约测试**：`original_gaoyuan_machine.rs` 3 件（真实 PlayCard 管道下
  转移成功：owner 免伤/candidate -1/cost 入弃牌堆/CardTargetAdded 事件、
  未标记者不出现在候选 / 取消后原目标掉血 cost 留手 / 包契约 ABI 720 +
  filter 名）。
- **登记债**：①targetEnabled 以「source 射程内+未取消」近似；②logSkill
  表现面未建模；③「取消后 card_targeted 重复交付」（批 282 更正：系夹具
  双重加载假象，单一加载无重复，原登记撤回）。

### §8.60 杀目标多段族状态机：duoduan 度断（2026-09-13 第二百七十二批，tag 745 / ABI 721）

**duoduan 度断**（mobile noop→approx）——每回合一次，成为【杀】的目标后可
重铸一张牌；再选一项：①令使用者摸两张牌且此【杀】无效（对 owner）；
②令使用者弃置一张牌且 owner 不能响应此【杀】。上游 trigger target
useCardToTargeted + usable 1；content recast → chooseControl 双选（使用者
无可弃牌时强制①）→ 分支结算。

- **机器结构**：`ResolveDuoduan(player, source, card, skill)`（tag 745，
  Cont 320 ResolveDuoduanStage 三段）——0=选重铸牌（1..1 可取消）→
  1=RecastCards+Custom[1,2] 双选（使用者无可弃牌强制①）→ 2=①draw 2+
  `exclude_saved_card_target` 内联排除／②为 source 开强制弃牌选择 →
  3=DiscardCards+`make_saved_sha_target_unrespondable`。
- **关键坑（入册）**：①「无效」排除必须在选择解析期内**同步**落 sha 挂起
  载荷——经 CancelIncomingCardUse 尾随提交会在响应询问之后才落地（静默
  无效），且该原语自带「获得实体牌」的杜达语义，与度断不符，故改用
  `exclude_saved_card_target` 直改；②CancelIncomingCardUse 的
  originating_skill_is_enabled 门对 SkillId::NONE 恒 false——Rust 侧直构
  该请求必须传真实技能实例 id（DSL 路径由 vm 从 host.context() 填充）；
  ③排除后引擎仍可能交付一次响应窗口询问（解决即可，无结算影响）。
- **触发过滤器**：`owner_is_target_sha_source_alive_hand_equip_nonempty_
  once_per_turn`——eval 以 `turn_fires == 0` 实现每回合一次（五处登记：
  catalog/dispatch/registry/compiler FILTERS/required-abi 阶梯 rung）。
- **注册**：ABI 721（领号+host bump 721）/ tag 745 / hash 154+盐 DDN /
  Cont 320。
- **契约测试**：`original_duoduan_machine.rs` 4 件（真实 PlayCard 管道：
  选项①使用者摸 2+owner 免伤 / 选项②使用者弃 1+owner 掉血 /
  取消重铸正常结算+cost 留手 / 包契约 ABI 721+filter 名）。
- **登记债**：①「无效」排除后引擎仍交付一次空响应询问；②recastable 以
  全部可弃手牌/装备近似（未建卡面白名单）。

### §8.61 简单主动+手牌数比较族状态机：dcanxu 暗蓄（2026-09-13 第二百七十三批，tag 746 / ABI 722）

**dcanxu 暗蓄**（refresh noop→approx，OL界李儒）——出牌阶段限一次，选两名
手牌数不同的角色，手牌少者从多者处选得一张手牌；所得牌非黑桃则 owner 摸
1；此后两人手牌数相等则 owner 回 1。

- **机器结构**：`ResolveDcanxu(player, targets[], skill)`（tag 746，Cont 321
  ResolveDcanxuStage 单段）——入口校验两目标手牌数不同并归一化少者/多者 →
  少者明选多者一张手牌（SelectCards 1..1 强制，观察者=少者）→ 同步移牌 →
  非黑桃 owner draw 1 / 相等 owner recover 1（DrawCards/RecoverHp 既有
  请求，run_effects 直调）。
- **目标传递**：沿用批 270 `host.selected_targets()` 模式——active 的多目标
  选择经 VM 上下文进入机器，无需逐目标 DSL 实参。
- **触发过滤器**：无（主动技）。
- **注册**：ABI 722（领号+host bump 722）/ tag 746 / hash 154+盐 DCX /
  Cont 321；refresh 包 ABI 721→722（该包无精确计数断言，仅 `>=` 宽松
  断言，无需重录）。
- **契约测试**：`original_dcanxu_active.rs` 4 件（黑桃所得+手牌数相等：
  owner 回 1 / 非黑桃所得+不相等：owner 摸 1 不回复 / 手牌数相同激活被拒 /
  包契约 ABI 722+active 2..2 other）。
- **登记债**：①「手牌数不同」约束在机器入口校验（激活即拒），目标选择器
  未提供该过滤面；②visibleMove 明选表现面未建模（选牌天然可见）。

### §8.62 简单主动逐目标族状态机：chulao 黜老（2026-09-13 第二百七十四批，tag 747 / ABI 723）

**chulao 黜老**（refresh noop→approx，费祎）——出牌阶段限一次，弃一张牌
（成本牌黑桃则 owner 摸 1），对任意名势力各异角色逐个由 owner 选弃其一张
牌（被弃牌黑桃则该目标摸 1）。上游 selectTarget [1, Infinity] +
filterTarget 势力各异/非 unknown/有牌可弃；content step0 成本判定 +
逐目标 choosePlayerCard→discard→黑桃摸 1。

- **机器结构**：`ResolveChulao(player, targets[])`（tag 747，Cont 322
  ResolveChulaoStage 逐目标循环续体）——入口校验势力各异
  （`state.player_group` 非空且互异）→ 0=owner 选成本牌（1..1 可取消）→
  1=弃成本+黑桃 owner 摸 1+对首目标开选牌 → 2=弃目标牌+黑桃目标摸 1+
  idx 前进循环（payload 尾部携带目标表 `[magic, stage, owner, cost, idx,
  t0…tn]`，index 4 为游标）。无可弃目标跳过（上游中断语义的近似）。
- **触发过滤器**：无（主动技）。
- **注册**：ABI 723（领号+host bump 723）/ tag 747 / hash 154+盐 CLC /
  Cont 322；refresh 包 ABI 722→723（断言同步 7 处，含批 273 新增
  dcanxu 测试；active 17→18、modifier 205→204、modules 166→167）。
- **契约测试**：`original_chulao_active.rs` 3 件（双目标全链：成本黑桃
  owner 摸 1+逐目标弃牌入弃牌堆+非黑桃不摸 / 势力互异激活被拒+取消成本
  零结算 / 包契约 ABI 723+active 1..N other）。
- **登记债**：①「势力各异」在机器入口校验（激活即拒），选择器无该过滤
  面；②choosePlayerCard 中断分支以跳过无牌目标近似。

### §8.63 点数比较族状态机：friendbiwei 鄙位（2026-09-13 第二百七十五批，tag 748 / ABI 724）

**friendbiwei 鄙位**（mobile noop→approx，友系列）——出牌阶段限一次，弃置
手牌中点数最大的一张，令一名其他角色弃置所有点数≥该牌的手牌；若其未因此
弃牌，重置此技能（可再次发动）。上游 filterCard 点数≥其余手牌最大值；
content 弃成本 → target 弃全部 ≥rank 手牌（无则 delete getStat("skill")）。

- **机器结构**：`ResolveFriendbiwei(player, target, card, skill)`（tag 748，
  单发无续体）——入口校验成本牌为 owner 手牌最大点数 → 弃成本 → 目标弃
  所有 rank≥成本牌的手牌 → 无弃牌时 `use_count=0 && play_phase_use_count=0`
  同回合退款重发。成本牌经 `host.selected_cards()`（select_hand 选择器），
  target 经 selected_targets。
- **关键坑（入册）**：每回合限制的门是 `inst.use_count`（uses_per_turn），
  `play_phase_use_count` 只对应 uses_per_play_phase——「重置技能」必须清
  use_count 才能退款生效。
- **触发过滤器**：无（主动技）。
- **注册**：ABI 724（领号+host bump 724）/ tag 748 / hash 154+盐 FBW；
  mobile 包 ABI 721→724（断言同步 15 处；active 46→47、modifier 231→230、
  modules 238→239）。
- **契约测试**：`original_friendbiwei_machine.rs` 3 件（目标弃全部 ≥rank
  牌、<rank 留手 / 无弃牌退款后同回合可再次发动 / 非最大点数成本被拒）。
- **登记债**：①「最大点数」约束在机器入口校验；多张同为最大点数时任选
  其一（与上游 filterCard 行为一致，非债）。

### §8.64 杀目标目标数门槛族状态机：mbzhennan 镇南（2026-09-13 第二百七十六批，tag 749 / ABI 725）

**mbzhennan 镇南**（mobile noop→approx）——当你成为其他角色使用的锦囊的
目标后，若此牌的目标数大于 max(1, 该角色体力值)，你可弃一张牌并对任意
角色造成 1 点伤害。上游 trigger global useCardToPlayered（isFirstTarget、
targets 含 owner、type2=trick、targets.length > max(1,source.hp)）；
cost chooseCardTarget（filterTarget:true=任意角色）→ 弃牌 + DealDamage 1。

- **机器结构**：`ResolveMbzhenann(player)`（tag 749，Cont 323 两段）——
  0=选成本牌（1..1 可取消）→ 1=选任意存活角色（1..1 强制）→ 2=弃牌+
  DealDamage。伤害目标含 owner 本人在内（上游 filterTarget:true 原义）。
- **触发过滤器**：`owner_is_target_trick_targets_gt_source_hp_hand_equip_
  nonempty`——eval 含 `ctx.targets.len() > max(1, source.hp)` 门槛（目标数
  门槛为首例；isFirstTarget 由 per-target 事件分派天然满足）。
- **注册**：ABI 725（领号+host bump 725）/ tag 749 / hash 154+盐 ZNN /
  Cont 323；mobile 包 ABI 724→725（断言同步 15 处；trigger 192→193、
  modifier 230→229、modules 239→240）。
- **契约测试**：`original_mbzhennan_machine.rs` 3 件（真实 PlayCard 管道
  南蛮 3 人局：弃牌→对 third 伤害、南蛮自身伤害并行计入 / 取消后目标仅
  受南蛮伤害 / 包契约 ABI 725+filter 名）。
- **登记债**：①锦囊类型以 CardKind::is_trick 近似（上游 type2 的延迟
  锦囊边界由事件面决定）。

### §8.65 杀目标后续族状态机：meiyong 姝勇（2026-09-14 第二百七十七批，tag 750 / ABI 726）

**meiyong 姝勇**（mobile noop→complete，无省略面）——当你使用或打出【杀】
时，可获得一名其他角色区域内（hej=手牌+装备+判定）的一张牌（owner 选牌），
然后其摸一张牌。上游 inherit xinfu_wuniang（trigger player useCard/respond，
filter card.name=sha）。

- **机器结构**：`ResolveMeiyong(player)`（tag 750，Cont 324 两段）——
  0=选目标（其他 hej 区有牌者，1..1 可取消）→ 1=owner 选其区域一张牌
  （1..1 强制）→ 2=移入 owner 手牌 + 目标 DrawCards 1。
- **双触发**：card_used `owner_is_source_sha_used` + card_responded
  `owner_is_responder_sha` 双臂共调同一机器（响应臂 ctx.player=打牌者，
  card_kind=response_kind）。
- **注册**：ABI 726（领号+host bump 726）/ tag 750 / hash 154+盐 MY /
  Cont 324；mobile 包 ABI 725→726（断言同步 16 处；trigger 193→195 双臂、
  modifier 229→228、modules 240→242）。
- **契约测试**：`original_meiyong_machine.rs` 3 件（真实打出【杀】链路：
  候选仅 hej 有牌者、判定区牌可取、移牌+目标摸 1 / 取消后零变动 /
  包契约 ABI 726+双触发臂）。
- **登记债**：无。

- **批 285 勘误（2026-09-15）**：头注「近似（登记债）：无」触发 ledger
  approx 分类（批 270 duansuo 同款措辞陷阱）——更正为「保真度：完整」，
  meiyong approx→complete。零代码变更（logic_hash 不变，npack 零重建）。
### §8.66 伤害门控+点数族状态机：mbqiangyong 羌勇（2026-09-14 第二百七十八批，tag 751 / ABI 727）

**mbqiangyong 羌勇**（mobile noop→approx）——锁定技，你使用【杀】造成伤害
时，弃置受伤角色的 X 张牌（X=本回合你使用【杀】的次数，owner 从目标 he 区
选牌强制弃置）；然后若其没有手牌，此【杀】伤害 +1。上游 forced，trigger
source damageBegin2（card=sha）；num=countHistory("useCard",sha)；
discardPlayerCard({selectButton:num, forced}) → trigger.num++。

- **机器结构**：`ResolveQiangyong(player, target, skill)`（tag 751，Cont 325
  单段续体）——X=SHA_PLAYED 回合旗；X>0 且目标有可弃牌 → owner SelectCards
  X..X 强制弃置 → 目标无手牌时 `ModifyCurrentDamage(+1)`；否则直判 +1。
  skill 经 host.context() 传递（ModifyCurrentDamage 需真实实例 id）。
- **关键坑（入册）**：①引擎对同一伤害可能重复交付 damage_before 事件——
  机器入口以 skill storage 记录已处理的 current_damage_id 幂等去重（damage_id
  在 run_pending_triggers 前已赋值，可靠）；②「重置技能」退款门是 use_count
  （批 275 同源教训重申：uses_per_turn ↔ use_count）。
- **触发过滤器**：`owner_is_source_sha_card_damage_before`（五处登记）。
- **注册**：ABI 727（领号+host bump 727）/ tag 751 / hash 154+盐 QY /
  Cont 325；mobile 包 ABI 726→727（断言同步 17 处；trigger 195→196、
  modifier 228→227、modules 242→243）。
- **契约测试**：`original_qiangyong_machine.rs` 3 件（真实 PlayCard 管道：
  X=1 弃 1 目标仍有手牌无加成 / 弃光手牌伤害 +1 / 包契约 ABI 727+filter 名）。
- **登记债**：①X 以 SHA_PLAYED 回合旗近似（owner 出牌回合内等价；跨回合
  残留伤害场景不区分）。

### §8.67 approx 升级：mbzhenfu 镇抚门槛补全（2026-09-14 第二百七十九批，ABI 728 / 零 bytecode）

**mbzhenfu 镇抚**（mobile approx 门槛补全）——结束阶段，若你本回合因弃置
失去过牌，可令一名其他角色获得 1 点护甲。批 279 补上缺失的门槛：新触发
过滤器 `owner_phase_end_lost_cards`（own 结束阶段 +
`card_loss_count(owner) > 0`，复用既有回合级失牌历史）。

- **过滤器机制确认（入册）**：`card_loss_count` 为回合级 per-player 失牌
  计数，但只在 `content.tracks_turn_card_loss_history()` 开启时记录——该
  开关由包加载时扫描 trigger filter 自动置位（registry.rs
  `track_turn_card_loss_history |= matches!(...)` 列表），新增过滤器加入
  该列表即全局生效。
- **manifest ABI 澄清（入册）**：`required_rule_vm_abi` 的 filter-rung 链
  对 module_required≥677 的现代包是死代码（declarative 分支被 677 阈值
  截断）——filter-only 批次的 npack manifest 不上升（保持 727），仅 host
  bump（version.rs 728）+ 领取行生效；npack 727 ≤ host 728 合规。
- **注册**：ABI 728（领号+host bump 728）/ 无 bytecode / 无续体；过滤器
  五处登记（catalog/dispatch/registry+opt-in 列表/compiler FILTERS+ladder
  rung）。
- **测试**：`original_choose_other_add_mark.rs` 增补（4 件全过）：既有
  镇抚链路补「先弃一张」前置 + 新增无失牌门槛压制件。
- **登记债**：①「因弃置失去」以「本回合失去过牌」超集近似
  （card_loss_count 含赠予/被获得等失去方式，口径略宽）。

### §8.68 缺口铸造：damage_before 触发期部分减伤桥 + rw_baiyin 迁移（2026-09-14 第二百八十/二批，桥 ABI 731 / 零 bytecode）

**rw_baiyin_skill 照月狮子盔**（mobile noop→approx）——arm1「受到 >1 伤害
时伤害改为 1」以 `damage_before owner_is_target_damage_amount_gt_one` +
`let delta = 1 - event_amount; modify_current_damage(self, delta)` 纯 DSL
落地；arm2「失去装备区…回复1摸2」因【照月狮子盔】实体牌不在引擎牌池
（mobile 无 cards.json 条目）为死代码（登记债）。

- **缺口铸造**：批 280 实测 ModifyCurrentDamage 负 delta 路径在触发执行期
  检索 deferred_tail_effects 失败（bound CommitDamage 尚未入列——触发 VM
  在 run_pending_triggers 内联执行，先于 defer_damage_tail）。批 282 铸造
  `pending_bound_damage_deltas`（Resolution 回合态字段，桥 ABI 731）：
  increase/reduce 检索失败时把增量暂存该队列，`defer_damage_tail` 在
  prepared CommitDamage 入列时统一套用（increase 同步 amount+base_amount、
  reduce 仅 amount）。
- **根因更正（重要，入册）**：批 276/278 在册的「引擎对同一事件重复交付
  触发」结论**有误**——真实根因是测试夹具**双重加载**（同时载入 npack 与
  同一技能的独立编译源，同一 skill_def 注册两份触发定义，触发成对触发）。
  单一加载下无重复交付（rw_baiyin 实测一次触发、一次套用）。夹具规则
  （入册）：测试需同时覆盖 npack 契约与行为时，行为机用 npack 内技能
  （AddSkill 后即有触发），不得再 load 同源编译包；qiangyong 的 storage
  守卫与 mbzhennan 的 drain 循环即为该假象的补丁，无害保留。
- **注册**：桥 ABI 731（领号+host bump 731）；过滤器
  `owner_is_target_damage_amount_gt_one` ABI 729（批 280 预备）；jsrg
  包（批 281）不受影响；mobile npack manifest 保持 727（filter/桥均无
  bytecode）。
- **契约测试**：`original_rw_baiyin_machine.rs` 3 件（真实 PlayCard 管道：
  酒+杀伤害 2 封顶为 1 / 伤害 1 不触发 / 无技能方不受封顶）。
- **登记债**：①arm2 卡牌不在牌池（死代码）；②改伤语义以 delta 调整近似
  （nature/归属不变，与上游「改为1」一致）。
### §8.71 债面收窄：dcqiaolve 弃牌代价补全（2026-09-15 第二百八十六批，tag 753 / ABI 732）

**dcqiaolve 巧略**（newjiang approx 债面收窄）——原实现缺「弃置一张牌」代价
（取目标手牌为无代价）。批 286 以单段机器
`ResolveDcqiaolve(player)`（tag 753，Cont 327）补全：owner 选成本牌
（1..1 强制）→ 弃置 → TakeOneHandCard 取事件目标一张随机手牌。
原 `take_one_event_target_hand` 裸调用（from=NONE 无事件上下文即静默空转）
替换为显式机器传递 target。

- **注册**：ABI 732（领号+host bump 732）/ tag 753 / hash 154+盐 QL /
  Cont 327；newjiang 包 ABI 730→732（yizu 精确 pin 695→732 同步——
  该 pin 停留在批 269 前的旧值，为换包扫描法在 newjiang 包的首次实战）。
- **登记债**（重构后）：①首现同名牌过滤未建模；②多目标 chooseTarget
  未建模；③取牌为随机手牌近似（上游 owner 自选 he 区）。
### §8.70 approx 债面更正：reliangyin 良姻（2026-09-15 第二百八十四批，零代码变更）

原登记债「置入/取出次数不设限（上游每回合一次，标记去重未建模）」经上游
def 全文复核（mobile/skill.js reliangyin L915-1050）证伪——定义无
usable/limited/once 字段，触发次数本就不设限，引擎行为与上游一致。
债面清空，reliangyin approx→complete（头注更正后 logic_hash 不变——
头注不进编译，npack 零重建零部署）。

### §8.69 伤害后续族状态机：jsrgdingce 定策（2026-09-14 第二百八十一批，tag 752 / ABI 730）

**jsrgdingce 定策**（jsrg noop→approx）——当你受到伤害后，你可以依次弃置
你与伤害来源的各一张手牌；若这两张牌颜色相同，你视为使用一张【洞烛先机】。
上游 trigger player damageEnd（source 在场、owner 有可弃手牌）；cost
chooseToDiscard 自身一张手牌（可取消）→ content 弃置 →
discardPlayerCard({target, forced, "h"}) owner 代来源选弃 → 同色
chooseUseTarget 虚拟【洞烛先机】。

- **机器结构**：`ResolveDingce(player, source)`（tag 752，Cont 326 两段）——
  0=owner 选自己一张手牌（1..1 可取消）→ 1=弃置 → 来源有可弃牌时 owner
  代选来源一张手牌（1..1 强制）→ 2=弃置来源牌。复用既有过滤器
  `owner_is_target_source_alive`（零过滤器注册）。
- **注册**：ABI 730（领号+host bump 730）/ tag 752 / hash 154+盐 DCE /
  Cont 326；jsrg 包 ABI 704→730（断言同步 10 文件：abi704 ×23 处 +
  trigger 55→56 ×2；jsrg 计数 trigger 55→56、modifier 126→125、
  modules 66→67）。
- **契约测试**：`original_dingce_machine.rs` 3 件（真实 DamageDealt 管道：
  双方各弃一张入弃牌堆 / 取消零结算 + 来源无手牌早退 / 包契约 ABI 730 +
  filter 名）。
- **登记债**：①同色分支的虚拟【洞烛先机】未结算——该牌不在引擎牌池
  （jsrg 无 cards.json 条目），触发面为死代码，待卡牌入库后补建；
  ②来源弃牌面以 he 口径近似（上游 position "h"）。

### §8.73 随机获得族状态机：yixiang 义襄（2026-09-15 第二百八十八批，tag 755 / ABI 734）

**yixiang 义襄**（mobile noop→approx），每名角色的回合限一次，当你成为
一名角色使用牌的目标后，若该角色的体力值大于你的体力值，你可以随机获得
牌堆里的一张你没有的基本牌。

- **机器结构**：ResolveYixiang(player)（tag 755，单发无续体），搜索牌堆
  顶向下第一张 kind.is_basic() 且 hand_kinds 不含的牌，直接移入 owner
  手牌区。每名角色的回合限一次由 filter 内 turn_fires==0 承载。
- **触发过滤器**：owner_is_target_source_hp_gt_owner_per_role_turn
  （五处登记：catalog/dispatch/registry/compiler FILTERS/ladder rung；
  eval = owner_alive + owner_is_player + source_alive + CardTargeted +
  source.hp > owner.hp + turn_fires == 0）。
- **注册**：ABI 734（领号+host bump 734）/ tag 755 / hash 154+盐 YX；
  mobile 包 ABI 733→734。
- **登记债**：（1）你没有的基本牌以手牌中不存在的 kind 去重近似（不按
  牌名去重）；（2）每名角色的回合限一次由 turn_fires==0 自然承载（正确
  但粒度为整个角色回合而非独立计时器）。

### §8.74 modifier-only 补面：jilei + chuiti（2026-09-15 第二百九十批，零 bytecode）

**jilei**（sp，noop→complete），**chuiti**（sp，noop→complete）——均为
modifier-only 锁定技，以纯 DSL 表达零 bytecode 零新原语。jilei 使用
`modifier hand_limit add 2`；chuiti 使用
`modifier hand_limit add owner_lost_hp`（既有 modifier 值源）。

- **注册**：零新 ABI（modifier 值源均为既有），零新 bytecode tag，
  零新续体。
- **契约测试**：sp 包既有测试覆盖 manifest 和 modifier_index 计数。
- **方法备忘**：modifier-only noop 不需要完整状态机——只需确认上游效果
  与既有 modifier 值源匹配，即可纯 DSL 迁入 complete。

### §8.75 濒死交牌循环状态机：xinfu_qiai 求援（2026-09-15 第二百九十一批，tag 756 / ABI 735）

**xinfu_qiai**（sp，noop→complete）——限定技，当 owner 进入濒死状态时，
其他所有角色依座次各交给 owner 一张牌（手+装备区，强制；无牌者跳过）。

- **机器结构**：ResolveXinfuQiai(player)（tag 756，entry+续体循环），
  entry 扫描首个 giver（座位序自 owner 之后环绕，跳过 owner/死亡/无 he
  牌者）开强制 SelectCards 1..1；ResolveXinfuQiaiStage（Cont 328）续体
  移交所选牌至 owner 手牌区（MoveCards）并循环下一 giver。
- **限定技门控**：复用既有过滤器 owner_dying_zhupeilan_once_per_game
  （Dying + owner 濒死 + lifetime_fire_count==0），零新过滤器注册。
- **注册**：ABI 735（领号+host bump 735）/ tag 756 / hash 154+盐 FQ；
  sp 包 ABI 714→735（编译器按所需效果自动计算）。
- **登记债**：座位序按存活者序环绕近似上游 sortBySeat 快照（循环中途
  死亡的 giver 逐段重扫时自然跳过，语义一致）。
- **方法备忘**：上游 dying trigger + limited + forced chooseCard 的
  「逐角色交牌」结构与 chulao 逐目标循环同构；chooser 逐段变化的循环
  把「当前 giver」存进续体 payload 即可（observer 校验对 payload giver）。

### §8.76 条件多目标摸牌状态机：olwenren 温仁（2026-09-15 第二百九十二批，tag 757 / ABI 736）

**olwenren**（sp，noop→complete）——出牌阶段限一次，任选任意名目标
（可含自己），每名目标若没有手牌摸 1 张、若其手牌数不大于 owner 再摸
1 张（可叠加为 2，可摸 0）。

- **机器结构**：ResolveOlwenren(player, targets[])（tag 757，单发无续
  体）——目标按座位序去重过滤存活，逐目标「先判后摸」（上游逐目标
  await draw 同序）；uses_per_turn 1 表达 usable:1（命令级权威限次）。
- **目标注入**：vm 臂经 host.selected_targets() 读取 active 技能目标
  （DrawSelectedTargets 同工艺）；vm_host 附注 ctx.skill_id；effect/mod
  审计 match 收录（skill 字段入审计）。
- **注册**：ABI 736（领号+host bump 736）/ tag 757 / hash 158 列表形
  （player+targets+skill+盐 WR）；sp 包 ABI 735→736（编译器自动计算）。
- **登记债**：无——判定基线 owner_hand 于循环前一次读取（上游 owner
  手牌在结算期不变，语义等价）。
- **方法备忘**：上游 enable+usable+filterTarget+selectTarget[1,∞] 的
  「任意多目标条件结算」主动技 = active uses_per_turn + targets 1 255
  + 单发机器（遍历 selected_targets），零续体零选择，比逐目标选择型
  （chulao）轻一档。

### §8.77 觉醒换技族状态机：spolaige 霸业（2026-09-15 第二百九十三批，tag 758 / ABI 737）

**spolaige**（sp，noop→complete）——觉醒技：全局第二次濒死结算后，失去
〖西向〗获得〖朔北〗，手牌摸至体力上限并回复体力至体力上限。

- **全局濒死计数器**：GameState 新字段 lifetime_dying_events（serde
  default u32，replay 兼容），在 effect/mod.rs 的 EnteredDying 事件排水
  循环（维护 lifetime_dying_players 同处）递增；hash.rs 状态摘要收录。
- **触发过滤器**：owner_alive_second_global_dying（五点注册：catalog/
  dispatch/registry/compiler FILTERS/lsp hover；eval = owner_alive +
  DyingAfter + lifetime_dying_events==2；lib.rs ladder rung ABI 737）。
  计数==2 天然一次性（取代上游 awakenSkill 的 juexingji 限次保护）。
- **机器结构**：ResolveSpolaige(player)（tag 758，单发无续体）——换技
  由 AddSkill(朔北 3309571556) + RemoveSkillByDef(西向 2531785939) 复合；
  摸/回至上限以机内差值表达（max_hp.saturating_sub(hand_count) 与
  i64 差值，零新原语；上游 drawTo/recoverTo 等价量）。
- **语义澄清（入册）**：引擎 DyingAfter ← LeftDying（被救回脱离濒死）
  发出；无救死亡路径无 LeftDying（上游 dyingAfter 同口径——死亡触发
  die 而非 dyingAfter）。测试须以 RecoverHp 救回驱动完整濒死窗口。
- **注册**：ABI 737（领号+host bump 737）/ tag 758 / hash 154+盐 PL；
  sp 包 ABI 736→737（编译器自动计算）。
- **登记债**：获得的〖朔北〗本体尚为 noop 空表面（待后续批次重写）；
  觉醒动画/皮肤切换不建模。

### §8.78 结束阶段射程伤害状态机：olzhijie 直谏（2026-09-15 第二百九十四批，tag 759 / ABI 738）

**olzhijie**（sp，noop→complete）——锁定技：一名角色的结束阶段，若你本
回合使用或打出过牌、且本回合被选择为牌的目标、且射程内有角色：选射程
内一名角色对其造成 1 点伤害（forced；上游 damage() 来源=owner 同口径）。

- **新 track**：turn_flag `card_targeted_count_by_player`（基 3300）+
  registry `track_card_target_history` 门控（opt-in，旧包 hash 不变）+
  dispatch 循环 CardTargeted ctx 记录点（plan 前，per-target 记一次）。
- **触发过滤器**：owner_used_and_was_targeted_this_turn_in_range（五点
  注册 + lib.rs rung ABI 738；eval = owner_alive + PhaseEnd + 全局任意
  角色 + card_action_count>0（既有用/打出双计）+ card_targeted_count>0
  （新 track）+ all_other_players_in_attack_range（既有射程查询））。
- **机器结构**：ResolveOlzhijie(player)（tag 759，Cont 329 单段续体）——
  entry 选射程内目标（SelectCandidates=player_in_attack_range 过滤，
  强制 1..1）→ 续体机内复验射程 + DealDamage 1（来源 owner）。
- **注册**：ABI 738（领号+host bump 738）/ tag 759 / hash 154+盐 ZJ；
  sp 包 ABI 737→738（编译器自动计算）。
- **登记债**：无——三条件与伤害来源全部按上游语义建模。
- **测试工艺备忘**：phase_end 触发事件 = TurnStateChanged(End)（而非
  PhaseEnded——后者映射 TurnEnded）；dispatch 产出的请求须 run_effects
  执行，open_choice 以 Err(ChoiceRequested) 短路属正常信号。

### §8.79 牌堆装备检索组合状态机：olsuyi 俗义（2026-09-15 第二百九十五批，tag 760 / ABI 739）

**olsuyi**（sp，noop→complete）——出牌阶段限一次，选一名其他角色：牌堆
检索一张其可装备的装备牌令其获得并装备之；然后对其视为使用一张【杀】
（完整结算，无来源消耗）。

- **机器结构**：ResolveOlsuyi(player, target)（tag 760，单发无续体）
  三段：①牌堆顶向下检索第一张 kind.is_equip() 且
  equip_slot_enabled（槽位门控，jiebing can_use 同工艺）；②MoveCards
  入目标手牌区 + use_equipment_from_hand（获得+装备）；③
  UseVirtualCardOnSelectedTargets（CardKind::Sha，play_as_kind 完整
  杀结算管道，上游 useCard isCard:true 等价表达）。
- **uses_per_turn 1** 表达 usable:1（命令级权威限次，olwenren 同款）。
- **注册**：ABI 739（领号+host bump 739）/ tag 760 / hash 154 双 u32+盐
  SY；sp 包 ABI 738→739（编译器自动计算）。
- **登记债**：上游 get.cardPile 为随机检索，以牌堆顶向下顺序检索近似
  （yixiang 同口径）。
- **方法备忘**：上游「检索牌堆+令目标使用+视为杀」三段组合全部落在
  既有原语上——yixiang 检索、jiebing 装备上手、meiyong 虚拟杀管道；
  组合机器不需要新续体（各段无选择分支时单发直结）。

### §8.80 亮顶选获 gaintag 状态机：olqiaozhi 巧织（2026-09-15 第二百九十六批，tag 761 / ABI 740）

**olqiaozhi**（sp，noop→complete）——出牌阶段，弃一张牌，亮出牌堆顶两
张并获得其中一张（所得打「巧织」标）；手牌有「巧织」牌时不可发动。

- **机器结构**：ResolveOlqiaozhi(player, stage)（tag 761，Cont 330 两段
  续体）——stage 0=机内 gaintag 门控 + 弃成本（强制 SelectCards 1..1，
  discardable he）；stage 1=弃置 + 亮牌堆顶 2（.rev().take(2)，不从堆
  移除）→ SelectCards 1..1（选项=两张）；stage 2=所选 MoveCards 入手
  牌 + add_content_card_tag（blake2b("olqiaozhi")=1909178934）+ 未选入
  弃（上游 cardsGotoOrdering 收尾语义）。
- **发动门控**：手牌有巧织标 → 机器静默早退（DSL active 无 gaintag
  gate，登记债：按钮可见性近似）。
- **注册**：ABI 740（领号+host bump 740）/ tag 761 / hash 154 双 u32+
  盐 QZ；sp 包 ABI 739→740（编译器自动计算）。
- **登记债**：①gaintag 门控机内校验（发动按钮始终可见）；②可弃区含
  装备（hand_nonempty 仅命令级近似）。
- **工程备忘（入册）**：亮牌对「从堆顶取 N 张」的 payload 顺序随
  .rev() 定向——「未选牌」必须按选中值对表判定，不得固定取 payload
  槽位（首测即暴露）。

### §8.81 view_as 动态张数原语：owner_basic_uses_plus_one_cards（2026-09-15 第二百九十七批，零 bytecode / ABI 741）

**缺口**：view_as 素材张数此前仅静态（min_cards/max_cards u8）或
「恰好 max(1,hp)」（owner_hp_min_one_cards，ABI 236）。上游 spolxixiang
西向需要「至少 1+本局基本牌使用数张」的动态下限、上限不封顶。

- **新 spec 字段** owner_basic_uses_plus_one_cards（PackageViewAsEntry/
  ViewAsIr/ViewAsAst/ViewAsEmit/ViewAsDef 五层 + parse token + typecheck
  约束组与 same_suit 豁免 + registry source_free 禁止组与 potqingyan/
  dddzhijie/dddtongyu 互斥组 + codegen ABI 门 + lib ladder rung 741 +
  logic_hash h.update 纳入），~40 点接线照 owner_hp_min_one_cards 同型。
- **新状态**：lifetime_basic_card_uses（GameState serde default；dispatch
  循环 CardUsed 排水处按 kind.is_basic() 递增，registry
  track_basic_use_history 门控 opt-in；hash 摘要 tag 205）。
- **运行时语义**：view_as_card_count 分支（置于 hp 分支之前）返回
  (1+uses, u8::MAX)；响应窗口 multi-card rule 查找（skill.rs 1115）与
  单张排除查询（query 2984）同步收录。
- **落地件**：spolxixiang 西向（sp approx→complete）——双 branch sha/
  juedou + ignore_distance + no_use_count + 动态张数。
- **登记债**：「每种牌名每回合限一次」未建模（view_as branch 限次需
  from none，实体转换不适用；待 per-branch turn gate 原语）。
- **工程备忘（入册）**：动态 min/max 分支必须置于 owner_hp 早退**之前**
  （首测即暴露不可达分支）；计数器记录点在 dispatch 循环而非
  run_effects 排水（测试直调 dispatch_for_events 绕过排水）。

### §8.82 view_as 不封顶张数语法：at_least N（2026-09-15 第二百九十八批，零 bytecode / ABI 742）

**缺口**：非 source_free 的多张转换被 typecheck/registry 的
`max_cards > 2` 上限拒绝——上游垦伤 rekenshang（[2, Infinity]）/
olkenshang（[1, Infinity]）的「至少 N 张不封顶」无法表达。

- **语法**：`at_least N`（parse 新 token；min_cards=N，max_cards=u8::MAX
  255 哨兵=不封顶；source_card_count_seen 互斥复用）。
- **校验放宽**：typecheck 与 registry 的 `v.max_cards > 2` 均改为
  `> 2 && != u8::MAX`（哨兵值豁免；用户显式 up_to 255 为无害超集）。
- **ABI 门**：lib ladder rung 742（view_as max_cards==255）+ codegen
  max_cards==255 → 742；logic_hash 经既有 max_cards 条目自动纳入。
- **落地件**：rekenshang（up_to 2 → at_least 2，张数面补齐）+
  olkenshang（默认 1 → at_least 1 不封顶）。既有测试适配（契约断言
  min/max 更新；行为断言 1 张拒绝/2 张接受）。
- **登记债（两件共有）**：「改目标为攻击范围外所有其他角色」effect
  子技未建模（待 saved-sha 目标整体替换原语；useCard2 时机+可选确认）。
- **运行时零改动**：view_as_card_count 对静态 (N, 255) 原生直通。

### §8.83 垦伤改目标路径实测：view_as 管道无使用期挂起窗口（2026-09-15 第二百九十九批，回退批）

**结论（负结果，工程事实入册）**：垦伤「改目标为范围外全体」子技无法
经由 saved-sha 替换路径（replace_saved_sha_targets_ignoring_distance，
批 298 预设）清偿——实测确认：

1. `ActivateSkill` 的 view_as 转换杀走**同步全结算**管道
   （play_as_kind 内部完成响应与伤害），Accept 返回时
   pending_card_resolutions 为空、resume_kind 为 None。
2. 该管道**不发 CardUsed DomainEvent**——`trigger card_used` 触发器
   在转换流中不 fire（机器入口零命中），与主动出杀管道（gaoyuan/
   meiyong 场景，触发窗口内杀处于 saved 挂起态）架构不同。
3. 因此机内 gate（originating_view_as_skill_def 比对 + resume 挂起
   检查 + 范围外候选）在转换流中必然早退。

**回退**：本批曾完整实现 ResolveKenshangRetarget（tag 762/ABI 743/
Cont 331 确认续体 + replace_saved_sha_targets_ignoring_distance 提交
+ 双件 card_used 触发臂），因上述架构事实全部回退（14 处注册点 +
DSL 触发臂），主干回归批 298 终态（ABI 742）。

**正确路径（备忘）**：转换管道需支持「使用期挂起」（CardUsed 触发窗
口）后，本路径可直接复活——机器、续体与提交函数的完整设计已在本 ADR
历史与本批日记中留档。

### §8.84 per-branch turn gate：once_per_turn_per_branch（2026-09-15 第三百批，零 bytecode / ABI 744）

**缺口**：view_as 的 branch 限次（once_per_round）要求 `from none`
（source-free 转换族专用记账），实体来源转换无按牌名限次能力——西向
spolxixiang 的「每种牌名每回合限一次」无法建模。

- **新 spec 字段** once_per_turn_per_branch（五层接线同 owner_basic_uses
  同型；无禁止组——通用组合）。
- **记账**：handle_view_as 的 charge 点以「规则在 skill 的 view_as 规
  列表中的索引」（%32）为位写既有 view_as_use_round/view_as_used_branches
  位图（token 语义 round*16+current 与 potqingyan 同款，位图设施复用零
  新状态）。
- **gate**：handle_view_as 的规则匹配链按每条规则自身索引位跳过本回合
  已用规则（inst.view_as_use_round==token 时）。
- **ABI 门**：codegen once_per_turn_per_branch → 744 + lib ladder rung
  744；logic_hash 纳入。
- **落地件**：spolxixiang 西向双规则补 once_per_turn_per_branch——
  「每种牌名每回合限一次」全建模（债面清零，西向语义完整）。
- **测试工艺（入册）**：动态 max 不封顶使双规则均可吃任意张数——限次
  断言用「三连发」（第三发无规则可用被拒），勿按张数区分规则。

### §8.85 active gaintag gate：hand_missing_gaintag（2026-09-15 第三百零一批，零 bytecode / ABI 745）

**缺口**：active 技能无法按「手牌无指定 gaintag」做命令级发动门控
（巧织 olqiaozhi 只能机内静默早退，按钮可见性失真）。

- **新 spec 字段** hand_missing_gaintag: Option<u32>（PackageActiveEntry/
  ActiveIr/ActiveAst/ActiveEmit/ActiveSkillDef 五层 + active 块 parse
  token `hand_missing_gaintag <u32>` + logic_hash 纳入）。
- **命令 gate**：ActivateSkill 前置校验块（hand_nonempty 同位）——手牌
  含携带该 gaintag 的牌 → Illegal 拒绝（按钮级可见性）。
- **ABI 门**：codegen actives 循环 + lib ladder rung 745。
- **落地件**：olqiaozhi 巧织补声明 gate——「手牌有巧织牌时不可发动」
  命令级建模，按钮可见性债面清偿（机内防御 gate 保留）。
- **工程坑（入册）**：gaintag 值为 u32 大数（blake2b 截断），不能复用
  expect_selection_count（u8 上限 0..=255）——按 mark_at_least 同款
  u32 Int token 解析。

### §8.86 闪电挂判定状态机：dczhee（2026-09-15 第三百零二批，tag 763 / ABI 746）

**dczhee**（xianding，noop→complete）——锁定技：进入游戏/游戏开始时从
牌堆、每名角色的结束阶段从弃牌堆检索一张【闪电】挂为判定牌（判定区
已有闪电则静默）。

- **机器结构**：ResolveDczhee(player, mode)（tag 763，单发无续体，双
  触发臂 game_started/phase_end 共调）——mode 0/1 切换检索区（牌堆/
  弃牌堆，区顶向下首张 kind==TrickShandian）；canAddJudge 以「owner
  判定区无 TrickShandian」近似；MoveCards 入判定区（引擎判定管道自然
  结算）。
- **零新 filter**：owner_alive 通用过滤 + 机器内 gate（forced 静默无
  害）；game_started/phase_end 全局触发先例复用（孛明/直谏）。
- **注册**：ABI 746（领号+host bump 746）/ tag 763 / hash 154 双 u32+
  盐 ZE；xianding 包 ABI 706→746（该包首台专属机器，阶梯从全局最大
  值落位；guili 契约 pin 勘误 1 文件）。
- **登记债**：无（双臂/检索区切换/唯一性门控全建模；闪电的判定结算
  由引擎既有管道承接）。
- **方法备忘**：xianding 包 274 件诚实 noop 为各包之最（130 件已解析
  锚定上游 def），后续批次按 def 行数升序逐件甄别。

### §8.87 点数检索收集状态机：dchaoxian 朝献（2026-09-15 第三百零三批，tag 764 / ABI 747）

**dchaoxian**（xianding，noop→complete）——限定技：出牌阶段，将弃牌堆
中所有点数为 3 的牌洗入牌堆，然后获得其他所有角色手牌中的 3 点牌。

- **机器结构**：ResolveDchaoxian(player)（tag 764，单发无续体）——
  ①弃牌堆扫 rank==3 → MoveCards 入牌堆 + zone 内 drain/insert(0)
  reorder 至头部（=牌堆底；上游随机插入近似）；②依座位序遍历其他存活
  角色，其手牌 rank==3 的 MoveCards 入 owner 手牌（give 语义）。
- **once_per_game** 表达 limited（命令级权威限次）。
- **注册**：ABI 747（领号+host bump 747）/ tag 764 / hash 154+盐 CX；
  xianding 包 ABI 746→747（guili+dczhee 契约 pin ×2）。
- **登记债**：随机插入以牌堆底近似。
- **测试工艺（入册）**：bootstrap 弃牌堆自带 3 点牌残留（5 张）——
  点数断言一律预记残留数动态化（测量点在注入之后时勿重复计数）。

### §8.88 死亡赠礼状态机：zhongjie 张介（2026-09-15 第三百零四批，tag 765 / ABI 748）

**zhongjie**（sp2，noop→complete）——当你死亡时，你可以令一名其他角色
加 1 点体力上限、回复 1 点体力、然后摸一张牌。

- **机器结构**：ResolveZhongjie(player)（tag 765，Cont 332 单段续体）
  ——选一名其他存活角色（可取消）→ ChangeMaxHp(+1) + RecoverHp(1) +
  DrawCards(1) 三连。
- **触发**：`trigger death owner_is_target`（无 alive 检查承接上游
  forceDie 语义——死者仍可发动/选择）；DSL 层 forced，机器 stage 0 可
  取消选目标 = 上游确认窗+选目标语义等价（少一层确认窗）。
- **注册**：ABI 748（领号+host bump 748）/ tag 765 / hash 154+盐 ZJ2；
  sp2 包 ABI 713→749（连带 starweilin）。
- **登记债**：无。

### §8.89 威临三条件过滤器 + 伤害增补（2026-09-15 第三百零四批，零 bytecode / ABI 749）

**starweilin**（sp2，noop→complete）——锁定技：你的回合内，当你对一
名本局未受受过伤害的角色造成伤害时，此伤害 +1（你本局使用牌数 ≥ 其
体力值）。

- **新过滤器** owner_source_in_turn_target_unharmed_card_actions_ge_
  target_hp（五点注册 + lib ladder rung 749 + **codegen trigger filter
  门**；eval = owner_alive + owner_is_source + DamageBefore +
  turn.current==owner + !damage_history 含目标 + card_action_totals ≥
  目标 hp）。
- **DSL**：modify_current_damage(self, 1) 既有原语（panshi_damage/
  olfeibai 同款）。
- **登记债**：用牌计数以 card_action_totals（用/打出双计）近似上游
  useCard-only。
- **工程备忘（入册）**：①filter 类 rung 若无 codegen 门，会在
  required_rule_vm_abi 顶部被 module_required_abi≥677 短路遮蔽——
  新 filter 必须同时在 codegen triggers 循环加 ABI 门；②ladder 分支
  插入须置于高 rung 区（降序），低处插入永不可达。

### §8.90 粗遐/未命孪生随机弃牌状态机（2026-09-15 第三百零五批，tag 766 / ABI 750）

**starcuxia + starweiming**（sp2，noop→complete，双件共机）——锁定技：
其他角色使用牌指定 owner 后，若（其体力更高 / owner 体力更高）或其本
局伤害过 owner：其随机弃 1 手牌；条件复验仍成立则 owner 摸 1。

- **机器结构**：ResolveStarcuxiaWeiming(player, source, variant)（tag
  766，单发无续体，双 DSL 共机 variant 反转条件）——RuleRng shuffle
  随机弃 1 + hp/伤害历史条件复验摸 1。
- **双过滤器**：owner_is_target_source_hp_gt_owner_or_source_damaged_
  owner / owner_is_target_owner_hp_gt_source_or_source_damaged_owner
  （五点注册 + codegen 门 + rung 750）。
- **注册**：ABI 750 / tag 766 / hash 154 三 u32+盐 XW；sp2 包 ABI
  713→750。
- **登记债**：①round 级伤害历史以 lifetime damage_history 放宽；②
  filter eval 集成（CardTargeted ctx 语义已按逐目标修正、plan 已见
  active；eval 未达之因待查——测试以机器直调验证语义）。
- **工程备忘（入册）**：CardUsed 的逐目标 ctx 是 **CardTargeted**
  （player=目标、source=使用者）；`trigger card_used` ctx.player=使用
  者——「被指定为目标」类条件须用 card_targeted。

### §8.90（补）filter eval 集成排查结案（2026-09-15 第三百零六批）

批 305 登记的「E1 eval 未达」排查结案——**触发器/过滤器/机器链路完全
正常**（E1 eval 到达、条件全真、vm arm 命中、机器执行）。根因是批 305
测试辅助 fire_card_used 只调 dispatch_for_events 未 run_effects 其返回
效果（调用者责任，dispatch 契约：产出待执行列表）。已补真管道测试
（CardUsed → CardTargeted 逐目标 ctx → filter eval → 机器），4/4 全过。

### §8.91 类型位图防伤状态机：sanshou 三首（2026-09-15 第三百零七批，tag 767 / ABI 751）

**sanshou**（xianding，noop→complete）——即将受到伤害时可亮牌堆顶三张：
存在类型（基本/锦囊/装备）在本局未被使用过的牌则防止此伤害；亮出的牌
移入弃牌堆。

- **新状态**：lifetime_used_type_groups: u8（3 位组类型位图，
  card_type_group() 位置位；CardUsed 排水 **track 门控维护**——registry
  track_used_type_groups 由专属 filter
  owner_is_target_sanshou_type_tracked 携带信号；skip-if-zero。**初版
  无条件维护曾致 golden 对局向量全量漂移**（对局中 CardUsed 置位→状态
  hash 变），已改门控——与批 297 track_basic_use_history 同工艺）+
  hash 摘要 tag 206。
- **机器结构**：ResolveSanshou(player)（tag 767，单发无续体）——亮顶 3
  （peek）→ 位图判定 → PreventDamage{当前 id}（damage_before 窗口内取
  消）→ 亮牌 MoveCards 入弃。
- **forced 化说明（实测入册）**：上游 check 为 AI 决策；optional confirm
  会消费 damage_before 窗口使 PreventDamage 出窗失效（ContentFault）——
  DSL forced + 机器「有全新类型才发动」与上游 AI 最优决策等价。
- **工程坑（入册）**：`PreventDamage{damage_id: 0}` 的重写仅存在于
  vm_host（DSL 路径）；机器侧必须直接读 resolution.current_damage_id。
- **注册**：ABI 751 / tag 767 / hash 154+盐 SS；xianding 包 ABI
  747→751（guili/dczhee/dchaoxian 契约 pin ×3 + sanshou 专属 filter
  五点注册 + lib ladder rung 751 + codegen 门）。
- **登记债**：亮牌展示表现面未建模（牌移入弃牌堆，结果语义等价）。

### §8.93 手牌比较双分支状态机：nhlundao 论道（2026-09-15 第三百零八批，tag 768 / ABI 752）

**nhlundao**（sp2，approx→complete）——当你受到伤害后，若伤害来源存活
且其手牌数与你不同：你的手牌数多于其，你摸一张牌；否则你弃置其一张牌
（其手牌/装备区，你选择）。

- **机器结构**：ResolveNhlundao(player, source)（tag 768，Cont 333 单段
  续体）——机器内 gate（来源存活+手牌数不同）；手牌多 → 摸 1 直结；
  否则 owner 从来源 he 区强制 SelectCards 1 → 续体验活弃（dcmoukui 选
  弃工艺）。
- **filter**：复用通用 owner_is_target（零新 filter；gate 全在机器内，
  dc zhee 同工艺）。
- **注册**：ABI 752 / tag 768 / hash 154 双 u32+盐 LD；sp2 包 ABI
  750→752（连带批 304 的 749 rung 落位）。
- **登记债**：无。
- **工程备忘（入册）**：sp2 pin 扫描需覆盖 `package.manifest` 与
  `sp2.manifest` 两种变量名（langmie 跨包闭包测试用后者）。

### §8.94 限定支援状态机：xiangshu 襄戍（2026-09-15 第三百零九批，tag 769 / ABI 753）

**xiangshu**（sp2，noop→complete）——限定技：结束阶段，若你本回合造成
过伤害，令一名已受伤的角色回复 X 点体力并摸 X 张牌（X=min(5, 你本回合
造成的伤害事件数)）。

- **机器结构**：ResolveXiangshu(player, targets[])（tag 769，单发无续
  体）——X=damage_dealt_count 机内直读（狐魅同款）；回复钳制至体力上
  限差；摸牌足额。once_per_game 承接 limited；wounded 目标 spec（命令
  级合法性）。
- **DSL 工艺（入册）**：active 机器的目标经 host.selected_targets()
  注入（dcanxu 同款）——DSL 单参 self，无 event_target 变量。
- **注册**：ABI 753 / tag 769 / hash 154 列表+盐 XS；sp2 包 ABI
  752→753。
- **登记债**：X 上限按 min(5, 事件数) 建模（上游 getStat("damage") 为
  伤害点数口径——事件数近似，同量级登记）。
