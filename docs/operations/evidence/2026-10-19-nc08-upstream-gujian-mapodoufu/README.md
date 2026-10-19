# NC08 第⑦层样本 93：**来源 U05（古剑）** 食物牌 `mapodoufu`（麻婆豆腐）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未验收**（**用牌效果缺失**，与样本 89 同现象）
- **作用**：判断该缺口是**个案**还是**食物牌共性**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`mapodoufu`）：

```js
mapodoufu: {
    type: "food", enable: true,
    filterTarget(card, player, target) { return !target.hasSkill("mapodoufu"); },
    content() {
        if (target === targets[0] && cards.length) target.$gain2(cards);   // 目标获得此牌
        target.storage.mapodoufu = card;
        target.storage.mapodoufu_markcount = 2;
        target.addSkill("mapodoufu");
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_mapodoufu.rs`）

| 观测点 | 实测 | 判读 |
|---|---|---|
| 发动 | `Accepted` | 可发动 ✅ |
| **目标获得此牌** | **手牌 0 → 0（含此牌=false）** | ✗ |
| 该牌去向 | **弃堆含此牌=true** | ✗ 直接入弃堆 |

**原生实现引用**：`content/host/step/gujian.rs:212`（食物机通用映射
`CardKind::Mapodoufu => (GJ_FOOD_MAPODOUFU_KEY, GJ_FOOD_ROUNDS_DEFAULT)`）、`:267`（分支）。

## 3 结论：**食物牌共性（2/2 抽样）**

| 样本 | 卡 | 「目标获得此牌」 |
|---|---|---|
| 89 | `liyutang`（鲤鱼汤） | ✗ 牌入弃堆 |
| **93** | **`mapodoufu`（麻婆豆腐）** | ✗ 牌入弃堆 |

⇒ 「用牌效果（`$gain2` + `storage` + `markcount` + `addSkill`）缺失」**至少是食物牌共性**；
子技的**逐轮 tick 已实现**（如鲤鱼汤：阶段开始体力最少 → +1 护甲）——缺的是**用牌那一段**。

**建议**：对古剑包**全部食物牌**（春饼/青团/油羹/茉莉茶/虾饺/蜜汁莲藕/鲈鱼汤/古洞羹…）
按本检查点做**逐卡抽样**，并检查 `GJ_FOOD_*` 机器的**用牌入口**。

## 4 未完成 / 待办

- [ ] 食物牌**用牌入口**修复（第 37 项）
- [ ] 计数的逐轮结算行为面
- [ ] 门槛（`!hasSkill`）
- [ ] 古剑包其余卡牌
