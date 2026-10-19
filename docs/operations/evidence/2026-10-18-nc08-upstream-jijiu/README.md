# NC08 第⑦层样本 30：`jijiu`（急救，**标准面** / 华佗）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-jijiu.js.txt](upstream-jijiu.js.txt)）

```js
jijiu: {
    enable: "chooseToUse",
    viewAsFilter(player) {
        return player !== _status.currentPhase          // 回合外
            && player.hasCards("hes", { color: "red" }); // 持有红色牌
    },
    filterCard(card) { return get.color(card) === "red"; },
    position: "hes",
    viewAs: { name: "tao" },
    prompt: "将一张红色牌当桃使用",
},
```

## 2 验收结果（`nc08_upstream_jijiu.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `filterCard` 红 + `viewAs: tao` | 原生 `is_tao_like` **确有**「持急救时红桃/方块可当桃」的判定（源码事实） | 通过 |
| 1 | 技能门槛 | 判定块内**仅在 `has_skill(responder, d)`** 时扩面 | 通过 |
| 2 | `player !== _status.currentPhase` | **差异登记①**：原生 `jijiu` 判定块**无回合限制** → 自己回合内也可用（**原生更宽松**） | 通过（**差异登记**） |

**原生实现引用**：`snap/asks.rs:78`（`is_tao_like`：`kind` 为桃/酒/遁虚双 → 真；否则持急救且
红桃/方块 → 真）。

## 3 登记差异说明（不夸大）

- **①回合限制缺失**：上游要求**回合外**（`player !== _status.currentPhase`）；原生只判花色为红
  → **自己回合内濒死救援时也可用红牌当桃**（上游不允许）→ **原生更宽松**；
- **②牌区**：上游 `position: "hes"` 含装备/判定区；原生按牌 id 判定、**未区分牌区**
  （本样本仅以源码事实固定）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **差异①修复**：`is_tao_like` 加入回合外判定（`snap/asks.rs` **未跟踪** → 并入待实施修复清单）
- [ ] 濒死救援的**完整行为面**（需构造濒死场景；本样本只做源码事实与门槛）
- [ ] 样本 31+：其余来源（当前 **21/37 来源**；标准面技能 **12 个样本**）
