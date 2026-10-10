# ADR 0598：国战（Guozhan）模式状态模型扩展设计

> **本 ADR 是设计决策，不动代码。** 实现按 §6 分阶段路线推进，每阶段另立实施 ADR。
> 调研结论由本文保留；对应一次性底稿已于 2026-09-16 清理。

## 背景

`mode/` 目录当前**只有身份局**（`identity.rs`，178 行）。国战、versus、斗地主均无实现；
全仓库 7 处「国战」引用都是个别技能（英魂/刚烈）的注释，不是模式代码。

架构计划把国战列为 M4 模式层缺口。任务是**状态模型层面的扩展，不是内容问题**：
身份局的 `Player` 是单武将，国战是双武将（主将 + 副将）+ 暗将 + 势力阵营 + 野心家，
这套状态在当前模型里不存在。

任务特别警告：**「改状态模型，重钉 golden 的成本随内容量线性增长（现已 2897 将）」**，
故应在扩内容之前完成状态模型扩展。本 ADR 锁定扩展方案，使后续实现有明确约束。

## 现状与缺口

### 既有可复用基础
- **`player_group(player) -> Option<&str>`**（`state/mod.rs:2053`）：优先查 `group_overrides`，
  否则 fallback 到角色 `character.group`。身份局里袁术「群」、孙权「吴」已可查。
  trigger/dispatch.rs 有 10+ 处依赖它。**国战可复用此入口，无需新查询函数。**
- **`group_overrides: BTreeMap<PlayerId, String>`**（`state/mod.rs:95`）：可承载国战运行时势力。
- **canonical hash 对 `group_overrides` 用条件编码**（`hash.rs:240`，非空才写
  `player-group-override/v1`）→ 身份局 `group_overrides` 默认空 → 身份局 hash 不含此段。

### 状态模型缺口（`Player`，`state/players.rs`）

| 字段 | 现状 | 国战需求 |
|---|---|---|
| 副将 | 仅 `character_def: u32`（单将） | 需副将 def + 主/副体力上限分开（国战体力 = 主将上限 + 副将上限，特定组合修正） |
| 暗将 | 无 | 需主/副将明暗状态（`isUnseen(0/1)`）；暗将技能不生效 |
| 势力 | 单将可查；双将未定义 | 双将势力判定：同势力→该势力；不同势力→野心家（`ye`） |
| 身份 | `Identity { Lord, Loyalist, Rebel, Renegade }` | 国战无身份局角色，按势力阵营判胜 |

### 枚举缺口
- `Identity`（`state/players.rs`）：无野心家。国战用势力（`group`）而非身份判胜，
  `Identity` 可保持不变（国战玩家统一 `Lord` 占位或新增 `Guozhan` 标记）。
- `MatchOutcome`（`engine-api/outcome.rs`）：仅 `RebelWin`/`LoyalistWin`/`RenegadeWin`/`Draw`/`Aborted`。
  国战需 **`GroupWin { group: String }`**（某势力阵营胜）。

### 入口缺口
- `run_identity_match`（`runner.rs:52`）硬编码身份局。`MatchConfig` 无 mode 字段。
  国战需 `run_guozhan_match` 或 `MatchConfig` 加 `mode` 分发。

## 决策

### D1：`Player` 扩展字段（向后兼容）

```rust
pub struct Player {
    // ... 既有字段 ...
    /// 国战副将 def（0 = 身份局单将 / 未设）。
    #[serde(default)]
    pub character_def2: u32,
    /// 国战主/副将是否暗置（[主, 副]；身份局默认全明 [false, false]）。
    #[serde(default)]
    pub unseen: [bool; 2],
}
```

- 全部 `#[serde(default)]`，身份局构造（`bootstrap_match`）不填这些字段 → 默认值。
- `bootstrap_match` 的 `Player { ... }` 字面量需补这两个字段（或加 `..Default::default()`，
  但 `Player` 非 `Default`；最简是显式写 `character_def2: 0, unseen: [false, false]`）。

### D2：canonical hash 条件编码（身份局零漂移）— 关键约束

`hash.rs` 的 Player 段，新字段**只在非默认值时编码**，复刻既有 `turned_over`/
`disabled_equip_slots` 的「if 非默认才写」模式：

```rust
// 既有：if p.turned_over { h.update(b"player-turned-over/v1"); }
if p.character_def2 != 0 {
    h.update(b"player-character-def2/v1");
    put_u32(&mut h, p.character_def2);
}
if p.unseen[0] || p.unseen[1] {
    h.update(b"player-unseen/v1");
    put_u8(&mut h, p.unseen[0] as u8);
    put_u8(&mut h, p.unseen[1] as u8);
}
```

**约束**：身份局 `character_def2 == 0` 且 `unseen == [false,false]` → 这两段不写入 →
**身份局的 3 个 conformance 向量 + 所有 `original_*` golden hash 零漂移**。
此约束是本 ADR 的核心保证，实现时必须验证（见 §7）。

### D3：势力判定复用 `group_overrides`

