use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, Faction, QueryCtx, Suit, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

#[path = "u01_batch_a_asks.rs"]
mod asks;
pub(crate) use asks::*;

fn finish_juxian(ctx: &mut Ctx<'_>, owner: PlayerId, paid: CardId, gained: Option<CardId>) -> Flow {
    if gained.is_some_and(|c| is_red(ctx.catalog, paid) != is_red(ctx.catalog, c)) {
        set_turn_flag(ctx.core, owner, "dddjuxian_ban", "dddjuxian_stamp", 1);
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddjuxian",
        owner,
        presentation: None,
    });
    Flow::Done
}

pub(crate) fn step_u01_batch_a(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.def_key.as_str() {
        "dddyuanzhen" => match machine.stage {
            HostStage::Act => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if he_cards(ctx.core, t).is_empty() {
                    u01_draw_now(ctx, owner, 1);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddyuanzhen",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                } else {
                    wait_ask(machine, ctx, HostStage::SelectCard, 1)
                }
            }
            HostStage::SelectCard => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_discard_one(ctx, t, c);
                } else {
                    u01_draw_now(ctx, owner, 1);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddyuanzhen",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddzhishu" => match machine.stage {
            HostStage::Act => wait_ask(machine, ctx, HostStage::SelectCard, 2),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some(src) = ctx
                    .core
                    .alive_players()
                    .find(|&p| ctx.core.player(p).equips.contains(&c))
                else {
                    return Flow::Done;
                };
                machine.card = Some(c);
                machine.target = Some(src);
                wait_ask(machine, ctx, HostStage::TargetChoice, 3)
            }
            HostStage::TargetChoice => {
                let (Some(c), Some(src), Some(ChoiceOption::Player(dst))) =
                    (machine.card, machine.target, ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                if ctx.core.player(src).equips.contains(&c) {
                    ctx.core.player_mut(src).equips.retain(|&x| x != c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, c);
                    ctx.core.player_mut(dst).equips.push(c);
                    crate::kernel::grant_equip_skills(ctx, dst, c);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddzhishu",
                    owner,
                    presentation: None,
                });
                if !he_cards(ctx.core, dst).is_empty() || !ctx.core.player(dst).judge.is_empty() {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: src,
                        targets: vec![dst],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Dismantle),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddjuxian" => match machine.stage {
            HostStage::Act => wait_ask(machine, ctx, HostStage::SelectNature, 4),
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(u32::from(mode));
                wait_ask(machine, ctx, HostStage::SelectCard, 5)
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.cards = vec![c1];
                wait_ask(machine, ctx, HostStage::TargetChoice, 6)
            }
            HostStage::TargetChoice => {
                let (Some(mode), Some(&c1), Some(ChoiceOption::Player(t))) = (
                    machine.card,
                    machine.cards.first(),
                    ctx.answer().first().copied(),
                ) else {
                    return Flow::Done;
                };
                let used = turn_flag(ctx.core, owner, "dddjuxian_used", "dddjuxian_stamp") + 1;
                set_turn_flag(ctx.core, owner, "dddjuxian_used", "dddjuxian_stamp", used);
                if !u01_remove_owned(ctx, owner, c1) {
                    return Flow::Done;
                }
                if mode == 0 {
                    set_turn_flag(ctx.core, owner, "dddjuxian_top_used", "dddjuxian_stamp", 1);
                    ctx.core.deck.push(c1);
                    machine.target = Some(t);
                    // Upstream gainPlayerCard shuffles hidden buttons; keep that map on resume.
                    let mut hand = ctx.core.player(t).hand.clone();
                    ctx.core.rng.shuffle(&mut hand);
                    machine.cards.extend(hand);
                    wait_ask(machine, ctx, HostStage::CardsAsk, 28)
                } else {
                    set_turn_flag(ctx.core, owner, "dddjuxian_hand_used", "dddjuxian_stamp", 1);
                    ctx.core.player_mut(t).hand.push(c1);
                    let gained = u01_draw_now(ctx, owner, 1).first().copied();
                    finish_juxian(ctx, owner, c1, gained)
                }
            }
            HostStage::CardsAsk => {
                let (Some(t), Some(&paid)) = (machine.target, machine.cards.first()) else {
                    return Flow::Done;
                };
                let player = ctx.core.player(t);
                let gained = match ctx.answer().first() {
                    Some(ChoiceOption::HiddenCard(slot)) => machine
                        .cards
                        .get(*slot as usize + 1)
                        .copied()
                        .filter(|c| player.hand.contains(c)),
                    Some(ChoiceOption::Card(c)) if player.equips.contains(c) => Some(*c),
                    _ => None,
                };
                let Some(gained) = gained else {
                    return Flow::Done;
                };
                if !u01_remove_owned(ctx, t, gained) {
                    return Flow::Done;
                }
                ctx.core.player_mut(owner).hand.push(gained);
                finish_juxian(ctx, owner, paid, Some(gained))
            }
            _ => Flow::Done,
        },
        "dddjungui" => {
            if machine.stage == HostStage::Act && machine.card == Some(1) {
                if let Some(&c) = machine.cards.first() {
                    let prev =
                        turn_flag(ctx.core, owner, "dddjungui_suits_mask", "dddjungui_stamp");
                    let next = prev | suit_bit(ctx.catalog, c);
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddjungui_suits_mask",
                        "dddjungui_stamp",
                        next,
                    );
                }
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => wait_ask(machine, ctx, HostStage::TargetChoice, 7),
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    machine.target = Some(t);
                    u01_draw_now(ctx, t, 2);
                    let mask =
                        turn_flag(ctx.core, owner, "dddjungui_suits_mask", "dddjungui_stamp");
                    let used_suits = if mask > 0 {
                        mask.count_ones()
                    } else if flag(ctx.core, owner, "dddjungui_used_suits") > 0 {
                        flag(ctx.core, owner, "dddjungui_used_suits")
                    } else if ctx.core.turn.cards_used_play_phase > 0 {
                        1
                    } else {
                        0
                    };
                    if used_suits > 0 {
                        machine.card = Some(used_suits);
                        wait_ask(machine, ctx, HostStage::SelectNature, 8)
                    } else {
                        let rem = distinct_suits(&ctx.core.player(t).hand, ctx.catalog);
                        if rem == 0 {
                            return Flow::Done;
                        }
                        if ctx.core.player(t).hand.len() <= rem {
                            let all = ctx.core.player(t).hand.clone();
                            for c in all {
                                u01_discard_one(ctx, t, c);
                            }
                            return Flow::Done;
                        }
                        machine.card = Some(rem as u32);
                        wait_ask(machine, ctx, HostStage::SelectCard, 9)
                    }
                }
                HostStage::SelectNature => {
                    let Some(t) = machine.target else {
                        return Flow::Done;
                    };
                    let dec = match ctx.answer().first().copied() {
                        Some(ChoiceOption::Tag(v)) => usize::from(v),
                        _ => 0,
                    };
                    let rem =
                        distinct_suits(&ctx.core.player(t).hand, ctx.catalog).saturating_sub(dec);
                    if rem == 0 {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddjungui",
                            owner,
                            presentation: None,
                        });
                        return Flow::Done;
                    }
                    if ctx.core.player(t).hand.len() <= rem {
                        let all = ctx.core.player(t).hand.clone();
                        for c in all {
                            u01_discard_one(ctx, t, c);
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddjungui",
                            owner,
                            presentation: None,
                        });
                        return Flow::Done;
                    }
                    machine.card = Some(rem as u32);
                    wait_ask(machine, ctx, HostStage::SelectCard, 9)
                }
                HostStage::SelectCard => {
                    let Some(t) = machine.target else {
                        return Flow::Done;
                    };
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    for c in picked {
                        u01_discard_one(ctx, t, c);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddjungui",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddshilie" => match machine.stage {
            HostStage::Act => wait_ask(machine, ctx, HostStage::SelectCard, 10),
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
                    return Flow::Done;
                }
                let sum: u32 = picked
                    .iter()
                    .map(|&c| u32::from(ctx.catalog.cards[c as usize].rank))
                    .sum();
                let need = (ctx.core.player(owner).hp + ctx.core.player(ctx.core.turn.current).hp)
                    .max(1) as u32;
                if sum < need {
                    return Flow::Done;
                }
                machine.cards = picked;
                machine.card = Some(u32::from(sum == need));
                wait_ask(machine, ctx, HostStage::TargetChoice, 11)
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(
                    ctx.core,
                    owner,
                    "dddshilie_shown_count",
                    machine.cards.len() as u32,
                );
                if machine.card == Some(1) {
                    u01_draw_now(ctx, owner, machine.cards.len());
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddshilie",
                    owner,
                    presentation: None,
                });
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![t],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddyeshen" => match machine.stage {
            HostStage::Act => wait_ask(machine, ctx, HostStage::SelectNature, 12),
            HostStage::SelectNature => {
                if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let marks = flag(ctx.core, owner, "dddyeshen_count");
                let n = 3usize.saturating_sub(marks as usize).max(1);
                if ctx.core.deck.len() < n && !ctx.core.discard.is_empty() {
                    let mut cards = std::mem::take(&mut ctx.core.discard);
                    ctx.core.rng.shuffle(&mut cards);
                    crate::kernel::elo_on_wash_deck(ctx.core, &cards);
                    cards.append(&mut ctx.core.deck);
                    ctx.core.deck = cards;
                }
                let take_n = n.min(ctx.core.deck.len());
                let revealed: Vec<CardId> = ctx.core.deck.drain(0..take_n).collect();
                let next_marks = marks + 1;
                let fire_self = if next_marks >= 3 {
                    set_flag(ctx.core, owner, "dddyeshen_count", 0);
                    true
                } else {
                    set_flag(ctx.core, owner, "dddyeshen_count", next_marks);
                    false
                };
                machine.cards = revealed.clone();
                machine.card = Some(u32::from(fire_self));
                if revealed.iter().any(|&c| !is_red(ctx.catalog, c)) {
                    wait_ask(machine, ctx, HostStage::TargetChoice, 13)
                } else {
                    ctx.core.deck.extend(revealed);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddyeshen",
                        owner,
                        presentation: None,
                    });
                    if fire_self {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            None,
                            owner,
                            1,
                            DamageNature::Fire,
                            None,
                        )));
                    }
                    Flow::Done
                }
            }
            HostStage::TargetChoice => {
                let turn_p = machine.target.unwrap_or(owner);
                let mut blk_taken = false;
                let mut rest = Vec::new();
                for &c in &machine.cards {
                    if !blk_taken && !is_red(ctx.catalog, c) {
                        blk_taken = true;
                        ctx.to_discard(c);
                    } else {
                        rest.push(c);
                    }
                }
                ctx.core.deck.extend(rest);
                let picked: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if picked.is_empty() {
                    u01_draw_now(ctx, turn_p, 1);
                } else {
                    for p in picked {
                        let new_linked = !ctx.core.player(p).linked;
                        ctx.core.player_mut(p).linked = new_linked;
                        ctx.emit(EventData::Linked {
                            player: p,
                            linked: new_linked,
                        });
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddyeshen",
                    owner,
                    presentation: None,
                });
                if machine.card == Some(1) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        None,
                        owner,
                        1,
                        DamageNature::Fire,
                        None,
                    )));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddqiaoduan" => match machine.stage {
            HostStage::Act => {
                let num = ctx
                    .core
                    .alive_players()
                    .filter(|&p| ctx.core.player(p).linked)
                    .count();
                if num == 0 {
                    return Flow::Done;
                }
                let is_linked_branch = machine.card == Some(1);
                machine.card = Some(num as u32);
                if is_linked_branch {
                    wait_ask(machine, ctx, HostStage::SelectCard, 14)
                } else {
                    wait_ask(machine, ctx, HostStage::TargetChoice, 16)
                }
            }
            HostStage::SelectCard => {
                let num = machine.card.unwrap_or(1) as usize;
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.len() < num {
                    return Flow::Done;
                }
                machine.cards = picked;
                wait_ask(machine, ctx, HostStage::TargetChoice, 15)
            }
            HostStage::TargetChoice => {
                let ask_id = flag(ctx.core, owner, ASK_KEY);
                if ask_id == 15 {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddqiaoduan_used_turn",
                        "dddqiaoduan_stamp",
                        1,
                    );
                    let cards = machine.cards.clone();
                    for c in cards {
                        if u01_remove_owned(ctx, owner, c) {
                            ctx.core.deck.insert(0, c);
                        }
                    }
                    ctx.recover(t, 1, None);
                } else {
                    let mut picked: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddqiaoduan_used_turn",
                        "dddqiaoduan_stamp",
                        1,
                    );
                    picked.sort_unstable();
                    for p in picked {
                        u01_draw_now(ctx, p, 1);
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddqiaoduan",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "ddddiedang" => match machine.stage {
            HostStage::Act => {
                let st = flag(ctx.core, owner, "ddddiedang_state");
                machine.card = Some(st);
                if st == 0 {
                    u01_draw_now(ctx, owner, 3);
                    wait_ask(machine, ctx, HostStage::SelectCard, 17)
                } else {
                    wait_ask(machine, ctx, HostStage::SelectCard, 18)
                }
            }
            HostStage::SelectCard => {
                let st = machine.card.unwrap_or(0);
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if st == 1 {
                    if picked.len() < 3 {
                        return Flow::Done;
                    }
                    for c in picked {
                        u01_discard_one(ctx, owner, c);
                    }
                    u01_draw_now(ctx, owner, 1);
                } else {
                    for c in picked {
                        u01_discard_one(ctx, owner, c);
                    }
                }
                let my_h = ctx.core.player(owner).hand.len();
                let is_max = ctx
                    .core
                    .alive_players()
                    .all(|p| my_h >= ctx.core.player(p).hand.len());
                let is_min = ctx
                    .core
                    .alive_players()
                    .all(|p| my_h <= ctx.core.player(p).hand.len());
                if is_max || is_min {
                    set_flag(ctx.core, owner, "ddddiedang_state", u32::from(st == 0));
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "ddddiedang",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddanliu" => match machine.stage {
            HostStage::Act => wait_ask(machine, ctx, HostStage::TargetChoice, 19),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.target = Some(t);
                wait_ask(machine, ctx, HostStage::CompareTargetPick, 20)
            }
            HostStage::CompareTargetPick => {
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(c1);
                wait_ask(machine, ctx, HostStage::ComparePlayerPick, 21)
            }
            HostStage::ComparePlayerPick => {
                let (Some(t), Some(c1), Some(ChoiceOption::Card(c2))) =
                    (machine.target, machine.card, ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                machine.cards = vec![c2];
                if is_red(ctx.catalog, c1) == is_red(ctx.catalog, c2) {
                    wait_ask(machine, ctx, HostStage::SelectNature, 22)
                } else {
                    if ctx.core.player(t).hand.contains(&c1)
                        && ctx.core.player(owner).hand.contains(&c2)
                    {
                        ctx.take_hand_card(t, c1);
                        ctx.take_hand_card(owner, c2);
                        ctx.core.player_mut(owner).hand.push(c1);
                        ctx.core.player_mut(t).hand.push(c2);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddanliu",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
            }
            HostStage::SelectNature => {
                let (Some(t), Some(c1), Some(&c2)) =
                    (machine.target, machine.card, machine.cards.first())
                else {
                    return Flow::Done;
                };
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm))
                    && ctx.core.player(t).hand.contains(&c1)
                    && ctx.core.player(owner).hand.contains(&c2)
                {
                    ctx.take_hand_card(t, c1);
                    ctx.take_hand_card(owner, c2);
                    ctx.core.player_mut(owner).hand.push(c1);
                    ctx.core.player_mut(t).hand.push(c2);
                    if is_red(ctx.catalog, c1) {
                        ctx.recover(owner, 1, None);
                    } else {
                        u01_draw_now(ctx, t, 2);
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddanliu",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddguiying" => match machine.stage {
            HostStage::Act => {
                let Some(t) = first_wu_responder(ctx.core, ctx.catalog, owner) else {
                    return Flow::Done;
                };
                machine.target = Some(t);
                wait_ask(machine, ctx, HostStage::TargetChoice, 23)
            }
            HostStage::TargetChoice => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    if let Some(max_c) = he_cards(ctx.core, t)
                        .into_iter()
                        .max_by_key(|&c| ctx.catalog.cards[c as usize].rank)
                    {
                        u01_remove_owned(ctx, t, max_c);
                        ctx.core.player_mut(owner).hand.push(max_c);
                    }
                    let mut min_ej: Option<(PlayerId, CardId, u8)> = None;
                    for p in ctx.core.alive_players() {
                        for &c in ctx
                            .core
                            .player(p)
                            .equips
                            .iter()
                            .chain(ctx.core.player(p).judge.iter())
                        {
                            let r = ctx.catalog.cards[c as usize].rank;
                            if min_ej.is_none_or(|(_, _, mr)| r < mr) {
                                min_ej = Some((p, c, r));
                            }
                        }
                    }
                    if let Some((p_ej, c_ej, _)) = min_ej {
                        u01_remove_owned(ctx, p_ej, c_ej);
                        ctx.core.player_mut(t).hand.push(c_ej);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddguiying",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddbingjian" => match machine.stage {
            HostStage::Act => {
                let hlen = ctx.core.player(owner).hand.len();
                if hlen == 2 {
                    return Flow::Done;
                }
                let used = turn_flag(
                    ctx.core,
                    owner,
                    "dddbingjian_used_turn",
                    "dddbingjian_stamp",
                ) + 1;
                set_turn_flag(
                    ctx.core,
                    owner,
                    "dddbingjian_used_turn",
                    "dddbingjian_stamp",
                    used,
                );
                if hlen > 2 {
                    let diff = hlen - 2;
                    machine.card = Some(diff as u32);
                    machine.cards = vec![1];
                    wait_ask(machine, ctx, HostStage::SelectCard, 24)
                } else {
                    let diff = 2 - hlen;
                    u01_draw_now(ctx, owner, diff);
                    machine.card = Some(diff as u32);
                    machine.cards = vec![0];
                    wait_ask(machine, ctx, HostStage::TargetChoice, 25)
                }
            }
            HostStage::SelectCard => {
                let diff = machine.card.unwrap_or(1) as usize;
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.len() < diff {
                    return Flow::Done;
                }
                for c in picked {
                    u01_discard_one(ctx, owner, c);
                }
                wait_ask(machine, ctx, HostStage::TargetChoice, 25)
            }
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    let diff = machine.card.unwrap_or(1) as usize;
                    if machine.cards.first() == Some(&0) {
                        u01_draw_now(ctx, t, diff);
                    } else {
                        let to_disc: Vec<CardId> =
                            he_cards(ctx.core, t).into_iter().take(diff).collect();
                        for c in to_disc {
                            u01_discard_one(ctx, t, c);
                        }
                    }
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddbingjian",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddtongyu" => match machine.stage {
            HostStage::Act => wait_ask(machine, ctx, HostStage::SelectCard, 26),
            HostStage::SelectCard => {
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.is_empty() || distinct_suits(&picked, ctx.catalog) != picked.len() {
                    return Flow::Done;
                }
                let x = picked.len();
                for c in picked {
                    u01_discard_one(ctx, owner, c);
                }
                let mut revealed = Vec::new();
                for _ in 0..x {
                    if let Some(c) = ctx.core.deck.pop() {
                        revealed.push(c);
                    }
                }
                machine.cards = revealed;
                machine.targets = ctx.core.alive_players().collect();
                if machine.targets.is_empty() {
                    return Flow::Done;
                }
                let first_t = machine.targets.remove(0);
                machine.target = Some(first_t);
                wait_ask(machine, ctx, HostStage::SelectNature, 27)
            }
            HostStage::SelectNature => {
                let t = machine.target.unwrap_or(owner);
                let tag = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(v)) => v,
                    _ => 0,
                };
                if tag == 0 {
                    u01_draw_now(ctx, t, 1);
                    set_flag(ctx.core, t, "dddtongyu_hand_locked", 1);
                } else {
                    let rem: Vec<CardId> = machine.cards.drain(..).collect();
                    ctx.core.player_mut(t).hand.extend(rem);
                }
                if !machine.targets.is_empty() {
                    let next_t = machine.targets.remove(0);
                    machine.target = Some(next_t);
                    wait_ask(machine, ctx, HostStage::SelectNature, 27)
                } else {
                    let rem: Vec<CardId> = machine.cards.drain(..).collect();
                    for c in rem {
                        ctx.to_discard(c);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddtongyu",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
