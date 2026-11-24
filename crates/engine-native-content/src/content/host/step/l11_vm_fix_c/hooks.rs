//! Lifecycle, Sha-begin, discard, and dying hooks for `HostBehavior::L11VmFixC`.

use super::helpers::*;
use super::*;
use crate::catalog::{Catalog, EquipSlot, Suit};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{Ctx, LogEvent};
use crate::state::GameStateCore;

pub(super) fn fuli_dying_save_skills(
    core: &GameStateCore,
    catalog: &Catalog,
    dying: PlayerId,
    saver: PlayerId,
) -> Vec<SkillDefId> {
    if dying != saver || !core.player(dying).alive {
        return Vec::new();
    }
    let Some(def) = catalog.skill_by_name("fuli") else {
        return Vec::new();
    };
    if catalog.skills[def as usize].behavior.as_host_behavior() != Some(HostBehavior::L11VmFixC) {
        return Vec::new();
    }
    if core.has_skill(dying, def)
        && is_vm_c_unblocked(core, catalog, dying, false)
        && flag(core, dying, "l11c_fuli_used") == 0
    {
        vec![def]
    } else {
        Vec::new()
    }
}

pub(super) fn resolve_fuli_dying_save(
    ctx: &mut Ctx<'_>,
    dying: PlayerId,
    saver: PlayerId,
    def: SkillDefId,
) -> bool {
    if dying != saver
        || !ctx.core.player(dying).alive
        || ctx.catalog.skills[def as usize].behavior.as_host_behavior() != Some(HostBehavior::L11VmFixC)
        || flag(ctx.core, dying, "l11c_fuli_used") != 0
    {
        return false;
    }
    set_flag(ctx.core, dying, "l11c_fuli_used", 1);
    log_skill(ctx, dying, "fuli");
    let groups = distinct_alive_factions(ctx.core, ctx.catalog) as i32;
    let delta = (groups - ctx.core.player(dying).hp).max(0) as u32;
    if delta > 0 {
        ctx.recover(dying, delta, None);
    }
    if ctx.core.player(dying).alive {
        ctx.turn_over(dying);
    }
    true
}

pub(super) fn on_game_start_l11_vm_c(core: &mut GameStateCore, catalog: &Catalog) {
    let seats = core.players.len() as PlayerId;
    for owner in 0..seats {
        if !core.player(owner).alive {
            continue;
        }
        for name in ["baonu", "nzry_jieying"] {
            if let Some(def) = catalog.skill_by_name(name) {
                if core.has_skill(owner, def) {
                    on_skill_granted_l11_vm_c(core, catalog, owner, def);
                }
            }
        }
    }
}

pub(super) fn on_skill_granted_l11_vm_c(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    def: SkillDefId,
) {
    let skill_name = catalog.skill_name(def);
    if skill_name == "xinfu_xionghuo" && flag(core, player, "xinfu_xionghuo_init") == 0 {
        set_flag(core, player, "xinfu_xionghuo_init", 1);
        set_flag(core, player, "xinfu_xionghuo_marks", 3);
        set_flag(core, player, "xionghuo", 3);
        return;
    }
    if catalog.skills[def as usize].behavior.as_host_behavior() != Some(HostBehavior::L11VmFixC) {
        return;
    }
    match skill_name {
        "baonu" => {
            if flag(core, player, "l11c_baonu_init") == 0 && !is_chanyuan_or_blank(core, catalog, player) {
                set_flag(core, player, "l11c_baonu_init", 1);
                let cur = flag(core, player, "l11c_baonu_marks");
                sync_mark_count(core, player, "baonu", cur + 2);
            }
        }
        "nzry_jieying" => {
            if flag(core, player, "l11c_jieying_init") == 0 && !is_chanyuan_or_blank(core, catalog, player) {
                set_flag(core, player, "l11c_jieying_init", 1);
                if !core.player(player).linked
                    && crate::rvm_bridge::modifiers::can_gain_link(core, catalog, player)
                {
                    core.player_mut(player).linked = true;
                }
            }
        }
        _ => {}
    }
}

