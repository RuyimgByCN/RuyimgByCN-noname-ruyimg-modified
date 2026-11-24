//! Phase, Pindian, discard, and turn-boundary triggered skills for `HostBehavior::L11VmFixC`.

use super::helpers::*;
use super::*;
use crate::catalog::{CardKind, Suit};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn step_phase(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    match m.def_key.as_str() {
        "dujin" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_DUJIN_CONFIRM),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                log_skill(ctx, owner, "dujin");
                let add = 1 + (ctx.core.player(owner).equips.len() / 2) as i32;
                bump_draw_phase_pending(ctx, owner, add);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "gnjinfan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::CardsAsk, ASK_GNJINFAN_CARDS),
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if chosen.is_empty() {
                    return Flow::Done;
                }
                log_skill(ctx, owner, "gnjinfan");
                let moved = take_owned_cards_from_zones(ctx, owner, &chosen);
                if !moved.is_empty() {
                    let n = moved.len();
                    push_to_pile(ctx.core, owner, "gnjinfan", &moved);
                    let len = pile_cards(ctx.core, owner, "gnjinfan").len() as u32;
                    sync_mark_count(ctx.core, owner, "gnjinfan", len);
                    let origins = vec![crate::event::LostOrigin::Hand; n];
                    ctx.emit(EventData::CardsLost {
                        player: owner,
                        cards: moved,
                        origins,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "juece" => match m.stage {
            HostStage::Act => {
                let victim = m.target.unwrap_or(owner);
                if victim == owner
                    || !ctx.core.player(victim).alive
                    || !ctx.core.player(victim).hand.is_empty()
                {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_JUECE_CONFIRM)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let victim = m.target.unwrap_or(owner);
                if victim != owner && ctx.core.player(victim).alive {
                    log_skill(ctx, owner, "juece");
                    emit_damage(ctx, Some(owner), victim, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "juejing" => {
            log_skill(ctx, owner, "juejing");
            let p = ctx.core.player(owner);
            let add = ((p.max_hp as i32) - p.hp).max(0);
            if add > 0 {
                bump_draw_phase_pending(ctx, owner, add);
            }
            Flow::Done
        }
        "nsfeixiong" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_NSFEIXIONG_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                log_skill(ctx, owner, "nsfeixiong");
                start_pindian_pick(m, ctx, ASK_NSFEIXIONG_PINDIAN_OWNER, ASK_NSFEIXIONG_PINDIAN_TARGET)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c1);
                advance_pindian_target_pick(m, ctx, ASK_NSFEIXIONG_PINDIAN_TARGET)
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let c1 = m.card.unwrap_or(0);
                enter_pindian_yingyang(m, ctx, c1, c2)
            }
            HostStage::JiefenOwnerDiscard => on_pindian_yingyang_owner(m, ctx),
            HostStage::JiefenTargetDiscard => on_pindian_yingyang_target(m, ctx),
            _ => Flow::Done,
        },
        "nsyice" => match m.stage {
            HostStage::Act => {
                let in_disc: Vec<CardId> = m
                    .cards
                    .iter()
                    .copied()
                    .filter(|c| ctx.core.discard.contains(c))
                    .collect();
                if in_disc.is_empty() {
                    return Flow::Done;
                }
                m.cards = in_disc;
                log_skill(ctx, owner, "nsyice");
                if m.cards.len() == 1 {
                    let picked = m.cards.clone();
                    nsyice_push_and_check(m, ctx, picked)
                } else {
                    wait_ask(m, ctx, HostStage::CardsAsk, ASK_NSYICE_ORDER)
                }
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.is_empty() {
                    return Flow::Done;
                }
                nsyice_push_and_check(m, ctx, picked)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(top_card)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let mut bottom = None;
                for &c in &m.cards {
                    if c != top_card {
                        bottom = Some(c);
                    }
                }
                if let Some(bc) = bottom {
                    ctx.core.deck.insert(0, bc);
                }
                ctx.core.deck.push(top_card);
                let anyone_dying = ctx.core.players.iter().any(|p| p.alive && p.dying);
                if anyone_dying {
                    Flow::Done
                } else {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_NSYICE_DAMAGE_TARGET)
                }
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive {
                        emit_damage(ctx, None, t, 1, DamageNature::Normal, None);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "nzry_jieying" => match m.stage {
            HostStage::Act => {
                if m.card.is_none() {
                    // TurnBegin initial link setup
                    if !ctx.core.player(owner).linked {
                        set_flag(ctx.core, owner, "l11c_jieying_init", 1);
                        if crate::rvm_bridge::modifiers::can_gain_link(ctx.core, ctx.catalog, owner)
                        {
                            ctx.core.player_mut(owner).linked = true;
                            ctx.log(LogEvent::Linked {
                                player: owner,
                                linked: true,
                            });
                        }
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_NZRY_JIEYING_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && !ctx.core.player(t).linked {
                        log_skill(ctx, owner, "nzry_jieying");
                        if crate::rvm_bridge::modifiers::can_gain_link(ctx.core, ctx.catalog, t) {
                            ctx.core.player_mut(t).linked = true;
                            ctx.log(LogEvent::Linked {
                                player: t,
                                linked: true,
                            });
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qiaoshui" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIAOSHUI_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                log_skill(ctx, owner, "qiaoshui");
                start_pindian_pick(m, ctx, ASK_QIAOSHUI_PINDIAN_OWNER, ASK_QIAOSHUI_PINDIAN_TARGET)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c1);
                advance_pindian_target_pick(m, ctx, ASK_QIAOSHUI_PINDIAN_TARGET)
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let c1 = m.card.unwrap_or(0);
                enter_pindian_yingyang(m, ctx, c1, c2)
            }
            HostStage::JiefenOwnerDiscard => on_pindian_yingyang_owner(m, ctx),
            HostStage::JiefenTargetDiscard => on_pindian_yingyang_target(m, ctx),
            _ => Flow::Done,
        },
        "ranshang2" => match m.stage {
            HostStage::Act => {
                let x = flag(ctx.core, owner, "l11c_ranshang_marks");
                if x > 0 {
                    log_skill(ctx, owner, "ranshang2");
                    m.cards = vec![x];
                    m.stage = HostStage::DrawSettle;
                    emit_lose_hp(ctx, owner, x);
                    return Flow::Yield;
                }
                Flow::Done
            }
            HostStage::DrawSettle => {
                let x = m.cards.first().copied().unwrap_or(0);
                if x > 2 && ctx.core.player(owner).alive {
                    let dec = 2u32.min(ctx.core.player(owner).max_hp as u32);
                    if dec > 0 {
                        ctx.lose_max_hp(owner, dec);
                    }
                    if ctx.core.player(owner).alive {
                        ctx.start_draw(owner, 2);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "redangxian" => {
            log_skill(ctx, owner, "redangxian");
            if let Some(pos) = ctx
                .core
                .discard
                .iter()
                .rposition(|&c| ctx.catalog.card_kind(c).is_sha_like())
            {
                let c = ctx.core.discard.remove(pos);
                ctx.core.player_mut(owner).hand.push(c);
                ctx.log(LogEvent::CardsDrawn {
                    player: owner,
                    cards: vec![c],
                });
            }
            ctx.core.turn.extra_phases.push(2);
            ctx.core.turn.extra_phases.push(0);
            Flow::Done
        }
        "rekuangcai" => {
            let phase_code = m.card.unwrap_or(0);
            if phase_code == 5 {
                // Phase::Discard begin
                let used = flag(ctx.core, owner, "l11c_turn_card_used_stamp") == tstamp;
                let dealt = flag(ctx.core, owner, "l11c_turn_source_dmg_stamp") == tstamp
                    && flag(ctx.core, owner, "l11c_turn_source_dmg_amt") > 0;
                if !used || !dealt {
                    log_skill(ctx, owner, "rekuangcai");
                    if !used {
                        let p = flag(ctx.core, owner, "l11c_rekuangcai_pos") + 1;
                        set_flag(ctx.core, owner, "l11c_rekuangcai_pos", p);
                    } else {
                        let n = flag(ctx.core, owner, "l11c_rekuangcai_neg") + 1;
                        set_flag(ctx.core, owner, "l11c_rekuangcai_neg", n);
                    }
                }
            } else if phase_code == 6 {
                // Phase::End begin (`rekuangcai_draw`)
                let dealt = if flag(ctx.core, owner, "l11c_turn_source_dmg_stamp") == tstamp {
                    flag(ctx.core, owner, "l11c_turn_source_dmg_amt")
                } else {
                    0
                };
                if dealt > 0 {
                    log_skill(ctx, owner, "rekuangcai");
                    ctx.start_draw(owner, dealt.min(5));
                }
            }
            Flow::Done
        }
        "reshanxi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_RESHANXI_COST),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_RESHANXI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if let Some(cost_card) = m.card {
                    log_skill(ctx, owner, "reshanxi");
                    discard_player_cards(ctx, owner, &[cost_card]);
                }
                m.target = Some(t);
                if !ctx.core.player(t).alive || he_cards(ctx.core, t).is_empty() || ctx.core.player(owner).hp <= 0 {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_RESHANXI_EXILE)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if chosen.is_empty() || !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let taken = take_owned_cards_from_zones(ctx, t, &chosen);
                if !taken.is_empty() {
                    let n = taken.len();
                    push_to_pile(ctx.core, t, "reshanxi2", &taken);
                    let total = pile_cards(ctx.core, t, "reshanxi2").len() as u32;
                    sync_mark_count(ctx.core, t, "reshanxi2", total);
                    ensure_perm_skill(ctx, t, "reshanxi2");
                    let origins = vec![crate::event::LostOrigin::Hand; n];
                    ctx.emit(EventData::CardsLost {
                        player: t,
                        cards: taken,
                        origins,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "reshanxi2" => {
            let exiled = pile_cards(ctx.core, owner, "reshanxi2");
            if !exiled.is_empty() {
                let taken = take_from_pile(ctx.core, owner, "reshanxi2", &exiled);
                sync_mark_count(ctx.core, owner, "reshanxi2", 0);
                if !taken.is_empty() && ctx.core.player(owner).alive {
                    ctx.core.player_mut(owner).hand.extend(taken.iter().copied());
                    ctx.log(LogEvent::CardsDrawn {
                        player: owner,
                        cards: taken,
                    });
                }
            }
            remove_named_skill(ctx, owner, "reshanxi2");
            Flow::Done
        }
        "shelie" => match m.stage {
            HostStage::Act => {
                let Some(f) = ctx
                    .res
                    .stack
                    .iter_mut()
                    .rev()
                    .find(|f| matches!(f.data, EventData::DrawPhase { .. }))
                else {
                    return Flow::Done;
                };
                let EventData::DrawPhase {
                    player,
                    ref mut pending,
                    ref mut num_fixed,
                    ..
                } = f.data
                else {
                    return Flow::Done;
                };
                if player != owner || *num_fixed {
                    return Flow::Done;
                }
                *pending = 0;
                *num_fixed = true;
                log_skill(ctx, owner, "shelie");
                let mut top5 = Vec::new();
                for _ in 0..5 {
                    if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
                        ctx.reshuffle_discard_into_deck();
                    }
                    if let Some(c) = ctx.core.deck.pop() {
                        top5.push(c);
                    }
                }
                if top5.is_empty() {
                    return Flow::Done;
                }
                m.cards = top5;
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_SHELIE_PICK)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let mut gained = Vec::new();
                let mut disc = Vec::new();
                for &c in &m.cards {
                    if picked.contains(&c) && ctx.core.player(owner).alive {
                        gained.push(c);
                        ctx.core.player_mut(owner).hand.push(c);
                    } else {
                        disc.push(c);
                        ctx.to_discard(c);
                    }
                }
                if !gained.is_empty() {
                    ctx.log(LogEvent::CardsDrawn {
                        player: owner,
                        cards: gained,
                    });
                }
                if !disc.is_empty() {
                    ctx.log(LogEvent::CardDiscarded {
                        player: owner,
                        cards: disc,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "twkujian_discard" => match m.stage {
            HostStage::Act => {
                let recipient = m.target.unwrap_or(owner);
                let mut seq = Vec::new();
                for p in ctx.core.seat_order_from(ctx.core.turn.current) {
                    if (p == owner || p == recipient)
                        && ctx.core.player(p).alive
                        && !he_cards(ctx.core, p).is_empty()
                    {
                        seq.push(p);
                    }
                }
                if seq.is_empty() {
                    return Flow::Done;
                }
                log_skill(ctx, owner, "twkujian");
                m.targets = seq;
                let first = m.targets.remove(0);
                m.target = Some(first);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWKUJIAN_DISCARD)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let who = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, who, &[c]);
                }
                while let Some(next) = (!m.targets.is_empty()).then(|| m.targets.remove(0)) {
                    if ctx.core.player(next).alive && !he_cards(ctx.core, next).is_empty() {
                        m.target = Some(next);
                        return wait_ask(m, ctx, HostStage::TargetChoice, ASK_TWKUJIAN_DISCARD);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinniluan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XINNILUAN_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t = m.target.unwrap_or(owner);
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                log_skill(ctx, owner, "xinniluan");
                set_flag(ctx.core, owner, "l11c_xinniluan_dist_unlimited", 1);
                let dealt_before = if flag(ctx.core, owner, "l11c_turn_source_dmg_stamp") == tstamp {
                    flag(ctx.core, owner, "l11c_turn_source_dmg_amt")
                } else {
                    0
                };
                m.cards = vec![dealt_before];
                m.stage = HostStage::DrawSettle;
                emit_sha_card_use(ctx, owner, Some(c), vec![t], CardKind::Sha, true);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                set_flag(ctx.core, owner, "l11c_xinniluan_dist_unlimited", 0);
                let t = m.target.unwrap_or(owner);
                let dealt_before = m.cards.first().copied().unwrap_or(0);
                let dealt_now = if flag(ctx.core, owner, "l11c_turn_source_dmg_stamp") == tstamp {
                    flag(ctx.core, owner, "l11c_turn_source_dmg_amt")
                } else {
                    0
                };
                if dealt_now > dealt_before
                    && ctx.core.player(t).alive
                    && !he_cards(ctx.core, t).is_empty()
                {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_XINNILUAN_DISCARD)
                } else {
                    Flow::Done
                }
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, t, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhuikong" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUIKONG_CONFIRM),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                log_skill(ctx, owner, "zhuikong");
                start_pindian_pick(m, ctx, ASK_ZHUIKONG_PINDIAN_OWNER, ASK_ZHUIKONG_PINDIAN_TARGET)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c1);
                advance_pindian_target_pick(m, ctx, ASK_ZHUIKONG_PINDIAN_TARGET)
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let c1 = m.card.unwrap_or(0);
                enter_pindian_yingyang(m, ctx, c1, c2)
            }
            HostStage::JiefenOwnerDiscard => on_pindian_yingyang_owner(m, ctx),
            HostStage::JiefenTargetDiscard => on_pindian_yingyang_target(m, ctx),
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn bump_draw_phase_pending(ctx: &mut Ctx<'_>, owner: PlayerId, delta: i32) {
    if let Some(f) = ctx
        .res
        .stack
        .iter_mut()
        .rev()
        .find(|f| matches!(f.data, EventData::DrawPhase { .. }))
    {
        if let EventData::DrawPhase {
            player,
            ref mut pending,
            num_fixed,
            ..
        } = f.data
        {
            if player == owner && !num_fixed {
                *pending += delta;
            }
        }
    }
}

fn nsyice_push_and_check(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    picked: Vec<CardId>,
) -> Flow {
    let owner = m.owner;
    let mut added = Vec::new();
    for c in picked {
        if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
            ctx.core.discard.remove(pos);
            added.push(c);
        }
    }
    if added.is_empty() {
        return Flow::Done;
    }
    push_to_pile(ctx.core, owner, "nsyice", &added);
    let mut storage = pile_cards(ctx.core, owner, "nsyice");
    sync_mark_count(ctx.core, owner, "nsyice", storage.len() as u32);
    let rank_of = |c: CardId| ctx.catalog.cards.get(c as usize).map(|d| d.rank).unwrap_or(0);
    let mut found = None;
    'outer: for i in 0..storage.len() {
        for j in ((i + 1)..storage.len()).rev() {
            if rank_of(storage[i]) == rank_of(storage[j]) {
                found = Some((i, j));
                break 'outer;
            }
        }
    }
    let Some((i, j)) = found else {
        return Flow::Done;
    };
    let span: Vec<CardId> = storage.drain(i..=j).collect();
    ctx.core
        .vm_aux
        .piles
        .insert((owner, "nsyice".to_string()), storage);
    let len_after = pile_cards(ctx.core, owner, "nsyice").len() as u32;
    sync_mark_count(ctx.core, owner, "nsyice", len_after);
    if span.len() < 2 {
        return Flow::Done;
    }
    let first = span[0];
    let last = span[span.len() - 1];
    let middle = &span[1..(span.len() - 1)];
    if !middle.is_empty() && ctx.core.player(owner).alive {
        ctx.core.player_mut(owner).hand.extend(middle.iter().copied());
        ctx.log(LogEvent::CardsDrawn {
            player: owner,
            cards: middle.to_vec(),
        });
    }
    m.cards = vec![first, last];
    wait_ask(m, ctx, HostStage::SelectCard, ASK_NSYICE_TOP)
}

#[allow(dead_code)]
fn _suit_check(_s: Suit) {}

fn start_pindian_pick(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner_ask: u32,
    target_ask: u32,
) -> Flow {
    let owner = m.owner;
    if ctx.core.player(owner).hand.len() == 1 {
        let c1 = ctx.core.player(owner).hand[0];
        m.card = Some(c1);
        advance_pindian_target_pick(m, ctx, target_ask)
    } else {
        wait_ask(m, ctx, HostStage::ComparePlayerPick, owner_ask)
    }
}

fn advance_pindian_target_pick(m: &mut HostMachine, ctx: &mut Ctx<'_>, target_ask: u32) -> Flow {
    let t = m.target.unwrap_or(m.owner);
    if ctx.core.player(t).hand.len() == 1 {
        let c1 = m.card.unwrap_or(0);
        let c2 = ctx.core.player(t).hand[0];
        enter_pindian_yingyang(m, ctx, c1, c2)
    } else {
        wait_ask(m, ctx, HostStage::CompareTargetPick, target_ask)
    }
}

fn enter_pindian_yingyang(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    c1: CardId,
    c2: CardId,
) -> Flow {
    let owner = m.owner;
    let t = m.target.unwrap_or(owner);
    let own_c1 = take_owned_cards_from_zones(ctx, owner, &[c1]);
    let own_c2 = take_owned_cards_from_zones(ctx, t, &[c2]);
    if !own_c1.is_empty() {
        ctx.to_discard(c1);
        ctx.out.push(LogEvent::CardDiscarded {
            player: owner,
            cards: own_c1,
        });
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![c1],
            origins: vec![crate::event::LostOrigin::Hand],
        });
    }
    if !own_c2.is_empty() {
        ctx.to_discard(c2);
        ctx.out.push(LogEvent::CardDiscarded {
            player: t,
            cards: own_c2,
        });
        ctx.emit(EventData::CardsLost {
            player: t,
            cards: vec![c2],
            origins: vec![crate::event::LostOrigin::Hand],
        });
    }
    m.cards = vec![c1, c2, 2, 2];
    if super::super::l11_common_b2::has_txyingyang_ready(ctx.core, ctx.catalog, owner) {
        return wait_ask(
            m,
            ctx,
            HostStage::JiefenOwnerDiscard,
            ASK_PINDIAN_YINGYANG_OWNER,
        );
    }
    if super::super::l11_common_b2::has_txyingyang_ready(ctx.core, ctx.catalog, t) {
        return wait_ask(
            m,
            ctx,
            HostStage::JiefenTargetDiscard,
            ASK_PINDIAN_YINGYANG_TARGET,
        );
    }
    finish_pindian_compare(m, ctx)
}

fn on_pindian_yingyang_owner(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    clear_ask(ctx, m.owner);
    let mode = match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(tag)) => u32::from(tag),
        _ => 2,
    };
    if m.cards.len() >= 4 {
        m.cards[2] = mode;
    }
    let t = m.target.unwrap_or(m.owner);
    if super::super::l11_common_b2::has_txyingyang_ready(ctx.core, ctx.catalog, t) {
        return wait_ask(
            m,
            ctx,
            HostStage::JiefenTargetDiscard,
            ASK_PINDIAN_YINGYANG_TARGET,
        );
    }
    finish_pindian_compare(m, ctx)
}

fn on_pindian_yingyang_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    clear_ask(ctx, m.owner);
    let mode = match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(tag)) => u32::from(tag),
        _ => 2,
    };
    if m.cards.len() >= 4 {
        m.cards[3] = mode;
    }
    finish_pindian_compare(m, ctx)
}

fn finish_pindian_compare(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let t = m.target.unwrap_or(owner);
    let tstamp = turn_stamp(ctx.core);
    let c1 = m.cards.first().copied().unwrap_or(0);
    let c2 = m.cards.get(1).copied().unwrap_or(0);
    let yy_owner = m.cards.get(2).copied().unwrap_or(2);
    let yy_target = m.cards.get(3).copied().unwrap_or(2);
    let r1 = card_effective_rank_with_mods(ctx, owner, c1, Some(yy_owner));
    let r2 = card_effective_rank_with_mods(ctx, t, c2, Some(yy_target));
    let won = r1 > r2;
    if won {
        set_flag(ctx.core, owner, "compare_win_turn", 1);
    } else if r2 > r1 {
        set_flag(ctx.core, t, "compare_win_turn", 1);
    }
    match m.def_key.as_str() {
        "qiaoshui" => {
            if won {
                set_flag(ctx.core, owner, "l11c_qiaoshui3_turn", tstamp);
                if let Some(def) = ctx.catalog.skill_by_name("qiaoshui3") {
                    ctx.add_skill_until_turn_end(owner, def);
                }
            } else {
                set_flag(ctx.core, owner, "l11c_qiaoshui2_turn", tstamp);
                if let Some(def) = ctx.catalog.skill_by_name("qiaoshui2") {
                    ctx.add_skill_until_turn_end(owner, def);
                }
            }
        }
        "nsfeixiong" => {
            if won && ctx.core.player(t).alive {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
            } else if r1 < r2 && ctx.core.player(owner).alive {
                let src = ctx.core.player(t).alive.then_some(t);
                emit_damage(ctx, src, owner, 1, DamageNature::Normal, None);
            }
        }
        "zhuikong" => {
            if won {
                if let Some(def) = ctx.catalog.skill_by_name("zishou2") {
                    ctx.add_skill_until_turn_end(t, def);
                }
                set_flag(ctx.core, t, "zishou2_turn_stamp", tstamp);
            } else {
                set_flag(ctx.core, t, "l11c_zhuikong_dist_turn", tstamp);
                set_flag(
                    ctx.core,
                    t,
                    "l11c_zhuikong_dist_target",
                    u32::from(owner) + 1,
                );
            }
        }
        _ => {}
    }
    Flow::Done
}
