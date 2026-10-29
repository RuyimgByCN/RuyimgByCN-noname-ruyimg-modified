use super::super::*;

pub(super) fn begin_discard(ctx: &mut Ctx<'_>, player: PlayerId) {
    // 入梦（swd 闻人绘，批 122，规则 v309）：跳过弃牌阶段位
    //（上游 `skip('phaseDiscard')` 同口径；一次性，用后清零）
    if ctx.core.player(player).skip_discard_phase {
        ctx.core.player_mut(player).skip_discard_phase = false;
        ctx.core.turn.discard_phase_discarded = 0;
        return;
    }
    // 虚失（Boss 朱厌，规则 v225）：出牌阶段结束自翻面（上游
    // phaseUseEnd turnOver 同口径；锚=弃牌阶段开始[原生无
    // PhaseEnd 帧]；经 TurnOverBefore 走翻面提交）
    if let Some(xdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XUSHI)
    {
        if ctx.core.has_skill(player, xdef) && ctx.core.player(player).alive {
            ctx.emit(EventData::TurnOverBefore {
                player,
                cancelled: false,
            });
        }
    }
    // 贪欲（Boss 饕餮，规则 v215）：锁定——跳过你的弃牌阶段
    //（上游 boss_tanyu phaseDiscardBefore trigger.cancel 同口径）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_TANYU)
    {
        if ctx.core.has_skill(player, def) {
            ctx.core.turn.discard_phase_discarded = 0;
            return;
        }
    }
    // 庆祝（Boss 明星，规则 v245）：锁定——跳过你的弃牌阶段
    //（上游 boss_qingzhu phaseDiscardBefore trigger.cancel 同
    // 口径；与急叮解锁无关，恒跳）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_QINGZHU)
    {
        if ctx.core.has_skill(player, def) {
            ctx.core.turn.discard_phase_discarded = 0;
            return;
        }
    }
    // 水没墓穴计数（mtg 第三批，规则 v102）：弃牌阶段开始清零
    ctx.core.turn.discard_phase_discarded = 0;
}
