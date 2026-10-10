# ADR 0329：SHIJI 神孙策「冯河」纯 DSL 移植

「冯河」（`pinghe`）以既有原语组合表达，不新增 Rule VM ABI：
- `modifier hand_limit_base`（旧 JS 的“基数=已损失体力”）当前 DSL 仅支持
  `owner_turn_damage_dealt` 值源，无法表达 `max_hp - hp`，该 modifier 未建模。
- 触发 `damage_before owner_is_target`：体力上限大于 1 且有手牌时，
  `prevent_current_damage()` 防伤害 + `change_max_hp(self, -1)` 失上限 +
  `choose_other_give_one_hand_equip(self)` 交一张手牌给其他角色。
- 旧 JS 的“若拥有【英霸】则伤害来源获平定标记”需要“拥有某技能”条件判断，
  DSL 暂不支持，该分支未建模。

神孙策（`shen_sunce`）的「英霸」「覆海」「冯河」技能组已全部移植，
但因旧 JS 开局体力为 1（`hp: 1, maxHp: 6`）而 characters schema 无 `hp` 字段，
暂不发布武将定义。旧 JavaScript 不进入 npack 或运行时。
