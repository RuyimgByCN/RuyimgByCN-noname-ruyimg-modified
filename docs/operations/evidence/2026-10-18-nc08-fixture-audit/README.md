# NC08 夹具审计：`debug_place` 实例问题（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**审计 4 个涉及花色/颜色的样本**；发现 **2 处伪差异 + 2 处脆弱夹具**，均已处置

## 1 根因（一次教训，四处影响）

`debug_place(player, name)` **只按名字查找**，同名牌可能有**不同花色实例**
（例：【无懈可击】既有 ♦ 红也有 ♣ 黑；【杀】有黑桃/梅花/红桃/方块）→ 依赖名字布阵的
**颜色/花色断言**会命中**非预期实例**，产生伪差异或脆弱通过。

## 2 审计结果

| 样本 | 是否依赖花色/颜色 | 处置 |
|---|---|---|
| `wusheng` 武圣 | 是（红/黑） | **2 处伪差异已撤回**（「红色锦囊不可转化」「原生接受主动使用无懈」）；助手改为按 id 布阵 |
| `qixi` 奇袭 | 是（黑） | 「状态依赖」根因查明＝夹具假象（♦ vs ♣ 实例）；改为按 id 布阵，用例转通过 |
| `ganglie` 刚烈 | 是（判定牌须**非红桃**） | 判定牌改为**按 id** 取非红桃【杀】并置入判定区（原先按名字，脆弱） |
| `xiuluo` 修罗 | 是（同花色判定牌） | **本就安全**：手牌从**运行时选项**中动态挑选（不假设花色） |

## 3 规则基准同步

「已撤回的猜测」补至 **4 条**，并写入方法学：**涉及颜色/花色的验证必须按 card id 布阵**
（`debug_place` 不保证实例）。

## 4 验证

| 检查 | 结果 | 证据 |
|---|---|---|
| 三个受影响样本定向 | 见 [audited-samples.log](audited-samples.log)（各 3–6 passed / 0 failed） | — |
| engine-native 全量 | 1249 passed / 0 failed / 5 ignored | — |
| 行数门禁 | 8310 文件 ≤ 2000 行 | — |

## 4b 共享助手已落地（同轮）

新增 `crates/engine-native/tests/nc08_common/mod.rs`（集成测试共享模块，**按 id 布阵**）：

| 助手 | 作用 |
|---|---|
| `card_id(e, name, want_red)` / `give_colored(e, player, name, red)` | 按**名字+颜色**取 card id（**不经** `debug_place`）并入手 |
| `put_hand` / `put_judge` / `put_equips` | 按 id 放入**手牌 / 判定区 / 装备区** |
| `drain` / `advance_to_phase` / `advance_to_play` / `next_dedup` | 通用驱动（触发确认一律确认） |

**已迁移**：`wusheng`（6/6）、`qixi`（4/4）、`ganglie`（3/3）→ 三样本定向全通过，
`engine-native` 全量见 [after-helper.log](after-helper.log)。**其余样本**按新增/改动时逐步迁移。

## 5 未完成 / 待办

- [ ] 其余来源样本（扩展类）同类审计：`kurou`/`kuwu`/`shengke`/`ciqiu` 等**未**使用花色断言
      （本轮 grep 已确认不含 `Suit::`）→ 但**新增样本**应统一采用「按 id 布阵」模板
- [x] **共享测试助手已落地**（`tests/nc08_common/mod.rs`）并迁移 3 个样本
