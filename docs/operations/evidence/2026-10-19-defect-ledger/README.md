# 缺陷台账（机器可读，**85 条**）—— 2026-10-19 第 788 轮

> **这是唯一权威的缺陷计数**。此前的「42 条」「93 条」都是**会话中的增量记账**；
> 本台账是**脚本生成 + 去重**的结果：`scripts/report_defect_ledger.py --write`。

## 1 计

| 类 | 条数 |
|---|---:|
| **A（改 pattern / 删分支 / 补动作，不动枚举）** | **41** |
| **B（需扩枚举或引擎能力）** | **44** |
| **合计** | **85** |

## 2 按缺陷族

| id | 条数 | 说明 |
|---|---:|---|
| **需档** | **33** | 「用牌前/指定目标时」23 + **「sha 幕」6** + 「死亡前」4 |
| **G14** | **14** | 上游 `phaseBegin`（任意阶段）却钉死 `Judge`/`End` |
| **G23** | **12** | `PhaseEnd` 未钉阶段 ⇒ 每阶段触发（含 1 条事件种类错 `G23b`）|
| **G20** | **5** | 上游 `phaseEnd` 写成 `End, begin: true`（早一步）|
| **G19** | **3** | `!= owner` 把技能主排除 |
| **G15** | **2** | 「之前」档 / `phaseAfter` 档缺失 |
| **G18** | **2** | 准备阶段缺档（`HlssFengqi` / `Xiuluo`）|
| **G22** | **1** | `GwentCiguState` 晚一步（摸牌数已定，效果失效）|
| 其余单条 | **12** | `5a`/`5b`/`5c`/`6`/`G16`/`G23b`/`凭空`/`自加`/`失效`/`第3次`/`B`/`G21 残项` |

## 3 与「93 条」的差额（**必须解释，否则两个数字会一直打架**）

**93 是增量记账，重复计了「同一类的增长」**：

- `G14` 从 9 长到 14，增量法记了「+5」；但**旧的 42 里已经含 9 条** ⇒ **重复 5 条**；
- `G23` 同理（先记「候选 17」，后记「确认 11」，增量把两者都加了）⇒ **重复 11 条**；
- 另有 `SwdJiguanyuanGain` 在「G23」与「事件种类错」两处各记一次 ⇒ **重复 1 条**。

**去重后 = 75**（第 747 轮进一步并入 `G18`×2 与 `G22`，并对齐枚举的「可接受」判定 ⇒ **77**）✅

⇒ **这就是「增量记账不可靠」的第三个实例**（前两个：缺陷 17 vs 25、G14 9 vs 14）。
⇒ **从此以后，缺陷数只从本台账读**，不再在正文里累加 ✅

## 4 列含义

| 列 | 含义 |
|---|---|
| `id` | 缺陷族编号（`G14`/`G20`/`G23`/`需档`/`5b`…）|
| `class` | **A** = 改 pattern/删分支；**B** = 需扩枚举 |
| `behavior` | 原生 `HostBehavior` 名（可直接 grep）|
| `upstream` | 上游 trigger 摘要 |
| `native` | 原生当前形态 |
| `severity` | **高** = 触发次数/事件种类错；**中** = 时机偏早或相邻 |
| `fix` | 修法一句话 |
| `permission` | **全部 `blocked`** —— 修复点都在未跟踪的 `content/host/**` / `kernel/**` |

## 5 复现与校验

```bash
python3 -B scripts/report_defect_ledger.py --write docs/operations/evidence/2026-10-19-defect-ledger/ledger.tsv
python3 -B -m unittest discover -s scripts -p 'test_report_defect_ledger.py'
```

**7 个单测**（含「不得出现 G21 假阳性」与「不得有重复 (id, behavior)」两条回归护栏）✅

---

## 6 按来源归因（第 743–744 轮）

`report_defect_ledger.py --by-source` 尝试把每条缺陷归到「37 个来源」之一（经 `host:` 键 → 包技能表 → 来源映射）：

