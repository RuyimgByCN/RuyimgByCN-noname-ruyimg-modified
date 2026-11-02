//! Source-qualified damageAfter callbacks, after the entire Dying/Source/End subtrees.
use super::*;
pub(super) fn post_damage_after(
    ctx: &mut Ctx<'_>,
    d: crate::event::DamageData,
    effect: crate::event::NativeDamageEffect,
    _owner: PlayerId,
) {
    if effect == crate::event::NativeDamageEffect::Huanxia {
        // 还侠跟踪（swd 屈娴，批 27）：还侠转化杀造成真实伤害 → 置位（上
        // 游 expire 子技 source damageAfter 同口径；牌 id 匹配最后提交者）
        if let Some(dcard) = d.card.filter(|_| d.source.is_some()) {
            if let Some((card, caused)) = ctx.core.turn.huanxia_track.last_mut() {
                if *card == dcard {
                    *caused = true;
                }
            }
        }
    }
    if effect == crate::event::NativeDamageEffect::Jidian {
        // 极点（Boss 张角，规则 v263）：你造成伤害后——距离受伤者 1 以内的
        // 一名其他角色判定：黑则其受到 1 点雷伤（上游 source damageAfter
        // chooseTarget get.distance(trigger.player,target)<=1 + judge 黑 →
        // damage("thunder") 同口径；chooseTarget AI→座位序首个确定性近似；
        // 链挡旗标=极点雷伤不自我递归，登记差异：上游 damageAfter 真递归；
        // 判定为内联确定性口径，无改判窗）
        if let Some(jddef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_JIDIAN)
        {
            if let Some(src) = d.source {
                if ctx.core.player(src).alive && ctx.core.has_skill(src, jddef) {
                    // 链挡：极点造成的雷伤不再触发极点
                    if !ctx.core.turn.jidian_chain {
                        let seats = ctx.core.players.len() as PlayerId;
                        if let Some(t) = (0..seats).find(|&t| {
                            t != d.target
                                && ctx.core.player(t).alive
                                && ctx.core.computed_distance(ctx.catalog, d.target, t) <= 1
                        }) {
                            // 置链挡后 emit：嵌套 Damage 帧的极点臂见旗标清除
                            // 并跳过（挡恰一层=极点雷伤不链式，登记差异）
                            ctx.core.turn.jidian_chain = true;
                            let judged = ctx.draw_judge_card();
                            let black = matches!(
                                ctx.catalog.cards[judged as usize].suit,
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            );
                            ctx.to_discard(judged);
                            if black {
                                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                    Some(src),
                                    t,
                                    1,
                                    crate::event::DamageNature::Thunder,
                                    None,
                                )));
                            }
                        }
                    } else {
                        // 嵌套极点雷伤到达：清链挡（仅挡一层）
                        ctx.core.turn.jidian_chain = false;
                    }
                }
            }
        }
    }
    if effect == crate::event::NativeDamageEffect::Minwan {
        // 冥顽锁定集（Boss 梼杌，规则 v219）：回合内自己使用牌对其他角色
        // 造成伤害后，受伤者入锁定集（首次同时记自己——上游 storage
        // boss_minwan=[player]+add(target) 同口径；回合更替自然清零）
        if let Some(mdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_MINWAN)
        {
            if let Some(src) = d.source {
                if ctx.core.turn.current == src
                    && ctx.core.has_skill(src, mdef)
                    && ctx.core.player(src).alive
                {
                    let turn = &mut ctx.core.turn;
                    if turn.minwan_targets.is_empty() {
                        turn.minwan_targets.push(src);
                    }
                    if !turn.minwan_targets.contains(&d.target) {
                        turn.minwan_targets.push(d.target);
                    }
                }
            }
        }
    }
}
