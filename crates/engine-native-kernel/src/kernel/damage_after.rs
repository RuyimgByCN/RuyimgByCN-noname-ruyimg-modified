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
