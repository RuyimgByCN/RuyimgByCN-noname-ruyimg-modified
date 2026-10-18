# NC08 第⑦层样本 3：`ciqiu`（刺酋，U07 玩点论杀/韩龙）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层「上游行为」对照样本 **3/37 来源**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**由上游 JS 推导的 5 条规则点全部对齐**（无新差异登记）；样本覆盖来源从 1（L20）扩到 **2（L20 + U07）**

## 1 上游源码（归档快照，逐字引用；见 [upstream-ciqiu.js.txt](upstream-ciqiu.js.txt)）

`archive/upstream/noname/apps/core/extension/玩点论杀/character/skill.js`：

```js
ciqiu: {
    trigger: { source: "damageBegin1" },
    forced: true,
    filter(event) { return event.card && event.card.name === "sha" && event.player.isHealthy(); },
    content() {
        "step 0";
        trigger.num++;
        if (trigger.num >= trigger.player.hp) {
            trigger.player.addTempSkill("ciqiu_dying");
            player.removeSkill("ciqiu");
        }
    },
},
ciqiu_dying: {
    trigger: { player: "dyingBegin" },
    forced: true, silent: true, firstDo: true,
    content() { player.die(); },
},
```

## 2 由 JS 推导的规则点 → 原生验收（新增 `nc08_upstream_ciqiu.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `event.card.name === "sha"` | **非【杀】伤害**（无牌 1 点伤害）→ **不加伤**（体力只 −1） | 通过 |
| 2 | `event.player.isHealthy()` | 满血判定口径 = `hp == max_hp`（原生同式，见 `kernel/damage.rs`） | 通过 |
| 3+5 | `trigger.num++` + `removeSkill("ciqiu")` | 满血目标被【杀】命中且加后 ≥ 其体力 → **濒死即死（跳过救援）** 且**使用者失去刺酋**（第二次出杀不再 +1） | 通过 |
| 4 | `addTempSkill("ciqiu_dying")` + `dyingBegin firstDo` | 原生以 `TurnState.ciqiu_dying` 等价（**回合级**、换回合清空）且在濒死机**最前**判定；无标记时走正常濒死流程（可被救援） | 通过 |

**原生实现引用**：`kernel/damage.rs:202-222`（加伤与标记）、`kernel/dying.rs:51-57`（标记在位
跳过救援直接死亡）、`state.rs:482`（`TurnState.ciqiu_dying`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 上游对照定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1169 passed / 0 failed / 2 ignored**（1166 → +3） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8275 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC08 第⑦层）

- [ ] 样本 4+：其余来源（当前 2/37 来源有上游源码推导样本）
- [ ] 逐命令对照工具（固定上游版本/局面/随机输入与玩家选择）
- [ ] 样本 1 登记的**顺序差异**（`loseHp` 与 `draw` 先后）上游判定
- [ ] 表现/策略面（`ai.effect`/`silent`/`popup`）不在规则对照范围（边界登记）
