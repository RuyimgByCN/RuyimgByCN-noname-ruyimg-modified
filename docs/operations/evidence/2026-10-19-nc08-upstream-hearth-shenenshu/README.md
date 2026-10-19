# NC08 第⑦层样本 122：**来源 U07（hearth）** 锦囊 `shenenshu`（神恩术）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未验收**（**效果未实现**，**源码自述「暂缓」**）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`shenenshu`）：

```js
shenenshu: {
    type: "trick", enable: true,
    selectTarget: -1, filterTarget(card, player, target) { return target === player; },
    modTarget: true,
    content() {
        "step 0"; var cards = target.getCards("h");
                  if (cards.length) target.lose(cards)._triggered = null;   // 失去全部手牌
                  event.num = 1 + cards.length;                             // 数量 = 1 + 原手牌数
        "step 1"; var list = get.typeCard("basic"); list.remove("du");
                  for (i < event.num) cards.push(game.createCard(list.randomGet()));
                  target.directgain(cards);                                 // 获得之
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_shenenshu.rs`）

| 观测点 | 实测 | 判读 |
|---|---|---|
| 发动 | `Accepted` | 可发动 ✅ |
| **失去全部原手牌** | **原非基本手牌已全部失去 = false**（手牌仍为原 2 张） | ✗ 未执行 |
| **获得 1 + 原手牌数 张基本牌** | 手牌 **2 张**（其中基本牌 **1** 张）⇒ **未获得新牌** | ✗ 未执行 |

## 3 **源码自述「暂缓」（决定性依据）**

`content/standard.rs:52-53` 版本注释原文：

> v40：G1 杀海拾遗·炉石子包切片——**灵魂之火/照明弹/急耗卡**（CardKind 三种 + 宿主行为；
> **`shenenshu` 等复杂牌暂缓**）。

⇒ **原生自己登记该卡为「暂缓」**（**非未记录的缺口**）⇒ 本样本按**已知暂缓**登记，
用例以 `#[ignore]` 作为实现后的验收条件（**第 44 项**）。

## 4 未完成 / 待办

- [ ] **神恩术效果实现**（第 44 项：失去全部手牌 → 生成 1+n 张基本牌）
- [ ] 生成牌的**具体牌名分布**（上游随机）；「排除毒」的边界
