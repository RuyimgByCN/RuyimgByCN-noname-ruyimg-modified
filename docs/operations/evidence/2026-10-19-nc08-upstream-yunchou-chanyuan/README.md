# NC08 第⑦层样本 72：**来源 U05（杀海拾遗·运筹）** 武将技 `diychanyuan`（蝉缘·变体）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:402`：

```js
diychanyuan: {
    trigger: { player: "dieBegin" },        // 自己（濒死→）死亡时
    forced: true,
    filter(event) { return event.source !== undefined; },   // 有来源
    content() { trigger.source.loseMaxHp(true); },          // 来源失去 1 点体力上限（并钳体力）
},
```

## 2 验收结果（`nc08_upstream_yunchou_chanyuan.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3 | `loseMaxHp(true)` | **行为面**：1 号（持蝉缘）被 0 号**致命伤害**击杀 → **来源（0 号）体力上限 4 → 3**，且 **hp 被钳到新上限**（hp=3 ≤ 3） | 通过 |
| 1+2 | `dieBegin` + `source !== undefined` | 原生 `yunchou.rs:26` `ctx.lose_max_hp(src, 1)`；上游 `dieBegin` + 有来源门槛 | 通过 |

## 3 夹具要点

- **双产物桥接**；技能目录名 **`ycndiychanyuan`**（包面向 `host:diychanyuan`）；
- 击杀须走**真实伤害路径** `debug_deal_damage`（直接写 hp 不会进入濒死/死亡流程）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **无来源死亡**分支（`filter` 排除，如体力流失致死）
- [ ] `ai.threaten`/`ai.result` 策略面
- [ ] `dieBegin` 与死亡结算的顺序细节
- [ ] 运筹其余武将技（纵火/烧营/隙焰/红莲/野升/归汉/厉焰×2/精械/巧思/狂骨变体/奸雄变体/蛊惑变体/悲歌变体）
