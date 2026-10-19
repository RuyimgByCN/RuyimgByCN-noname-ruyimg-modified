# NC08 第⑦层样本 97：**来源 U05（古剑）** 锦囊 `heilonglinpian`（黑龙鳞片）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**（另 1 项**待核对**）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`heilonglinpian`）：

```js
heilonglinpian: {
    type: "trick", enable: true, toself: true,
    filterTarget(card, player, target) { return target === player; },   // 仅自己
    selectTarget: -1, modTarget: true,
    content() {
        target.changeHujia();                                   // +1 护甲
        target.addTempSkill("heilonglinpian", { player: "phaseBegin" });   // 临时技至自己回合开始
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_heilonglinpian.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `target.changeHujia()` | **行为面**：使用者**护甲 0 → 1** | 通过 |
| 2 | `addTempSkill("heilonglinpian", …)` | **待核对**：以**上游键**查询临时技返回 `false`——原生可能使用**带前缀的目录名**（如 `gj_...`）⇒ **未判定为缺失** | 待核对 |

**原生实现引用**：`content/host/constants.rs:84`（`GJ_HEILONGLINPIAN_CARD_KEY = "gj:heilonglinpian"`）。

## 3 未完成 / 待办

- [ ] **临时技的目录名**核对（再判定是否授予成功）
- [ ] 临时技的**到期移除**（至自己回合开始，需跨回合驱动）
- [ ] `modTarget` 的额外影响面
- [ ] 古剑包其余卡牌
