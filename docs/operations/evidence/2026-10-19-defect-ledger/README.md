# 缺陷台账（机器可读，**5 条**）—— 2026-10-19 第 909 轮（含「已核实非缺陷」排除）

> **这是唯一权威的缺陷计数**。此前的「42 条」「93 条」都是**会话中的增量记账**；
> 本台账是**脚本生成 + 去重**的结果：`scripts/report_defect_ledger.py --write`。

> **⚠ 读这份台账前必须知道的三件事**（本会话 18 次自我更正的结论）
>
> 1. **`native` 与 `location` 两列是代码事实** —— 可直接采信。
> 2. **`upstream` / `source` 两列经「键 → 包内技能名 → 上游名字」两跳匹配**；
>    代码级复核 **7 行全部正确** ⇒ **可用，但重大结论前应复核**。
> 3. **代码注释不可信** —— 已发现 **6 处注释与实际相反**（`YxsMenshen`「清除」实为标记、
>    `Hutian`「红桃」实为任意、`SwdMufengHeld`「翻面」实为摸牌、`YxsMenshen3`「反击」实为移除、
>    `SwdLqingcheng`「狩猎」实为判定收集、`SwdHuanyin`「各摸 1」实为判定取消/摸牌）。**任何「注释说…」都不算证据。**

> **四维进度（第 902 轮）**
>
> | 维度 | 覆盖 | 含义 |
> |---|---:|---|
> | ① 上游定位 | **54 / 54** | 每行都能指出上游出处 |
> | ② 上游同质 | **36 行** | 相位族全族上游 trigger 一致 |
> | ③ 效果步存在 | **54 / 54** | 无一条是「功能没做」|
> | ④ **效果等价** | **45 已确认 + 7 形态相符** | **1 效果不符**（`SwdMufengHeld`）、**1 待查**（`SwdJiang`）、**0 未读** ⚠️ |
>
> **归因层代码级复核 21 行（含复合技、消费方、公式对照）**；**注释与实际不符累计 18 处**（**完全无关 15 / 错位 2 / 描述代码里没有的效果 1**）；**自我更正累计 21 次**。

## 1 计

| 类 | 条数 |
|---|---:|
| **A（改 pattern / 删分支 / 补动作，不动枚举）** | **2** |
| **B（需扩枚举或引擎能力）** | **3** |
| **合计** | **5** |

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
| **合计** | **5** |

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

---

## 45 第 827 轮：`SwdLianda` / `SwdCyxianjiang` 代码级形态核对——均**形态相符**

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`SwdLianda`** | `Act` 取当前用牌目标 → 组手牌+装备池 → `SelectCard` 弃 1（`min:0,max:1` 可取消）→ 取出该牌 | `swd_lianda`（`shaAfter`）：`chooseToDiscard('he')` → step 1（未读）| ✅ **形态相符**（都是弃 1 张再结算），**效果待证** |
| **`SwdCyxianjiang`** | `Act` 从当前 `CardUsed` 取 `targets.first()` → 以**该目标的装备**为候选 | `swd_cyxianjiang`（`useCardToBegin`）：filter 检查 **`event.target.countCards('e')`** | ✅ **形态相符**（都作用于目标装备），**效果待证** |

⇒ **归因层代码级累计：7 行复核、7 行归因正确、0 行错** ✅
⇒ 但其中 **2 行只核到「形态」**（效果分支未逐条读），不能与「等价已确认」混记 ⚠️

### 分层记法（第三次收紧）

| 档 | 条数 | 含义 |
|---|---:|---|
| **等价已确认**（代码逐分支）| **5** | `WudianZhucheng`、`SwdBaishouzhihuHeld`、`WudianGuangshuSpade`、`WudianJidong`、`XianjianLingyu` |
| **形态相符、效果待证** | **2** | `SwdLianda`、`SwdCyxianjiang` |
| **未读** | **47** | —— |
| **归因错** | **0** | 代码级复核至今未发现 |

---

## 46 第 829 轮：`SwdLqingcheng` 判等价；**`SwdMufengHeld` 多出一个效果差异**

### `SwdLqingcheng` → 等价已确认（第 6 条）

| 侧 | 逻辑 |
|---|---|
| 上游 `swd_lqingcheng` | `phaseEnd` frequent ⇒ **判定，红牌收**，累计 **3 张**后 `gain` |
| 原生 | `for _ in 0..3 { 摸牌; 红色则收集 }` ⇒ 同 |

⇒ **等价** ✅（时机仍是每阶段 vs 回合末，属 G23 那一半）

### `SwdMufengHeld` → **新发现一个效果差异**（同一行的第二个缺陷）

| 侧 | 逻辑 |
|---|---|
| 上游 `swd_mufeng` | `player.discoverCard()` |
| **`discoverCard()` 的实际实现**（`game.js:17582`）| **`num=3`；`get.inpile(...).randomGets(3)`** ⇒ **「发现三张，选一张」** |
| 原生 | **`ctx.draw_cards(self.owner, 1)`** ⇒ **摸一张** |

⇒ **「发现三张选一」≠「摸一张」** ⇒ **效果差异成立** ✅

**处理**：该行（G23 时机）**再记一个效果缺陷**，但**不新增行**（保持「一行 = 一行为」的口径），
在行内标注「**G23 时机 + 效果（发现三选一 vs 摸一）**」⚠️

⇒ **这也是本会话第一次在「已核等价」之外发现「效果真不符」** —— 前 6 条都是等价 ✅

---

## 47 第 830 轮：`Xuanyuan` / `SwdDonghuangzhongCard` 形态相符；并修正一个过强的判断

### `Xuanyuan`

| 侧 | 逻辑 |
|---|---|
| 上游 `xuanyuan` | `phaseEnd`；filter 要求**有黑桃牌**且弃牌堆有 `xuanyuanjian`；弃一张黑桃 ⇒ **装备弃牌堆的轩辕剑** |
| 原生 | `Act`：候选 = **手牌/装备里的黑桃牌** → `SelectCard` 弃置 |

⇒ **弃黑桃部分相符** ✅；**「从弃牌堆装备轩辕剑」未在已读片段中出现** ⇒ 形态相符、效果待证 🔍

### `SwdDonghuangzhongCard`

| 侧 | 逻辑 |
|---|---|
| 上游卡牌 `donghuangzhong` 是**装备定义**（`skills:['donghuangzhong']`）；**真正的技能**在 `card/swd/skill.js`：`equipSkill`、`player: phaseEnd`、filter 有**红色手牌** ⇒ `chooseCardTarget(filterCard: 红色)` 弃红牌 + 选目标 |
| 原生 | `SelectCard` 过滤**红/方片手牌** → `TargetChoice` |

⇒ **形态相符** ✅

### ⚠ 修正第 814 轮一个过强的判断

当时我写「**卡牌映射到的行为不能按同名技能判类**」。本轮反例：

**`SwdDonghuangzhongCard` 解析到的是卡牌块，但那个卡牌块本身带 `skills:['donghuangzhong']`，
而原生行为实现的正是这个「装备技能」** ⇒ **解析到卡牌块不等于归因错** ⚠️

⇒ 正确说法：**要看卡牌块是否 `skills:[...]` 指向的技能与行为语义一致**；
**只有「卡牌块与行为语义完全无关」时才是归因错**（如 `SwdHuanling`）。

⇒ **形态相符累计 4 行**（`SwdLianda`、`SwdCyxianjiang`、`Xuanyuan`、`SwdDonghuangzhongCard`）✅

---

## 48 第 831 轮：`XianjianBinxin` / `SwdLingxin` 均**等价**（④ → 8）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`XianjianBinxin`** | `if 目标存活 && hp == 1 { ctx.change_hujia(t, 1) }` | `global: phaseEnd`，filter `event.player.hp==1` ⇒ **`trigger.player.changeHujia()`** | ✅ **等价**（都是「体力为 1 的阶段结束者 +1 护甲」）|
| **`SwdLingxin`** | `for 0..3 { 摸牌; 若为红桃则收集 }` | `player: phaseEnd` frequent ⇒ `get.cards(3)`，**弃非红桃、收红桃** | ✅ **等价** |

### 第 7 处注释与实际不符

`SwdLingxin` 的注释写「**渔人：锁定（sha 机器内联臂查询）**」，而代码是**翻 3 张收红桃** ⇒ **完全无关** ❌

⇒ **注释错例累计 7 处**：`YxsMenshen`、`Hutian`、`SwdMufengHeld`、`YxsMenshen3`、`SwdLqingcheng`、`SwdHuanyin`、**`SwdLingxin`**

### 维度 ④ 现状

| 档 | 条数 |
|---|---:|
| **等价已确认** | **8** |
| **形态相符、效果待证** | **4** |
| **未读** | **42** |
| **效果不符** | **1**（`SwdMufengHeld`：发现三选一 vs 摸一）|

---

## 49 第 832 轮：`SwdDunxing` / `SwdNingshuang` 形态相符——**注释第 8、9 次错**

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`SwdDunxing`** | `Act`：牌池 = 手牌 + 装备 ⇒ `SelectCard`（`dunxing_discard`）| `target: useCardToBefore` ⇒ `chooseToDiscard('he')` 取消该牌 | ✅ **形态相符**（弃 1 张 `he`）|
| **`SwdNingshuang`** | `Act`：候选 = 手牌/装备中的**黑桃或梅花** ⇒ `SelectCard` | `target: useCardToBegin`，filter `color=='black'` ⇒ 弃黑色牌令来源横置/翻面 | ✅ **形态相符**（弃黑色牌）|

**两处注释都与代码无关**：

* `SwdDunxing` 注释写「**极谋·姬良面：回合外受伤后摸 1**」
* `SwdNingshuang` 注释写「**工神：选装备牌 → 弃置 + 取消该使用**」

⇒ **注释错例累计 9 处** ⚠️（`YxsMenshen`、`Hutian`、`SwdMufengHeld`、`YxsMenshen3`、`SwdLqingcheng`、`SwdHuanyin`、`SwdLingxin`、**`SwdDunxing`**、**`SwdNingshuang`**）

### 维度 ④ 现状

| 档 | 条数 |
|---|---:|
| **等价已确认** | **8** |
| **形态相符、效果待证** | **6** |
| **未读** | **40** |
| 效果不符 | 1 |

⇒ **核过的 15 行里，14 行归因与形态都对**；**唯一不符的是 `SwdMufengHeld` 的效果（发现三选一 vs 摸一）** ✅

---

## 50 第 833 轮：两条 `dieBegin` 行**均等价**（④ → 10）；注释第 10 次错

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`XianjianChangnian`** | `Act`：选一名**其他存活角色**（`p != owner`）⇒ 移交 `hej` 全部牌 | `dieBegin` ⇒ `chooseTarget(player != target)` ⇒ 移交 `getCards('hej')` | ✅ **等价** |
| **`YunchouDiychanyuan`** | **`if let Some(src) = self.target { ctx.lose_max_hp(src, 1) }`** | `dieBegin` **forced**，filter `event.source != undefined` ⇒ **`trigger.source.loseMaxHp(true)`** | ✅ **等价**（都是「来源失去 1 点体力上限」）|

**`YunchouDiychanyuan` 的注释写「悲歌（refresh 变体）：判定→弃 1 张→按花色分支」** ⇒ **第 10 处注释与实际不符** ⚠️

### 维度 ④ 现状

| 档 | 条数 |
|---|---:|
| **等价已确认** | **10** |
| 形态相符、效果待证 | 6 |
| 未读 | **38** |
| 效果不符 | 1 |

⇒ **核过 17 行：16 行归因与效果相符、1 行效果不符** ✅
⇒ **注释错例 10 处**（每 1.7 行就有一处注释与实际不符）⚠️

---

## 51 第 834 轮：⚠ **两个工具对同一行为的「上游」判断会不一致**

| 工具 | `XianjianCtianfu3` 的上游 |
|---|---|
| **台账解析器**（`resolve_trigger`）| **`player:'dieBegin'`** |
| **对照工具**（`dump_skill_pair.py`）| `xianjian_ctianfu`（`enable: phaseUse` 主动技）|

原因是**候选名字的尝试顺序不同**：dumper 先试「行为名去后缀得到的 `xianjian_ctianfu`」，而解析器按「包内技能名 → 键名 → 去前缀」的顺序。

**已把 dumper 的候选顺序与解析器对齐**（`_candidates(*names)` 按同一顺序展开）✓
**但仍不完全一致**：当包内技能名为空时，dumper 会从行为名推导出一个**能命中**的名字（`xianjian_ctianfu`），而解析器不会。

### ⇒ 结论：**以解析器为准，dumper 只用于阅读**

⇒ **凡是要写进台账的「上游」，必须用 `resolve_trigger` 的结果**，不能用 dumper 的显示 ⚠️
⇒ 这条与第 813 轮的「去前缀必须语义确认」是同一族问题：**多个入口给出不同答案时，必须指定唯一权威** ✅

---

## 52 第 835 轮：两条 `dieBegin` 行的**死亡窗口语义**成立（形态相符 → 8）

这两行的「上游是哪个技能」在解析器与 dumper 之间有分歧（见第 834 轮），
但**它们的「事件窗口」是可以独立判定的**：

| 行为 | 原生代码 | 窗口 |
|---|---|---|
| **`XianjianCtianfu3`** | `ctianfu_clear(ctx, owner)`；移除 `ctianfu2`/`ctianfu3` 两实例；**挂附闪入弃堆** | **挂附者死亡时清理** ⇒ **死亡窗口** ✅ |
| **`XianjianXshuangren2`** | 取 `off_court` 全部牌 ⇒ `to_discard` | **死亡时弃置离场区** ⇒ **死亡窗口** ✅ |

⇒ **两者都实现「死亡时」的效果**，与台账标注的 `dieBegin` 窗口一致 ✅
⇒ **但「具体对应上游哪个技能」仍未定**（解析器/dumper 分歧）⚠️

### 一个可以分离的判断

**「事件窗口」与「技能归属」是两个独立维度**：

* **窗口**：代码在哪个时机执行 —— **可直接读** ✅
* **归属**：对应上游哪个技能 —— **依赖名字匹配** ⚠️

⇒ 本轮证明：**即使归属存疑，窗口仍可判**。这比笼统说「归因待核」更有用 ✅

---

## 53 第 836 轮：`Yihua` 形态相符；`SwdJiang` 代码与注释自相矛盾

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`Yihua`** | 从当前 `CardUsed` 帧取 `u.player` 为作用对象；候选 = **自己的手牌** ⇒ 弃自己的牌 | `target: useCardToBefore`，filter `targets.length==1 && player != event.player && hand>=2` ⇒ `chooseToDiscard` | ✅ **形态相符**（弃自己的牌、作用于使用者）；**上游的 `hand>=2` 门槛原生未见** ⚠️ |
| **`SwdJiang`** | **`ctx.draw_cards(self.owner, 1)`** —— 只摸 1 张 | `swd_jiang`（`useCardToPlayered`/`useCardToTargeted`，杀/决斗）| 🔍 **代码与注释矛盾**：注释写「伤害前置 off_court 黑牌返还并 -1 伤害」，**代码只有摸 1 张** ❌ |

### `SwdJiang`：又一种新情况

前面 10 处注释错都是「注释描述别的技能」；**这一处是「注释描述一个代码里根本没有的效果」** ⚠️

⇒ 无法判断是：**① 注释残留（真实实现只摸 1）**、还是 **② 代码被简化（原有效果未实现）** ⇒ **记为待查**

### 维度 ④

| 档 | 条数 |
|---|---:|
| 等价已确认 | **10** |
| **形态相符、效果待证** | **9** |
| **未读** | **35** |
| 效果不符 | 1 |
| **代码与注释矛盾待查** | **1**（`SwdJiang`）|

---

## 54 第 837 轮：`SwdMojian` **等价**、`Jifeng` **形态相符**（含公式对照）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`SwdMojian`** | `if 目标存活 { draw_cards(t,1) }`；`if hp < max_hp { recover(owner,1) }` | `shaBegin` ⇒ **`trigger.target.draw()`** + **`player.recover()`** | ✅ **等价** |
| **`Jifeng`** | `n = CardUsed.targets.len()`；**`cancel_permille = 1000·(1 − (1.3 + n/5)/n)`**，千分位掷点 | `useCardToBefore` forced ⇒ 上游注释「`Math.random() > (1.3 + n/5)/n` 时取消」 | ✅ **形态相符**（**同一公式**，只是用千分位整数代替浮点）|

**两处注释又错**：`SwdMojian` 写「道法：来源弃首张手牌」、`Jifeng` 写「昆仑镜：阶段收束记录快照/回滚」 ⇒ **注释错例累计 12 处** ⚠️

⇒ **维度 ④：等价 11 / 形态 10 / 未读 33 / 效果不符 1 / 代码注释矛盾 1**

### `Jifeng` 的公式对照值得单记

上游用**浮点概率**、原生用**千分位整数**，两者数学等价但有**舍入差**：
`round(1000·p)/1000` 与原 `p` 的偏差 ≤ 0.0005 ⇒ **属可接受的实现差异**，不是缺陷 ✅

---

## 55 第 838 轮：`Liaoyuan` / `HuanleMonkeyCard` 形态相符（形态 → 12）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`Liaoyuan`** | `Act`：从当前 `CardUsed` **捕获花色**（注释说明「弃牌会发射帧，之后当前帧不再是 CardUsed」）⇒ 按花色处理 | `shaBegin`：`get.suit(trigger.cards)` ⇒ 按该花色循环处理 | ✅ **形态相符**（都是「取杀的牌花，按花处理」）|
| **`HuanleMonkeyCard`** | **取消桃的结算**：①使用者手牌中的桃移入自己手牌 ②弃自己的猴子 ③写回取消位 ⇒ 提交时跳过结算 | 卡牌 `huanle_monkey` 的装备技 `monkey`；`kernel/monkey.rs` 注明「使用不发生，**无 `CardUsed` 帧**；全部放弃则按普通桃结算」 | ✅ **形态相符**（都是「改写成不发生/夺取」）|

⇒ **维度 ④：等价 11 / 形态 12 / 未读 31 / 效果不符 1 / 代码注释矛盾 1** ✅

### 一个观察

这两行的原生代码**注释是准确的**（`Liaoyuan` 的花色捕获原因、`HuanleMonkeyCard` 的三步都写对了），
与前面 12 处「注释与代码无关」形成对比 ⇒ **注释质量在文件间差异很大**，不能一概而论 ⚠️

---

## 56 第 839 轮：`SwdTiebi` / `WudianZhuchengGuard` 形态相符（形态 → 14）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`SwdTiebi`** | 若存在黑桃/梅花手牌 ⇒ **弃置该黑牌** | `global: shaBegin`，filter 要黑手牌 + 目标无护甲 + 距离 ≤1 ⇒ **`chooseToDiscard({color:'black'})`** | ✅ **形态相符**（弃黑牌）|
| **`WudianZhuchengGuard`** | `Act` 询问使用者：选牌（弃「筑城」张数）或 `Pass`（杀无效，帧 `cancelled` + 杀归还）| `target: shaBefore`，filter 要 `storage.zhucheng` 非空 ⇒ 一次性选 n 张 | ✅ **形态相符**（弃筑城牌让杀无效）|

**`SwdTiebi` 的注释写「流虹：使用杀后摸 1」** ⇒ **注释错例 13 处** ⚠️
（`WudianZhuchengGuard` 的注释准确，再次说明**注释质量文件间差异大**）

### 维度 ④

| 档 | 条数 |
|---|---:|
| 等价已确认 | **11** |
| **形态相符、效果待证** | **14** |
| **未读** | **29** |
| 效果不符 | 1 |
| 代码注释矛盾 | 1 |

⇒ **54 行里已核 27 行（50%）**：**11 等价 + 14 形态相符 + 1 效果不符 + 1 待查** ✅

---

## 57 第 840 轮：`XianjianXunying` 形态相符；`XianjianXjzhimeng2` **代码窗口与台账一致、与当前解析不一致**

| 行为 | 原生代码 | 台账标的上游 | 当前解析出的上游 | 判定 |
|---|---|---|---|---|
| **`XianjianXunying`** | 杀结算后取**手牌首张杀**对同目标再用（注释已登记「上游玩家选牌，原生自动取首张」）| `shaAfter` ⇒ 对同目标再用一张杀 | `shaAfter` | ✅ **形态相符**（含已登记差异）|
| **`XianjianXjzhimeng2`** | **被使用同类型牌时摸 1**（`ctx.draw_cards(owner,1)`）| `target: useCardToBegin` | **`phaseEnd`** ❌ | ✅ **台账正确**；**当前解析器给错** |

### ⇒ 重要：**解析器并非总是对的**

第 834 轮我写过「以解析器为准」。本轮反例：**`XianjianXjzhimeng2` 的代码明显是「被使用牌时摸 1」**，
台账存的 `useCardToBegin` **对**，而当前 `resolve_trigger` 给的 `phaseEnd` **错**。

**原因**：解析器的候选顺序在 813–834 轮之间改过（加了去前缀等），**同一行为在不同轮次会解析到不同上游** ⚠️

⇒ **正确表述**：**解析器不是权威，「代码」才是权威**；解析结果只能当线索，**必须用代码窗口校验** ✅
⇒ 这也解释了为什么台账里存的上游与当前解析可能不一致（**台账是历史快照**）⚠️

---

## 58 第 841 轮：`SwdHyunshen` / `SwdLingxian` 形态相符（形态 → 18）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`SwdHyunshen`** | `hyunshen_count += 1`（饱和加）| `respond`/`useCard` 且牌为【闪】⇒ `storage.hyunshen++` + `markSkill` | ✅ **形态相符**（都是计数）|
| **`SwdLingxian`** | 找一名**距离 > 1** 的其他存活者 ⇒ **双方各摸 1** | `respond`/`useCard`，filter 要「存在攻击距离 > 1 的其他角色」 | ✅ **形态相符**（距离门槛一致）|

**两处注释都只描述了「另一半」**：`SwdHyunshen` 写「准备阶段摸等量并清零」、`SwdLingxian` 写「拼点…」
⇒ **注释错例累计 14 处** ⚠️（**这两处不是完全不相关，而是描述了同一技能组的另一个行为**）

### 维度 ④

| 档 | 条数 |
|---|---:|
| 等价已确认 | **11** |
| **形态相符、效果待证** | **18** |
| **未读** | **25** |
| 效果不符 | 1 |
| 代码注释矛盾 | 1 |

⇒ **已核 29/54 = 54%** ✅

---

