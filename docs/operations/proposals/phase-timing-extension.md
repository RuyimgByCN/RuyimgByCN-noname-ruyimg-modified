# 提案：扩展「阶段时机」的枚举表达（2026-10-19）

> **为什么提**：**42 条缺陷里有 13 条属「引擎缺档」**，其中 **「阶段之前/之后」一类占 6 条，是最大的一类**。
> **扩这一个档位，能一次消掉 6 条缺陷**，且不需要逐技能改逻辑。

## 1 现状

### `Phase` 只有 5 个变体（`crates/engine-native/src/event.rs`）

```rust
pub enum Phase { Judge, Draw, Play, Discard, End }
```

**缺「准备阶段」（`Prepare`）** ⇒ **G18 那 2 条 + `dddmoyan` 只能借 `Judge`** ⚠️

### `EventData::Phase` 只有「开始/结束」两态

```rust
EventData::Phase { player: PlayerId, phase: Phase, begin: bool }
```

**缺「之前（before）」与「之后（after）」** ⇒ 上游的
`phaseUseBefore` / `phaseAfter` / `phaseJieshuBefore` 等**无法精确表达** ⚠️

## 2 受影响的 6 条缺陷

| 缺陷 | 上游 | 原生当前 | 缺什么 |
|---|---|---|---|
| `swd_rumeng` | `global: phaseUseBefore` | `Play, begin=true` | 「之前」 |
| `swd_fenglue` | `player: phaseUseBefore` | `Play, begin=true` | 「之前」 |
| `swd_fuchou2` | `global: phaseAfter` | `End, begin=false` | 「之后」（**任意阶段**）|
| `swd_reyingzi` | `player: phaseDrawBegin2` | `Draw, begin=true` | 「第二次」 |
| `gujian_mizhilianou` | `player: phaseAfter` | 未触发 | 「之后」 |
| `swd_baishouzhihu` | `PhaseEnd` + `begin==true` 语义冲突 | 注册与 filter 不一致 | 「结束前/后」区分 |

## 3 建议的最小改动

**方案 A（最小）**：给 `EventData::Phase` 加一个时序字段

```rust
pub enum PhaseTiming { Before, Begin, After, End }   // 新增
EventData::Phase { player, phase, timing: PhaseTiming }
```

- **`before` 与 `begin` 的区别**：`before` 在**阶段动作开始前**（可被取消/替换），`begin` 在**阶段已确立后**；
- **`After` vs `End`**：`End` 是阶段结束事件本体，`After` 是**该阶段所有结算完成后**；
- **`Begin2`** 可用「同一阶段内 `timing == Begin` 的第二次发射」表达，或新增 `BeginSecond`。

**方案 B（补充）**：`Phase` 加 `Prepare` 变体，覆盖 G18 那 2 条 + `dddmoyan`。

## 4 影响面

| 文件 | 状态 | 改动 |
|---|---|---|
| `crates/engine-native/src/event.rs` | **已跟踪** ✅ | 加 `PhaseTiming` / `Phase::Prepare` |
| `crates/engine-native/src/kernel/**` | **未跟踪** ❌ | **发射点**需按新时序发事件（**这是主要工作量**）|
| `content/host/filtering*.rs` | **未跟踪** ❌ | 6 条的 filter 改成新时序 |
| `crates/engine-native/tests/**` | **已跟踪** ✅ | **6 个 `#[ignore]` 用例转正**（改造后可删 ignore）|

## 5 验证方式

1. **`cargo test --offline -p engine-native`** 应仍为 **1538 / 0 / 21**（**新增能力不应改变既有行为**）；
2. **把 6 个 `#[ignore]` 逐个去掉**，跑通即证明该条已修；
3. **`scripts/report_phase_filters.py --check`** 应仍「一致」（**52 条不变**），但数值会变；
4. **`nc08_*` 上游对照测试**全绿后，对应来源的「规则差异」才可签。

## 6 成本估计

| 项 | 估计 |
|---|---|
| **`event.rs` 加枚举** | **1 小时**（**已跟踪，可直接改**）|
| **`kernel` 发射点** | **主要成本**（**需逐处确认「阶段前/后」的语义边界**）|
| **filter 改 6 条** | **各 1 行** |
| **测试转正 6 个** | **各需确认夹具** |

⇒ **不是「1 行修复」，而是「1 个能力 + 6 处接线」** ⚠️
⇒ **但它一次覆盖 6 条缺陷，且 5c/五行生克（「指定目标时」）与它同属「事件时机」一类，可一并设计** ✅