pub(super) fn on_skill_removed_l11_vm_c(
    core: &mut GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
) -> Vec<CardId> {
    let mut reclaimed = Vec::new();
    match skill_name {
        "baonu" => {
            set_flag(core, owner, "l11c_baonu_init", 0);
        }
        "gnjinfan" => {
            let cards = pile_cards(core, owner, "gnjinfan");
            reclaimed.extend(take_from_pile(core, owner, "gnjinfan", &cards));
            sync_mark_count(core, owner, "gnjinfan", 0);
        }
        "nsyice" => {
            let cards = pile_cards(core, owner, "nsyice");
            reclaimed.extend(take_from_pile(core, owner, "nsyice", &cards));
            sync_mark_count(core, owner, "nsyice", 0);
        }
        "nzry_jieying" => {
            set_flag(core, owner, "l11c_jieying_init", 0);
        }
        "qiaoshui2" => {
            set_flag(core, owner, "l11c_qiaoshui2_turn", 0);
        }
        "qiaoshui3" => {
            set_flag(core, owner, "l11c_qiaoshui3_turn", 0);
        }
        "new_repaoxiao2" => {
            set_flag(core, owner, "l11c_new_repaoxiao2_turn", 0);
        }
        "reshanxi2" => {
            let cards = pile_cards(core, owner, "reshanxi2");
            reclaimed.extend(take_from_pile(core, owner, "reshanxi2", &cards));
            sync_mark_count(core, owner, "reshanxi2", 0);
        }
        "rekuangcai" => {
            set_flag(core, owner, "l11c_rekuangcai_pos", 0);
            set_flag(core, owner, "l11c_rekuangcai_neg", 0);
        }
        "rejizhi" => {
            set_flag(core, owner, "l11c_rejizhi_hand_bonus", 0);
            set_flag(core, owner, "l11c_rejizhi_hand_turn", 0);
        }
        _ => {}
    }
    let _ = catalog;
    reclaimed
}

pub(super) fn on_card_use_after_l11_vm_c(
    ctx: &mut Ctx<'_>,
    _fid: EventId,
    u: &crate::event::CardUseData,
) {
    let user = u.player;
    let tstamp = turn_stamp(ctx.core);
    if ctx.core.turn.current == user {
        if has_vm_c_skill(ctx.core, ctx.catalog, user, "rekuangcai") {
            set_flag(ctx.core, user, "l11c_turn_card_used_stamp", tstamp);
        }
        let eff_kind = u.effective_kind(ctx.catalog);
        let targets_other = u.targets.iter().any(|&t| t != user)
            || (u.targets.is_empty()
                && matches!(eff_kind, CardKind::Taoyuan | CardKind::Wugu)
                && (0..ctx.core.players.len() as PlayerId).any(|t| {
                    t != user
                        && ctx.core.player(t).alive
                        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            ctx.core,
                            ctx.catalog,
                            user,
                            t,
                            u.card.unwrap_or(0),
                            eff_kind,
                            crate::event::DamageNature::Normal,
                        )
                }));
        if targets_other
            && (0..ctx.core.players.len() as PlayerId).any(|p| {
                p != user
                    && ctx.core.player(p).alive
                    && has_vm_c_skill(ctx.core, ctx.catalog, p, "xinniluan")
            })
        {
            set_flag(ctx.core, user, "l11c_used_other_target_turn", tstamp);
        }
    }
    if let Some(c) = u.card {
        set_flag(ctx.core, user, &format!("l11c_twjiange_card:{c}"), 0);
        set_flag(ctx.core, user, &format!("l11c_xinniluan_card:{c}"), 0);
    }
    flush_twkujian_events(ctx);
    flush_gnjinfan_gain(ctx);
    flush_juece_empty_hands(ctx);
}

pub(super) fn on_damage_end_l11_vm_c(ctx: &mut Ctx<'_>, d: &crate::event::DamageData) {
    if d.prevented || d.amount <= 0 {
        return;
    }
    let tstamp = turn_stamp(ctx.core);
    let amt = d.amount as u32;
    if has_vm_c_skill(ctx.core, ctx.catalog, d.target, "shibei") {
        if flag(ctx.core, d.target, "l11c_turn_dmg_taken_stamp") != tstamp {
            set_flag(ctx.core, d.target, "l11c_turn_dmg_taken_stamp", tstamp);
            set_flag(ctx.core, d.target, "l11c_turn_dmg_taken_count", 1);
        } else {
            let cur = flag(ctx.core, d.target, "l11c_turn_dmg_taken_count");
            set_flag(ctx.core, d.target, "l11c_turn_dmg_taken_count", cur + 1);
        }
    }
    if let Some(src) = d.source {
        if has_vm_c_skill(ctx.core, ctx.catalog, src, "rekuangcai")
            || has_vm_c_skill(ctx.core, ctx.catalog, src, "xinniluan")
        {
            if flag(ctx.core, src, "l11c_turn_source_dmg_stamp") != tstamp {
                set_flag(ctx.core, src, "l11c_turn_source_dmg_stamp", tstamp);
                set_flag(ctx.core, src, "l11c_turn_source_dmg_amt", amt);
            } else {
                let cur = flag(ctx.core, src, "l11c_turn_source_dmg_amt");
                set_flag(ctx.core, src, "l11c_turn_source_dmg_amt", cur + amt);
            }
        }
    }
}

