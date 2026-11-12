use super::super::*;
use super::u01_batch_a::{
    ASK_KEY, flag, hand_options, he_cards, he_options, is_red, set_flag, set_turn_flag,
    slot_occupied, turn_flag, u01_discard_one, u01_draw_now, u01_remove_owned,
};
use crate::catalog::{CardKind, Catalog, QueryCtx, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

#[path = "u01_batch_b_asks.rs"]
mod asks;
pub(crate) use asks::*;

fn pop_top_deck_card(ctx: &mut Ctx<'_>) -> Option<CardId> {
    if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
        let mut cards = std::mem::take(&mut ctx.core.discard);
        ctx.core.rng.shuffle(&mut cards);
        crate::kernel::elo_on_wash_deck(ctx.core, &cards);
        ctx.core.deck = cards;
        ctx.out.push(LogEvent::DeckReshuffled);
    }
    ctx.core.deck.pop()
}

fn give_owned_to_hand(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId, card: CardId) -> bool {
    if ctx.core.player(from).hand.contains(&card) {
        ctx.give_hand_card(from, to, card);
        true
    } else if u01_remove_owned(ctx, from, card) {
        ctx.core.player_mut(to).hand.push(card);
        true
    } else {
        false
    }
}

pub(crate) fn step_u01_batch_b(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.def_key.as_str() {
        "dddzhilian" => {
            if machine.stage == HostStage::Act && machine.card == Some(1) {
                if let Some(p) = machine.target {
                    let prev = turn_flag(ctx.core, owner, "dddzhilian_mask", "dddzhilian_stamp");
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddzhilian_mask",
                        "dddzhilian_stamp",
                        prev | (1 << p),
                    );
                }
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => {
                    let Some(t) = dddzhilian_partner(ctx.core, ctx.catalog, owner) else {
                        return Flow::Done;
                    };
                    machine.target = Some(t);
                    wait_ask_b(machine, ctx, HostStage::SelectNature, 101)
                }
                HostStage::SelectNature => {
                    let Some(t) = machine.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if mode == 0 {
                        u01_draw_now(ctx, owner, 1);
                        u01_draw_now(ctx, t, 1);
                    } else {
                        if let Some(c) = he_cards(ctx.core, owner).first().copied() {
                            u01_discard_one(ctx, owner, c);
                        }
                        if let Some(c) = he_cards(ctx.core, t).first().copied() {
                            u01_discard_one(ctx, t, c);
                        }
                    }
                    let shas: Vec<CardId> = ctx
                        .core
                        .discard
                        .iter()
                        .copied()
                        .filter(|&c| ctx.catalog.card_kind(c).is_sha_like())
                        .collect();
                    if !shas.is_empty() {
                        ctx.core.discard.retain(|c| !shas.contains(c));
                        ctx.core.player_mut(owner).hand.extend(shas);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddzhilian",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddjijian" => match machine.stage {
            HostStage::Act => wait_ask_b(machine, ctx, HostStage::SelectCard, 102),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(c);
                wait_ask_b(machine, ctx, HostStage::TargetChoice, 103)
            }
            HostStage::TargetChoice => {
                let (Some(c), Some(ChoiceOption::Player(t))) =
                    (machine.card, ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                give_owned_to_hand(ctx, owner, t, c);
                machine.target = Some(t);
                wait_ask_b(machine, ctx, HostStage::CompareTargetPick, 104)
            }
            HostStage::CompareTargetPick => {
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
                if picked.is_empty() {
                    return Flow::Done;
                }
                u01_draw_now(ctx, owner, 1);
                u01_draw_now(ctx, t, 1);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddjijian",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddjijian_old" => match machine.stage {
            HostStage::Act => wait_ask_b(machine, ctx, HostStage::TargetChoice, 105),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.target = Some(t);
                wait_ask_b(machine, ctx, HostStage::CompareTargetPick, 106)
            }
            HostStage::CompareTargetPick => {
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
                machine.cards = picked;
                let has_sha = ctx
                    .core
                    .player(owner)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.card_kind(c).is_sha_like());
                if has_sha {
                    wait_ask_b(machine, ctx, HostStage::SelectCard, 107)
                } else {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddjijian_old",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(sha)) = ctx.answer().first().copied() {
                    let color_match = machine
                        .cards
                        .iter()
                        .any(|&sc| is_red(ctx.catalog, sc) == is_red(ctx.catalog, sha));
                    if give_owned_to_hand(ctx, owner, t, sha) && color_match {
                        u01_draw_now(ctx, t, 1);
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddjijian_old",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddzhengjun" => match machine.stage {
            HostStage::Act => {
                let cat = machine.card.unwrap_or(1);
                let find_hp = || {
                    (turn_flag(ctx.core, owner, "dddzhengjun_hp_used", "dddzhengjun_stamp") == 0)
                        .then(|| {
                            ctx.core
                                .alive_players()
                                .find(|&p| p != owner && ctx.core.player(p).hp == ctx.core.player(owner).hp)
                        })
                        .flatten()
                };
                let find_hs = || {
                    (turn_flag(ctx.core, owner, "dddzhengjun_hs_used", "dddzhengjun_stamp") == 0)
                        .then(|| {
                            ctx.core.alive_players().find(|&p| {
                                p != owner
                                    && ctx.core.player(p).hand.len()
                                        == ctx.core.player(owner).hand.len()
                            })
                        })
                        .flatten()
                };
                let find_es = || {
                    (turn_flag(ctx.core, owner, "dddzhengjun_es_used", "dddzhengjun_stamp") == 0)
                        .then(|| {
                            ctx.core.alive_players().find(|&p| {
                                p != owner
                                    && ctx.core.player(p).equips.len()
                                        == ctx.core.player(owner).equips.len()
                            })
                        })
                        .flatten()
                };
                let resolved = match cat {
                    1 => find_hp().map(|p| (1u32, p)),
                    2 => find_hs()
                        .map(|p| (2u32, p))
                        .or_else(|| find_es().map(|p| (3u32, p))),
                    3 => find_es()
                        .map(|p| (3u32, p))
                        .or_else(|| find_hs().map(|p| (2u32, p))),
                    _ => None,
                };
                let Some((rcat, t)) = resolved else {
                    return Flow::Done;
                };
                machine.card = Some(rcat);
                machine.target = Some(t);
                wait_ask_b(machine, ctx, HostStage::SelectNature, 107 + rcat)
            }
            HostStage::SelectNature => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                match machine.card {
                    Some(1) => {
                        let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        set_turn_flag(
                            ctx.core,
                            owner,
                            "dddzhengjun_hp_used",
                            "dddzhengjun_stamp",
                            1,
                        );
                        if tag == 0 {
                            ctx.recover(t, 1, None);
                        } else {
                            ctx.emit(EventData::LoseHp {
                                player: t,
                                amount: 1,
                            });
                        }
                    }
                    Some(2) => {
                        let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        set_turn_flag(
                            ctx.core,
                            owner,
                            "dddzhengjun_hs_used",
                            "dddzhengjun_stamp",
                            1,
                        );
                        if tag == 0 {
                            u01_draw_now(ctx, t, 1);
                        } else if let Some(c) = he_cards(ctx.core, t).first().copied() {
                            u01_discard_one(ctx, t, c);
                        }
                    }
                    Some(3) => {
                        if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                            return Flow::Done;
                        }
                        set_turn_flag(
                            ctx.core,
                            owner,
                            "dddzhengjun_es_used",
                            "dddzhengjun_stamp",
                            1,
                        );
                        let mut moved = false;
                        let alive: Vec<PlayerId> = ctx.core.alive_players().collect();
                        'outer: for &src in &alive {
                            let eqs = ctx.core.player(src).equips.clone();
                            for c in eqs {
                                let Some(slot) = ctx.catalog.card_slot_of(ctx.core, c) else {
                                    continue;
                                };
                                if let Some(&dst) = alive.iter().find(|&&dst| {
                                    dst != src && !slot_occupied(ctx.core, ctx.catalog, dst, slot)
                                }) {
                                    ctx.core.player_mut(src).equips.retain(|&x| x != c);
                                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, c);
                                    ctx.core.player_mut(dst).equips.push(c);
                                    crate::kernel::grant_equip_skills(ctx, dst, c);
                                    moved = true;
                                    break 'outer;
                                }
                            }
                        }
                        if !moved {
                            if let Some(&src) =
                                alive.iter().find(|&&p| !ctx.core.player(p).equips.is_empty())
                            {
                                if let Some(&dst) = alive.iter().find(|&&p| p != src) {
                                    let c = ctx.core.player(src).equips[0];
                                    ctx.core.player_mut(src).equips.retain(|&x| x != c);
                                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, c);
                                    ctx.core.player_mut(dst).equips.push(c);
                                    crate::kernel::grant_equip_skills(ctx, dst, c);
                                }
                            }
                        }
                    }
                    _ => return Flow::Done,
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddzhengjun",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddxianxi" => match machine.stage {
            HostStage::Act => {
                let mut mask = 1u32 << owner;
                for &p in &machine.targets {
                    mask |= 1u32 << p;
                }
                set_flag(ctx.core, owner, "dddxianxi_excl_mask", mask);
                wait_ask_b(machine, ctx, HostStage::TargetChoice, 111)
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.target = Some(t);
                if he_cards(ctx.core, owner).is_empty() {
                    u01_draw_now(ctx, owner, 1);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddxianxi",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                } else {
                    wait_ask_b(machine, ctx, HostStage::SelectCard, 112)
                }
            }
            HostStage::SelectCard => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_discard_one(ctx, owner, c);
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        t,
                        1,
                    )));
                } else {
                    u01_draw_now(ctx, owner, 1);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxianxi",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddlingyong" => match machine.stage {
            HostStage::Act => wait_ask_b(machine, ctx, HostStage::SelectNature, 113),
            HostStage::SelectNature => {
                if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let Some(c) = pop_top_deck_card(ctx) else {
                    return Flow::Done;
                };
                if !ctx.catalog.card_kind(c).is_sha_like() {
                    ctx.core.player_mut(owner).hand.push(c);
                    if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                        return wait_ask_b(machine, ctx, HostStage::ComparePlayerPick, 114);
                    }
                } else {
                    ctx.to_discard(c);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddlingyong",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            HostStage::ComparePlayerPick => {
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    if let Some(c2) = pop_top_deck_card(ctx) {
                        if !ctx.catalog.card_kind(c2).is_sha_like() {
                            ctx.core.player_mut(owner).hand.push(c2);
                        } else {
                            ctx.to_discard(c2);
                        }
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddlingyong",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddxuxiao" => match machine.stage {
            HostStage::Act => wait_ask_b(machine, ctx, HostStage::SelectCard, 115),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(c);
                wait_ask_b(machine, ctx, HostStage::TargetChoice, 116)
            }
            HostStage::TargetChoice => {
                let (Some(c), Some(ChoiceOption::Player(t))) =
                    (machine.card, ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                if ctx.core.discard.contains(&c) {
                    ctx.core.discard.retain(|&x| x != c);
                    ctx.core.player_mut(t).judge.push(c);
                    ctx.core.declared_kinds.insert(c, CardKind::Bing);
                }
                u01_draw_now(ctx, t, 2);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxuxiao",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddxuyu" => match machine.stage {
            HostStage::Act => {
                if turn_flag(ctx.core, owner, "dddxuyu_must_discard", "dddxuyu_stamp") > 0 {
                    set_turn_flag(ctx.core, owner, "dddxuyu_must_discard", "dddxuyu_stamp", 0);
                    if he_cards(ctx.core, owner).is_empty() {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddxuyu",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    } else {
                        wait_ask_b(machine, ctx, HostStage::SelectCard, 117)
                    }
                } else {
                    wait_ask_b(machine, ctx, HostStage::SelectNature, 118)
                }
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_discard_one(ctx, owner, c);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxuyu",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            HostStage::SelectNature => {
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    u01_draw_now(ctx, owner, 1);
                    set_turn_flag(ctx.core, owner, "dddxuyu_must_discard", "dddxuyu_stamp", 1);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddxuyu",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddshijian" => match machine.stage {
            HostStage::Act => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                let cnt = turn_flag(ctx.core, t, "dddshijian_play_cnt", "dddshijian_stamp") + 1;
                set_turn_flag(ctx.core, t, "dddshijian_play_cnt", "dddshijian_stamp", cnt);
                if cnt == 2 && !he_cards(ctx.core, owner).is_empty() {
                    wait_ask_b(machine, ctx, HostStage::SelectCard, 119)
                } else if cnt == 3
                    && turn_flag(ctx.core, owner, "dddshijian_armed", "dddshijian_stamp")
                        == u32::from(t) + 1
                {
                    wait_ask_b(machine, ctx, HostStage::SelectNature, 120)
                } else {
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if give_owned_to_hand(ctx, owner, t, c) {
                        set_turn_flag(
                            ctx.core,
                            owner,
                            "dddshijian_armed",
                            "dddshijian_stamp",
                            u32::from(t) + 1,
                        );
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddshijian",
                            owner,
                            presentation: None,
                        });
                    }
                }
                Flow::Done
            }
            HostStage::SelectNature => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                let tag = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(v)) => v,
                    _ => 0,
                };
                if tag == 0 {
                    u01_draw_now(ctx, t, 1);
                } else {
                    u01_draw_now(ctx, owner, 1);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddshijian",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddxuyu_old" => match machine.stage {
            HostStage::Act => {
                let cnt =
                    turn_flag(ctx.core, owner, "dddxuyu_old_cnt", "dddxuyu_old_stamp") + 1;
                set_turn_flag(
                    ctx.core,
                    owner,
                    "dddxuyu_old_cnt",
                    "dddxuyu_old_stamp",
                    cnt,
                );
                if cnt == 1 {
                    wait_ask_b(machine, ctx, HostStage::SelectNature, 121)
                } else if cnt == 3
                    && turn_flag(ctx.core, owner, "dddxuyu_old_armed", "dddxuyu_old_stamp") == 1
                    && !he_cards(ctx.core, owner).is_empty()
                {
                    wait_ask_b(machine, ctx, HostStage::SelectCard, 122)
                } else {
                    Flow::Done
                }
            }
            HostStage::SelectNature => {
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    u01_draw_now(ctx, owner, 2);
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddxuyu_old_armed",
                        "dddxuyu_old_stamp",
                        1,
                    );
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddxuyu_old",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_discard_one(ctx, owner, c);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxuyu_old",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddshijian_old" => match machine.stage {
            HostStage::Act => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                let cnt =
                    turn_flag(ctx.core, t, "dddshijian_old_cnt", "dddshijian_old_stamp") + 1;
                set_turn_flag(
                    ctx.core,
                    t,
                    "dddshijian_old_cnt",
                    "dddshijian_old_stamp",
                    cnt,
                );
                if cnt == 3 {
                    wait_ask_b(machine, ctx, HostStage::SelectNature, 123)
                } else if cnt == 4
                    && turn_flag(
                        ctx.core,
                        owner,
                        "dddshijian_old_armed",
                        "dddshijian_old_stamp",
                    ) == u32::from(t) + 1
                    && !he_cards(ctx.core, t).is_empty()
                {
                    u01_draw_now(ctx, owner, 2);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddshijian_old",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                } else {
                    Flow::Done
                }
            }
            HostStage::SelectNature => {
                let Some(t) = machine.target else {
                    return Flow::Done;
                };
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    u01_draw_now(ctx, t, 1);
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddshijian_old_armed",
                        "dddshijian_old_stamp",
                        u32::from(t) + 1,
                    );
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddshijian_old",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddtaisi" => {
            if machine.stage == HostStage::Act && machine.card == Some(1) {
                set_turn_flag(
                    ctx.core,
                    owner,
                    "dddtaisi_hp_changed",
                    "dddtaisi_stamp",
                    1,
                );
                if let Some(s) = machine.target {
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddtaisi_dmg_src",
                        "dddtaisi_stamp",
                        u32::from(s) + 1,
                    );
                }
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => wait_ask_b(machine, ctx, HostStage::TargetChoice, 124),
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    machine.target = Some(t);
                    wait_ask_b(machine, ctx, HostStage::SelectCard, 125)
                }
                HostStage::SelectCard => {
                    let (Some(t), Some(ChoiceOption::Card(c))) =
                        (machine.target, ctx.answer().first().copied())
                    else {
                        return Flow::Done;
                    };
                    if ctx.core.discard.contains(&c) {
                        ctx.core.discard.retain(|&x| x != c);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                    let dmg_src =
                        turn_flag(ctx.core, owner, "dddtaisi_dmg_src", "dddtaisi_stamp");
                    let raw_src = flag(ctx.core, owner, "dddtaisi_dmg_src");
                    if dmg_src == u32::from(t) + 1 || raw_src == u32::from(t) + 1 {
                        u01_draw_now(ctx, owner, 2);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddtaisi",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddquche" => match machine.stage {
            HostStage::Act => wait_ask_b(machine, ctx, HostStage::SelectCard, 126),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c_red)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(c_red);
                wait_ask_b(machine, ctx, HostStage::TargetChoice, 127)
            }
            HostStage::TargetChoice => {
                let (Some(c_red), Some(ChoiceOption::Player(t_give))) =
                    (machine.card, ctx.answer().first().copied())
                else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "dddquche_used", 1);
                give_owned_to_hand(ctx, owner, t_give, c_red);
                machine.target = Some(t_give);
                let has_black = he_cards(ctx.core, owner)
                    .iter()
                    .any(|&c| !is_red(ctx.catalog, c));
                if has_black {
                    wait_ask_b(machine, ctx, HostStage::ComparePlayerPick, 128)
                } else {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddquche",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
            }
            HostStage::ComparePlayerPick => {
                let Some(t_give) = machine.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c_blk)) = ctx.answer().first().copied() {
                    u01_discard_one(ctx, owner, c_blk);
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![t_give],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddquche",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddqianlong" => {
            u01_draw_now(ctx, owner, 3);
            set_flag(ctx.core, owner, "ddd_qianlong_kill_reward", 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddqianlong",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        "ddddongcha" => {
            if machine.card == Some(0) {
                match machine.stage {
                    HostStage::Act => wait_ask_b(machine, ctx, HostStage::SelectCard, 129),
                    HostStage::SelectCard => {
                        let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        machine.cards = vec![c1];
                        wait_ask_b(machine, ctx, HostStage::CompareTargetPick, 130)
                    }
                    HostStage::CompareTargetPick => {
                        let (Some(t), Some(&c1), Some(ChoiceOption::Card(c2))) = (
                            machine.target,
                            machine.cards.first(),
                            ctx.answer().first().copied(),
                        ) else {
                            return Flow::Done;
                        };
                        let ok1 = u01_remove_owned(ctx, owner, c1);
                        let ok2 = u01_remove_owned(ctx, t, c2);
                        let pile = ctx
                            .core
                            .vm_aux
                            .piles
                            .entry((owner, "ddd_jian".into()))
                            .or_default();
                        if ok1 {
                            pile.push(c1);
                        }
                        if ok2 {
                            pile.push(c2);
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "ddddongcha",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            } else {
                match machine.stage {
                    HostStage::Act => wait_ask_b(machine, ctx, HostStage::SelectNature, 131),
                    HostStage::SelectNature => {
                        let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        let turn_p = machine.target.unwrap_or(ctx.core.turn.current);
                        let all: Vec<CardId> = ctx
                            .core
                            .vm_aux
                            .piles
                            .remove(&(owner, "ddd_jian".into()))
                            .unwrap_or_default();
                        for c in all {
                            if c == picked {
                                ctx.core.player_mut(owner).hand.push(c);
                            } else {
                                ctx.core.player_mut(turn_p).hand.push(c);
                            }
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "ddddongcha",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
        }
        "dddzhijie" => match machine.stage {
            HostStage::Act => wait_ask_b(machine, ctx, HostStage::SelectCard, 132),
            HostStage::SelectCard => {
                let picked: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.len() < 2 {
                    return Flow::Done;
                }
                if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(owner, "ddd_jian".into())) {
                    pile.retain(|c| !picked.contains(c));
                }
                for &c in &picked {
                    ctx.to_discard(c);
                }
                u01_draw_now(ctx, owner, 2);
                let same_color = is_red(ctx.catalog, picked[0]) == is_red(ctx.catalog, picked[1]);
                set_flag(
                    ctx.core,
                    owner,
                    "dddzhijie_viewas",
                    if same_color { 1 } else { 2 },
                );
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddzhijie",
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
