# ADR 0302：YINGBIAN 纯 DSL 批次 1（宣公主：追击）

本批移植宣公主（`xuangongzhu`）的「追击」（`ybzhuiji`），以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「追击」（`ybzhuiji`，3463543744）：`phase_play` 触发 + `choose(0,1)` 双分支：
  - 摸两张牌，并 `add_skill_until_turn_end(ybzhuiji_losehp)`（出牌阶段结束时失去 1 点体力）；
  - 若已受伤（`hp(self) < max_hp(self)` 门控），可回复 1 点体力并
    `add_skill_until_turn_end(ybzhuiji_discard)`（出牌阶段结束时弃置两张牌）。
  未受伤时仅提供摸牌分支，与旧 JS 的 `isDamaged()` 条件分支一致。
- 旧 JS 以 `addTempSkill(...,"phaseUseAfter")` 绑定阶段末效果，DSL 以
  `phase_play_end` 触发 + `lose_hp` / `discard_hand_equip` 等价表达。

派生临时技能 `ybzhuiji_losehp`（2726345451）与 `ybzhuiji_discard`（2840957682）
为目录外新分配的稳定定义 ID，随包发布。旧 JavaScript 不进入 npack 或运行时。

宣公主仍依赖未移植的隐藏技「高陵」（`gaoling`，需 showCharacter 事件）与「齐眉」
（`qimei`，需按目标手牌数条件摸牌），暂不发布该武将（沿用 moon_jushou 先例）。
