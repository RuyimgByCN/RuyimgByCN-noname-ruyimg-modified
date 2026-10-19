# NC08 第⑦层样本 52：`tieji`（铁骑，**标准面** / 马超）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 **2 项差异登记**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-tieji.js.txt](upstream-tieji.js.txt)）

```js
tieji: {
    trigger: { player: "useCardToPlayered" },
    filter(event, player) { return event.card.name === "sha"; },
    async content(event, trigger, player) {
        const { bool } = await player.judge({
            judge(card) {
                if (get.zhu(get.player(), "shouyue")) {          // 守约（主公技变体）
                    if (get.suit(card) !== "spade") return 2;
                } else {
                    if (get.color(card) === "red") return 2;     // 红色即成功
                }
                return -0.5;
            },
            judge2(result) { return result.bool; },
        }).forResult();
        if (bool) trigger.getParent()?.directHit.add(trigger.target);   // 成功 → 目标必中
    },
},
```

## 2 验收结果（`nc08_upstream_tieji.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | 红色判定 → `directHit` | 牌堆顶置**红色**牌（摸牌阶段**之后**放）→ 目标**无 `respond_shan` 询问**（询问列表为空）且**受伤**（hp 4 → 3），尽管其持【闪】 | 通过 |
| 1 | `get.zhu(player, "shouyue")` → 非黑桃 | **差异①**：原生只判**红色**，**未建模守约变体**（与既有 `wusheng` 守约登记**同族**） | 通过（**差异登记**） |
| 2 | `directHit.add(target)` | **差异②**：原生把结果存在**杀机器字段**（`tieji_red`），**未写通用 `directHit`** → 其它技能读不到 → **与样本 50（无双未检查 `directHit`）互为印证**（同一机制面缺口） | 通过（**差异登记**） |

**原生实现引用**：`kernel/sha/step.rs:850`（判定红色 → 目标不能出闪；结果存 `self.tieji_red`）。

## 3 夹具经验（写入证据）

判定牌必须**在摸牌阶段之后**放到牌堆顶——放在 `advance_to_play` **之前**会被摸牌阶段抽走，
导致判定牌不是预期那张（初版实测 `置顶牌=63 花色=Diamond` 但**仍出现** `respond_shan`，
即判定用的并非该牌）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①（守约变体）与 `wusheng` 守约同族，宜一并评估
- [ ] 差异②（必中标记未走通用 `directHit`）与样本 50（无双）**同属一机制面**，宜一并修
- [ ] **铁骑 × 无双** 的交互行为面（判定成功 + 无双持有者 → 应只需一张【闪】）
- [ ] `ai.directHit_ai` 策略面
- [ ] 样本 53+：其余来源/技能（标准面技能 **31 个样本**；来源覆盖 **23/37**）
