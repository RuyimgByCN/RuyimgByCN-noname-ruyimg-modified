# NC08 第⑦层样本 131：**来源 U10（玩点论杀）** 响应锦囊 `youdishenru`（诱敌深入）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**（并行为面确认基准登记差异）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js`（`youdishenru`）：

```js
youdishenru: {
    type: "trick", notarget: true, wuxieable: true, global: "g_youdishenru",
    content() {
        "step 0"; var info = …youdiinfo…; info.evt.cancel();          // 取消该【杀】
                  event.source = info.source; event.source.addSkill("youdishenru");
        "step 1"; var next = event.source.chooseToUse({ name: "sha" }, player, -1,
                      "对…使用一张杀，或受到1点伤害");                  // 来源须出杀，否则受 1 点伤害
        "step 2"; if (result.bool) { if (storage.youdishenru) event.goto(1); … }   // 连用循环
                  else { event.… }
    },
},
```

## 2 验收结果（`nc08_upstream_wudian_youdishenru.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 1 | **取消该【杀】** | 目标（1 号）**掉血 0** | 通过 |
| 2 | 来源**不出杀 ⇒ 受 1 点伤害** | 来源（0 号）**掉血 1** | 通过 |
| 3 | **登记差异**「无连用循环」 | 行为面一致（出杀后不再进入循环） | 确认 |

**原生实现引用**：`engine/play_card.rs:1313`（`CardKind::Youdishenru`）。

## 3 夹具要点（**新增**）

「诱敌深入」的**响应窗口形态**为 **`respond_shan`**（要求出【闪】的窗口），
**该牌作为可选项之一出现**（`options: [Pass, Card(133)]`）⇒ 识别窗口应**同时按「选项含该牌」与「询问名」**
判断（本会话金莲珠/石灰粉的教训：窗口形态多样）。

## 4 未完成 / 待办

- [ ] **连用循环**（基准已登记为「无」）
- [ ] `wuxieable` 无懈面
