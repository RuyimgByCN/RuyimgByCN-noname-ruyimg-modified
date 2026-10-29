# NC08 第⑦层样本 129：**来源 U09（mtg）** 地牌 `mtg_haidao`（海岛）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 三分支）**，**未发现差异**
- **意义**：**mtg 来源的首个行为面样本**（该来源此前仅有转换/受理层核对）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_haidao`）：

```js
mtg_haidao: {
    type: "land", fullborder: "wood",
    enable(card, player) { return !player.hasSkill("land_used"); },   // 每回合限一次
    notarget: true,                                                   // 无目标
    content() {
        "step 0"; game.changeLand("mtg_haidao", player);
                  if (player.isHealthy()) { player.changeHujia(); event.finish(); }   // 满血 → +1 护甲
                  else { player.chooseToDiscard("he", "海岛")… }                       // 否则弃 1 张（可取消）
        "step 1"; if (result.bool) player.recover();                   // 弃了 → 回复 1
                  else player.changeHujia();                           // 取消 → +1 护甲
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_haidao.rs`，**三分支**）

| 分支 | 实测 | 结果 |
|---|---|---|
| **满血** | hp **+0**、护甲 **+1**（**无询问**） | 通过 |
| **受伤且弃 1 张** | hp **+1**、护甲 **+0**（询问 `mtg_haidao_discard`） | 通过 |
| **受伤且取消** | hp **+0**、护甲 **+1**（询问 `mtg_haidao_discard`） | 通过 |

**原生实现引用**：`content/host/intrinsic.rs:717`（`CardKind::Haidao` → `HostBehavior::MtgHaidaoLand`）。

## 3 未完成 / 待办

- [ ] `game.changeLand`（**地牌变更**）的完整语义
- [ ] `land_used`（**每回合限一次**）的**重置时机**
- [ ] mtg 其余 9 张地牌
