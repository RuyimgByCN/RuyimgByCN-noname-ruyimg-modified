# ADR 0603：国战规则完整化——开局暗置 + 拉拢人心结盟 + 珠联璧合濒死回血 + 武将元数据

> 完成 [ADR 0598](0598-guozhan-mode-design.md) §6 阶段 C 的全部规则件 + 国战武将元数据登记。
> 前置：[0599](0599-guozhan-stage-a-state-model.md)（状态模型）、[0600](0600-guozhan-stage-b-skeleton.md)
> （骨架）、[0601](0601-guozhan-stage-c-rules.md)（C1/C2/C3 基础）、[0602](0602-guozhan-reveal-general.md)
> （明置触发）。

## 做了什么

### 1. 开局暗置双将（激活明置闭环）
`guozhan_player` 的 `unseen` 默认改为 `[true, true]`——开局所有武将暗置，符合国战规则。
ADR 0602 的 AI 明置分支现在激活：每个回合 AI 优先翻开暗将。`bootstrap_guozhan` 仍写
`group_overrides`（真实势力，用于胜利判定），暗置只让技能休眠（C1 门控）。

### 2. 野心家拉拢人心结盟（不动 ABI）
**关键决策**：拉拢人心**不需要新 ContinuationKind**——复用 `ContinuationKind::Custom` +
`LOUREN_XIN_CHOICE` (0x5A4C5258) 魔数标记（与 GuardedChoice/resolve_* 同模式）。接线从
8 文件降到 4 文件，**零 ABI 改动**。

- `GameState.ambition_alliances: BTreeSet<[u32;2]>`：结盟关系图（规范化 [min,max] 存储）。
  hash 条件编码（非空才写 `ambition-alliances/v1`，身份局零漂移）。
- `are_allies`/`form_alliance`：双向查询/建立盟约。
- `resolve_lourenxin_choice`：choice resume 处理器（选目标 → 建盟约）。
- `handle_reveal_general`：明置变野心家且无盟友时，开拉拢人心 choice（选目标，
  `Custom` continuation + `LOUREN_XIN_CHOICE` payload）。
- `resolve_choice/mod.rs`：加 `LOUREN_XIN_CHOICE` 路由臂。
- `check_guozhan_victory`：用 union-find 把盟友合并为同阵营——盟友间不判胜，全同盟则结束。
- AI 兜底：`Custom` continuation 落 `_ =>` 选第一个 token（选目标结盟）。

### 3. 珠联璧合濒死回血
`enter_dying`（`pack_tw_1.rs`）：国战模式下，进入濒死时若玩家有 `ZHULIAN_BIHE_MARK`，
消耗 1 层 → 回 1 血（取消濒死）。身份局无此标记 → 行为不变。
「标记当无中生用」需 view_as DSL，留内容移植。

### 4. 国战武将元数据登记（256 个）
从上游 `mode/guozhan/src/character/*.js` 提取 256 个 `gz_` 武将（wei 50/shu 46/wu 45/
qun 61/jin 32/ye 9/shen 12/key 1），生成 `content/guozhan/guozhan_characters.json`
（`PackageCharacterEntry` 格式，skills 待逐个移植填充）。

**未做**：256 个 `gz_` 技能的 DSL 实现（每个需对照上游 JS 重写，是内容侧数百小时工作）+
npack 打包。当前 `bootstrap_guozhan` 仍从身份局武将池分配（group 过滤），国战对局可运行。

## 验证（身份局零回归）

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量重钉 3 次：暗置→结盟→濒死回血）✅ |
| 国战单测 | **21 passed**（+结盟 4）✅ |
| 国战烟雾 | **2 passed**（含拉拢人心 choice 流）✅ |
| 失败集合 | **257 = 257，双向差集空** ✅ |
| warnings | **59** ✅ |

身份局 3 向量 hash 逐字节不变。国战 2 向量本轮重钉至：
- `guozhan_4_42`: `517fd73debf49b366c736180f647495acc059c6525a6a127d050edee4d64cfbd`
- `guozhan_2_7`: `630330a7fdf29105cd2d4581fd172f9c58d03241b4282a025e61784c0d7f8105`

## 关于 ABI

**本轮国战工作全程不动 ABI。** 用户授权「动 ABI」，但审计后发现：
- `ContinuationKind` 已满（`Custom=255`），新变体不可行；
- 拉拢人心复用 `Custom` + 魔数标记（与既有模式一致）；
- 明置用 `CommandKind`（客户端命令，非 `EffectRequest`）；
- 珠联璧合濒死回血是既有 `enter_dying` 内分支。

故无需领 ABI 641+。omp 的 ddd 段（619-629）已全部 ✓ 完成，641+ 段实际未被占用。

## 国战整体完成度

| 件 | 状态 |
|---|---|
| 状态模型（双将/暗将字段）| ✅ 阶段 A |
| 双将分发 + 势力判定 + 胜利 | ✅ 阶段 B |
| 暗将技能门控 | ✅ C1 |
| 珠联璧合（配对表+标记+体力+濒死回血）| ✅ C2 + 本轮 |
| 野心家判定 + 拉拢人心结盟 | ✅ C3 + 本轮 |
| 明置触发 | ✅ ADR 0602 |
| 开局暗置 | ✅ 本轮 |
| 武将元数据登记（256）| ✅ 本轮 |
| gz_ 技能 DSL 实现（256 武将）| ⏳ 内容移植 |
| 珠联璧合当无中生用（view_as）| ⏳ 内容移植 |
| 国战卡牌包 | ⏳ 内容移植 |

国战引擎层规则件**全部完成**。剩余的是内容侧工作（技能 DSL + 卡牌），需逐个对照上游。
