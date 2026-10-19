# NC08 第⑦层样本 82：**来源 U05（运筹）** 武将技 `olbeige`（悲歌 / refresh 变体）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含行为面 · 红桃分支）+ **1 项差异登记**（草花分支取牌）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/character/refresh/skill.js:3957`（`olbeige`；运筹包蔡文姬技能之一，
`character.js:51` 列为 `["beige", "guihan"]`）：

```js
olbeige: {
    trigger: { global: "damageEnd" }, logTarget: "player",
    filter(event, player) {
        return event.card && event.card.name == "sha" && event.player.isIn() && player.countCards("he") > 0;
    },
    prompt2: "令其进行判定，然后你可根据判定结果，弃置一张牌并令其执行对应效果。",
    async content(event, trigger, player) {
        let result = await trigger.player.judge().forResult();     // 受伤者判定
        switch (result.suit) { case "heart": … 令其回复 1 点体力 … }   // 按花色分支
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_beige.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2+3 | 判定 → 弃 1 张 → 红桃分支回复 | **行为面**：询问 **`["trigger_confirm","beige_discard"]`**、`弃牌=true`；1 号被【杀】打至 3 后**回复 1** ⇒ **hp `4 → 4`**（净零） | 通过 |
| 3 | 草花分支取牌 | **差异登记**：原生注释自述「**草花来源弃 2 取首两张确定性**」（上游为玩家自选）→ 与「**取牌选择权**」族**同族**（**第 5 例**） | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/yunchou.rs`（`YunchouBeige`：`draw_judge_card()` → 弃 1 张（可取消）
→ 按判定花色分支 + 收回奖励）。

## 3 夹具要点

- **判定牌置顶须回读核对花色**（新纪律）：`debug_set_deck_top(name)` 按**牌种**取牌，同种多花色会
  **静默失配** → 本样本循环尝试牌名并**回读 `deck.last()`** 确认为**红桃**后才继续；
- 悲歌持有者（2 号）须有 `he` 牌（弃一张的费用）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **黑桃/方块/草花**三个分支的行为面（本样本只覆盖红桃）
- [ ] **草花分支取牌**随「取牌选择权」族统一评估
- [ ] `addExpose`（AI 面）
- [ ] 运筹其余武将技（归汉）
