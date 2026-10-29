# NC08 第⑦层样本 64：**来源 U05（杀海拾遗·swd）** 卡牌 `hslingjian_shijianhuisu`（时间回溯）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 1 项差异登记）；**来源 U05 的样本深度 +1**（该包 90 卡 / 79 将，此前仅数例）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1392`：

```js
hslingjian_shijianhuisu: {
    type: "hslingjian", vanish: true, enable: true, derivation: true,
    filterTarget(card, player, target) {
        return target !== player && target.countCards("e") > 0;   // 非自己 + 有装备
    },
    content() {
        var es = target.getCards("e");
        target.gain(es);                                          // 装备区全部牌收回手牌
        target.$gain2(es);
    },
},
```

同文件另有 `hslingjian_shijianhuisu_equip1..4`（装备态子技：装备后摸牌 / 距离修饰等）——
属该灵剑的**装备态**效果，**未纳入本样本**（登记）。

## 2 验收结果（`nc08_upstream_swd_shijianhuisu.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `target.gain(target.getCards("e"))` | **行为面**：1 号装备 1 张 → 0 号打出时间回溯指定 1 号 → **装备 1 → 0**、**手牌 0 → 1**，且收回的正是那张装备牌 | 通过 |
| 1 | `filterTarget`：`target !== player && countCards("e") > 0` | **登记差异**：原生 `SwdShijianhuisuCard` 分支**只做效果**（`if alive`），**未见**对「目标装备区有牌」的校验（是否由 `filter_swd` 统一承载**未核对**） | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/swd_lingjian_cards.rs:119`（遍历 `equips` → `revoke_equip_skills`
+ `hand.push`；本体入弃堆）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] `_equip1..4` 装备态子技（装备后摸牌 / `globalTo` 修饰）
- [ ] 「目标装备区有牌」门槛是否由 `filter_swd` 承载（核对后决定是否升级为差异）
- [ ] 样本 65+：其余来源/卡牌（**swd 包 90 卡**中仍有大量未采样）
