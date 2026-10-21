# ADR 0600：国战阶段 B——mode 骨架 + 双将分发 + 势力判胜（身份局零回归）

> 实施了 [ADR 0598](0598-guozhan-mode-design.md) §6 阶段 B。状态模型基础见
> [ADR 0599](0599-guozhan-stage-a-state-model.md)。本 ADR 只记录实施与验证。

## 做了什么

### 新增 `mode/guozhan.rs`（220 行，含 9 单测）
- `compute_group_from_pair(a, b) -> Option<String>`：势力判定。同势力→该势力；
  不同势力→野心家（`ye`）；任一为空→取另一；均空→`None`。
- `guozhan_max_hp(main, lieutenant) -> u32`：体力上限 = 主将上限 + 副将上限。
- `check_guozhan_victory(state) -> MatchOutcome`：单一势力存活→`GroupWin { group }`；
  多势力存活→`InProgress`；无存活→`Draw`。
- `apply_guozhan_victory_if_any(state)`：写入 `state.outcome` + 发 `MatchEnded` 事件。
- `guozhan_player(seat, main_def, lt_def, max_hp) -> Player`：构造国战玩家（双将、
  `unseen=[false,false]`、identity 占位 Lord）。

### `bootstrap_match` 按 ruleset 分发（`pack_tw_1.rs`）
入口加：`if ruleset_name.starts_with("guozhan") { return bootstrap_guozhan(...); }`。
身份局路径（`ruleset` 不以 guozhan 开头）**完全不变**。

### 新增 `bootstrap_guozhan`（`pack_tw_1.rs`，~180 行）
- 从内容池筛 `group ∈ {wei,shu,wu,qun,jin}` 的武将，洗牌后每玩家分 2 张（主+副）。
- 体力上限 = 双将上限之和（缺省各 4）。
- 挂载双将技能（revealed；阶段 C 加暗将门控）。
- 调 `compute_group_from_pair` 算势力写入 `group_overrides`。
- 复用身份局的 zone/牌堆/开局摸牌/触发器/首回合摸牌逻辑。

### `kill_player` 胜利检查分发（`pack_tw_1.rs`）
按 ruleset 分发：guozhan → `apply_guozhan_victory_if_any`；否则身份局路径不变。

### `run_guozhan_match`（`runner.rs`）
- 入口函数，复用 `choose_command` 主循环，传 `guozhan-standard` ruleset。
- `enemy_targets` 加国战分支：按 `player_group` 判敌我（同势力=友，不同=敌），
  身份局路径（按 Identity 角色）不变。
- 2 个烟雾单测：`guozhan_match_runs_and_assigns_factions`（4 人，2000 步，
  bootstrap + 自动对局不 panic）、`guozhan_match_is_deterministic`（同 seed 可复现）。

### `lib.rs` 导出
`run_guozhan_match` 加入 `pub use runner::{...}`。

### 国战 conformance 向量（`conformance/src/lib.rs`）
新增 `GUOZHAN_MATCH_VECTORS`（2 个，从 0 新建并钉 hash）：
- `guozhan_4_42`：`48b085491677a13ce3ed9d741db5b1f7ce8f8a0956017e852205d7f7c854c16e`
- `guozhan_2_7`：`99888176fdaabe6a4b35acabe94da28959dc74597e00f2075db8f6bbe0e315dc`
- `allow_capped: true`（骨架 AI 可能 cap，待阶段 C 强化）。
- 接入 `run_all` 循环 + `all_golden_vectors` 断言 + `guozhan_match_vectors_individually` 测试。

## 验证（身份局零回归）

| 检查 | 阶段 A 后 | 阶段 B 后 | 结论 |
|---|---|---|---|
| `cargo test -p conformance` | 3 测试全过 | **4 测试全过**（+ 国战 individually）| ✅ |
| guozhan 单测 | — | **9 passed** | ✅ |
| guozhan 烟雾（run_guozhan_match）| — | **2 passed**（bootstrap+对局+确定性）| ✅ |
| workspace 失败集合 | 257 套件 | **257 套件，逐项一致** | ✅ 零回归 |
| engine-core lib warnings | 59 | **59** | ✅ |
| 编译错误 | 0 | **0** | ✅ |

失败集合双向差集（`comm -23`/`comm -13`）为空——257 个失败在阶段 B 前后逐项相同，
是他人 `original_*` 领域的既有漂移，与本次改动无关。

身份局 3 向量（`identity_4_42`/`identity_2_7`/`identity_8_99`）hash 逐字节不变，
且 `bootstrap_match` 的身份局路径代码完全未改（只在入口加了一个 `if guozhan` 早返回）。

## 不动什么

- **不动 ABI**（国战无新原语；胜利检查是按 ruleset 分发的普通函数调用）。
- **不动身份局 golden**（身份局路径代码零改动）。
- **不动 DSL/content-compiler**（国战武将/卡牌阶段 C 移植）。
- 不实现：暗将状态机（技能门控）、珠联璧合、野心家建国（拉拢人心/结盟）——阶段 C。

## 已知局限（阶段 C 待补）

1. **暗将**：当前双将全明（`unseen=[false,false]`），技能全挂载。阶段 C 需按 `unseen`
   门控技能生效，并实现明置触发（`showCharacter`）。
2. **AI 简陋**：国战 AI 复用身份局启发式（按势力判敌我），无珠联璧合/野心家策略，
   `allow_capped: true` 容忍 cap。阶段 C 强化后可收紧。
3. **野心家**：`compute_group_from_pair` 不同势力直接判 `ye`，但无「拉拢人心」建国、
   无 `wontYe` 人口阈值判定。阶段 C 按上游 `player.js` 实现。
4. **体力上限修正**：未实现珠联璧合 +1、特定武将的 max_hp 调整。阶段 C。

## 后续（阶段 C）

国战规则完整化 + 武将/卡牌内容移植（上游 `mode/guozhan/src/character/`）+ 与内容侧
agent 协调 ABI/npack。阶段 C 实施前在 `ABI-COORDINATION.md` 登记。
