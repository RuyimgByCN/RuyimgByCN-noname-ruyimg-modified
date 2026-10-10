# ADR 0599：国战阶段 A——状态模型扩展（身份局零回归已验证）

> 实施了 [ADR 0598](0598-guozhan-mode-design.md) §6 阶段 A。设计依据见 0598，本 ADR
> 只记录实施与验证。阶段 B（国战 mode 骨架）未做。

## 做了什么

按 0598 §D1/D2/D4，给国战模式预留状态模型字段，**身份局行为完全不变**：

### D1：`Player` 加两个字段（`state/players.rs`）
```rust
/// Guozhan lieutenant (副将) definition raw id. 0 in identity mode.
#[serde(default)]
pub character_def2: u32,

/// Guozhan: whether each general is face-down (暗将). [main, lieutenant].
#[serde(default = "default_unseen")]
pub unseen: [bool; 2],
```
全 `#[serde(default)]`，身份局序列化向后兼容。`bootstrap_match`（`pack_tw_1.rs:1724`）
补 `character_def2: 0, unseen: [false, false]`。

### D2：`hash.rs` 条件编码（身份局零漂移的保证）
新字段只在非默认值时写入 canonical hash，复刻既有 `turned_over` 模式：
```rust
if p.character_def2 != 0 {
    h.update(b"player-character-def2/v1");
    put_u32(&mut h, p.character_def2);
}
// ...
if p.unseen[0] || p.unseen[1] {
    h.update(b"player-unseen/v1");
    put_u8(&mut h, p.unseen[0] as u8);
    put_u8(&mut h, p.unseen[1] as u8);
}
```
身份局 `character_def2 == 0` 且 `unseen == [false,false]` → 两段都不写入 → **身份局
canonical hash 不变**。

### D4：`MatchOutcome::GroupWin`（`engine-api/outcome.rs`）
```rust
GroupWin { group: String },  // 国战势力判胜；group = "wei"/"shu"/"wu"/"qun"/"jin"/"ye"
```
hash.rs `hash_outcome` 加 `GroupWin => tag 6`。身份局 outcome 永不取此值 → 不影响身份局。

### Exhaustive match 补臂（4 处）
新增枚举变体触发的 `non-exhaustive` 编译错误，逐一补：
- `engine-core/src/hash.rs::hash_outcome`：tag 6 + put_str(group)。
- `engine-core/src/replay/mod.rs::outcome_tag`：返回 6（replay 持久化 tag）。
- `engine-ai/src/search/mod.rs`：`GroupWin => 0`（中性评分，国战 AI 阶段 B 细化）。
- `apps/engine-cli/src/main.rs`：`GroupWin => println!("{group} 势力胜利！")`。

## 验证（身份局零回归）

| 检查 | 基线（阶段 A 前） | 阶段 A 后 | 结论 |
|---|---|---|---|
| `cargo test -p conformance` | 3 向量全过 | **3 向量全过** | ✅ hash 零漂移 |
| `cargo test --workspace` 失败集合 | 257 套件 | **257 套件，逐项完全一致** | ✅ 零回归 |
| `cargo check -p engine-core --lib` warnings | 59 | **59** | ✅ 不增 |
| 编译错误（全 workspace + all-targets）| 0 | **0** | ✅ |

**失败集合对比方法**：阶段 A 前后各跑一次全量 `cargo test --workspace`，提取失败套件名
集合做 `comm -23`/`comm -13` 双向差集，**两组都为空**（257 = 257，无新增无修复）。
257 个失败是既有漂移（他人 `original_*` 领域），与本改动无关。

关键证明：conformance 的 `identity_4_42`/`identity_2_7`/`identity_8_99` 三个身份局向量
hash **逐字节不变**，直接验证了 D2 条件编码策略——新字段在身份局默认值下不进入 hash 流。

## 不动什么

- **不动 ABI**（国战无新原语；`GroupWin` 是 `MatchOutcome` 枚举扩展，非 `EffectRequest`）。
- **不动 golden**（身份局零漂移；国战向量阶段 B 新建）。
- **不动 DSL/content-compiler**（国战技能/武将阶段 C 移植）。
- 不实现国战规则（双将分发、势力判定、明暗状态机、野心家、胜利条件）——阶段 B。

## 后续（阶段 B）

阶段 B 将在 0598 §6 路线基础上：
1. `mode/guozhan.rs`：`distribute_guozhan_characters`（双将分发）、
   `compute_group_from_pair`（势力判定）、`check_guozhan_victory`。
2. `run_guozhan_match` 入口。
3. 国战 conformance 向量（从 0 新建）。

阶段 B 前在 `ABI-COORDINATION.md` 登记「将新增 `mode/guozhan.rs` + 改 `runner.rs`」。
阶段 C（内容移植）与内容侧 agent 协调 ABI/npack。