pub(super) fn on_phase_end_l11_vm_c(ctx: &mut Ctx<'_>) {
    flush_gnjinfan_gain(ctx);
    flush_twkujian_events(ctx);
    flush_juece_empty_hands(ctx);
}

pub(super) fn on_turn_phase_after_l11_vm_c(ctx: &mut Ctx<'_>, _player: PlayerId) {
    let seats = ctx.core.players.len() as PlayerId;
    for seat in 0..seats {
        set_flag(ctx.core, seat, "l11c_retieji_sealed", 0);
        // Defensive cleanup for `reshanxi2` if turn ended early.
        let exiled = pile_cards(ctx.core, seat, "reshanxi2");
        if !exiled.is_empty() && ctx.core.player(seat).alive {
            let taken = take_from_pile(ctx.core, seat, "reshanxi2", &exiled);
            sync_mark_count(ctx.core, seat, "reshanxi2", 0);
            ctx.core.player_mut(seat).hand.extend(taken);
            remove_named_skill(ctx, seat, "reshanxi2");
        }
    }
}

pub(super) fn on_cards_to_discard_l11_vm_c(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    cards: &[CardId],
    is_ordering_discard: bool,
) {
    if is_ordering_discard
        && !cards.is_empty()
        && has_live_l11c_skill(ctx.core, ctx.catalog, player, "nsyice", true)
    {
        let in_disc: Vec<CardId> = cards
            .iter()
            .copied()
            .filter(|c| ctx.core.discard.contains(c))
            .collect();
        if !in_disc.is_empty() {
            spawn_vm_c_park(ctx, "nsyice", player, Some(player), None, in_disc, Vec::new());
        }
    }
    flush_gnjinfan_gain(ctx);
    flush_twkujian_events(ctx);
}

pub(super) fn on_hand_card_removed_l11_vm_c(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    card: CardId,
) {
    let kujian_key = format!("l11c_twkujian_card:{card}");
    let kujian_owner_plus1 = flag(ctx.core, player, &kujian_key);
    let used_or_resp_key = format!("l11c_twkujian_use_resp:{card}");
    let was_use_resp = ctx
        .core
        .vm_aux
        .flags
        .remove(&(player, used_or_resp_key))
        .is_some();
    if kujian_owner_plus1 > 0 {
        set_flag(ctx.core, player, &kujian_key, 0);
        let kj_owner = (kujian_owner_plus1 - 1) as PlayerId;
        if kj_owner != player {
            if was_use_resp {
                let cnt = flag(ctx.core, kj_owner, &format!("l11c_twkujian_pend_draw:{player}"));
                set_flag(
                    ctx.core,
                    kj_owner,
                    &format!("l11c_twkujian_pend_draw:{player}"),
                    cnt + 1,
                );
            } else {
                let cnt = flag(ctx.core, kj_owner, &format!("l11c_twkujian_pend_disc:{player}"));
                set_flag(
                    ctx.core,
                    kj_owner,
                    &format!("l11c_twkujian_pend_disc:{player}"),
                    cnt + 1,
                );
            }
        }
    }
    if ctx.core.player(player).hand.is_empty()
        && ctx.core.player(player).alive
        && player != ctx.core.turn.current
    {
        let current = ctx.core.turn.current;
        if has_live_l11c_skill(ctx.core, ctx.catalog, current, "juece", false) {
            set_flag(
                ctx.core,
                current,
                &format!("l11c_juece_empty_cand:{player}"),
                turn_stamp(ctx.core),
            );
        }
    }
}

pub(super) fn on_outer_settled_l11_vm_c(ctx: &mut Ctx<'_>) {
    flush_nsyice_respond_pending(ctx);
    flush_twkujian_events(ctx);
    flush_gnjinfan_gain(ctx);
    flush_juece_empty_hands(ctx);
}

