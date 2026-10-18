# NC08 第⑦层样本 11：`chunbing`（春饼，U05 古剑卡牌子包）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个古剑卡牌子包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项验收通过**；其中 2 项为**登记差异的事实固定**（计数建模、本体归属）

## 1 上游源码（归档快照，逐字引用；见 [upstream-chunbing.js.txt](upstream-chunbing.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`：

```js
chunbing: {
    fullskin: true,
    type: "food",
    enable: true,
    filterTarget(card, player, target) {
        return !target.hasSkill("chunbing");     // 目标未持有该食物技能
    },
    content() {
        target.$gain2(cards);                    // 目标**获得**本体
        target.storage.chunbing = card;
        target.storage.chunbing_markcount = 5;   // 5 个标记
        target.addSkill("chunbing");             // 授予监听技
    },
    ai: { … },
},
```

## 2 验收结果（`nc08_upstream_chunbing.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `filterTarget: !target.hasSkill("chunbing")` | **已持有该食物技能**的目标再次被指定 → **被拒** | 通过 |
| 2 | `chunbing_markcount = 5` | 目标获得监听技实例且**计数 = 5**（原生以 `rounds = 5` 建模，常量 `GJ_FOOD_ROUNDS_CHUNBING = 5`） | 通过（**差异登记**：标记 ↔ 回合） |
| 3 | `target.$gain2(cards)` | **本体归属**：原生**入弃牌堆**（上游为目标获得） | 通过（**差异登记**） |

**原生实现引用**：`kernel/intrinsic.rs:244`（食物牌 → `GujianFoodGrant`）、
`content/host/step/gujian.rs:197`（授予监听技实例 + 弃置本体）、
`content/host/constants.rs:12`（`GJ_FOOD_ROUNDS_CHUNBING = 5`）。

## 3 工程发现（写入证据）

- 目录牌名可能带**命名空间/花色点数前后缀**（如 `gw_yanziyaoshui#heart8`）→ 取牌应按
  **`contains` 匹配**而非精确名/`starts_with`（本样本与 gwent 样本各踩一次）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 样本 12+：其余来源（当前 **10/37 来源**）
- [ ] 古剑其余 39 张卡的上游样本（本样本为 1/40）
- [ ] 差异（标记↔回合、本体归属）的最终口径判定
- [ ] 逐命令对照工具；表现/策略面（`fullskin`/`ai`）不在规则对照范围
