# NC08 第⑦层样本 75：**来源 U05（杀海拾遗·运筹）** 武将技 `honglian`（红莲）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过（含行为面 · 正反例）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`honglian`）：

```js
honglian: {
    trigger: { player: "damageEnd" },
    check(event, player) { return get.attitude(player, event.player) < 0; },
    filter(event, player) {
        return event.source && event.source !== player
            && event.source.countCards("he", { color: "red" }) > 0;      // 来源（非自己）有红牌
    },
    content() { trigger.source.discard(trigger.source.getCards("he", { color: "red" })); },
},
```

## 2 验收结果（`nc08_upstream_yunchou_honglian.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 3 | `discard(getCards("he", {color:"red"}))` | **行为面（正例）**：来源持 **2 红 + 1 黑** → 伤害持技者后，出现 `trigger_confirm`，来源手牌变为 **`[0]`（仅黑牌）** ⇒ **全部红牌被弃** | 通过 |
| 2 | `countCards("he", {color:"red"}) > 0` | **行为面（反例）**：来源**无红牌**（0 红 1 黑）→ **无任何询问**（`询问=[]`）、手牌不变 | 通过 |
| 1 | `damageEnd` + 来源非自己 | 原生 `yunchou.rs:321`（来源弃手牌与装备区全部红牌，装备走 `revoke`） | 通过 |

## 3 夹具要点

- **双产物桥接**；技能目录名 **`ycnhonglian`**（包面向 `host:honglian`）；
- **先收集 id 再布阵**（`e.catalog()` 借用与 `put_hand(&mut e, …)` 冲突 → 编译期即报 E0502）；
- 红莲**非锁定** → 需处理 `trigger_confirm`。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **装备区红牌**分支（弃置 + 装备技撤销 `revoke`）
- [ ] `check`（AI 是否发动）与 `ai.expose`/`ai.result` 策略面
- [ ] 运筹其余武将技（纵火/烧营/隙焰/归汉/厉焰×2/精械/巧思/奸雄变体/蛊惑变体/悲歌变体）
