use super::super::super::*;
use super::super::l11_common::{active_skill_machine, alive_others, flag, he_cards};
use super::helpers::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, SkillMachine, Suit, TriggerMeta};
use crate::event::{DamageNature, DrawOutcome, EventData, EventKind, Phase, Timing};
use crate::ids::{EventId, PlayerId};
use crate::state::GameStateCore;

fn meta(
    event: EventKind,
    timing: Timing,
    priority: i32,
    optional: bool,
    extra_events: Vec<EventKind>,
) -> TriggerMeta {
    TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: optional,
        optional,
        extra_events,
    }
}

pub(in crate::content::host) fn trigger_meta(name: &str) -> Option<TriggerMeta> {
    match name {
        "miji" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, true, Vec::new())),
        "fuji" => Some(meta(EventKind::DamageBegin1, Timing::Normal, 1, false, Vec::new())),
        "fulin" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 2, false, Vec::new())),
        "refankui" => Some(meta(EventKind::Damage, Timing::Normal, 1, false, Vec::new())),
        "fenwei" => Some(meta(EventKind::CardUsed, Timing::Normal, 2, false, Vec::new())),
        "kuangfu" => Some(meta(EventKind::DamageSource, Timing::Normal, 1, false, Vec::new())),
        "langmie" => Some(meta(
            EventKind::PhaseEnd,
            Timing::Normal,
            1,
            false,
            vec![EventKind::TurnPhaseEnd],
        )),
        "wushen" => Some(meta(EventKind::CardUsed, Timing::Normal, 3, false, Vec::new())),
        "yingjian" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, false, Vec::new())),
        "qiaomeng" => Some(meta(EventKind::DamageSource, Timing::Normal, 1, false, Vec::new())),
        "shenxian" => Some(meta(
            EventKind::CardsDiscarded,
            Timing::Normal,
            1,
            false,
            vec![EventKind::CardsLost],
        )),
        "jiezi" => Some(meta(EventKind::DrawPhaseAfter, Timing::Normal, 1, false, Vec::new())),
        "lianpo" => Some(meta(EventKind::TurnPhaseAfter, Timing::Normal, 1, false, Vec::new())),
        "bihuo" => Some(meta(EventKind::DyingSaved, Timing::Normal, 1, false, Vec::new())),
        "chunlao" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, false, Vec::new())),
        "fenming" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, false, Vec::new())),
        "fenyin" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            false,
            vec![EventKind::Equipped],
        )),
        "guzheng" => Some(meta(EventKind::PhaseEnd, Timing::Normal, 1, false, Vec::new())),
        "gzyinghun" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, false, Vec::new())),
        "jianying" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            false,
            vec![EventKind::Equipped],
        )),
        "kaikang" => Some(meta(EventKind::CardUsed, Timing::Normal, 1, false, Vec::new())),
        "kunfen" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, false, Vec::new())),
        "lihuo" => Some(meta(EventKind::CardUsed, Timing::First, 1, false, Vec::new())),
        "manji" => Some(meta(EventKind::LoseHp, Timing::Normal, 1, false, Vec::new())),
        "nzry_longnu" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 2, false, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits(name: &str) -> SkillLimits {
    match name {
        "wurong" | "qiangwu" | "mingce" | "nsyangwu" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn has_phase_use(name: &str) -> bool {
    matches!(
        name,
        "wurong" | "yeyan" | "qiangwu" | "drlt_xiongluan" | "mingce" | "nsyangwu"
    )
}

pub(in crate::content::host) fn activate(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if state.turn.phase != Phase::Play
        || state.turn.current != owner
        || !owner_unblocked(state, catalog, owner, &skill.name, false)
    {
        return None;
    }
    let def = catalog.skill_by_name(&skill.name)?;
    let inst = state.skills.get(&(owner, def))?;
    if !inst.enabled || inst.once_used {
        return None;
    }
    let ok = match skill.name.as_str() {
        "wurong" => {
            !state.player(owner).hand.is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "yeyan" => state.alive_players().next().is_some(),
        "qiangwu" => !state.deck.is_empty() || !state.discard.is_empty(),
        "drlt_xiongluan" => {
            !alive_others(state, owner).is_empty()
                && (flag(state, owner, "l11_judge_slot_disabled") == 0
                    || [
                        crate::catalog::EquipSlot::Weapon,
                        crate::catalog::EquipSlot::Armor,
                        crate::catalog::EquipSlot::HorsePlus,
                        crate::catalog::EquipSlot::HorseMinus,
                        crate::catalog::EquipSlot::Treasure,
                    ]
                    .into_iter()
                    .any(|s| catalog.is_equip_slot_enabled(state, owner, s)))
        }
        "mingce" => {
            !mingce_eligible_cards(state, catalog, owner).is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| is_mingce_give_candidate(state, catalog, owner, p))
        }
        "nsyangwu" => {
            let my_hand = state.player(owner).hand.len();
            state
                .player(owner)
                .hand
                .iter()
                .any(|&c| catalog.effective_card_suit(state, owner, c) == Suit::Heart)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| state.player(p).hand.len() > my_hand)
        }
        _ => false,
    };
    ok.then(|| active_skill_machine(&skill.name, HostBehavior::L11VmFixA, owner))
        .flatten()
}

pub(in crate::content::host) fn filter_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return false;
    }
    let Some(def) = catalog.skill_by_name(&skill.name) else {
        return false;
    };
    let locked = matches!(
        skill.name.as_str(),
        "fulin" | "wushen" | "jiezi" | "manji" | "nzry_longnu"
    ) || (skill.name == "kunfen" && flag(state, owner, "kunfen_unlocked") == 0);
    if !owner_unblocked(state, catalog, owner, &skill.name, locked) {
        return false;
    }
    match (skill.name.as_str(), event) {
        ("miji", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && state.player(owner).hp < state.player(owner).max_hp as i32
        }
        ("fuji", EventData::Damage(d)) => {
            !d.prevented
                && d.amount > 0
                && d.nature == DamageNature::Thunder
                && d.source.is_some_and(|s| state.player(s).alive)
                && (!state.deck.is_empty() || !state.discard.is_empty())
        }
        ("fulin", EventData::Phase { player, phase: Phase::Discard, begin: true }) => {
            *player == owner
        }
        ("refankui", EventData::Damage(d)) => {
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && d.source.is_some_and(|s| {
                    state.player(s).alive
                        && if s != owner {
                            !he_cards(state, s).is_empty()
                        } else {
                            !state.player(s).equips.is_empty()
                        }
                })
        }
        ("fenwei", EventData::CardUsed(u)) => {
            let unused = state.skills.get(&(owner, def)).is_some_and(|i| !i.once_used);
            let eff = u.effective_kind_in(state, catalog);
            let is_trick = !eff.is_basic_kind() && eff.equip_slot().is_none() && !is_delay_trick_kind(eff);
            unused
                && !u.cancelled
                && is_trick
                && !is_multitarget_trick_kind(eff)
                && fenwei_effective_targets(state, catalog, u).len() > 1
        }
        ("kuangfu", EventData::Damage(d)) => {
            !d.prevented
                && d.amount > 0
                && d.source == Some(owner)
                && state.player(d.target).alive
                && !state.player(d.target).equips.is_empty()
                && is_sha_damage_in_stack(state, catalog, frames, d)
        }
        ("langmie", EventData::Phase { player, phase: Phase::Play, begin: false }) => {
            *player != owner
                && !he_cards(state, owner).is_empty()
                && (0..=2u32).any(|cat| {
                    flag(state, *player, &format!("l11_langmie_play_cat:{cat}")) >= 2
                })
        }
        ("langmie", EventData::TurnEndWindow { player, after: false }) => {
            *player != owner
                && state.player(*player).alive
                && !he_cards(state, owner).is_empty()
                && flag(state, *player, "l11_turn_dmg_dealt") > 1
        }
        ("wushen", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && u.effective_kind_in(state, catalog).is_sha_like()
                && crate::kernel::card_use_suit(state, catalog, u) == Some(Suit::Heart)
        }
        ("yingjian", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && state
                    .alive_players()
                    .any(|p| can_virtual_sha_target(state, catalog, owner, p, true))
        }
        ("qiaomeng", EventData::Damage(d)) => {
            !d.prevented
                && d.amount > 0
                && d.source == Some(owner)
                && state.player(d.target).alive
                && !state.player(d.target).equips.is_empty()
                && is_black_sha_damage_in_stack(state, catalog, frames, d)
        }
        ("shenxian", EventData::CardsDiscarded { player, cards }) => {
            state.turn.current != owner
                && *player != owner
                && flag(state, owner, "l11_shenxian_used") == 0
                && cards.iter().any(|&c| {
                    flag(state, *player, &format!("l11_discard_judge:{c}")) == 0
                        && catalog.resolved_card_kind(state, c).is_basic_kind()
                })
        }
        ("shenxian", EventData::CardsLost { player, cards, origins }) => {
            state.turn.current != owner
                && *player != owner
                && flag(state, owner, "l11_shenxian_used") == 0
                && cards.iter().zip(origins.iter()).any(|(&c, &orig)| {
                    !matches!(orig, crate::event::LostOrigin::Judge)
                        && catalog.resolved_card_kind(state, c).is_basic_kind()
                        && flag(state, *player, &format!("u01_discard_reason:{c}")) > 0
                })
        }
        ("jiezi", EventData::DrawPhase { player, outcome, .. }) => {
            *player != owner && matches!(outcome, DrawOutcome::Skipped | DrawOutcome::Cancelled)
        }
        ("lianpo", EventData::TurnEndWindow { after: true, .. }) => {
            flag(state, owner, "l11_lianpo_kills") > 0
        }
        ("bihuo", EventData::DyingSaved { player, .. }) => {
            let unused = state.skills.get(&(owner, def)).is_some_and(|i| !i.once_used);
            unused && state.player(*player).alive
        }
        ("chunlao", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
                && state.vm_aux.piles.get(&(owner, "chunlao".to_string())).is_none_or(|p| p.is_empty())
                && state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| catalog.resolved_card_kind(state, c).is_sha_like())
        }
        ("fenming", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner && state.player(owner).linked
        }
        ("fenyin", EventData::CardUsed(u)) => {
            let fid = frame.map_or(0, |f| f.id);
            !u.cancelled
                && u.player == owner
                && state.turn.current == owner
                && check_fenyin_match(state, catalog, owner, fid, crate::kernel::card_use_suit(state, catalog, u), false)
        }
        ("fenyin", EventData::Equipped { player, card, .. }) => {
            let fid = frame.map_or(0, |f| f.id);
            *player == owner
                && state.turn.current == owner
                && flag(state, owner, &format!("l11_play_equip_use:{fid}")) > 0
                && check_fenyin_match(state, catalog, owner, fid, Some(catalog.effective_card_suit(state, owner, *card)), true)
        }
        ("guzheng", EventData::Phase { player, phase: Phase::Discard, begin: false }) => {
            *player != owner && !guzheng_discarded_lists(state, *player).1.is_empty()
        }
        ("gzyinghun", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            *player == owner
                && state.player(owner).hp < state.player(owner).max_hp as i32
                && !alive_others(state, owner).is_empty()
        }
        ("jianying", EventData::CardUsed(u)) => {
            let fid = frame.map_or(0, |f| f.id);
            !u.cancelled
                && u.player == owner
                && state.turn.current == owner
                && state.turn.phase == Phase::Play
                && check_jianying_match(
                    state,
                    catalog,
                    owner,
                    fid,
                    crate::kernel::card_use_suit(state, catalog, u),
                    u.card.and_then(|c| catalog.cards.get(c as usize).map(|cd| cd.rank)).unwrap_or(0),
                    false,
                )
        }
        ("jianying", EventData::Equipped { player, card, .. }) => {
            let fid = frame.map_or(0, |f| f.id);
            *player == owner
                && state.turn.current == owner
                && state.turn.phase == Phase::Play
                && flag(state, owner, &format!("l11_play_equip_use:{fid}")) > 0
                && check_jianying_match(
                    state,
                    catalog,
                    owner,
                    fid,
                    Some(catalog.effective_card_suit(state, owner, *card)),
                    catalog.cards.get(*card as usize).map_or(0, |cd| cd.rank),
                    true,
                )
        }
        ("kaikang", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.effective_kind_in(state, catalog).is_sha_like()
                && u.targets
                    .iter()
                    .any(|&t| state.player(t).alive && state.computed_distance(catalog, owner, t) <= 1)
        }
        ("kunfen", EventData::Phase { player, phase: Phase::End, begin: true }) => {
            *player == owner
        }
        ("lihuo", EventData::CardUsed(u)) => {
            if u.player != owner || u.cancelled {
                return false;
            }
            let eff = u.effective_kind_in(state, catalog);
            (eff == CardKind::Sha && u.nature == DamageNature::Normal)
                || ((eff == CardKind::FireSha || u.nature == DamageNature::Fire)
                    && state.alive_players().any(|p| {
                        p != owner
                            && !u.targets.contains(&p)
                            && can_virtual_sha_target(state, catalog, owner, p, false)
                    }))
        }
        ("manji", EventData::LoseHp { player, amount }) => {
            let ow = state.player(owner);
            let tp = state.player(*player);
            *amount > 0 && *player != owner && (ow.hp >= tp.hp || ow.hp < ow.max_hp as i32)
        }
        ("nzry_longnu", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            *player == owner
        }
        _ => false,
    }
}

