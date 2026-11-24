//! Shared helpers for `HostBehavior::L11VmFixC`.

pub(super) use super::super::l11_common::{
    alive_all, alive_others, discard_player_cards, emit_damage, emit_lose_hp, ensure_perm_skill,
    has_named_skill, he_cards, is_locked_unblocked_l11_owner, is_unblocked_l11_owner,
    move_cards_to_hand, remove_named_skill,
};
pub(super) use super::super::l11_npc_b::helpers::{
    card_effective_rank_with_mods, card_usable_on_target_with_card, distinct_alive_factions,
    equip_card_onto_player, flag, is_red_suit, is_weapon_card, log_skill, phase_stamp, pile_cards,
    push_to_pile, set_flag, suit_bit, take_from_pile, take_owned_cards_from_zones, turn_stamp,
};
use super::super::*;
use super::asks::build_ask_l11_vm_c;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::event::{DamageNature, EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{card_use_as, Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(super) const QICE_KINDS: [CardKind; 11] = [
    CardKind::Wuzhong,
    CardKind::Dismantle,
    CardKind::Snatch,
    CardKind::Duel,
    CardKind::SavageAttack,
    CardKind::ArcheryAttack,
    CardKind::Taoyuan,
    CardKind::Wugu,
    CardKind::Linghunzhihuo,
    CardKind::Chain,
    CardKind::Jiedao,
];

pub(super) fn card_use_virtual(
    player: PlayerId,
    targets: Vec<PlayerId>,
    nature: DamageNature,
    use_as: CardKind,
) -> crate::event::CardUseData {
    crate::event::CardUseData {
        card: None,
        player,
        targets,
        nature,
        use_as: Some(use_as),
        cancelled: false,
    }
}

pub(super) fn is_vm_c_unblocked(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    locked: bool,
) -> bool {
    if locked {
        is_locked_unblocked_l11_owner(state, catalog, owner)
    } else {
        is_unblocked_l11_owner(state, catalog, owner)
    }
}

pub(super) fn hes_cards(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let mut out = he_cards(core, p);
    out.extend(pile_cards(core, p, "gnjinfan"));
    out
}

pub(super) fn has_vm_c_skill(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    name: &str,
) -> bool {
    catalog.skill_by_name(name).is_some_and(|def| {
        catalog.skills[def as usize].behavior.as_host_behavior() == Some(HostBehavior::L11VmFixC)
            && state.has_skill(player, def)
    })
}

pub(super) fn has_live_l11c_skill(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    name: &str,
    locked: bool,
) -> bool {
    has_vm_c_skill(state, catalog, player, name)
        && is_vm_c_unblocked(state, catalog, player, locked)
}

pub(super) fn is_chanyuan_or_blank(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    catalog.is_chanyuan_sealed(state, player)
        || has_named_skill(state, catalog, player, "txhj_feilong_skill_blank")
}

pub(super) fn sync_mark_count(
    core: &mut GameStateCore,
    owner: PlayerId,
    name: &str,
    val: u32,
) {
    set_flag(core, owner, name, val);
    set_flag(core, owner, &format!("mark:{name}"), val);
    set_flag(core, owner, &format!("l11c_{name}_marks"), val);
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l11c_ask_id", ask_id);
    m.stage = stage;
    if let Some((observer, spec)) = build_ask_l11_vm_c(
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l11c_ask_id", 0);
        Flow::Done
    }
}

pub(super) fn clear_ask(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "l11c_ask_id", 0);
}

pub(super) fn spawn_vm_c_park_in_slot(
    ctx: &mut Ctx<'_>,
    slot: crate::resolution::Slot,
    def_key: &str,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
) {
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot,
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: def_key.to_string(),
            behavior: HostBehavior::L11VmFixC,
            owner,
            target,
            card,
            cards,
            targets,
            stage: HostStage::Act,
        }),
    });
}

pub(super) fn spawn_vm_c_park(
    ctx: &mut Ctx<'_>,
    def_key: &str,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
) {
    spawn_vm_c_park_in_slot(
        ctx,
        crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        def_key,
        owner,
        target,
        card,
        cards,
        targets,
    );
}

