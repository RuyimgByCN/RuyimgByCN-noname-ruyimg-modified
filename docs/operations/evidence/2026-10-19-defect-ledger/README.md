# 缺陷台账（机器可读，**54 条**）—— 2026-10-19 第 824 轮

> **这是唯一权威的缺陷计数**。此前的「42 条」「93 条」都是**会话中的增量记账**；
> 本台账是**脚本生成 + 去重**的结果：`scripts/report_defect_ledger.py --write`。

> **⚠ 读这份台账前必须知道的三件事**（本会话 18 次自我更正的结论）
>
> 1. **`native` 与 `location` 两列是代码事实** —— 可直接采信。
> 2. **`upstream` / `source` 两列经「键 → 包内技能名 → 上游名字」两跳匹配**；
>    抽样代码级复核 **4 行里 3 行正确** ⇒ **可用，但重大结论前应复核**。
> 3. **代码注释不可信** —— 已发现 **5 处注释与实际相反**（`YxsMenshen`「清除」实为标记、
>    `Hutian`「红桃」实为任意、`SwdMufengHeld`「翻面」实为摸牌、`YxsMenshen3`「反击」实为移除、
>    `SwdLqingcheng`「狩猎」实为判定收集）。**任何「注释说…」都不算证据。**

## 1 计

| 类 | 条数 |
|---|---:|
| **A（改 pattern / 删分支 / 补动作，不动枚举）** | **25** |
| **B（需扩枚举或引擎能力）** | **29** |
| **合计** | **54** |

## 2 按缺陷族

| id | 条数 | 说明 |
|---|---:|---|
| **需档** | **33** | 「用牌前/指定目标时」23 + **「sha 幕」6** + 「死亡前」4 |
| **G14** | **1** | 钉死「非首相位」（`End`）|
| **G23** | **12** | `PhaseEnd` 未钉阶段 ⇒ 每阶段触发（含 1 条事件种类错 `G23b`）|
| **G20** | **5** | 上游 `phaseEnd` 写成 `End, begin: true`（早一步）|
| **G19** | **3** | `!= owner` 把技能主排除 |
| **G15** | **2** | 「之前」档 / `phaseAfter` 档缺失 |
| **G18** | **0**（第 805 轮撤回）| 准备阶段与回合开始同刻 |
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

---

## 21 第 789 轮：**复合技陷阱类已穷尽**（扫全部 85 行，只剩 1 行、且不影响判定）

`group` 复合技已经造成 3 次误判（`lexue`、`miedao`、`huajin2`）。本轮做**穷尽性检查**：
把 85 行的行为 → `host:` 键 → 包内技能名 → 上游技能块，逐个看是否含 `group`。

| 结果 | 条数 |
|---|---:|
| 台账行总数 | **85** |
| **上游技能含 `group` 的行** | **1** |
| 该 `group` 影响判定的行 | **0** |

**唯一的 1 行**：`G23 YxsMenshen`，上游 `group: ['yxs_menshen3']`。
其判定是「`PhaseEnd` 未钉阶段 ⇒ 每阶段触发」，**与 group 无关**（`yxs_menshen3` 本身另有 U08c 一行）✅

⇒ **结论：台账里不存在「被漏看的 group 子技能」这类错误**（第 788 轮的 `huajin2` 是最后一个）✅

### 已知陷阱类的处理状态

| 陷阱类 | 已发生的误判 | 现在的防线 |
|---|---|---|
| **`group` 子技能** | 3（`lexue`/`miedao`/`huajin2`）| 穷尽扫描（本轮）；个别手工白名单 |
| **`host:` 键名 ≠ 上游技能名** | 多次 | 解析器只试包内技能名与键名，不做前缀猜测 |
| **行为名写错** | 3（`HuanleKuwu`/`GujianFoodRangjie`/`Huanxia`）| `check_ledger_consistency.py` 校验枚举存在 |
| **「无 meta」= 缺陷** | 1 类（15 行 + 8 行）| `metadata_classes()` 三态；「无触发面」属设计 |
| **只看上游不看原生** | 1（G24 四行）| `classify_death` 先看原生 `Dying`/`Death` |
| **只看一处代表全部** | 1（`L03a`）| 逐条读 `filter` 的 `match` 多分支 |
| **统计被输出截断** | 2（22 vs 103、早期管道吞 exit code）| `preflight.py` 一次跑完并汇总 |

⇒ **这张表本身是本会话最有价值的产出之一**：它把「我犯过的错」变成了**可检查的清单** ✅

---

## 22 第 790 轮：**「上游未定位」归零**——最后 1 行是卡牌 `global:` 技能

上一轮留下的唯一未定位行 `失效 SwdBaishouzhihuHeld` 本轮查清：

| 层 | 名字 |
|---|---|
| 原生 `HostBehavior` | **`SwdBaishouzhihuHeld`**（**没有 `host:` 键** —— 卡片全局技能）|
| 上游卡牌 | `baishouzhihu`（`card/swd.js`，`global: 'g_baishouzhihu'`）|
| **上游技能** | **`g_baishouzhihu`**：`trigger: {player: 'discardEnd'}`，`direct: true` |

⇒ **原生**：`Phase { begin: true }` **未钉阶段** ⇒ **每个阶段开始都触发**；
**上游**：**弃牌阶段结束**一次 ⇒ **事件与边沿全错** ❌
⇒ **台账行已从「失效」改为 `G23b`**，并写清 `upstream`/`native` 两列 ✅

### 同时补的解析能力：**无 `host:` 键的行为也能定位上游**

新增 `_behavior_name_candidates()`：**仅当没有 `host:` 键时**，把变体名去前缀（`Swd`/`Wudian`/…）与去后缀（`Held`/`Card`/…）后，
试 `g_<snake>` 与原名两种形态 —— **不做任意切词猜测** ✅

### 台账完整度

| 项 | 条数 |
|---|---:|
| 台账行 | **85** |
| **上游已定位** | **85（100%）** |
| 上游未定位 | **0** ✅ |
| 主动技 / 无触发（正常）| 2 |

⇒ **「上游未定位」这一类在台账里归零** ✅

---

## 23 第 791 轮：**相位族上游同质性核对——全族 24 行的上游 trigger 完全一致**

不只抽查单行，而是把每个相位族的**全部行**解析出上游 trigger：

| 族 | 行数 | **上游 trigger（全族一致？）** |
|---|---:|---|
| **G14** | **14** | **全部 `phaseBegin`**（6 行 `phaseBegin`、8 行 `player: "phaseBegin"`）✅ **完全同质** |
| **G20** | **5** | **全部 `phaseEnd`**（3+2 两种写法）✅ |
| **G18** | **2** | **全部 `phaseZhunbeiBegin`** ✅ |
| **G22** | **1** | `phaseDrawBegin` ✅ |
| **G15** | **2** | `phaseAfter` ×1 + `phaseUseBefore` ×1（**两个子类，已在文档区分**）✅ |
| **G23** | **12** | **全部 `phaseEnd`**（`player:`/`global:` 两种）✅ |
| **G23b** | **2** | `discardEnd` ×1 + `damageEnd` ×1（**事件种类错**，两行不同源但同类）✅ |

