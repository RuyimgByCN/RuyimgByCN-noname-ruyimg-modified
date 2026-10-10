# ADR 0271：SP「励战」的受伤多目标摸牌

ABI 195 增加可复用目标域 `wounded`、过滤器 `owner_is_target_any_alive_wounded`，以及
`choose_optional_targets_draw(player, count, min_count, max_count, scope)`。目标选择可取消，
数量上限按当前合法目标数收窄；选择恢复时重验目标，并按座次依次执行带技能来源的摸牌。

SP「励战」据此重写结束阶段任意受伤角色摸牌。SP 曹仁的另一技能「伪溃」尚未重写，因此本次
只发布技能，不提前发布不完整武将；旧 JavaScript 仅作为语义证据。
