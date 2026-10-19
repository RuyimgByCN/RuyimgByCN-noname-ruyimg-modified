# NC08 第⑦层样本 67：**来源 U05（杀海拾遗·swd）** 锦囊 `yangpijuan`（羊皮卷）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 **3 项差异登记**）；**来源 U05 样本深度 +1**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js`（`yangpijuan`）：

```js
yangpijuan: {
    type: "trick", enable: true, toself: true, modTarget: true,
    filterTarget(card, player, target) { return target === player; },   // 仅自己
    content() {
        "step 0";
        target.chooseControl("basic", "trick", "equip", () => choice)
              .set("prompt", "选择一种卡牌类型");                        // ① 选类别
        "step 1";
        var list = get.inpile(result.control, "trick");
        list = list.randomGets(3);                                       // ② 该类随机 3 张
        // → 择一获得（createCard 凭空造牌）
    },
},
```

## 2 验收结果（`nc08_upstream_swd_yangpijuan.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | 两段：`chooseControl` → `randomGets(3)` → 择一 | **行为面**：实测询问序列 **`["swd_yangpijuan_category","swd_yangpijuan_pick"]`**、类别选项 **[0,1,2]**、择定牌 **76** 进入手牌（手牌 1 → 1：打出本体 −1、获得 1） | 通过 |
| 3 | `get.inpile(class)` + `createCard` | **差异①**：上游**凭空造牌** ↔ 原生从**牌堆发现**（原生注释自述并登记） | 通过（**差异登记**） |
| 3 | 候选牌来源 | **差异②**：上游造出的候选**不属于牌堆** ↔ 原生候选**择定前留在牌堆**（注释自述「挂起态区域完整」） | 通过（**差异登记**） |
| 3 | `randomGets(3)` | **差异③**：上游恒取 3 张 ↔ 原生 `pool.len().min(3)`（牌堆该类不足 3 张时**更少**） | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/swd_card_effects.rs:452`（三段机 `Act` → 类别询问 →
按类别过滤牌堆 + Fisher-Yates 取 ≤3 → 择一 → 择定后才离牌堆入手）。

## 3 夹具要点

- 该卡为 **`toself: true`** → **`targets: vec![]`** 受理（实测 `vec![0]` 会 `Rejected(IllegalTargets)`）；
  对照：同批 `mujiaren`（`selectTarget: -1` + `filterTarget: target === player`）须 `vec![0]`。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①②③的口径与修复（与「生成 vs 实体」族合并评估）
- [ ] 类别 AI 默认值（`choice` 启发式）属 AI 面
- [ ] 样本 68+：其余来源/卡牌（**swd 90 卡**仍有大量未采样）
