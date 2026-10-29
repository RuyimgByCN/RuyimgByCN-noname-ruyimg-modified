# NC08 第⑦层样本 121：**来源 U07（hearth）** 锦囊 `jihuocard`（激活）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（摸 1）** + **1 项未验收**（临时技 `jihuocard2` 未实现 → 第 43 项）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`jihuocard`）：

```js
jihuocard: {
    type: "trick", enable: true, toself: true,
    filterTarget(card, player, target) { return player === target; },
    selectTarget: -1, modTarget: true,
    content() {
        if (_status.currentPhase === target) target.addTempSkill("jihuocard2");   // 自己回合内 → 临时技
        target.draw();                                                          // 摸 1
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_jihuocard.rs`）

| # | 要点 | 实测 | 结果 |
|---|---|---|---|
| 2 | **摸 1** | **牌堆 123 → 122**；手牌 **1 → 1**（打出 −1、摸 1） | 通过 |
| 2 | **临时技 `jihuocard2`** | **候选名探测命中 = []**；**全库 grep `jihuocard2` = 0 命中** ⇒ **未实现** | **未验收（第 43 项）** |

**原生实现引用**：`content/host/intrinsic.rs:1024`（`CardKind::Jihuocard` →
`HostBehavior::JihuocardDraw`）；`content/host/step/skills_status.rs:790`（`JihuocardDraw`，**仅摸牌**）；
`constants.rs:210`（`HT_JIHUO_KEY = "ht:jihuocard"`，**卡键**非临时技名）。

## 3 方法（**命名纪律 + 双向核对**）

① 先按**候选名**探测（含前缀形式）；② 再**全库 grep**（原生 src + 全部包）⇒ 两者皆空才判「未实现」，
**排除「名称不匹配」**（样本 97/99 的教训）。

## 4 未完成 / 待办

- [ ] **临时技 `jihuocard2`**（第 43 项）：授予 + 效果 + 到期移除
- [ ] `modTarget` 影响面