fn flush_nsyice_respond_pending(ctx: &mut Ctx<'_>) {
    let keys: Vec<(PlayerId, String, u32)> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter(|((_, k), &v)| v > 0 && k.starts_with("l11c_nsyice_resp_card:"))
        .map(|((p, k), v)| (*p, k.clone(), *v))
        .collect();
    for (p, key, _) in keys {
        ctx.core.vm_aux.flags.remove(&(p, key.clone()));
        if let Ok(c) = key.trim_start_matches("l11c_nsyice_resp_card:").parse::<CardId>() {
            if ctx.core.discard.contains(&c)
                && has_live_l11c_skill(ctx.core, ctx.catalog, p, "nsyice", true)
            {
                spawn_vm_c_park(ctx, "nsyice", p, Some(p), None, vec![c], Vec::new());
            }
        }
    }
}

fn flush_twkujian_events(ctx: &mut Ctx<'_>) {
    let draw_keys: Vec<(PlayerId, String, u32)> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter(|((_, k), &v)| v > 0 && k.starts_with("l11c_twkujian_pend_draw:"))
        .map(|((p, k), v)| (*p, k.clone(), *v))
        .collect();
    for (kj_owner, key, count) in draw_keys {
        ctx.core.vm_aux.flags.remove(&(kj_owner, key.clone()));
        if !has_live_l11c_skill(ctx.core, ctx.catalog, kj_owner, "twkujian", false) {
            continue;
        }
        if let Ok(user) = key.trim_start_matches("l11c_twkujian_pend_draw:").parse::<PlayerId>() {
            for _ in 0..count {
                spawn_vm_c_park(
                    ctx,
                    "twkujian_draw",
                    kj_owner,
                    Some(user),
                    None,
                    Vec::new(),
                    Vec::new(),
                );
            }
        }
    }
    let disc_keys: Vec<(PlayerId, String, u32)> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter(|((_, k), &v)| v > 0 && k.starts_with("l11c_twkujian_pend_disc:"))
        .map(|((p, k), v)| (*p, k.clone(), *v))
        .collect();
    for (kj_owner, key, count) in disc_keys {
        ctx.core.vm_aux.flags.remove(&(kj_owner, key.clone()));
        if !has_live_l11c_skill(ctx.core, ctx.catalog, kj_owner, "twkujian", false) {
            continue;
        }
        if let Ok(loser) = key.trim_start_matches("l11c_twkujian_pend_disc:").parse::<PlayerId>() {
            for _ in 0..count {
                spawn_vm_c_park(
                    ctx,
                    "twkujian_discard",
                    kj_owner,
                    Some(loser),
                    None,
                    Vec::new(),
                    Vec::new(),
                );
            }
        }
    }
}

pub(super) fn flush_gnjinfan_gain(ctx: &mut Ctx<'_>) {
    let lost_keys: Vec<(PlayerId, String, u32)> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter(|((_, k), &v)| v > 0 && k.starts_with("l11c_gnjinfan_lost:"))
        .map(|((p, k), v)| (*p, k.clone(), *v))
        .collect();
    for (owner, key, _) in lost_keys {
        ctx.core.vm_aux.flags.remove(&(owner, key.clone()));
        let Ok(c) = key.trim_start_matches("l11c_gnjinfan_lost:").parse::<CardId>() else {
            continue;
        };
        let remaining_len = pile_cards(ctx.core, owner, "gnjinfan").len() as u32;
        sync_mark_count(ctx.core, owner, "gnjinfan", remaining_len);
        if !has_live_l11c_skill(ctx.core, ctx.catalog, owner, "gnjinfan", false) {
            continue;
        }
        let want_suit = ctx
            .catalog
            .cards
            .get(c as usize)
            .map(|cd| cd.suit)
            .unwrap_or(Suit::Spade);
        if let Some(pos) = ctx.core.deck.iter().rposition(|&dc| {
            ctx.catalog
                .cards
                .get(dc as usize)
                .is_some_and(|cd| cd.suit == want_suit)
        }) {
            let gained = ctx.core.deck.remove(pos);
            log_skill(ctx, owner, "gnjinfan_gain");
            ctx.core.player_mut(owner).hand.push(gained);
            ctx.log(LogEvent::CardsDrawn {
                player: owner,
                cards: vec![gained],
            });
        }
    }
}

