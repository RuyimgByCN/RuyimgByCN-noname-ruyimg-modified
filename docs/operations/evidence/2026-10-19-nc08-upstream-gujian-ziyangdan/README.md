# NC08 第⑦层样本 95：**来源 U05（古剑）** 基本牌 `ziyangdan`（紫阳丹）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**
- **作用**：**反证**「食物机卡全坏」的粗放推论——本卡用同一 `GJ_FOOD_*` 机，但效果**正常**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`ziyangdan`）：

```js
ziyangdan: {
    type: "basic", enable: true, filterTarget: true,
    content() {
        target.changeHujia(3);                             // +3 护甲
        if (target.hasSkill("ziyangdan")) target.storage.ziyangdan += 3;
        else target.addSkill("ziyangdan");
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_ziyangdan.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `target.changeHujia(3)` | **行为面**：1 号**护甲 0 → 3**；**子技授予成功**（`gj_food_ziyangdan=true`） | 通过 |

**原生实现引用**：`content/host/step/gujian.rs:219`（`CardKind::Ziyangdan => (GJ_FOOD_ZIYANGDAN_KEY, …)`）、
`:424`（`GujianFoodZiyangdan`）。

## 3 与第 37 项的关系（**反证价值**）

本卡使用**同一族**的 `GJ_FOOD_*` 机器，但**效果正常**（护甲 +3 且授技成功）——
其上游 `content` **没有 `$gain2`** 段 ⇒ **与第 37 项的精确化结论一致**：
缺失的是 **`$gain2`（本体归属）**，而**机器本身与授技/计数面正常**。

⇒ **不支持**「食物机卡一律失效」的粗放推论；**支持**「仅带 `$gain2` 的食物牌缺本体归属」。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] `storage` 累计与计时子技的逐轮结算
- [ ] `ai.result` 策略面
- [ ] 古剑包其余卡牌
