# NC08 第⑦层样本 86：**来源 U05（古剑）** 机关牌 `longxugou`（龙须钩）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过**（门槛反例）+ **1 项差异登记**（获得后未装备）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`longxugou`）：

```js
longxugou: {
    type: "jiguan", enable: true, wuxieable: true,
    filterTarget(card, player, target) {
        return target !== player && target.countGainableCards(player, "e");   // 其他角色 + 可获得装备
    },
    content() {
        "step 0"; var es = target.getGainableCards(player, "e");
                  if (es.length) player.choosePlayerCard("e", target, true).set("es", es)…;  // 使用者自选
        "step 1"; if (result.bool) { target.$give(result.links[0], player);
                                     target.lose(result.links[0], ui.special); event.card = result.links[0]; }
        "step 2"; if (event.card && get.position(event.card) === "s") player.equip(event.card);   // **自己装备之**
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_longxugou.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `countGainableCards(player, "e")` | **门槛反例**：目标装备区无牌 → **`Rejected(IllegalTargets)`**（已正确实现） | 通过 |
| 2 | `choosePlayerCard` + **`player.equip`** | **差异登记**：询问 `gj_longxu_pick`（**自选，与上游一致** ✅）、目标装备 `1 → 0` ✅，但使用者装备 **`0 → 0`**、该牌进入**手牌** ✗ ⇒ **只获得未装备** | 登记（第 36 项） |

**原生实现引用**：`engine/play_card.rs:1695`（目标校验：恰 1 名其他存活**且有装备**的目标）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **获得后装备**（第 36 项）
- [ ] `wuxieable` 无懈面；`game.delay()` 表现面
- [ ] 古剑包其余卡牌
