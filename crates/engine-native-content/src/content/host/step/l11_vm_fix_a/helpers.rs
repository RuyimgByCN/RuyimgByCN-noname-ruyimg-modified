use super::super::super::*;
use super::super::l11_common::set_flag;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::GameStateCore;

pub(super) const ASK_KEY: &str = "l11_vm_a_ask_id";

pub(super) const ASK_MIJI_CARDS: u32 = 1;
pub(super) const ASK_MIJI_TARGET: u32 = 2;
pub(super) const ASK_FUJI_CONFIRM: u32 = 3;
pub(super) const ASK_WURONG_TARGET: u32 = 4;
pub(super) const ASK_WURONG_CARD: u32 = 5;
pub(super) const ASK_WURONG_GAIN: u32 = 6;
pub(super) const ASK_REFANKUI_GAIN: u32 = 7;
pub(super) const ASK_YEYAN_MODE: u32 = 8;
pub(super) const ASK_YEYAN_CARD: u32 = 9;
pub(super) const ASK_YEYAN_SMALL_TARGETS: u32 = 10;
pub(super) const ASK_YEYAN_GREAT_TARGETS: u32 = 11;
pub(super) const ASK_YEYAN_2DMG_TARGET: u32 = 12;
pub(super) const ASK_YEYAN_DMG_AMOUNT: u32 = 13;
pub(super) const ASK_FENWEI_TARGETS: u32 = 14;
pub(super) const ASK_KUANGFU_EQUIP: u32 = 15;
pub(super) const ASK_KUANGFU_EQUIP_SELF: u32 = 16;
pub(super) const ASK_LANGMIE_PLAY_DISCARD: u32 = 17;
pub(super) const ASK_LANGMIE_END_DISCARD: u32 = 18;
pub(super) const ASK_YINGJIAN_TARGET: u32 = 19;
pub(super) const ASK_QIAOMENG_DISCARD: u32 = 20;
pub(super) const ASK_SHENXIAN_CONFIRM: u32 = 21;
pub(super) const ASK_LIANPO_CONFIRM: u32 = 22;
pub(super) const ASK_BIHUO_CONFIRM: u32 = 23;
pub(super) const ASK_CHUNLAO_STORE: u32 = 24;
pub(super) const ASK_CHUNLAO_SPEND: u32 = 25;
pub(super) const ASK_XIONGLUAN_TARGET: u32 = 26;
pub(super) const ASK_FENMING_CONFIRM: u32 = 27;
pub(super) const ASK_FENMING_CARD: u32 = 28;
pub(super) const ASK_FENYIN_CONFIRM: u32 = 29;
pub(super) const ASK_GUZHENG_CONFIRM: u32 = 30;
pub(super) const ASK_GUZHENG_KEEP: u32 = 31;
pub(super) const ASK_GUZHENG_GAIN: u32 = 32;
pub(super) const ASK_GZYINGHUN_TARGET: u32 = 33;
pub(super) const ASK_GZYINGHUN_MODE: u32 = 34;
pub(super) const ASK_GZYINGHUN_DISCARD: u32 = 35;
pub(super) const ASK_JIANYING_CONFIRM: u32 = 36;
pub(super) const ASK_KAIKANG_CONFIRM: u32 = 37;
pub(super) const ASK_KAIKANG_GIVE: u32 = 38;
pub(super) const ASK_KAIKANG_USE_EQUIP: u32 = 39;
pub(super) const ASK_KUNFEN_CONFIRM: u32 = 40;
pub(super) const ASK_LIHUO_CONVERT: u32 = 41;
pub(super) const ASK_LIHUO_EXTRA: u32 = 42;
pub(super) const ASK_MINGCE_CARD: u32 = 43;
pub(super) const ASK_MINGCE_GIVE_TARGET: u32 = 44;
pub(super) const ASK_MINGCE_SHA_TARGET: u32 = 45;
pub(super) const ASK_MINGCE_CHOICE: u32 = 46;
pub(super) const ASK_NSYANGWU_TARGET: u32 = 47;
pub(super) const ASK_NSYANGWU_CARD: u32 = 48;
pub(super) const ASK_NSYANGWU_GAIN: u32 = 49;

