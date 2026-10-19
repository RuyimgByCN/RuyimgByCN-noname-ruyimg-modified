# NC08 第⑦层样本 48：`qianxun`（谦逊，**标准面** / 陆逊）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过**（含 1 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-qianxun.js.txt](upstream-qianxun.js.txt)）

```js
qianxun: {
    mod: {
        targetEnabled(card, player, target, now) {
            if (card.name === "shunshou" || card.name === "lebu") return false;
        },
    },
    audio: 2,
},
```

## 2 验收结果（`nc08_upstream_qianxun.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `"shunshou"` | 持谦逊者被【顺手牵羊】指定 → **`Rejected(IllegalTargets)`** | 通过 |
| 1 | `"lebu"` | 持谦逊者被【乐不思蜀】指定 → **`Rejected(IllegalTargets)`** | 通过 |
| 1 | 仅这两张牌（**反例**） | 【过河拆桥】指定 → **`Accepted`**（谦逊不拦其它锦囊） | 通过 |
| 1 | `mod.targetEnabled` | **差异登记**：上游 **`mod` 统一拦截**；原生 **2 处显式校验**（顺手牵羊 + `kind == CardKind::Le`），与两张牌**一一对应** → 语义等价、**维护性风险同 `kongcheng`** | 通过（**差异登记**） |

**原生实现引用**：`engine/play_card.rs:688`（顺手牵羊分支）、`:791`（`kind == CardKind::Le` 分支）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] `audio` 表现面
- [ ] 与 `kongcheng` 同属「`mod` ↔ 显式校验」**维护性风险**族（当前 2 例：kongcheng、qianxun）
- [ ] 样本 49+：其余来源/技能（标准面技能 **27 个样本**；来源覆盖 **23/37**）
