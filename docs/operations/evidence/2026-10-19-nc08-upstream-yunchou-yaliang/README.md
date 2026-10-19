# NC08 第⑦层样本 70：**来源 U05（杀海拾遗·运筹）** 武将技 `yaliang`（雅量）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`yaliang`）：

```js
yaliang: {
    trigger: { player: "damageEnd", source: "damageSource" },   // 受伤者/来源皆可触发
    getIndex: event => event.num,
    filter(event) {
        if (event._notrigger.includes(event.player)) return false;
        return event.num && event.source?.isIn() && event.player?.isIn()
            && event.source !== event.player;                    // 来源≠受伤者
    },
    async content(event, trigger, player) {
        await game.asyncDraw([trigger.player, trigger.source].sortBySeat());   // 双方按座位序各摸 1
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_yaliang.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 3 | `game.asyncDraw([player, source].sortBySeat())` | **行为面**：0 号（持雅量）对 1 号造成 1 点伤害 → **来源手牌 0 → 1**、**受伤者手牌 0 → 1**（双方各摸 1） | 通过 |
| 1+2 | `damageEnd`/`damageSource` + `source !== player` | 原生 `yunchou.rs:14`：`seats = targets` → `sort_unstable` + `dedup` → 存活者各 `draw_cards(s,1)`（**座位序**与上游一致）；注册 tag 244 | 通过 |

## 3 夹具要点

- **双产物桥接**（武将面在 `character.json`）：`bridge_pack` + `bridge_generals`；
- 技能目录名为 `ycnyaliang`（包面向 `mapping` 为 `host:yaliang`）；
- 伤害走**真实路径** `debug_deal_damage`；雅量**非锁定** → 需处理 `trigger_confirm`。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **「来源==受伤者」自伤分支**（上游 `event.source !== event.player` 排除）未构造
- [ ] `getIndex: event.num` 的**多点伤害**索引语义
- [ ] `ai.maixie` 策略面；`preHidden` 表现面
- [ ] 运筹其余武将技（厉焰/纵火/烧营/隙焰/红莲/…，共 21 技）行为面
