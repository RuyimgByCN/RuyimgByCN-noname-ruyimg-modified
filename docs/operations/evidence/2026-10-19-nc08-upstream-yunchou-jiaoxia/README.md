# NC08 第⑦层样本 71：**来源 U05（杀海拾遗·运筹）** 武将技 `diy_jiaoxia`（狡黠）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面 · 两分支）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`diy_jiaoxia`）：

```js
diy_jiaoxia: {
    trigger: { target: "useCardToBegin" },                      // 成为牌的目标时
    filter(event, player) { return event.card && get.color(event.card) === "red"; },   // 红色牌
    frequent: true,                                             // 可多次
    content() { player.draw(); },                               // 摸 1
},
```

## 2 验收结果（`nc08_upstream_yunchou_jiaoxia.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | 红色 → `player.draw()` | **行为面（正例）**：1 号（持狡黠）成为**红色**【杀】的目标 → **手牌净增 1** | 通过 |
| 2 | `get.color(...) === "red"` | **行为面（反例）**：成为**黑色**【杀】目标 → **净增 0** | 通过 |
| 1 | `useCardToBegin` | 原生过滤 `filtering/common.rs:473`：`CardUsed` 且 `targets.contains(&owner)` 且花色 **Heart\|Diamond**；效果 `yunchou.rs:317` `draw_cards(owner, 1)` | 通过 |

## 3 夹具要点

- **双产物桥接**（武将面在 `character.json`）；技能目录名 **`ycnjiaoxia`**（包面向 `host:jiaoxia`）；
- 用【杀】作红/黑对照：**净变化**只取手牌数差（受伤/出闪不影响手牌数）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] `frequent` 的**多次**语义（同一回合多次成为红牌目标）
- [ ] 装备牌/锦囊牌作为红牌来源的分支
- [ ] `ai.effect` 策略面
- [ ] 运筹其余武将技（纵火/烧营/隙焰/红莲/野升/归汉/厉焰×2/精械/巧思/蝉缘/狂骨变体/奸雄变体/蛊惑变体/悲歌变体）
