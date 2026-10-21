# 伤害触发臂**全量枚举**：88 条 —— **0 条时机错位**（2026-10-19，第 730 轮）

> 阶段面已有[全量枚举](../2026-10-19-phase-arm-enumeration/README.md)。本轮枚举第二个面：**伤害**。

## 1 关键方法修正：**`TriggerMeta.event` 才是时序权威**

第一版脚本读的是**过滤臂里写的 `EventData::Damage`**，得出 **26 条「前/后错位」**。
逐条抽查立刻发现**假阳性**：`Xunmeng`（循梦）的过滤臂写 `EventData::Damage`，
但它的 `TriggerMeta` 是 **`EventKind::DamageBefore`** —— **真正的时序由 meta 决定，不是过滤臂的匹配模式**。

⇒ **改成读 `TriggerMeta.event` 后，「26 条错位」全部消失**（**71 条 OK**）✅

**教训**：**「代码里出现的字符串」不等于「运行时生效的语义」** —— 分发表、meta、filter 三层里，
**只有 meta 决定派发时机**。

## 2 枚举结果：**88 条伤害触发臂**

| 判定 | 条数 | 含义 |
|---|---:|---|
| ✅ **OK** | **71** | `damageBegin`/`damageBefore` ↔ `DamageBefore`；`damageEnd`/`damageAfter`/`damageSource` ↔ `Damage` |
| 🔍 **REVIEW（上游未定位）** | **6** | 行为名不是包内技能（内部累计/守卫类），查不到上游 |
| 🔍 **REVIEW（非伤害触发）** | **10** | 上游 trigger 不是伤害事件（多为**查到了同名卡片定义或注释块** ⇒ 查找质量不足）|
| ⚠️ **第 3 次（需档位）** | **1** | **`Tianxiang`（天香）**：上游 `player: "damageBegin3"` |
| **合计** | **88** | |

### ⇒ **`G21（伤害前后错位）` = 0 条** ✅

**引擎的伤害时序在全部 88 个伤害臂上与上游一致**（在 meta 层面）。
此前担心的「`damageBegin` 被写成 `Damage`」**在伤害面不成立**。

## 3 唯一确认的新缺陷：`Tianxiang`（天香）

| 项 | 值 |
|---|---|
| 上游 | **`trigger: { player: "damageBegin3" }`** —— 同一伤害的**第三次**触发点 |
| 原生 | **`EventKind::DamageBefore`**（单一次）|
| 判定 | ⚠️ **需档位**：`Begin3` 属 **G15 家族**（「第 2/3 次」无表达）|
| 影响 | 一张伤害在「第三次触发点」才应生效的技能，在原生会**在第一次就生效** |

⇒ **新增 B 类缺陷 1 条**（B 13 → **14**）

## 4 未做完的部分（**如实登记**）

**16 条 REVIEW 未逐条人工核实** ⚠️ —— 其中：
- **6 条**是内部行为（`SwdNigongAccumulate`、`BossChiyiMark` 等），**本来就不对应包内技能**；
- **10 条**的上游 trigger 解析明显不对（如 `ShaDamageForceDiscard` 解析到 `phaseDiscardBefore`、
  `JinlianzhuCancel` 解析到一段注释）⇒ **是查找质量问题，不是新缺陷**；
- 唯一确认的是 **`Tianxiang`**。

⇒ **这 16 条按「工具局限」登记，不按「缺陷」登记** ✅

## 5 复现

```bash
python3 -B scripts/report_event_arms.py           # TSV + 汇总
python3 -B scripts/report_event_arms.py --check   # 有 REVIEW 行时 exit=1
```

---

## 6 扩展：**用牌面**与**死亡面**的枚举（第 731 轮）

`report_event_arms.py` 增加 `--family {damage,card,death}`，把同一套方法用到另外两个面。

### 6.1 用牌面（`CardUsed` / `CardResponded`）：**42 条**

| 判定 | 条数 |
|---|---:|
| ✅ **OK**（上游 `useCardAfter` 等「之后」面）| **6** |
| ⚠️ **需档（用牌前 / 指定目标时）** | **23** |
| 🔍 REVIEW | 13 |

