# NC08 第⑦层样本 65：**来源 U05（杀海拾遗·swd）** 机关牌 `gouhunluo`（勾魂锣）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（**原登记差异已更正为「已校验」**）；**来源 U05 样本深度 +1**

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
| 1 | `filterTarget: !target.hasSkill("gouhunluo")` | **门槛已校验（2026-10-19 更正）**：原生在**出牌命令面** `engine/play_card_targets.rs:32` 校验 `gouhunluo_rounds > 0 → IllegalTargets`；**行为面实测**：第一张命中后目标 `rounds=3`，**第二张对同一目标 `Rejected(IllegalTargets)`** | 通过（**非差异**） |

> **更正说明（诚实记录）**：初版把该项登记为「差异」，原因是**只读了效果分支、未查完整校验链**
> —— 属本仓库已沉淀方法学「断言缺陷前先读完整校验链」的**再次命中**（第 5 条）。现以**行为面**
> （两张勾魂锣）钉死：门槛**确实生效**。

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
