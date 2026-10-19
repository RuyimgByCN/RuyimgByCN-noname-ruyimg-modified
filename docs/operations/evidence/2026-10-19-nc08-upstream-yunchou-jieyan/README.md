# NC08 第⑦层样本 76：**来源 U05（杀海拾遗·运筹）** 武将技 `jieyan`（隙焰）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含行为面）+ **1 项差异登记**（使用者是否被波及）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`jieyan`）：

```js
jieyan: {
    enable: "phaseUse",
    filter(event, player) { return player.hasCard(card => get.color(card) === "red", "h"); },
    filterCard(card, player) { return get.color(card) === "red"; },     // 弃一张红手牌
    filterTarget: true,                        // 任何人（含使用者）
    selectTarget: -1, multitarget: true, multiline: true,
    async content(event, trigger, player) {
        const targets = event.targets.sortBySeat();
        for (const target of targets) await target.damage("fire", "nocard");
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_jieyan.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3 | 弃红手牌 + 座位序火焰伤害 | **行为面**：0 号持 1 红 + 1 黑 → 发动后 **红牌被弃、黑牌保留**；hp **`[4,4,4,4] → [4,3,3,3]`**（其余三名角色**各受 1 点火伤**） | 通过 |
| 2 | `filterTarget: true` + `selectTarget: -1` | **差异登记**：原生 `seats` 过滤 **`i != self.owner`** → **排除使用者**（0 号 hp 不变）；上游 `filterTarget: true` + `selectTarget: -1` 按 noname 语义**含使用者** | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/yunchou.rs:191`（弃红手牌 → 除使用者外全部存活角色
→ 逐座位 `Damage(Fire, 1, card=None)`）。

## 2b 差异裁定（**已完成**，2026-10-19 同轮）

**结论：上游含使用者 → 原生排除使用者 = 规则损失。**

依据 `archive/upstream/noname/apps/core/noname/library/index.js:11681` 的
`lib.filter.targetEnabled(card, player, target)`：

```js
const filterTarget = info.filterTarget;
if (typeof filterTarget === "boolean") {
    return filterTarget;              // 布尔 filterTarget 直接返回，**无「目标不得为使用者」隐式检查**
}
if (typeof filterTarget === "function") return Boolean(filterTarget(card, player, target));
return false;
```

配合 `selectTarget: -1`（全部合法目标）⇒ **使用者是合法目标** ⇒ 隙焰**应波及使用者**。
对照：上游需要排除使用者时会显式写 `filterTarget: (card, player, target) => target !== player`
（如 `jiguantong`、`liuli` 等）——`jieyan` 写的是 `filterTarget: true`，**不排除**。

**分类**：**规则损失**（原生少造成 1 点火焰伤害）；已并入修复清单**第 30 项**（修法：去掉
`i != self.owner` 过滤；验收：本文件「0 号 hp 不变」断言改为 **hp −1**）。

## 3 未完成 / 待办

- [x] **差异裁定已完成**（上游含使用者，见 §2b）
- [ ] `multitarget`/`multiline` UI 表现面
- [ ] `sortBySeat` 与原生座位序的**起始点**（原生自 0 号起 vs 上游自当前回合角色起）
- [ ] 运筹其余武将技（纵火/烧营/归汉/厉焰×2/精械/巧思/奸雄变体/蛊惑变体/悲歌变体）