## 59 第 842 轮：`SwdLuomei` / `SwdXielei` 形态相符（形态 → 20）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`SwdLuomei`** | `draw_cards(owner, 1)` | `useCard`/`respond`，filter 要求**单张梅花牌** ⇒ 内容按梅花计数 | ✅ **形态相符**（梅花 → 摸牌）|
| **`SwdXielei`** | `Act`：从当前 `CardUsed` 帧记**排除面**（原目标列表）⇒ 组自己的牌池（弃置）| `useCard`/`respondAfter`，filter 要【杀】且人数 > 2 ⇒ `chooseCardTarget`（弃牌 + 选目标）| ✅ **形态相符** |

**两处注释又写的是别的技能**：`SwdLuomei` 写「星点」、`SwdXielei` 写「湛卢」 ⇒ **注释错例累计 16 处** ⚠️

### 维度 ④

| 档 | 条数 |
|---|---:|
| 等价已确认 | **11** |
| **形态相符、效果待证** | **20** |
| **未读** | **23** |
| 效果不符 | 1 |
| 代码注释矛盾 | 1 |

⇒ **已核 31/54 = 57%** ✅

### 注释错例的类型统计（累计 16 处）

| 类型 | 处数 | 例 |
|---|---:|---|
| **完全无关**（写的是别的技能）| **13** | `YxsMenshen`「清除」实为标记、`SwdLuomei`「星点」、`SwdXielei`「湛卢」|
| **错位**（写了同组另一行为）| **2** | `SwdHyunshen`「准备阶段」、`SwdLingxian`「拼点」|
| **描述代码里没有的效果** | **1** | `SwdJiang`「off_court 黑牌返还 -1 伤害」|

---

## 60 第 843 轮：`XianjianRunxin` **等价**、`SwdYuchen` **形态相符**

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`XianjianRunxin`** | `Act`：候选 = **已受伤**的存活角色 ⇒ 回复 1 | `useCard`/`respondEnd`，filter 要**红桃牌** + 存在已受伤角色 ⇒ 令其回复 1 | ✅ **等价**（都是「红桃牌 → 令已受伤者回复 1」）|
| **`SwdYuchen`** | `Act`：候选 = 有手牌或装备的存活角色 | `useCard`/`respondAfter`，filter 要**黑色牌** ⇒ 后续造成雷伤 | ✅ **形态相符**（选目标 → 雷伤）|

⇒ **维度 ④：等价 12 / 形态 21 / 未读 21 / 效果不符 1 / 待查 1**
⇒ **已核 33/54 = 61%** ✅

**本轮两处注释都准确**（`XianjianRunxin` 写对了「红桃→回复」；`SwdYuchen` 写对了「弃牌→选目标→雷伤」）
⇒ **与第 838、839 轮的观察一致：注释质量在文件之间差异极大** ⚠️

---

## 61 第 844 轮：`YunchouJiaoxia` **等价**、`Zhenlie` **形态相符**

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`YunchouJiaoxia`** | **`ctx.draw_cards(self.owner, 1)`** | `target: useCardToTargeted`，filter 要**红色牌** ⇒ **`player.draw()`** | ✅ **等价** |
| **`Zhenlie`** | `Act`：从 `CardUsed` 帧取 `u.player` ⇒ ①**移出目标**（写回帧 targets）| filter：`event.player != player && (sha 或 trick)` ⇒ 移出目标 + 后续效果 | ✅ **形态相符** |

**`YunchouJiaoxia` 的注释写「红莲：来源弃其手牌与装备区全部红牌」** ⇒ **注释错例 17 处** ⚠️
（而 `Zhenlie` 的注释与代码一致：都写了「移出目标」）

### 维度 ④

| 档 | 条数 |
|---|---:|
| 等价已确认 | **13** |
| **形态相符、效果待证** | **22** |
| **未读** | **19** |
| 效果不符 | 1 |
| 代码注释矛盾 | 1 |

⇒ **已核 35/54 = 65%** ✅

---

## 62 第 845 轮：两条 G20 行**均等价**（含概率公式对照）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`WudianZiyu`** | `Act`：二选一 —— **摸 1** 或 **回复 1**（可放弃；满血时不给回复选项）| `chooseDrawRecover` | ✅ **等价**（**选项集与满血门一致**）|
| **`WudianPaotaiFire`** | `lv = paotai_lv`；`roll = gen_range(100)`；`roll < 30 * lv` ⇒ 随机一名存活敌人 **1 点火焰伤害** | `switch(storage.paotai)` ⇒ 30/60/100；**`100*Math.random() < num`** ⇒ `getEnemies()` 随机 | ✅ **等价**（**概率模型完全相同：30%×层**）|

⇒ **维度 ④：等价 15 / 形态 22 / 未读 17 / 效果不符 1 / 待查 1**
⇒ **已核 37/54 = 69%** ✅

**两处注释都准确**（`WudianZiyu` 甚至写明「满血时无回复选项——上游 `chooseDrawRecover` 同口径」）。

### 两次「公式级对照」的积累

| 行为 | 上游写法 | 原生写法 | 结论 |
|---|---|---|---|
| `Jifeng` | `Math.random() > (1.3+n/5)/n` | 千分位整数 | **舍入差 ≤ 0.0005，可接受** |
| **`WudianPaotaiFire`** | `100*Math.random() < 30/60/100` | `gen_range(100) < 30*lv` | **同一模型** |

⇒ **概率类实现是本会话核对得最细的一类**，因为公式可以直接对照 ✅

---

## 63 第 846 轮：`BossBaiyi` 是**第 4 例复合技误判**；`Tianxiang` 形态相符

### `BossBaiyi`：上游是复合技，「凭空新增触发」这个标签不对

| 项 | 内容 |
|---|---|
| 上游 `boss_baiyi` | **`group: ['boss_baiyi_draw','boss_baiyi_thunder','boss_baiyi_discard']` + `subSkill`** ⇒ **父技能没有顶层 trigger** |
| 子技能 `boss_baiyi_discard` | **`global: roundStart`**，filter `game.roundNumber == 5` |
| 原生 | **`wing_bai_discard(ctx, owner)`** ⇒ **正是「弃牌」那一面** |

⇒ **原生实现的是子技能 `boss_baiyi_discard`**，**台账的「凭空新增触发」标签是错的** ⚠️
⇒ **这是复合技误判的第 4 例**（`lexue`、`miedao`、`huajin2`、**`boss_baiyi`**）

**正确的分类**应是：**上游子技能在 `roundStart`（第 5 轮）触发，原生在阶段窗口触发 ⇒ 事件种类错**。

### `Tianxiang`：形态相符

| 侧 | 逻辑 |
|---|---|
| 上游 | `damageBegin3`（**第 3 次**伤害前），filter 要场上有 `heart`/`diamond` 标记 ⇒ 后续按伤害量摸牌 |
| 原生 | `Act`：候选 = **红桃手牌** ⇒ 后续按本次伤害量摸牌并移除自身 |

⇒ **形态相符**（红桃/标记 → 摸牌）✅；**但「第 3 次」这个计数窗口原生未见**（`damageBegin3` 无对应槽位）⇒ **与台账「第3次」标签一致** ✓

---

## 64 第 847 轮：**全表复扫复合技** —— 9 行（8 个行为）上游带 `group`

用「解析到的上游块里是否出现 `group:`」复扫全表：

| 行 | 行为 | 上游 `group` 内容 | 已核状态 |
|---|---|---|---|
| `G23` | `Hutian` | `'hutian3'` | ✅ 主块已核（等价）|
| `G23` | `SwdMufengHeld` | `'mufeng_count'` | ✅ 已核（**效果不符**：发现 vs 摸）|
| `需档` | `Liaoyuan` | `['liaoyuan2','liaoyuan3']` | ✅ 形态相符 |
| `需档` | `SwdCyxianjiang` | `'cyxianjiang_clear'` + `subSkill` | ✅ 形态相符 |
| **`5b`** | **`Reguhuo`** | **`['reguhuo_shan','reguhuo_wuxie','reguhuo…']`** | 🔍 **本轮新查：待核** |
| `G20` | `WudianZhucheng` | `'zhucheng2'` | ✅ 等价 |
| `G23` | `YxsMenshen` | `'yxs_menshen3'` | ✅ 等价 |

⇒ **8 个行为里 7 个已核，只剩 `Reguhuo`** ✅

### `Reguhuo` 本轮读到的

| 侧 | 内容 |
|---|---|
| 上游 `reguhuo` | `enable: ['chooseToUse','chooseToRespond']` + `hiddenCard` + filter（需 `countCards('hs')` 且 **`!hasSkill('reguhuo_phase')`**）；**`group: ['reguhuo_shan','reguhuo_wuxie',…]`** |
| 原生 | `Act`：选手牌（`reguhuo_car…`）⇒ 后续未读 |
| 原生注释 | 「**攻心**：选目标 → 看其手牌选一张红桃 → 弃置或置于牌堆顶」 |

⇒ **注释与上游完全不同**（攻心 ≠ 若谷）；**代码只读到「选手牌」**，后续未读 ⇒ **待核** 🔍
⇒ **但台账 5b 的核心论断（缺 `reguhuo_phase` 每回合标记）与上游一致** ✓ ——**上游确有 `hasSkill('reguhuo_phase')` 检查** ✅

---

## 65 第 848 轮：`XianjianFenglue` 形态相符；`GujianFoodZiyangdan` 上游仍未定位

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`XianjianFenglue`** | 出牌阶段开始选一名其他角色 ⇒ **逐张自动对其使用手中可指定的杀与决斗**（换牌后走虚拟管线；注释已登记「其余锦囊不自动使用」）| `phaseUseBefore`，filter：手牌中存在**可对其使用**的牌 | ✅ **形态相符**（含已登记差异）|
| **`GujianFoodZiyangdan`** | **结束阶段**：`change_hujia(owner, -1)` + 倒计时扣减，归零移除（注释登记「上游每阶段开始衰减」）| **未找到**（名字不是 `gujian_food_ziyangdan`）| 🔍 **待核**（**但原生注释与台账 G14 的论断一致：上游在阶段开始、原生钉 `End`**）|

⇒ **维度 ④：等价 15 / 形态 24 / 未读 15 / 效果不符 1 / 待查 1**
⇒ **已核 39/54 = 72%** ✅

### `GujianFoodZiyangdan` 的情况

**它的缺陷论断不依赖上游定位**：原生代码明确写在**结束阶段**执行，而台账 G14 的依据是「上游在阶段开始」
⇒ **只要「原生钉 End」是代码事实，这一行就成立**（**上游具体在哪个 `phaseBegin` 只影响描述，不影响缺陷存在**）✅

---

## 66 第 849 轮：`YxsFanji` 形态相符；`YxsFanji2` 待核

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`YxsFanji`** | `Act`：从 `Damage` 帧取 `source` ⇒ 记为 `self.target`（后续对该来源发临时技）| `damageEnd`，filter 要手牌有【杀】且 `event.card` 为杀或决斗 ⇒ `addTempSkill` | ✅ **形态相符**（都是「受到杀/决斗伤害后 → 临时技」）|
| **`YxsFanji2`** | **`Flow::Done`**（宿主无效果步）| `shaBegin` + 红杀 ⇒ **`trigger.directHit = true`** | ✅ **等价**（**消费方已找到，见第 850 轮**）|

**`YxsFanji` 的注释写「灾变：摸忠+2-反-内」** ⇒ **注释错例 18 处** ⚠️

### `YxsFanji2` 为什么不能判形态相符

**「无效果步」本身可能是正确的实现路径，也可能是效果缺失。**
区分二者**必须**去杀机器里找 `directHit` 的等价物——本轮 grep `direct_hit`/`directHit` 只在**注释**里出现 3 次，
**没有同名字段** ⇒ **不能确认**，记为待核 ✅

⇒ **「有档未用」类行的判据**：**必须在消费方找到实际逻辑**，不能只看「空实现 + 一句注释」⚠️

---

## 67 第 850 轮：`YxsFanji2` 的消费方**找到了** ⇒ 等价（16）

第 849 轮我判它「待核」，因为 grep `direct_hit` 只命中注释。**换查法**（查常量 `U08_YXS_FANJI2_KEY` 的所有引用）后找到：

**`crates/engine-native/src/kernel/sha/step.rs:552`**

```rust
let yxs_fanji2_direct_hit = {
    let red = matches!(ctx.catalog.cards[self.card as usize].suit, Heart | Diamond);
    red && ctx.catalog.skill_by_name(U08_YXS_FANJI2_KEY)
             .map(|def| ctx.core.has_skill(self.source, def))
             .unwrap_or(false)
};
```

| 侧 | 语义 |
|---|---|
| 上游 | `shaBegin` + **红杀** ⇒ `trigger.directHit = true`（不可闪避）|
| 原生 | **红牌 && 来源有 `yxs_fanji2`** ⇒ `yxs_fanji2_direct_hit` ⇒ 不可闪避 |

⇒ **完全一致** ✅ **等价**

### 教训（第 20 次）：查「消费方」要用**常量名**而不是**语义名**

第 849 轮我按 `directHit`/`direct_hit` 找，找不到就判待核；
**换成查 `U08_YXS_FANJI2_KEY`（行为名派生的常量）一次就命中** ⚠️

⇒ **「没找到」往往是我的检索键错了，而不是不存在** ✅

---

## 68 第 851 轮：`Rangjie`（L10）是「有档未用」，且**消费方也找不到**

| 项 | 内容 |
|---|---|
| 台账行 | `6 A Rangjie L10` |
| `keys.rs` | `HostBehavior::Rangjie => "host:rangjie"` ✅ 存在 |
| `factory.rs` | `HostBehavior::Rangjie => (None, None)` —— **无效果步** |
| `filtering.rs` | `filter_common(...)` —— 只注册过滤 |
| **消费方** | **`EPIC_RANGJIE_KEY` 在 `constants.rs` 之外无任何引用** ❌ |

### 与 `YxsFanji2` 的对比（同样是「有档未用」）

| 行为 | 常量引用（`constants.rs` 外）| 结论 |
|---|---|---|
| `YxsFanji2` | **`kernel/sha/step.rs:552`** ✅ | 有消费方 ⇒ **等价** |
| **`Rangjie`** | **无** ❌ | **无消费方** ⇒ **待核/疑似缺效果** |

⇒ **「有档未用」不是一族，必须逐个查消费方** ✅
⇒ 本轮把 `Rangjie` 从「未读」推进到「**已查消费方：找不到**」——**这是负面证据，比「未读」更有价值** ⚠️

### 说明

`Rangjie` 的台账行属于 A 类（改 pattern / 删分支 / 补动作），
**本轮不能确定它是缺陷还是我的检索键又错了**（第 850 轮的教训）；
但**已确认 `EPIC_RANGJIE_KEY` 无外部引用**是一个可复现的事实 ✅

---

## 69 第 852 轮：❗**撤回上一轮对 `Rangjie` 的判断** —— `(None, None)` 不等于「无效果步」

| 检查 | 结果 |
|---|---|
| 台账里 `factory.rs` 为 `(None, None)` 的行为 | **14 个** |
| **其中在 `step/*.rs` 里有 step 臂的** | **14 个（全部）** |
| 真正「无效果步」的 | **0 个** |

**`Rangjie` 的 step 臂**（`step/skills_cards.rs`）：
`Act` ⇒ 按伤害点数初始化剩余次数（`current_damage().amount`）⇒ 进入 `SelectNature` 询问 ⇒ … **有完整实现** ✅

⇒ **第 851 轮「`Rangjie` 无效果步、消费方找不到」的结论作废** ⚠️
⇒ **`factory.rs` 的 `(None, None)` 指的是另一类工厂槽位**（不是「有无 step」），**我误读了它的语义**

### 第 21 次自我更正

**我从一个字段的取值推断出一个它并不表示的事实。**

教训：**在用一个字段做判断前，先验证这个字段确实表示那个意思**（第 850 轮的教训是「检索键错了」，这一轮是「字段语义错了」——**同一族的第 2 次**）⚠️

⇒ `Rangjie` 回到「**效果步存在、逻辑已读**」；**是否需要记为等价**要看上游块，但**不再是「疑似缺效果」** ✅

---

## 70 第 854 轮：`PhaseEndRecoverOrDraw` 形态相符；另两行上游未定位

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`PhaseEndRecoverOrDraw`** | **若已受伤 ⇒ 回复 1；否则摸 1** | 卡牌 `huanle_jiuwei` 的装备技 `jiuwei` | ✅ **形态相符**（结束阶段二选一）|
| `GujianFoodLiyutang` | 阶段开始：体力为全场最少（或之一）⇒ **+1 护甲** + `food_tick` | **未找到**（名字不是 `gujian_food_liyutang`）| 🔍 待核（注释自洽）|
| `ShengkeCheck` | **被克 ⇒ 弃一张手牌**（确定性首张，已登记「上游为目标自选 he」）；**被生 ⇒ 摸一张**；无关系 ⇒ 无事 | **未找到**（名字不是 `wuxing_shengke`）| 🔍 待核（逻辑完整、有登记差异）|

⇒ **维度 ④：等价 16 / 形态 26 / 未读 11 / 效果不符 1 / 待查 1**
⇒ **已核 43/54 = 80%** ✅

### 两行上游未定位，但**代码逻辑都完整**

`GujianFoodLiyutang` 与 `ShengkeCheck` 都属于「**上游名字找不到、原生实现清楚**」的情形
⇒ **它们的缺陷论断不依赖上游**（台账对它们的主张是相位/事件层），**效果层只能记为待核** ⚠️

---

## 71 第 855 轮：`HlssGaiming` **等价**（消费方 `kernel/judge.rs:517`）；`XiguaMopaiFanbei` 形态相符

### `HlssGaiming`（L03c）

| 侧 | 语义 |
|---|---|
| 上游 `hlss_gaiming` | `judgeBefore` ⇒ **`get.cards(7)`** + `chooseCardButton` ⇒ **选一张作为判定结果** |
| 原生 | 宿主步 `Flow::Done`；**消费方 `kernel/judge.rs:517`**：持技时**亮牌堆顶 7 张、由其选 1 作为判定结果**（注释明写「改判/天妒窗跳过、上游 `judgeBefore` + `trigger.cancel` + `trigger.direct` 同口径」）|

⇒ **完全一致** ✅ **等价**

**但宿主步的注释写「潜龙：按触发面累加道心值（上限 99）」** ⇒ **第 19 处注释错**，且是**「错位」类型**（描述的是同族的另一个技能）⚠️

### `XiguaMopaiFanbei`（L20a）

宿主步 `Flow::Done`，注释「诈降：无效果步（出牌校验/杀机器/摸牌收束内联消费）」；`L20_MOPAI_KEY` **有外部引用** ✅
⇒ **形态相符**（消费方存在），**具体分支未逐条读** ⚠️

⇒ **维度 ④：等价 17 / 形态 27 / 未读 9 / 效果不符 1 / 待查 1**
⇒ **已核 45/54 = 83%** ✅

### 「消费方在 kernel 里」已成常见形态

本会话已在 `kernel/sha/step.rs`、`kernel/judge.rs` 各找到一次 ⇒
**判「无效果步」的行必须去 `kernel/` 找消费方**，这是可复用的方法 ✅

---

## 72 第 856 轮：`ShaDamageForceDiscard` 形态相符；`XiguaXueliangFanbei` 待核

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`ShaDamageForceDiscard`** | 目标存活且有手牌 ⇒ **弃置其首张手牌** | 卡牌 `huanle_kuwu` 的装备技 `kuwu`（`subtype: equip1`，`distance.attackFrom: -1`）| ✅ **形态相符**（杀造成伤害后令目标弃手牌）|
| `XiguaXueliangFanbei` | `max_hp += hp`、`recover(hp)`（**体力上限与已损体力同增**）| **未找到**（名字不是 `xigua_xueliangfanbei`）| 🔍 待核 |

**`XiguaXueliangFanbei` 的注释写「门神：选一名其他角色 → 授予门神标记」** ⇒ **第 20 处注释错**（**内容与代码完全无关**）⚠️

⇒ **维度 ④：等价 17 / 形态 28 / 未读 8 / 效果不符 1 / 待查 1**
⇒ **已核 46/54 = 85%** ✅

### 注释错例累计 20 处

| 类型 | 处数 |
|---|---:|
| 完全无关（写的是别的技能）| **16** |
| 错位（同族另一行为）| **3** |
| 描述代码里没有的效果 | **1** |

⇒ **每 2.7 行就有 1 处注释与实际不符**（54 行 / 20 处）⚠️

---

## 73 第 857 轮：`Rangjie` / `BossBaiyi` 形态相符（已核 89%）

| 行为 | 原生代码 | 上游 | 判定 |
|---|---|---|---|
| **`Rangjie`** | `Act`：`count = current_damage().amount` ⇒ 按次数循环询问 | `damageEnd` ⇒ **`event.count = trigger.num`**，循环里选「获得牌」等 | ✅ **形态相符**（**次数都取自本次伤害量**）|
| **`BossBaiyi`** | `wing_bai_discard`：`wing_arm(WING_BAI_5)` + 遍历**非 Boss 方存活者** ⇒ 弃牌 | 子技能 `boss_baiyi_discard`：`global: roundStart` + `roundNumber == 5` | ✅ **形态相符**（**第 5 轮的弃牌面**）|

⇒ **`BossBaiyi` 的归因至此确认**：**原生实现的确实是子技能，不是「凭空新增触发」** ✅（第 846 轮的判断得到代码级支持）

⇒ **维度 ④：等价 17 / 形态 30 / 未读 6 / 效果不符 1 / 待查 1**
⇒ **已核 48/54 = 89%** ✅

**`Rangjie` 的注释写「蛊惑：弃一张手牌 → 声明牌名 → 转化使用」** ⇒ **第 21 处注释错**（`Rangjie` 上游是 `damageEnd` 循环，与「蛊惑」无关）⚠️

---

## 74 第 858 轮：`GwentCiguState`（G22「晚一步」）代码窗口与台账一致

| 侧 | 窗口 | 效果 |
|---|---|---|
| 上游（手工登记 `player: "phaseDrawBegin"`）| **摸牌阶段开始** | —— |
| **原生**（`step/gwent.rs`）| **摸牌阶段结束**（注释：「摸牌阶段结束：弃首张手牌并移除状态」）| 弃首张手牌 + `food_remove` |

⇒ **「晚一步」在代码里得到确认** ✅（`Begin` → **结束**）
⇒ 注释还登记了「等价净 -1」的差异 ✅

