# 相位钉死臂**全量枚举**：48 条（2026-10-19，第 729 轮）

> 此前「G14 有 10 条」「G20 有 5 条」都是**人工回查**得出的。
> 本轮把「人工抽样」换成「**脚本枚举**」：把宿主过滤里**所有钉死某个 `Phase` 的臂**全部列出，逐条与上游 trigger 对照。

## 1 怎么枚举

`scripts/report_phase_arms.py`：

1. 扫 `crates/engine-native/src/content/host/**` 里所有含 `Phase::(Judge|Draw|Play|Discard|End), begin: <bool>` 的行；
2. 向上回溯到最近的 `HostBehavior::X =>` 得到行为名；
3. 行为 → `host:` 键（`keys.rs`）→ 包内技能名（`packages/extensions/*/assets/*.json`）；
4. 上游 trigger：按**六种「查不到」形态**逐一尝试（裸名 / `lib.skill.X =` / 前后下划线 / 加去扩展前缀），并跳过卡片定义；
5. 用**阶段名映射**对照：**`Play` ↔ `phaseUse…`**、`End` ↔ `phaseJieshu…`、`Judge`/`Draw`/`Discard` 同名。

## 2 枚举结果：**48 条**

| 判定 | 条数 | 含义 |
|---|---:|---|
| ✅ **OK** | **21** | 上游 `phaseXxxBegin`/`phaseXxxEnd` 与原生 `begin` 一致 |
| ❗ **G14（钉死阶段）** | **11** | 上游 **`phaseBegin`（任意阶段）**，原生钉死 `Judge` **或 `End`** |
| ⚠️ **G20（早一步）** | **5** | 上游 **`phaseEnd`（全部阶段后）**，原生 `End, begin: true`（结束阶段开始）|
| ⚠️ **G15（`phaseAfter`）** | **1** | `fuchou2`：上游 `global: phaseAfter`，无对应档位 |
| ⚠️ **G15（「之前」档）** | **1** | `fenglue`：上游 `phaseUseBefore`，原生 `begin: true` |
| ❗ **自加分支** | **1** | `huajin2`：上游只有 `damageBegin`，原生 OR 了阶段分支 |
| ✅ **语义选择** | **1** | `tanhuadong`（上游代码与文案矛盾，见规则基准）|
| 🔍 **REVIEW** | **7** | 见下（**全部是已知项**）|
| **合计** | **48** | |

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
