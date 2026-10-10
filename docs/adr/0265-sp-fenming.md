# ADR 0265: SP「奋命」横置角色顺序弃牌

ABI 189 增加通用效果 `choose_discard_one_from_each_linked(player, skill)`：技能拥有者横置时，
冻结按座次排列、横置且有可弃手牌或装备牌的存活角色，并依次复用现有目标区域单牌选择续体。
角色在轮到结算前失去全部合法牌时由既有选择器安全跳过；技能来源失效则拒绝执行。

过滤器 `owner_is_target_linked` 保证只有结束阶段的横置拥有者收到可选发动窗口。SP 只发布技能，
XIANDING 通过精确依赖将其与「断绁」组成陈董完整武将闭包；旧 JavaScript 只作语义证据。