| 来源 | 条数 | 缺陷族 |
|---|---:|---|
| **`?`（无法归因）** | **58** | 5a, 6, B, G14, G15, G16, G19, G20, G23, G23b, 凭空, 失效, 第3次, 需档 |
| **U05** 杀海拾遗 | 10 | G14, G20, G23, 自加, 需档 |
| U07 玩点论杀 | 2 | G14, G20 |
| L06 wuxing | 1 | 5c |
| L10 史诗卡牌 | 1 | 5b |
| L27 选将美化 | 1 | 需档 |
| U06 欢乐卡牌 | 1 | G20 |
| U08 英雄杀 | 1 | G23 |

### 第 744 轮改进：加 `source` 列，改用**证据型归因**

归因不再靠 `host:` 键猜，而是**按缺陷触发点所在的上游文件路径**匹配各包的
`upstream_path` 字段（包数据自带，属证据）：

| 来源 | 条数 | 缺陷族 |
|---|---:|---|
| **`?`（仍未归因）** | **55** | B, G14, G15, G20, G23, 第3次, 自加分支, 需档 |
| **U05** 杀海拾遗 | **12** | G14, G16, G19, G23, G23b, 失效, 自加, 需档 |
| U06 欢乐卡牌 | 3 | 5a, G20, 需档 |
| L10 史诗卡牌 | 2 | 5b, 6 |
| L03 boss | 1 | 凭空 |
| L06 wuxing | 1 | 5c |
| L27 选将美化 | 1 | 需档 |

⇒ **归因数 17 → 20**（`U06` 2→3、`L10` 1→2、新增 `L03`）✅

### ❌ 我上一轮怀疑「`WudianJidong` 归因错了」——**是我错了，解析器是对的**（第 745 轮更正）

第 744 轮我写道：「`jidong` 在 `character/ow.js` 与 `wudian` 包里都存在……`WudianJidong` 从命名看应属玩点论杀（U07）⇒ 归因可能是错的」。

**核查结果**：

| 检查 | 结果 |
|---|---|
| `packages/extensions/wudian-upstream-noname` 的技能表 | **没有 `jidong`**（其技能为 `wd_wangmeizhike`/`ciqiu`/`shuaiyan`/`zhucheng`…）|
| `packages/extensions/shaai-ow-upstream-noname` | **`upstream_path = extension/杀海拾遗/character/ow/`** |
| 上游 `jidong` 定义位置 | **`noname-liuli/noname/character/ow.js`** |

⇒ **`wudian_jidong` 这个 `host:` 键产生的行为，其上游确实在「杀海拾遗 / ow」里** ✅
⇒ **我拿「`Wudian` 前缀 ⇒ 玩点论杀」去质疑解析器，而前缀本来就不是来源标签** ❌

**这是本会话「前缀/命名 ≠ 归属」的第 3 次教训**（前两次：`Swd` 前缀跨多个子包、`_xigua_*_` 下划线名）。

### ⇒ 修正后的归因口径

| 结论 | 状态 |
|---|---|
| **`source` 列（路径型）** | ✅ **可信度高于前缀猜测** —— 它来自「上游定义在哪个文件」+「包的 `upstream_path`」 |
| **仍标 `?` 的 55 条** | ⚠️ **不是「没有来源」，而是「解析器找不到」**（多为内部子行为，上游措辞与包数据都对不上）|
| **要断言某来源缺陷数** | ⚠️ **仍须人工**（本会话已人工核过 `U05`/`U07`/`L06`/`L10`/`L19`/`U01` 等）|

### 其余 55 条归不到来源的原因

**大部分缺陷行为是「内部子行为」**（如 `SwdHujing`、`SwdLingxin`、`WudianJidong`、`XianjianBinxin`），
它们**不在任何包的 `skills[].mapping` 里**（包里只登记对外技能名），所以：
`behavior → host:键` 这一步就断了 ⇒ **归因链断在第一步** ⚠️

### 这条发现的意义

1. **「按来源验收」不能只靠台账**：**77% 的缺陷没有来源标签** ⇒
   **要判某个来源是否可签，仍必须像 `L19` 那样逐来源人工对照** ✅（**本会话已建立该方法的模板**）；
