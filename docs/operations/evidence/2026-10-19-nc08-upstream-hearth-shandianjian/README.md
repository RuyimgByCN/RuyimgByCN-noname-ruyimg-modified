# NC08 第⑦层样本 125：**来源 U07（hearth）** 锦囊 `shandianjian`（闪电箭）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未验收**（**规则差异**：被通用「杀」路径接管）
- **意义**：**hearth 包 12 张已转换卡采样完成**

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`shandianjian`）：

```js
shandianjian: {
    type: "trick", enable: true, cardnature: "thunder",
    filterTarget(card, player, target) { … return target.countCards("h") > 0; },
    async content(event, trigger, player) {
        const result = await event.chooseToShow(event, player, target);   // 目标展示一张手牌
        event.cards2 = cards;
        const showEvent = target.showCards(cards, `…因【闪电箭】展示的牌`);
        event.chooseToDiscard ??= async (event, player, target) => {
            const { discardPosition = "h", cards2,
                    filterDiscard = { suit: get.suit(cards2[0]) } } = event;   // 同花色
            const result = await player.chooseToDiscard(discardPosition, filterDiscard)…; … };
        …
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_shandianjian.rs`，**登记基线**）

| 观测点 | 实测 | 判读 |
|---|---|---|
| 发动 | `Pending` → **`respond_shan`**（要求目标出【闪】） | ✗ **上游无此询问** |
| 效果 | 1 号 **hp 4 → 3**（**受伤**） | ✗ 上游为「展示手牌 + 使用者弃同花色」 |

⇒ 原生把本卡当作**通用「杀」类**处理（响应【闪】+ 伤害），与上游语义**完全不同**。

## 3 源码核对（**交付但无实现、且无登记**）

| 检查 | 结果 |
|---|---|
| `grep -rli shandianjian crates/engine-native/src/` | **0 命中** |
| 包内 `cards.json` | **有**（已交付） |
| `content/standard.rs` 的 v40 注释 | **未提及**（该注释只列「灵魂之火/照明弹/急耗卡」） |
| `docs/**` 检索「闪电箭/shandianjian」 | **0 命中** |

⇒ **包内交付、无原生实现、无登记** ⇒ **第 46 项**（本样本只记录**受理基线**，**不写成通过**）。

## 4 未完成 / 待办

- [ ] **闪电箭语义实现**（第 46 项：展示手牌 → 使用者弃同花色 → 后续分支）
- [ ] 上游**未弃同花色**时的**雷伤口径**逐字确认
- [ ] 通用「杀」路径为何接管该卡（`CardKind` 归类核对）
