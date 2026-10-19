# NC08 第⑦层样本 22：`keji`（克己，**标准面** / 吕蒙）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 1 项差异登记）；**1 项未验收（如实登记）**
- **基准面**：`character/standard/skill.js`（先确认基准面**确有**该技能——本轮先用 `jushou` 时发现
  标准面无此技能（仅 `sb` 面有且语义不同），故改用 `keji`）

## 1 上游源码（归档快照，逐字引用；见 [upstream-keji.js.txt](upstream-keji.js.txt)）

```js
keji: {
    trigger: { player: "phaseDiscardBefore" },
    frequent(event, player) { return player.needsToDiscard(); },
    filter(event, player) {
        if (player.getHistory("skipped").includes("phaseUse")) return true;
        const history = player.getHistory("useCard").concat(player.getHistory("respond"));
        for (const evt of history) {
            if (evt.card.name === "sha" && evt.isPhaseUsing()) return false;
        }
        return true;
    },
    async content(event, trigger, player) { … }   // 跳过弃牌阶段
},
```

## 2 验收结果（`nc08_upstream_keji.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | 未使用【杀】 → 跳过弃牌 | 手牌 5 张（>上限 4）时持克己、本回合未使用杀 → **不弃牌** | 通过 |
| 2 | `getHistory("respond")` | **差异登记②**：原生只看 `turn.sha_used_play_phase`（出牌阶段**使用**），**未**建模 `respond`（**打出**）历史（源码事实固定） | 通过（**差异登记**） |
| 2 | 使用过【杀】 → 须弃牌 | **未验收（登记）**：夹具推进到弃牌阶段后手牌 `9 → 9`（强制弃牌未被驱动） | — |

**原生实现引用**：`kernel/intrinsic.rs:1391`（持有 `KE_JI` 且 `!turn.sha_used_play_phase` →
不做手牌上限裁剪 = 视为无限手牌上限）。

## 3 登记的差异①（实现方式）

上游是**事件级**「跳过弃牌阶段」（`phaseDiscardBefore` 触发 + `content` 跳过）；
原生是**上限级**「手牌上限视为无限」（弃牌流程不做裁剪）→ **结果等价**（都不弃牌），
但**事件面不同**（上游会发出「跳过阶段」历史，原生不会）→ 登记，不写成对齐。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **强制弃牌驱动**夹具修复（弃牌阶段的实际弃置路径）后补验收
- [ ] 差异②（`respond` 历史）口径判定：打出【杀】是否应取消克己
- [ ] 样本 23+：其余来源（当前 **19/37 来源**；标准面技能 4 个样本）
