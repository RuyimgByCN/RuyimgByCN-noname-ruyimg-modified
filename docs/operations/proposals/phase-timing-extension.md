# 提案：扩展「阶段时机」的枚举表达（2026-10-19）

> **为什么提**：**93 条缺陷里，属「引擎缺档」的有 41 条**（B 类），其中 **「用牌前/指定目标时」一项就占 25 条，是最大的一类**。
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

### ✅ 已查（第 703 轮）：`Dying` 是**内联**发射的 ⇒ `Damage` 确实早于「结算完成」

`crates/engine-native/src/kernel/damage.rs:977-987`：

```rust
// 内联濒死检查：体力 ≤ 0 且尚未处于濒死。
let entering = ctx.core.player(d.target).hp <= 0 && !ctx.core.player(d.target).dying;
if entering {
    ctx.core.player_mut(d.target).dying = true;
    ctx.log(LogEvent::DyingEntered { player: d.target });
    ctx.emit(EventData::Dying { player: d.target, source: d.source });   // ← 内联
}
// 铁索连环传导：属性伤害结算后…
```

⇒ **濒死是在「伤害结算函数的中途」内联发射的**，**不是独立阶段** ✅
**并且行号也证实了顺序**：`EventData::Damage` 的发射点在 **`damage.rs:518 / 558 / 587 / 637`**，
而濒死内联在 **`:977`** ⇒ **`Damage` 必然早于濒死** ✅（**518 < 977**）

⇒ **所以 `EventKind::Damage` 必然早于「濒死救援完成」** ❗
⇒ **「`damageEnd`（全部结算完成后）」这一档确实不存在** ⚠️

**建议**：**新增 `EventKind::DamageResolved`**，在**濒死处理与铁索传导之后**发射，用于覆盖 `damageEnd`。
⇒ **批 B 第 1 问已回答：需要新档位，不是语义澄清** ✅

### 7.3 嵌套结算（虚拟杀在水攻内发起被丢弃）—— **覆盖 1 条**

**上游**：技能结算内**可以再发起一次完整的牌结算**（嵌套）。
**现状**：`luweiyan` 的虚拟杀在【水攻】结算内**被丢弃**。

**建议**：**这一条不是「缺枚举」，而是「结算栈的能力」** ⚠️

### ✅ 已查（第 702 轮）：`kernel` **没有通用的嵌套结算机制或守卫**

| 检查 | 结果 |
|---|---|
| `kernel/**` 里 `nested` / `reentran` / `resolving` 等 | **零命中** ❌ |
| `kernel/duel.rs` | 有 `second_pending`——**但那是决斗专用的一次性标记**，不是通用嵌套机制 |
| `engine/*.rs` | 只有 `snapshot.rs:91` 的 `checkpoint_nested`（**快照相关，非结算**）|

⇒ **结论：`kernel` 里没有「允许/禁止嵌套结算」的显式机制** ✅
⇒ **因此 `luweiyan` 的虚拟杀被丢弃，更可能是「技能效果步没有把它加入结算队列」** ⚠️
⇒ **要定案仍需读 `step/skills_*.rs` 里 `luweiyan` 的效果步** ❗（**本轮未做**）

**对提案的影响**：**批 B 的第 2 问已回答一半**——**不是「引擎禁止嵌套」，而是「没有通用机制」** ✅
⇒ **这一条可能从「能力缺口」降级为「单点 bug」** ⚠️

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

---

## 9 缺档分类**从五类扩到七类**（2026-10-19，第 711 轮）

审计 `L20` 时又发现两类（见规则基准「L20 对照」）：

| # | 缺档 | 覆盖 | 状态 |
|---:|---|---|---|
| 1 | **阶段之前 / 之后 / 第二次** | **6 条** | ✅ 可立即设计 |
| 2 | **准备阶段 / 回合开始** | **3 条** | ✅ 可立即设计 |
| 3 | **指定目标时** | **2 条** | ✅ 可立即设计 |
| 4 | **`damageEnd` 后的窗口** | **1 条** | ⚠️ 需新增 `DamageResolved` |
| 5 | **嵌套结算** | **1 条** | ⚠️ 可能降级为单点 bug |
| **6** | **游戏开始（`gameStart`）** | **1 条**（`_xigua_xueliangfanbei_`）| ❗ **新发现** |
| **7** | **死亡前（`dieBegin`）** | **1 条**（`_xigua_zhuwanzhongji_`）| ❗ **新发现** |

### 这两类的性质

- **`gameStart`**：**没有「游戏开始」事件** ⇒ 该技能**在原生里永远不会触发** ⚠️
  - 修法：`EventKind::GameStart`，在**建局完成、第一阶段之前**发射；
- **`dieBegin`**：**引擎有 `Death`（死亡后）无「死亡前」** ⇒ **在「死亡前」修改死活/身份的技能无法表达** ⚠️
  - 修法：`EventKind::DeathBefore`，**与 `DamageBefore` 对称**（**已有先例** ✅）

⇒ **`DeathBefore` 的实现成本应低于其它类**（**`DamageBefore` 是现成范式** ✅）

### 对「缺档总数」的判断

**七类覆盖约 15 条缺陷**，**但七类可能仍不是全部** ——
**本轮只是又审计了一个来源（`L20`，6 项）** 就新增两类 ⇒ **剩余 16 个有规则内容的来源里可能还有** ⚠️
⇒ **「缺档清单」应视为「已发现集合」，不是「完备集合」** ✅

---

## 10 ⚠ **收益重估：批 A 从 8 条 → 约 30 条**（2026-10-19，第 732 轮）