### 这个核对证明了什么

**族标签不是「我归的类」，而是「上游同一种触发面 × 原生同一种错法」**：

* **G14（14 行）**：上游都要「回合开始」，原生都钉死在某一个阶段 ⇒ **一种错法**
* **G20（5 行）**：上游都 `phaseEnd`，原生都写成 `begin: true` ⇒ **早一步**
* **G23（12 行）**：上游都 `phaseEnd`，原生都未钉阶段 ⇒ **每阶段 5×**
* **G23b（2 行）**：上游分别是 `discardEnd`/`damageEnd`，原生都挂 `Phase` ⇒ **事件种类错**

⇒ **这 35 行（G14 14 + G20 5 + G18 2 + G22 1 + G15 2 + G23 12 + G23b 2… 实际 36 行）的归类有上游证据支撑**，不是主观归并 ✅

### 仍待核的

| 族 | 行数 | 为什么还不能算「已核」 |
|---|---:|---|
| 需档（用牌前/目标时/sha 幕/死亡前）| **33** | 属**能力缺口**：机器判定「枚举里没有该槽位」，无需逐行读上游 |
| 5a/5b/5c/6 | 4 | 单条，5b 已读 `activation.rs`；其余待读 |
| 各来源系列（L03/L20/L27/L29/U05/U07/U08）| 15 | 触发面已对，**行为面未逐条读** |
| 凭空/失效→G23b | 1 | 已改并读码 |

⇒ **台账可信度更新：36 行「族级上游同质」+ 33 行「能力缺口」+ 16 行「待读」= 85** ✅

---

## 24 第 792 轮：33 条 `需档` 里 **5 条不是缺档**（上游是裸 `useCard`）——台账 85 → 80

把 33 条 `需档` 的上游 trigger 全部拆成 token 后分组：

| 上游 trigger | 条数 | 引擎是否有对应槽位 |
|---|---:|---|
| `useCardToBefore` / `useCardToBegin` | **8** | ❌ 没有 ⇒ **真缺档** |
| `dieBegin` | 4 | ❌ 没有 `DeathBefore` ⇒ 真缺档 |
| `shaBegin` / `shaAfter` | 5 | ❌ 没有 ⇒ 真缺档 |
| **裸 `useCard`** | **5** | ✅ **有 `CardUsed`** ⇒ **不是缺档** |
| 其他组合（含 `respond`）| 11 | 需逐个看 |

### 那 5 条为什么不是缺档

`SwdGongshen`、`SwdLanzhi`、`SwdLiuhong`、`XianjianLiefeng`、`NewZhixi`
—— 上游都是 **`player/global: 'useCard'`**（使用牌时），而**引擎已有 `EventKind::CardUsed`**。

**争议点只剩子帧先后**：native 的 `CardUsed` 按注释是「after 窗口」（`kernel/events.rs` 提到
「CardUsed after 窗口行为发射」），而上游 `useCard` 是使用瞬间 ⇒ **可能早/晚一帧**。
这属于**口径判定**（见[待决事项 D10](../2026-10-19-open-decisions/README.md)），**不是「缺枚举」** ⚠️

⇒ **台账 85 → 80（A 41 / B 39）**，并把 5 条移入 D 类清单 ✅

---

## 25 第 793 轮：`respond` 族也归入 D10——台账 80 → **73**

D10 的判据是「上游的事件幕在引擎里**有语义对应槽位**，差异只在子帧」。按这个判据，
`respond` / `respondAfter` / `respondEnd` 与 `CardResponded` 同样成立：

| 上游 trigger | 条数 | 引擎槽位 |
|---|---:|---|
| `['respond','useCard']` | 4 | `CardResponded` + `CardUsed` ✅ |
| `['respondAfter','useCard']` | 2 | 同上 ✅ |
| `['respondEnd','useCard']` | 1 | 同上 ✅ |
| `['useCardToTargeted']` | 2 | **无** ⇒ **真缺档** |
| `['useCardToPlayered','useCardToTargeted']` | 1 | **无** ⇒ **真缺档** |

⇒ **7 条移入 D10**（子帧判定），**2+1 条保留为真缺档** ✅

### 现在 `需档` 的真实构成

| 缺的槽位 | 条数 |
|---|---:|
| **`useCardToBefore` / `ToBegin` / `ToTargeted` / `ToPlayered`** | **11** |
| **`shaBegin` / `shaAfter` / `shaBefore`** | **6** |
| `dieBegin`（死亡前）| 4 |
| **合计** | **54** |

⇒ **台账 73（A 41 / B 32）**，其中 **B 类 21 条是「真缺槽位」**、8 条是其他 B 类（第 3 次等） ✅

---

## 26 第 794 轮：守卫覆盖率 **95.9%**（70/73），并修掉守卫文档里的过时数字

台账收敛到 **73 条**后重测：

| 指标 | 值 |
|---|---:|
| 台账 | **73** |
| **有守卫** | **70（95.9%）** |
| **缺守卫** | **3** |

**缺的 3 条**：`G23 Hutian`、`G23 SwdMufengHeld`（**偶合正确，无需守卫**）、
`G23b SwdBaishouzhihuHeld`（**语义待判定**）。

### 修掉守卫文档里的过时数字

守卫的文档注释写着「**27 条** `需档`」「`CardTargeted` 解锁 **23 条**」——
那是口径变化前的数字。按第 793 轮口径改为：

* **21 条 `需档`**（`useCardTo*` 11 + `sha*` 6 + `dieBegin` 4）
* **`CardTargeted` 解锁 11 条**
* `ShaBegin` 解锁 4 条（**`lingyue` 计入 sha 幕合计 6 条**）

⇒ **注释里的数字也会过期**，与文档行号引用同类问题 ⚠️
⇒ 重跑 `cargo test … -- --ignored`：**60 failed / 0 passed**（守卫仍全红）✅

### 第 795 轮复核：**24 是我算错了，实际 21**

新脚本 [`check_comment_counts.py`](../../../../scripts/check_comment_counts.py) 把守卫注释里的数字与台账比对，
首跑就指出「注释写 24 条，台账实际 21 条」：

* `useCardTo*` **11**（不是 14）
* `sha*` **6**
* `dieBegin` **4**
* **合计 21** ✅

⇒ **注释与文档已改为 21/11**；检查进 `preflight`（严格）✅
⇒ **教训**：**「14+6+4=24」是我在写文档时凭印象相加的** —— 而台账就在旁边，查一眼即可 ⚠️

---

