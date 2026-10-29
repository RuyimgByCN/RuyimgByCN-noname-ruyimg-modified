//! 效果原语分派（apply_effect）：具名技能结算器（Resolve*）。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ResolveTurnEndDebates { player, skill } => {
            resolve_turn_end_debates(tx, player, skill)
        }

        EffectRequest::ResolvePotheyun { player, skill } => resolve_potheyun(tx, player, skill),

        EffectRequest::ResolveTwjieqiu {
            player,
            target,
            subaction,
            skill,
        } => resolve_twjieqiu(tx, player, target, subaction, skill),

        EffectRequest::ResolveJsrgsheju {
            player,
            counterpart,
            skill,
        } => resolve_jsrgsheju(tx, player, counterpart, skill),

        EffectRequest::ResolveOlJieDanshou { player, skill } => {
            resolve_ol_jie_danshou(tx, player, skill)
        }

        EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_youdi(tx, player, target, card, stage, skill),

        EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_kuangfu(tx, player, target, card, stage, skill),

        EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            skill,
        } => resolve_shanjia(tx, player, count, stage, skill),

        EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        } => resolve_new_yajiao(
            tx,
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        ),

        EffectRequest::ResolveDingpin {
            player,
            target,
            skill,
        } => resolve_dingpin(tx, player, target, skill),

        EffectRequest::ResolveSpxizhan {
            player,
            phase_player,
            fangzong_skill_def,
        } => resolve_spxizhan(tx, player, phase_player, fangzong_skill_def),

        _ => unreachable!("effect routed to a different primitive group"),
    }
}