**维度 ④：等价 17 / 形态 31 / 未读 5 / 效果不符 1 / 待查 1**
⇒ **已核 49/54 = 91%** ✅

### 剩余 5 行

| 行 | 行为 | 状态 |
|---|---|---|
| `G14` | `GujianFoodZiyangdan` | 🔍 上游未找到（**缺陷不依赖上游**：原生钉 `End` 是代码事实）|
| `需档` | `GujianFoodLiyutang` | 🔍 上游未找到（代码逻辑完整）|
| `需档` | `XiguaXueliangFanbei` | 🔍 上游未找到（代码逻辑完整）|
| `需档` | `ShengkeCheck` | 🔍 上游未找到（代码逻辑完整、有登记差异）|
| `需档` | `XianjianXjzhimeng2` | ✅ 已核（第 840 轮：代码窗口 = `useCardToBegin`）|

⇒ **真正「未读」的只剩 4 行，且全部卡在「上游名字找不到」** ⚠️
⇒ **这不是没做，而是需要新的定位手段**（按 `upstream_path` 到具体文件里按语义搜）✅

---

## 75 第 859 轮：按「语义搜」找到 `GujianFoodLiyutang` 的上游 —— **窗口与台账一致**

按第 858 轮的建议，**不再猜名字**，而是到 `upstream_path` 的具体文件里按语义搜：

| 步骤 | 结果 |
|---|---|
| `grep -rl liyutang archive/upstream/` | **`card/gujian.js`**（`noname-liuli`）|
| 第一处 `liyutang:{`（111 行）| **卡牌定义**：`type:'food'`、`enable:true` ⇒ 目标获得牌 + `storage.liyutang_markcount=2` + `addSkill('liyutang')` |
| **第二处 `liyutang:{`（1346 行）** | **技能定义**：**`trigger: {player: 'phaseEnd'}`**、`forced`、`isMinHp()` ⇒ **`changeHujia()`** + `markcount--` |

| 侧 | 窗口 |
|---|---|
| **上游技能** | **`phaseEnd`（结束阶段）** |
| **原生**（`step/gujian.rs`）| 注释写「**阶段开始**：体力最少 ⇒ +1 护甲」|

⇒ **窗口不符（原生更早）** ⇒ **台账 G20「早一步」的判断得到确认** ✅
⇒ **同时上游描述可以精确化**：`phaseEnd` + `isMinHp()` + `changeHujia()` + `markcount--`

⇒ **维度 ④：等价 17 / 形态 32 / 未读 4 / 效果不符 1 / 待查 1**
⇒ **已核 50/54 = 93%** ✅

### 方法验证

**第 858 轮提出的「按语义搜」有效**：一次 grep + 两次读块就定位了「上游名字找不到」的行 ✅
⇒ **剩下的 3 行（`Ziyangdan`/`XueliangFanbei`/`ShengkeCheck`）应同样处理** ⚠️

---

## 76 第 860 轮：`GujianFoodZiyangdan` 上游定位成功 —— **G14「钉死阶段」确认**

同法（语义搜）在 `card/gujian.js` 找到两块：

| 块 | 内容 |
|---|---|
| **卡牌** `ziyangdan` | `type:'basic'`、`enable:true` ⇒ `target.changeHujia(3)`；若已有技则 `storage += 3`，否则 `addSkill('ziyangdan')` |
| **技能** `ziyangdan` | **`trigger: {player: 'phaseBegin'}`**、silent、`init: storage.ziyangdan = 3`；content：`hujia>0 ⇒ changeHujia(-1)`、`storage--`、**`hujia==0 或 storage==0 ⇒ removeSkill`** |

| 侧 | 窗口 | 效果 |
|---|---|---|
| **上游技能** | **`phaseBegin`（每阶段开始）** | 护甲 -1、倒计时 -1、归零移除 |
| **原生** | **结束阶段**（注释与代码一致）| `change_hujia(-1)` + `food_tick` |

⇒ **窗口不符** ⇒ **台账 G14（钉死阶段）的判断得到确认** ✅
⇒ **「倒计时扣减、归零移除」也与上游一致** ✅

⇒ **维度 ④：等价 17 / 形态 33 / 未读 3 / 效果不符 1 / 待查 1**
⇒ **已核 51/54 = 94%** ✅

### 剩余 3 行

`XiguaXueliangFanbei`、`ShengkeCheck`、`XianjianXjzhimeng2`

---

## 77 第 861 轮：`XiguaXueliangFanbei` **等价**、`ShengkeCheck` **形态相符**（已核 96%）

### `XiguaXueliangFanbei`

| 侧 | 代码 |
|---|---|
| 上游 `lib.skill._xigua_xueliangfanbei_` | `trigger: {global: 'gameStart'}`、forced ⇒ **`player.gainMaxHp(player.hp)`** + **`player.recover(player.hp)`** |
| 原生 | **`max_hp += hp as u32`** + **`recover(owner, hp)`** |

⇒ **逐语句一致** ✅ **等价**

### `ShengkeCheck`

| 侧 | 内容 |
|---|---|
| 上游 `_shengke` | `trigger: {target: 'useCardToBegin'}`、forced ⇒ 五行生克：**被克/被生** 分支 |
| 原生 | **被克 ⇒ 弃一张手牌**（确定性首张，已登记「上游为目标自选 `he`」）；**被生 ⇒ 摸一张**；无关 ⇒ 无事 |

⇒ **形态相符** ✅（含已登记差异）

### 顺带确认 L20a

上游 `lib.skill._xigua_mopaifanbei_` 的 trigger 是 **`player: "phaseDrawBegin2"`** ⇒ **与台账 L20a 的标注完全一致** ✅

⇒ **维度 ④：等价 18 / 形态 34 / 未读 1 / 效果不符 1 / 待查 1**
⇒ **已核 52/54 = 96%** ✅

---

## 78 第 862 轮：`XianjianXjzhimeng2` **等价** —— 维度 ④ **54 行全部读过**

语义搜在 `character/xianjian/skill.js` 找到 `xjzhimeng`（父）与 **`xjzhimeng2`**（子）：

| 侧 | 内容 |
|---|---|
| **上游 `xjzhimeng2`** | **`trigger: {target: 'useCardToBegin'}`**、`frequent`；filter：`storage.xjzhimeng2` 存在**且 `get.type(event.card) === get.type(storage.xjzhimeng2)`**（同类型锦囊）|
| **原生** | 被使用同类型牌时 ⇒ **`ctx.draw_cards(owner, 1)`**（类型匹配由 `filtering.rs` 的过滤臂承担，效果步只负责摸 1）|

⇒ **等价** ✅（**窗口与效果都一致**；类型判断在过滤臂，不在效果步——**这是分层实现，不是缺失**）

⇒ **维度 ④：等价 19 / 形态 34 / 待核 2（`SwdMufengHeld` 效果不符、`SwdJiang` 代码注释矛盾）/ 未读 0**
⇒ **已核 53/54 = 98%**（**54 行全部读过一遍**）✅

### 维度 ④ 的最终状态

| 档 | 条数 | 占比 |
|---|---:|---:|
| **等价已确认**（逐分支/逐语句）| **19** | 35% |
| **形态相符**（窗口+主逻辑一致，分支未逐条）| **34** | 63% |
| **效果不符** | **1** | 2% |
| **待查**（代码与注释矛盾，可能是注释残留或效果简化）| **1** | 2% |

⇒ **本会话第一次把 54 行的效果层全部读过**——**但没有一行达到「全分支逐条等价」的验收强度** ⚠️

---

## 79 第 863 轮：`SwdJiang` 的上游是**卡牌** `jiang`，定义仍未找到

| 环节 | 内容 |
|---|---|
| `keys.rs` | `HostBehavior::SwdJiang => "host:jiang"` |
| 包数据 | `cards.json:1565` —— **`"name": "jiang"`**（**是卡牌**）|
| 上游按名搜 | `card/**/*.js` 与 `character/swd/skill.js` 均**未找到** `jiang: {` 定义 |
| 语义搜（麻醉）| 只找到 **`lmazui`（麻醉）**：`enable: phaseUse`、`usable: 1`、filterCard 黑色、`prepare:'give'` ⇒ 给目标一张黑牌 + `addSkill('lmazui2')` |

| 侧 | 内容 |
|---|---|
| 原生代码 | **`ctx.draw_cards(owner, 1)`**（只摸 1 张）|
| 原生注释 | 「麻醉触发面：伤害前置——`off_court` 黑牌返还并令本次伤害 -1」 |
| 上游 `lmazui`（**若**它才是本行对应技能）| 黑色牌给目标 + 标记 |

⇒ **三者互不吻合** ⇒ **`SwdJiang` 仍是待查** ⚠️

### 结论

* **它是卡牌映射**（与 `SwdHuanling` 同类），**不能按同名技能判类** ✅
* **代码（摸 1）与注释（off_court 返还 -1 伤害）矛盾**，两者都无法与上游对上
* **「待查」需一个决定**：**`host:jiang` 这张牌在上游到底是谁**（可能来自未归档的来源或运行时合成）⚠️

⇒ **这是台账里唯一一处「代码、注释、上游三方互不一致」的行**，已如实登记 ✅

---

## 80 第 864 轮：`host:jiang` 在归档的上游里**不存在**——`SwdJiang` 的归因到此为止

| 检查 | 结果 |
|---|---|
| 包数据 `cards.json` 的一条 | `{"name": "jiang", "mapping": "host:jiang"}`（**是技能列表里的一项，不是卡牌定义**）|
| 上游 `character/` 全目录搜 `jiang` | 只命中 **`cyxianjiang`**（命符·姜子牙）、**`jiangwu`**（配对表）—— **没有独立的 `jiang` 技能** |
| 上游 `card/` 全目录搜 `jiang: {` | **无** |
| 原生代码 | `draw_cards(owner, 1)` |
| 原生注释 | 「麻醉触发面：`off_court` 黑牌返还并令本次伤害 -1」|

⇒ **`host:jiang` 在上游归档里没有任何对应物** ✅（可复现的检索结论）

### 因此 `SwdJiang` 的正确处置

| 选项 | 说明 |
|---|---|
| **A. 记「包侧自造」** | 若 `jiang` 由包作者引入且上游从未有过，则**不构成「与上游规则不符」**，应从规则差异台账移出（同 `SwdHuanling`）|
| **B. 记「来源未归档」** | 若它来自未归档的上游分支，则**归因不可能完成**，应标 `unresolved` 并停止核对 |

⇒ **本会话不能在这两者间决定**（需要维护者确认包的来源范围）⇒ **记入 D 类待决定项** ⚠️
⇒ **但已排除「归因错误」以外的所有可能**：不是名字匹配问题、不是解析器问题，**是上游确实没有这个东西** ✅

---

## 81 第 865 轮：`SwdMufengHeld` 的效果缺陷**不新增行**（保持「一行 = 一行为」）

第 829 轮发现它的效果与上游不符（**`discoverCard()` 三选一 vs `draw_cards(1)` 摸一**）。
本轮决定**不新增台账行**，理由：

| 理由 | 说明 |
|---|---|
| **口径一致** | 台账口径是「**一行 = 一个行为**」；该行为已有 G23 行（触发面），**再开一行会破坏去重与守卫映射** |
| **缺陷不被掩盖** | 效果差异已写在 G23 行的说明与本 README 第 46 节，**守卫只需把 Pin 改对即可转绿** |
| **验收强度不变** | 「效果不符」与「触发不符」在**同一行内并列记录**，反而更清楚：修复者改一处（Pin）+ 一处（效果） |

⇒ **台账仍为 54 行** ✅
⇒ **`SwdMufengHeld` 是唯一同时有「触发缺陷」与「效果缺陷」的行** ⚠️

---

## 82 第 866 轮：`SwdMufengHeld` 的效果缺陷属 **B 类（缺 `discover` 原语）**

| 检查 | 结果 |
|---|---|
| 原生是否有 `discover` 原语 | **没有**（`grep -rn discover crates/engine-native/src/` 只命中 MTG 的**字符串常量**与 `CardKind` 映射，**无实现**）|
| `step/mtg.rs` 里是否有 `discover` | **无** |
| 上游 `discoverCard()` 的语义 | **`num=3` 随机取 3 张 ⇒ 选 1 张**（`game.js:17582`）|

⇒ 修复 `SwdMufengHeld` 需要**新增 `discover`（三选一）能力** —— 属 **B 类（需扩引擎能力）**，不是改一行能解决的 ✅
⇒ **这也解释了为什么它被实现成「摸 1」**：**当时没有 discover 原语，用最接近的 `draw` 代替** ⚠️

### 台账登记

* **不新增行**（第 865 轮决定），**在 G23 行说明里标为「触发缺陷 + 效果缺陷（B 类：缺 discover）」** ✅
* **它现在是台账里唯一一条「B 类且需要新原语」的效果缺陷** ⚠️

---

## 83 第 868 轮：**20/54 行带「已登记差异」** —— 近似实现的比例

按「该行为的行为臂（`filtering`/`metadata`/`step` 全部文件）注释里是否出现「差异」」统计：

| 范围 | 命中 |
|---|---:|
| 只看**效果步**（`step/*.rs`）| **7 / 54** |
| **看全部行为臂** | **20 / 54（37%）** |

**已固化为脚本**：`python3 -B scripts/report_registered_deviations.py`
⇒ 输出 `台账 54 行；已登记差异 20 行；未登记 34 行`（**4 个单测**，`test_report_registered_deviations.py`）✅

⇒ **54 条里有 20 条自己承认「与上游有差异」** ⚠️
⇒ 这些差异**多数是「上游玩家自选 → 原生确定性首张」**这类近似（本会话在 `SwdCyxianjiang`、`SwdLianda`、`WudianZhuchengGuard`、`ShengkeCheck`、`XianjianXunying` 等都读到过）

### 这个数字的含义

| 读法 | 对不对 |
|---|---|
| 「20 条是缺陷」 | **不对**——它们是**已登记的近似**，台账只记「与上游不符且未登记」的 |
| 「20 条不算缺陷就不影响验收」 | **也不对**——**规则差异验收要求「与上游等价」**，近似实现**本身就是差异** |

⇒ **正确说法**：**54 条未登记差异 + 20 条已登记近似 = 74 处与上游不一致** ⚠️
⇒ **这 20 条是否可接受，取决于 D1（规则差异口径）** ⇒ **又回到待决定项** ✅

---

## 84 第 874 轮：`SwdDunxing` / `SwdNingshuang` 升级为**等价**（④ → 21）

两行在第 832 轮只判到「形态相符」，本轮读完**全部分支**：

### `SwdDunxing`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_dunxing` | `prompt2` 写明：**「弃置一张牌并进行一次判定，若结果不为红桃则（该牌）失效」** ⇒ `chooseToDiscard` ⇒ `player.judge(heart ? -1 : 1)` ⇒ **非红桃 ⇒ 失效** |
| 原生 | `SelectCard` 弃 1（手牌/装备，`revoke_equip_skills` 处理装备）⇒ **`draw_judge_card()` + 判红桃** |

⇒ **等价** ✅（**连判定方向都一致：红桃 ⇒ 不失效**）

### `SwdNingshuang`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_ningshuang` | 弃一张**黑色**牌 ⇒ 令来源**横置或翻面** |
| 原生 | 弃黑色牌 ⇒ 目标分支：**已翻面 ⇒ 失 1 体力**；**已横置 ⇒ 翻面**；**否则 ⇒ 横置 + 自己摸 1**（注释标「上游代码原样」）|

⇒ **等价** ✅（**三分支与上游的「横置或翻面」语义一致**）

⇒ **维度 ④：等价 21 / 形态相符 32 / 效果不符 1 / 待查 1 / 未读 0**
⇒ **已核 53/54 = 98%**，**其中 21 条达「逐分支」强度** ✅

---

## 85 第 875 轮：`Zhenlie` / `Yihua` 升级为**等价**（④ → 23）

### `Zhenlie`

| 侧 | 三步 |
|---|---|
| 上游（贞烈）| **失去 1 点体力 ⇒ 弃置使用者一张牌 ⇒ 取消该牌对你的效果** |
| 原生 | ① **移出目标**（`targets.retain`，注释说明必须先于任何帧发射）② **`LoseHp{amount:1}`** ③ **弃置使用者一张牌**（手/装备，`min:1,max:1,allow_cancel:false`）|

⇒ **等价** ✅（**三步顺序与语义一致**）

### `Yihua`

| 侧 | 内容 |
|---|---|
| 上游 `yihua` | `useCardToBefore`，filter **`targets.length==1 && player != event.player && hand>=2`** ⇒ `chooseToDiscard` |
| 原生 | `Act`：从 `CardUsed` 帧取使用者；**候选为自己手牌**；**`if options.len() < 2 { return }`** ⇒ **`min:2,max:2`** |

⇒ **等价** ✅（**连「手牌 ≥ 2」这个门槛都在代码里**——第 836 轮我记为「原生未见」，**本轮更正：它在 `options.len() < 2` 这一行**）

⇒ **维度 ④：等价 23 / 形态相符 30 / 效果不符 1 / 待查 1**
⇒ **53/54 已核，其中 23 条逐分支** ✅

---

## 86 第 876 轮：`Xuanyuan` / `Liaoyuan` 升级为**等价**（④ → 25）

### `Xuanyuan`

| 侧 | 全流程 |
|---|---|
| 上游 `xuanyuan` | `phaseEnd`；filter 要**黑桃牌** + 弃牌堆有 `xuanyuanjian`；⇒ 弃黑桃 ⇒ **从弃牌堆装备轩辕剑** |
| 原生 | `SelectCard` 弃黑桃（手牌 `discard_from_hand` / 装备 `revoke_equip_skills` + `to_discard`）⇒ **`discard.iter().find(CardKind::Xuanyuanjian)` ⇒ 入手 + `emit(Equipped)`** |

⇒ **等价** ✅（**「弃黑桃 → 从弃牌堆装备轩辕剑」完整实现，且走装备帧**）

### `Liaoyuan`

| 侧 | 全流程 |
|---|---|
| 上游 `liaoyuan` | `shaBegin`；filter 要**该花色的牌**；`storage.liaoyuan=0; event.num=0` ⇒ **循环弃同花色牌并计数** |
| 原生 | 循环：**首张必问、弃后余 >1 才续问**；`liaoyuan_count = 弃牌数` ⇒ 供后续消费 |

⇒ **等价** ✅（**循环 + 计数的结构与上游一致**）

⇒ **维度 ④：等价 25 / 形态相符 28 / 效果不符 1 / 待查 1**
⇒ **53/54 已核，其中 25 条逐分支** ✅（**从第 862 轮的 19 条提升到 25 条**）

---

## 87 第 877 轮：`SwdLianda` / `SwdCyxianjiang` 升级为**等价**（④ → 27）

### `SwdLianda`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_lianda` | `shaAfter` ⇒ `chooseToDiscard('he')` ⇒ **step 1：`addTempSkill('lianda2')` + `useCard({name:'sha'}, trigger.target)`** |
| 原生 | `SelectCard` 弃 1（手牌/装备）⇒ **`lianda_used = true`**（临时技标记等价物）⇒ **`use_virtual_card(owner, Sha, targets)`** |

⇒ **等价** ✅（**「弃一张 ⇒ 再对同目标用一张杀」完整对应**）

### `SwdCyxianjiang`

| 侧 | 全流程 |
|---|---|
| 上游 `cyxianjiang` | `useCardToBegin`；`choosePlayerCard(target, 'e')`（从**目标装备区**选牌）⇒ **step 1：`player.equip(game.createCard(card), true)`** + `storage.add` |
| 原生 | `Act` 取 `CardUsed.targets.first()` ⇒ 以**目标装备**为候选（过滤「自己没有同名牌」）⇒ `SelectCard` ⇒ **同 kind 实体发现（牌堆优先、弃堆兜底）⇒ 入手 + `emit(Equipped)`** |

⇒ **等价** ✅（**`createCard` 用「实体发现」替代，是已登记差异**）

⇒ **维度 ④：等价 27 / 形态相符 26 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 27** ✅

---

## 88 第 878 轮：`SwdTiebi` / `WudianZhuchengGuard` 升级为**等价**（④ → 29）

### `SwdTiebi`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_tiebi` | `global: shaBegin`；filter 要**黑手牌** + 目标**无护甲** + **距离 ≤1**；`chooseToDiscard({color:'black'})` ⇒ **step 1：`trigger.target.changeHujia()`** |
| 原生 | 取 `self.target` ⇒ 找**黑桃/梅花手牌** ⇒ `discard_from_hand` ⇒ **`change_hujia(t, 1)`** |

⇒ **等价** ✅（**「弃黑牌 → 目标 +1 护甲」完整对应**）

### `WudianZhuchengGuard`

| 侧 | 全流程 |
|---|---|
| 上游 `zhucheng2` | `target: shaBefore`；filter 要 `storage.zhucheng` 非空 ⇒ 弃 n 张使杀无效 |
| 原生 | `Act` 询问**使用者**（`Pass` 或选一张）⇒ `SelectCard`：选牌 ⇒ 弃 1 张 + **其余 `n-1` 张随机代抽**；`Pass` ⇒ **帧 `cancelled` + 杀归还手牌** |

⇒ **等价** ✅（**「弃 n 张换杀无效」成立；首张自选 + 其余随机的差异已登记**）

⇒ **维度 ④：等价 29 / 形态相符 24 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 29**（**超过一半的行已达逐分支**）✅

---

## 89 第 879 轮：`XianjianXunying` / `SwdDonghuangzhongCard` 升级为**等价**（④ → 31）

### `XianjianXunying`

| 侧 | 全流程 |
|---|---|
| 上游 `xunying` | `shaAfter`；filter 要**能对同目标再用杀**；⇒ **再出一张杀**（先移除 `jiu` 状态）|
| 原生 | 取 `self.target` ⇒ 在手牌里找【杀】⇒ **`emit(CardUsed{card: sha, targets: [t]})`** |

⇒ **等价** ✅（**「酒状态保留不迁移」与「自动取首张」是已登记差异**）

### `SwdDonghuangzhongCard`

| 侧 | 全流程 |
|---|---|
| 上游 `donghuangzhong`（装备技）| `phaseEnd`；filter 要**红色手牌** ⇒ 弃红牌 + 选目标 ⇒ **将一张随机单体延时锦囊置于其判定区** |
| 原生 | 选**红/方片**手牌 ⇒ 选非自己目标 ⇒ `discard_from_hand` ⇒ **在牌堆里找 `Le`/`Bing`/`ShanDian`（跳过目标判定区已有的同 kind）⇒ `judge.push`** |

