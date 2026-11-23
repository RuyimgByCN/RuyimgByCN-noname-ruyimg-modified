//! U05-7d 杀海拾遗 swd 人物技能牌/判定区修正（`swd_u05_7d_fixes_cards.rs`）。

use super::super::*;
use super::swd_u05_7d::{discard_owned_card_once, lexue_apply_pool, refund_once_used};
use super::swd_u05_7d_fixes::he_cards;
use std::collections::BTreeSet;

pub(in crate::content::host) fn xuanzhou_delay_tags(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    target: PlayerId,
) -> Vec<u8> {
    let existing: Vec<CardKind> = core
        .player(target)
        .judge
        .iter()
        .map(|&j| core.declared_kinds.get(&j).copied().unwrap_or_else(|| catalog.card_kind(j)))
        .collect();
    let mut tags = Vec::new();
    if !existing.contains(&CardKind::Bing) {
        tags.push(1);
    }
    if !existing.contains(&CardKind::Le) {
        tags.push(2);
    }
    if !existing.contains(&CardKind::ShanDian) {
        tags.push(3);
    }
    if !existing.contains(&CardKind::Guiyoujie) {
        tags.push(4);
    }
    if !existing.contains(&CardKind::HongShui) {
        tags.push(5);
    }
    tags
}

fn delay_kind_from_tag(tag: u8) -> Option<CardKind> {
    match tag {
        1 => Some(CardKind::Bing),
        2 => Some(CardKind::Le),
        3 => Some(CardKind::ShanDian),
        4 => Some(CardKind::Guiyoujie),
        5 => Some(CardKind::HongShui),
        _ => None,
    }
}

