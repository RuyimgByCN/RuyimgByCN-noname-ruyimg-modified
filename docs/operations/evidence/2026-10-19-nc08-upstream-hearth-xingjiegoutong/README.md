# NC08 第⑦层样本 118：**来源 U07（hearth）** 锦囊 `xingjiegoutong`（星界沟通）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 三步）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`xingjiegoutong`）：

```js
xingjiegoutong: {
    type: "trick", enable: true,
    selectTarget: -1, modTarget: true, toself: true,
    filterTarget(card, player, target) { return player === target; },
    content() {
        target.gainMaxHp();                        // 体力上限 +1
        target.recover();                          // 回复 1 点体力
        target.discard(target.getCards("h"));      // 弃置全部手牌
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_xingjiegoutong.rs`）

| # | 步骤 | 实测 | 结果 |
|---|---|---|---|
| 2 | **体力上限 +1** | **4 → 5** | 通过 |
| 2 | **回复 1 点体力** | **3 → 4** | 通过 |
| 2 | **弃置全部手牌** | **3 → 0** | 通过 |

**原生实现引用**：`kernel/intrinsic.rs:443/470`（`CardKind::Xingjiegoutong`）；
规则基准 golden 表第 55 行（「星界沟通（**上限+1 回 1 弃全部**）」）。

## 3 未完成 / 待办

- [ ] `modTarget` 的影响面
- [ ] 体力上限变化对**后续结算**（濒死阈值等）的影响
- [ ] hearth 最后 1 卡（传送门）