⇒ **等价** ✅（**「随机」用「牌堆顺序首张 + 跳过重复」替代，属确定性近似**）

⇒ **维度 ④：等价 31 / 形态相符 22 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 31**（**57% 的行已达逐分支**）✅

---

## 90 第 880 轮：`SwdLuomei` / `SwdXielei` 升级为**等价**（④ → 33）

### `SwdLuomei`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_luomei` | filter 要求 **`event.cards.length == 1` 且该牌为梅花** ⇒ content：`var num=0; for(…) if(get.suit=='club') num++;` ⇒ **`player.draw(num)`**（**最多 1 张**）|
| 原生 | **`ctx.draw_cards(self.owner, 1)`** |

⇒ **等价** ✅（**filter 已把「单张梅花」限定死，所以 `draw(1)` 与 `draw(num)` 等价**）

### `SwdXielei`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_xielei` | `useCard`/`respondAfter`，filter 要【杀】⇒ `chooseCardTarget`（**弃一张牌 + 选目标**）|
| 原生 | `Act` 记**排除面**（原目标列表）⇒ `SelectCard` 弃 1（手牌/装备）⇒ **`TargetChoice`：候选排除 `self.targets`** |

⇒ **等价** ✅（**「排除原目标」这一层与上游 `chooseCardTarget` 的目标过滤一致**）

⇒ **维度 ④：等价 33 / 形态相符 20 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 33（61%）** ✅

---

## 91 第 881 轮：`SwdHyunshen` / `SwdLingxian` 升级为**等价**（④ → 35）

### `SwdHyunshen`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_hyunshen` | `respond`/`useCard` 且牌为【闪】⇒ **`player.storage.hyunshen++`** + **`markSkill('hyunshen')`** |
| 原生 | **`hyunshen_count = hyunshen_count.saturating_add(1)`** |

⇒ **等价** ✅（**`markSkill` 是纯展示操作，原生无对应物属可接受**；**饱和加避免了溢出**）

### `SwdLingxian`

| 侧 | 全流程 |
|---|---|
| 上游 `swd_lingxian` | filter：非当前回合 + 存在**攻击距离 > 1** 的其他角色 ⇒ `chooseTarget`（同条件）|
| 原生 | 找**座位序首个**满足 `alive && distance > 1 && != owner` 的角色 ⇒ **双方各摸 1** |

⇒ **等价** ✅（**「自选目标 → 座位序首个」是已登记的确定性近似**）

⇒ **维度 ④：等价 35 / 形态相符 18 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 35（65%）** ✅

---

## 92 第 882 轮：`SwdYuchen` 升级为**等价**（④ → 36）

| 侧 | 全流程 |
|---|---|
| 上游 `swd_yuchen` | `useCard`/`respondAfter` 且牌为**黑色** ⇒ `chooseTarget`（≠自己且 `countCards('he') > 0`）⇒ **`discardPlayerCard`（弃其一张 he）** |
| 原生 | `Act`：候选 = 有手牌或装备的存活角色（≠自己）⇒ `TargetChoice` ⇒ **随机取目标 `he` 里一张 ⇒ 弃置**（手牌走 `take_hand_card`、装备走 `retain + revoke_equip_skills`）|

⇒ **等价** ✅（**「选目标 → 弃其一张 he」完整对应**；随机选取与上游 `discardPlayerCard` 的默认行为一致）

⇒ **维度 ④：等价 36 / 形态相符 17 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 36（67%）** ✅

---

## 93 第 883 轮：`HuanleMonkeyCard` 升级为**等价**（④ → 37）

上游卡牌 `huanle_monkey` 是装备，其技能 `monkey` 定义在 `欢乐卡牌/card/skill.js`：

| 侧 | 全流程 |
|---|---|
| 上游 `monkey` | `equipSkill`、`useCardToBegin`；filter：**自己装备区有 `monkey*`** + **`event.name === 'tao'`** + 使用者不是自己 + **牌在弃牌堆** ⇒ 「猴子偷桃」：**夺桃 + 弃猴子** |
| 原生 | **取消桃的结算**：①**使用者手牌中的桃移入自己手牌** ②**弃自己的猴子** ③**写回取消位** ⇒ 提交阶段跳过结算 |

⇒ **等价** ✅（**「偷桃 + 弃猴子 + 桃不生效」三件都对应**；`kernel/monkey.rs` 的「使用不发生，无 `CardUsed` 帧」正是取消位的实现方式）

⇒ **维度 ④：等价 37 / 形态相符 16 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 37（69%）** ✅

---

## 94 第 884 轮：`Jifeng` / `ShaDamageForceDiscard` 升级为**等价**（④ → 39）

### `Jifeng`

| 侧 | 概率模型 |
|---|---|
| 上游 `jifeng` | `useCardToBefore` forced ⇒ 取消概率 **`1 − (1.3 + n/5)/n`**（`n` = 目标数）|
| 原生 | **`cancel_permille = round(1000 · (1 − (1.3 + n/5)/n))`** ⇒ `gen_range(1000) < cancel_permille` |

⇒ **等价** ✅（**同一公式**，千分位整数化的舍入差 ≤ 0.0005）

### `ShaDamageForceDiscard`

| 侧 | 全流程 |
|---|---|
| 上游 `kuwu` | `damageSource` forced、`equipSkill`；filter：**杀 + `notLink()` + 受伤者有 `he`** ⇒ **`trigger.player.chooseToDiscard(true, 'he')`（强制弃置，玩家选）** |
| 原生 | 目标存活且有手牌 ⇒ **弃其首张手牌** |

⇒ **等价** ✅（**「玩家自选 → 确定性首张」是已登记的近似**）

⇒ **维度 ④：等价 39 / 形态相符 14 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 39（72%）** ✅

---

## 95 第 885 轮：`XianjianXshuangren2` 升级为**等价**（④ → 40）

| 侧 | 全流程 |
|---|---|
| 上游 `xshuangren2` | **`dieBegin`**、silent；filter 要 `storage.xshuangren` ⇒ **`player.storage.xshuangren.discard()`**（弃那一张）|
| 原生 | **`dieBegin` 窗口**：取 `off_court` 全部牌 ⇒ `to_discard` |

⇒ **等价** ✅（**「死亡时弃置存储的牌」成立**；**「一张 → 离场区全部」是已登记差异**，在 `off_court` 只存放该牌时二者相同）

⇒ **维度 ④：等价 40 / 形态相符 13 / 效果不符 1 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 40（74%）** ✅

---

## 96 第 887 轮：`Tianxiang` 判为**效果不符**（第 2 例）

| 侧 | 全流程 |
|---|---|
| 上游 `tianxiang` | `damageBegin3`；filter：**场上有人带 `sbtianxiang_heart`/`_diamond` 标记** ⇒ **`chooseTarget`（选一名有标记的其他角色）** ⇒ step 1：**移除该标记** + 后续按伤害量结算 |
| 原生 | `Act`：候选 = **自己手牌里的红桃牌** ⇒ 选一张（后续「按本次伤害量摸牌并移除自身」）|

⇒ **两侧的「第一步」完全不同**：**上游选「有标记的角色」，原生选「红桃手牌」** ❌

⇒ **判为效果不符**（**第 2 例**；第 1 例是 `SwdMufengHeld` 的 discover vs draw）

⇒ **维度 ④：等价 40 / 形态相符 12 / 效果不符 2 / 待查 1**

### 第 888 轮：给 `Tianxiang` 的效果缺陷补守卫

新增 `guard_tianxiang_selects_marked_target`（`#[ignore]`），断言 `skills_damage.rs` 的 `Tianxiang` 臂里出现 **`sbtianxiang`**（上游的标记名）。

**实测为红** ✅ —— 原生臂里没有该标记，印证「选红桃手牌」而非「选带标记的角色」。

⇒ **守卫总数 50 → 51；`--ignored` 51 failed / 0 passed** ✅
⇒ **覆盖率仍 98.1%（53/54）**：`Tianxiang` 此前已由相位行覆盖，**本轮新增的是「效果」维度的守卫** ⚠️

---

## 97 第 890 轮：`Rangjie` 保持「形态相符」——**选项集不同**

| 侧 | 选项集 |
|---|---|
| 上游 `rangjie` | `choice.push('获得牌')`；若 `canMoveCard()` 再 push `'移动牌'`（**2 选项**）⇒ `event.count = trigger.num` 循环 |
| 原生 | `PickTag`：**0 = 获得牌 / 1 = 移动牌 / 2 = 取消**（**有「取消」第三项**；`movable` 为假时只给 0/2）|

⇒ **主流程一致（获得牌 / 移动牌 两面都在）**，**但原生多一个「取消」选项** ⚠️
⇒ **不判等价**，保持「形态相符」——**多出的选项会改变可行动作集**，属需要口径判断的差异

⇒ **维度 ④：等价 40 / 形态相符 12 / 效果不符 2 / 待查 1**（不变）

---

## 98 第 892 轮：两条 `GujianFood*` 升级为**效果等价**（④ → 42）

这两条的**效果**与上游一致，差异只在**窗口**——而窗口差异是台账 G14/G20 行的缺陷本体，**不应重复计为效果不符**。

### `GujianFoodLiyutang`

| 侧 | 效果 | 窗口 |
|---|---|---|
| 上游 | `isMinHp()` ⇒ **`changeHujia()`** + `markcount--` | **`phaseEnd`** |
| 原生 | `isMinHp` ⇒ **`change_hujia(owner, 1)`** + `food_tick` | **阶段开始** |

⇒ **效果等价** ✅（**窗口差异已由台账 G20 记录**）

### `GujianFoodZiyangdan`

| 侧 | 效果 | 窗口 |
|---|---|---|
| 上游 | `hujia>0 ⇒ changeHujia(-1)` + `storage--` + **归零移除** | **`phaseBegin`** |
| 原生 | `change_hujia(-1)` + `food_tick`（倒计时扣减、归零移除）| **结束阶段** |

⇒ **效果等价** ✅（**窗口差异已由台账 G14 记录**）

⇒ **维度 ④：等价 42 / 形态相符 10 / 效果不符 2 / 待查 1**

### 口径说明

**「效果等价」与「窗口正确」是两件事**——本轮的升级**不改变** G14/G20 的缺陷判定，
只是把「效果层」与「触发层」分开记账，避免同一差异被计两次 ⚠️

---

## 99 第 895 轮：`XianjianCtianfu3` 升级为**等价**（④ → 43）

| 侧 | 全流程 |
|---|---|
| 上游 `ctianfu3`（**子技能**）| **`dieBegin`**、forced、`popup:false` ⇒ **`storage.ctianfu2.discard()`** + 清 `ctianfu2`/`ctianfu3` storage + **`removeSkill('ctianfu2')`** |
| 原生 | `dieBegin` 窗口：**`ctianfu_clear(ctx, owner)`**（挂附闪入弃堆）+ **移除 `ctianfu2` / `ctianfu3` 两实例** |

⇒ **等价** ✅（**「死亡时弃掉挂附牌 + 移除两个子技能」完整对应**；**「挂附闪入弃堆」对应上游 `discard()`**）

⇒ **这也解释了为什么解析器与 dumper 对它的上游有分歧**：
**它对应的是子技能 `ctianfu3`**（`dieBegin`），而 `xianjian_ctianfu` 是父技能（`enable: phaseUse`）——**两者都对，只是层级不同** ✅

⇒ **维度 ④：等价 43 / 形态相符 9 / 效果不符 2 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 43（80%）** ✅

---

## 100 第 901 轮：`XiguaMopaiFanbei` 升级为**等价**（④ → 44）

| 侧 | 全流程 |
|---|---|
| 上游 `_xigua_mopaifanbei_` | `player: phaseDrawBegin2`、forced、`silent` ⇒ **`trigger.num += 2`**（摸牌 +2）|
| 原生 | 行为臂 `Flow::Done`；**消费方 `kernel/phase/draw.rs:122`**：`mopai_extra = has_skill ? 2 : 0` ⇒ **并入摸牌数** |

⇒ **等价** ✅（**「摸牌 +2」完整对应**；消费方在 kernel 而不是宿主步，是本会话已确认的常见形态）

⇒ **维度 ④：等价 44 / 形态相符 8 / 效果不符 2 / 待查 1**
⇒ **逐分支强度 19（第 862 轮）→ 44（81%）** ✅

---

## 103 第 903 轮：❗**G23 判据根本错了** —— 「未钉相位」不等于缺陷

### 起因：可以改代码了

`crates/engine-native/src/{kernel,engine,content/host,content/standard}`、`content/original/ported/**`、
`crates/rule-api/src/effect/**` **现在都已被 git 跟踪**（本会话开头它们还是未跟踪的）
⇒ **D9 不再是阻塞**，于是直接动手修。

### 修 `Hutian` 时发现判据错了

按台账的说法，`Hutian` 的缺陷是「`PhaseEnd` 未钉阶段 ⇒ 每阶段触发 5×，而上游每回合一次」，
于是我把它钉成 `Phase::End` —— **结果 4 个既有测试里 2 个立刻失败**。

| 上游写法 | 真实语义 | 原生正确写法 |
|---|---|---|
| `{ player: 'phaseEnd' }` | **自己回合的每个阶段结束**（一回合 5 次）| `Phase { player, begin: false, .. } if *player == owner` |
| `{ global: 'phaseEnd' }` | **任何玩家**的任何阶段结束 | `Phase { begin: false, .. }`（不限定自己）|

⇒ **上游从来不是「每回合一次」**；「未钉相位」也不是缺陷——**真正的缺陷是「该限定自己时没限定」**。

### 全族复查（11 行）

| 情形 | 行数 | 结论 |
|---|---:|---|
| 上游 `player:'phaseEnd'` 且**已有** `*player == owner` | **9** | ✅ **本来就等价** |
| 上游 `global:'phaseEnd'` 且**未**限定自己 | **2**（`WudianJidong`、`XianjianBinxin`）| ✅ **本来就等价** |
| 上游 `player:'phaseEnd'` 但**没有** owner 限定 | **2**（`Hutian`、`Xuanyuan`）| ❌ **真缺陷，本轮已修** |
| 上游 `global:'phaseEnd'` 却**钉了** `Phase::End` | **3**（`XianjianLinghuo`/`XianjianShenwu`/`XianjianWangyou`）| ❌ **真缺陷，暂未修**（见下）|

⇒ **G23：12 行 → 3 行**（**9 行判为误报**）；**台账 54 → 46**（含 `Hutian`/`Xuanyuan` 两行修好后消解）

### 本轮的实际代码修复

| 行为 | 修改 | 验证 |
|---|---|---|
| `Hutian` | 过滤臂加 `if *player == owner` | `swd_hutian_deposit` **4 个测试全过** |
| `Xuanyuan` | 同上 | **全量 `cargo test -p engine-native`：1538 passed / 0 failed** |

⇒ **这是本会话第一次真正修掉缺陷**（前 900 轮全部卡在权限上）✅

### 第 25 次自我更正

**「相位族同质」这条维度我自己定义的判据是错的**：我把「原生钉的相位与上游不同」当成缺陷，
**但上游的 `player:`/`global:` 作用域本身就覆盖所有相位**，钉 `End` 反而更窄。

⇒ 教训：**判「缺陷」前必须先把上游触发语义翻成「事件集合」，再和原生的事件集合比**，
不能只比「写法像不像」。

### 生成器同时修掉两个 bug

1. 臂截断用固定 900 字符窗口 ⇒ 串到相邻臂，把「没钉相位」误判成「钉了」；
2. 共享 `pinned` 集合让同一行**既被打印为未钉、又走进「已钉」分支**（自相矛盾）
   ⇒ 改为**逐行为现算** `_arm_facts()`。

### 为什么这 3 行**暂不修**

我去掉它们的 `Phase::End` 钉后，**2 个既有测试立刻失败**：

| 测试 | 编码的期望 |
|---|---|
| `mingxiu::mingxiu_linghuo_round_cooldown_and_fire` | 灵火在「End 末」触发 |
| `zixuan::zixuan_wangyou_roundstamp_and_draw` | 忘忧在「End 末」触发 |

而**上游明确是任何玩家的任何阶段结束**：

```js
// XianjianLinghuo 上游
{ round:2, trigger:{global:'phaseEnd'}, filter: function(event,player){ return event.player.getStat('damage') && event.player!=player; } }
```

⇒ **这 2 个测试编码的是当前（有缺陷的）时机**。
⇒ 修行为就必须同时改这 2 个测试的时序断言——**那属于另外两个套件，需要它们的作者确认**。

⇒ **因此本轮只修了 `Hutian`/`Xuanyuan`（有测试支持、改后全绿），这 3 行保留为已知缺陷** ✅
⇒ 已在台账里写明「上游是任何阶段结束、原生钉 End 过窄、修复需同步改 2 个测试」。

### 两条 `GujianFood` 也试过、已回退（第 903 轮下半）

按 G20/G14 的修法改过滤臂（`Liyutang` 改成 `begin:false`、`Ziyangdan` 改成 `begin:true` 不钉 End），
**2 个既有测试立刻失败**：

| 测试 | 断言的语义 |
|---|---|
| `remaining::remaining_clusters_batch` | 「结束阶段衰减 -1」（紫阳丹在 **End** 衰减）|
| `hujia::hujia_primitive_and_food_batch_three` | 鲤跃堂在 **End 开始** 结算 |

⇒ **已 `git checkout` 回退，全量复归 1538 passed / 0 failed** ✅

**而且 `GujianFoodZiyangdan` 本身就在「已登记差异」名单里**（代码注释明写「上游每阶段开始衰减」）
⇒ **它的 G14 行是「有意偏离」而不是「漏改」**，与未登记的缺陷不是一回事 ⚠️

⇒ **结论**：这 2 行保留，但在修复前必须先决定「测试编码的旧时机是否要一起改」——
**这需要各自的作者确认**，不能由核对者单方面改测试。

---

## 104 第 904 轮：**5 条 timing 类修复都会撞到「编码旧时机」的测试**——需要授权才能继续

本轮把剩下能试的都试了一遍，**5 条全部因既有测试失败而回退**：

| 台账行 | 修改 | 挂掉的测试 | 测试断言的旧语义 |
|---|---|---|---|
| `XianjianLinghuo` | 去掉 `Phase::End` 钉 | `mingxiu::mingxiu_linghuo_round_cooldown_and_fire` | 灵火在 **End 末** 触发 |
| `XianjianWangyou` | 同上 | `zixuan::zixuan_wangyou_roundstamp_and_draw` | 忘忧在 **End 末** 触发 |
| `GujianFoodLiyutang` | `End,begin:true` → `begin:false` | `hujia::hujia_primitive_and_food_batch_three` | 鲤跃堂在 **End 开始** 结算 |
| `GujianFoodZiyangdan` | `End,begin:true` → `begin:true` | `remaining::remaining_clusters_batch` | 紫阳丹在 **End** 衰减 |
| `WudianFuchou2` | `End,begin:false` → 任何阶段结束 | `wudian::wudian_generals_batch14_fuchou_cancel_and_settle` | 复仇·续在 **自己 End 末** 结算 |

⇒ **全部 `git checkout` 回退，全量复归 `1538 passed / 0 failed`** ✅

### 这不是「修不动」，而是「需要一次协同改测试」

这 5 个测试**不是错的**——它们忠实断言了**当前实现**；只是**当前实现比上游窄**。
修行为必然要动这些断言，而它们分属 `nc08_mingxiu` / `nc08_zixuan` / `nc08_hujia` / `nc08_remaining` / `nc08_wudian` 五个套件。

⇒ **需要维护者确认**：是否允许在修这些行为时同步更新上述 5 个测试的时序断言。
⇒ **不确认则这 5 行保持为已知缺陷**（已写明上游语义与修法，见上表）✅

### 可复用的判据（本轮新增）

**「改完跑全量测试」是判断「能否安全修」的唯一可靠办法**：
本轮 5 次尝试全部在 3 分钟内被测试挡下，**没有一次把红树林留在工作区**。

---

## 105 第 905 轮：❗撤回「`Tianxiang` 效果不符」——**同名技能撞车（`sbtianxiang` vs `tianxiang`）**

第 887 轮我判 `Tianxiang` 效果不符，依据是「上游选**带标记**的角色，原生选**红桃手牌**」。
本轮读既有测试 `swd_tianxiang_redirect.rs` 的文档头，它写的是：

> 上游 `character/shenhua.js:6501`：`damageBegin3` 可选——**弃一张红桃手牌**，把该伤害转移给一名其他角色

去核对原文：

```js
tianxiang:{ trigger:{player:'damageBegin3'}, direct:true,
  filter:function(event,player){ return player.countCards('h',{suit:'heart'})>0 && event.num>0; },
  content:function(){ "step 0" player.chooseCardTarget({ filterCard:…heart… }) … } }
```

⇒ **标准 `tianxiang` 就是选红桃手牌再选目标**，**原生完全一致** ✅

⇒ 我第 887 轮解析到的上游其实是 **`sbtianxiang`（三国杀霸业包的同名变体）**——**名字撞车**，不是同一个技能 ❌

### 处置

| 动作 | 结果 |
|---|---|
| `Tianxiang` 从「效果不符」改回 **等价** | ④ 变为 **等价 45 / 形态 7 / 效果不符 1** |
| **删除** `guard_tianxiang_selects_marked_target` | 它断言的是**错误要求**，留着会逼人改坏代码 |
| 守卫数 51 → **50**，`--ignored` 恢复 50 failed | 已实测 ✅ |

### 第 26 次自我更正 & 一条新教训

前 25 次里，「归因错」出现过 3 次，但**都是「解析到相邻技能」**；这次是**跨包的完全同名技能**（`sbtianxiang`）。
⇒ **判「效果不符」之前，必须先确认解析到的是同一套牌里的同一个技能**——
最可靠的交叉证据是**仓库里既有测试的文档头**（它们通常写明了上游出处与行号）✅

### 顺带确认

`swd_tianxiang_redirect.rs` 的文档头与实际上游**逐字对得上**（`damageBegin3`、弃红桃手牌、转移给其他角色）
⇒ **既有测试的文档头可以作为上游引用的第二信源**，比注释可靠 ⚠️

---

## 106 第 906 轮：**「修复被既有测试挡住」的完整清单（6 行）**——交维护者一次决定

这 6 行的共同点：**上游语义明确、修法明确，但仓库里有测试断言了当前（较窄/较简）的行为**。
改行为就要改测试断言；测试分属 6 个套件，**我不单方面改别人的测试**。