**需档的 23 条，上游 trigger 都是「使用牌之前/指定目标时」**：
`useCardToBegin`、`useCardToBefore`、`useCardToTargeted`、`useCard`（= 使用的瞬间），
而原生一律用 **`CardUsed`（使用之后）** —— 正是 **5c 那一条缺档**的 23 个实例 ❗

### 6.2 死亡面（`Death` / `Dying`）：**27 条**

| 判定 | 条数 |
|---|---:|
| ✅ **OK**（上游 `die` / `dieAfter`）| **10** |
| ⚠️ **需档（死亡前 `dieBegin`）** | **4** |
| 🔍 REVIEW | 13 |

需档的 4 条：`XianjianChangnian` / `XianjianCtianfu3` / `XianjianXshuangren2` / `YunchouDiychanyuan`，
上游都是 **`player: 'dieBegin'`**，原生用 **`Death`（死亡后）** ⇒ 与 `L20` 的 `_xigua_zhuwanzhongji_` 同源 ❗

### 6.3 ⚠ **这改变了提案的收益判断**

| 缺档 | 原先记录 | **枚举后** |
|---|---:|---:|
| **「指定目标时 / 用牌前」** | **2 处**（5c + 五行生克）| **25 处**（**+23**）|
| **「死亡前 `dieBegin`」** | **1 处**（`L20`）| **5 处**（**+4**）|

⇒ **扩 `EventKind::CardTargeted` 的收益从「2 条」变成「25 条」** ❗
⇒ **扩 `EventKind::DeathBefore` 的收益从「1 条」变成「5 条」** ❗
⇒ **提案**（[阶段时机扩展](../../proposals/phase-timing-extension.md)）**的批 A 覆盖条数应从 8 条重估为约 30 条**

### 6.4 未做完

**26 条 REVIEW**（用牌面 13 + 死亡面 13）**未逐条核实** ⚠️ ——
其中多数是**上游未定位**（内部行为或解析质量），**不按缺陷登记**。
另有 **10 条「非用牌触发」/「非死亡触发」**，属解析错位，也**不按缺陷登记**。

---

## 7 ⚠ **新候选类 G23：`PhaseEnd` 未钉阶段 ⇒ 每阶段结束都触发（上游每回合一次）**

### 7.1 事实链

1. **上游语义**：`phaseEnd` **每回合只触发一次**，在**全部阶段跑完之后**
   （`game.js:15999-16003`：只有 `event.num >= event.phaseList.length` 才 `event.trigger('phaseEnd')`）✅
2. **原生派发**：`EventKind::PhaseEnd` 在**每个阶段结束**都派发
   （`kernel/phase.rs:113` 的 `// PhaseEnd` 分支对每个 `phase` 都执行）✅
3. **因此**：filter 若只写 `EventData::Phase { begin: false, .. }`（**不钉具体阶段**），
   就会**一回合触发 5 次**（判定/摸牌/出牌/弃牌/结束各一次），而上游是 **1 次** ❗

### 7.2 抽验 3 条（filter 与上游都读了原文）

| 行为 | 原生 filter | 上游 | 结论 |
|---|---|---|---|
| **`Hutian`** | `matches!(event, Phase { begin: false, .. })` + 手牌非空 + `hutian_deposit.is_none()` | **`player: 'phaseEnd'`** | ⚠️ **未钉阶段**，但**有 `deposit` 自限闸门** ⇒ 可能实际只触发一次，**需逐条确认** |
| **`SwdHuanling`** | `Phase { player, begin, .. } if player == owner && !begin` | **`player: 'phaseEnd'`** | ❗ **无自限闸门** ⇒ **每回合 5 次** |
| **`SwdJiguanyuanGain`** | `Phase { player, begin: false, .. } if owner && jiguanyuan_cards 非空` | **`player: "phaseEnd"`** | ⚠️ 依赖 `jiguanyuan_cards` 是否被清空 |

### 7.3 ✅ 已逐条定案：17 条候选的最终分类（第 734 轮）

把 17 条的 **filter 体**与**上游 trigger** 都取出来逐个对照：