fn flush_juece_empty_hands(ctx: &mut Ctx<'_>) {
    let tstamp = turn_stamp(ctx.core);
    let keys: Vec<(PlayerId, String, u32)> = ctx
        .core
        .vm_aux
        .flags
        .iter()
        .filter(|((_, k), &v)| v > 0 && k.starts_with("l11c_juece_empty_cand:"))
        .map(|((p, k), v)| (*p, k.clone(), *v))
        .collect();
    for (owner, key, stamp) in keys {
        ctx.core.vm_aux.flags.remove(&(owner, key.clone()));
        if stamp != tstamp
            || ctx.core.turn.current != owner
            || !has_live_l11c_skill(ctx.core, ctx.catalog, owner, "juece", false)
        {
            continue;
        }
        if let Ok(victim) = key.trim_start_matches("l11c_juece_empty_cand:").parse::<PlayerId>() {
            if victim != owner
                && ctx.core.player(victim).alive
                && ctx.core.player(victim).hand.is_empty()
            {
                spawn_vm_c_park(ctx, "juece", owner, Some(victim), None, Vec::new(), Vec::new());
            }
        }
    }
}

pub(super) fn sha_begin_step_l11_vm_c(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
    priority_ge_6: bool,
) -> bool {
    if !priority_ge_6 {
        return false;
    }
    let cid = sha_card.unwrap_or(0);
    if has_live_l11c_skill(ctx.core, ctx.catalog, source, "jintao", true)
        && ctx.core.turn.phase == crate::event::Phase::Play
        && ctx.core.turn.current == source
    {
        let cnt = current_sha_damage_base_from_jintao_check(ctx.core, source);
        let jintao_key = format!("l11c_jintao_seen:{sha_frame_id}");
        if cnt == 1 && flag(ctx.core, source, &jintao_key) == 0 {
            set_flag(ctx.core, source, &jintao_key, 1);
            log_skill(ctx, source, "jintao");
            let prev = flag(ctx.core, source, &format!("sha_frame_damage_bonus:{sha_frame_id}"));
            set_flag(
                ctx.core,
                source,
                &format!("sha_frame_damage_bonus:{sha_frame_id}"),
                prev + 1,
            );
        } else if cnt == 2 {
            let unresp_seen = format!("l11c_jintao_unresp:{sha_frame_id}:{target}");
            if flag(ctx.core, source, &unresp_seen) == 0 {
                set_flag(ctx.core, source, &unresp_seen, 1);
                log_skill(ctx, source, "jintao");
                set_flag(
                    ctx.core,
                    source,
                    &format!("sha_unrespondable:{cid}:{target}"),
                    1,
                );
            }
        }
    }
    if has_live_l11c_skill(ctx.core, ctx.catalog, source, "xinanjian", true)
        && !ctx.core.in_attack_range(ctx.catalog, target, source)
    {
        let key = format!("l11c_xinanjian_done:{sha_frame_id}:{target}");
        if flag(ctx.core, source, &key) == 0 {
            set_flag(ctx.core, source, &key, 1);
            spawn_vm_c_park_in_slot(
                ctx,
                crate::resolution::Slot::Trigger {
                    frame: sha_frame_id,
                },
                "xinanjian",
                source,
                Some(target),
                Some(cid),
                Vec::new(),
                Vec::new(),
            );
            return true;
        }
    }
    if has_live_l11c_skill(ctx.core, ctx.catalog, source, "retieji", false) {
        let key = format!("l11c_retieji_done:{sha_frame_id}:{target}");
        if flag(ctx.core, source, &key) == 0 {
            set_flag(ctx.core, source, &key, 1);
            spawn_vm_c_park_in_slot(
                ctx,
                crate::resolution::Slot::Trigger {
                    frame: sha_frame_id,
                },
                "retieji",
                source,
                Some(target),
                Some(cid),
                Vec::new(),
                Vec::new(),
            );
            return true;
        }
    }
    if has_live_l11c_skill(ctx.core, ctx.catalog, target, "yizhong", true) {
        let has_armor_slot =
            crate::rvm_ops::has_enabled_equip_slot(ctx.core, target, EquipSlot::Armor);
        let armor_empty = !ctx
            .core
            .player(target)
            .equips
            .iter()
            .any(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Armor));
        let is_black = sha_card.is_some_and(|c| {
            matches!(
                card_suit(ctx.core, ctx.catalog, source, c),
                Suit::Spade | Suit::Club
            )
        });
        if has_armor_slot && armor_empty && is_black {
            log_skill(ctx, target, "yizhong");
            set_flag(
                ctx.core,
                target,
                &format!("sha_cancelled_target:{sha_frame_id}"),
                1,
            );
        }
    }
    false
}