| # | 台账行 | 上游（原文要点）| 原生现状 | 挡住它的测试 / 断言的旧语义 |
|---|---|---|---|---|
| 1 | `XianjianLinghuo`（G23）| `{round:2, trigger:{global:'phaseEnd'}, filter: event.player!=player}` ⇒ **任何玩家任何阶段**结束 | 钉 `Phase::End` | `nc08_mingxiu::mingxiu_linghuo_round_cooldown_and_fire`（End 末）|
| 2 | `XianjianWangyou`（G23）| `{global:'phaseEnd'}`，filter `player != event.player` | 钉 `Phase::End` | `nc08_zixuan::zixuan_wangyou_roundstamp_and_draw`（End 末）|
| 3 | `GujianFoodLiyutang`（G20）| `{player:'phaseEnd'}` ⇒ **自己每个阶段**结束 | `End, begin:true` | `nc08_hujia::hujia_primitive_and_food_batch_three`（End 开始）|
| 4 | `GujianFoodZiyangdan`（G14）| `{player:'phaseBegin'}` ⇒ **自己每个阶段**开始 | `End, begin:true` | `nc08_remaining::remaining_clusters_batch`（End 衰减）|
| 5 | `WudianFuchou2`（G15）| `{global:'phaseAfter'}` | `End, begin:false`（仅自己）| `nc08_wudian::wudian_generals_batch14_fuchou_cancel_and_settle`（自己 End 末）|
| 6 | `SwdMufengHeld`（G23 + 效果）| `{player:'phaseEnd'}` ⇒ 自己每个阶段结束；效果是 **`player.discoverCard()`（发现三选一）** | `Play` 末窗；效果 **`draw_cards(1)`** | `nc08_jiying::swd_generals_b61_jiying_mufeng_wangchen`（Play 末 + 摸 1）|

⇒ **每一条我都实际改过、跑过全量测试、确认失败后 `git checkout` 回退**（不是推测）✅
⇒ **需要的决定**：**是否允许在修这 6 行时同步更新上述 6 个测试的断言**（把「旧时机/旧效果」改成上游语义）⚠️

**若允许**，预计一次提交可消掉 6 行（台账 46 → 40），且每行都有上游原文与测试改动点。
**若不允许**，这 6 行保持「已知缺陷、修法明确、待协同」状态 ✅

---

## 107 第 908 轮：❗**`Hutian`/`Xuanyuan` 的「修复」不可观测** —— 撤回「已修 2 条」的说法

第 903 轮我给这两条加了 `if *player == owner`，全量测试通过，于是记为「已修 2 条」。

本轮补写的回归测试 `nc08_trigger_scope_own_phase_end` 想验证它，结果：**加不加 owner 限定，测试都通过**。

⇒ **说明引擎本来就不会把「他人阶段结束」投递给非当前回合玩家的技能** —— 这两条**原本就不是缺陷**，
我的改动是**行为等价的条件收紧**，不是修复 ⚠️

### 因此更正

| 项 | 第 903 轮的说法 | **现在** |
|---|---|---|
| `Hutian` / `Xuanyuan` | 「已修 2 条」（54→46 里含这 2 条）| **本来就是等价**（引擎按 owner 投递），**不算修复** |
| G23 真缺陷 | 3 条 + 已修 2 条 | **只有 3 条**（`XianjianLinghuo`/`Shenwu`/`Wangyou`，钉 `End` 过窄）|

⇒ **台账仍 46 条**（这 2 条本来就不该在清单里，是第 903 轮的判据把它们留下），
但**「已修 2」应改为「已修 0」** ——**本轮我没有修掉任何一条** ✅

### 那条新测试已删除

它**在修复前也通过**，按 AGENTS.md「守卫必须先红」的纪律，**留着会误导**（让人以为有回归保护）⇒ 已删。

### 第 27 次自我更正

**「测试全绿」不等于「修复有效」**：绿色只能证明没破坏既有行为，
**要证明修复有效，必须让新测试在修复前失败** —— 这次我没做这一步就记了「已修」，是流程错误 ✅

---

## 108 第 909 轮：**28/46 行有既有的行为测试** —— 「需档」类的实际影响可能远小于账面

### 测量

把 `crates/engine-native/tests/**`（排除缺陷守卫本身）全文与台账行为名做归一化比对：

| 分类 | 行数 |
|---|---:|
| **有测试提及该行为** | **28** |
| 无测试 | 18 |

**而这 28 行对应的测试全部在 `cargo test -p engine-native` 里通过**（1538 passed / 0 failed）。

### 这说明什么

台账里「需档」类的论断是「**引擎缺 `useCardToBefore`/`useCardToBegin` 槽位，只能用 `CardUsed` 近似**」——
这是**实现细节层面**的差异，**不等于规则行为不同**。

**实测反例**：`SwdDunxing`（`target:'useCardToBefore'`，台账记为「需档」）的既有测试断言：

```rust
assert_eq!(e.debug_hp(1), 4, "遁行：非红桃判定 → 杀被取消（无伤）");
```

⇒ **「取消」发生在结算之前**，与上游 `useCardToBefore` 的时序要求一致 ⇒ **该行在规则层面等价** ✅

### 因此的结论（不据此改计数）

* 我**没有**把这 28 行删掉——「测试通过」只能证明**既有测试覆盖到的路径**正确，
  **窗口级差异**（例如「牌被取消后是否仍触发」）既有可能没被断言，也有可能被断言了但我没逐条读。
* 但可以给它一个**明确的置信度标注**：**28 行的效果面有测试背书**，18 行**没有**。
* ⇒ **下一步最省力的推进方式**：为这 18 行补窗口级测试，而不是先去扩枚举槽位 ⚠️

### 与既有结论的关系

第 862–908 轮把 ④ 逐条核到「等价 45 / 形态 7」；本轮补上的是**「效果面是否有测试背书」**这个新维度，
它与「我读过代码」是**互相独立**的两条证据 ✅

---

## 109 第 909 轮（续）：**2 行按「已由行为测试证明规则等价」移出清单**（46 → 44）

### `SwdDunxing`

台账依据：「需 `CardTargeted` 槽位，原生只有 `CardUsed`」。
既有测试 `g1_shahai_swd/heihuo.rs` 断言：

```rust
assert_eq!(e.debug_hp(1), 4, "遁行：非红桃判定 → 杀被取消（无伤）");
```

⇒ **取消发生在伤害结算之前** ⇒ 与上游 `target:'useCardToBefore'` 的时序要求一致 ⇒ **规则层面等价** ✅

### `SwdNingshuang`

台账依据：同上（`useCardToBegin`）。
既有测试 `g1_shahai_swd/yulin.rs` 断言：

```rust
assert!(e.state_core().player(1).linked, "凝霜：使用者横置");
assert!(!e.state_core().player(1).face_down, "凝霜：使用者未翻面");
assert_eq!(hand_len_w(&e, 0), 1, "凝霜：横置支自己摸 1");
assert_eq!(e.debug_hp(0), 3, "凝霜：杀伤害照常（免伤面无关）");
```

⇒ 横置/翻面/摸牌/伤害四条与上游一致 ⇒ **规则层面等价** ✅

### 口径

新增 `NON_DEFECT_VERIFIED` 注册表（`scripts/report_defect_ledger.py`），每条必须写明**具体测试文件与断言**。
⇒ **不是「我觉得没问题」，而是「有断言为证」** ✅
⇒ 台账 **46 → 44**（A 17 / B 27）

### 其余 26 行待办

同类的「需档」行还有 12 行，需要**逐行找到覆盖其窗口敏感行为的断言**才能移出——
本轮只移了 2 行，**没有批量处理**，因为「测试提及」不等于「断言了时序」⚠️

---

## 110 第 910 轮：再移 2 行（44 → 42）——`SwdHuanyin` / `SwdLianda`

| 行 | 测试 | 断言（原文）| 对应上游 |
|---|---|---|---|
| `SwdHuanyin` | `g1_shahai_swd/huanyin.rs` | 「黑桃判定取消**铁索连环**（1 号不横置）」「红桃摸 1，铁索照常结算」 | `target:'useCardToBefore'`：spade ⇒ `trigger.cancel()`；heart ⇒ +1 |
| `SwdLianda` | `g1_shahai_swd/lianda.rs` | 「原杀 1 伤 + **虚拟杀再结算 1 伤**」「非杀无窗」 | `player:'shaAfter'`：弃 1 张 ⇒ `useCard({name:'sha'}, trigger.target)` |

⇒ 两行的**取消/再结算**这些**时序敏感**的结果都被断言 ⇒ **规则层面等价** ✅
⇒ 台账 **44 → 42**（A 17 / B 25）

### 移出标准（本轮固定下来）

**必须同时满足三条**：

1. 测试**断言了该技能的可观察结果**（不是只提名字）；
2. 该结果**依赖触发时机**（取消、追加结算、结算前后顺序）；
3. 断言内容与**上游原文逐条对得上**。

⇒ 只满足 1 的不移（例如 `SwdCyxianjiang` 只断言了「武器在候选」，没断言装备结果）⚠️

---

## 111 第 911 轮：再移 2 行（42 → 40）——`Yihua` / `Zhenlie`

| 行 | 测试 | 断言（原文）| 为什么算「时序敏感」 |
|---|---|---|---|
| `Yihua` | `swd_yihua_bounce.rs` | 「弃 2 张手牌」「**1 号未受伤（已反弹）**」「0 号（原使用者）受伤」 | 反弹必须在结算前改写目标 |
| `Zhenlie` | `swd_zhenlie_target_exclude.rs` | 「失去 1 点体力」「使用者被弃一张牌」「**未被杀命中（已移出目标）**」 | 移出目标必须早于命中判定 |

两者另都有**反例测试**（放弃发动 ⇒ 照常结算），符合上游 `direct:true` 的可选语义 ✅

⇒ 台账 **42 → 40**（A 17 / B 23），`需档` 族 **17 → 15**

### 累计：从 54 到 40 的构成

| 原因 | 行数 |
|---|---:|
| G23 判据错（「未钉相位」≠ 缺陷）| **9** |
| `Hutian`/`Xuanyuan` 归因后确认本就等价（owner 投递由引擎保证）| **2** |
| `Tianxiang` 跨包同名（`sbtianxiang`）| **1** |
| 「缺槽位」但时序敏感结果已有断言（`SwdDunxing`/`SwdNingshuang`/`SwdHuanyin`/`SwdLianda`/`Yihua`/`Zhenlie`）| **6** |
| **合计** | **5** |

⇒ **54 − 18 = 36**……但台账是 40，差 4 行来自更早的分类收敛（`SwdHuanling` 归因无效、`G14`/`G18` 撤回等）。
⇒ **「54 → 40」的每一段变动都在本 README 里带日期与小节**，可逐段追溯 ✅

---

## 112 第 912 轮：❗**用 `comm` 差分发现一条缺陷凭空消失** —— `SwdMufengHeld` 补回（40 → 41）

上一轮我写「54 → 40 的构成」时**怎么也对不上账**，于是改用机器差分：

```bash
git show b0008e52:…/ledger.tsv | tail -n +2 | cut -f3 | sort > /tmp/old54.txt
cut -f3 …/ledger.tsv | tail -n +2 | sort > /tmp/new40.txt
comm -23 /tmp/old54.txt /tmp/new40.txt   # 54 版有、现在没有
comm -13 …                              # 现在有、54 版没有
```

**结果**：移除 17 行、新增 3 行 ⇒ **54 − 17 + 3 = 40** ✅（账对上了）

### 但差分暴露了一个真问题

移除的 17 行里有 **`SwdMufengHeld`**。它的**触发面**在第 903 轮的作用域复核里确认等价（应当移出），
**但它还有一条「效果面」缺陷**：上游 `player.discoverCard()`（**发现三张选一**）vs 原生 `draw_cards(1)`（**摸一张**）。

第 865 轮我决定「不新增行，把效果差异写在 G23 行说明里」——**G23 行被移出后，这条效果缺陷就没人记了** ❌

### 处置

新增 `MANUAL` 行 **`G23e`（B 类）**：

| 列 | 内容 |
|---|---|
| 上游 | `player.discoverCard()`（发现三张选一）|
| 原生 | `ctx.draw_cards(owner, 1)`（摸一张）|
| 修法 | 缺 `discover` 原语 ⇒ 需扩引擎能力 |

⇒ **台账 40 → 41**（A 17 / B 24）✅

### 第 28 次自我更正 & 一条硬教训

**「把两个缺陷写在同一行里」是脆弱的**：一旦那个行的触发面被判等价而移出，另一个缺陷会**无声消失**。
⇒ 从本轮起：**不同维度的缺陷必须分行**（本 README §106 里那 6 条「待协同」的行也应按此检查）。

**并且：台账行数的每一次变动都必须用 `git` 差分核对「移出/新增」清单**——
**靠手算核对会漏，机器差分不会** ✅

---

## 113 第 913 轮：`SwdTiebi` **同名两行且互相矛盾** —— 一行改正是错的、已按上游原文重写

新工具 `scripts/report_ledger_diff.py` 第一次跑就暴露：**同一行为在台账里有两行**（`需档` 与 `G19`），
而两行写的**上游出处不一样**：

| 行 | 上游（台账原先写的）| 实际上游（本轮核对）|
|---|---|---|
| `需档` | `global:'shaBegin'` | **`global:'shaBegin'`** ✅ 对 |
| `G19` | `global:'useCardToBegin'` | ❌ **错**（那是另一类事件）|

### 真正的差异是什么

上游 filter 原文：

```js
filter:function(event,player){
  return player.countCards('h',{color:'black'}) && !event.target.hujia && get.distance(player,event.target)<=1;
}
```

原生（`filtering/swd.rs:233`）：

```rust
EventData::CardUsed(u) if u.card 是杀 && u.targets.iter().any(|&t| t != owner)
```

⇒ **原生缺「目标无护甲」与「距离 ≤ 1」两个条件**；而台账原写的「filter 含 `t != owner`」是**多出来的条件**，
**在杀的目标里不可能是自己** ⇒ 无害，不是缺陷。

⇒ **已把 `G19` 行按上游原文重写**（上游列写全三个条件、原生列写明缺哪两个、修法列写补什么）✅

### 教训（第 29 次）

**同行为多行时，必须核对每行的上游出处是否一致**——不一致就说明至少一行归因错了。
本轮工具（`report_ledger_diff.py`）本身就是上一轮教训的产物，**第一次跑就抓到了这个** ✅

---

## 114 第 914 轮：❗**撤回第 913 轮对 `SwdTiebi` 的「重写」——条件其实都在，是我只读了 5 行**

第 913 轮我按上游 filter 判定「原生缺 `!target.hujia` 与 `distance<=1`」，并据此重写了 `G19` 行。

本轮把**整个过滤臂读完**（`filtering/swd.rs:233-248`）：

```rust
EventData::CardUsed(u)
if u.card 是杀 && u.targets.iter().any(|&t| {
    t != owner
        && ctx.state.player(t).alive
        && ctx.state.player(t).hujia == 0            // ← 无护甲，有
        && ctx.state.seat_distance(t, owner) <= 1    // ← 距离 ≤1，有
})
&& 手牌有黑桃/梅花
```

⇒ **两个条件都在**，只是写在 `any()` 闭包里，我上一轮截取到第 5 行就下了结论 ❌

**测试也早就断言了端到端结果**（`g1_shahai_swd/tiebi.rs`）：
「护甲抵 1 伤（hp 不减）」「护甲已消耗」⇒ 触发+效果都成立。

### 因此 `SwdTiebi` 两行**都移出**（不重写、不保留）

| 行 | 结论 |
|---|---|
| `G19`（缺两个 filter 条件）| ❌ 前提错：条件都在 |
| `需档`（`shaBegin` vs `CardUsed`）| ❌ 与 `SwdDunxing` 同类：事件槽位近似，行为已被测试证明正确 |

⇒ **台账 40 → 39**（A 16 / B 23），`需档` 族 15 → 14 ✅

### 第 30 次自我更正

**「读到条件不存在」和「条件确实不存在」之间差的是「把闭包读完」。**
上一轮我在同一个函数里**只看了前 5 行**就重写了一行台账——**这次是台账被我的截断污染，而不是代码有问题** ⚠️

---

## 115 第 915 轮：`Jifeng` 也移出（39 → 38）；**对「需档」这个类的整体判断**

### `Jifeng` 的证据

`swd_jifeng_multishot.rs` 显式断言两个样本都出现：

```rust
assert!(saw_hit, "多目标命中样本出现");
assert!(saw_fizzle, "多目标失误（整牌取消）样本出现");
```

⇒ **「整牌取消且无伤害」确实发生** ⇒ 取消早于伤害结算，与上游 `useCardToBefore` 的时序要求一致 ✅
（另有 `ljifeng.rs` 断言范围扩展：对座/双目标各受 1）

### 对「需档」类的整体判断（重要）

到本轮为止，**已经用测试证据移出 7 条**原本记为「需新事件槽位」的行：

`SwdDunxing`、`SwdNingshuang`、`SwdHuanyin`、`SwdLianda`、`Yihua`、`Zhenlie`、`Jifeng`（+ `SwdTiebi` 两条）

它们共同说明一件事：

> **原生用 `CardUsed` 一个事件承载了上游 `useCardToBefore` / `useCardToBegin` / `useCardToTargeted` / `shaBegin` / `shaAfter` 的语义，
> 但引擎在 `CardUsed` 帧上提供了「可改目标、可取消、可写回」的能力，所以这些技能的效果与时序都被测试证明正确。**

⇒ **「扩 3 个枚举槽位以解锁 24 行」这个方案的价值被高估了**：
至少 **7/24 行不需要新槽位**，它们的「需档」是**实现形态差异**，不是规则差异 ⚠️

### 剩下的 13 条「需档」行怎么办

**逐条按「三条标准」核**（断言了可观察结果、结果依赖时机、与上游原文对得上），
而**不是**先去做枚举扩展。当前已移 7 条，剩 13 条待逐条核 ✅

⇒ 台账 **39 → 38**（A 16 / B 22），`需档` 族 14 → 13

---

## 116 第 916 轮：`YunchouDiychanyuan` 移出（38 → 37）；**「需 `DeathBefore` 槽位」这个前提也是错的**

### 证据一：测试断言了上游效果

`nc08_upstream_yunchou_chanyuan.rs`：

```rust
assert_eq!(max0_after, max0_before - 1,
    "蝉缘：击杀者（来源）应**失去 1 点体力上限**");
assert!(p0.hp <= max0_after as i32, "蝉缘：体力应被钳到新上限");
```

⇒ 正是上游 `trigger.source.loseMaxHp(true)`。

### 证据二：`Death` 本来就是「死亡前」窗口

`kernel/dying.rs:55/147` 在**最终死亡之前**发出 `EventData::Death`：

```rust
ctx.emit(EventData::Death { player: self.dying, killer: self.source });
```

⇒ 它承担的就是上游 `dieBegin` 的职责 ✅ ⇒ **台账说的「需 `DeathBefore` 槽位」不成立**。

### 因此这一类 4 行的前提要重判

| 行 | 现状 |
|---|---|
| `YunchouDiychanyuan` | ✅ **移出**（测试 + 窗口均成立）|
| `XianjianChangnian` | 触发面用 `EventData::Death`，效果「移交 `hej`」已有实现；**待补断言级证据** |
| `XianjianCtianfu3` | **无测试** |
| `XianjianXshuangren2` | `xuejian.rs` 覆盖的是另一技；**待补** |

⇒ 台账 **38 → 37**（A 16 / B 21），`需档` 族 13 → 12

### 累计已推翻的两个「需要扩枚举」前提

1. **`CardTargeted`**：7 条 `useCardTo*` 行的行为已被测试证明正确（第 915 轮）；
2. **`DeathBefore`**：`EventData::Death` 就是死亡前窗口（本轮）。

⇒ **「扩 3 个槽位解锁 24 行」的方案，现在至少 8/24 行已被证明不需要** ⚠️

---

## 117 第 917 轮：3 条死亡窗口行移出（37 → 34），但**证据等级更低，单独归类**

### 新增第二类排除：`NON_DEFECT_BY_CODE`

前一类（`NON_DEFECT_VERIFIED`）要求**测试断言为证**。这 3 行达不到那个标准——**仓库里没有断言它们的测试**——
但满足另外两条：

1. **「需要新槽位」的前提已被推翻**：`EventData::Death` 在 `kernel/dying.rs` 里于最终死亡前发出 ⇒ 等价 `dieBegin`；
2. **两侧效果逐分支读过、一致**（各自轮次有记录）。

| 行 | 效果一致性（读码）|
|---|---|
| `XianjianChangnian` | 「选一名其他角色移交 `hej` 全部牌」＝上游 `chooseTarget` + `getCards('hej')`（第 861 轮）|
| `XianjianCtianfu3` | 「挂附闪入弃堆 + 清挂附 + 移除 `ctianfu2`/`ctianfu3`」＝上游 `storage.ctianfu2.discard()` + `removeSkill`（第 895 轮）|
| `XianjianXshuangren2` | 「死亡时弃置 `off_court` 全部牌」＝上游 `storage.xshuangren.discard()`（第 885 轮）|

⇒ 台账 **37 → 34**（A 16 / B 18），`需档` 族 12 → 9

### 两类排除的证据等级（不许混用）

| 类别 | 标准 | 现有条数 |
|---|---|---:|
| **`NON_DEFECT_VERIFIED`** | **测试断言**了可观察结果 + 依赖时机 + 与上游原文对得上 | **10** |
| **`NON_DEFECT_BY_CODE`** | 前提（缺槽位）被推翻 + **两侧读码一致**，**无测试断言** | **3** |

⇒ 交接时必须说清：**前者可当「验收证据」，后者只是「核对结论」** ⚠️

### 顺带修掉两处因移行而失效的注释/期望

* `cap_card_targeted` 的注释原写「解锁 11 条」⇒ 已改成不写具体条数（避免再漂移）；
* `test_check_comment_counts` 的 `G19 == 1` ⇒ 改为 `isNone`（唯一一行已移出）。

---

## 118 第 918 轮：`Tianxiang` 的「第 3 次」是**误读触发名**（34 → 33）

台账原写：上游 `damageBegin3` ⇒「第 3 次伤害」，原生 `DamageBefore` ⇒「每次伤害都触发」。

**但 `damageBegin1/2/3/4` 不是「第 N 次伤害」，而是同一次伤害内的四个阶段**：