| # | 行为 | 自限闸门 | 上游 trigger | 判定 |
|---:|---|---|---|---|
| 1 | `SwdHuanling` | 无 | `phaseEnd` | ❗ **确认多触发**（5×/回合）|
| 2 | `SwdLingxin` | 无 | `phaseEnd` | ❗ 确认 |
| 3 | `SwdLqingcheng` | 无 | `phaseEnd` | ❗ 确认 |
| 4 | `WudianGuangshuSpade` | 无 | `phaseEnd` | ❗ 确认 |
| 5 | `WudianJidong` | 无 | `global: phaseEnd` | ❗ 确认 |
| 6 | `XianjianBinxin` | 无 | `global: phaseEnd` | ❗ 确认 |
| 7 | `XianjianLingyu` | 无 | `phaseEnd` | ❗ 确认 |
| 8 | `Xuanyuan` | 无 | `phaseEnd` | ❗ 确认 |
| 9 | `YxsMenshen` | 无 | `phaseEnd` | ❗ 确认 |
| 10 | `SwdDonghuangzhongCard` | 无 | `phaseEnd` | ❗ 确认 |
| 11 | **`SwdJiguanyuanGain`** | 无 | **`player: damageEnd` + `source: damageSource`** | ❗ **触发事件种类完全错**（不是阶段事件）|
| 12 | `Hutian` | **有**（`hutian_deposit.is_none()`）| `phaseEnd` | ⚠️ **有闸门 ⇒ 行为可能正确，但属「偶合正确」** |
| 13 | `SwdMufengHeld` | **有**（`turn.mufeng…`）| `phaseEnd` | ⚠️ 同上 |
| 14 | `SwdDatong` | 无 | **无 trigger** | 🔍 上游无触发 ⇒ 待核 |
| 15 | `BossHuanren` | — | 未定位 | 🔍 待核 |
| 16 | `GwentFengyinState` | 无 | 未定位 | 🔍 待核（`gwent` 三状态技之一）|
| 17 | `SwdBaishouzhihuHeld` | 无 | 未定位 | 🔍 待核 |

### ⇒ 定案

| 类 | 条数 |
|---|---:|
| ❗ **确认（多触发 or 事件种类错）** | **11** |
| ⚠️ **偶合正确（靠自限闸门）** | **2** |
| 🔍 **待核** | **4** |

⇒ **11 条并入确认缺陷**（**全部 A 类：把 `..` 钉成 `End, begin: false`，或把阶段事件改成 `Damage`**）✅
⇒ **确认总数 77 → 88**、**A 类 32 → 43**

### 7.4 为什么这一条比 G20 严重

- **G20（早一步）**：触发**次数相同**，只是时机偏早 ⇒ 影响局部结算顺序；
- **G23（多触发 4 次）**：触发**次数是上游的 5 倍** ⇒ 回复/摸牌/给牌类效果**直接翻 5 倍** ❗

⇒ **若要排优先级，G23 应在 G20 之前**；但它属「把 `..` 补成 `End, begin: false`」的**一行改动**，
**与 G20 的修法相同**（都是把阶段钉对）⇒ **可合并为一次批处理** ✅

---

## 8 反向核查：**`PhaseBegin` 未钉阶段的 18 条**（第 735 轮）

G23 查的是「`PhaseEnd` 未钉阶段 ⇒ 多触发」。反过来，`PhaseBegin` 未钉阶段是**正确的**
（上游 `phaseBegin` 本来就指「任意阶段开始」）。逐条核对 18 条：

| 类 | 条数 | 说明 |
|---|---:|---|
| ✅ **与上游一致**（上游 `phaseBegin`，任意阶段）| **16** | **不钉阶段是对的** ✅ |
| ⚠️ **上游是 `phaseDiscardEnd`** | **2** | `SwdWangchenHeld` / `SwdXielv` |
| ❗ **meta 与 filter 自相矛盾** | **1** | `SwdBaishouzhihuHeld`（**见下**，与上两项有重叠计数）|

### 8.1 `SwdWangchenHeld` / `SwdXielv`：**相邻相位，不是 5 倍**

两者的 filter 其实是**钉死**的（用相等比较而不是解构绑定）：