不新增 `Player.group` 字段。国战初始化时，按双将算出势力写入 `group_overrides`：
- 主副同势力 → 写该势力；
- 不同势力 → 写 `ye`（野心家）；
- 暗将阶段势力可能未定（留空，`player_group` 返回 None，技能按需处理）。

明置副将导致势力变化时，更新 `group_overrides`。`player_group` 入口不变，所有既有
trigger/dispatch 调用点零改动。

### D4：`MatchOutcome` 加 `GroupWin`

```rust
pub enum MatchOutcome {
    // ... 既有 ...
    /// 国战：某势力阵营胜利。
    GroupWin { group: String },
}
```

`hash.rs` 的 `MatchOutcome` 编码段需加对应臂（条件式或新 tag）。身份局 `outcome` 永不取
`GroupWin`，故身份局 hash 不受影响。

### D5：国战 mode 新模块 + 入口

- `mode/guozhan.rs`（新）：`distribute_guozhan_characters`（双将分发）、
  `compute_group_from_pair`（势力判定）、`check_guozhan_victory`（势力阵营判胜）。
- `mode/mod.rs`：导出。
- `runner.rs`：加 `run_guozhan_match(cfg) -> MatchReport`（或 `MatchConfig` 加 `mode` 字段后
  在 `run_match` 内分发）。倾向独立函数，避免动既有 `run_identity_match` 签名。

## 国战规则要点（上游 `mode/guozhan/` 语义，实现时参照）

- **势力**：wei/shu/wu/qun/jin（+ ye 野心家；key/jin 见于扩展包）。
- **双将分发**：从国战武将池按势力配对，双势力武将（`doubleGroup`）优先匹配已有势力。
- **暗将**：暗置的将技能不生效；明置由技能/事件触发（`showCharacter`）。
- **珠联璧合**：特定主副组合触发奖励（额外摸牌/回复）。
- **野心家**：某势力存活数 > 半数时，弱势力玩家可变野心家；野心家可「拉拢人心」建国结盟。
- **胜利**：某势力消灭所有异势力玩家 → 该势力阵营胜（`GroupWin { group }`）。

## 分阶段路线（§6）

### 阶段 A：最小状态模型扩展（不实现国战规则）
1. `Player` 加 `character_def2`/`unseen`（D1）。
2. `hash.rs` 条件编码（D2），跑全量验证**身份局零回归**。
3. `MatchOutcome::GroupWin`（D4）。
4. `bootstrap_match` 补新字段默认值。
- **验收**：conformance 3 向量不变、original_* 失败集合与脏基线完全一致（无新增）、
  engine-core warnings 不增。

### 阶段 B：国战 mode 骨架 + 最小规则
1. `mode/guozhan.rs`：双将分发、`compute_group_from_pair`、`check_guozhan_victory`。
2. `run_guozhan_match` 入口。
3. 国战 conformance 向量（从 0 新建，至少 1 个势力判胜向量）。
- **验收**：身份局仍零回归；国战向量全过。

### 阶段 C：国战规则完整 + 内容移植
1. 暗将状态机、珠联璧合、野心家建国完整实现。
2. 国战武将包（上游 `mode/guozhan/src/character/`）移植为 npack。
3. 国战卡牌包（`card/guozhan.js`）进 `card_index`（任务说现仅 6 包 57 张）。
4. 与内容侧 agent 协调 ABI 段、npack 打包。

**本 ADR 只锁定设计，不规定各阶段排期。** 每阶段实施前另立实施 ADR，在
`ABI-COORDINATION.md` 登记（阶段 A/B 碰 `state/`/`hash.rs`/`mode/`，须与并行 agent 协调）。

## 验证策略（§7）

每个阶段完成后必须验证：
1. **身份局零回归**：conformance 3 向量（`identity_4_42`/`identity_2_7`/`identity_8_99`）
   hash 不变；`original_*` 失败集合与该阶段开始前的脏基线**完全一致**（无新增失败）。
2. **新字段默认值隔离**：身份局 `Player` 的 `character_def2 == 0`、`unseen == [false,false]`，
   hash 编码不写入这两段（用单测断言 `hash_with_default_player == hash_without_fields`）。
3. **国战向量**：阶段 B 起新建国战 conformance 向量，单独管理（不与身份局共享向量）。

## 当前不做什么

- **本轮不动代码**（用户决策 2026-08-04）。工作区已有 244 套件漂移（他人领域），
  叠加 canonical state 改动会让局面更复杂，且 `state/`/`hash.rs` 与并行 agent 高冲突。
- 本 ADR 不占用 ABI 号（不动 ABI；阶段 C 内容移植时按内容侧流程领取）。
- 不在 `scripts/` 目录加国战脚本（避免触碰其所有权）。

## 后续

- 实施阶段 A 前：确认工作区已收敛（244 漂移被各 owner 重钉），在 `ABI-COORDINATION.md`
  登记「将改 `state/players.rs` + `hash.rs`」。
- 阶段 A 实施后立实施 ADR（含实际 hash 验证数字）。
- 阶段 C 与内容侧 agent 协调国战武将/卡牌的 npack 与 ABI。
