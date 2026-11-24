use super::super::super::*;
use super::super::l11_common::{emit_lose_hp, flag, set_flag};
use super::helpers::{has_l11_vm_a_skill, owner_unblocked};
use crate::catalog::Catalog;
use crate::event::CardUseData;
use crate::ids::{EventId, PlayerId, SkillDefId};
use crate::machine::Ctx;
use crate::state::GameStateCore;

pub(in crate::content::host) fn on_round_start(ctx: &mut Ctx<'_>) {
    for p in 0..ctx.core.players.len() as PlayerId {
        ctx.core
            .vm_aux
            .flags
            .remove(&(p, "l11_bihuo_distance".to_string()));
    }
}

pub(in crate::content::host) fn on_phase_use_end(ctx: &mut Ctx<'_>, player: PlayerId) {
    let total = ctx.core.players.len() as PlayerId;
    for key in [
        "nzry_longnu_fire_turn",
        "nzry_longnu_thunder_turn",
        "l11_qiangwu_active",
        "l11_qiangwu_rank",
        "l11_xiongluan1_active",
        "l11_xiongluan_target",
    ] {
        ctx.core.vm_aux.flags.remove(&(player, key.to_string()));
    }
    release_xiongluan_from(ctx, player, total);
}

pub(in crate::content::host) fn on_card_use_after(
    ctx: &mut Ctx<'_>,
    fid: EventId,
    u: &CardUseData,
) {
    let converted = ctx
        .core
        .vm_aux
        .flags
        .remove(&(u.player, format!("l11_lihuo_converted:{fid}")))
        .unwrap_or(0);
    let did_dmg = ctx
        .core
        .vm_aux
        .flags
        .remove(&(u.player, format!("l11_lihuo_did_dmg:{fid}")))
        .unwrap_or(0);
    if converted > 0 && did_dmg > 0 && ctx.core.player(u.player).alive {
        emit_lose_hp(ctx, u.player, 1);
    }
}

pub(in crate::content::host) fn on_player_died(ctx: &mut Ctx<'_>, dead: PlayerId) {
    let total = ctx.core.players.len() as PlayerId;
    release_xiongluan_from(ctx, dead, total);
    ctx.core
        .vm_aux
        .flags
        .remove(&(dead, "l11_bihuo_distance".to_string()));
    if let Some(pile) = ctx.core.vm_aux.piles.remove(&(dead, "chunlao".to_string())) {
        for c in pile {
            ctx.to_discard(c);
        }
    }
}

pub(in crate::content::host) fn dying_save_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    dying: PlayerId,
) -> Vec<SkillDefId> {
    let mut out = Vec::new();
    if !core.player(responder).alive || core.player(dying).hp > 0 {
        return out;
    }
    if !has_l11_vm_a_skill(catalog, "chunlao")
        || !owner_unblocked(core, catalog, responder, "chunlao", false)
        || catalog.wansha_blocks_rescue(core, responder, dying)
        || catalog.blocks_card_use_and_save(core, responder)
    {
        return out;
    }
    if core
        .vm_aux
        .piles
        .get(&(responder, "chunlao".to_string()))
        .is_some_and(|p| !p.is_empty())
    {
        if let Some(def) = catalog.skill_by_name("chunlao") {
            out.push(def);
        }
    }
    out
}

pub(in crate::content::host) fn resolve_dying_save(
    ctx: &mut Ctx<'_>,
    responder: PlayerId,
    dying: PlayerId,
    def: SkillDefId,
) -> Option<bool> {
    if ctx.catalog.skill_name(def) != "chunlao"
        || ctx.catalog.skills[def as usize].behavior.as_host_behavior()
            != Some(HostBehavior::L11VmFixA)
    {
        return None;
    }
    if !ctx.core.player(responder).alive
        || !ctx.core.player(dying).alive
        || ctx.core.player(dying).hp > 0
        || !owner_unblocked(ctx.core, ctx.catalog, responder, "chunlao", false)
        || ctx.catalog.wansha_blocks_rescue(ctx.core, responder, dying)
        || ctx.catalog.blocks_card_use_and_save(ctx.core, responder)
        || !ctx
            .core
            .vm_aux
            .piles
            .get(&(responder, "chunlao".to_string()))
            .is_some_and(|p| !p.is_empty())
    {
        return Some(true);
    }
    set_flag(ctx.core, responder, "hs_yuelu_saved", 0);
    let machine = Box::new(HostMachine {
        def_key: "chunlao_save".to_string(),
        behavior: HostBehavior::L11VmFixA,
        owner: responder,
        target: Some(dying),
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    });
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger {
            frame: ctx
                .res
                .stack
                .last()
                .map_or(crate::resolution::NO_FRAME, |f| f.id),
        },
        machine,
        awaiting_answer: false,
    });
    // `resolve_hearth_dying_save` returns `false` when suspending `DyingMachine` on a child `Park`.
    Some(false)
}

fn release_xiongluan_from(ctx: &mut Ctx<'_>, source: PlayerId, total: PlayerId) {
    for s in 0..total {
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(s, format!("l11_xiongluan2_from:{source}")))
            .is_some()
        {
            let cur = flag(ctx.core, s, "l11_xiongluan2_hand_ban");
            if cur <= 1 {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(s, "l11_xiongluan2_hand_ban".to_string()));
            } else {
                set_flag(ctx.core, s, "l11_xiongluan2_hand_ban", cur - 1);
            }
        }
    }
}
