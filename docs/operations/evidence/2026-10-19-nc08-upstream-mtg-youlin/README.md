# NC08 第⑦层样本 139：**来源 U09（mtg）** 地牌 `mtg_youlin`（幽林）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**；**登记差异**为原生**自述**的「池收窄」（属既有族）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_youlin`）：

```js
mtg_youlin: {
    type: "land", enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        game.changeLand("mtg_youlin", player);
        var list = get.inpile("trick");                   // 全普通锦囊池
        while (list.length) {
            var name = list.randomRemove();               // 随机取一张
            if (lib.card[name].multitarget) continue;
            … var target = targets.randomRemove();        // 随机选目标
              if (… get.effect(target, {name}, player, player) > 0) {   // 正收益判定
                  player.useCard({ name: name }, target); …
```

## 2 原生实现（**源码注释自述**，`content/host/step/mtg.rs:683-687`）

> 幽林（mtg 第九批，规则 v108）：Act 换地 → **池=牌堆中的决斗/无中生有**
> （**登记差异：上游全普通锦囊池 + 正收益 AI 判定；原生收窄为内核结算已确证者**，无懈窗口照常）
> → **随机一张随机其他存活目标**（无中无目标）→ emit 结算；池空空发。

## 3 验收结果（`nc08_upstream_mtg_youlin.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **使用后确有锦囊结算** | 0 号手牌 **+1**；牌堆 **−3**；各方掉血 **全 0**；**`询问=[]`** | 通过 |

**读数解释**：牌堆 **−3** = 取用一张锦囊（−1）+ 该锦囊为**无中生有**（摸 2，−2）⇒ 手牌 **−1 + 2 = +1**；
掉血全 0 与「无中生有」一致（非决斗）⇒ **行为面与原生自述的池（决斗/无中生有）自洽**。

## 4 未完成 / 待办

- [ ] **池收窄**的具体清单与上游对齐（既有「池收窄」族，如集智）
- [ ] `land_used` 每回合重置
- [ ] `mtg_yixialan`（**无 `content`** 的地牌）——仅登记，不另设样本
