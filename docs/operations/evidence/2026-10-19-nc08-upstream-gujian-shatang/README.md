# NC08 第⑦层样本 **100**：**来源 U05（古剑）** 锦囊 `shatang`（沙棠）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 双效果顺序不变量）**，**未发现差异**
- **里程碑**：本样本为 **NC08 第 100 个样本**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`shatang`）：

```js
shatang: {
    type: "trick", enable: true, filterTarget: true,
    cardcolor: "red", cardnature: "fire",
    content() {
        "step 0"; target.damage("fire");      // 1 点火焰伤害
        "step 1"; target.changeHujia();       // +1 护甲（伤害之后）
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_shatang.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `damage("fire")` → `changeHujia()` | **行为面（双效果）**：1 号 **hp 4 → 3**（火伤）**且** **护甲 0 → 1**（+1 护甲）⇒ 两步**均执行** | 通过 |

**原生实现引用**：`content/host/step/jiguan.rs`（`HostBehavior::JiguanShatangCard`）；
`kernel/intrinsic.rs:409`（`CardKind::Shatang | Heilonglinpian | Liutouge` 分支）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] `cardnature: fire` 的**属性面**（火伤修饰/藤甲等载体）
- [ ] `ai.result` 策略面
- [ ] 古剑包其余卡牌（机关族余：`bingpotong` 天女散花、`liutouge` 六骰格）
