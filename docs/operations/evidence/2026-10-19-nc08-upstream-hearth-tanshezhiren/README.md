# NC08 第⑦层样本 117：**来源 U07（hearth）** 锦囊 `tanshezhiren`（坦蛇至人）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 链式次数）**，**未发现差异**（并行为面确认基准登记差异）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`tanshezhiren`）：

```js
tanshezhiren: {
    type: "trick", enable: true,
    filterTarget(card, player, target) { return target === player; },
    selectTarget: -1, modTarget: true,
    content() {
        "step 0"; event.current = target;
                  event.num = game.countPlayer(); if (event.num % 2 === 0) event.num--;   // 奇数
        "step 1"; if (event.num) {
            var enemies = event.current.getEnemies(); enemies.remove(player);
            … var enemy = enemies.randomGet();                       // 随机
              enemy.discard(enemy.getCards("h").randomGet());        // 随机弃 1 张
              event.current = enemy; event.num--; event.redo();      // 链式
        }
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_tanshezhiren.rs`）

| # | 要点 | 实测 | 结果 |
|---|---|---|---|
| 2 | **链式弃牌次数**（4 人 ⇒ `num` 取奇 = **3**） | 1..3 号手牌 `[1,1,1] → [0,0,0]`，**合计弃 3 张** | 通过 |
| — | **登记差异的行为面确认** | **`询问=[]`** ⇒ 原生为**座位序 + 首张**（自动），上游为**两处 `randomGet()`**；与基准 golden 表第 55 行「**随机改确定性**」一致 | 确认 |

**原生实现引用**：宿主行为（`Hearth*Tanshezhiren*`；基准 golden 表第 55 行登记其确定性口径）。

## 3 未完成 / 待办

- [ ] `modTarget` 的影响面；`recastable`（上游已注释）
- [ ] **链式路径**（座位序具体经过哪些角色）与**手牌为空时提前终止**的分支
- [ ] hearth 其余卡（星界沟通/传送门）
