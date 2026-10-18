# NC08 第⑦层样本 7：`rangjie`（让节，L10 史诗卡牌）（2026-10-18 采集 / 2026-10-19 补写 README）

- 日期：2026-10-18（采集）、2026-10-19（补写说明）
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项验收通过**；**1 项缺陷守护（未修复）**

## 1 上游源码（归档快照，逐字引用；见 [upstream-rangjie.js.txt](upstream-rangjie.js.txt)）

`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js`（让节）：

- **时机**：受到伤害后（`damageEnd`）；
- **效果（取消路径与正常路径）**：非取消路径在**分支前**无条件 `player.draw()`，随后按选择的花色
  「移动牌」或执行另一分支。

## 2 验收结果（`nc08_upstream_rangjie.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | 触发时机与门槛 | 受到伤害后出现询问（可取消） | 通过 |
| 2 | 取消路径 | 取消 → 不摸牌、不改变牌区 | 通过 |
| 3 | **移动牌分支须摸 1 张** | 原生该分支**未摸牌** → 用例 `#[ignore]` 守护 | **缺陷守护（未通过）** |

**原生实现引用**：`content/host/step/skills_cards.rs` 的 `HostStage::SelectNature` 分支
（`1 =>` 移动牌分支缺 `ctx.draw_cards(self.owner, 1)`）。

## 3 缺陷与修法（已并入待实施修复清单）

- **修法**：在该分支开头补 `ctx.draw_cards(self.owner, 1);`（上游 `player.draw()` 在分支前无条件执行）；
- **验收**：移除 `#[ignore]` 后 `upstream_rangjie_move_branch_must_draw_one` 应通过；
- **阻塞**：`content/host/step/skills_cards.rs` **未跟踪**（他人 host 拆分）→ 见
  [待实施修复清单](../2026-10-18-blocked-fixes/README.md)。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | 2 passed / 1 ignored（缺陷守护） | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见 [engine-native-tests.log](engine-native-tests.log) | — |

## 5 未完成 / 待办

- [ ] 移动牌分支补摸（待拆分落定）
- [ ] 样本 21+：其余来源（当前 **17/37 来源**）
