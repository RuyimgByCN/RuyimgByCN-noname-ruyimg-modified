# `playpackconfig` 上游口径整理（U03/L04 差异 #5 的补充，2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- 来源：`archive/upstream/noname/apps/core/extension/cardpile/extension.js:67` 起（**逐字**摘录）

## 1 上游机制（**逐段**）

```js
// ① 按类型的配置乘数
const getn = (i, j) => Math.round(data[i][j] * parseFloat(lib.config[`cardpile_${i}_playpackconfig`]));

// ② 需要补的总量
let num = 0;
for (const i in data) for (const j in data[i]) num += getn(i, j);
let dn = (num * (lib.card.list.length - data.total)) / (data.total - num);
if (dn > 1000) dn = 1000;                    // ③ **上限 1000**

// ④ 按比例补牌，点数由 Math.random 决定
if (dn > 0) for (const i in data) for (const j in data[i]) {
    let n = Math.round((dn * getn(i, j)) / num);
    while (n--) lib.card.list.push([j, rand(), i]);   // rand() = ceil(Math.random()*13)
}
```

⇒ **要点**：
1. **配置键**：`cardpile_<类型>_playpackconfig`（**每个牌型一个乘数**）；
2. **配置缺省时** `parseFloat(undefined)` = `NaN` ⇒ `getn` = `NaN` ⇒ `num` = `NaN` ⇒ `dn` = `NaN` ⇒
   **`if (dn > 0)` 为 false** ⇒ **不补牌**（与 U03/L04 差异 #4「**未配置时不追加**」一致 ✅）；
3. **补牌点数**用 `Math.random`（与差异 #3「**点数不是 `Math.random`**」对应）；
4. **上限 1000**。

## 2 原生对应

原生有 `content/pile.rs` 的 **`scale_plan` / `extension_pack_from_scale` / `snapshot_scale` / `restore_scale`**
（**另一套**机制）——**未逐项对照**（与 U03/L04 差异 #5 一致）。

## 3 本轮的贡献

**差异 #5「`playpackconfig` 未逐项对照」**本身**不变**，但**对照所需的「上游口径」已整理成文**：
配置键格式、`dn` 公式、上限 1000、`Math.random` 点数、**未配置即不补**。

## 4 ✅ 对照已完成（同日，读 `content/pile.rs:144-200` 的 `scale_plan`）

| # | 要点 | 上游 | 原生 `scale_plan` | 一致 |
|---|---|---|---|---|
| 1 | 配置键/乘数 | `cardpile_<类型>_playpackconfig` | `factors.get(&entry.card)` | ✅ |
| 2 | **未配置 ⇒ 不补** | `NaN` ⇒ `dn > 0` 为 false | **`return Ok(ScaleStatus::Unconfigured)`** | ✅ |
| 3 | **`dn` 公式** | `num*(list-total)/(total-num)` | `num*(deck_len-total)/(total-num)` | ✅ |
| 4 | **上限 1000** | `if (dn > 1000) dn = 1000` | `if … || dn > 1000.0 { dn = 1000.0 }` | ✅ |
| 5 | **补牌点数** | **`Math.random`** | **确定性** | ❌ 差异 #3（已登记） |
| — | 取整方式 | `Math.round` | `.round()` | ✅ |
| — | 分母为 0 的处理 | JS `Infinity/NaN` 自然传播 | **显式** `Infinity`/`NaN`，并归入 1000 上限或 `NoAddition` | ✅ 更严格 |

⇒ **`playpackconfig` 的机制本身与上游一致**（配置键/未配置不补/公式/上限/取整），
**唯一差异是「补牌点数」**（上游 `Math.random`，原生确定性）——**已在差异 #3 登记** ✅。

## 5 结论

**U03/L04 差异 #5 从「未逐项对照」升级为「已逐项对照：机制一致，点数差异见 #3」**。
