# ADR 0298：SHIJI SP 花鬘「嬉战」与 `resolve_spxizhan`（ABI 403）

ABI 403 增加 `resolve_spxizhan(player, phase_player, fangzong_skill_def)`：
其他角色回合开始时，拥有者须弃置一张手牌/装备或失去 1 点体力。

弃牌分支：
1. 弃置所选牌；
2. `disable_skill_by_def_until_turn_end` 暂时禁用传入的技能定义（芳踪 `2304282956`）；
3. 按弃牌有效花色视为用牌：
   - ♠：回合角色视为对自己使用【酒】；
   - ♥：拥有者视为对自己使用【无中生有】（回合角色已死亡时仍结算）；
   - ♣：拥有者视为对回合角色使用【铁索连环】（非法则跳过）；
   - ♦：拥有者视为对回合角色使用火【杀】并无视距离（非法则跳过）。

无可弃牌时直接失去体力。旧 JS 的 `chooseToDiscard` 在取消后仍以空牌进入失体力分支；
本实现以 `min=0 max=1` 的不可取消选牌选择对齐。

SP 花鬘（`sp_huaman`）由「象阵」+「芳踪」+「嬉战」组成完整 4 体力蜀势力武将闭包。
旧 JavaScript 不进入 npack 或运行时。
