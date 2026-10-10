# ADR 0601：国战阶段 C——暗将门控 + 珠联璧合 + 野心家判定

> 实施了 [ADR 0598](0598-guozhan-mode-design.md) §6 阶段 C 的三项规则。状态模型基础见
> [ADR 0599](0599-guozhan-stage-a-state-model.md)，骨架见 [ADR 0600](0600-guozhan-stage-b-skeleton.md)。

## 做了什么

### C1：暗将技能门控

**`SkillInstance` 加 `general_slot: u8`**（`state/skills.rs`）：
- 0 = 主将，1 = 副将，`GENERAL_SLOT_NONE` (255) = 身份局/无归属。
- `#[serde(default = "general_slot_none")]`，身份局构造不显式设此字段 → 默认 255。

**`hash.rs` 条件编码**：`general_slot != GENERAL_SLOT_NONE` 时才写
`skill-general-slot/v1` 段。身份局技能 slot 恒为 255 → 段不写入 → **身份局 hash 零漂移**。

**`skill_effectively_enabled` 加门控**（`state/mod.rs`）：新增 `guozhan_skill_hidden`
判定——国战模式下，若技能的 `general_slot` 对应武将 `unseen[slot]` 为 true，技能休眠。
身份局技能 slot=255，`guozhan_skill_hidden` 直接返回 false，**身份局行为不变**。

**`bootstrap_guozhan` 标注 slot**（`pack_tw_1.rs`）：挂双将技能时按 `[(&main_ch, 0), (&lt_ch, 1)]`
标注 `general_slot`。`grant_skill_instance`（运行时授予）和身份局 `bootstrap_match` 的技能
构造补 `general_slot: GENERAL_SLOT_NONE`。

### C2：珠联璧合

**配对表**（上游 `character/perfectPairs.js`，117 条）：解析为 `PERFECT_PAIRS: &[(&str, &[&str])]`
静态切片，内嵌于 `mode/guozhan.rs`。

**`is_perfect_pair(main_name, lt_name) -> bool`**：双向对称查找（任一方为 key、另一方在其
mate 列表即成立），按 `PackageCharacterEntry.name_key` 匹配。

**开局授予**（`bootstrap_guozhan`）：
- 珠联璧合组合 **max_hp +1**；
- 授予 `ZHULIAN_BIHE_MARK`（0x5A4C4842）到玩家 `marks`。

**标记花费效果**（当无中生用/濒死回血）依赖国战卡牌内容移植，留作内容侧工作。

### C3：野心家判定

**人口统计**：`faction_population(state) -> BTreeMap<String, u32>`（按势力数存活玩家），
`total_population(state) -> u32`。

**`should_become_ambition(state, player) -> bool`**：简化版 `wontYe`（上游反向）——
当某势力存活数 **严格过半**（> total/2）且该势力非玩家自身势力时，弱势力成员可变野心家。
已在 `AMBITION` 势力的不再重判。

**未实现**（留后续）：`拉拢人心` 建国（`yexinjia_friend` 结盟关系图）需新 ABI choice 事件 +
联盟状态字段；`showYexings` 事件流。`should_become_ambition` 已可在明置时机被调用，
后续接入 choice 后即可完整。

## 验证（身份局零回归）

| 检查 | 阶段 B 后 | 阶段 C 后 | 结论 |
|---|---|---|---|
| `cargo test -p conformance` | 4 测试全过 | **4 测试全过** | ✅ |
| 国战单测 | 9 passed | **15 passed**（+珠联璧合3 +野心家3）| ✅ |
| 国战烟雾（run_guozhan_match）| 2 passed | **2 passed** | ✅ |
| workspace 失败集合 | 257 套件 | **257 套件，逐项一致** | ✅ 零回归 |
| engine-core lib warnings | 59 | **59** | ✅ |
| 编译错误 | 0 | **0** | ✅ |

失败集合双向差集为空——257 个失败在阶段 C 前后逐项相同，是他人 `original_*` 领域既有漂移。

身份局 3 向量（`identity_4_42`/`identity_2_7`/`identity_8_99`）hash 逐字节不变。国战 2 向量
因 C1（general_slot）+ C2（珠联璧合 mark/+1hp）改了国战 canonical state 而**重钉**：
- `guozhan_4_42`: `e6c9241c9b026615dfd1bf0a43bc51eebd2521c6bc89fb7a2374e1bafa8bf83f`
- `guozhan_2_7`: `fa044f2f0e46d80fd6809a2410febde385a7825ebb85166e121fe718dacc50ad`

## 不动什么

- **不动 ABI**（暗将门控是 `skill_effectively_enabled` 内部分支；珠联璧合是开局标记；
  野心家判定是纯函数。无新 `EffectRequest`/`ContinuationKind`）。
- **不动身份局 golden**（身份局路径：slot=255、无珠联璧合标记、无野心家判定）。
- **不动 DSL/content-compiler**。

## 已知局限（后续）

1. **暗将明置触发**：`unseen` 字段已就位 + 门控已生效，但 `showCharacter` 明置事件未实现
   （需 choice/触发器接入）。当前双将全明，门控是 inert 的——明置机制落地后即激活。
2. **珠联璧合花费**：标记已授予，但「当无中生用/濒死回血」效果需国战卡牌内容移植。
3. **野心家建国**：人口判定已实现，`拉拢人心` 结盟（新 ABI choice）未做。
4. **国战武将/卡牌内容移植**：与内容侧 agent 协调（独立工作流）。