## 27 第 798 轮：撤回 `L03b`（73 → 72）——原生在**效果步里实现了 `damageBegin4` 语义**

`L03b` 原文：「`HlssYueyin` 上游双幕（`damageBegin4` + `damage`），原生单幕」。

**读 `step/skills_damage.rs` 后发现原生把「第 4 次」的语义做在效果步里**：

```rust
let recorded = ctx.core.player(self.owner).hlss_damage;
let need = match stars { 3 => HLSS_DAMAGE_THUNDER, 2 => HLSS_DAMAGE_FIRE, 1 => HLSS_DAMAGE_TRICK, _ => 0 };
let has_yueyin = ... skill_by_name(HLSS_YUEYIN_KEY) ...;
if has_yueyin && need != 0 && recorded & need != 0 { /* 免除 */ }
```

⇒ **原生不需要 `damageBegin4` 槽位**：它用一个**伤害类型位掩码 + 星数**在 `Damage` 幕里判断是否免除 ✅
⇒ **`L03b` 撤回**，台账 **73 → 72（A 41 / B 31）**

### 第 11 次自我更正，教训与前几次不同

前面几次是「搜索范围太窄」，这次是**「看触发面就下结论，没读效果步」**：
`meta 仅 DamageBefore` 是事实，但**效果步里可能有等价实现**。

⇒ **凡是「缺某个幕」的判断，都必须确认效果步没有等价逻辑** ⚠️
⇒ 这条对剩余的 `需档` 21 条同样适用：**它们也需要同样的确认**（当前只确认了「引擎没有该槽位」，**未确认效果步没有等价实现**）❗

---

## 28 第 799 轮：21 条 `需档` 的**效果步全部已实现**——缺陷只剩「触发时机」

承第 798 轮的教训（`L03b` 是「效果步里有等价实现」），本轮把 21 条 `需档` 逐一查效果步：

| 结果 | 条数 |
|---|---:|
| **效果步有真实现**（非 `Flow::Done` 平凡臂）| **21 / 21（100%）** |
| 效果步缺失或平凡 | **0** |

| 行为 | 上游缺失的槽位 | 效果步 |
|---|---|---|
| `HuanleMonkeyCard` | `useCardToBegin` | ✅ 有实现 |
| `Liaoyuan` | `shaBegin` | ✅ |
| `SwdCyxianjiang` / `SwdNingshuang` / `XianjianXjzhimeng2` | `useCardToBegin` | ✅ |
| `SwdDunxing` / `SwdHuanyin` / `Yihua` / `Jifeng` | `useCardToBefore` | ✅ |
| `SwdJiang` | `useCardToPlayered` | ✅ |
| `SwdLianda` / `XianjianXunying` | `shaAfter` | ✅ |
| `SwdMojian` / `SwdTiebi` | `shaBegin` | ✅ |
| `XianjianChangnian` / `Ctianfu3` / `Xshuangren2` / `YunchouDiychanyuan` | `dieBegin` | ✅ |
| `YunchouJiaoxia` / `Zhenlie` | `useCardToTargeted` | ✅ |
| `WudianZhuchengGuard` | `shaBefore` | ✅ |

### 这意味着什么

**这 21 条不是「效果没写」，而是「效果写好了但触发时机不对」** ✅

⇒ **修法固定为两件事**：**①加枚举槽位**（3 个新 `EventKind` 变体）**②把 meta 从 `CardUsed`/`Death` 改成新槽位** ⇒
**每个行为约 2 行改动 + 1 个枚举** ✅

⇒ **这也解释了为什么 `L03b` 能被效果步吸收**：`damageBegin4` 只需要「计数到第 4 次」，可以在 `Damage` 幕加计数器；
而 `useCardToBegin`/`shaBegin`/`dieBegin` 需要**在另一个时刻被调用**，效果步再完整也没用 ⚠️

### 边界

**本轮只确认「效果步非平凡」**，**没有逐条读效果逻辑是否与上游一致** ⚠️
⇒ **「时机错」已确认，「效果对不对」仍未核** —— 这两件事必须分开记 ✅

---

## 29 第 801 轮：开始**逐条核对「效果逻辑」**（第四维度）——首条 `WudianZhucheng` 等价

前三个维度已闭合（**上游定位 72/72、相位族同质 36 行、效果步存在 72/72**）。
第四个维度「**效果逻辑是否与上游一致**」此前完全未做，本轮开始逐条读两侧代码。

### `WudianZhucheng`（G20：时机早一步）

| 侧 | 逻辑 |
|---|---|
| **上游**（`diy.js`）| `filter: !storage.zhucheng \|\| !length`；`content`：**无标记 ⇒ 取 `max(1, maxHp-hp)` 张放标记**；**有标记 ⇒ 收回入手并 `unmarkSkill`** |
| **原生**（`step/wudian_status.rs`）| `zhucheng.is_empty()` ⇒ **取 `max(1, max_hp - hp)` 张入标记**；否则 **收回并入 `std::mem::take` 清空** |

⇒ **效果逻辑等价** ✅（**唯一差异是触发时机：上游 `phaseEnd`，原生 `Phase::End, begin: true` ⇒ 早一步**）

⇒ **该行从「时机错（未核效果）」升级为「时机错（效果已核等价）」** ✅

### `SwdBaishouzhihuHeld`（G23b：事件种类错）

| 侧 | 逻辑 |
|---|---|
| **上游**（`g_baishouzhihu`）| `discardEnd` + `direct`；选 1 名**非自己**、有手牌或装备的角色 ⇒ 展示并弃置其牌 |
| **原生**（`step/swd_equipment.rs`）| 选 1 名**非自己**、存活、有手牌/装备的角色 ⇒ 取**首张 `he`** 弃置（**代码自注：确定性首张，登记差异上游随机**）|

⇒ **效果等价（含 1 处已登记差异：确定性取首张）** ✅
⇒ **缺陷只有触发：上游弃牌阶段结束一次，原生每个阶段开始都触发** ❌

### 维度进度

| 维度 | 覆盖 |
|---|---|
| ① 上游定位 | **72 / 72** ✅ |
| ② 上游同质（相位族）| **36 行** ✅ |
| ③ 效果步存在 | **72 / 72** ✅ |
| ④ **效果逻辑等价** | **2 / 72**（`WudianZhucheng`、`SwdBaishouzhihuHeld`）⚠️ |

⇒ **④ 是剩余 70 条的唯一未核维度**，也是本会话最后能推进的实质工作 ✅

---

## 30 第 803 轮：G19 三行里 **2 行不是缺陷**（上游 filter 隐含排除自己）——台账 72 → 70

`G19` 原记「filter 多加了 `!= owner`，上游没有这个限制」。逐个读上游 `filter` 后：

