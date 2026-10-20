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

---

## 7 另外三类缺档（**同一提案的其余部分**，2026-10-19）

### 7.1 「指定目标时」（`useCardToBegin`）—— **覆盖 2 条**

**上游语义**：**牌被使用、目标已指定、但效果尚未结算**（`trigger: { target: "useCardToBegin" }`）。
**原生现状**：**只有 `EventKind::CardUsed`（使用后）** ⇒ **5c（五行生克）+ 51（五行生克分支）都拿错了时机**。

**建议**：新增 `EventKind::CardTargeted`，**在 `CardUsed` 之前发射**，载荷沿用 `CardUsed` 的结构（`card` + `targets`）。

**影响**：`event.rs`（可改 ✅）+ `kernel` 的 `use_card` 路径（未跟踪 ❌）+ 2 条 filter。

### 7.2 「`damageEnd` 之后的窗口」—— **覆盖 1 条**

**上游**：`趁火打劫` 用 `damageEnd`（**伤害结算完成后**），原生**窗口未出现**。
**现状**：`EventKind::Damage` 是「伤害发生」后，但**可能早于「伤害结算完成」**（如濒死插入）。

**建议**：**明确 `Damage` 与「结算完成」的边界**，或新增 `DamageResolved`。
**这一条需要先确认语义**（**不是纯加枚举**）⚠️

### 7.3 嵌套结算（虚拟杀在水攻内发起被丢弃）—— **覆盖 1 条**

**上游**：技能结算内**可以再发起一次完整的牌结算**（嵌套）。
**现状**：`luweiyan` 的虚拟杀在【水攻】结算内**被丢弃**。

**建议**：**这一条不是「缺枚举」，而是「结算栈的能力」** ⚠️ ——
**需先确认 `kernel` 是否允许嵌套 `CardUsed` 事件**，才能判断是「缺能力」还是「有 bug」。

## 8 五类缺档的**性质差异**（重要）

| 缺档 | 性质 | 修法 |
|---|---|---|
| **阶段前后/`Prepare`** | **枚举缺值** | **加枚举变体** ✅ |
| **指定目标时** | **枚举缺值** | **加 `EventKind` 变体** ✅ |
| **`damageEnd` 后窗口** | **语义边界未定** | **先定语义，再决定加不加** ⚠️ |
| **嵌套结算** | **能力未知** | **先查 `kernel` 是否支持** ⚠️ |

⇒ **前两类是「确定可扩」，后两类是「先要回答问题」** ❗
⇒ **因此提案分两批**：**批 A（可立即设计）= 阶段前后 + 指定目标时（覆盖 8 条）** ✅；
**批 B（需先澄清）= `damageEnd` 边界 + 嵌套结算（覆盖 2 条）** ⚠️
