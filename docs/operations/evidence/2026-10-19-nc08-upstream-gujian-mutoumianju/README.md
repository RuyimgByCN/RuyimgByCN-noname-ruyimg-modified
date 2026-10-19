# NC08 第⑦层样本 103：**来源 U05（古剑）** 装备牌 `mutoumianju`（木头面具）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`mutoumianju`）：

```js
mutoumianju: {
    fullskin: true, type: "equip", subtype: "equip2",     // 防具位
    skills: ["mutoumianju_skill"],                        // 装备即授予技能
    ai: { equipValue: 4 },
    onLose() { … delete player.getStat().skill.mutoumianju_skill; },
},
```

## 2 验收结果（`nc08_upstream_gujian_mutoumianju.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `skills: ["mutoumianju_skill"]` | **行为面**：装备后 **装备区 0 → 1** 且 **`gj_mutoumianju_skill` 已授予（true）** | 通过 |

**原生实现引用**：`content/host/constants.rs:112`（`GJ_MUTOU_SKILL_KEY = "gj_mutoumianju_skill"`）；
规则基准 golden 表第 53 行（「木头面具（**view-as 装备技**——`equip_skills` + `once_used`，
`EquipSlot::Armor` 注册）」）。

## 3 夹具要点（**纪律应用**）

技能名**取自原生常量**（`gj_mutoumianju_skill`），**不按上游键猜名** —— 直接应用了
本会话固化的纪律（样本 97/99 的命名教训）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] `onLose`（**失去装备时的技能移除**）
- [ ] `mutoumianju_skill` 的**效果面**
- [ ] 古剑包其余卡牌（玉衡系 3 张装备 + 食物族 9 张的**非归属效果**）