2. **反过来，这条也是「来源覆盖」的真实瓶颈**：**缺陷数 ≠ 来源可签性**，
   一个来源可能有 10 条缺陷（`U05`），也可能 0 条（`L19`）✅

### 未做的（如实登记）

- **没有**用「行为名前缀（`Swd*`→U05、`Wudian*`→U07）猜来源** ⚠️ ——
  **前缀不等于来源**（`Swd` 既出现在 U05 的子包，也出现在跨来源共享行为里），
  **猜错会让「按来源验收」产生假数字**，而本会话已因「猜」误判过多次 ❌
- **要真正修好**，需要**包数据的 `skills[]` 补齐内部子技能登记**，或在
  `HostBehavior` 上加来源字段 —— **属数据结构决策**，需维护者定 ⚠️

---

## 7 第 748 轮：新增「sha 幕」缺档类（6 条）并修掉一个归因隐患

### 7.1 新缺档：`shaBegin` / `shaAfter` / `shaBefore`

上游有一整套**【杀】专用事件幕**，引擎只有 `ShaDodged` 一种：

| 行为 | 上游 |
|---|---|
| `Liaoyuan` | `player: 'shaBegin'` |
| `SwdMojian` | `player: 'shaBegin'` |
| `SwdTiebi` | `global: 'shaBegin'` |
| `SwdLianda` | `player: 'shaAfter'` |
| `XianjianXunying` | `player: 'shaAfter'` |
| `WudianZhuchengGuard` | `target: 'shaBefore'` |

⇒ **6 条归入「需档(sha 幕)」**（B 类）✅
⇒ **这是第 8 类缺档**（前七类：阶段前后 / 准备阶段 / 用牌前 / 死亡前 / `phaseAfter` / `GameStart` / `damageEnd` 后窗口）

### 7.2 ⚠ 修掉一个归因隐患：删掉「键尾段」回退

`upstream_trigger()` 原本在包内技能名查不到时，会退回试 **`host:` 键的最后一段**。
这个回退曾把 `shengke_check` 解析到**另一扩展的同名技能**，得出 `loseAfter` 这种错误 trigger
（而 `5c` 的真实上游是 wuxing 扩展的 `_shengke: { target: 'useCardToBegin' }`）❌

⇒ **已删掉该回退**：只试「包内技能名」与「host 键本身」✅
⇒ **代价**：REVIEW 会略增（宁可标「未定位」，也不要标错）✅

---

## 8 第 749 轮：新增 `G24` —— **濒死自救主动技被挂成 `Death` 触发**（4 条）

### 8.1 证据（读了两条上游函数体）

| 技能 | 上游形态 | 关键点 |
|---|---|---|
| **`niepan`（涅槃）** | `enable: 'chooseToUse'`, `limited`, `storage.niepan`, `filter(event, player)` | **主动技**，`filter` 要求 `event.type` 为濒死 |
| **`shouyin`（收印）** | `enable: 'chooseToUse'`, `skillAnimation`, `filter` | **`filter: if (event.type != 'dying') return false;`** ⇒ **只在濒死时可用** |

⇒ **两者都是「濒死时主动发动」**，**不是「死亡后触发」** ✅

### 8.2 原生的问题

原生 meta 是 **`EventKind::Death`** ⇒ **死亡已经结算完成后才触发** ⇒
**这类「自救」效果永远来不及** ❗（等同失效）

### 8.3 影响 4 条

`Niepan` / `SwdShouyin` / `XianjianChengxin` / `XianjianShuiyun5`
（**已读体 2 条，另 2 条按 `enable` 形态推定** ⚠️ —— 保留这条限定）

**修法**：meta 从 `Death` 改成 **`Dying`**（`EventKind::Dying` 已存在）⇒ **A 类**

---

## 9 第 750 轮：`L03` boss 逐个对照，+3 条 B 类（台账 87 → 90）

| id | 行为 | 上游 | 原生 | 缺的档位 |
|---|---|---|---|---|
| **`L03a`** | `HlssXiangxing` | `player: [damageEnd, loseHpEnd]` | meta 仅 `Damage` | **「失去体力后」** |
| **`L03b`** | `HlssYueyin` | `player: [damageBegin4, damage]` | meta 仅 `DamageBefore` | **「第 4 次」+ 伤害后双幕** |
| **`L03c`** | `HlssGaiming` | `player: judgeBefore` | **无 TriggerMeta** | **「判定前」** |

