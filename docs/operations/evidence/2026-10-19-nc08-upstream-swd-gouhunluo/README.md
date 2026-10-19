# NC08 第⑦层样本 65：**来源 U05（杀海拾遗·swd）** 机关牌 `gouhunluo`（勾魂锣）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 1 项差异登记）；**来源 U05 样本深度 +1**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:592`：

```js
gouhunluo: {
    enable: true, type: "jiguan", wuxieable: true,
    filterTarget(card, player, target) { return !target.hasSkill("gouhunluo"); },
    content() {
        target.storage.gouhunluo = 3;        // 3 轮倒计时
        target.storage.gouhunluo2 = player;  // 来源记录
        target.addSkill("gouhunluo");
    },
},
```

## 2 验收结果（`nc08_upstream_swd_gouhunluo.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `storage.gouhunluo = 3` + `storage.gouhunluo2 = player` | **行为面**：0 号打出勾魂锣指定 1 号 → 目标 **`rounds=3`**、**`source=Some(0)`**（实测 `rounds=3 source=Some(0)`） | 通过 |
| 1 | `filterTarget: !target.hasSkill("gouhunluo")` | **登记差异**：原生分支只做效果（`if alive`），**未见**对「目标已持该技」的拒绝（是否由 `filter_swd` 承载**未核对**） | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/swd_card_effects.rs:613`（`gouhunluo_rounds = 3`；
`gouhunluo_source = Some(self.owner)`；授予 `SWD_GOUHUNLUO_SUB_KEY` 实例）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **3 轮倒计时的逐轮结算行为**（需跨多回合驱动）
- [ ] 「目标已持该技」门槛是否由 `filter_swd` 承载（核对后决定是否升级为差异）
- [ ] `wuxieable` 的无懈面
- [ ] 样本 66+：其余来源/卡牌（**swd 包 90 卡**仍有大量未采样）
