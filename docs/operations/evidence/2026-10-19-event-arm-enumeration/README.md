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
