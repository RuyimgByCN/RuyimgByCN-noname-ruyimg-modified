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

## 4 待办（**可做**）

- [ ] 读原生 `scale_plan` 的实现，与本文 §1 的**四个要点**逐项对照；
- [ ] 若原生采用**确定性点数**（差异 #3 已登记），对照时只比**数量**、不比点数；
- [ ] 对照完成后，把 U03/L04 差异 #5 从「未对照」改为「已对照（差异清单）」。
