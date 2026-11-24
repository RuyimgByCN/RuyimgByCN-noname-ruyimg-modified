use super::*;

/// Apply `L28` (`shengong`), `L11VmFixC` (`xswuyan`), and `eval_card_target_canceled_opt`
/// exclusions at `CardUsed` intrinsic initialization (`kernel::intrinsic_machine`).
/// Returns `false` if no legal targets remain and carriers should be discarded.
pub(in crate::kernel) fn filter_intrinsic_card_targets(
    ctx: &mut Ctx<'_>,
    frame_id: EventId,
    u: &mut crate::event::CardUseData,
) -> bool {
    let excl_kind = u.effective_kind_in(ctx.core, ctx.catalog);
    let excl_targets = match excl_kind {
        CardKind::Wugu if u.targets.is_empty() => ctx.core.alive_players().collect(),
        CardKind::Taoyuan if u.targets.is_empty() => {
            ctx.core.alive_players().filter(|&p| p != u.player).collect()
        }
        _ => u.targets.clone(),
    };
    let total = excl_targets.len();
    if total == 0 {
        return true;
    }
    let filtered: Vec<PlayerId> = excl_targets
        .into_iter()
        .enumerate()
        .filter(|&(idx, t)| {
            let jiedao_victim = excl_kind == CardKind::Jiedao && idx > 0;
            !crate::rvm_bridge::modifiers::d34_l28_mods::shengong_excludes_target(
                ctx.core,
                ctx.catalog,
                u.player,
                t,
                u.card,
                excl_kind,
                u.nature,
                total,
            ) && (jiedao_victim
                || !crate::rvm_bridge::modifiers::d34_l11_mods_vm_c::xswuyan_excludes_target(
                    ctx.core,
                    ctx.catalog,
                    u.player,
                    t,
                    excl_kind,
                ))
                && !(matches!(excl_kind, CardKind::Wugu | CardKind::Taoyuan)
                    && crate::rvm_bridge::modifiers::eval_card_target_canceled_opt(
                        ctx.core,
                        ctx.catalog,
                        u.player,
                        t,
                        u.card,
                        excl_kind,
                        u.nature,
                    ))
        })
        .map(|(_, t)| t)
        .collect();
    if filtered.len() != total {
        u.targets = filtered;
        ctx.res
            .frame_mut(frame_id)
            .expect("card use frame")
            .card_use_mut()
            .targets = u.targets.clone();
        if !u.targets.is_empty() && matches!(excl_kind, CardKind::Wugu | CardKind::Taoyuan) {
            ctx.core
                .vm_aux
                .flags
                .insert((u.player, format!("dddanzhi_targets:{frame_id}")), 1);
        }
        if u.targets.is_empty() || (excl_kind == CardKind::Jiedao && u.targets.len() < 2) {
            ctx.core
                .vm_aux
                .flags
                .remove(&(u.player, "vm_va_recover_bonus".to_string()));
            return false;
        }
    }
    true
}
