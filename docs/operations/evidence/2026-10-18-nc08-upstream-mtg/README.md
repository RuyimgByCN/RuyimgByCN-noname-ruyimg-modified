# NC08 第⑦层样本 13：`haidao`（海岛，U05 mtg 卡牌子包 / 地）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个 mtg 卡牌子包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**由上游 JS 推导的 4 条规则点全部对齐**（无新差异登记）

## 1 上游源码（归档快照，逐字引用；见 [upstream-haidao.js.txt](upstream-haidao.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`：

```js
haidao: {
    type: "land",
    enable(card, player) { return !player.hasSkill("land_used"); },  // 每回合限一张地
    notarget: true,                                                  // 无目标
    content() {
        "step 0";
            game.changeLand("mtg_haidao", player);
            if (player.isHealthy()) { player.changeHujia(); event.finish(); }
            else { player.chooseToDiscard("he", "海岛") … }
        "step 1";
            if (result.bool) player.recover(); else player.changeHujia();
        "step 2"; delete player._temp_mtg_haidao;
    },
    ai: { … },
},
```

## 2 验收结果（`nc08_upstream_haidao.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | `notarget` + `isHealthy()` → `changeHujia()` | **满血**使用 → **不出现询问**，且 `land_used` 置位（换地生效） | 通过 |
| 4 | `chooseToDiscard("he")` → `recover()` | 非满血 → 询问**可取消**；选牌 → **回复 1** | 通过 |
| 4 | 取消分支 `changeHujia()` | 非满血 → **取消** → 体力不变且**不弃牌** | 通过 |
| 1 | `enable: !hasSkill("land_used")` | 同回合第二张地（换另一张地）→ **被拒** | 通过 |

**原生实现引用**：`content/host/step/mtg.rs:60`（Act：换地/`land_used`/满血 +1 甲/否则询问）、
`:100`（SelectCard：弃牌 → 回复，取消 → +1 甲）。

## 3 工程发现

- 包内**同名卡只有一张**（`haidao`）→ 验证「每回合限一张地」这类**与具体卡无关**的规则时，
  应换用**另一张地**（本样本用 `yixialan`），否则 `debug_place` 会因包内无第二张而 panic。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 样本 14+：其余来源（当前 **12/37 来源**）
- [ ] mtg 其余 9 张地 + 地图效果面的上游样本（本样本为 1/10）
- [ ] zhenfa（阵法）子包尚无上游样本
- [ ] 逐命令对照工具；表现/策略面（`fullborder`/`ai`）不在规则对照范围
