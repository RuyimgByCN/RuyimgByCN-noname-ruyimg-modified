# 施工单（Work Order）—— 75 条缺陷按「一次机械改动」分组

> **这份文件写给「有写权限的人」**。每条都给出：改哪个文件、改什么、改完哪个 `#[ignore]` 守卫会转绿。
> 源数据：[ledger.tsv](ledger.tsv)（10 列，含 `location` = `file:line`）。

## 批次 0：3 个枚举变体（**覆盖 39 条 = 52%**）

| 批 | 改动 | 覆盖 | 前置 |
|---|---|---:|---|
| **0-A** | `EventKind::CardTargeted`（用牌前/指定目标时）| **23 条** | `kernel` 发射点 |
| **0-B** | `PhaseTiming{Before,Begin,After,End}` + `Phase::Prepare` | **9 条**（G20 5 + 准备阶段 2 + G15 2）| 同上 |
| **0-C** | `EventKind::DeathBefore`（与 `DamageBefore` 对称）| **4 条** | 同上（**现成范式，最省力**）|

⇒ **3 个变体 → 39 条**。这是全表 ROI 最高的一批 ✅

## 批次 1：G14 —— **14 条，同一处改法**

**改法**：把 filter 里的 `phase: Phase::Judge, begin: true` 改成 **`begin: true, ..`**（不钉阶段）——上游 `phaseBegin` 本就是「任意阶段」。

`ledger.tsv` 里 `id == G14` 的 14 行，`location` 列即 `file:line`。
**改完转绿**：`g14_*_any_phase`（14 个守卫）。

⚠ **注意**：`SwdHujing` / `Qinglongzhigui` / `Huanglinzhicong` 三条是**相等比较写法**
（`*phase == Phase::Judge`），改法相同但要改的是比较式。

## 批次 2：G23 —— **12 条，差异在「该钉到哪个阶段」**

| 子类 | 条数 | 改法 |
|---|---:|---|
| **未钉阶段**（`Phase { begin: false, .. }`）| **10** | 加 `phase: Phase::End` |
| **事件种类错**（`SwdJiguanyuanGain`）| **1** | meta 从 `PhaseEnd` 改 `Damage` + 同步 filter |
| **触发失效**（`SwdBaishouzhihuHeld`）| **1** | filter 的 `*begin` 改 `!*begin` |

**改完转绿**：`g23_*_turn_end` + `jiguanyuan_gain_must_trigger_on_damage`（11 个守卫）

## 批次 3：A 类零散 —— **17 条，每条一处**

含 `5a`（`kuwu` 少 2 条件）、`5b`（`reguhuo` 标记）、`6`（`rangjie` 分支 1 摸牌）、
`G19`×3（删 `!= owner`）、`G16`（`GwentCiguState` 挪到 `Draw, begin: true`）、
`凭空`（`baiyi` 删阶段触发）、`自加`（`huajin2` 删阶段分支）、`失效` 等。

每条的 `fix` 列已写成一句话，`location` 列给出文件。

## 批次 4：需档 27 条 —— **依赖批次 0-A / 0-C**

- **23 条**等 `CardTargeted`（批次 0-A）落地后，把 filter 的 `EventKind::CardUsed` 指向新事件；
- **4 条**等 `DeathBefore`（批次 0-C）落地后，把 `EventKind::Death` 指向它。

⇒ **批次 0 不做，批次 4 无法开始** ⚠️

## 总量核对

| 批次 | 覆盖 | 可否立即开始 |
|---|---:|---|
| **0（枚举）** | **39** | ❌ 需 `kernel` 权限 |
| **1（G14）** | **14** | ❌ 需 `content/host` 权限 |
| **2（G23）** | **12** | ❌ 同上 |
| **3（A 零散）** | **17** | ❌ 同上 |
| **4（需档）** | **27**（与 0 重叠计）| ❌ 依赖 0 |
| **合计** | **75（去重叠）** | **全部 blocked** ⚠️ |

## 本会话能做的与不能做的

| 能做 | 已做 |
|---|---|
| 定位每一条（`file:line`）| ✅ `ledger.tsv` |
| 写出每条修法 | ✅ `fix` 列 |
| 给出可执行的验收条件 | ✅ **57 个 `#[ignore]` 守卫**，改完去掉 ignore 即验证 |
| **实际修改** | ❌ **修复点全在未跟踪的 `content/host/**` 与 `kernel/**`** |

---

## 终版（第 800 轮）：**72 条缺陷 = 72 处小改动，无一处需要新写效果逻辑**

**实测**：把 72 行的行为逐个查效果步（`step*.rs` / `step/*.rs`）——

| 结果 | 条数 |
|---|---:|
| **效果步有真实现** | **72 / 72（100%）** |
| 效果步缺失或平凡 | **0** |

⇒ **整个缺陷清单里没有一条是「功能没做」**，全部是**触发接线 / 时机 / 过滤条件**问题 ✅

### 按修法分批（每批的改动形态固定）

| 批次 | 内容 | 条数 | 改动形态 | 涉及文件 |
|---|---|---:|---|---|
| **0** | **加 3 个 `EventKind` 槽位**（`CardTargeted`、`ShaBegin/After/Before`、`DeathBefore`）| 覆盖 **21** | 枚举 + meta 改一行/行为 | `event.rs` + `metadata*.rs` |
| **1** | **`begin: true, ..`**（G14：钉 `Judge` 改不钉）| **14** | 每行 1 处 pattern | `filtering/*.rs` |
| **2** | **钉 `End`**（G23：`PhaseEnd` 未钉阶段）| **12** | 每行 1 处 pattern | `filtering/*.rs` |
| **3** | **`begin: false`**（G20：`phaseEnd` 写成 `begin: true`）| **5** | 每行 1 处 | `filtering/*.rs` |
| **4** | **删 `!= owner`**（G19）| **3** | 每行 1 条件 | `filtering/swd.rs` |
| **5** | **改事件种类**（G23b）| **2** | 每行 1 处 | `filtering/*.rs` |
| **6** | **G18 换阶段**（`Prepare`）| **2** | 每行 1 处 + 1 个 `Phase` 变体 | `filtering/*.rs` + `event.rs` |
| **7** | **单条**（5a/5b/5c/6/G15/L03c/L20a/b/U08c/凭空/第3次/G22）| **13** | 各异，均已写明 | 见台账 `fix` 列 |

⇒ **三个枚举槽位覆盖 21 + 2 + 1 = 24 条（33%）**，其余 **48 条是纯 pattern/条件改动** ✅

### 为什么可以这样断言

**每一行都做了三件事**（本会话累计）：

1. **上游定位**：85 → **72 行全部 100% 定位**（第 790 轮归零）
2. **上游同质**：相位族 **36 行族级同质**（第 791 轮）
3. **原生效果步存在**：**72/72**（本轮）

⇒ **剩下的未知只有「效果逻辑是否正确」**（第 799 轮已注明边界），
**而那一项与「触发时机错」是两个独立的验收维度** ⚠️