⇒ **`L03` = 6 技能 / 4 缺陷 / 2 一致**，**不可签** ❌
⇒ 与 [`L19`（0 缺陷）](../../../architecture/native-rules-baseline.md) 对照：**小来源也能一条不错，问题在实现不在规模** ✅

### 顺带更正一个口径

**`L03` 此前按「1 技能 + 1 武将」计为 2 项**，实际武将 `boss_zhugeliang` **自带 5 个技能**
⇒ **`L03` 的真实内容是 6 个技能** ⚠️ —— 这也是「按包内 `skills[]` 计」与「按武将挂载计」的差别。


---

## 10 ❌ 撤回：第 763–764 轮并入的 18 条（台账 120 → **102**）

**两条工具/转录错误**导致 18 条假缺陷：

1. **`metadata_events()` 只匹配字面 `Some(TriggerMeta {`**，漏掉 **`self.trigger_meta_*()` helper** 形态
   ⇒ `U05` 的「98 条无 meta」实际是 **0 条**（239 键全部有 metadata arm）；
2. **候选清单里丢了 `HostBehavior` 变体前缀**（`SwdHuanxia` → `Huanxia`）
   ⇒ 后续 `HostBehavior::X` 检索全部落空 ⇒ 「15 条四路全空」是**查错了名字**。

**撤回 18 条**（15 inert + 3 no-meta）⇒ **台账回到 102（A 50 / B 52）** ✅

---

## 11 第 769 轮：新增**台账一致性校验**（`check_ledger_consistency.py`）

台账由三处合并而来（相位枚举 / 事件枚举 / 手工表），三处都被改过多次，**行与来源可能漂移**。
新脚本做交叉校验：

| 台账 id 属于 | 校验方式 |
|---|---|
| **相位判定**（`G14`/`G18`/`G20`/`G22`/`G15`/凭空/自加分支）| 必须出现在 `report_phase_arms.py` 输出，且**判定一致** |
| **事件判定**（`需档`/`第3次`/`G24`）| 必须出现在 `report_event_arms.py` 三个族的输出，且判定一致 |
| **`G23`** | 必须出现在 `g23_rows()`（`PhaseEnd` 未钉阶段普查）里 |
| **手写** | 豁免（无枚举可比） |

### 首跑就抓到 1 处重复

`SwdHuajin2` 同时以 **`自加`（手工行）** 与 **`自加分支`（枚举）** 记了两次 ⇒ **删掉手工行** ✅
⇒ **台账 102 → 101（A 49 / B 52）**

### 现状

```text
台账 101 行；相位枚举 53 行；事件枚举 157 行
台账与枚举一致 ✅
```

**`--strict` 已进单测**（4 个用例），任一处不一致即失败 ✅

---

## 12 第 770 轮：**守卫覆盖率 38/101 = 37.6%**（`report_guard_coverage.py`）

台账每条缺陷都应有可执行的 `#[ignore]` 守卫（改完去掉 ignore 即验收）。
新脚本把守卫测试里的行为名扫出来与台账比对：

| 指标 | 值 |
|---|---:|
| 台账缺陷 | **101** |
| **有守卫** | **38（37.6%）** |
| **缺守卫** | **63** |

**缺守卫的分布**（见脚本 stderr）：

| id | 条数 |
|---|---:|
| **需档** | **32** |
| G24（濒死主动技） | 4 |
| G15 / G18 / G23 | 各 2 |
| G22 / 第3次 | 各 1 |

### 两个测量陷阱（本轮踩到）

1. **宏守卫把变体名当字符串传参**（`g14_guard!(…, "SwdHuanxing", …)`）
   ⇒ 只扫 `HostBehavior::X` 会**漏掉全部宏生成的守卫** ⇒ **补扫双引号 CamelCase** ✅
2. **「后缀放宽」会虚增覆盖**：加 `endswith` 匹配后覆盖率从 37.6% 涨到 55%
   ⇒ **是假象**（`SwdHuanxing` 会被 `Huanxing` 之类误配）⇒ **改回精确匹配** ✅

