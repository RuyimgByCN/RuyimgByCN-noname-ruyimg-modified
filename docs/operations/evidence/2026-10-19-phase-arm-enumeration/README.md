# 相位钉死臂**全量枚举**：48 条（2026-10-19，第 729 轮）

> 此前「G14 有 10 条」「G20 有 5 条」都是**人工回查**得出的。
> 本轮把「人工抽样」换成「**脚本枚举**」：把宿主过滤里**所有钉死某个 `Phase` 的臂**全部列出，逐条与上游 trigger 对照。

## 1 怎么枚举

`scripts/report_phase_arms.py`：

1. 扫 `crates/engine-native-content/src/content/host/**` 里所有含 `Phase::(Judge|Draw|Play|Discard|End), begin: <bool>` 的行；
2. 向上回溯到最近的 `HostBehavior::X =>` 得到行为名；
3. 行为 → `host:` 键（`keys.rs`）→ 包内技能名（`packages/extensions/*/assets/*.json`）；
4. 上游 trigger：按**六种「查不到」形态**逐一尝试（裸名 / `lib.skill.X =` / 前后下划线 / 加去扩展前缀），并跳过卡片定义；
5. 用**阶段名映射**对照：**`Play` ↔ `phaseUse…`**、`End` ↔ `phaseJieshu…`、`Judge`/`Draw`/`Discard` 同名。

## ⚠ 修订（第 736 轮）：**48 → 53 条**，`G14` 11 → **12**

**第一版只识别「解构绑定」写法**（`phase: Phase::Judge, begin: true`），
**漏掉了「相等比较」写法**（`Phase { player, phase, begin } if *phase == Phase::Judge && *begin`）。
后者多了 **5 条**：

| 行为 | 钉的阶段 | 上游 | 判定 |
|---|---|---|---|
| **`SwdHujing`** | **`Judge`** | **`player: 'phaseBegin'`** | ❗ **G14 第 12 条**（新发现）|
| `SwdQinglongzhiguiHeld` | `Judge` | 未定位（装备子技）| 🔍 REVIEW |
| `SwdHuanglinzhicongHeld` | `Judge` | 未定位 | 🔍 REVIEW |
| `SwdXielv` | `End` | `phaseDiscardEnd` | ⚠️ 相邻相位（登记差异）|
| `SwdWangchenHeld` | `End` | `phaseDiscardEnd` | ⚠️ 相邻相位（登记差异）|

⇒ **脚本已修**（`EQ_RE` + 负向 `*begin` 判定）✅

### 再修订（第 737 轮）：`SwdQinglongzhiguiHeld` / `SwdHuanglinzhicongHeld` 也是 G14

这两条此前记「上游未定位」。补查发现它们的效果挂在**机巧卡片的 `global:` 技能**上：

```js
qinglongzhigui:    { type:'jiqi', global:'g_qinglongzhigui' }        // 卡片定义
g_qinglongzhigui:  { trigger: { player: 'phaseBegin' } }             // ← 真实触发面
```

⇒ **上游 `phaseBegin`（任意阶段），原生钉死 `Judge`** ⇒ **G14 第 13、14 条** ❗

**这是「查不到」的第 7 种形态**：**效果不在同名技能上，而在卡片的 `global: 'g_<name>'` 技能里** ✅

⇒ **`G14` 最终 = 14 条**；单测期望同步 **53 / G14 14** ✅

## 2 枚举结果：**53 条**

| 判定 | 条数 | 含义 |
|---|---:|---|
| ✅ **OK** | **21** | 上游 `phaseXxxBegin`/`phaseXxxEnd` 与原生 `begin` 一致 |
| ❗ **G14（钉死阶段）** | **14** | 上游 **`phaseBegin`（任意阶段）**，原生钉死 `Judge` **或 `End`** |
| ⚠️ **G20（早一步）** | **5** | 上游 **`phaseEnd`（全部阶段后）**，原生 `End, begin: true`（结束阶段开始）|
| ⚠️ **G15（`phaseAfter`）** | **1** | `fuchou2`：上游 `global: phaseAfter`，无对应档位 |
| ⚠️ **G15（「之前」档）** | **1** | `fenglue`：上游 `phaseUseBefore`，原生 `begin: true` |
| ❗ **自加分支** | **1** | `huajin2`：上游只有 `damageBegin`，原生 OR 了阶段分支 |
| ✅ **语义选择** | **1** | `tanhuadong`（上游代码与文案矛盾，见规则基准）|
| 🔍 **REVIEW** | **11** | **新增 5 条里 4 条进 REVIEW**（见修订节）|
| **合计** | **53** | |

## 3 那 7 条 REVIEW **全都是已知的**