pub(super) fn owner_unblocked(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    skill_name: &str,
    locked: bool,
) -> bool {
    let Some(def) = catalog.skill_by_name(skill_name) else {
        return false;
    };
    if !state.player(owner).alive
        || !state.has_skill(owner, def)
        || catalog.is_skill_chanyuan_blocked(state, owner, def)
        || super::super::l11_common::has_named_skill(state, catalog, owner, "txhj_feilong_skill_blank")
    {
        return false;
    }
    locked
        || (!catalog.is_tieji_sealed(state, owner) && !catalog.is_xj_fengyin_sealed(state, owner))
}

pub(super) fn has_l11_vm_a_skill(catalog: &Catalog, name: &str) -> bool {
    catalog.skill_by_name(name).is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior() == Some(HostBehavior::L11VmFixA)
    })
}

pub(super) fn mark_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, skill_name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(skill_name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = true;
        }
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = super::asks::build_ask(
        ask_id,
        next_stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = next_stage;
        return Flow::WaitChoice { observer, spec };
    }
    set_flag(ctx.core, m.owner, ASK_KEY, 0);
    Flow::Done
}

pub(super) fn clear_ask(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, ASK_KEY, 0);
}

pub(super) fn is_confirmed(ctx: &Ctx<'_>) -> bool {
    matches!(
        ctx.answer().first(),
        Some(ChoiceOption::Confirm) | Some(ChoiceOption::Tag(1))
    )
}

pub(super) fn answered_cards(ctx: &Ctx<'_>) -> Vec<CardId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_players(ctx: &Ctx<'_>) -> Vec<PlayerId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_tag(ctx: &Ctx<'_>) -> Option<u32> {
    ctx.answer().iter().find_map(|o| match o {
        ChoiceOption::Tag(t) => Some(u32::from(*t)),
        _ => None,
    })
}

pub(super) fn suit_idx(suit: Suit) -> usize {
    match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

pub(super) fn can_virtual_sha_target(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    ignore_distance: bool,
) -> bool {
    if target == source || !state.player(target).alive {
        return false;
    }
    if catalog.has_duijue_out(state, target) {
        return false;
    }
    let kongcheng = catalog
        .skill_by_name(crate::content::standard::KONG_CHENG)
        .is_some_and(|d| state.has_skill(target, d))
        && state.player(target).hand.is_empty();
    if kongcheng {
        return false;
    }
    if !ignore_distance
        && !state.in_attack_range(catalog, source, target)
        && !crate::rvm_bridge::modifiers::tanbei_unlimited_target(state, catalog, source, target)
    {
        return false;
    }
    !crate::rvm_bridge::modifiers::eval_card_target_canceled_opt(
        state,
        catalog,
        source,
        target,
        None,
        CardKind::Sha,
        DamageNature::Normal,
    )
}

pub(super) fn is_multitarget_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Jiedao
            | CardKind::Tianfuzhen
            | CardKind::Qixingzhen
            | CardKind::Niaoxiangzhen
            | CardKind::Dizaizhen
            | CardKind::Changshezhen
    )
}

pub(super) fn is_delay_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::Guiyoujie
    )
}

pub(super) fn fenwei_effective_targets(state: &GameStateCore, catalog: &Catalog, u: &CardUseData) -> Vec<PlayerId> {
    let eff = u.effective_kind_in(state, catalog);
    match eff {
        CardKind::Wugu | CardKind::Taoyuan if u.targets.is_empty() => state.alive_players().collect(),
        _ => u.targets.clone(),
    }
}

#[allow(dead_code)]
pub(super) fn active_card_use_data(frames: &[crate::event::Frame]) -> Option<(EventId, CardUseData)> {
    frames.iter().rev().find_map(|f| match &f.data {
        EventData::CardUsed(u) => Some((f.id, u.clone())),
        _ => None,
    })
}

fn is_sha_damage_with(
    frames: &[crate::event::Frame],
    d: &crate::event::DamageData,
    card_kind_in: impl Fn(CardId) -> CardKind,
) -> bool {
    if let Some(c) = d.card {
        return card_kind_in(c).is_sha_like();
    }
    frames.iter().rev().any(|f| {
        matches!(
            &f.data,
            EventData::CardUsed(u)
                if Some(u.player) == d.source
                    && u.use_as
                        .or_else(|| u.card.map(&card_kind_in))
                        .is_some_and(|k| k.is_sha_like())
        )
    })
}