第 731 轮用枚举脚本（`report_event_arms.py --family card|death`）把两个缺档的**实例数**数了出来，
**这彻底改变了优先级判断**。

| 缺档 | 本提案原先记 | **枚举实测** | 倍率 |
|---|---:|---:|---:|
| **「指定目标时 / 用牌前」**（`useCardToBegin`/`useCardToBefore`/`useCardToTargeted`/`useCard`）| **2** | **25** | **12.5×** |
| **「死亡前 `dieBegin`」** | **1** | **5** | **5×** |
| **「阶段前后 / 准备阶段」** | 6 + 3 | 6 + 3（不变）| — |
| **批 A 合计覆盖** | **8** | **约 30** | **3.75×** |

### 为什么会有 12.5 倍的差距

我此前是**按缺陷发现的顺序**记账的：先撞到 5c（`wuxing_shengke`），再撞到五行生克（51），
于是记成「2 条」。但**同一个缺档在代码里被用了 25 次** —— 只有**枚举**才能看见这一点。

⇒ **这是「抽样 vs 全量」的教科书差别**：抽样能发现「有问题」，只有全量能回答「有多少」。

### 工程含义的变化

| 视角 | 结论 |
|---|---|
| **按缺陷数看** | 「修 25 条」听起来要排期 |
| **按根因看** | 「加 1 个 `EventKind::CardTargeted` + 把 25 处 filter 指向它」——**25 处是机械替换** |
| **ROI** | **一个枚举变体换 25 条缺陷**，是本会话所有候选中**收益最高的一项** ✅ |

### 修订后的建议顺序

| 优先级 | 动作 | 覆盖 | 前置 |
|---|---|---:|---|
| **1** | **`EventKind::CardTargeted`**（用牌前/指定目标时）| **25 条** | 需 `kernel` 发射点权限 |
| **2** | **`PhaseTiming` + `Phase::Prepare`** | **9 条** | 同上 |
| **3** | **`EventKind::DeathBefore`**（与 `DamageBefore` 对称）| **5 条** | 同上（**有现成范式，最省力**）|
| **4** | **`DamageResolved`**（`damageEnd` 后窗口）| 1 条 | 同上 |
| **5** | **A 类 32 条**（改 pattern/删分支）| 32 条 | 需 `content/host/**` 权限 |

⇒ **前 3 项（3 个枚举变体）合计覆盖 39 条**，**占全部 77 条的 51%** ❗
⇒ **全部都需要 `kernel/**` 或 `content/host/**` 的写权限**——**本会话只有读权限** ⚠️

---

## 11 完整缺档总表：**13 类 + U05 不可达 15 条**（2026-10-19，第 764 轮更新）

前几节按「发现顺序」写，本节按**引擎缺的槽位**归并（数据来自[缺陷台账](../evidence/2026-10-19-defect-ledger/README.md)）。

### 11.1 需新增 `EventKind`/`Phase` 的（**B 类，11 类**）

| # | 缺的槽位 | 覆盖条数 | 台账里的表现 |
|---:|---|---:|---|
| 1 | **「用牌前 / 指定目标时」**（`useCardToBegin`/`ToBefore`/`ToTargeted`）| **23** | `需档` |
| 2 | **「sha 幕」**（`shaBegin`/`shaAfter`/`shaBefore`）| **6** | `需档(sha 幕)` |
| 3 | **「死亡前」**（`dieBegin`）| **4 + L20c** | `需档(死亡前)` |
| 4 | **「第 2/3/4 次」**（`*Begin2/3/4`）| **1 + L20a** | `第3次` |
| 5 | **「准备阶段」**（`phaseZhunbeiBegin`）| **2 + L03?** | `G18` |
| 6 | **「阶段之前」**（`phaseUseBefore`）| **1** | `G15(之前档)` |
| 7 | **「阶段之后」**（`phaseAfter`）| **1** | `G15(phaseAfter)` |
| 8 | **「游戏开始」**（`gameStart`）| **1** | `L20b` |
| 9 | **「用牌目标后」**（`useCardToPlayered`）| **1** | `L20d` |
| 10 | **「失去体力后」**（`loseHpEnd`）| **1** | `L03a` |
| 11 | **「判定前」**（`judgeBefore`）| **1** | `L03c` |

### 11.2 不需要新增枚举的（**A 类**，43 条）

| 缺陷族 | 条数 | 改法 |
|---|---:|---|
| **G14**（钉 `Judge`，上游 `phaseBegin`）| **14** | `begin: true, ..` |
| **G23**（`PhaseEnd` 未钉阶段 ⇒ 5×/回合）| **12** | 钉 `End` / 改 meta |
| **G20**（`phaseEnd` 写成 `begin: true`）| **5** | `begin: false` |
| **G24**（濒死主动技挂成 `Death`）| **4** | meta 改 `Dying` |
| **G19**（`!= owner`）| **3** | 删条件 |
| 其余单条 | 5 | 见台账 `fix` 列 |

### 11.3 ⇒ 修复策略的两句话

1. **11 类缺档覆盖约 40 条（42%）** ⇒ **每加一个槽位平均消 3.6 条**，其中
   **「用牌前/指定目标时」一项就占 23 条**（**ROI 第一**）；
2. **A 类 43 条是 5 种固定改法** ⇒ **每个族一次机械替换**，不是 43 次独立工作。

⇒ **所以「120 条缺陷」听起来吓人，实际是「11+ 个槽位 + 5 种改法」**（**第 763 轮又并入 `U05` 的 15 条不可达行为**）✅
⇒ **这也解释了为什么逐条修不如先扩档** ❗
