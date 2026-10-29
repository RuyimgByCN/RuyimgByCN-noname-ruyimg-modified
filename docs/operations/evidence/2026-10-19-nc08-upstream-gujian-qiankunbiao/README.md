# NC08 第⑦层样本 84：**来源 U05（古剑）** 机关牌 `qiankunbiao`（乾坤镖）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 额外目标）** + **1 项差异登记**（原生更宽松）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:870`（`qiankunbiao`）：

```js
qiankunbiao: {
    type: "jiguan", enable: true, wuxieable: true,
    filterTarget(card, player, target) { return target !== player && target.countCards("he") > 0; },
    changeTarget(player, targets) {                    // 额外影响
        game.filterPlayer(current =>
            get.distance(targets[0], current, "pure") === 1 && current.countCards("he"), targets);
    },
    content() { var he = target.getCards("he"); if (he.length) target.discard(he.randomGet()).delay = false; },
},
```

## 2 验收结果（`nc08_upstream_gujian_qiankunbiao.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | `changeTarget`（距离 1 且 he 有牌）+ 随机弃 1 张 | **行为面**：首目标 1 号手牌 **1 → 0**、**邻座 2 号 1 → 0**（额外影响生效） | 通过 |
| 1 | `filterTarget`：`countCards("he") > 0` | **差异登记（原生更宽松）**：目标 `he` 无牌时**仍受理**（`Accepted`）——原生 `play_card.rs:1016` 只校验「恰 1 名其他存活目标」 | 登记（第 35 项） |

**原生实现引用**：`engine/play_card.rs:1016`（目标校验）；`kernel/intrinsic.rs:251`（映射宿主键
`GJ_QIANKUNBIAO_CARD_KEY`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **补 `he` 有牌门槛**（第 35 项；**注意** `shenhuofeiya` 不要求该门槛，须分别校验）
- [ ] 「**随机弃一张**」是否与原生一致（上游 `he.randomGet()`）
- [ ] `wuxieable` 无懈面；`contentAfter` 的 `game.delay(0.5)` 表现面
