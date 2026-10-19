# NC08 第⑦层样本 92：**来源 U05（古剑）** 机关牌 `mianlijinzhen`（棉里针）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`mianlijinzhen`）：

```js
mianlijinzhen: {
    type: "jiguan", enable: true,
    filterTarget(card, player, target) { return target.hp >= player.hp; },   // 体力不低于使用者
    content() {
        "step 0"; target.draw();      // 目标摸 1
        "step 1"; target.loseHp();    // 目标失去 1 点体力（体力流失，非伤害）
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_mianlijinzhen.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `target.draw()` + `target.loseHp()` | **行为面**：1 号手牌 **0 → 1**（摸 1）、hp **4 → 3**（失去 1 点体力） | 通过 |
| 1 | `filterTarget`：`target.hp >= player.hp` | **门槛反例**：目标体力**低于**使用者 → **`Rejected(IllegalTargets)`** | 通过 |

**原生实现引用**：`content/host/step/gujian.rs:71`（`GujianMianliCard`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] `loseHp` 与 `damage` 的语义差异（需护甲载体）
- [ ] `ai.result` 策略面
- [ ] 古剑包其余卡牌
