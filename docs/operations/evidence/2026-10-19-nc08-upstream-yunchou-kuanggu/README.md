# NC08 第⑦层样本 73：**来源 U05（杀海拾遗·运筹）** 武将技 `diykuanggu`（狂骨·变体）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面 · 三分支）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`diykuanggu`）：

```js
diykuanggu: {
    trigger: { source: "damageEnd" },
    filter(event, player) {
        if (player.isHealthy()) return get.distance(trigger.player, player, "attack") > 1;
        return true;
    },
    getIndex(event, player, triggername) { return event.num; },
    forced: true,
    async content(event, trigger, player) {
        if (get.distance(trigger.player, player, "attack") > 1) await player.draw();
        else await player.recover();
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_kuanggu.rs`，2 项）

| # | 分支 | 判据 | 结果 |
|---|---|---|---|
| 1 | **相邻 + 未满血** | 造成伤害后 **hp +1**、手牌 +0（**回复 1**） | 通过 |
| 2 | **距离 > 1**（满血） | 造成伤害后 **手牌 +1**、hp +0（**摸 1**） | 通过 |
| 3 | **相邻 + 满血** | **不触发**（hp +0、手牌 +0） | 通过 |

**实测**：`①hp+1/手牌+0；②hp+0/手牌+1；③hp+0/手牌+0` —— 三分支**逐点对齐**。

**原生实现引用**：`content/host/step/yunchou.rs:436`（距离>1 → `draw_cards`；否则未满血 → `recover`）；
过滤 `filtering/common.rs:526`（来源为本人 且（未满血 **或** 距离>1））。

## 3 夹具要点

- **双产物桥接**；技能目录名 **`ycndiykuanggu`**（包面向 `host:diykuanggu`）；
- 座位距离：0↔1 = **1**（相邻）、0↔2 = **2**（远）——本样本无马匹，**攻击距离 == 座位距离**。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **攻击距离 ≠ 座位距离**的情形（马匹/技能修正）——上游用 `get.distance(..., "attack")`
- [ ] `getIndex: event.num` 的**多点伤害**索引语义
- [ ] `ai.effect` 策略面
- [ ] 运筹其余武将技（纵火/烧营/隙焰/红莲/野升/归汉/厉焰×2/精械/巧思/奸雄变体/蛊惑变体/悲歌变体）
