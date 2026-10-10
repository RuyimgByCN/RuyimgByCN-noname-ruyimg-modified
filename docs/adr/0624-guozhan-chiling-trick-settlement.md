# ADR 0624：国战锦囊「敕令」（chiling）结算

> 承接 [ADR 0623](0623-guozhan-haolingtianxia-trick-settlement.md)：国战 12 锦囊结算第十一张——
> chiling（敕令，上游即名 `chiling`；此前任务清单误记「叱咤」）。全新 CardKind（52）＋
> `CHILING_CUSTOM_CHOICE` 两阶段选择；`RULE_VM_ABI_VERSION` 不变（无新 EffectRequest 变体）。

## 上游语义（旧 JS 仅作语义证据，不进入运行时）

guozhan.js:999 `chiling`（梅花 3，牌堆 1 张）：
- `enable: 存在任一角色 isUnseen()`（有暗置武将）；`filterTarget: target.isUnseen()` +
  `selectTarget: -1`——目标 = **所有有暗置武将的角色（含使用者**，filterTarget 无自身排除）。
- 每目标三选一（`content`）：
  1. **明置一张武将牌，然后摸一张牌**——先选主将/副将（仅暗置槽），`showCharacter(slot)`
     后 `draw()`；
  2. **失去1点体力**（`loseHp`，无来源）；
  3. **弃置一张装备牌**（he 中类型为装备者，`forced`）——仅当目标有装备牌时可选。
- 目标有技能标签 `nomingzhi`（不可明置）时选项①移除——引擎无该技能标签查询，近似省略。
- **`destroy`/`onDestroy`**：敕令实体卡被弃置（非使用）销毁时，当前回合者获得临时技能
  `g_chiling3`（其 phaseEnd 对全部暗置角色再跑一遍同样的三选一），并将「诏书」
  （zhaoshu，梅花 3）放入牌堆底（若牌堆中已无）。

## 做了什么

### CardKind + CardKindTag + trick_effect（8 件套新增尾件）

- `state/cards.rs`：`CardKind::TrickChiling = 52`（尾件，序数稳定）＋ from_tag 臂 ＋
  is_trick 列表。
- `engine-api/src/view.rs`：`CardKindTag::TrickChiling` ＋ projection.rs 映射。
- `command/play.rs`：`trick_effect::CHILING = 28`；新增 `equip_typed_zone_cards` 辅助
  （he 中装备类牌，对应上游 `hasCards("he", {type:"equip"})`）。

### 分发（command/play.rs）

- 目标自动 = `alive_by_seat` 中 `unseen[0] || unseen[1]` 者（**含使用者**，与上游一致）；
  无暗置目标 → `Illegal`。经 `use_independent_trick_targets` 入队（含无懈窗口）。
- `skill_card_target_spec`：`Immediate(Vec::new())`。

### 结算（CHILING_CUSTOM_CHOICE 两阶段）

- **Stage 1**（挂 target，选项 [1,2]＋有装备时 [3]，不可取消）：三选一。
  - ① → **Stage 2 mode 1**：选一个仍暗置的槽位（选项集 = unseen 槽）→
    `mode::reveal_general`（确定性状态变更，重算势力）＋ `DrawCards 1`。
  - ② → `DealDamage { is_lose_hp: true, source: NONE, amount: 1 }`（失去体力无来源）。
  - ③ → **Stage 2 mode 3**：SelectCards 选一张装备类 he 牌 → `DiscardCards`。
- 收尾经 `finish_independent_trick_target` 推进独立锦囊队列。

魔数 `CHILING_CUSTOM_CHOICE = 0x4348_4C4E` 复用 `ContinuationKind::Custom` 分发。

### ⚠️ 近似省略

1. **onDestroy / phaseEnd 全局敕令 + 诏书回牌堆**：引擎卡牌移动管线无「按牌类销毁钩子」
   （上游 `destroy`/`onDestroy` 卡牌追踪机制），亦无「牌堆底插入」原语；该机制整体记作
   **不可表达，维持元数据登记**（与 zhibi 的观看类判断一致，ADR 0621 同判）。诏书
   （zhaoshu）本身已是牌堆中的真实装备（deck.rs:418 `CardKind::Treasure`），不受影响。
2. **`nomingzhi` 技能标签**：引擎无按名技能标签查询，选项①恒提供（暗置槽非空时）。
3. **明置后野心家「拉拢人心」**：`reveal_general` 仅重算势力；「拉拢人心」邀约属
   `RevealGeneral` 命令流，敕令路径省略（ADR 0624 记录）。

### 牌堆映射 + AI + query

- `content/deck.rs`：`"chiling"`（梅花 3）由 `Other` → `CardKind::TrickChiling`。
- `engine-ai/legal.rs`：存在任一存活暗置角色时给出 `targets: vec![]` 合法行动。
- `query/mod.rs`：牌名长度 4（敕令）＋ trick 列表并入。

### 单测

`crates/engine-core/tests/guozhan_trick_chiling.rs` 3 项，全部通过：
1. 目标含使用者（2 人局两目标按座次）；① 明置主将 + 摸 1，另一目标 ② 失去 1 点体力；
2. ③ 弃置装备牌：目标持装备 → 选项含 ③，弃置后手牌清零；
3. 槽位选择仅列暗置槽：先 `RevealGeneral{slot:0}` → ① 的槽位选项仅剩 [1]。

## 验证

| 检查 | 结果 |
|---|---|
| 新单测 | **guozhan_trick_chiling 3 passed** ✅ |
| conformance | 全量重跑中（国战向量可能因牌堆 kind 变化漂移，如漂移则本域重钉） |
| 失败集合 | 与 claude-a-failures-hlx.txt（1：original_offline_jdlianhuan）comm 对比，须零新增 |
| `cargo build` | engine-core / engine-ai 无编译错误 ✅ |

**ABI 说明**：无 `RULE_VM_ABI_VERSION` bump——新增 CardKind/trick_effect/自定义 choice
均为序数稳定尾件，无新 EffectRequest 变体。

## 后续

1. **zhibi（知己知彼）**：纯信息揭示——建议维持元数据登记（ADR 0621 同判）；至此
   国战 12 锦囊中 11 张已有真实 CardKind 结算，仅剩 zhibi 按元数据登记。
2. **gwyewu（叶舞）造卡入口**：需随机弃牌原语（ADR 0613 已登记）。
3. **gw_dudayuanshuai2 per-card 禁弃置**：需按 def 的 card 门禁（ADR 0608 已登记）。
4. **敕令 onDestroy 机制**：若后续为引擎增加「按牌类销毁钩子 + 牌堆底插入」原语，可补。
