# ADR 0602：国战明置触发（RevealGeneral 命令）

> 让 ADR 0601 phase C 的暗将门控字段真正可激活——玩家可主动翻开武将。

## 背景

ADR 0601 加了 `unseen` 字段 + `skill_effectively_enabled` 门控，但 `bootstrap_guozhan` 双将
开局全明（`unseen=[false,false]`），门控是 inert 的——没有途径让武将变暗或翻开暗将。
本 ADR 补上「明置」命令，使暗将机制形成闭环：暗置 → 明置触发 → 技能激活。

## 做了什么

### `CommandKind::RevealGeneral { slot: u8 }`（`engine-api/src/command.rs`）
新命令变体，`slot` = 0（主将）/ 1（副将）。**这是客户端命令，不是 `EffectRequest`**，
所以不碰 ABI/hash/DSL——只在 command 处理管线加分支。

### `handle_reveal_general`（`command/mod.rs`）
国战专用包装：非 guozhan ruleset 拒绝；调 `mode::reveal_general`。

### `reveal_general`（`mode/guozhan.rs`）
- 翻开 `unseen[slot] = false`（已翻开或越界拒绝）。
- 重算势力：按双将 `character.group` 经 `compute_group_from_pair` 重解（暗将不贡献势力）。
- 野心家接入：若翻开后该玩家势力处少数（`should_become_ambition`），势力改写为 `AMBITION`。
- 借用顺序：先读 character group（不可变借用），再 `get_mut` 翻 slot。

### 国战 AI 明置分支（`runner.rs::choose_command`）
国战 Play 阶段，若当前玩家有暗将，优先发 `RevealGeneral`。身份局不进此分支。
当前 `bootstrap_guozhan` 开局全明，此分支条件不满足（inert）——当后续让开局暗置时即激活。

### `devtools/src/timeline.rs` 补臂
`describe_command` 加 `RevealGeneral` 显示。

## 验证

| 检查 | 结果 |
|---|---|
| conformance | **4 测试全过**（国战向量未漂移——开局全明，明置不触发）✅ |
| 国战单测 | **17 passed**（+明置翻转/防重复/身份局拒绝 3）✅ |
| 国战烟雾 | **2 passed** ✅ |
| 失败集合 | **257 = 257，双向差集空** ✅ |
| warnings | **59** ✅ |

**不动 ABI/身份局 golden/DSL。** 国战向量零漂移（明置路径未被 conformance 触发，因开局全明）。

## 后续

- 让 `bootstrap_guozhan` 开局暗置双将（符合国战真实规则）会让明置路径在 conformance 中激活，
  需重钉国战向量——留作下一步（会改变国战对局行为，需谨慎评估 AI 稳定性）。
- 珠联璧合花费效果、野心家拉拢人心结盟、国战内容移植——仍留后续。
