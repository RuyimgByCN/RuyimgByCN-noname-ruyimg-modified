use super::super::*;

pub(super) fn begin_discard(ctx: &mut Ctx<'_>, player: PlayerId) {
    // 入梦（swd 闻人绘，批 122，规则 v309）/ PA12 批⑨ skip_next_discard_phase：
    // 跳过弃牌阶段位（上游 `skip('phaseDiscard')` 同口径；在 begin_discard 记
    // skipped 并短路入场行为，随后由 intrinsic_machine 消费并清零以抑制
    // DiscardToLimitMachine）。
    if ctx.core.player(player).skip_discard_phase {
        ctx.record_phase_skipped(player);
        ctx.core.turn.discard_phase_discarded = 0;
        return;
    }
    // 虚失（Boss 朱厌，规则 v225）：出牌阶段结束自翻面（上游
    // 虚失·自翻面（K56，A2）：已迁 Phase{Play, begin:false}（kernel/
    // phase.rs commit_phase 出牌阶段结束臂——上游 player phaseUseEnd
    // turnOver；原锚弃牌阶段开始在跳过弃牌/跳过出牌路径上漂移）。
    // 贪欲（Boss 饕餮，规则 v215）：锁定——跳过你的弃牌阶段
    //（上游 boss_tanyu phaseDiscardBefore trigger.cancel 同口径）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_TANYU)
    {
        if ctx.core.has_skill(player, def) {
            ctx.core.turn.discard_phase_discarded = 0;
            ctx.record_phase_skipped(player);
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
            ctx.record_phase_skipped(player);
            return;
        }
    }
    // 水没墓穴计数（mtg 第三批，规则 v102）：弃牌阶段开始清零
    ctx.core.turn.discard_phase_discarded = 0;
}