pub(super) fn finish_respond_result(ctx: &mut Ctx<'_>, owner: PlayerId, ok: bool) {
    let fid = flag(ctx.core, owner, "u01_respond_fid");
    set_flag(ctx.core, owner, "u01_respond_fid", 0);
    set_flag(
        ctx.core,
        owner,
        &format!("u01_respond_result:{fid}"),
        u32::from(ok),
    );
}

/// Consume cards from `'hes'` (`hand` / `equips` / `'gnjinfan'` pile) to discard pile,
/// emitting `CardsDiscarded`/`CardsLost` and marking `l11c_gnjinfan_lost:{c}` for pile cards.
pub(super) fn discard_hes_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut he = Vec::new();
    let mut from_s = Vec::new();
    for &c in cards {
        if ctx.core.player(p).hand.contains(&c) || ctx.core.player(p).equips.contains(&c) {
            he.push(c);
        } else if !take_from_pile(ctx.core, p, "gnjinfan", &[c]).is_empty() {
            from_s.push(c);
        }
    }
    if !he.is_empty() {
        discard_player_cards(ctx, p, &he);
    }
    if !from_s.is_empty() {
        for &c in &from_s {
            ctx.to_discard(c);
            set_flag(ctx.core, p, &format!("l11c_gnjinfan_lost:{c}"), 1);
        }
        ctx.note_cards_lost(p, &from_s);
        ctx.log(LogEvent::CardDiscarded {
            player: p,
            cards: from_s.clone(),
        });
        let origins = vec![crate::event::LostOrigin::Hand; from_s.len()];
        ctx.emit(EventData::CardsLost {
            player: p,
            cards: from_s.clone(),
            origins,
        });
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: from_s,
        });
    }
}

pub(super) fn card_suit(core: &GameStateCore, catalog: &Catalog, p: PlayerId, c: CardId) -> Suit {
    catalog.effective_card_suit(core, p, c)
}

pub(super) fn card_matches_suit(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    c: CardId,
    suit: Suit,
) -> bool {
    card_suit(core, catalog, p, c) == suit
}

pub(super) fn longhun_need(core: &GameStateCore, owner: PlayerId) -> usize {
    core.player(owner).hp.max(1) as usize
}

pub(super) fn qice_hand_usable(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    !core.player(owner).hand.is_empty()
        && !catalog.blocks_hand_cards(core, owner)
        && !crate::rvm_bridge::modifiers::d34_flags::is_u01_hand_locked(core, catalog, owner)
        && flag(core, owner, "l11c_qice_phase") != phase_stamp(core)
        && !qice_legal_tricks(core, catalog, owner).is_empty()
}

pub(super) fn qice_legal_tricks(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<u8> {
    QICE_KINDS
        .iter()
        .enumerate()
        .filter_map(|(idx, &k)| trick_usable_qice(core, catalog, owner, k).then_some(idx as u8))
        .collect()
}

pub(super) fn trick_usable_qice(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
) -> bool {
    let nature = if kind == CardKind::Linghunzhihuo {
        DamageNature::Fire
    } else {
        DamageNature::Normal
    };
    if crate::rvm_bridge::modifiers::eval_card_use_canceled(core, catalog, owner, 0, kind, nature) {
        return false;
    }
    match kind {
        CardKind::Wuzhong => true,
        CardKind::Taoyuan | CardKind::Wugu | CardKind::SavageAttack | CardKind::ArcheryAttack => {
            alive_all(core, owner).into_iter().any(|t| {
                !((kind == CardKind::SavageAttack || kind == CardKind::ArcheryAttack) && t == owner)
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core,
                        catalog,
                        owner,
                        t,
                        0,
                        kind,
                        DamageNature::Normal,
                    )
            })
        }
        CardKind::Jiedao => !qice_jiedao_pairs(core, catalog, owner).is_empty(),
        CardKind::Chain => alive_all(core, owner).into_iter().any(|t| {
            !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core,
                catalog,
                owner,
                t,
                0,
                kind,
                DamageNature::Normal,
            )
        }),
        _ => !single_targets_for_virtual_trick(core, catalog, owner, kind, false).is_empty(),
    }
}