⇒ **真实覆盖率 37.6%**，**63 条待补** —— 这是本会话留给下一位的**最明确的待办**之一 ⚠️

---

## 13 第 771 轮：守卫覆盖率 **88/91 = 96.7%**；并撤回 9 条「无 meta」假缺陷（101 → 91）

### 13.1 新增 60 个 `#[ignore]` 守卫（全部实测为红）

| 守卫族 | 数量 | 断言 |
|---|---:|---|
| **能力守卫**（`capability_guard!`）| 8 | `event.rs` 里是否出现 `CardTargeted`/`ShaBegin`/`ShaAfter`/`ShaBefore`/`DeathBefore`/`GameStart`/`JudgeBefore`/`DamageBegin2` |
| **无 meta 行为**（`meta_event_guard!`）| 7 | 行为 arm 附近的期望事件名 |
| **能力/幕位**（手写）| 10 | `Prepare`/`LoseHpEnd`/`DamageBegin4`/`JudgeBefore`/`CardTargeted`/`DrawBegin2`/`dieBegin`/`EnterGame` 等 |
| **分支动作** | 1 | `rangjie` 分支 1 必须有 `draw` |

**`cargo test … -- --ignored` 结果：60 failed / 0 passed** ⇒ **每个守卫都是真反例** ✅

### 13.2 ❌ 撤回 9 条「无 meta」缺陷（与第 765 轮同一根因）

`no_meta_arm_guard!` 断言「`metadata*.rs` 里有该行为的 arm」——**9 个守卫直接通过**，
说明这些行为**确实有 arm**，只是此前 `metadata_events()` 只认字面量而漏判：

`L20c` `ZhuWanzhongJi`、`L20d` `FzLiegong`、`L27a` `Xiefang`、`L27b` `SbZhaXiang`、
`L29a` `MbQianlong`、`L29b` `MbWeitong`、`L07a` `WudianCiqiu`、`U08a` `YxsFanji2`、`U08b` `YxsMenshen2`

⇒ **9 条全部撤回**（**没有足够证据说它们缺 meta**）⇒ **台账 101 → 92**，再删 `Tianxiang` 重复行 ⇒ **91（A 46 / B 45）** ✅
⇒ **教训**：**守卫若一开始就是绿的，它守的不是缺陷，而是我的误判** ⚠️

### 13.3 剩余 3 条无守卫

| 行 | 理由 |
|---|---|
| `G23 Hutian`、`G23 SwdMufengHeld` | **偶合正确**（未钉阶段但本地只在该帧触发）⇒ **无需守卫** |
| `失效 SwdBaishouzhihuHeld` | **语义缺口**，需产品判定后才能写断言（D 类）|

---

## 14 第 781 轮：❌ 撤回 G24 四行（91 → 87）——规则缺了「原生」前提

对 91 行做**逐族抽查 6 行**（G14/G18/G19/G20/G22/G23/G23b/G24），**6 行里 5 行准确、1 族全错**：

| 行 | 代码实际 | 台账判定 | 结论 |
|---|---|---|---|
| `HlssFengqi` | `Phase::Judge, begin: true` | **G18**（应 `phaseZhunbeiBegin`）| ✅ 准确 |
| `Hutian` | `Phase { begin: false }` 未钉阶段 | **G23**（5×/回合）| ✅ 准确 |
| `SwdJiguanyuanGain` | `Phase { begin: false }` | **G23b**（上游 `damageEnd`）| ✅ 准确 |
| `GwentCiguState` | `Phase::Draw, begin: false` | **G22**（晚了）| ✅ 准确 |
| `WudianZhucheng` | `Phase::End, begin: true` | **G20**（早了）| ✅ 准确 |
| **`Niepan` 等 4 条** | **`EventKind::Dying`** | **G24（挂成 Death）** | ❌ **全错** |

### 错因：规则只看上游，不看原生

```python
if trigger.startswith("(enable"):        # 上游是主动技
    return "G24(濒死主动技挂成 Death)"    # ← 直接判缺陷，没检查原生事件
```

