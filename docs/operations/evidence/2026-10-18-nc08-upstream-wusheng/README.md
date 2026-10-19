# NC08 第⑦层样本 23：`wusheng`（武圣，**标准面** / 关羽）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**5 项通过**（含 2 项差异登记 + 1 项**撤回的猜测**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-wusheng.js.txt](upstream-wusheng.js.txt)）

```js
wusheng: {
    enable: ["chooseToRespond", "chooseToUse"],       // 使用或打出
    filterCard(card, player) {
        if (get.zhu(player, "shouyue")) return true;  // 主公技「守约」：任意牌
        return get.color(card) === "red";             // 否则：红色牌
    },
    position: "hes",
    viewAs: { name: "sha" },
    prompt: "将一张红色牌当杀使用或打出",
},
```

## 2 验收结果（`nc08_upstream_wusheng.rs`，5 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `get.color === "red"` + `viewAs: sha` | 红色【闪】当【杀】使用 → **被接受**且目标结算伤害 | 通过 |
| 1 | 同上（反例） | **未持武圣** → 红色牌不可当【杀】（被拒） | 通过 |
| 1 | 同上（黑色） | 黑色牌不满足颜色条件 → 登记差异②：原生**接受**出牌阶段主动使用【无懈可击】（上游 `chooseToUse` 面不允许） | 通过（**差异登记②**） |
| 2 | `chooseToRespond` | 登记差异①：原生转化位于 `PlayCard`（**使用**）路径；**打出**路径未验证 | 通过（**差异登记①**） |
| 2 | `get.zhu(player, "shouyue")` | 登记差异②（守约）：原生**未建模**该分支（源码事实：全 crate 无 `shouyue`） | 通过（**差异登记**） |

**原生实现引用**：`engine/play_card.rs:135/154`（`wusheng && red` → `kind = CardKind::Sha`；
与龙胆/奇袭同族转化臂）。

## 3 **撤回的猜测**（保留记录，2026-10-19 同轮）

曾据一次 `Rejected(WrongPhase)` 断言「**红色锦囊不可转化**」；随后专门构造**红色【无懈可击】**
复测为 **Accepted** → **该猜测不成立，已撤回**（当时的 `WrongPhase` 来自另一张红牌——出牌阶段
本就不能使用的延时锦囊类实体）。对应用例改为 `upstream_wusheng_red_trick_convertible_note`，
断言「红色无懈**被接受**」并写明撤回原因。

## 4 夹具经验（写入证据）

- 【闪】**全为红色**（无法构造「黑色闪」）→ 黑色非杀实体改用【无懈可击】；
- `contains("shan")` 会命中【闪电】（**可主动使用**）→ 必须用**精确名**匹配（`== "shan"`）；
- 「出牌阶段不可主动使用」的实体（闪/无懈）才是验证**转化**的正确载体（否则「被接受」只说明本体可用）。

## 5 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **5 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 6 未完成 / 待办

- [ ] 差异①（**打出**路径）是否覆盖：需构造响应场景（借刀/决斗/南蛮等要求出杀）
- [ ] 差异②（守约）：是否补齐主公技分支
- [ ] `position: "hes"` 中**装备/判定区**牌的转化路径未验证
- [ ] 样本 24+：其余来源（当前 **20/37 来源**；标准面技能 5 个样本）