| 行为 | 判定 | 已知身份 |
|---|---|---|
| `BossBaiyi` | 上游无 trigger | **凭空新增触发**（已在守卫 `baiyi_must_not_invent_a_phase_trigger`）|
| `HlssFengqi` | 上游 `phaseZhunbeiBegin` | **G18（准备阶段缺档）** |
| `Xiuluo` | 上游 `phaseZhunbeiBegin` | **G18** |
| `Miedao` | 上游无 trigger | **不可达登记** |
| `GwentBirinState` / `GwentCiguState` / `GwentQinpenState` | 上游未定位 | **`gwent` 三个状态技**，此前已登记为不可达/未定位 |

⇒ **枚举没有发现新问题** ✅ —— 这是本轮最有价值的结果：**人工审计与脚本枚举互相印证**。

## 4 交叉验证：枚举 = 人工

| 缺陷族 | 人工回查 | **枚举** | 一致？ |
|---|---:|---:|---|
| **G14** | 10 + `ziyangdan` 1 = **11** | **11** | ✅ |
| **G20** | **5** | **5** | ✅ |
| **G18** | 2 | **2** | ✅ |
| **`huajin2` 自加分支** | 1 | **1** | ✅ |
| **`tanhuadong` 语义选择** | 1 | **1** | ✅ |

⇒ **两条独立路径给出同一组数字** ✅ —— 比单一路径可信得多。

## 5 复现

```bash
python3 -B scripts/report_phase_arms.py           # TSV + 汇总
python3 -B scripts/report_phase_arms.py --check   # 有 REVIEW 行时 exit=1
```

---

## 9 ✅ 三条 `gwent` 天气状态技定案（第 738 轮）

上游定义在 `杀海拾遗/card/gwent/skill.js`（**不在 `昆特牌/` 扩展下**，所以首轮「未定位」）：

| 原生行为 | 场景 | 上游 trigger | 语义 | 判定 |
|---|---|---|---|---|
| `GwentBirinState` | 蔽日浓雾 | **`player: "phaseUseAfter"`** | 不能用【杀】**直到下一个出牌阶段结束** | ✅ **近似可接受**（`After` 与 `Play, begin: false` 相邻）|
| `GwentQinpenState` | 倾盆大雨 | **`player: "phaseDiscardAfter"`** | 手牌上限 −1 直到**下一个弃牌阶段结束** | ✅ **近似可接受** |
| **`GwentCiguState`** | **刺骨寒霜** | **`player: "phaseDrawBegin"`** | **下个摸牌阶段摸牌数 −1** | ❗ **确认缺陷** |

### `GwentCiguState` 为什么是缺陷

- 上游在 **`phaseDrawBegin`（摸牌阶段开始）** 就把 `event.num` 减 1 ⇒ **影响本阶段的摸牌数**；
- 原生挂在 **`Draw, begin: false`（摸牌阶段结束）** ⇒ **摸牌已经结束**，此时再减 1 **不会影响任何东西** ⇒ **效果实际失效** ❗

⇒ **新增 A 类缺陷 1 条**（**修法：`Draw, begin: true` + 在摸牌数确定前生效**）⇒ **A 47 → 48、总 92 → 93**

### 另两条为什么算「可接受」

`phaseUseAfter` / `phaseDiscardAfter` 是「阶段**之后**」，原生 `begin: false` 是「阶段**结束**」——
在原生的事件模型里两者是**同一帧**（阶段结束即该阶段之后），**不影响任何结算顺序** ⇒ **登记差异，不计缺陷** ✅

---

## 10 ✅ 第 768 轮：`miedao` 结案，**53 条全部有判定（REVIEW 归零）**

`miedao` 上游是**复合技**：

```js
miedao:  { group: ['miedao1', 'miedao2'], ai: { threaten: 1.4 } }   // 父技能无 trigger
miedao1: { trigger: { player: 'phaseDrawBegin' } }                  // 摸牌阶段开始
miedao2: { trigger: { player: 'phaseDiscardBegin' } }               // 弃牌阶段开始
```

**原生**：`Phase::Discard, begin: true` ⇒ **对应 `miedao2` ✓**
⇒ **判定「OK」**（**此前记 REVIEW，是因为只看了父技能** ⚠️ —— **与 `lexue` 同一形态**）

**`miedao` 已补进 `MANUAL_TRIGGERS`**（复合技无法靠单一名字解析）✅

### ⇒ 相位枚举最终账

| 判定 | 条数 |
|---|---:|
| ✅ **OK** | **24** |
| ❗ G14 | **14** |
| ⚠️ G20 | **5** |
| ❗ G18 | **2** |
| ✅ 相邻相位（可接受）| **2** |
| ⚠️ G15（`phaseAfter` / 「之前」）| **2** |
| ✅ 语义选择 | **1** |
| ❗ G22（晚一步）| **1** |
| ❗ 凭空新增 | **1** |
| ❗ 自加分支 | **1** |
| **合计** | **53** ✅ **REVIEW 0** |

⇒ **`report_phase_arms.py --check` 现已 exit 0** ✅