fn check_fenyin_match(
    state: &GameStateCore,
    _catalog: &Catalog,
    owner: PlayerId,
    fid: EventId,
    suit: Option<Suit>,
    is_equip: bool,
) -> bool {
    let pkey = if is_equip {
        format!("l11_prev_eq_turn:{fid}")
    } else {
        format!("l11_prev_card_turn:{fid}")
    };
    let prev = flag(state, owner, &pkey);
    let prev_color = (prev >> 16) & 0xff;
    let cur_color = match suit {
        Some(Suit::Heart | Suit::Diamond) => 1u32,
        Some(Suit::Spade | Suit::Club) => 2u32,
        None => 0u32,
    };
    prev_color != 0 && cur_color != 0 && prev_color != cur_color
}

fn check_jianying_match(
    state: &GameStateCore,
    _catalog: &Catalog,
    owner: PlayerId,
    fid: EventId,
    suit: Option<Suit>,
    rank: u8,
    is_equip: bool,
) -> bool {
    let pkey = if is_equip {
        format!("l11_prev_eq_play:{fid}")
    } else {
        format!("l11_prev_card_play:{fid}")
    };
    let prev = flag(state, owner, &pkey);
    let prev_suit = (prev >> 8) & 0xff;
    let prev_rank = prev & 0xff;
    let cur_suit = match suit {
        Some(Suit::Spade) => 1u32,
        Some(Suit::Heart) => 2u32,
        Some(Suit::Club) => 3u32,
        Some(Suit::Diamond) => 4u32,
        None => 0u32,
    };
    let cur_rank = u32::from(rank);
    (prev_suit != 0 && cur_suit != 0 && prev_suit == cur_suit)
        || (prev_rank != 0 && cur_rank != 0 && prev_rank == cur_rank)
}

pub(in crate::content::host) fn machine(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("refankui", EventData::Damage(d)) => {
            target = d.source;
            card = Some(d.amount.max(1) as u32);
        }
        ("kuangfu" | "qiaomeng", EventData::Damage(d)) => {
            target = Some(d.target);
        }
        ("langmie", EventData::Phase { player, .. }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("langmie", EventData::TurnEndWindow { player, .. }) => {
            card = Some(2);
            target = Some(*player);
        }
        ("bihuo", EventData::DyingSaved { player, .. }) => {
            target = Some(*player);
        }
        ("guzheng", EventData::Phase { player, .. }) => {
            target = Some(*player);
        }
        ("kaikang", EventData::CardUsed(u)) => {
            targets = u.targets.clone();
        }
        ("manji", EventData::LoseHp { player, .. }) => {
            target = Some(*player);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11VmFixA,
        owner,
        target,
        card,
        cards: Vec::new(),
        targets,
        stage: HostStage::Act,
    }
}
