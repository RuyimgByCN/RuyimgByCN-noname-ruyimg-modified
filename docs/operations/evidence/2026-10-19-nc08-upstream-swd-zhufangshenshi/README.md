# NC08 第⑦层样本 158：**来源 U06（swd）** 法器 `zhufangshenshi`（朱fang神石）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · `global` 窗口）**，**未发现差异**
- 意义：**`global` 卡窗口第 7 张验证**（第 5 张通过），且**为 `Phase` 事件问题的归结提供反证**

## 1 上游源码（**逐字**）

```js
zhufangshenshi: { type: "jiqi", global: "g_zhufangshenshi" },   // 卡本体无 content
g_zhufangshenshi: {
    trigger: { player: "useCardAfter" },                         // 本人使用牌后
    forced: true, popup: false,
    filter(event, player) { return event.card.name === "zhufangshenshi"; },  // 仅本卡
    content() { player.draw(); },                                // 摸 1
},
```

## 2 验收结果（`nc08_upstream_swd_zhufangshenshi.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 1 | 可使用 | `PlayCard { targets: [1] }` ⇒ **`Accepted`** | ✅ |
| 2 | **使用后摸 1（`forced`）** | 牌堆 **196 → 195** | ✅ |

## 3 **归结对第 55/56 项的意义**

本卡窗口是 **`useCardAfter`**（**非 `Phase` 事件**）⇒ **正常触发** ✅；
而 `qinglongzhigui`/`huanglinzhicong` 依赖 **`Phase{ begin: true }`** ⇒ **未触发** ❌
⇒ **支持「问题特异于 `Phase{begin:true}` 事件，而非宿主派发整体失效」**。

## 4 未完成（登记）

- [ ] `popup: false` 的表现面
- [ ] 「**非本卡**」的负例（使用其它牌时不应触发）