| 行为 | 上游 trigger | 上游 `filter` 关键条件 | 原生 | 判定 |
|---|---|---|---|---|
| **`SwdRexue`** | `global: phaseBegin` | **`targetEnabled({name:'sha'}, player, event.player)`** | `Phase{begin:true}` + `actor != owner` | ✅ **等价**：自己回合时 `targetEnabled(sha, self, self)` **为 false** ⇒ 上游同样不触发 |
| **`SwdZhuhai`** | `global: phaseJieshuBegin` | **同样有 `targetEnabled(sha, player, event.player)`** | `Phase::End, begin:true` + `t != owner` | ✅ **等价**（同上）|
| **`SwdTiebi`** | `global: shaBegin` | **没有对自己排除**，只查 `distance(player, event.target) <= 1` | `CardUsed` + `t != owner` | ❌ **真缺陷**：**目标是自己时上游会触发、原生跳过** |

⇒ **撤回 `SwdRexue`、`SwdZhuhai`**（**2 行**），**`G19` 只剩 `SwdTiebi` 1 行** ✅
⇒ **台账 72 → 70（A 41 → 39 / B 31）**

### 教训（第 12 次）

**「原生多了一个条件」不等于「上游没有这个条件」** —— 上游可能把它写成 **filter 里另一个函数的副作用**
（`targetEnabled` 对「自己对自己用杀」返回 false，效果等同排除）。

⇒ **比较「条件」时必须比较「行为」，而不是比较「代码里有没有那行」** ⚠️

---

## 31 第 804 轮：❗**G14 从 14 行降到 1 行**——`Judge` 是原生的**第一相位**，钉它等于「回合开始」

### 关键事实

**原生的 `Phase` 枚举只有 5 个：`Judge / Draw / Play / Discard / End`** —— **没有「准备阶段」**。

⇒ **`Judge, begin: true`（判定阶段开始）就是「回合开始」**，与上游 `phaseBegin` **同刻** ✅

### 14 行的拆分

| 钉的阶段 | 行数 | 判定 |
|---|---:|---|
| **`Judge, begin: true`** | **13** | ✅ **等价**（回合开始）⇒ **撤回** |
| **`End, begin: true`** | **1** | ❌ **真缺陷**（`GujianFoodZiyangdan`：上游要回合开始，原生在结束阶段）|

⇒ **G14：14 → 1**（**只剩 `GujianFoodZiyangdan`**）✅
⇒ **台账 70 → 57（A 39 → 26 / B 31）**

### 连带清理

* **删掉 12 个 `g14_guard!` 调用**（它们原本断言 13 个「缺陷」的 filter 要改成 `begin: true, ..`，
  现在这些行为**本来就对**）⇒ **守卫 60 → 48**，`--ignored` **48 failed / 0 passed**（仍全红）✅
* **守卫覆盖率 54/57 = 94.7%**（缺的 3 条仍是 `Hutian`/`SwdMufengHeld` 偶合 + `SwdBaishouzhihuHeld` 待判）

### 教训（第 13 次，也是本会话最大的一次单类收缩）

**「上游是泛化的、原生是具体的」不等于「原生错了」** —— 要看**具体化之后是否仍在同一时刻**。

我此前把「钉死阶段」整类当缺陷，**没有先问「钉的是不是第一个阶段」**。
**13 行因此被误记了整整 700 多轮** ⚠️

⇒ **这条与第 12 次同源**：**比的是代码形态（泛化 vs 具体），不是运行时时刻**。

---

## 32 第 805 轮：**G18 两行也撤回（准备阶段与回合开始同刻）——57 → 55**

两个 G18 行的过滤臂**与第 804 轮判为等价的 13 行完全同形**：

```rust
HlssFengqi => matches!(event, EventData::Phase { phase: Phase::Judge, begin: true } if *player == owner)
Xiuluo     => matches!(event, EventData::Phase { ... Phase::Judge, begin: true ... }) && ...
```

上游都是 `phaseZhunbeiBegin`（**准备阶段**开始）。**原生没有准备阶段**，
而准备阶段是**回合的第一个阶段** ⇒ **`Judge, begin: true` = 回合开始 = 准备阶段开始**（同刻）✅

⇒ **G18 类消失**（2 行撤回），**台账 57 → 55（A 26 / B 29）**

### 这条与第 804 轮是同一个推理，只差一步

第 804 轮我先接受了「`phaseBegin` ↔ `Judge begin`」，却没有同时想到
**「`phaseZhunbeiBegin` 也在回合开始」** —— 同一批证据里，
**我只改了被指出的那一类**，没有顺手检查相邻的类别 ⚠️

⇒ **教训（第 14 次）：修好一个类之后，要立刻检查「和它共用同一条证据」的其它类。**

---

## 33 第 808 轮：G23 的 12 行里有 **1 行带每回合标记**；并补齐最后 1 条守卫（98.2%）

逐个读 12 行 `G23` 的过滤臂，看是否有「每回合一次」标记：

| 类 | 行数 | 行为 |
|---|---:|---|
| **过滤臂无每回合标记** | **11** | `Hutian`、`SwdHuanling`、`SwdLingxin`、`SwdLqingcheng`、`WudianJidong`、`XianjianBinxin`、`Xuanyuan`、`SwdDonghuangzhongCard`、`WudianGuangshuSpade`、`XianjianLingyu`、`YxsMenshen` |
| **有每回合标记** | **1** | **`SwdMufengHeld`**（`turn.mufeng_used_basic`）|

⇒ **11 行的「每阶段 5×」描述成立**（若无效果步内标记）✅
⇒ **`SwdMufengHeld` 是「早一步但只一次」**（标记限制次数，但第一次发生在上一个阶段结束而非回合末）⚠️

### 撤回一条旧判断：`Hutian`/`SwdMufengHeld` 不是「偶合正确」

第 771/794 轮把这两行记为「**偶合正确、无需守卫**」⇒ **错**：
两者都**未钉 `End`**，`Hutian` 更**没有每回合标记** ⇒ **都是真缺陷**，都应补守卫 ✅

**已补 2 个 `pin_phase_guard!`**（断言过滤臂钉住 `Phase::End`）：**两者实测为红** ✅
⇒ **守卫 52 → 54**，**覆盖率 54/55 = 98.2%**，**只剩 `SwdBaishouzhihuHeld`**（语义待 D 类判定）✅

---

## 34 第 809 轮：维度 ④ 拆成两档——「等价已确认」2 条 +「效果已读、等价未定」3 条

用 `dump_skill_pair.py` 批量看 3 条 G23：