**上游是「濒死主动技」只说明它需要 `Dying`，不代表原生没有 `Dying`。**
实际 4 条全部是 `event: EventKind::Dying` + `filter` 要求 `Dying` ⇒ **完全正确**。

**修正后规则**：

```python
if trigger.startswith("(enable"):
    if which == "Dying": return "OK"
    if which == "Death": return "G24(...)"
```

⇒ **`death` 族从 10 OK / 4 G24 变成 14 OK / 0 G24** ✅
⇒ **台账 91 → 87（A 42 / B 45）**

### 教训（第 8 次自我更正）

**判定规则里的每一项都必须有「原生侧」的对应检查。** 只看上游就给结论，
等价于「假定原生一定是错的」—— 这是**反向的确认偏误** ⚠️

---

## 15 缺陷族的**代码级抽查**：哪些行是「读代码确认」的（第 782 轮）

台账的 87 行来自机器扫描 + 手工表。**机器扫描可能误判**（本会话已发生 8 次），
所以下面单独记「**已读代码逐条确认**」与「**仅有机器判定**」的比例。

### 已读代码确认的族（共 **30 行**）

| 族 | 行数 | 抽查方式 | 结果 |
|---|---:|---|---|
| **G14** | 14 | 读 `filter_common` 的 `matches!`（如 `HlssFengqi`）| ✅ 钉 `Judge` 阶段属实 |
| **G18** | 2 | 同上（`HlssFengqi`/`Xiuluo`）| ✅ 上游 `phaseZhunbeiBegin`，原生钉 `Judge` |
| **G19** | 3 | 读 `filtering/swd.rs`（`SwdRexue`/`Zhuhai`/`Tiebi`）| ✅ 三处都真有 `!= owner` |
| **G20** | 5 | 读 `WudianZhucheng` | ✅ `Phase::End, begin: true`（早一步）|
| **G22** | 1 | 读 `GwentCiguState` | ✅ `Phase::Draw, begin: false`（晚一步）|
| **G23** | 12 | 读 `Hutian` | ✅ `Phase { begin: false }` 未钉阶段 |
| **G23b** | 1 | 读 `SwdJiguanyuanGain` | ✅ 上游 `damageEnd` 却挂 `Phase` |
| **G24** | ~~4~~ → **0** | 读 4 条 `TriggerMeta` | ❌ **全部撤回**（都是 `Dying`，正确）|

### 仅机器判定、尚未逐条读代码（**约 57 行**）

| 族 | 行数 | 为什么还没读 |
|---|---:|---|
| **需档（用牌前/指定目标）** | 23 | 需要新增枚举才能验证「缺失」，属**能力缺口**，机器判定已足够 |
| **需档（sha 幕）** | 6 | 同上 |
| **需档（死亡前）** | 4 | 同上 |
| **5a/5b/5c/6** | 4 | 各自单条，需逐条读（`5b` 已读过 `activation.rs`）|
| **自加分支 / 凭空新增 / 失效** | 3 | 需逐条读 |
| **G15** | 2 | 需读「之前」档 |
| **L03/L20/L27/L29/U05/U07/U08 系列** | 15 | 逐来源普查得出，**触发面对照**已做，**行为面未逐条读** |

### ⇒ 台账可信度的分层说法

* **30 行（34%）**：**读过代码、逐族抽查确认** ✅
* **57 行（66%）**：**机器判定**（其中 33 行是「缺能力」类，机器判定已足够）⚠️

⇒ **不能笼统说「87 条都确认过」**；正确的说法是**「30 条已读码确认 + 33 条能力缺口 + 24 条待读码」** ⚠️

---

## 16 第 783 轮：9 行逐条读码，1 行撤回（`L03a`）、1 行子标签修正（`G15`）

