# 缺陷台账（机器可读，**102 条**）—— 2026-10-19 第 765 轮

> **这是唯一权威的缺陷计数**。此前的「42 条」「93 条」都是**会话中的增量记账**；
> 本台账是**脚本生成 + 去重**的结果：`scripts/report_defect_ledger.py --write`。

## 1 计

| 类 | 条数 |
|---|---:|
| **A（改 pattern / 删分支 / 补动作，不动枚举）** | **50** |
| **B（需扩枚举或引擎能力）** | **52** |
| **合计** | **102** |

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