pub(super) fn is_sha_damage_in_stack(
    state: &GameStateCore,
    catalog: &Catalog,
    frames: &[crate::event::Frame],
    d: &crate::event::DamageData,
) -> bool {
    is_sha_damage_with(frames, d, |c| catalog.resolved_card_kind(state, c))
}

pub(super) fn is_black_sha_damage_in_stack(
    state: &GameStateCore,
    catalog: &Catalog,
    frames: &[crate::event::Frame],
    d: &crate::event::DamageData,
) -> bool {
    if !is_sha_damage_in_stack(state, catalog, frames, d) {
        return false;
    }
    if let Some(c) = d.card {
        let s = catalog.effective_card_suit(state, d.source.unwrap_or(d.target), c);
        return matches!(s, Suit::Spade | Suit::Club);
    }
    frames.iter().rev().find_map(|f| match &f.data {
        EventData::CardUsed(u) if Some(u.player) == d.source && u.effective_kind_in(state, catalog).is_sha_like() => {
            crate::kernel::card_use_suit(state, catalog, u)
                .map(|s| matches!(s, Suit::Spade | Suit::Club))
        }
        _ => None,
    }).unwrap_or(false)
}

pub(super) fn equip_stolen_card(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    card: CardId,
) {
    let Some(slot) = ctx.catalog.card_slot_of(ctx.core, card) else {
        return;
    };
    ctx.core.player_mut(from).equips.retain(|&c| c != card);
    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, card);
    ctx.note_cards_lost(from, &[card]);
    ctx.emit(EventData::CardsLost {
        player: from,
        cards: vec![card],
        origins: vec![crate::event::LostOrigin::Equip],
    });
    let old: Vec<CardId> = ctx
        .core
        .player(to)
        .equips
        .iter()
        .copied()
        .filter(|&eq| ctx.catalog.card_slot_of(ctx.core, eq) == Some(slot))
        .collect();
    for eq in &old {
        ctx.core.player_mut(to).equips.retain(|&c| c != *eq);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, to, *eq);
        ctx.to_discard(*eq);
        ctx.note_cards_lost(to, &[*eq]);
        ctx.emit(EventData::CardsLost {
            player: to,
            cards: vec![*eq],
            origins: vec![crate::event::LostOrigin::Equip],
        });
    }
    if !old.is_empty() {
        ctx.out.push(LogEvent::CardDiscarded {
            player: to,
            cards: old.clone(),
        });
        ctx.emit(EventData::CardsDiscarded {
            player: to,
            cards: old.clone(),
        });
    }
    ctx.core.player_mut(to).equips.push(card);
    crate::kernel::grant_equip_skills(ctx, to, card);
    ctx.log(LogEvent::Equipped {
        player: to,
        card,
        replaced: old.first().copied(),
    });
}

