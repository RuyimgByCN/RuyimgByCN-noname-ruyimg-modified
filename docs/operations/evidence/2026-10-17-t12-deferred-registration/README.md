# 2026-10-17 T12 运筹/玩点论杀余暂缓：面级工程登记（T12-YUNCHOU-DEFERRED / T12-WUDIAN-DEFERRED）

**核验日期**：2026-10-17。**性质**：余暂缓的差异登记定案（无代码/版本/golden 变化；规则 v295 / 存档 v80）。
前置勘察：[T12 勘察定案（仇渡/调兵遣将、奸雄·变体）](../2026-10-14-t12-deferred-survey/README.md)。
上游采用版本：本地 `archive/upstream/noname`（v1.11.7）杀海拾遗 `character/{yunchou,wudian}`、`card/yunchou`。

## 结论：两来源余暂缓均为面级工程，登记为差异（不实现）

| 来源 | 暂缓项 | 上游障碍（新原语） | 处置 |
|---|---|---|---|
| 玩点论杀（U07） | `diaobingqianjiang`（调兵遣将×2，包内 `deferred`） | **亮牌替换型锦囊**：动态亮牌 `ceil(存活/2)`、逐目标「1 手牌替换 1 亮牌」替换机、结算后剩余牌**任意顺序置顶**（牌堆顶任意序操作面）——三触点皆新原语（[勘察](../2026-10-14-t12-deferred-survey/README.md)） | 登记差异（随「亮牌替换型锦囊面」整体立项） |
| 玩点论杀（U07） | `choudu`（杨仪·仇渡，generals 未映射） | 依赖调兵遣将（视为使用）⇒ 随上项 | 登记差异（依赖上项） |
| 玩点论杀（U07） | `duoqi`（刘馥·夺启）、`youzhan`（田豫·佑赞），generals 未映射 | `duoqi`=phaseUse 中止未建模；`youzhan`=shaBefore 时点映射偏晚 + 虚拟 useCard 组合（[收尾证据](../2026-10-08-u07-deferral-closeout/README.md)） | 登记差异（时点/中止原语缺口） |
| 运筹帷幄（U05 子包） | `diyjianxiong`（曹操·奸雄·变体，generals 未映射） | **身份改写面**：运行时改 `GameMode::Identity` 的 roles、showIdentity 标记与视图投影、死亡/胜负按新身份联动、国战 trueIdentity 随机面——四触点新面（[勘察](../2026-10-14-t12-deferred-survey/README.md)） | 登记差异（随身份局深化批立项） |
| 运筹帷幄（U05 子包） | `jingxie` / `qiaosi`（马钧·精谐/巧思，generals 未映射） | **骰子原语**：上游 `throwDice`（含 `isLuckyStar` 六点分支）→ 摸/给/弃等量牌的确定性方向选择（`chooseControl` 态度面） | 登记差异（随骰子/态度原语立项） |

## 解除条件

- 引入「亮牌替换型锦囊 + 牌堆顶任意序操作」原语 → 重评调兵遣将与仇渡。
- 引入「运行时身份改写 + 视图投影」面 → 重评奸雄·变体。
- 引入「骰子（含幸运星）+ 态度选面」原语 → 重评精谐/巧思。
- 解除任一原语后，按内容批实现并递增规则版本、重录 golden、补定向回归。

## 影响

无代码、版本、golden 变化。两来源的余暂缓由「暂缓挂账」转为**已登记差异 + 解除条件**；相应 `T12-SRC-U05`/`T12-SRC-U07` 整包行仍开放（内容面未全）。
