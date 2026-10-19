# NC08 第⑦层样本 112：**来源 U05（古剑）** 食物牌 `mizhilianou`（蜜汁藕）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未验收**（**差异登记**：红桃当桃的转化未实现）
- **意义**：**食物族最后一张** ⇒ 食物族 9 张「非归属效果」抽样**全覆盖**

## 1 上游源码（归档快照，逐字引用）

`.../gujian/skill.js` 的子技 `mizhilianou`：

```js
mizhilianou: {
    mark: "card", trigger: { player: "phaseAfter" }, forced: true,
    intro: { content: (storage, player) =>
        "你可以将一张红桃牌当作桃使用（剩余" + player.storage.mizhilianou_markcount + "回合）" },
    group: "mizhilianou_use",
    subSkill: { use: { enable: "chooseToUse", filterCard: { suit: "heart" },
                       position: "he", viewAs: { name: "tao" } } },   // 红桃 → 桃
    content() { player.storage.mizhilianou_markcount--; … },
},
```

## 2 验收结果（`nc08_upstream_gujian_mizhilianou.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| — | **对照组**：普通【桃】对自己 | **`Accepted`**（hp 3 → 4）⇒ **目标形态正确、夹具无问题** | 对照通过 |
| 1 | **红桃非桃牌当【桃】** | **`Rejected(IllegalTargets)`**（hp 不变） | **未验收**（第 41 项） |

**原生实现引用**：`content/host/step/gujian.rs:375`（`GujianFoodMizhilianou => Flow::Done`）——
该**空 step 本身合理**（转化应在**出牌路径**实现，同武圣/倾国）；但**行为面显示转化未实现**。

## 3 方法学（**对照组的价值**）

初版直接判定「转化未实现」时，**先补对照组**（普通【桃】）——它暴露了**真正的**夹具问题
（【桃】须用 `targets: vec![]` 而非 `vec![0]`）；修正后再测，才得到**可信**的结论：
**夹具正确 + 转化缺失**。若省略对照组，会把**夹具问题**误记为**功能缺失**。

## 4 未完成 / 待办

- [ ] **红桃当桃**的转化（第 41 项）；`position: "he"`（**含装备区**，牌区族）
- [ ] `markcount` 递减与归零移除
- [ ] 上游 `card.js` 侧的食物牌「本体归属」（第 37 项）