fn read_guzheng_flag_cards(
    core: &GameStateCore,
    target: PlayerId,
    key_prefix: &str,
) -> Vec<CardId> {
    let len = core
        .vm_aux
        .flags
        .get(&(target, format!("{key_prefix}_len")))
        .copied()
        .unwrap_or(0);
    let mut out = Vec::new();
    for i in 0..len {
        if let Some(&c) = core.vm_aux.flags.get(&(target, format!("{key_prefix}:{i}"))) {
            if core.discard.contains(&c) && !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

pub(super) fn guzheng_discarded_lists(
    core: &GameStateCore,
    target: PlayerId,
) -> (Vec<CardId>, Vec<CardId>) {
    let all = read_guzheng_flag_cards(core, target, "l11_gz_all");
    let own = read_guzheng_flag_cards(core, target, "l11_gz_own")
        .into_iter()
        .filter(|c| all.contains(c))
        .collect();
    (all, own)
}

pub(super) fn is_mingce_give_candidate(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cand: PlayerId,
) -> bool {
    cand != owner
        && state.player(cand).alive
        && state
            .alive_players()
            .any(|p| p != cand && state.in_attack_range(catalog, cand, p))
}

pub(super) fn mingce_eligible_cards(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<CardId> {
    let pl = state.player(owner);
    let mut out: Vec<CardId> = pl
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let k = catalog.resolved_card_kind(state, c);
            k.is_sha_like() || k.equip_slot().is_some()
        })
        .collect();
    for &c in &pl.equips {
        if catalog.resolved_card_kind(state, c).equip_slot().is_some() && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

pub(super) fn give_single_discard_card_from_discard(
    ctx: &mut Ctx<'_>,
    to: PlayerId,
    card: CardId,
) -> bool {
    if !ctx.core.player(to).alive || !ctx.core.discard.contains(&card) {
        return false;
    }
    ctx.core.discard.retain(|&c| c != card);
    ctx.core.player_mut(to).hand.push(card);
    ctx.out.push(LogEvent::CardsDrawn {
        player: to,
        cards: vec![card],
    });
    ctx.emit(EventData::CardsDrawn {
        player: to,
        cards: vec![card],
    });
    true
}

pub(super) fn has_four_distinct_suits_in_hand(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    let mut mask = 0u8;
    for &c in &state.player(owner).hand {
        let s = catalog.effective_card_suit(state, owner, c);
        mask |= 1u8 << suit_idx(s);
    }
    mask == 0b1111
}

pub(super) fn give_he_cards_to_hand(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    cards: &[CardId],
) {
    let mut moved = Vec::new();
    for &c in cards {
        let owns =
            ctx.core.player(from).hand.contains(&c) || ctx.core.player(from).equips.contains(&c);
        if owns && ctx.core.player(to).alive {
            ctx.take_owned_card(from, c);
            ctx.core.declared_kinds.remove(&c);
            ctx.core.declared_slots.remove(&c);
            ctx.core.player_mut(to).hand.push(c);
            moved.push(c);
        }
    }
    if !moved.is_empty() {
        ctx.out.push(LogEvent::CardsDrawn {
            player: to,
            cards: moved.clone(),
        });
        ctx.emit(EventData::CardsDrawn {
            player: to,
            cards: moved,
        });
    }
}

pub(super) fn discard_judge_cards(ctx: &mut Ctx<'_>, player: PlayerId) {
    let cards: Vec<CardId> = ctx.core.player(player).judge.clone();
    if cards.is_empty() {
        return;
    }
    for &c in &cards {
        ctx.take_owned_card(player, c);
        ctx.to_discard(c);
        set_flag(ctx.core, player, &format!("l11_discard_non_hand:{c}"), 1);
        set_flag(ctx.core, player, &format!("l11_discard_judge:{c}"), 1);
    }
    ctx.out.push(LogEvent::CardDiscarded {
        player,
        cards: cards.clone(),
    });
    ctx.note_cards_lost(player, &cards);
    let origins = vec![crate::event::LostOrigin::Judge; cards.len()];
    ctx.emit(EventData::CardsLost {
        player,
        cards: cards.clone(),
        origins,
    });
    ctx.emit(EventData::CardsDiscarded { player, cards });
}

pub(super) fn sort_by_seat_from(players: &mut Vec<PlayerId>, origin: PlayerId, total: usize) {
    if total == 0 {
        return;
    }
    players.sort_by_key(|&p| (usize::from(p) + total - usize::from(origin)) % total);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::standard::standard_catalog;
    use crate::event::{CardUseData, DamageNature, EventData, EventKind, Frame, Stage};
    use crate::machine::damage_data_with;

    #[test]
    fn is_sha_damage_in_stack_checks_explicit_non_sha_card_first() {
        let cat = standard_catalog();
        let sha = cat.cards.iter().position(|c| c.kind == CardKind::Sha).unwrap() as CardId;
        let trick = cat
            .cards
            .iter()
            .position(|c| c.kind == CardKind::Dismantle)
            .unwrap() as CardId;
        let frames = vec![Frame {
            id: 1,
            parent: None,
            kind: EventKind::CardUsed,
            stage: Stage::Intrinsic,
            data: EventData::CardUsed(CardUseData {
                card: Some(sha),
                player: 0,
                targets: vec![1],
                nature: DamageNature::Normal,
                use_as: None,
                cancelled: false,
            }),
            queue: Vec::new(),
            planned: false,
            intrinsic_done: true,
        }];
        let mut dmg = damage_data_with(Some(0), 1, 1, DamageNature::Normal, Some(trick));
        assert!(!is_sha_damage_with(&frames, &dmg, |c| cat.card_kind(c)));
        dmg.card = None;
        assert!(is_sha_damage_with(&frames, &dmg, |c| cat.card_kind(c)));
        dmg.card = Some(sha);
        assert!(is_sha_damage_with(&[], &dmg, |c| cat.card_kind(c)));
    }
}