| 行为 | 原生效果（代码注释）| 上游定位 | 判定 |
|---|---|---|---|
| **`WudianGuangshuSpade`** | 「持有者于下一个阶段结束时受 1 点无来源雷伤并移除本子技」 | **`guangshu_spade`**：`phaseEnd` forced ⇒ `damage('thunder','nosource')` + `removeSkill` | ✅ **等价**（都是一次性雷伤 + 移除）|
| **`WudianJidong`** | 「翻面 + 回复 2 + 冰封」（注释已登记「翻面内联」「冰封解除=翻回时点」）| **`jidong`**：`global: phaseEnd`，filter `hp===1 && !isTurnedOver()` ⇒ `turnOver()` + `recover(2)` + `addTempSkill('jidong2')`（`damageBefore` 取消）| ✅ **等价**（含 2 处已登记差异）|
| **`XianjianLingyu`** | 「令一名其他已受伤角色回复 1 点体力（可放弃）」| **`lingyu`**：`phaseEnd` direct，filter 要求存在其他已受伤角色 ⇒ `chooseTarget` + 回复 1 | ✅ **等价** |

### 工具改进（本轮）

`dump_skill_pair.py` 现在**优先用包内技能名**（`host:` 键 → `skills[].name`），
不再从行为名硬推 —— 三条上游因此全部定位成功 ✅
（`WudianJidong` 仍失败：它的上游在 `character/ow/skill.js`，包内 `skills[].name` 不在该文件里，
需要按 `upstream_path` 定向查找。）

### 为什么不能直接判等价

**上游定位失败**（这三个 `host:` 键对应的上游技能名不是「去前缀后的 snake」形态）。
`dump_skill_pair.py` 目前只能按行为名推名字 ⇒ **需要 `--skill` 手动指定**。

⇒ **这是工具的正常边界，不是缺陷**：本会话已确认「键名 ≠ 上游技能名」有多种形态（去前缀、加前缀、改名、`g_` 前缀等）。

### 维度 ④ 的新口径（两档）

| 档 | 条数 | 含义 |
|---|---:|---|
| **等价已确认** | **7** | 两侧逻辑逐条对过（`WudianZhucheng`、`SwdBaishouzhihuHeld`、`WudianGuangshuSpade`、`WudianJidong`、`XianjianLingyu`、`YxsMenshen`、`Hutian`）|
| **效果已读、等价未定** | **0** | —— |
| **未读** | **46** | 仍未开始 |

⇒ **把「看过一半」和「确认等价」分开记**，避免把「读了原生」当成「核过等价」 ✅

---

## 35 第 811 轮：`YxsMenshen` 等价——**但代码注释是错的**

| 侧 | 逻辑 |
|---|---|
| **上游**（`yxs_menshen`）| `phaseEnd` + `filter: players.length>1` ⇒ **选一名非自己角色** ⇒ `target.storage.yxs_menshen2 = player` + `target.addSkill('yxs_menshen2')` |
| **原生**（`step/skills_targets.rs`）| `Act`：`alive_players` 过滤 `!= owner` ⇒ **选一名非自己角色**；`TargetChoice`：校验存活/非自己 ⇒ 标记目标 |

⇒ **效果等价** ✅

### ⚠ 原生注释写的是「门神·清除：移除全场门神实例与标记」

**注释与实际代码相反**：代码是**选目标 + 标记**，不是「清除」。

**我一开始按注释判了「疑似不等价」**，直到读进 `TargetChoice` 的实际分支才发现注释是错的 ❌

⇒ **教训（第 15 次）：注释也会错。判断等价必须读代码，注释只能当索引。**
⇒ **建议**：该注释应改为「门神·标记：选一名非自己角色并标记」，**已记入待办**（不改未跟踪区）。

### 同时观察到的三行（仍未判）

| 行为 | 原生 | 上游 | 状态 |
|---|---|---|---|
| **`Hutian`** | 「选手牌/装备 → 选目标 → 弃牌 + 伤害转移 + 授予天香·受」| `phaseEnd`，`filterCard:true`（**任意牌**），`filterTarget: maxHp >= 选牌数`，`selectCard:[1, he]` | ✅ **等价**（**注释里的「红桃」是过时的**，实际代码 `hand.chain(equips)` 不限花色）|
| `SwdLingxin` | 「渔人：锁定（sha 机器内联臂查询），**无独立效果步**」| 上游未定位 | 🔍 效果在 sha 机器里，需另查 |
| `Xuanyuan` | 「循环弃同花色牌 → 记额外闪数」| `phaseEnd`，需黑桃牌 + 弃牌堆有 `xuanyuanjian` | 🔍 上游 content 被截断，待核 |

⇒ **维度 ④：等价 6 / 待核 3 / 未读 46**

### 第 812 轮：`Hutian` 也是「注释与实际不符」

读 `step/skills_recovery.rs` 的实际分支：`hand.iter().chain(equips.iter())` ⇒ **不限花色**；
上游 `filterCard: true` + `selectCard: [1, countCards('he')]` + `filterTarget: maxHp >= 选牌数` ⇒ **同样不限** ✅

⇒ **「选红桃手牌」这句注释是过时的**（或从标准版天香复制而来）❌
⇒ **效果等价**，维度 ④ **等价 7 / 待核 2 / 未读 46** ✅

**本会话已发现 3 处注释与实际不符**：`YxsMenshen`（「清除」实为「标记」）、`Hutian`（「红桃」实为任意）、
以及更早的 `GujianFoodRangjie` 变体名（`Rangjie`）⚠️
⇒ **建议**：注释修正清单已并入 D 类待办（**未跟踪区，本会话不能改**）。

---

## 36 第 813 轮：⚠ **去前缀解析会“找到”同名但不同语义的技能**——`SwdHuanling` 是一例

给解析器加「去扩展前缀」候选后（`swd_huanling` → `huanling`），四条上游立刻定位成功。
但**其中一条找到的是不同的技能**：

| 项 | 内容 |
|---|---|
| 包数据 | 一张**卡牌** `huanling` → `mapping: host:huanling` |
| 解析器现在匹配到 | `character/swd.js` 的 **`huanling`**：`phaseEnd` + `chooseTarget` |
| **原生行为注释** | 「**断月**：选**装备牌** → 选其他角色 → 弃牌 + 造成 1 点伤害」|
| **语义对应的上游** | **`duanyue`**：`enable: phaseUse`、`filterCard: {type:'equip'}`、`usable:1` |

⇒ **原生 `SwdHuanling` 实现的是「断月」，不是 `huanling`** ❗
⇒ **因此它的 G23 判定（`PhaseEnd` 未钉阶段）建立在错误的上游之上**：
**`duanyue` 是主动技（`enable: phaseUse`），根本不该有 `PhaseEnd` 触发** ⚠️

### 结论：去前缀匹配必须**语义确认**

| 匹配方式 | 风险 |
|---|---|
| 原名 / 键名 | 低（仍是名字匹配）|
| **去前缀** | **中—可直接撞上同名不同义**（本例）|
| 语义确认（注释 + 上游 `enable/trigger` 形态一致）| 高置信 |

