# NC08 第⑦层样本 62：`xinfu_jijie`（**机捷**，标准面 / 伊籍）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过（含行为面），未发现差异**（取牌位置 / 目标范围 / 限次均与上游一致）
- **基准面**：`character/standard/skill.js`
- **命名提示**：上游键 `xinfu_jijie` 在 **`translate.js`** 中映射为「**机捷**」（**不是**「新服结姻」）；
  原生常量 **`JI_JIE`**（`vocabulary/names.rs:46`）——按上游键名检索原生源码会**查不到**（见实现索引）

## 1 上游源码（归档快照，逐字引用；见 [upstream-jijie.js.txt](upstream-jijie.js.txt)）

```js
xinfu_jijie: {
    enable: "phaseUse", usable: 1,                 // 出牌阶段限一次
    async content(event, trigger, player) {
        const card = get.bottomCards()[0];          // 牌堆底一张
        game.cardsGotoOrdering(card);
        const { bool, targets } = await player.chooseTarget({ forced: true, ai(target) { … } });
        …
    },
},
```

## 2 验收结果（`nc08_upstream_jijie.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `get.bottomCards()[0]` | 原生注释与实现取**牌堆底**一张 | 通过 |
| 3 | `chooseTarget({forced:true})` 且**无 `filterTarget`** | 原生注释「上游 `chooseTarget` **无过滤器**，AI 含自引加成，原生**同允许自选**」→ **任意存活角色（含自己）** | 通过 |
| — | 命名 | `translate.js` 中 **`xinfu_jijie: "机捷"`**（本用例同时断言该映射，固定命名提示） | 通过 |
| 2+3 | **行为面** | 发动机捷 → 询问 `jijie_give_target` → 选 1 号 → **牌堆底那张（`deck[0]=56`）进入目标手牌**（`[56]`），牌堆 **106 → 105** | 通过 |

**原生实现引用**：`content/standard/skills/ji_jie.rs`。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [x] **行为面已验收**（牌堆底 `deck[0]` → 目标手牌；牌堆 −1）
- [ ] `ai(target)` 自引加成评分与 `audio` 表现面
- [ ] 样本 63+：其余来源/技能（标准面技能 **40 个样本**；来源覆盖 **23/37**）
