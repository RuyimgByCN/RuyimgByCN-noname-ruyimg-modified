# 2026-10-17 T12 mtg 两张地：永久 deferred 登记（T12-MTG-DEFERRED-LANDS）

**核验日期**：2026-10-17。**性质**：结构性依赖的永久 deferred 定案（无代码/版本/golden 变化；规则 v295 / 存档 v80）。
上游依据：[mtg 第十批](../2026-10-08-u05-mtg-batch10/README.md)（archive/upstream noname 杀海拾遗 `card/mtg/` 只读核验）。

## 结论：两张地永久 deferred（不实现）

| 卡 | 上游障碍 | 处置 |
|---|---|---|
| 藏宝海湾（♣12 `mtg_cangbaohaiwan`） | `enable` 守卫 `lib.cardPack.mode_derivation` 为空即 `return false`——**上游自身禁用**；效果完全围绕衍生包/武将包选牌 | **永久 deferred**：上游即禁用态，原生无「模式衍生牌包」概念，不编码 |
| 毒沼（♣8 `mtg_duzhao`） | 效果 `createCard("du")`——毒为 `mode_derivation` 衍生牌实体，原生无对应实体与中毒结算 | **永久 deferred**：依赖衍生牌实体与中毒结算，不编码 |

mtg 12 张上游清单至此处置完毕：10 张转换 + 2 张本项永久 deferred（[batch10](../2026-10-08-u05-mtg-batch10/README.md)）。两张卡不入转换包（`cards.json` 维持 10 张）。

## 解除条件

若未来引入「模式衍生牌包（`mode_derivation`）+ 衍生牌实体 + 中毒结算」原语，可重新评估毒沼；藏宝海湾的上游 `enable` 在采用玩法版本下恒 false，除非上游改变，否则维持 deferred。

## 影响

无代码、版本、golden 变化。台账 U05 mtg 行的两张 deferred 由「暂缓评估」转为**永久 deferred 定案**；登记表 `T12-MTG-DEFERRED-LANDS` resolved。
