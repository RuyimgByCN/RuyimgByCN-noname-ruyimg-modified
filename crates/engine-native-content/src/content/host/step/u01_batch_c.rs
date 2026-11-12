use super::super::*;
use super::u01_batch_a::{
    ASK_KEY, flag, hand_options, he_cards, he_options, set_flag, set_turn_flag, suit_bit,
    turn_flag, u01_discard_one, u01_draw_now, u01_remove_owned,
};
use crate::catalog::{CardKind, Catalog, QueryCtx, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

#[path = "u01_batch_c_asks.rs"]
mod asks;
pub(crate) use asks::*;

pub(crate) fn step_u01_batch_c(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.def_key.as_str() {
        "dddqiahua" => match machine.stage {
            HostStage::Act => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                let n = ctx.core.player(t).hp.max(1) as usize;
                if ctx.core.player(owner).hand.len() < n {
                    return Flow::Done;
                }
                machine.card = Some(n as u32);
                wait_ask_c(machine, ctx, HostStage::SelectCard, 201)
            }
            HostStage::SelectCard => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                let n = machine.card.unwrap_or(1) as usize;
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.len() < n {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "ddd_shown_count", picked.len() as u32);
                set_turn_flag(ctx.core, t, "dddxunxun_granted", "dddxunxun_stamp", 1);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddqiahua",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddxunxun" => match machine.stage {
            HostStage::Act => {
                let mut revealed = Vec::new();
                for _ in 0..4 {
                    if let Some(c) = ctx.core.deck.pop() {
                        revealed.push(c);
                    }
                }
                if revealed.len() < 2 {
                    for c in revealed.into_iter().rev() {
                        ctx.core.deck.push(c);
                    }
                    return Flow::Done;
                }
                machine.cards = revealed;
                wait_ask_c(machine, ctx, HostStage::SelectCard, 202)
            }
            HostStage::SelectCard => {
                let need = machine.cards.len().min(2);
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.len() < need {
                    for c in machine.cards.drain(..).rev() {
                        ctx.core.deck.push(c);
                    }
                    return Flow::Done;
                }
                let all = std::mem::take(&mut machine.cards);
                for c in all {
                    if !picked.contains(&c) {
                        ctx.core.deck.insert(0, c);
                    }
                }
                for c in picked {
                    ctx.core.deck.push(c);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxunxun",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddfusi" => {
            set_turn_flag(ctx.core, owner, "dddfusi_active", "dddfusi_stamp", 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfusi",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        "dddfusi_global" => match machine.stage {
            HostStage::Act => {
                let Some(src) = fusi_source(ctx.core, ctx.catalog, owner) else {
                    return Flow::Done;
                };
                machine.target = Some(src);
                wait_ask_c(machine, ctx, HostStage::SelectCard, 203)
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(c);
                wait_ask_c(machine, ctx, HostStage::SelectNature, 204)
            }
            HostStage::SelectNature => {
                let src = machine.target.unwrap_or(owner);
                let Some(c) = machine.card else {
                    return Flow::Done;
                };
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    let cur = ctx.core.turn.current;
                    if src != cur && u01_remove_owned(ctx, src, c) {
                        ctx.core.player_mut(cur).hand.push(c);
                    }
                    set_turn_flag(ctx.core, src, "dddfusi_used_card", "dddfusi_stamp", 1);
                    set_turn_flag(ctx.core, owner, "dddfusi_used_card", "dddfusi_stamp", 1);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddfusi_global",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddtuoji" => match machine.stage {
            HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectNature, 205),
            HostStage::SelectNature => {
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    u01_draw_now(ctx, owner, 3);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddtuoji",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddchashi" => {
            if machine.card == Some(1) {
                let t = machine.target.unwrap_or(ctx.core.turn.current);
                set_turn_flag(ctx.core, owner, "dddchashi_suit", "dddchashi_stamp", 0);
                set_turn_flag(ctx.core, owner, "dddchashi_trick", "dddchashi_stamp", 0);
                u01_draw_now(ctx, owner, 1);
                if t != owner && ctx.core.player(t).alive {
                    u01_draw_now(ctx, t, 1);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddchashi",
                    owner,
                    presentation: None,
                });
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => {
                    set_turn_flag(ctx.core, owner, "dddchashi_suit", "dddchashi_stamp", 0);
                    set_turn_flag(ctx.core, owner, "dddchashi_trick", "dddchashi_stamp", 0);
                    wait_ask_c(machine, ctx, HostStage::SelectCard, 206)
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let sb = suit_bit(ctx.catalog, c);
                    let is_tr = is_trick_kind(ctx.catalog.card_kind(c));
                    u01_discard_one(ctx, owner, c);
                    set_turn_flag(ctx.core, owner, "dddchashi_suit", "dddchashi_stamp", sb);
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddchashi_trick",
                        "dddchashi_stamp",
                        u32::from(is_tr),
                    );
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddchashi",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddqice" => {
            if machine.stage == HostStage::Act && machine.card == Some(1) {
                set_flag(ctx.core, owner, "ddd_qice_disabled", 0);
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectNature, 207),
                HostStage::SelectNature => {
                    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                        u01_draw_now(ctx, owner, 2);
                        set_flag(ctx.core, owner, "ddd_qice_disabled", 1);
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddqice",
                            owner,
                            presentation: None,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddmiaoxing" => match machine.stage {
            HostStage::Act => {
                let p1_empty = ctx
                    .core
                    .vm_aux
                    .piles
                    .get(&(owner, "dddmiaoxing_1".to_string()))
                    .is_none_or(Vec::is_empty);
                let p2_empty = ctx
                    .core
                    .vm_aux
                    .piles
                    .get(&(owner, "dddmiaoxing_2".to_string()))
                    .is_none_or(Vec::is_empty);
                if p1_empty && p2_empty {
                    let mut pile1 = Vec::new();
                    for _ in 0..2 {
                        if let Some(c) = ctx.core.deck.pop() {
                            pile1.push(c);
                        }
                    }
                    let mut pile2 = Vec::new();
                    for _ in 0..2 {
                        if let Some(c) = ctx.core.deck.pop() {
                            pile2.push(c);
                        }
                    }
                    ctx.core
                        .vm_aux
                        .piles
                        .insert((owner, "dddmiaoxing_1".to_string()), pile1);
                    ctx.core
                        .vm_aux
                        .piles
                        .insert((owner, "dddmiaoxing_2".to_string()), pile2);
                }
                wait_ask_c(machine, ctx, HostStage::SelectNature, 208)
            }
            HostStage::SelectNature => {
                let key = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(1)) => "dddmiaoxing_2",
                    _ => "dddmiaoxing_1",
                };
                let target_len = ctx.core.player(owner).hand.len();
                let mut pile = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, key.to_string()))
                    .unwrap_or_default();
                while pile.len() < target_len {
                    if let Some(c) = ctx.core.deck.pop() {
                        pile.push(c);
                    } else {
                        break;
                    }
                }
                while pile.len() > target_len {
                    if let Some(c) = pile.pop() {
                        ctx.to_discard(c);
                    }
                }
                ctx.core.vm_aux.piles.insert((owner, key.to_string()), pile);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddmiaoxing",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddfushi" => match machine.stage {
            HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectNature, 209),
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let key = if tag == 1 {
                    "dddmiaoxing_2"
                } else {
                    "dddmiaoxing_1"
                };
                set_turn_flag(ctx.core, owner, "dddfushi_used_turn", "dddfushi_stamp", 1);
                let pile = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, key.to_string()))
                    .unwrap_or_default();
                let old_hand = std::mem::replace(&mut ctx.core.player_mut(owner).hand, pile);
                ctx.core
                    .vm_aux
                    .piles
                    .insert((owner, key.to_string()), old_hand);
                machine.card = Some(u32::from(tag));
                if has_male_other(ctx.core, ctx.catalog, owner) {
                    wait_ask_c(machine, ctx, HostStage::TargetChoice, 210)
                } else {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddfushi",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
            }
            HostStage::TargetChoice => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    let key = if machine.card == Some(1) {
                        "dddmiaoxing_2"
                    } else {
                        "dddmiaoxing_1"
                    };
                    let pile = ctx
                        .core
                        .vm_aux
                        .piles
                        .remove(&(owner, key.to_string()))
                        .unwrap_or_default();
                    let old_hand = std::mem::replace(&mut ctx.core.player_mut(t).hand, pile);
                    ctx.core
                        .vm_aux
                        .piles
                        .insert((owner, key.to_string()), old_hand);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddfushi",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddfenji" => match machine.stage {
            HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectNature, 211),
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if tag > 2 {
                    return Flow::Done;
                }
                machine.card = Some(u32::from(tag));
                wait_ask_c(machine, ctx, HostStage::TargetChoice, 212)
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let tag = machine.card.unwrap_or(0).min(2);
                let chosen_kind = match tag {
                    0 => CardKind::Duel,
                    1 => CardKind::Dismantle,
                    _ => CardKind::Snatch,
                };
                let mask = flag(ctx.core, owner, "dddfenji_used_mask") | (1 << tag);
                set_flag(ctx.core, owner, "dddfenji_used_mask", mask);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddfenji",
                    owner,
                    presentation: None,
                });
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![t],
                    nature: DamageNature::Normal,
                    use_as: Some(chosen_kind),
                    cancelled: false,
                }));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddxiaheng" => match machine.stage {
            HostStage::Act => wait_ask_c(machine, ctx, HostStage::TargetChoice, 213),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let to_disc: Vec<CardId> = he_cards(ctx.core, t1).into_iter().take(2).collect();
                for c in to_disc {
                    u01_discard_one(ctx, t1, c);
                }
                machine.target = Some(t1);
                wait_ask_c(machine, ctx, HostStage::JiefenTarget, 214)
            }
            HostStage::JiefenTarget => {
                let Some(ChoiceOption::Player(t2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t1 = machine.target.unwrap_or(owner);
                if t1 != owner && t2 != owner && ctx.core.player(owner).max_hp > 1 {
                    let pl = ctx.core.player_mut(owner);
                    pl.max_hp -= 1;
                    pl.hp = pl.hp.min(pl.max_hp as i32);
                }
                if t1 == t2 {
                    set_flag(ctx.core, owner, "dddxiaheng_same_target", 1);
                } else {
                    set_flag(ctx.core, owner, "dddxiaheng_same_target", 0);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxiaheng",
                    owner,
                    presentation: None,
                });
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t2,
                    1,
                    DamageNature::Normal,
                    None,
                )));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddfengzheng" => {
            if machine.card == Some(1) {
                return match machine.stage {
                    HostStage::Act => {
                        let x = flag(ctx.core, owner, "dddfengzheng_count") as usize;
                        if x == 0 {
                            return Flow::Done;
                        }
                        let mut revealed = Vec::new();
                        for _ in 0..x {
                            if let Some(c) = ctx.core.deck.pop() {
                                revealed.push(c);
                            }
                        }
                        machine.cards = revealed;
                        wait_ask_c(machine, ctx, HostStage::TargetChoice, 216)
                    }
                    HostStage::TargetChoice => {
                        let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        let cards = std::mem::take(&mut machine.cards);
                        ctx.core.player_mut(t).hand.extend(cards);
                        set_flag(ctx.core, owner, "dddfengzheng_count", 0);
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddfengzheng",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    }
                    _ => Flow::Done,
                };
            }
            match machine.stage {
                HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectCard, 215),
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
                    for c in picked {
                        u01_discard_one(ctx, owner, c);
                    }
                    u01_draw_now(ctx, owner, 2);
                    let cur = flag(ctx.core, owner, "dddfengzheng_count");
                    set_flag(ctx.core, owner, "dddfengzheng_count", cur + 1);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddfengzheng",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddyulv" => {
            if machine.card == Some(1) {
                return match machine.stage {
                    HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectNature, 218),
                    HostStage::SelectNature => {
                        let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        let cur = ctx.core.turn.current;
                        if tag == 0 {
                            u01_draw_now(ctx, cur, 1);
                        } else if let Some(c) = he_cards(ctx.core, cur).first().copied() {
                            u01_discard_one(ctx, cur, c);
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddyulv",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    }
                    _ => Flow::Done,
                };
            }
            match machine.stage {
                HostStage::Act => {
                    u01_draw_now(ctx, owner, 1);
                    wait_ask_c(machine, ctx, HostStage::SelectCard, 217)
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if u01_remove_owned(ctx, owner, c) {
                        ctx.core
                            .vm_aux
                            .piles
                            .entry((owner, "dddyulv".to_string()))
                            .or_default()
                            .push(c);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddyulv",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddyulv_old" => {
            if machine.card == Some(1) {
                return match machine.stage {
                    HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectNature, 220),
                    HostStage::SelectNature => {
                        let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        let cur = ctx.core.turn.current;
                        if tag == 0 {
                            u01_draw_now(ctx, cur, 1);
                        } else if let Some(c) = he_cards(ctx.core, cur).first().copied() {
                            u01_discard_one(ctx, cur, c);
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddyulv_old",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    }
                    _ => Flow::Done,
                };
            }
            match machine.stage {
                HostStage::Act => wait_ask_c(machine, ctx, HostStage::SelectNature, 219),
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(r)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    set_flag(ctx.core, owner, "dddyulv_old_rank", u32::from(r));
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddyulv_old",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddfenye" => match machine.stage {
            HostStage::Act => wait_ask_c(machine, ctx, HostStage::TargetChoice, 221),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.target = Some(t);
                wait_ask_c(machine, ctx, HostStage::ComparePlayerPick, 222)
            }
            HostStage::ComparePlayerPick => {
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(c1);
                wait_ask_c(machine, ctx, HostStage::CompareTargetPick, 223)
            }
            HostStage::CompareTargetPick => {
                let (Some(t), Some(c1), Some(ChoiceOption::Card(c2))) =
                    (machine.target, machine.card, ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                let r1 = ctx.catalog.cards[c1 as usize].rank;
                let r2 = ctx.catalog.cards[c2 as usize].rank;
                u01_discard_one(ctx, owner, c1);
                u01_discard_one(ctx, t, c2);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddfenye",
                    owner,
                    presentation: None,
                });
                if r1 > r2 {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                } else if r2 > r1 {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: t,
                        targets: vec![owner],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddshichao" => match machine.stage {
            HostStage::Act => wait_ask_c(machine, ctx, HostStage::TargetChoice, 224),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let cap = ctx.core.player(owner).max_hp.min(5) as usize;
                let hlen = ctx.core.player(t).hand.len();
                if hlen < cap {
                    u01_draw_now(ctx, t, cap - hlen);
                } else if hlen > cap {
                    let excess = hlen - cap;
                    let to_disc: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .take(excess)
                        .collect();
                    for c in to_disc {
                        u01_discard_one(ctx, t, c);
                    }
                }
                set_flag(ctx.core, owner, "dddshichao_guard", u32::from(t) + 1);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddshichao",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