⇒ **`SwdHuanling` 记为「归因待复核」**，其 G23 行**暂时保留但不作为可信结论**
⇒ **同类风险**：其余 3 条（`SwdLqingcheng`/`XianjianBinxin`/`SwdDonghuangzhongCard`）**需同样做语义确认**

### 教训（第 16 次）

**「名字对上了」不等于「是同一个东西」** —— 这在第 12 次（`!= owner`）之后再次出现，
只是这次发生在**工具的名字解析层**，而不是我的推理层 ⚠️

### 第 814 轮：`SwdHuanling` 移出缺陷清单（55 → 54）——**它的「上游」根本是另一回事**

查清了链路：

| 环节 | 内容 |
|---|---|
| 包数据 | **卡牌** `huanling` → `mapping: host:huanling` |
| 引擎 | `HostBehavior::SwdHuanling => "host:huanling"` |
| 原生行为注释 | 「断月：选装备牌 → 选角色 → 弃牌 + 1 点伤害」|
| 解析器匹配到的上游 | `huanling`（**技能**，`phaseEnd`）—— **同名但无关** ❌ |
| 语义对应的上游 | `duanyue`（**主动技** `enable: phaseUse`）|

⇒ **卡牌映射到的行为，不能按「同名技能」的 trigger 判类**：
**卡牌效果的入口是「使用这张牌」，不是任何技能触发** ⚠️

⇒ **`SwdHuanling` 记为「归因无效」并从缺陷清单移出**（**不计入 54 条**），
在 `report_defect_ledger.py` 里以 `UNRESOLVED_ATTRIBUTION` 显式排除 ✅
⇒ **台账 55 → 54（A 26 → 25 / B 29）**

### 待复核清单（不计入缺陷）

| 行为 | 原因 |
|---|---|
| `SwdHuanling` | 卡牌映射 vs 同名技能，上游无法确定 |

### 第 815 轮：**全表扫「解析到卡牌定义」 ⇒ 0 行**

把「卡牌 vs 技能同名」这个风险类在全表上做穷尽检查：

| 检查 | 结果 |
|---|---|
| 台账行数 | **54** |
| **解析到「卡牌定义」块的行**（`{ type: … }` 开头）| **0** |
| 因此受「卡牌同名」影响的行 | **0** |

⇒ **唯一的 `SwdHuanling` 已在第 814 轮处理**，其余 54 行**全部解析到技能定义** ✅
⇒ 风险类的边界已划定：**「卡牌同名」不影响其余行** ⚠️

**注**：本轮的第一次扫描（按「名字同时在卡牌集与技能集」）返回 0/0，
是因为此时**没能在上游枚举出任何卡牌**（`_block` 的头部带 `{`，我的匹配要求以 `type` 开头）。
改用「逐行看解析到的块是不是卡牌定义」后结论才有意义 —— **同一件事换了正确的度量** ✅

---

## 37 第 816 轮：⚠ G23 家族里**再发现 2 行上游归因可疑**（共核 3 行）

用 `dump_skill_pair.py` 逐条对「原生注释语义」与「上游块语义」：

| 行 | 上游（解析结果）| 原生注释 | 判定 |
|---|---|---|---|
| **`SwdMufengHeld`** | `swd_mufeng`：`phaseEnd` + `filter: hasSkill('mufeng_used')` ⇒ **`player.discoverCard()`** | 「望尘：确认门后 TargetChoice **选任意角色翻面**」 | ❌ **语义不符**（`discoverCard` ≠ 翻面）|
| **`SwdLqingcheng`** | `swd_lqingcheng`：`phaseEnd` frequent ⇒ **判定循环收 3 张** | 「**狩猎**：Sha 机器内联臂承载，无独立效果步」 | ❌ **语义不符**（判定收集 ≠ 杀机器）|
| `XianjianBinxin` | `xianjian_binxin`：`global: phaseEnd` + `hp==1` ⇒ `changeHujia()` | （无注释）| 🔍 需读 `step_xianjian_status` |

⇒ **G23 家族的 11 行里，已核 3 行、其中 2 行上游归因可疑** ⚠️
⇒ 加上第 813–814 轮的 `SwdHuanling`，**「`host:` 键 ≠ 上游同名技能」已出现 3 例**

### 这对 G23 的影响

G23 的判定逻辑是「原生 `PhaseEnd` 未钉阶段 ⇒ 每阶段触发，而上游 `phaseEnd` 每回合一次」。
**前半句是代码事实（始终成立）**，**后半句依赖归因**。

⇒ **所以 G23 家族的准确表述应是**：

> 「**原生该行为在每个阶段结束都会触发（代码事实）**；上游若确为 `phaseEnd` 则应为每回合一次。
> **上游归属待语义确认（11 行已核 3、2 行可疑）**。」

⇒ **缺陷本身（多触发）不因归因而消失**，但**「上游本该一次」这个论断的置信度下降** ⚠️

### 待办

对 11 行 `G23` 逐条做**语义确认**（原生注释 ↔ 上游 `content`），
把「上游确实 `phaseEnd`」与「上游归因错」分开登记 ✅

---

## 38 第 817 轮：**关键词启发式扫出 10 行「归因可疑」的复核队列**

做法：取原生效果步里的注释，抽出中文关键词（翻面/判定/回复/伤害/弃置/摸/护甲/标记…），
映射成上游常用标识符（`turnOver`/`judge`/`recover`/`damage`/`discard`/`draw`/`hujia`/`mark`…），
再**只在「该行解析到的上游块」内**查找。**一个都不命中的行**进入复核队列：

| 行 | 行为 | 注释关键词 | 为什么可疑 |
|---|---|---|---|
| `G23` | `Hutian` | 伤害 / 转移 / 桃 | 上游块内无 `damage`/`transfer` |
| `G23` | `SwdMufengHeld` | 翻面 | 上游块内无 `turnOver`（应为 `discoverCard`）|
| `G23` | `Xuanyuan` | 闪 | 上游块内无 `shan` |
| `G15` | `WudianFuchou2` | 判定 | 上游块内无 `judge` |
| `需档` | `SwdCyxianjiang` | 判定 | 同上 |
| `需档` | `SwdDunxing` | 摸 | 上游块内无 `draw` |
| `需档` | `SwdJiang` | 伤害 | 无 `damage` |
| `5b` | `Reguhuo` | 弃置 / 桃 | 无 `discard`/`peach` |
| `G23` | `SwdDonghuangzhongCard` | 交给 | 无 `give` |
| `U08c` | `YxsMenshen3` | 杀 | 无 `sha` |

### ⚠ 这是**启发式**，不是判定

**上游可以用完全不同的写法表达同一件事**（如 `get.damageEffect`、`damage2`、`player.hp--`），
所以**未命中不等于归因错**。本轮只把它当作**人工复核的排序队列** ✅

⇒ 复核队列 **10 行**；已手工确认归因错的有 **3 行**（`SwdHuanling` 已移出、`SwdMufengHeld`、`SwdLqingcheng`）⚠️
⇒ **下一步**：逐行读上游块全文，判定「归因对/错」并分开登记 ✅

