# NC08 第⑦层样本 146：**来源 U06（swd）** 机器牌 `cangchizhibi`（苍螭之璧）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**样本不可行**（**该卡属已确认的转换缺口**）⇒ **固化为缺口回归用例**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js`（`cangchizhibi`）：

```js
cangchizhibi: {
    type: "jiqi", global: "g_cangchizhibi",
    filterTarget(card, player, target) { return target !== player; },     // 其他角色
    content() { "step 0"; if (target.countCards("h")) { var next = target.chooseToDiscard… } },
},
```

## 2 为何不可行（**与台账第 117 行互证**）

- 台账第 117 行的**转换缺口清单**（swd **11 项**）已含 **`cangchizhibi`（苍螭之璧，`jiqi`）**；
- 该清单经**双向核对**（`crates/engine-native/src/` + 全部 `packages/extensions/*`）**0 命中** ⇒ **未转换**；
- 实测：`catalog` 中**不存在**该卡（本用例断言 `存在=false` ✅）⇒ **无法构造行为面样本**。

## 3 处置

| 用例 | 作用 |
|---|---|
| `upstream_cangchizhibi_is_a_known_conversion_gap` | **通过**：断言「目录中不存在」⇒ 把**缺口**固化为**可执行事实**（缺口清单的**回归防线**） |
| `upstream_cangchizhibi_target_discards` | **`#[ignore]`**：**转换后**的验收条件（目标弃一张手牌） |

**注**：若将来该卡被转换，**第一个用例会失败**，提示把第二个用例改为行为面样本 ⇒ **缺口闭合可被自动发现**。

## 4 未完成 / 待办

- [ ] **转换 `cangchizhibi`**（第 117 行清单 swd 11 项之一）
- [ ] 转换后补行为面样本（含 `global` 响应窗口用法）