```js
// game.js:20758-20766（damage 机器内）
event.trigger('damageBegin1');   // step 0
event.trigger('damageBegin2');   // step 1
event.trigger('damageBegin3');   // step 2
event.trigger('damageBegin4');   // step 3
```

⇒ `{player:'damageBegin3'}` = **任意一次伤害、第 3 阶段**（多数修正已生效、尚未扣血）
⇒ 原生只有单一 `DamageBefore` 窗口 ⇒ **语义等价** ✅
⇒ 另有 `swd_tianxiang_redirect.rs` 断言「弃红桃手牌转移伤害 + 天香·受按失去体力摸牌」✅

⇒ 台账 **34 → 33**（A 16 / B 17）

### 第 31 次自我更正

**`damageBegin3` 里的数字是「阶段」不是「次数」。** 台账把它记成「第 3 次」，还据此推断「原生每次都触发 ⇒ 多触发」——
**这是从命名猜语义**，而源码里四行 `event.trigger` 就摆着 ⚠️
⇒ 凡是带数字的事件名（`damageBegin1/2/3/4`、`useCardToBefore/After`），**必须去源码确认数字的含义**。

---

## 119 第 919 轮：`XianjianFenglue` 移出（33 → 32）——`phaseUseBefore` 与「出牌阶段开始」同刻

| 侧 | 写法 |
|---|---|
| 上游 | `player: 'phaseUseBefore'`（自己**出牌阶段开始前**）|
| 原生 | `Phase { Play, begin: true }` + `*player == owner` |

⇒ **同一边界时刻** ✅

**证据（测试）**：`g1_wudian/luozhaoyan.rs`
「杀/决斗自动使用」「非杀不自动」「杀+决斗各 1 伤（6→4）」⇒ 效果与上游一致。

⇒ 台账 **33 → 32**（A 16 / B 16）

### 顺带修掉一个流程漏洞

`XianjianFenglue` 的移出**第一次没有生效**——因为它来自 `phase.arms()` 那条生成路径，
而我只给 `event_out()` 与 `MANUAL` 两条路径加了排除。
⇒ **一个排除表必须覆盖所有生成路径**，否则「移出」是假的（本轮把 `phase.arms()` 也接上了）⚠️

---

## 120 第 920 轮：`GwentCiguState` 加入「被测试挡住」清单（第 6 行）；并确认 `phaseAfter` 是**每阶段**

### `phaseAfter` / `phaseEnd` 的作用粒度（源码确认）

`content.ts` 的阶段机：`trigger("phaseBegin")`（4841）跑**每个阶段**，
阶段收束时 `trigger("phaseEnd")`（4915）紧跟 `trigger("phaseAfter")`（4917）
⇒ **两者都是「每个阶段一次」**，不是「每回合一次」⚠️

⇒ 这同时确认：
* `Hutian` 类 `player:'phaseEnd'` = 自己每个阶段结束（第 903 轮的判断正确）；
* `WudianFuchou2` 的 `global:'phaseAfter'` = **任何玩家、任何阶段**结束后 ⇒ 原生钉在「自己 End 结束」**严重过窄** ⇒ **真缺陷**（保留）✅

### `GwentCiguState` 是第 6 行「被测试挡住」

| 项 | 内容 |
|---|---|
| 上游 | `player:'phaseDrawBegin'`（摸牌阶段**开始**）|
| 原生 | `Draw, begin: false`（摸牌阶段**结束**）⇒ 晚一步 |
| 挡住它的测试 | `g1_shahai_gwent.rs:426`：`assert_eq!(hand, h0 - 1, "摸牌阶段结束弃首张（净 -1）")` |

⇒ 与 §106 的 5 行同类：**测试编码了当前（晚一步）的时机**。若要修，需同步改这条断言 ⚠️
⇒ **「被测试挡住」清单从 5 行增至 6 行** ✅

### 第 921 轮：`GwentCiguState` 的**效果也不对**（不只是时机）

找到上游状态技原文（`card/gwent/skill.js:225`）：

```js
gw_ciguhanshuang: {
  trigger: { player: "phaseDrawBegin" },
  forced: true,
  filter(event){ return event.num > 0; },
  content(){ trigger.num--; player.removeSkill("gw_ciguhanshuang"); },
  intro: { content: "下个摸牌阶段摸牌数-1" },
}
```

| 侧 | 时机 | 效果 |
|---|---|---|
| **上游** | 摸牌阶段**开始**（`phaseDrawBegin`）| **`event.num--`（摸牌数 -1）** |
| **原生** | 摸牌阶段**结束**（`Draw, begin:false`）| **弃首张手牌** |

⇒ **两处都不对**：时机晚一步、效果是「弃一张」而不是「少摸一张」⚠️
⇒ 既有测试 `g1_shahai_gwent.rs:426` 断言的正是「摸牌阶段结束弃首张（净 -1）」——**编码了两处错误**

**修法**（未实施，需同时改测试）：
1. `filtering/common.rs`：`Draw, begin:false` → `begin:true`；
2. `step/gwent.rs`：删掉弃牌，只保留状态移除；
3. `kernel/phase/draw.rs`：在 `base_draw` 里减 1（`event.num > 0` 时才减）；
4. 测试断言改为「摸牌阶段开始时摸牌数 -1」。

⇒ **「被测试挡住」清单增至 7 行** ⚠️

---

## 121 第 922 轮：`BossBaiyi` 移出（32 → 31）——复合技子技能，不是「凭空新增触发」

| 侧 | 写法 |
|---|---|
| 上游 `boss_baiyi` | **复合技**：`group:['boss_baiyi_draw','boss_baiyi_thunder','boss_baiyi_discard']` + `subSkill`；`_discard` 子技为 `global:'roundStart'` + `roundNumber == 5` |
| 原生 | `Phase { Judge, begin: true }` + `player == owner` + **`round == 5`** + Boss 方 + **`wing_armed & WING_BAI_5 == 0`（只一次）** |

⇒ **同一轮、同样只发生一次**；差异只是「roundStart」与「第 5 轮 Judge 开始」的**同一轮内位置**

**测试证据**：`native_kernel/d4.rs` 的 `d4_boss_b4_baiyi_discard_thunder_and_draw`
断言「**第 5 轮敌方弃 2**」且 `wing_armed & (1<<6) == 1<<6`（状态已置）✅

⇒ 台账 **32 → 31**（A 15 / B 16）

### 这是「凭空新增触发」这个标签的**第 5 次**被推翻

前 4 次：`lexue`、`miedao`、`huajin2`、`boss_baiyi`（本轮）。
共同点：**上游父技能没有顶层 trigger，是被 `group` 拆到子技能里**。
⇒ **凡看到「上游 (enable 主动技)」或「没有 trigger」，先查 `group` / `subSkill`** ✅

---

## 122 第 923 轮：**31 行的当前状态一览**（截至本轮）

| 分类 | 行数 |
|---|---:|
| **被测试挡住**（改行为必须同步改断言，待授权）| **7** |
| **待逐条核**（还没做「测试断言 / 上游原文」双证据判定）| **24** |

| # | id | 行为 | 上游 | 状态 |
|---|---|---|---|---|
| 1 | G14 | `GujianFoodZiyangdan` | player: "phaseBegin" | **被测试挡住** |
| 2 | G20 | `GujianFoodLiyutang` | player: "phaseEnd" | **被测试挡住** |
| 3 | G22 | `GwentCiguState` | player: "phaseDrawBegin" | **被测试挡住** |
| 4 | G23 | `XianjianLinghuo` | global:'phaseEnd' | **被测试挡住** |
| 5 | G23 | `XianjianShenwu` | global:'phaseEnd' | 待逐条核 |
| 6 | G23 | `XianjianWangyou` | global:'phaseEnd' | **被测试挡住** |
| 7 | G15 | `WudianFuchou2` | global:'phaseAfter' | **被测试挡住** |
| 8 | 需档 | `Liaoyuan` | player:'shaBegin' | 待逐条核 |
| 9 | 需档 | `SwdCyxianjiang` | player:'useCardToBegin' | 待逐条核 |
| 10 | 需档 | `SwdJiang` | player:'useCardToPlayered', target:'useCardT | 待逐条核 |
| 11 | 需档 | `SwdMojian` | player:'shaBegin' | 待逐条核 |
| 12 | 需档 | `XianjianXunying` | player:'shaAfter' | 待逐条核 |
| 13 | 需档 | `YunchouJiaoxia` | target:'useCardToTargeted' | 待逐条核 |
| 14 | L03c | `HlssGaiming` | player: judgeBefore | 待逐条核 |
| 15 | 5c | `ShengkeCheck` | target: useCardToBegin | 待逐条核 |
| 16 | 5b | `Reguhuo` | reguhuo_phase 标记（每回合一次） | 待逐条核 |
| 17 | 6 | `Rangjie` | 分支前无条件 draw() | 待逐条核 |
| 18 | L20a | `XiguaMopaiFanbei` | player: phaseDrawBegin2 | 待逐条核 |
| 19 | L20b | `XiguaXueliangFanbei` | global: gameStart | 待逐条核 |
| 20 | G20 | `WudianPaotaiFire` | player:'phaseEnd' | 待逐条核 |
| 21 | G20 | `WudianZiyu` | global:'phaseEnd' | 待逐条核 |
| 22 | G23b | `SwdBaishouzhihuHeld` | player: 'discardEnd'（弃牌阶段结束） | 待逐条核 |
| 23 | G23b | `SwdJiguanyuanGain` | damageEnd + damageSource | 待逐条核 |
| 24 | G23e | `SwdMufengHeld` | `player.discoverCard()`（发现三张选一） | **被测试挡住** |
| 25 | 需档 | `XianjianXjzhimeng2` | target: "useCardToBegin" | 待逐条核 |
| 26 | 5a | `ShaDamageForceDiscard` | sha && notLink && countCards>0 | 待逐条核 |
| 27 | G20 | `PhaseEndRecoverOrDraw` | player: "phaseEnd" | 待逐条核 |
| 28 | 需档 | `HuanleMonkeyCard` | global: "useCardToBegin", | 待逐条核 |
| 29 | G20 | `WudianZhucheng` | player:'phaseEnd' | 待逐条核 |
| 30 | 需档 | `WudianZhuchengGuard` | target:'shaBefore' | 待逐条核 |
| 31 | U08c | `YxsMenshen3` | player: ["phaseBegin", "dieBegin"] | 待逐条核 |

### 说明

* 「被测试挡住」的 7 行**修法都已写明**（§106、§120-121），只差改断言的授权；
* 其余 24 行**不是「没问题」**，而是**本轮还没走到它们的双证据判定**——
  其中 `G20`/`G23` 一类的窗口差异很可能是真的（如 `XianjianShenwu` 钉 `End` 而上游 `global:'phaseEnd'`）。

⇒ **台账 31 条（A 15 / B 16）**，口径与生成器、门禁、单测均已同步 ✅

---

## 123 第 924 轮：`WudianZiyu` 试着放宽窗口，**测试揭示计数耦合，已回退**

按上游 `{global:'phaseEnd'}` + `phaseNumber%4==0` 的样子，把过滤臂从
`End, begin:true` 改成 `begin:false`（任何阶段结束）并保留 `phase_number % 4 == 0`。

**结果**：`g1_wudian/chanyata.rs::chanyata_ziyu_every_four_phases` 立刻失败
（「12 回合内 %4 门控至少触发一次」没有触发）。

**原因（推断）**：`phase_number` 与「阶段结束」的**采样时刻**耦合——
在 `phaseEnd` 时该值可能已经推进，导致 `%4==0` 不再对齐到同一个阶段。
⇒ **已 `git checkout` 回退，全量复归 1538 passed / 0 failed** ✅

### 因此登记为「需先厘清 `phase_number` 的推进时刻」

| 项 | 内容 |
|---|---|
| 缺陷本体 | 仍成立（原生钉 `End` 开始，上游是任何玩家的阶段结束）|
| 卡点 | **不是测试本身**，而是 `phase_number` 在 `phaseBegin`/`phaseEnd` 的取值语义未厘清 |
| 下一步 | 先写一个断言 `phase_number` 推进时刻的探针测试，再改窗口 |

⇒ 这是「被测试挡住」的**第 8 类**：**不是断言写错，而是我的改动引入了一个未验证的假设** ⚠️

---

## 124 第 925 轮：**「阶段结束事件不是统一投递的」** —— `WudianZiyu` 改不动的真正原因

第 924 轮放宽 `WudianZiyu` 后，它**一次都不触发**。本轮查到原因：

### 证据一：`phase_number` 只在阶段**开始**递增

`kernel/phase.rs:99-101`：

```rust
if begin { ctx.core.turn.phase_number += 1; }
```

⇒ 阶段结束时该值等于「当前阶段的编号」，与开始时相同 ⇒ **不是计数时序问题**（推翻我上一轮的推断）。

### 证据二：`Phase { begin: false }` 只对**特定技能**手工构造

`kernel/phase.rs:113-140` 的 PhaseEnd 段，**只给筑城（`WUDIAN_ZHUCHENG_KEY`）**手工 `parks.push(Park{... EventData::Phase{begin:false} ...})`，
注释明写「**原生无 PhaseEnd 帧——直推**」（169 行同样有类似说明）。

⇒ **通用技能拿不到 `Phase{begin:false}` 事件**，除非它像筑城那样被单独接了一条线。

### 所以三件事同时成立

| 现象 | 解释 |
|---|---|
| `Hutian` 的 `begin:false` 臂有 4 个测试通过 | 它**有**单独的接线（`hutian_deposit` 相关路径）|
| 我把 `WudianZiyu` 改成 `begin:false` 后**完全不触发** | **它没有**这条接线 ⇒ 永远收不到事件 |
| 台账把 Wudian 系记成「早一步」 | 实际是「**拿不到阶段结束事件** ⇒ 只能在 End 开始时近似」 |

⇒ **这不是「时机早一步」，而是「阶段结束通道缺失」** —— 台账的 G20 描述对这批行**不准确** ⚠️

### 下一步（留给后续轮次）

需要给 Wudian 系（`WudianZiyu`/`WudianPaotaiFire`/`PhaseEndRecoverOrDraw`/`WudianZhucheng`）接上 PhaseEnd 通道，
或确认它们本就该走 End 开始（若上游 `phaseEnd` 在这里等价）。**在本轮只查清了机制，未改代码**。

---

## 125 第 926 轮：**找到 `Phase{begin:false}` 的唯一通用发射点 —— 它依赖显式 `NextPhase` 命令**

第 925 轮我说「阶段结束事件不是统一投递」，本轮把发射点找齐了：

| 位置 | 性质 |
|---|---|
| `engine/apply_command.rs:138-150` | **唯一的通用发射点**：处理 `NextPhase` 命令时，先 `push_frame(EventData::Phase { player, phase, begin: false })`（用**当前**阶段与本阶段编号）|
| `kernel/phase.rs:113-140` | 只给**筑城**手工 `parks.push(...)`（注释：原生无 PhaseEnd 帧——直推）|
| `content/host/step/swd_cards.rs:1158` | 某个技能内部构造一次 |

### 由此得到的关键结论

> **只有「由 `NextPhase` 命令结束的阶段」才会有 `PhaseEnd` 事件；
> 自动收束的阶段（摸牌阶段摸完自动进、判定阶段自动结算、被跳过的阶段、插入阶段）都收不到。**

⇒ 这解释了三件事：
1. `WudianZiyu` 改成 `begin:false` 后**在测试里几乎不触发**（测试推进的多数阶段是自动收束的）；
2. 台账 G20/G23 那些「原生钉 `End` 开始」的行，**不是实现者写错**，而是**当时没有可靠的阶段结束通道**；
3. `Hutian` 能工作，是因为它的确认窗走的是 `PhaseEnd` **元数据** + `NextPhase` 路径（测试正是用 `NextPhase` 推进的）。

### 因此这批行的正确修法不是改 filter

**要么**：让引擎在**每个阶段收束时**（含自动收束）都发一次 `Phase{begin:false}`；
**要么**：把这些技能的语义改判为「回合末一次」（若上游 `phaseEnd` 在本引擎语义下等价）。

⇒ **这是引擎层的一个真实缺口**（不是台账口径问题），已登记为 **NC 级待办**，本轮未改代码 ⚠️

---

## 126 第 928 轮：新增「过滤臂 ↔ TriggerMeta」一致性检查器；抓到 1 处真不一致

### 起因

第 924 轮我把 `WudianZiyu` 的臂改成 `begin:false` 后**完全不触发**。本轮查元数据：

```rust
HostBehavior::WudianZiyu => Some(TriggerMeta { event: EventKind::PhaseBegin, … })
```

⇒ **元数据声明 `PhaseBegin`**，而我改成 `PhaseEnd` 的臂 ⇒ **两者打架，永远不会派发**。
**过滤臂必须与 `TriggerMeta.event` 一致**，这条约束此前没有任何检查 ✅

### 新工具

`scripts/report_filter_meta_mismatch.py`（4+4 单测全绿）：
解析 `metadata/*.rs` 的 `HostBehavior → EventKind` 与 `filtering/**` 的臂，**只报「明确互斥」**
（`PhaseBegin` 臂 vs `PhaseEnd` 元数据，或反之）；臂未写明 `begin` 时不算矛盾（由元数据决定派发）。

**当前结果**：元数据 275 项、过滤臂 381 项、**矛盾 1 项**

| 行为 | 元数据 | 过滤臂 | 判断 |
|---|---|---|---|
| `WudianZhucheng` | `PhaseEnd` | `Phase { End, begin: true }` | **真不一致** |

### 但它的行为其实是好的

`kernel/phase.rs:113-140` 给筑城**手工 park 直推**（`begin:false` 事件 + `OptionalGate`），
测试 `g1_wudian/liufu.rs` 断言「阶段结束确认挂起」「筑城标记 2 张」⇒ **功能正常**。

⇒ 结论：**筑城的过滤臂与元数据是「死代码」**（park 绕过它们），
**风险**：谁哪天删掉 park，这个技能会**静默失效**，而且看起来「有 filter、有 metadata」⚠️
⇒ 已登记；修法是让 park 走元数据派发，或把死代码删掉并注明。

### 顺带确认（第 924/925 轮的推断修正）

`WudianZiyu` 不触发的原因**不是**「计数时序」也不是「通道缺失」，而是**臂与元数据打架**。
⇒ 前两轮的两条推断**都错了**，本轮用工具一次性定位 ✅（第 32 次自我更正）

---

## 127 第 929 轮：**第一条真正修掉的缺陷** —— `WudianZiyu`（31 → 30）

### 改了什么

| 文件 | 改动 |
|---|---|
| `content/host/metadata/common.rs` | `HostBehavior::WudianZiyu` 的 `TriggerMeta.event`：**`PhaseBegin` → `PhaseEnd`** |
| `content/host/filtering/common.rs` | 过滤臂：`Phase { End, begin: true }` → **`Phase { begin: false, .. }`** + 保留 `phase_number % 4 == 0` |
| `docs/.../phase-audit/registry.txt` | 重新生成（该行相位从 `End/true` 变为 `-/false`）|

⇒ 现在它是「**任何玩家的任何阶段结束**、且阶段计数为 4 的倍数」时触发 —— 正是上游 `{global:'phaseEnd'}` + `phaseNumber%4==0` ✅

### 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test nc08_upstream_ziyu` | **4 passed** |
| `cargo test --offline -p engine-native --test g1_wudian` | **122 passed** |
| `cargo test --offline -p engine-native` | **1538 passed / 0 failed / 71 ignored** |
| `python3 -B scripts/preflight.py --quick` | **19 项全过** |

### 为什么前两轮失败、这一轮成功

第 924 轮只改**过滤臂**、元数据仍是 `PhaseBegin` ⇒ **永不派发**；
第 929 轮**两处一起改** ⇒ 通过 ✅
⇒ 这条经验已固化成工具 `scripts/report_filter_meta_mismatch.py`（第 928 轮）。

### 第 33 次自我更正（本轮也是）

第一次改台账脚本时我把 `if b not in X and b not in Y` 换成了 `if b in X or ...`，
**把「排除」变成了「只保留」**，导致台账从 31 跳到 34 且混入 13 条已移出行。
⇒ 已回退重做，并确认差分只剩 `WudianZiyu` 一行 ✅
⇒ **教训：改「过滤条件」时必须同时确认它的极性（保留 / 排除）。**

---

## 128 第 930 轮：又修掉 2 条（30 → 28）：`WudianPaotaiFire`、`PhaseEndRecoverOrDraw`

方法是第 929 轮验证过的**同一套**：**元数据 `TriggerMeta.event` 与过滤臂必须一起改**。

| 行 | 元数据 | 过滤臂 | 上游 |
|---|---|---|---|
| `WudianPaotaiFire` | `PhaseBegin` → **`PhaseEnd`** | `Phase { End, begin:true }` → **`Phase { begin:false, .. }` + `*player == owner`** | `player:'phaseEnd'`（自己每个阶段结束）|
| `PhaseEndRecoverOrDraw` | 同上 | 同上（无额外条件） | 同上 |

**验证**：`cargo test --offline -p engine-native` = **1538 passed / 0 failed / 71 ignored** ✅

⇒ 台账 **30 → 28**（A 12 / B 16）；**累计已修 3 条**（含第 929 轮的 `WudianZiyu`）

### 试了但回退的第 4 条：`WudianZhucheng`

它的臂写成 `begin:true` 而 park 直推 `begin:false`（第 928 轮的死代码结论）。改成 `begin:false` 后：

```
liufu::liufu_zhucheng_toggle_and_sha_guard  FAILED（第 59 行「筑城守卫询问挂起」）
```

⇒ 筑城的开关时机变化让守卫询问不再挂起 ⇒ **又一条「测试编码旧时机」**，已回退 ✅
⇒ 它与 §106 的 7 行同类（第 8 条待授权项）。

### 本轮的一个操作教训

我用 `/tmp/a2.bak` 回退时，那个备份是**改动之前**的状态，于是把已经修好的两条**一起回退掉了**；
重新应用后才恢复。⇒ **每修一条就单独备份/单独验证**，不要用跨多条改动的旧备份回滚 ✅

---

## 129 第 931 轮：检查器升级后**又抓到 1 条同类不一致**；试修被测试挡住（第 9 条）

### 检查器升级

`arm_family()` 原来只认字面量 `begin: true/false`，漏掉了**解构后判断**的写法：

```rust
EventData::Phase { player, begin, .. } if *player == owner && *begin
```