| 行 | 代码实际 | 台账 | 结论 |
|---|---|---|---|
| `XianjianFenglue` | `Phase::Play, begin: true` | G15（上游 `phaseUseBefore`）| 准确 |
| `SwdHuajin2` | `Damage` 或 `Phase{…}` 双分支 | 自加分支 | 准确 |
| `BossBaiyi` | `Phase{…}` + `boss_stage` | 凭空新增触发 | 准确 |
| `SwdBaishouzhihuHeld` | `Phase { begin: true }` 未钉阶段 | 失效 | 准确 |
| `ShengkeCheck` | `CardUsed(u) if u.targets.contains(owner)` | 5c | 准确 |
| `HlssYueyin` | 仅 `Damage` | L03b | 准确 |
| `HlssXiangxing` | `Damage` 与 `LoseHp` 双分支 | L03a | 撤回（两个幕都已覆盖）|
| `WudianFuchou2` | `Phase::End, begin: false` | G15 | 分类对、子标签应为 `phaseAfter` |
| `HuanleKuwu` | 过滤臂无 `matches!` | 5a | 待查 |

### 撤回 `L03a` 的原因

`HlssXiangxing` 的过滤臂是 `match` 双分支，`EventData::Damage` 与 `EventData::LoseHp` 都在，
上游 `['damageEnd','loseHpEnd']` 两个幕全覆盖。我此前只看 `metadata` 登记了 `EventKind::Damage`
就判「缺 `loseHpEnd`」——**又是一次「只看一处代表全部」**（第 9 次自我更正）。

**台账 87 → 86（A 42 / B 44）**

---

## 17 第 784 轮：一致性检查新增「行为名必须存在」——当场抓出 3 个错名

台账的 `behavior` 列曾出现**源码里根本不存在的变体名**，且**没有任何门禁能发现**：

| 台账写法 | 实际 | 后果 |
|---|---|---|
| `HuanleKuwu` | **`ShaDamageForceDiscard`**（U06 的 `huanle_kuwu` 映射到**共享行为**）| 5a 的缺陷描述指向了不存在的对象 |
| `GujianFoodRangjie` | **`Rangjie`** | 6 号行同上 |
| `Huanxia`（已撤回的 15 行里）| **`SwdHuanxia`** | 15 条假缺陷的根因之一 |

⇒ **新增检查**：`check_ledger_consistency.py` 现在要求**每行的 `behavior` 出现在 `keys.rs` 的 `HostBehavior` 列表里**
（用枚举全集而不是 `host:` 键映射 —— `SwdQinglongzhiguiHeld`、`SwdBaishouzhihuHeld` 这类卡片全局技能**没有 `host:` 键**）✅

### 同时修掉的

* `5a` 改为 `ShaDamageForceDiscard`，并注明**这是共享行为** ⇒ 缺陷描述改为「共享行为未含苦无的 2 个条件（**待复核**）」；
* `6` 改为 `Rangjie`。

### 这条检查的价值

**行为名写错时，之前的检查全都通过**（因为它只比对判定字符串，不校验对象是否存在）。
现在这类错误会在提交前失败。

⇒ **台账 86 行（A 42 / B 44）不变**（只改名字，未改计数）✅

---

## 18 第 785 轮：守卫也查「名字存在」——`guard_rangjie_branch1_draws` 引用了不存在的变体

新增 [`check_guard_references.py`](../../../../scripts/check_guard_references.py)（已进 `preflight`）：
解析守卫文件里的 `HostBehavior::X` / `EventKind::X` / `Phase::X` 与双引号 CamelCase，逐个核对是否存在于
`keys.rs`、`event.rs`，或属于**待补能力白名单**。

**首跑抓出 3 个名字**：

| 名字 | 判定 |
|---|---|
| `Damage4` | ✅ 待补能力（与 `DamageBegin4` 同义的候选名）⇒ 进白名单 |
| `EPIC_REGUHUO_KEY` | ✅ `constants.rs` 里真有的**常量**⇒ 进 `ALLOWED_CONSTANTS` |
| **`GujianFoodRangjie`** | ❌ **不存在的变体** ⇒ 守卫的 `src.find(...)` **永远 panic**（`expect("找不到 rangjie 的效果步")`）|

⇒ **`guard_rangjie_branch1_draws` 已改为 `Rangjie`** ✅
⇒ **重跑 `--ignored`：60 failed / 0 passed**（**改名后仍为真反例**）✓

### 为什么这类错误特别隐蔽