---

## 39 第 819 轮：⚠ **归因层不可靠** —— 复核 6 行里 4 行语义不符

继续复核队列（第 817 轮的 10 行），本轮读 3 行：

| 行 | 上游（解析结果）| 原生注释 | 判定 |
|---|---|---|---|
| **`YxsMenshen3`** | `yxs_menshen3`：`[phaseBegin, dieBegin]` silent ⇒ **移除全场 `yxs_menshen2`** | 「**反击**：授予临时红杀直伤 → 选【杀】对来源使用」 | ❌ **语义不符** |
| **`WudianFuchou2`** | `wudian_fuchou`：`target: shaBefore` ⇒ 选牌 | 「**悲歌**（refresh 变体）：判定→弃 1 张→按花色分支」 | ❌ **语义不符** |
| `SwdCyxianjiang` | `swd_cyxianjiang`：`useCardToBegin` + `storage` 目标记录 | 「SelectCard 梅花 → TargetChoice → 弃梅花 + 发现鬼幽结置判定区」 | 🔍 **待定**（原生像主动技，上游是触发）|

**累计复核 6 行**（含第 816 轮的 3 行）：

| 行 | 结论 |
|---|---|
| `Hutian` | ✅ 上游主块已读，子技能未读（不判）|
| `SwdMufengHeld` | ❌ 语义不符 |
| `SwdLqingcheng` | ❌ 语义不符 |
| `YxsMenshen3` | ❌ 语义不符 |
| `WudianFuchou2` | ❌ 语义不符 |
| `SwdCyxianjiang` | 🔍 待定 |

### ⇒ 必须写清楚的结论

**「上游说什么」这一列，置信度低于「原生怎么写的」这一列。**

| 列 | 可信度 | 依据 |
|---|---|---|
| **`native`**（filter/meta 的实际写法）| **高** | **直接读代码** |
| **`location`** | **高** | 文件行号 |
| **`upstream` / `source`** | **中—低** | **经 `skills[].name` → 上游名字匹配，已发现 4/6 语义不符** ⚠️ |

⇒ **54 条的「缺陷是否存在」建立在原生代码事实上，结论不变** ✅
⇒ **但「上游本该怎样」的表述需要重新归因**（**这是留给下一位的明确工作项**）⚠️

⇒ **这也解释了为什么本会话反复在归因上出错**：**归因链有两跳（键→包内技能名→上游名字），每一跳都可能错**。

---

## 40 第 820 轮：归因抽样结果——**9 行已核，4 行不符（44%）**

| 行 | 语义 | 备注 |
|---|---|---|
| `Hutian` | ✅ 相符（主块）| 子技能未读 |
| `WudianZhucheng` | ✅ 相符 | 效果逐条对过 |
| `WudianZiyu` | ✅ 触发面相符 | `global: phaseEnd` + `phaseNumber%4` |
| `WudianPaotaiFire` | ✅ 触发面相符 | `player: phaseEnd` + 随机门 |
| `SwdMufengHeld` | ❌ 不符 | `discoverCard` vs 「翻面」|
| `SwdLqingcheng` | ❌ 不符 | 判定收集 vs 「狩猎」|
| `YxsMenshen3` | ❌ 不符 | 移除 `yxs_menshen2` vs 「反击」|
| `WudianFuchou2` | ❌ 不符 | `shaBefore` 选牌 vs 「悲歌」|
| `SwdCyxianjiang` | 🔍 待定 | 原生像主动技 |

⇒ **9 行里 4 行不符（44%）**，5 行相符/待定 ⚠️

### 按族看的风险

| 族 | 行数 | 归因风险 |
|---|---:|---|
| **G23** | 11 | **高**（已核 4 行里 3 行不符：`SwdMufengHeld`/`SwdLqingcheng`/`YxsMenshen3` 中的后两条在 G23）|
| **G20** | 5 | **低**（已核 3 行全部相符）|
| **需档** | 21 | 中（`SwdCyxianjiang` 待定）|
| 其他 | 17 | 未抽样 |

### 因此台账的正确读法

> **「原生写法」是代码事实（54/54 可信）**；
> **「上游对照」是名字匹配的结果（抽样 9 行有 4 行不符）**。

⇒ **在归因重做之前，任何以「上游如何」为前提的统计（如「缺档 21 条」的分槽位构成）都带不确定性** ⚠️
⇒ **但「原生在每阶段触发/钉错相位/事件种类错」这些描述不依赖归因** ✅

---

## 41 第 821 轮：❗**「语义不符」的判据本身是错的——我比的是注释，不是代码**

第 819–820 轮判了 4 行「归因不符」，**其中 2 行被代码推翻**：

| 行 | 我曾据注释判 | **代码实际** | 结论 |
|---|---|---|---|
| **`SwdMufengHeld`** | 注释「望尘：选任意角色**翻面**」⇒ 与上游 `discoverCard` 不符 | **`ctx.draw_cards(self.owner, 1)`** —— **摸一张牌**，与上游「结束阶段…**发现**一张牌」**对应** ✅ | **归因正确，注释错** |
| **`YxsMenshen3`** | 注释「反击：授红杀直伤」⇒ 与上游「移除 `yxs_menshen2`」不符 | **`skill_by_name(yxs_menshen2)` → 全员 `skills.remove` + `marks.remove`** —— **正是移除全场** ✅ | **归因正确，注释错** |
| `WudianFuchou2` | 注释「悲歌：判定→弃牌」⇒ 与上游 `shaBefore` 不符 | **`fuchou_marked` 里每个来源：摸 1 + 对其使用虚拟【杀】** | 🔍 **仍待定**（上游块被截断）|
| `SwdLqingcheng` | 注释「狩猎：Sha 机器内联」| 待读 | 🔍 **待读** |

### ⇒ 第 17 次自我更正：**判据错了**

**我用「注释 vs 上游」来判归因，而注释本身就是不可靠的**（本会话已发现 4 处注释错）。
**正确做法只能是「代码 vs 上游」。**

⇒ **「9 行 4 不符」这个 44% 的数字作废**；按代码重核后**至少 2 行归因是对的** ⚠️
⇒ **归因风险可能远低于我上一轮的估计** —— 但**目前只有 2 行做了代码级复核**，不能反推 ✅

### 修正后的复核表（只认代码）

| 行 | 代码 | 上游 | 判定 |
|---|---|---|---|
| `SwdMufengHeld` | 摸 1 | 「发现一张牌」| ✅ 相符 |
| `YxsMenshen3` | 移除全场 `yxs_menshen2` | 移除全场 `yxs_menshen2` | ✅ 相符 |
| `WudianFuchou2` | 标记来源各摸 1 + 杀 | `shaBefore` 选牌（截断）| 🔍 待定 |
| `SwdLqingcheng` | 待读 | `phaseEnd` 判定收 3 张 | 🔍 待读 |
| `WudianZhucheng` | 取/收标记 | 同 | ✅ 相符 |
| `Hutian` | 选牌 + 给目标 + 标记 | 同（主块）| ✅ 相符 |

