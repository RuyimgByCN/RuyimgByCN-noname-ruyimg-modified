# ADR 0315：SB 界华雄（扬威）

本批为 SB 包界华雄（`sb_huaxiong`）的完整移植，全部以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

## 技能移植

「扬威」（`sbyangwei`，3601751055）：
- 出牌阶段主动技（`active play_phase uses_per_turn 1`）：摸两张牌
  （`draw_cards(self, 2)`），本局内此技能失效（`remove_skill_by_def(self, sbyangwei)`，
  等价旧 JS 的 `tempBanSkill("sbyangwei", "forever")`），并授予效果技能
  （`add_skill(self, sbyangwei_effect)`）。
- 「扬威·效果」（`sbyangwei_effect`，2849167933，新派生 def）：使用【杀】无距离限制
  （`card_distance_unlimited(sha)`）、次数上限 +1（`sha_use_limit add 1`）、无视防具
  （`ignore_armor override 1`）。

注：旧 JS 的 `sbyangwei_counter` 子技能仅管理 tempBan 生命周期（结束阶段清理标志），
DSL 的 `remove_skill_by_def` 永久移除等价表达该生命周期，无需移植。

界华雄（`sb_huaxiong`，2843923224，男/群/3 体力）由「扬威」+ 已移植的 REFRESH
「新耀武」（`new_reyaowu`，2159889222）构成完整闭包，加入新创建的
`characters.json` 选将池；sb npack 重建为 5 技能 + 1 武将。
旧 JavaScript 不进入 npack 或运行时。
