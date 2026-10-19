# NC08 第⑦层样本 130：**来源 U10（玩点论杀）** 锦囊 `wangmeizhike`（望梅止渴）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 两支）**，**未发现差异**
- **意义**：**玩点论杀来源的首个行为面样本**（该来源此前仅有转换/受理层核对）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js`（`wangmeizhike`）：

```js
wangmeizhike: {
    type: "trick", enable: true, filterTarget: true,
    async content(event, trigger, player) {
        const target = event.targets[0];
        let rec = false;
        if (target.isMinHp() && target.isDamaged()) { await target.recover(); rec = true; }
        if (target.isMinHandcard()) { await target.draw(rec ? 1 : 2); }
    },
},
```

## 2 验收结果（`nc08_upstream_wudian_wangmeizhike.rs`，**两支**）

| 分支 | 实测 | 结果 |
|---|---|---|
| **体力最少且受伤**（手牌也最少） | hp **+1**、手牌 **+1**（**回复 1 + 摸 1**） | 通过 |
| **未受伤**（仅手牌最少） | hp **+0**、手牌 **+2**（**摸 2**） | 通过 |

**原生实现引用**：宿主行为（玩点论杀锦囊机）。

## 3 未完成 / 待办

- [ ] `isMinHp` / `isMinHandcard` 的**并列最少**语义
- [ ] `ai` 面
- [ ] 玩点论杀其余卡（`sanjian`/`tiesuo`/`youdishenru`）
