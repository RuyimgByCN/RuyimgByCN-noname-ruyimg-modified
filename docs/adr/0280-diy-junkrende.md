# ADR 0280：DIY 旧界仁德与不同目标连续分配

ABI 204 增加主动技效果
`give_selected_then_continue_distinct_recover_at_two(player, target)`。首次批次沿用统一主动技框架
选择的手牌和其他角色；每批给牌及其嵌套触发结束后，可取消或继续选择当前手牌，并只能选择
本次尚未获得过牌的其他存活角色。

状态机保存已使用目标与累计给牌数。累计首次达到两张时，在对应给牌移动触发完成后回复一点
体力；继续分配不再回复。旧 JavaScript 的 `assignedTargets`、异步 `while`、提示、动画和 AI
回调不进入 npack 或运行时。