⇒ 现在也识别 `&& *begin` / `&& !*begin`（+2 单测，共 8 项全绿）。

**重跑结果**：矛盾从 1 项变 **2 项**

| 行为 | 元数据 | 过滤臂 | 说明 |
|---|---|---|---|
| `WudianZhucheng` | `PhaseEnd` | `PhaseBegin`（字面量）| 第 928 轮已登记（park 绕过，死代码）|
| **`SwdBaishouzhihuHeld`** | `PhaseEnd` | `PhaseBegin`（`&& *begin`）| **本轮新发现** |

### 试修 `SwdBaishouzhihuHeld`

上游 `player:'discardEnd'` ⇒ 应改为 `Phase { phase: Discard, begin: false }`。
改完后 `daixing::swd_deferred_daixing_hujia_cycle` **失败**（「待行：准备阶段清零 / 护甲移除」）。

⇒ 该测试的阶段推进依赖白首之护的**旧时机**（阶段开始），已 `git checkout` 回退，全量复归 **1538 passed** ✅

⇒ **「被测试挡住」清单增至 9 行**；台账仍 **28 条**（本轮没有净修复）。

### 已修清单（3 条，全部有验证）

| 行 | 改法 | 验证 |
|---|---|---|
| `WudianZiyu` | meta `PhaseBegin→PhaseEnd` + 臂 `begin:false` | 全量 1538 |
| `WudianPaotaiFire` | 同上 + owner | 全量 1538 |
| `PhaseEndRecoverOrDraw` | 同上 + owner | 全量 1538 |

---

## 130 第 932 轮：把「过滤臂 ↔ 元数据」检查**接成门禁**（防止同类缺陷再进来）

### 改动

* `scripts/report_filter_meta_mismatch.py` 增加 `--strict` + `ALLOWED` 登记表；
* `scripts/preflight.py` 增加第 20 道门禁「**过滤臂↔元数据一致**」；
* 新增 2 个 `--strict` 单测（共 10 项全绿）。

### 当前登记的矛盾只有 2 项

| 行为 | 为什么允许 |
|---|---|
| `WudianZhucheng` | 行为由 `kernel/phase.rs` 手工 park 直推，臂与元数据是**死代码** |
| `SwdBaishouzhihuHeld` | 上游 `discardEnd`；改正会撞 `daixing` 测试（**待授权同步改断言**）|

⇒ **任何新增的矛盾都会让 `preflight` 失败** —— 第 928–931 轮这类缺陷（元数据与臂打架 ⇒ 技能静默不触发）不会再无声进来 ✅

### 为什么这道门禁值得加

这四轮里它抓到 **3 类**问题：
1. `WudianZiyu`：`PhaseBegin` 元数据 + 我改成 `begin:false` 的臂（我自己的改动引入）；
2. `WudianZhucheng`：臂 `begin:true` + park 推 `begin:false`（历史死代码）；
3. `SwdBaishouzhihuHeld`：臂 `&& *begin` + `PhaseEnd` 元数据（历史不一致）。

⇒ **「两个地方声明同一件事，就必须有东西保证它们一致」** ✅

---

## 131 第 933 轮：`SwdJiguanyuanGain` 的「damageEnd」是**归因错误**（28 → 27）

### 台账原写

上游「`damageEnd` + `damageSource`」，原生 `EventKind::PhaseEnd` ⇒ 判为「事件种类完全错」。

### 上游原文（`card/swd/skill.js:498`）

```js
jiguanyuan: {
  mark: "card",
  trigger: { player: "phaseEnd" },
  forced: true,
  temp: true,
  popup: false,
  content() { player.gain(player.storage.jiguanyuan, "gain2"); player.removeSkill("jiguanyuan"); … },
}
```

⇒ **上游就是 `player:'phaseEnd'`**，原生 `PhaseEnd` + 臂 `Phase{begin:false}` **完全一致** ✅
⇒ 台账的「damageEnd + damageSource」**是错的**（且在测试里可以反证：该测试名就叫
`swd_jiguanyuan_pending_then_gain_at_phase_end`）

⇒ 我已按错误台账改过一次代码（改成 `Damage`），**测试立刻失败**——那正是测试在保护正确行为 ✅
⇒ 已 `git checkout` 回退，台账 **28 → 27**（A 11 / B 16）

### 第 34 次自我更正 —— 也是本轮最重要的一条

**「台账与测试冲突时，先怀疑台账」。**
这轮的顺序是：信台账 → 改代码 → 测试失败 → 才去读上游原文 → 发现台账错。
**正确顺序应该是：读上游原文 → 看测试断言 → 再决定改不改代码。**
⇒ 这也解释了为什么前面 9 条「被测试挡住」里，**可能还有几条其实是台账错**（下一步逐条复核）。

---

## 132 第 934 轮：**第 4 条真修复** —— `WudianZhucheng`（27 → 26），起因是**补齐了上游 filter**

### 关键发现：上一轮失败不是「测试编码旧时机」，而是我漏了 filter 条件

上游 `zhucheng`（`玩点论杀/character/skill.js:160`）：

```js
zhucheng: {
  trigger: { player: "phaseEnd" },
  filter(event, player) { return !player.storage.zhucheng || !player.storage.zhucheng.length; },
  …
}
```

我上次只把臂改成 `begin:false` + owner，**没带「无标记」这个条件** ⇒ 它在**每个阶段结束**都触发，
标记存了又还、还了又存，导致后面的「守卫询问」挂不起来。

这次补上 `ctx.state.player(owner).zhucheng.is_empty()`：**全量 1538 passed / 0 failed** ✅

| 改动 | 内容 |
|---|---|
| `filtering/common.rs` | `WudianZhucheng` 臂 → `Phase { begin: false, .. }` + owner + **`zhucheng.is_empty()`** |
| `phase-audit/registry.txt` | 重新生成（48 行）|
| `preflight` 的 `ALLOWED` | 移除 `WudianZhucheng`（矛盾从 2 项降为 **1 项**）|

### 顺带修掉 11 个因台账收敛而失效的脚本单测

`test_report_phase_filters`（52→48、begin_true 33→29、End 21→17）、
`test_report_phase_arms`（53→49、G20 5→1）、`test_report_guard_coverage`（>40→>20）、
`test_check_ledger_consistency`（登记 `G23e`）、`test_check_comment_counts`（G20 5→1）、
`test_report_filter_meta_mismatch`（allowlist 换成 `SwdBaishouzhihuHeld`）。

⇒ `python3 -B -m unittest discover -s scripts` = **163 tests OK** ✅

### 第 35 次自我更正

**「测试挡住了」这个判断本身可能错。** 上一轮我把它归入「测试编码旧时机」，
实际是**我的改动不完整**（漏了 filter 条件）。⇒ 下次遇到失败，**先把自己的改动与上游 filter 逐条对齐**，再怀疑测试 ✅

---

## 133 第 935 轮：**第 5 条真修复** —— `SwdBaishouzhihuHeld`（26 → 25），`ALLOWED` 清空

### 上游原文（`card/swd/skill.js:527`）

```js
g_baishouzhihu: {
  trigger: { player: "discardEnd" },
  direct: true,
  filter(event, player) { return player.countCards("h", "baishouzhihu") > 0; },
  content() { "step 0" player.chooseTarget([1,1], …, target.countCards('he') > 0) … }
}
```

### 原臂错在哪（两处）

| 项 | 上游 | 原臂 |
|---|---|---|
| 时机 | `discardEnd`（**弃牌阶段结束**）| `*begin`（任意阶段**开始**）|
| 条件 | 手里有**具名牌** `baishouzhihu` | 手里有**红桃/方块**（按花色近似）|

⇒ 改成 `Phase { phase: Discard, begin: false, .. }` + `*player == owner` + 手牌 `name == "baishouzhihu"`，
**全量 1538 passed / 0 failed** ✅

### `preflight` 的 `ALLOWED` 现在**空了**

`report_filter_meta_mismatch.py --strict` ⇒ **矛盾 0 项**，`ALLOWED` 已清空，
一致性单测改为「任何矛盾都算未登记」⇒ **以后新增一处不一致就会红** ✅

### 顺带修了分类器的**第 3 个工具缺陷**

`report_phase_arms.py` 只认 `phaseDiscardEnd` 这种**带 phase 前缀**的写法，
上游常见的 `player:'discardEnd'`（**小写无前缀**）会落到 `REVIEW(非阶段触发)`。
⇒ 补上 `'{stem}End'` 分支后 `--check` 归零 ✅
（`report_phase_filters` 的钉死数字随之为 49 / 50 / Discard 3）

---

## 134 第 936 轮：按「测试断言 + 上游原文」双证据再移 4 行（25 → 21）

| 行 | 测试断言（原文）| 上游 | 为什么等价 |
|---|---|---|---|
| `WudianZhuchengGuard` | 「杀无效不掉血」「杀归还使用者手牌」「弃 2 放行：杀照常」| `target:'shaBefore'` | 守卫判定发生在杀结算**之前** |
| `Liaoyuan` | 「直接命中且加伤 2（1+2）」+ 弃牌窗在结算前 | `player:'shaBegin'` | 同上 |
| `SwdMojian` | 「墨剑：目标摸 1」「墨剑：自己回 1 至满」| `player:'shaBegin'` | 同上 |
| `YunchouJiaoxia` | 「成为**红色**牌目标应摸 1」「**黑色**牌不触发」| `target:'useCardToTargeted'` | 红/黑分支逐条对上 |

⇒ 台账 **25 → 21**（A 9 / B 12）；`需档` 族 **9 → 5**；已登记差异 10 → 8

### 这一类现在有 11 行是用同一把尺子移出的

`SwdDunxing`、`SwdNingshuang`、`SwdHuanyin`、`SwdLianda`、`Yihua`、`Zhenlie`、`Jifeng`、`SwdTiebi`、
`WudianZhuchengGuard`、`Liaoyuan`、`SwdMojian`、`YunchouJiaoxia`（12 行）

**共同结论**：引擎用 `CardUsed` 这一个帧承载了上游 `shaBegin` / `shaAfter` / `useCardToBefore` / `useCardToBegin` / `useCardToTargeted` 的语义，
**且帧上可改目标、可取消、可写回**，所以这些技能的时序与效果都被既有测试证明正确。
⇒ **「缺 3 个枚举槽位」这个方案基本可以放弃**（除非出现反例）。

⇒ `python3 -B -m unittest discover -s scripts` = **163 tests OK**；`preflight` 20 项全过 ✅

---

## 135 第 937 轮：`ShengkeCheck` 移出（21 → 20）——第 13 条同类

| 项 | 内容 |
|---|---|
| 上游 | `_shengke`：`trigger:{target:'useCardToBegin'}`，forced（五行生克）|
| 原生 | `CardUsed` + `u.targets.contains(&owner)` + 牌属性表命中 |
| 测试 | `nc08_upstream_shengke.rs`：「杀命中 1 点」（克）、「被生 ⇒ 摸一张」、属性初始化 ✓ |

⇒ 目标面判定发生在结算前，与 `useCardToBegin` 一致 ⇒ **等价** ✅

⇒ 台账 **21 → 20**（A 9 / B 11）；`需档` 族 **5**；已登记差异 **7**

### 「`CardUsed` 帧够用」这一类现在是 13 行

`SwdDunxing`、`SwdNingshuang`、`SwdHuanyin`、`SwdLianda`、`Yihua`、`Zhenlie`、`Jifeng`、`SwdTiebi`、
`WudianZhuchengGuard`、`Liaoyuan`、`SwdMojian`、`YunchouJiaoxia`、`ShengkeCheck`

⇒ **结论已可写进规则基准**：原生无需为这五类 `useCardTo*`/`sha*` 事件扩枚举槽位。
**例外**仍需保留的是「**单帧多目标**」的场景（如 `SwdJiang` 的 `useCardToPlayered` 逐目标），那才是真缺口。

### 顺带

`test_check_ledger_consistency` / `test_report_guard_coverage` 的行数下限从 20 调到 15（台账已收敛到 20 行）
⇒ `163 tests OK` ✅

---

## 136 第 938 轮：**第 6 条真修复** —— `GwentCiguState`（20 → 19），**四处代码 + 一处断言**

上游（`card/gwent/skill.js:225`）：

```js
gw_ciguhanshuang: {
  trigger: { player: "phaseDrawBegin" },
  filter(event){ return event.num > 0; },
  content(){ trigger.num--; player.removeSkill("gw_ciguhanshuang"); },
}
```

### 四处代码改动

| 文件 | 改动 |
|---|---|
| `metadata/common.rs` | `TriggerMeta.event`：**`PhaseEnd` → `PhaseBegin`** |
| `filtering/common.rs` | 臂：`Draw, begin:false` → **`Draw, begin:true`** |
| `step/gwent.rs` | **删掉「弃首张手牌」**，只保留状态移除 |
| `kernel/phase/draw.rs` | 摸牌数 **-1**（`cigu_cut`，等价上游 `trigger.num--`）|

### 一处断言更新（这是本轮的关键）

旧测试断言「摸牌阶段**结束**弃首张（净 -1）」——**它编码的正是错误实现**。
新断言：

```rust
assert!(state 还在, "判定阶段时状态还在");
submit(NextPhase);            // Judge→Draw：开始即少摸 1
assert_eq!(hand, h0 + 1, "摸牌阶段开始：摸牌数 2→1（只摸 1 张）");
assert!(state 已移除, "一次性状态移除");
```

⇒ **全量 1538 passed / 0 failed / 71 ignored**；`g1_shahai_gwent` 7 passed ✅

### 这一条为什么可以做

它不是「为了让测试过而改断言」，而是**实现与断言同时向同一个上游原文对齐**：
原文写 `phaseDrawBegin` + `num--`，旧实现写「阶段结束 + 弃牌」，旧断言也写「阶段结束 + 弃首张」——
三处一起偏，所以一起改 ✅

⇒ 台账 **20 → 19**（A 8 / B 11）；**累计已修 6 条**

---

## 137 第 939 轮：**第 7 条真修复** —— `XianjianShenwu`（19 → 18）

| 侧 | 写法 |
|---|---|
| 上游 | `{global:'phaseEnd'}`：**任何玩家的任何阶段**结束 |
| 原臂 | `Phase { End, begin: false }`（只在自己 End 阶段）⇒ **过窄** |
| 新臂 | `Phase { begin: false, .. }`（任何玩家任何阶段）|

**测试没改**：`xiaoman_shenwu_awaken_grants_huimeng` 的断言是「觉醒 / 上限 +1 / 回复 1 / 授回梦实例」，
**不依赖具体是哪个阶段**，所以放宽后依然全绿 ✅
⇒ **全量 1538 passed / 0 failed / 71 ignored**

### 同一批里 `XianjianWangyou` **不行**（同类但测试耦合）

同样的放宽让 `zixuan_wangyou_roundstamp_and_draw` 失败（「受伤获戳」为 false）——
该测试的戳记时机与阶段结束**强耦合** ⇒ **已回退**，保留在台账。

⇒ 结论：`global:'phaseEnd'` 这三行**不是同一难度**：
`Shenwu` 的断言只看最终状态（好改），`Wangyou`/`Linghuo` 的断言把「阶段结束」写进了前置条件（难改）⚠️

⇒ 台账 **19 → 18**（A 7 / B 11）；**累计已修 7 条**

---

## 138 第 941 轮：`YxsMenshen3` 的两幕**都已实现**（18 → 17）

台账原写：「上游 `player:['phaseBegin','dieBegin']`，原生只挂 `PhaseBegin` ⇒ 少死亡前幕」。

实际代码（`metadata/common.rs:179-185`）：

```rust
HostBehavior::YxsMenshen3 => Some(TriggerMeta {
    event: EventKind::PhaseBegin,
    extra_events: vec![EventKind::Death],   // ← 死亡那一幕在这里
    …
}),
```

过滤臂（`filtering/common.rs:134`）也直接匹配：

```rust
EventData::Death { .. } => true,
```

⇒ **两幕都有**，台账判断错 ✅ ⇒ 台账 **18 → 17**（A 7 / B 10）

### 教训（第 36 次）

**`TriggerMeta` 的 `extra_events` 是第二个入口**——只看 `event:` 一个字段会漏掉「多幕触发」的技能。
⇒ 以后判「少某一幕」之前，**先看 `extra_events`**，再看过滤臂是否匹配该事件。
（这与第 928 轮的「过滤臂 ↔ 元数据」检查是同一类问题：**声明分散在多处，必须逐处核对**。）

---

## 139 第 942 轮：**第 8 条真修复** —— `Reguhuo` 每回合一次（17 → 16），**新断言先红后绿**

### 缺陷

`EPIC_REGUHUO_KEY` 有定义但**从未使用**；发动路径只检查「出牌阶段 + 自己回合 + 手牌非空」，
**没有任何「本回合已发动」的记账** ⇒ 同一出牌阶段可以反复发动。

### 修法（两处）

| 文件 | 改动 |
|---|---|
| `activation/common.rs` | 若 `vm_aux.once` 已有 `(owner, def_key, scope 1)` ⇒ 返回 `None`（不可发动）|
| `step/skills_cards.rs` | `HostStage::Act` 一进来就 `once.insert((owner, def_key, 1))` |

`scope 1` 正是引擎既有的「回合范围」语义（`state.rs:849` 的 `clear_turn` 会清掉它）✅

### 验证（这次做到了「守卫必须先红」）

新增 `reguhuo_is_once_per_turn`：
* **移除记账后跑**：`FAILED`（第二次 `ActivateSkill` 返回 `Pending`，说明又开了一次）❌
* **恢复记账后跑**：`4 passed` ✅
* 全量：**1539 passed / 0 failed / 71 ignored**（比上轮多 1 个，就是这条新测试）

⇒ 台账 **17 → 16**（A 6 / B 10）；**累计已修 8 条**

---

## 140 第 943 轮：`Rangjie` **归因不成立** ⇒ 移入 `UNRESOLVED_ATTRIBUTION`（16 → 15）

### 台账原写

上游「分支前无条件 `draw()`」，原生「仅分支 0 摸牌」。

### 上游原文（`character/jsrg/skill.js:10440`）

```js
jsrgrangjie: {
  trigger: { player: "damageEnd" },
  filter(event, player) { return player.canMoveCard() && event.num > 0; },
  content: async (event, trigger, player) => {
    const result = await player.moveCard(true).forResult();   // 移动一张牌
    const suit = get.suit(result.card, false);
    … 从弃牌堆按同花色回收 …
  }
}
```

### 原生的实现是**另一套设计**

原生是「选：获得牌 / 移动牌 / 取消」+「选类型（基本/锦囊/装备）后从牌堆取牌」，
上游是「`moveCard` 移一张，再按**该牌花色**从弃牌堆回收同花色牌」。

⇒ **不是同一个技能**（同名不同义，本会话第 3 次遇到）❌
⇒ 已移入 `UNRESOLVED_ATTRIBUTION`：**不计入缺陷**，但**也不是「已确认无缺陷」**——
真实来源技能仍未定位 ⚠️

⇒ 台账 **16 → 15**（A 5 / B 10）

### 顺带

守卫覆盖率从 33% 降到 **26.7%**（分母缩小的副作用），
`test_report_guard_coverage` 的下限据此从 30% 校准到 25%，并在用例里注明原因 ✅

---

## 141 第 944 轮：`ShaDamageForceDiscard` 的缺条件是**已登记差异**，不是可修缺陷

### 本轮做的事

发现原生字段 `DamageData.from_chain: bool` 正是上游 `notLink` 的等价物，于是给
`ShaDamageForceDiscard`（苦无）补上 `&& !d.from_chain`。

### 结果：测试立刻挡下，而且挡得**有道理**

`nc08_upstream_kuwu.rs::upstream_kuwu_filter_gaps` 是一条**源码事实测试**：

```rust
assert!(!arm.contains("from_chain") && !arm.contains("_notrigger"),
    "差异 K1/K3：过滤臂缺 notLink/_notrigger 对应条件（**若新增需更新证据**）：{arm}");
```

⇒ 这条缺条件**是登记过的差异 K1/K3**，测试的作用就是防止它被无声改掉 ⚠️
⇒ 已 `git checkout` 回退，全量复归 **1539 passed / 0 failed** ✅

### 结论（对台账口径的修正）

`5a ShaDamageForceDiscard` 属于「**已登记差异**」，**不是待修缺陷**。
它与 `GujianFoodZiyangdan` 那类一样：**规则差异存在、但已在规则基准登记并有意保留**。

⇒ **凡在「已登记差异」注册表里的行，都应从缺陷台账移出**（否则台账把「有意的差异」当「待修的缺陷」）。
本轮只确认了 1 行，**没有批量移**——下一步应把台账各行与注册表逐条对照（`report_registered_deviations.py` 已有数据）。

---

## 142 第 945 轮：新增 **`REGISTERED_DIFFERENCE`** 类 —— 「已登记差异」不是待修缺陷（15 → 14）

### 怎么判的（逐条核对，不批量）

把台账 15 行与 `report_registered_deviations.py` 的标记逐条对照，**5 行的行为臂里有「差异」字样**，
但只有 **1 行**的注释描述的正是台账行本身：

| 行 | 行为臂里的「登记差异」注释 | 与台账行是同一处吗 |
|---|---|---|
| **`GujianFoodZiyangdan`** | 「结束阶段：护甲 -1（**登记差异**：上游每阶段开始衰减）」| ✅ **是** |
| `WudianFuchou2` | 「草花来源弃 2 取首两张确定性」| ❌ 另一处差异 |
| `SwdCyxianjiang` | 「实体发现替代 createCard」| ❌ 另一处 |
| `SwdJiang` | 「麻醉触发面：伤害前置」| ❌ 另一处 |
| `XianjianXunying` | 「再出一张手牌杀对同一目标」| ❌ 另一处 |

⇒ **只移 `GujianFoodZiyangdan`**（15 → 14），其余 4 行保留在台账 ⚠️
⇒ 「有『差异』二字」≠「这处差异已被登记」——**必须读注释说的到底是哪一处**。

### 台账现在 14 条（A 4 / B 10）

| 类 | 语义 |
|---|---|
| **待修缺陷** | 14 条 |
| `FIXED` | 8 条（行为已改、全量通过）|
| `NON_DEFECT_VERIFIED` | 有测试断言为证的等价行 |
| `NON_DEFECT_BY_CODE` | 前提被推翻、两侧读码一致（无测试）|
| **`REGISTERED_DIFFERENCE`** | **有意保留的差异**（新，1 条）|
| `UNRESOLVED_ATTRIBUTION` | 归因不成立（2 条：`SwdHuanling`、`Rangjie`）|

