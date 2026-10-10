# ADR 0272：SP「伪溃」的濒死时序与绑定距离

ABI 196 增加 `reveal_target_hand_sha_or_discard_bind(player, target, helper_skill_def)`。效果内部先
失去 1 点体力并等待完整濒死窗口，再按目标当前手牌是否含有效【闪】走强制弃牌或私密展示、
普通次数限制的无距离虚拟【杀】分支；杀结算完成后仅为仍存活的目标添加绑定技能。

新增 `distance_from_skill_bound_target override 1`，只把拥有者至该技能实例绑定目标的最终距离
设为 1。`weikui2` 复用既有回合结束移除技能机制；旧 JavaScript 仅作为语义证据。
