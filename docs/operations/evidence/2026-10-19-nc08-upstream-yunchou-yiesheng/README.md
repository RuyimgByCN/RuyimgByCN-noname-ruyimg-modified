# NC08 第⑦层样本 74：**来源 U05（杀海拾遗·运筹）** 武将技 `yiesheng`（野升）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`yiesheng`）：

```js
yiesheng: {
    enable: "phaseUse",
    filterCard: { color: "black" },                       // 仅黑色手牌
    filter(event, player) { return player.countCards("h", { color: "black" }) > 0; },
    selectCard: [1, Infinity],                            // 任意张
    prompt: "弃置任意张黑色手牌并摸等量的牌",
    content() { player.draw(cards.length); },             // 摸等量
},
```

## 2 验收结果（`nc08_upstream_yunchou_yiesheng.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `filterCard: { color: "black" }` | **行为面**：手持 2 黑 + 1 红 → 询问 `yiesheng_discard` 中**红色不入候选**（`红色入候选=false`） | 通过 |
| 3 | `player.draw(cards.length)` | **行为面**：弃 **2** 张黑 → **摸 2** → 手牌 **3 → 3**（净变化 0）；**红牌仍在**、**黑牌已离手** | 通过 |
| 1 | `enable: phaseUse` + `filter` | 原生 `yunchou.rs:221`（候选 = 黑桃/草花；`min 1 / max n / allow_cancel`；`Act` 段弃后摸等量） | 通过 |

## 3 夹具要点

- **双产物桥接**；技能目录名 **`ycnyiesheng`**（包面向 `host:yiesheng`）；
- 布阵须取**两张不同的**黑色牌（按目录下标 `take(2)`）——重复放同一张牌只会得到一张。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **取消**分支（`allow_cancel: true` → 不弃牌）
- [ ] `check`（AI 弃牌评分）与 `ai.order` 策略面；`prompt` 文案表现面
- [ ] 运筹其余武将技（纵火/烧营/隙焰/红莲/归汉/厉焰×2/精械/巧思/奸雄变体/蛊惑变体/悲歌变体）