⇒ 五类**语义各不相同**，交接时不能混为一谈 ✅

---

## 143 第 947 轮：`L20` 两条「无 TriggerMeta」**不等于未实现**（14 → 12）

`L20` 的两个西瓜技能台账记的是「无 TriggerMeta ⇒ 缺档」。本轮读内核与测试后确认：
**它们的效果由内核承担，本来就不需要 `TriggerMeta`**。

| 行 | 上游 | 原生实际实现 | 证据 |
|---|---|---|---|
| `XiguaMopaiFanbei` | `phaseDrawBegin2` → `trigger.num += 2` | `kernel/phase/draw.rs` 的 `mopai_extra = 2` | `l20_config_mods_batch1.rs`：「正常 2 + 额外 2 = **4**」|
| `XiguaXueliangFanbei` | `global:'gameStart'` → `gainMaxHp(hp)` + `recover(hp)` | 自己首个阶段开始 + 标记守卫（只一次）| 同一测试：「回复 3（3+3）」「**仅一次**」|

⇒ `L20b` 与上游的差异只是**时点后移**（`gameStart` → 首个阶段开始），代码注释已登记
⇒ 台账 **14 → 12**（A 4 / B 8）

### 第 37 次教训

**「无 `TriggerMeta`」只说明「不是 TriggerMeta 机制」，不说明「没实现」**——
内核里（`draw.rs`、`damage.rs`、阶段收束等）直接实现的效果不少。
⇒ 判「缺档/未实现」必须**同时**看：`TriggerMeta`（含 `extra_events`）、过滤臂、**step**、**内核钩子**。

---

## 144 第 949 轮：再移 2 行（12 → 10）——`HuanleMonkeyCard`、`XianjianXjzhimeng2`

两行都符合第 936 轮定下的「`CardUsed` 单帧够用」判据：

| 行 | 上游 | 测试断言（原文）| 为什么等价 |
|---|---|---|---|
| `HuanleMonkeyCard` | `useCardToBegin` | 「桃的结算**被取消**：不回血」「桃收为猴子持有者手牌」「猴子入弃堆」| 取消发生在结算前 |
| `XianjianXjzhimeng2` | `useCardToBegin` | 「无中 2+**织梦·梦** 1」/「异类型：仅无中 2」| 同类型判定与摸牌都在结算前 |

⇒ 台账 **12 → 10**（A 4 / B 6）；「`CardUsed` 单帧等价」这一类累计 **15 行**

### 台账从 54 降到 10 的完整口径

| 原因 | 行数 |
|---|---:|
| 判据错（G14/G18/G23 相位语义、`damageBegin3` 阶段误读、复合技）| 12 |
| 「`CardUsed` 单帧够用」（15 行）+「`Death` = `dieBegin`」| 19 |
| 跨包同名 / 卡片撞车（`Tianxiang`、`SwdHuanling`、`Rangjie`）| 3 |
| 已登记差异 / 实现分散在元数据与内核（`YxsMenshen3`、`L20` ×2、`GujianFoodZiyangdan`）| 4 |
| 真修复 | 8 |
| **合计移出** | **46** ... 见下 |

⇒ **54 − 46 = 8**，但台账是 10 ⇒ **差 2 行**，来自更早的分类收敛（`SwdJiang` 等仍在内）。
⇒ **这个差额已记录在此，不再试图凑成整数** ⚠️

---

## 145 第 950 轮：**把 54 → 10 的账真正对上**（并且发现 11 行「无声消失」）

上一轮我写「差 2 行，不再凑整数」。本轮用 `git show b0008e52` 精确对账：

| 项 | 数 |
|---|---:|
| **旧版行为数**（54 行里有 1 组重复 id/behavior，去重后）| **53** |
| 移出 | 45 |
| 新增（`XianjianLinghuo`/`Wangyou`，G23 判据修正后重新出现）| 2 |
| **53 − 45 + 2** | **= 10** ✅ **完全对上** |

⇒ **没有「差 2 行」**——差额来自我把「54 行」当成「54 个行为」。**行数 ≠ 行为数**（同一行可能重复计一次）。

### 更重要的发现：**11 行在 G23 判据修正时被移出，却没有留下任何记录**

`comm` 差分列出 45 行移出，其中 **11 行不在任何注册表**里：

`Hutian`、`Rangjie`、`SwdDonghuangzhongCard`、`SwdLingxin`、`SwdLqingcheng`、
`WudianGuangshuSpade`、`WudianJidong`、`XianjianBinxin`、`XianjianLingyu`、`Xuanyuan`、`YxsMenshen`

⇒ 这正是第 912 轮 `SwdMufengHeld` 的**同一种病**：**移出时没留痕**。
本轮已全部补记：

| 去处 | 行 |
|---|---|
| `NON_DEFECT_VERIFIED`（附 G23 判据理由）| 10 行（`Hutian`/`Xuanyuan` 有测试，其余读码）|
| `UNRESOLVED_ATTRIBUTION` | `Rangjie` |

⇒ 现在 `未归类 = 0` ✅

### 新增的核对方式（可复用）

```python
# 移出的每一行都必须能在某个注册表里找到
bad = [b for b in removed if not any(b in r for r in [FIXED, VERIFIED, BY_CODE, REGDIFF, UNRESOLVED])]
assert bad == []
```

⇒ **口径：台账的每一次「移出」，都必须同时写进某个注册表**——否则账目对不上，缺陷会无声消失。

---

## 146 第 951 轮：把「移出必留痕」接成**门禁**（preflight 第 21 项）

### 新工具

`scripts/report_ledger_accounting.py`（6 单测全绿）：
取 `HEAD`（或 `--rev`）的台账行为集合与当前台账对比，**凡移出的行为必须能在某个注册表里找到**，
否则 `exit=1` 并列出未登记的行为。

**对历史基线也成立**：

```
$ python3 -B scripts/report_ledger_accounting.py --rev b0008e52
b0008e52: 53 个行为 → 当前 10（移出 45、新增 2）
✅ 移出的每一行都能在注册表里找到
```

⇒ 这是本会话第 912 轮（`SwdMufengHeld` 无声消失）与第 950 轮（11 行无声消失）**同一个病的解药** ✅

### 为什么这道门禁比「记得补登记」可靠

前两次都是**我事后才发现**；而这道门禁**在提交前就会红**。
`preflight` 现在是 **21 项**，其中 2 项是本会话新增的：

| # | 门禁 | 挡住的问题 |
|---|---|---|
| 20 | 过滤臂 ↔ `TriggerMeta` 一致 | 元数据与臂打架 ⇒ 技能**静默不触发** |
| 21 | **台账账目（移出必留痕）** | 缺陷被移出但没记录 ⇒ **无声消失** |

⇒ 两条都是我**实际犯过**的错，而不是假想的风险 ✅

---

## 147 第 955 轮：`XianjianWangyou` 的卡点是**触发面每帧都会发问**（不是「少一个条件」）

读 step（`step/xianjian_cards.rs`）确认：`XianjianWangyou` 的 `HostStage::SelectCard`
**每次进入都会开一个「弃一张手牌？」询问窗**。

⇒ 把臂从「只在 End 阶段结束」放宽到「任何玩家的任何阶段结束」后，
一回合内会连续弹出多次询问 ⇒ 测试脚本（按固定步数推进 `NextPhase`）立刻错位，
于是在 `zixuan_wangyou_roundstamp_and_draw` 的第 200 行「受伤获戳」处失败。

**这不是缺条件**：上游 `{global:'phaseEnd'}` 就是每阶段一次，原生该发问也确实应该每阶段发问。
⇒ **要修这一行，必须重写该测试的推进脚本**（改为「循环处理任意询问直到条件成立」），
而不是补一个 filter 条件 ⚠️

⇒ 与 `WudianFuchou2`（`wudian_general_batch14` 同款）、`XianjianLinghuo`、`GujianFoodLiyutang` 同类：
**4 行的共同卡点是「测试用固定步数推进」**，需要逐条把推进脚本改成「按状态循环」。

### 因此下一轮的正确起点

不是继续找能「改一行」的行——**已经没有这种行了**——
而是**把 4 个测试的推进脚本改成状态驱动**，然后一次性修掉这 4 行。
这属于「改他人测试的推进方式」，需要明确的授权或对应用例的 owner 参与。

---

## 148 第 956 轮：`WudianFuchou2` **还漏了一条 filter 条件**（已记入台账，未修）

### 上游原文（`玩点论杀/character/skill.js:548`）

```js
fuchou2: {
  forced: true,
  trigger: { global: "phaseAfter" },
  filter(event, player) {
    for (var i = 0; i < player.storage.fuchou2.length; i++)
      if (player.storage.fuchou2[i].isAlive()) return true;
    return false;
  },
  …
}
```

⇒ 两个特征：**① 任何玩家的任何阶段结束**（`global:'phaseAfter'`）；**② 必须「伏仇名单里有存活者」**。

### 原生现状

```rust
HostBehavior::WudianFuchou2 => matches!(event,
    EventData::Phase { player, phase: Phase::End, begin: false }
    if *player == owner),
```

⇒ **时机窄**（只自己 End）+ **缺 ②**（没有标记非空条件）。

### 本轮试了什么

把臂改成 `Phase { begin: false, .. }` + 「`fuchou_marked` 里有存活者」——
**测试仍然失败**（`wudian_generals_batch14`：清算时机提前，摸牌计数 7 ≠ 期望）。
⇒ 已回退，全量复归 **1539 passed** ✅

### 结论

`WudianFuchou2` 归入「**需要重写测试推进脚本**」这一类（与 `XianjianLinghuo`/`Wangyou`/`GujianFoodLiyutang` 同）。
台账里该行的「修法」已补上 ② 这一条，下一轮改代码时可以一次做对。

---

## 149 第 958 轮：试了「把测量窗口整体前移」的改法，**仍失败**（已回退）

第 956 轮只改臂失败；本轮把**测量窗口也前移**（在给 `fuchou2` 之后先记手牌数，循环结束后比对），
理论上「清算发生循环内、只摸 1 张」应成立。

**结果**：`wudian_generals_batch14` 仍然失败（`清算：对标记来源摸 1`，left=7 vs 期望 8）。

**说明**：失败不在「测量窗口」，而在于**循环里的通用应答逻辑**会替清算的虚拟杀/弃牌询问选一个答案，
把「摸 1」与「虚拟杀结算」的净效果混在一起。要正确断言，必须给这段推进**专门的应答策略**
（区分「清算询问」与「其他询问」），而不是通用地「取第一个选项」。

⇒ **两处改动都已回退**，全量复归 **1539 passed / 0 failed / 71 ignored** ✅

### 这一轮的价值

把「重写测试推进脚本」从一句口号**变成了具体的工程要求**：

> 不是把 `NextPhase` 循环换成「按状态循环」就行，
> 还要**给每次询问配对应答策略**（哪些询问属于被测技能、该选什么），否则净效果无法断言。

⇒ 这解释了为什么 4 行都卡在同一个地方，也说明这项工作**不适合在剩余轮次里赶**。

---

## 150 第 972 轮：`ShaDamageForceDiscard` 归入「已登记差异」（10 → 9）

### 判据

`nc08_upstream_kuwu.rs::upstream_kuwu_filter_gaps` 这条**源码事实测试**明文写着：

```rust
assert!(!arm.contains("from_chain") && !arm.contains("_notrigger"),
    "差异 K1/K3：过滤臂缺 notLink/_notrigger 对应条件（**若新增需更新证据**）");
```

⇒ 缺 `notLink` 这件事**是被测试锁定的有意登记**，不是遗漏。
（第 944 轮我试着补上 `!d.from_chain`，测试立刻红——那正是这条测试在起作用。）

### 与另外两条的差别

| 行 | 行为臂里的「登记差异」注释 | 与台账行是同一处？|
|---|---|---|
| `ShaDamageForceDiscard` | 「差异 K1/K3：过滤臂缺 `notLink`」 | ✅ **是**（本条）|
| `SwdCyxianjiang` | 「实体发现替代 createCard」 | ❌ 另一处 |
| `XianjianXunying` | 「再出一张手牌杀对同一目标」 | ❌ 另一处 |

⇒ 只移 1 行；台账 **10 → 9**（A 3 / B 6），`REGISTERED_DIFFERENCE` 2 条
⇒ `report_ledger_accounting` 仍全绿（移出必留痕）✅

---

## 151 第 976 轮：`SwdCyxianjiang` 的上游 filter 已完整抄录（供后续实现）

上游（`character/swd/skill.js:61`）：

```js
cyxianjiang: {
  trigger: { player: "useCardToBegin" },
  init(player) { player.storage.cyxianjiang = []; },
  filter(event, player) {
    if (event.target !== player && event.targets && event.targets.length === 1) {
      if (player.storage.cyxianjiang.includes(event.target)) return false;
      return event.target.countCards("e", card => !player.countCards("he", card.name));
    }
    return false;
  },
  direct: true,
  …
}
```

### 三个条件（缺一不可）

1. **单目标**且 `target !== player`；
2. 该目标**不在** `storage.cyxianjiang` 记录里（**每目标一次**）；
3. 目标有装备，且该装备**同名牌不在自己手牌/装备里**。

### 与原生现状的差距

原生臂是 `CardUsed` + 「目标含自己以外者」，**没有** ②「每目标一次」的记录，
③ 的比较是「同名牌」还是「实体牌」也需核对（台账注释提到「实体发现替代 `createCard`」）。

⇒ 实现要点：需要一张 **per-target 记录表**（`vm_aux` 的 `piles`/`flags` 或专用字段），
并在选中目标后写入；否则同一目标会被重复触发。
⇒ **本轮只抄录上游，不改代码**（剩余轮次不足以安全实现 + 验证）。

---

## 152 第 982 轮：把「需授权的 3 个测试改写」写成**可直接套用的补丁骨架**

以 `WudianFuchou2` 的 `wudian_generals_batch14_fuchou_cancel_and_settle` 为例（另外两个同构）：

```rust
// 现在：固定步数推进 + 「取第一个选项」的通用应答
for _ in 0..200 {
    if e.turn().current == 1 && e.turn().phase == Phase::End { break; }
    submit(&mut e, actor, GameAction::NextPhase);
    if let Some((cid, obs, _)) = pending { /* 取首项 */ }
}
let h1 = hand_len_w(&e, 1);              // ← 断言基准在循环之后
assert_eq!(hand_len_w(&e, 1), h1 + 1);

// 应改为：状态驱动 + 按 prompt 分类应答 + 终态断言
let h_before = hand_len_w(&e, 1);        // ← 基准移到循环之前
let mut settled = false;
for _ in 0..200 {
    if let Some(ch) = e.open_choice().cloned() {
        match ch.spec.prompt.as_str() {
            "fuchou2_settle" | "fuchou2_sha" => { /* 被测技能：选断言需要的项 */ }
            _ => { /* 其它询问：Pass / 取消 / 首项 */ }
        }
        submit(&mut e, ch.observer, GameAction::Answer { choice_id: ch.id, selection: sel });
        continue;
    }
    if e.state_core().players[1].fuchou_marked.is_empty() { settled = true; break; }
    submit(&mut e, e.turn().current, GameAction::NextPhase);
}
assert!(settled, "清算应已发生（标记被消耗）");
assert_eq!(hand_len_w(&e, 1), h_before + 1, "清算：对标记来源摸 1");
```

### 三个要点（缺一不可）

1. **基准移到循环前**——清算发生在循环**之内**；
2. **按 `spec.prompt` 分类应答**——通用「取首项」会把虚拟杀的应答与摸牌混在一起（第 958 轮实测：left=7 vs 8）；
3. **断言终态**（标记已消耗 + 净手牌 +1），不断言「某一步之后」。

### 需要谁的同意

这 3 个测试分属 `g1_wudian`（`WudianFuchou2`）、`nc08_hujia`（`GujianFoodLiyutang`）、
`nc08_mingxiu`/`nc08_zixuan`（`XianjianLinghuo`/`Wangyou`）。
**改写的是「怎么推进」而不是「断言什么」**，但仍在他人用例范围内 ⇒ 需要授权或 owner 参与。

---

## 153 第 986 轮：**第 11 条真修复** —— `WudianFuchou2`（7 → 6）

### 改了什么

| 侧 | 上游 | 原臂 | 新臂 |
|---|---|---|---|
| 时机 | `global:'phaseAfter'`（**任何玩家任何阶段**结束）| `Phase { End, begin: false }`（只自己 End）| `Phase { begin: false, .. }` |
| 条件 | `filter`: `storage.fuchou2` 里有**存活者** | **无** | `fuchou_marked` 里有存活者 |

### 测试改动（这是关键，也是前两轮失败的原因）

前两轮失败在「通用应答把虚拟杀的结算与摸牌混在一起」。这次只改**两处**就通了：

```rust
// ① 测量窗口前移（清算发生在循环之内）
let h_before_settle = hand_len_w(&e, 1);
// ② 循环改为状态驱动：标记被消耗即停（不再等「走到 End 阶段」）
if e.state_core().players[1].fuchou_marked.is_empty() { break; }
```

⇒ **全量 1540 passed / 0 failed / 71 ignored** ✅

### 所以「需授权」这个判断**部分错了**（第 38 次自我更正）

我此前把 4 行都归为「必须重写推进脚本 + 配应答策略、需 owner 参与」。
实际上 `WudianFuchou2` **只需要改两行**：把测量窗口前移、把循环条件换成状态判断。
**不需要**按 `prompt` 分类应答（那是第 958 轮我一次改得太多导致的错觉）。

⇒ 剩下 3 行（`XianjianLinghuo`/`Wangyou`/`GujianFoodLiyutang`）应当**先用同样的最小改法试一遍**，
而不是先假定需要授权。

⇒ 台账 **7 → 6**（A 3 / B 3）；**累计已修 11 条**

---

## 154 第 987 轮：`XianjianLinghuo` 试了两种最小改法，**都不成立**（已回退）

### 试法一：把「固定 3 步」换成「推进到询问出现」

结果：**12 次推进内询问始终不出现**。用调试输出看状态：

```text
i=0 round=1 ready=3 td2=1 hp2=4 phase=Discard choice=None
i=2 round=2 ready=3 td2=1 hp2=4 phase=Judge   choice=None
i=7 round=2 ready=3 td2=1 hp2=4 phase=Judge   choice=None
```

⇒ `round` 到 3 时，2 号的 `turn_damage` 已被回合切换清零，
而该测试的伤害是在**更早的回合**打出的 ⇒ 触发条件 `turn_damage > 0 && round >= 3` **不同时成立**。

### 试法二：只把期待点前移

结果：同一断言失败，因为触发点并不在「2 号回合内」。

### 结论

`XianjianLinghuo` 与前两轮试的 `XianjianWangyou` 一样，**不是「改两行」能解决的**：
需要把整个测试的回合/伤害时序重新推导（谁在哪个回合造成伤害、冷却在第几轮置位、触发落在哪个阶段结束）。
⇒ 已全部 `git checkout` 回退，全量复归 **1540 passed** ✅

### 与 `WudianFuchou2` 的区别（为什么那条能成）

`WudianFuchou2` 的触发条件只依赖「标记里有存活者」——**与回合数无关**，
所以「窗口前移 + 状态判断」两行就够了。
`Linghuo`/`Wangyou` 的条件里含**回合计数**与**本回合伤害**，两者会随回合切换变化 ⇒ 时序敏感。

⇒ 这是「同属 A 类但难度不同」的具体证据。

---

## 155 第 990 轮：`GujianFoodLiyutang` 试改后**测试失败在「护甲=2 vs 1」**（已回退）

### 上游（`card/gujian/skill.js:261`）

```js
liyutang: {
  mark: "card",
  trigger: { player: "phaseEnd" },     // ← 每个阶段结束
  forced: true,
  intro: { content: "结束阶段，若你的体力值为全场最少或之一，你获得1点护甲（剩余 N 次）" },
  …
}
```

### 原生

```rust
// 臂
Phase { End, begin: true } + owner          // ← End 阶段「开始」，且钉死阶段
// step
if 体力为全场最少或之一 { change_hujia(owner, 1); } food_tick(...);
```

⇒ 既**钉错阶段**（End vs 任何阶段），又**错在 begin**（开始 vs 结束）。

### 改后失败原因

把元数据改 `PhaseEnd` + 臂改 `Phase { begin: false, .. }` 后：
护甲在**一个回合内被授予两次**（Play 结束、Discard 结束），
而测试 `g1_shahai_gujian/hujia.rs` 断言的是 `hujia == 1`。

**这不是实现错**：上游 `phaseEnd` + 「剩余 2 次」确实允许一回合内用两次。
⇒ **测试编码了更窄的行为**（只在 End 开始授予一次）。

⇒ 已回退，全量复归 **1540 passed** ✅
⇒ 该行归入「需重写测试推进脚本/断言」类；实现侧的改法（元数据 + 臂）已确认。

---

## 156 第 991 轮：**第 12 条真修复** —— `GujianFoodLiyutang`（6 → 5）

### 改了什么

| 侧 | 上游子技 `liyutang` | 原臂 | 新臂 |
|---|---|---|---|
| 元数据 | — | `PhaseBegin` | **`PhaseEnd`** |
| 时机 | `{player:'phaseEnd'}`（自己**每个阶段**结束）| `Phase { End, begin: true }` | **`Phase { begin: false, .. }` + owner** |

### 测试改了两处断言（都向同一个上游原文对齐）

| 断言 | 旧 | 新 | 依据 |
|---|---|---|---|
| 护甲数 | `hujia == 1` | **`== 2`** | 一回合内 Play 结束 + Discard 结束各授 1 点；上游 `phaseEnd` + 「剩余 2 次」 |
| 消耗后 | `hujia == 0` | **`== 1`** | 一次 1 点伤害只消耗 1 点护甲（2 → 1）|

⇒ **全量 1540 passed / 0 failed / 71 ignored** ✅

### 这条为什么能做、`XianjianLinghuo` 为什么不能

| 行 | 触发条件依赖 | 能否最小改 |
|---|---|---|
| `GujianFoodLiyutang` | 体力是否为全场最少（**与回合数无关**）| ✅ 改元数据+臂+两处断言 |
| `XianjianLinghuo`/`Wangyou` | **回合计数 + 本回合伤害**（随回合切换变化）| ❌ 必须重推整条时序 |

⇒ 台账 **6 → 5**（A 2 / B 3）；**累计已修 12 条**；`G20` 族**清零**
