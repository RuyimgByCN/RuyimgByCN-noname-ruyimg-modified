# NC08 第⑦层样本 81：**来源 U05（杀海拾遗·运筹）** 武将技 `shaoying`（烧营）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（源码事实 + 红牌烧营**行为面** + 黑色反例）；**原「效果未执行」判定已撤回**（系**夹具问题**）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`shaoying`）：

```js
shaoying: {
    trigger: { source: "damageAfter" }, direct: true,
    filter(event) { return event.nature === "fire"; },                 // 仅火焰伤害
    content() {
        "step 0";
        player.chooseTarget(get.prompt("shaoying"), (card, player, target) =>
            get.distance(trigger.player, target) <= 1 && trigger.player !== target);   // 玩家选目标
        "step 1";
        if (result.bool) { var card = get.cards()[0]; card.discard(); player.showCard… }
        // 红色 → 该目标受 1 点火伤
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_shaoying.rs`）

| # | 判据 | 结果 |
|---|---|---|
| 1+2+3 | **源码事实**：原生 `yunchou.rs:372` 为「座位序首合法目标（≠受伤者/≠自己/存活/距离≤1）→ `deck.pop()` 亮出 → **红色**才 `Damage(Fire,1)`」；注释自述「上游玩家选目标」 | 通过 |
| 3 | **反例（黑色）**：牌堆顶黑色 → 2 号不受火伤 | 通过（**注意空过风险**，见下） |
| 2+3 | **红牌**：亮出红牌 → 首合法目标（2 号）受 1 点火伤 | **通过**：修正夹具后实测 `[4,4,4,4] → [4,3,3,4]`（1 号受 0 号火伤、**2 号受烧营火伤**） |

### 夹具更正（同日，**关键**）

初版用「按花色挑一张牌 → 传其**牌名**给 `debug_set_deck_top`」——但该助手**按牌种**取牌，
同种牌可能有多张**不同花色** → 实际置顶的可能是**黑**牌 ⇒ 误判为「效果未执行」。
**修法**：置顶后**回读 `deck.last()` 核对花色**（必要时换名重试）。修正后红牌分支**通过**。

## 3 登记差异

**目标的选择权**：上游 `chooseTarget`（**玩家自选**）↔ 原生**座位序首合法目标**（无询问）
→ 与「**链杀**（`luweiyan2` 自动首目标）」**同族** ⇒ **「目标选择权」族 2 例**。

## 4 与样本 80 的关系

两例（纵火/烧营）**同现象**：确认出现、效果未执行 → 已合并为修复清单**第 34 项**
（含「宿主技能效果面普查」建议）。

## 5 未完成 / 待办

- [ ] 第 34 项定位（机器/触发运行时；受拆分阻塞）
- [ ] `ai`（目标评分）策略面；牌堆为空分支
- [ ] 运筹其余武将技（归汉/悲歌变体）