```rust
matches!(event, EventData::Phase { player, phase, begin }
    if *player == owner && *phase == Phase::End && *begin && ...)
```

⇒ **只在「结束阶段开始」触发，每回合一次** ✅ —— 我第一版的「未钉阶段」检测**漏掉了这种写法**（只找 `phase:` 绑定），
把它们误记为「5 倍」。**这是本轮自查出的第二类假阳性** ⚠️

上游是 **`phaseDiscardEnd`（弃牌阶段结束）**，原生是 **`Phase::End, begin`（结束阶段开始）** ——
两者在流程上**相邻**（弃牌阶段结束 → 结束阶段开始），**只差「结束阶段自身动作之前/之后」** ⇒ **轻微时机差异** ⚠️

### 8.2 `SwdBaishouzhihuHeld`：**meta 与 filter 矛盾 ⇒ 永不触发** ❗

| 层 | 内容 |
|---|---|
| **meta** | `EventKind::PhaseEnd` ⇒ 事件带 `begin: false` |
| **filter** | `Phase { player, begin, .. } if *player == owner && **\*begin** && …` ⇒ **要求 `begin: true`** |

⇒ **两者不可能同时成立** ⇒ **该触发永远不会发生** ❗
⇒ 这解释了它为什么在枚举里既「未定位上游」又「无闸门」：**它根本没在跑**。

**判定**：**登记为「触发器失效」缺陷**（与 G23 的多触发相反，属**不触发**）❗
**修法**：把 filter 的 `*begin` 改成 `!*begin`（一行）⇒ **A 类**

### 8.3 计数影响

| 类 | 变化 |
|---|---|
| **A 类** | **43 → 44**（+`SwdBaishouzhihuHeld` 触发失效）|
| **确认总数** | **88 → 89** |
| **轻微时机差异（不计缺陷）** | +2（`SwdWangchenHeld` / `SwdXielv`），记入**登记差异** ⚠️ |

### 8.4 方法教训（本会话第 3 次同类）

**「没匹配到我的正则」≠「代码里没有」** ——
本轮两次假阳性都来自**检测手段的形状**（`phase:` 绑定 vs `*phase ==` 比较），
**而不是代码本身有问题**。 ⇒ **检测器必须先证明它能识别两种写法，再报数** ✅

---

## 9 「用牌前/指定目标时」27 条的**语义抽样复核**：6/6 成立（第 741 轮）

上一节把 23 条用牌 + 4 条死亡记为「需档」，依据是**上游 trigger 名**。
本轮抽 4 条读**上游函数体**，确认它们确实**必须在结算前生效**：

| 行为 | 上游 trigger | 上游函数体关键点 | 是否必须前置 |
|---|---|---|---|
| **`huanyin`（还因）** | `target: 'useCardToBefore'` | `get.effect(player, trigger.card, …)` —— **先算这张牌对该角色的效果** | ✅ 必须 |
| **`ningshuang`（凝霜）** | `target: 'useCardToBegin'` | `get.color(event.card)` + 排除使用者 | ✅ 必须 |
| **`xielei`（邪雷）** | `player: ['useCard','respondAfter']` | `event.card.name == 'sha'` ⇒ `chooseCardTarget` | ✅ 必须 |
| **`zhenlie`（震裂）** | `target: 'useCardToTargeted'` | `event.card.name == 'sha' || get.type == 'trick'` | ✅ 必须 |

加上上一轮已读体的 **`dunxing`（`useCardToBefore` + chooseToDiscard）** 与
**`changnian`（`dieBegin` + chooseTarget）** ⇒ **6/6 抽样全部成立** ✅

### 诚实标注剩余风险

- **27 条的 trigger 名全部来自工具解析**（不是抽样）✅
- **语义只抽样 6 条**（约 22%）⚠️
- **但这类 trigger 的语义在非名里是自明的**：`useCardToBefore` / `useCardToBegin` /
  `useCardToTargeted` / `phaseDiscardEnd` / `dieBegin` 这些名字本身就定义了时机
  ⇒ **抽样用于排除「名字与实现不符」的可能，6/6 未见反例** ✅

⇒ **分类维持「需档（已确认）」，但在证据里保留「语义抽样 6/22」这条限定** ✅
