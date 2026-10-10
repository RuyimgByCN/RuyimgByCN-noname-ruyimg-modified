# ADR 0622：国战锦囊「文和乱武」（gz_wenheluanwu）结算

> 承接 [ADR 0621](0621-guozhan-yuanjiao-map-existing.md)：国战 12 锦囊结算第九张——
> gz_wenheluanwu（文和乱武）。全新 CardKind（50）＋ `WENHELUANWU_CUSTOM_CHOICE`
> 三阶段自定义 choice 结算；`RULE_VM_ABI_VERSION` 不变（无新 EffectRequest 面）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

guozhan.js:339 `gz_wenheluanwu`（梅花 12，牌堆 1 张）：
- `filterTarget: true` + `selectTarget: -1` + `ignoreTarget: 手牌数为 0`——
  **对所有其他角色使用**，无手牌者自动排除。
- 每目标（`content`）：目标展示所有手牌 → 使用者二选一：
  1. 令目标**弃置两张类型不同的手牌**（仅一种类型可弃时退化为弃一张；无可弃则无事）；
  2. 使用者**弃置其一张手牌**（visible）。
- 随后若目标为**群势力**且仍存活、无手牌、且弃牌确实发生（result2.bool）→
  目标摸至当前体力值（至多为 5）：`target.draw(Math.min(5, target.hp))`。

「展示手牌」为纯信息揭示（与 zhibi 同类），确定性引擎不入运行时——弃牌本体已蕴含
选择依据，故省略 showHandcards 的观察面（ADR 0621 同判）。

## 做了什么

### CardKind + CardKindTag + trick_effect（8 件套新增尾件）

- `state/cards.rs`：`CardKind::TrickWenheluanwu = 50`（尾件，序数稳定）＋ from_tag 臂
  ＋ is_trick 列表；新增 `card_type_group()` 辅助（0=基本, 1=锦囊, 2=装备，对应上游
  `get.type2` 三分）。
- `engine-api/src/view.rs`：`CardKindTag::TrickWenheluanwu` ＋ projection.rs 映射。
- `command/play.rs`：`trick_effect::WENHELUANWU = 26`。

### 分发（command/play.rs）

- 目标自动 = `alive_by_seat` 中除自己外**手牌数 > 0** 的角色（等价上游 ignoreTarget）；
  无目标 → `Illegal`。经 `use_independent_trick_targets` 入队（含无懈窗口）。
- `skill_card_target_spec`：`Immediate(Vec::new())`（目标自动，无需选择）。

### 结算（WENHELUANWU_CUSTOM_CHOICE 三阶段）

- **Stage 1**（挂 `actor`＝使用者，ChoiceKind::Custom，选项 [1,2]）：
  使用者二选一。① 时按目标手牌类型组数决定弃牌数——≥2 组 → 弃 2；仅 1 组 → 弃 1。
- **Stage 2**（ChoiceKind::SelectCards，按阶段 1 决定的 discarder 挂给目标或使用者）：
  ① 目标自弃 count 张（count=2 时校验两牌类型不同，同型 → Illegal）；
  ② 使用者从目标手牌中弃 1 张。`run_effects(DiscardCards{ player: target, … })`
  立即执行。
- **Stage 3（同一 resolver 内联）**：弃牌后 `player_group == "qun"` 且手牌空 →
  `run_effects(DrawCards{ count: min(5, hp) })`，随后 `finish_independent_trick_target`
  推进独立锦囊队列。

魔数 `WENHELUANWU_CUSTOM_CHOICE = 0x5745_4C57` 复用 `ContinuationKind::Custom` 分发
（payload[0]），不新增 ContinuationKind 变体——保持回放序数稳定。

### 牌堆映射 + AI + query

- `content/deck.rs`：`"gz_wenheluanwu"`（梅花 12）由 `Other` → `CardKind::TrickWenheluanwu`。
- `engine-ai/legal.rs`：存在「其他有手牌角色」时给出 `targets: vec![]` 合法行动。
- `query/mod.rs`：牌名长度 4（文和乱武）＋ trick 列表并入。

### 单测

`crates/engine-core/tests/guozhan_trick_wenheluanwu.rs` 4 项，全部通过：
1. ① 弃两张类型不同（杀+无中）→ 群空手补至当前体力（至多 5）；
2. ② 使用者弃目标一张（群单牌）→ 补至当前体力；
3. 非群（shu）空手不补牌；
4. 同类型两手牌 → ① 退化为弃一张，弃后仍有一张不触发补牌。

## 验证

| 检查 | 结果 |
|---|---|
| 新单测 | **guozhan_trick_wenheluanwu 4 passed** ✅ |
| conformance | 全量重跑中（国战向量可能因牌堆 kind 变化漂移，如漂移则本域重钉） |
| 失败集合 | 与 claude-a-failures-yj.txt（204）comm 对比，须零新增 |
| `cargo build` | engine-core 无编译错误 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——新增 CardKind/trick_effect/自定义 choice
均为序数稳定尾件，无新 EffectRequest 变体。

## 后续

1. **gz_haolingtianxia（号令天下）**：全员依次二选一（最复杂，待实现）。
2. **chiling（叱咤）**：全局临时技能。
3. **zhibi（知己知彼）**：纯信息揭示——建议维持元数据登记（ADR 0621 同判）。
4. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
5. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
