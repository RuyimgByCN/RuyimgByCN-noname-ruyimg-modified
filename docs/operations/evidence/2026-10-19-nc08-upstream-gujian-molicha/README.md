# NC08 第⑦层样本 109：**来源 U05（古剑）** 食物牌 `molicha`（茉莉茶）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 限制面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`.../gujian/skill.js` 的子技 `molicha`：

```js
molicha: {
    mark: "card", trigger: { player: "phaseAfter" }, forced: true,
    intro: { content: (storage, player) =>
        "你不能成为其他角色的黑色牌的目标（剩余" + player.storage.molicha_markcount + "回合）" },
    mod: { targetEnabled(card, player, target) {
        if (player !== target && get.color(card) === "black") return false; } },
    content() { player.storage.molicha_markcount--; … },
},
```

## 2 验收结果（`nc08_upstream_gujian_molicha.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `mod.targetEnabled` 对**黑色**牌返回 false | **行为面**：1 号用**黑**【杀】打 0 号（已喝茉莉茶）→ **`Rejected(IllegalTargets)`** | 通过 |
| 2 | 仅限黑色（`get.color(card) === "black"`） | **对照**：**红**【杀】→ **`Accepted`** | 通过 |

**实测**：`子技=true；黑牌 → Rejected(IllegalTargets)；红牌 → Accepted`

## 3 **重要对照**：与石灰粉（第 38 项）相反

| 卡 | 宿主 step | 限制是否生效 |
|---|---|---|
| **`molicha` 茉莉茶** | `GujianFoodMolicha => Flow::Done`（**空**） | ✅ **生效**（限制在**目标校验路径**实现） |
| `shihuifen` 石灰粉 | `GujianFoodShihuifenBan => Flow::Done`（**空**） | ✗ **未生效**（第 38 项） |

⇒ **「宿主 step 为空」既可能「在别处实现」也可能「确实缺失」** ⇒ **必须逐卡行为面验证**，
不能由 step 是否为空推断（本会话两条相反实例即为证据）。

## 4 说明

原生注释 `step/gujian.rs:334`（「摸牌阶段：额外摸 1，然后弃 1」）实际属于 **`GujianFoodXiajiao`（虾饺）**；
本样本据**上游 `skill.js`** 更正了对茉莉茶效果的归属（**不是**摸牌阶段效果）。

## 5 未完成 / 待办

- [ ] 倒计时（`markcount`）递减与归零移除
- [ ] `mark: "card"` 标记面
- [ ] 食物族其余卡（虾饺/蜜汁藕/酉羹）