守卫用 `expect("…")` 取字符串位置：**名字错了不会编译失败，只在 `--ignored` 跑时才 panic**，
而 `#[ignore]` 的守卫**平时根本不跑** ⇒ 一个永远 panic 的守卫可以存活很久，还给人「有覆盖」的错觉 ⚠️

⇒ **现在它是一条提交前门禁**（`preflight` 里的「守卫引用名存在」）✅

---

## 19 第 786 轮：**22 处文档 `file:line` 引用已失效**（源码拆分导致）

新增 [`check_doc_line_refs.py`](../../../../scripts/check_doc_line_refs.py)：
扫 `docs/**/*.md` 里的 `path.rs:123` 引用，核对文件存在且**引用行号不超过文件行数**。

| 结果 | 数量 |
|---|---:|
| 检查的 `file:line` 引用 | 数百处 |
| **行号越界（失效）** | **22 处** |
| 找不到的文件 | 5 类（多为上游快照路径或示例）|

**失效样例**：

| 文档 | 引用 | 实际 |
|---|---|---|
| `native-coverage-manifest.md` | `kernel.rs`、`kernel.rs` | 现存 `kernel.rs` **只有 77 行**（已拆分）|
| `adr/0599` | `pack_tw_1.rs` | 现存文件 **14 行** |
| `adr/0623` | `play.rs` | 现存 **136 行** |
| `adr/0643` | `content/registry.rs` | 现存 **632 行** |

⇒ **成因**：本会话期间 `kernel/`、`content/host/` 被大幅拆分，**行号引用随之失效** ⚠️
⇒ **处理**：检查以**报告模式**接入 `preflight`（不阻塞），**22 处待清理**；`--strict` 会失败，可用于清理进度跟踪 ✅

### 这条检查的边界

它**只能验证「行号在文件范围内」**，不能验证「那一行还是文档说的那段代码」——
后者需要语义匹配，**假装能查比不查更糟**。所以脚本注释里写明了这条限制 ⚠️

### 处理结果（第 787 轮）：**103 处失效行号已清除**

初测报 22 处，实际逐条扫描 **103 处**（此前统计被 `head` 截断）。

**处理方式**：**只去掉失效行号、保留文件名**（`play.rs` 的旧行号 → 只保留 `play.rs`），
不猜新行号 —— 因为源码拆分后行号对应的是**新的位置**，猜一个数字等于制造新的假精度 ⚠️

**清理后**：`check_doc_line_refs.py --strict` **exit 0**，已把 `preflight` 里该检查**从报告改为严格** ✅

---

## 20 第 788 轮：逐条读 `huajin` 家族 ⇒ 撤回「自加分支」（86 → 85）

**上游 `huajin` 是三个技能的组合**：

| 技能 | 触发 | 效果 |
|---|---|---|
| `huajin` | `enable: phaseUse, usable:1` | 弃一张 `he`，**授予 `huajin2`** |
| `huajin2` | `trigger: {source: 'damageBegin'}` | `trigger.num++`（伤害 +1）|
| **`huajin3`** | **`trigger: {player: 'phaseBegin'}`** | **移除 `huajin2`**（效果只持续到自己回合开始）|

**原生** `SwdHuajin2` 的过滤臂：

```rust
matches!(event, EventData::Damage(d) if d.source == Some(owner) && !d.prevented)
|| matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true } if *player == owner)
```

⇒ **伤害面 = `huajin2` ✓、相位面 = `huajin3` ✓** —— **两面都有上游依据，不是「自加分支」** ❌

**错因**：`classify` 见到上游 `damageBegin` 就把「同时有相位分支」判成自加，
**没有考虑上游通过 `group` 挂的子技能**（**第三例**：`lexue`、`miedao`、`huajin2`）。

**处理**：登记 `behavior_has_damage_and_phase_two_faces()` 白名单（当前只有 `SwdHuajin2`，
且要求**已逐条读过上游**才能进），并**不**做通用的 `group` 合并 ——
因为**合并会把「清理型子技能」的触发面误当成主触发面**（实测会让 `G14` 从 14 涨到 17，属误判）。

**台账 86 → 85（A 41 / B 44）**