pub(super) fn qice_jiedao_pairs(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<(PlayerId, PlayerId)> {
    let mut pairs = Vec::new();
    for h in alive_others(core, owner) {
        if !core.player(h).equips.iter().any(|&c| is_weapon_card(catalog, core, c)) {
            continue;
        }
        if crate::rvm_bridge::modifiers::eval_card_target_canceled(
            core,
            catalog,
            owner,
            h,
            0,
            CardKind::Jiedao,
            DamageNature::Normal,
        ) {
            continue;
        }
        for v in alive_all(core, owner) {
            if v != h && core.in_attack_range(catalog, h, v) {
                pairs.push((h, v));
            }
        }
    }
    pairs
}

pub(super) fn single_targets_for_virtual_trick(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
    ignore_distance: bool,
) -> Vec<PlayerId> {
    let kongcheng_def = catalog.skill_by_name(crate::content::standard::KONG_CHENG);
    let qianxun_def = catalog.skill_by_name(crate::content::standard::QIAN_XUN);
    let pool = if kind == CardKind::Chain {
        alive_all(core, owner)
    } else {
        alive_others(core, owner)
    };
    pool.into_iter()
        .filter(|&t| {
            if kind == CardKind::Duel
                && kongcheng_def.is_some_and(|d| core.has_skill(t, d))
                && core.player(t).hand.is_empty()
            {
                return false;
            }
            if kind == CardKind::Snatch
                && qianxun_def.is_some_and(|d| core.has_skill(t, d))
            {
                return false;
            }
            card_usable_on_target_with_card(core, catalog, owner, t, 0, kind, ignore_distance)
        })
        .collect()
}

pub(super) fn sha_targets_for_card(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    ignore_distance: bool,
) -> Vec<PlayerId> {
    let cid = card.unwrap_or(0);
    let kongcheng_def = catalog.skill_by_name(crate::content::standard::KONG_CHENG);
    alive_others(core, owner)
        .into_iter()
        .filter(|&t| {
            if kongcheng_def.is_some_and(|d| core.has_skill(t, d))
                && core.player(t).hand.is_empty()
            {
                return false;
            }
            card_usable_on_target_with_card(core, catalog, owner, t, cid, kind, ignore_distance)
        })
        .collect()
}

pub(super) fn can_use_sha_limit_for_card(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    force_no_count: bool,
) -> bool {
    if core.turn.current != owner || core.turn.phase != Phase::Play {
        return true;
    }
    if force_no_count
        || crate::rvm_bridge::modifiers::eval_sha_no_use_count(core, catalog, owner, card, kind)
        || crate::rvm_bridge::modifiers::eval_card_use_unlimited(
            core,
            catalog,
            owner,
            card,
            kind,
            DamageNature::Normal,
        )
    {
        return true;
    }
    let (limit, rvm_unlim) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        owner,
        card,
        kind,
        DamageNature::Normal,
        1,
    );
    if rvm_unlim {
        return true;
    }
    let count = if core.turn.sha_used_play_phase {
        flag(core, owner, "sha_used_count").max(1)
    } else {
        0
    };
    count < limit
}

pub(super) fn current_sha_damage_base_from_jintao_check(core: &GameStateCore, owner: PlayerId) -> u32 {
    flag(core, owner, "q4_play_sha_count")
}

pub(super) fn reshanxi_cost_cards(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            is_red_suit(card_suit(core, catalog, owner, c))
                && catalog.resolved_card_kind(core, c).is_basic_kind()
        })
        .collect()
}

pub(super) fn xinniluan_usable_cards(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<CardId> {
    let mut out = Vec::new();
    for &c in &hes_cards(core, owner) {
        if catalog.resolved_card_kind(core, c).is_sha_like() {
            out.push(c);
        }
    }
    out
}

pub(super) fn emit_sha_card_use(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    card: Option<CardId>,
    targets: Vec<PlayerId>,
    kind: CardKind,
    no_count: bool,
) {
    if let Some(c) = card {
        if no_count {
            set_flag(ctx.core, owner, &format!("l11c_twjiange_card:{c}"), 1);
            set_flag(ctx.core, owner, &format!("l11c_xinniluan_card:{c}"), 1);
        }
        ctx.emit(EventData::CardUsed(card_use_as(c, owner, targets, Some(kind))));
    } else {
        ctx.emit(EventData::CardUsed(card_use_virtual(
            owner,
            targets,
            if kind == CardKind::FireSha {
                DamageNature::Fire
            } else if kind == CardKind::LeiSha {
                DamageNature::Thunder
            } else {
                DamageNature::Normal
            },
            kind,
        )));
    }
}