pub(super) fn step_swd_u05_7d_cards(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::SwdShehun => match m.stage {
            HostStage::Act => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(m.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "shehun_discard",
                        options,
                        min: 1,
                        max: 4,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let mut seen_suits = BTreeSet::new();
                let mut valid = true;
                for &c in &picked {
                    let s = ctx.catalog.effective_card_suit(ctx.core, m.owner, c) as u8;
                    if !ctx.core.player(m.owner).hand.contains(&c) || !seen_suits.insert(s) {
                        valid = false;
                        break;
                    }
                }
                if !valid {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(m.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    return Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "shehun_discard",
                            options,
                            min: 1,
                            max: 4,
                            allow_cancel: true,
                        },
                    };
                }
                m.cards = picked;
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != m.owner
                            && !ctx.catalog.has_undist(ctx.core, *p)
                            && (!ctx.core.player(*p).hand.is_empty()
                                || !ctx.core.player(*p).equips.is_empty())
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "shehun_target",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                m.target = Some(t);
                let he = he_cards(ctx.core, t);
                if he.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let take = m.cards.len().min(he.len());
                m.stage = HostStage::CardsAsk;
                Flow::WaitChoice {
                    observer: t,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "shehun_target_discard",
                        options: he.into_iter().map(ChoiceOption::Card).collect(),
                        min: take,
                        max: take,
                        allow_cancel: false,
                    },
                }
            }
            HostStage::CardsAsk => {
                let t_discard: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                let own = std::mem::take(&mut m.cards);
                let own_suits: Vec<crate::catalog::Suit> = own
                    .iter()
                    .map(|c| ctx.catalog.effective_card_suit(ctx.core, m.owner, *c))
                    .collect();
                for c in own {
                    discard_owned_card_once(ctx, m.owner, c);
                }
                let mut damage = false;
                for c in t_discard {
                    let s = ctx.catalog.effective_card_suit(ctx.core, t, c);
                    if own_suits.contains(&s) {
                        damage = true;
                    }
                    discard_owned_card_once(ctx, t, c);
                }
                if damage && ctx.core.player(t).alive {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(m.owner),
                        t,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdXuanzhou => match m.stage {
            HostStage::Act => {
                let options: Vec<ChoiceOption> = he_cards(ctx.core, m.owner)
                    .into_iter()
                    .filter(|c| ctx.catalog.card_kind(*c).is_ordinary_trick_kind())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "xuanzhou_trick",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                }) else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                if !he_cards(ctx.core, m.owner).contains(&picked) {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.card = Some(picked);
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != m.owner && !xuanzhou_delay_tags(ctx.core, ctx.catalog, *p).is_empty()
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "xuanzhou_target",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                let tags = xuanzhou_delay_tags(ctx.core, ctx.catalog, t);
                if tags.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.target = Some(t);
                m.stage = HostStage::SelectNature;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: "xuanzhou_delay_kind",
                        options: tags.into_iter().map(ChoiceOption::Tag).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectNature => {
                let Some(tag) = ctx.answer().iter().find_map(|o| match o {
                    ChoiceOption::Tag(v) => Some(*v),
                    _ => None,
                }) else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                let (Some(c), Some(t), Some(kind)) =
                    (m.card.take(), m.target, delay_kind_from_tag(tag))
                else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                let origin = if ctx.core.player(m.owner).hand.contains(&c) {
                    ctx.take_hand_card(m.owner, c);
                    crate::event::LostOrigin::Hand
                } else if ctx.core.player(m.owner).equips.contains(&c) {
                    ctx.core.player_mut(m.owner).equips.retain(|x| *x != c);
                    ctx.core.declared_slots.remove(&c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, m.owner, c);
                    crate::event::LostOrigin::Equip
                } else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                ctx.note_cards_lost(m.owner, &[c]);
                ctx.emit(EventData::CardsLost {
                    player: m.owner,
                    cards: vec![c],
                    origins: vec![origin],
                });
                ctx.core.declared_kinds.insert(c, kind);
                ctx.core.player_mut(t).judge.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: m.owner,
                    to: t,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdLexue => {
            lexue_apply_pool(ctx, m.owner, &["制衡", "集智", "缔盟", "驱虎"]);
            Flow::Done
        }
        HostBehavior::SwdPozhen => match m.stage {
            HostStage::Act => {
                let Some(d) = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .map(|f| &f.data)
                    .and_then(|data| match data {
                        EventData::Damage(d) => Some((d.target, d.source)),
                        _ => None,
                    })
                else {
                    return Flow::Done;
                };
                let Some(src) = d.1 else {
                    return Flow::Done;
                };
                m.target = Some(src);
                let my = ctx.core.player(m.owner).hand.len() as i32;
                let their = ctx.core.player(src).hand.len() as i32;
                let diff = my - their;
                if diff > 0 {
                    let n = diff as usize;
                    let hand: Vec<ChoiceOption> = ctx
                        .core
                        .player(m.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if hand.len() < n {
                        return Flow::Done;
                    }
                    m.cards = vec![0; n];
                    m.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "pozhen_discard",
                            options: hand,
                            min: n,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                } else if diff < 0 {
                    let n = (-diff) as usize;
                    if ctx.core.player(src).hand.len() < n {
                        return Flow::Done;
                    }
                    m.cards = vec![0; n];
                    m.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: m.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "pozhen_discard_source",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                } else {
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let n = m.cards.len();
                if picks.len() != n {
                    return Flow::Done;
                }
                for c in picks {
                    if ctx.core.player(m.owner).hand.contains(&c) {
                        ctx.discard_from_hand(m.owner, c);
                    }
                }
                if let Some(t) = m.target {
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(m.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                }
                m.cards.clear();
                m.target = None;
                Flow::Done
            }
            HostStage::TargetChoice => {
                let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
                let n = m.cards.len();
                if confirmed {
                    if let Some(t) = m.target {
                        if ctx.core.player(t).alive {
                            let mut hand: Vec<CardId> =
                                ctx.core.player(t).hand.iter().copied().collect();
                            let take = n.min(hand.len());
                            for _ in 0..take {
                                let j = ctx.core.rng.gen_range(hand.len() as u32) as usize;
                                let c = hand.swap_remove(j);
                                ctx.discard_from_hand(t, c);
                            }
                        }
                    }
                }
                m.cards.clear();
                m.target = None;
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdGuiyan2 => {
            ctx.recover(m.owner, 1, None);
            for name in ["guiyan", "guiyan2", "swd_guiyan2"] {
                if let Some(d) = ctx.catalog.skill_by_name(name) {
                    ctx.core.skills.remove(&(m.owner, d));
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