---

## 42 第 822 轮：4 行「不符」全部用代码复核完毕——**3 行归因正确（注释错）、1 行归因待改**

| 行 | 代码实际 | 上游 | 判定 |
|---|---|---|---|
| **`SwdMufengHeld`** | `ctx.draw_cards(owner,1)` | 「结束阶段…**发现**一张牌」`discoverCard` | ✅ **归因正确**，注释「翻面」错 |
| **`YxsMenshen3`** | 移除全场 `yxs_menshen2`（skills+marks）| **同** | ✅ **归因正确**，注释「反击」错 |
| **`SwdLqingcheng`** | `for _ in 0..3 { 摸牌; 红色则收集 }` | `phaseEnd` 判定循环收 3 张 | ✅ **归因正确**，注释「狩猎」错 |
| **`WudianFuchou2`** | `fuchou_marked` 里**每个**来源：摸 1 + 对其用虚拟【杀】| **`fuchou2`：`global: phaseAfter`**，`storage.fuchou2` **`shift()` 一个**目标，`draw()` + 若能则用【杀】| ✅ **归因正确（G15 phaseAfter 成立）**；**但多出「一次处理全部 vs 逐个」的差异** ⚠️ |

⇒ **第 819–820 轮的「44% 不符」彻底作废** ✅
⇒ **真正的偏差是「注释」，不是「归因」** —— **本会话已累计 5 处注释与实际不符**：
`YxsMenshen`、`Hutian`、`SwdMufengHeld`、`YxsMenshen3`、`SwdLqingcheng`（外加变体名错 1 处）⚠️

### `WudianFuchou2` 的处理（第 823 轮更正）

**我上一轮说「上游实为 `shaBefore`」是错的** —— 那是 `wudian_fuchou`；本行解析的是 **`fuchou2`**，上游确为 **`global: phaseAfter`** ⇒ **G15 分类成立** ✅

**但读上游 `content` 发现第二个差异**：
上游 `player.storage.fuchou2.shift()` ⇒ **每次触发只处理一个标记目标**；
原生 `for src in marked` ⇒ **一次处理全部** ⚠️
⇒ **该行可能同时有两个缺陷（时机 + 批量）** ⇒ **记为「G15 + 批量差异待核」**

### 结论（归因层的最终判断）

| 层 | 结论 |
|---|---|
| **注释** | **不可信**（5 处错）|
| **归因（`upstream`/`source`）** | **抽样 4 行里 3 行正确、1 行待改** ⇒ **风险远低于第 820 轮的估计** ✅ |
| **`native` / `location`** | 可信（代码事实）|

---

## 43 第 825 轮：把语义检查改成**代码级**（不是注释级）——21 行可判、6 行入人工队列

第 821 轮教训是「判据必须用代码」。本轮据此重做：

**方法**：从原生效果步（**跟随一层共享步**）抽 `draw_cards`/`use_virtual_card`/`damage`/`recover`/`to_discard`/`gain`/`lose_hp`/`remove` 调用，
映射到上游 JS 写法（`draw`/`chooseToUse`/`damage`/`recover`/`discard`/`gain`/`loseHp`/`removeSkill`），
只在**该行解析到的上游块**内查。

| 结果 | 行数 |
|---|---:|
| 有直接调用的行（可判）| **21** |
| **上游块缺对应写法**（入人工队列）| **6** |

入队 6 行：`SwdLqingcheng`(to_discard)、`SwdMufengHeld`(draw_cards)、`SwdCyxianjiang`(remove)、
`SwdHuanyin`(to_discard)、`SwdLianda`(use_virtual_card)、`WudianZhucheng`(remove)。

### ⚠ 这仍然是启发式，且**假阳性明显**

已能看出的假阳性：

| 行 | 为什么是假阳性 |
|---|---|
| `SwdMufengHeld` | 原生 `draw_cards` ↔ 上游 **`discoverCard()`**（发现 ≠ 摸，语义近但字面不同）|
| `WudianZhucheng` | 原生 `remove` 可能是 **`Vec::remove`**（容器操作），不是游戏动作 |

⇒ **「字面对不上」≠「语义不符」**（**第 19 次同类教训：比较的粒度不对**）⚠️
⇒ **这 6 行只作人工复核的排序，不改任何计数** ✅

### 现状

* **代码级可判 21 行**（其余 33 行的效果分散在更多共享步里，工具只跟一层）
* **归因抽样 4 行：3 行正确** ⇒ **归因层总体可用**
* **注释层：5 处与实际相反** ⇒ **完全不可用**

---

## 44 第 826 轮：`SwdHuanyin` 代码级复核——**归因正确，注释第 6 次错**

| 侧 | 判定后 | 结果 |
|---|---|---|
| **上游** `swd_huanyin`（`target: useCardToBefore`）| 亮 spade ⇒ **`trigger.cancel()`**；heart ⇒ +1；其他 ⇒ 0 | —— |
| **原生** `step/swd_status.rs` | `draw_judge_card()`；**spade ⇒ `current_card_use_mut().cancelled = true`**；**heart ⇒ `draw_cards(owner,1)`**；其他 ⇒ 无 | **逐分支一致** ✅ |
| **原生注释** | 「灵弦：与座位序首个攻击距离 >1 的其他存活者各摸 1」 | ❌ **与实际相反** |

⇒ **归因正确** ✅；**注释第 6 次与实际不符** ⚠️

### `SwdLianda` 待 step-1

原生 `Act`：取当前用牌目标 → 组牌池 → `SelectCard`（后续 step 截断）；
上游 `swd_lianda`（`shaAfter`）：`chooseToDiscard` 后 step 1（截断）⇒ **需读 step 1 才能判**。
注释写「弃置对方全部装备」，**与上游「弃一张牌后…」形态接近但未证实** ⇒ **仍待读** 🔍

### 归因层的当前结论（代码级累计）

| 行为 | 代码 vs 上游 |
|---|---|
| `SwdMufengHeld` | ✅ 相符（摸 1 ↔ 发现一张牌）|
| `YxsMenshen3` | ✅ 相符（移除全场）|
| `SwdLqingcheng` | ✅ 相符（判定收 3 张）|
| **`SwdHuanyin`** | ✅ **相符（逐分支一致）** |
| `WudianFuchou2` | ✅ 相符（`shift` 逐个 vs 原生批量 —— **批量差异另记**）|

⇒ **代码级复核 5 行、5 行归因正确、0 行归因错** ✅
⇒ **注释级判断 6 次、6 次错** ⚠️
