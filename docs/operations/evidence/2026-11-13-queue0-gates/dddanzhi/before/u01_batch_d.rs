use super::super::*;
use super::u01_batch_a::{
    ASK_KEY, distinct_suits, flag, hand_options, he_cards, he_options, is_red, set_flag,
    set_turn_flag, turn_flag, u01_discard_one, u01_draw_now, u01_remove_owned,
};
use crate::catalog::{CardKind, Catalog, Faction, QueryCtx, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

#[path = "u01_batch_d_asks.rs"]
mod asks;
pub(crate) use asks::*;

pub(crate) fn step_u01_batch_d(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.def_key.as_str() {
        "dddyouxue" => match machine.stage {
            HostStage::Act => wait_ask_d(machine, ctx, HostStage::TargetChoice, 301),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let holder_raw = flag(ctx.core, owner, "dddyouxue_holder");
                let prev_holder =
                    if holder_raw > 0 && (holder_raw as usize - 1) < ctx.core.players.len() {
                        (holder_raw - 1) as PlayerId
                    } else {
                        owner
                    };
                let dist = ctx.core.computed_distance(ctx.catalog, prev_holder, t).max(1) as usize;
                set_flag(ctx.core, owner, "dddyouxue_holder", u32::from(t) + 1);
                u01_draw_now(ctx, owner, dist);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddyouxue",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddchengjing" => {
            if machine.stage == HostStage::Act && machine.card == Some(1) {
                clear_once_used(ctx.core, ctx.catalog, owner, "dddchengjing");
                if let Some(last) = ctx.core.discard.pop() {
                    let old_cards: Vec<CardId> = ctx
                        .core
                        .vm_aux
                        .piles
                        .entry((owner, "dddchengjing".to_string()))
                        .or_default()
                        .drain(..)
                        .collect();
                    for old in old_cards {
                        ctx.to_discard(old);
                    }
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((owner, "dddchengjing".to_string()))
                        .or_default()
                        .push(last);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddchengjing",
                        owner,
                        presentation: None,
                    });
                }
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => {
                    clear_once_used(ctx.core, ctx.catalog, owner, "dddchengjing");
                    wait_ask_d(machine, ctx, HostStage::SelectCard, 302)
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    machine.card = Some(c);
                    wait_ask_d(machine, ctx, HostStage::TargetChoice, 303)
                }
                HostStage::TargetChoice => {
                    let (Some(c), Some(ChoiceOption::Player(t))) =
                        (machine.card, ctx.answer().first().copied())
                    else {
                        return Flow::Done;
                    };
                    set_turn_flag(
                        ctx.core,
                        owner,
                        "dddchengjing_used_turn",
                        "dddchengjing_stamp",
                        1,
                    );
                    clear_once_used(ctx.core, ctx.catalog, owner, "dddchengjing");
                    let use_kind = ctx
                        .core
                        .vm_aux
                        .piles
                        .get(&(owner, "dddchengjing".to_string()))
                        .and_then(|p| p.last().copied())
                        .map(|pc| ctx.catalog.card_kind(pc))
                        .unwrap_or(CardKind::Sha);
                    let use_card = if matches!(
                        use_kind,
                        CardKind::Le | CardKind::Bing | CardKind::ShanDian
                    ) {
                        Some(c)
                    } else {
                        if u01_remove_owned(ctx, owner, c) {
                            ctx.to_discard(c);
                        }
                        None
                    };
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddchengjing",
                        owner,
                        presentation: None,
                    });
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: use_card,
                        player: owner,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(use_kind),
                        cancelled: false,
                    }));
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddduanbing" => {
            if matches!(machine.stage, HostStage::Act | HostStage::SelectNature)
                && machine.card == Some(1)
            {
                match machine.stage {
                    HostStage::Act => {
                        clear_once_used(ctx.core, ctx.catalog, owner, "dddduanbing");
                        wait_ask_d(machine, ctx, HostStage::SelectNature, 306)
                    }
                    HostStage::SelectNature => {
                        clear_once_used(ctx.core, ctx.catalog, owner, "dddduanbing");
                        if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                            if let Some(t) = machine.target {
                                if let Some(jc) = ctx.core.player_mut(owner).judge.pop() {
                                    ctx.core.player_mut(t).judge.push(jc);
                                }
                            }
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "dddduanbing",
                                owner,
                                presentation: None,
                            });
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            } else {
                match machine.stage {
                    HostStage::Act => {
                        clear_once_used(ctx.core, ctx.catalog, owner, "dddduanbing");
                        wait_ask_d(machine, ctx, HostStage::SelectCard, 304)
                    }
                    HostStage::SelectCard => {
                        let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        machine.card = Some(c);
                        wait_ask_d(machine, ctx, HostStage::TargetChoice, 305)
                    }
                    HostStage::TargetChoice => {
                        let (Some(c), Some(ChoiceOption::Player(t))) =
                            (machine.card, ctx.answer().first().copied())
                        else {
                            return Flow::Done;
                        };
                        set_turn_flag(
                            ctx.core,
                            owner,
                            "dddduanbing_used_turn",
                            "dddduanbing_stamp",
                            1,
                        );
                        clear_once_used(ctx.core, ctx.catalog, owner, "dddduanbing");
                        if u01_remove_owned(ctx, owner, c) {
                            ctx.core.player_mut(owner).judge.push(c);
                            ctx.core.declared_kinds.insert(c, CardKind::Bing);
                        }
                        u01_draw_now(ctx, owner, 2);
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddduanbing",
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
                }
            }
        }
        "dddxiaoxing" => {
            if machine.card == Some(1) {
                set_flag(ctx.core, owner, "dddxiaoxing_lost", 1);
            } else if flag(ctx.core, owner, "dddxiaoxing_init_drawn") == 0 {
                set_flag(ctx.core, owner, "dddxiaoxing_init_drawn", 1);
                u01_draw_now(ctx, owner, 3);
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddxiaoxing",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        "dddlangzhi" => match machine.stage {
            HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectNature, 307),
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if tag == 0 {
                    if !he_cards(ctx.core, owner).is_empty() {
                        wait_ask_d(machine, ctx, HostStage::SelectCard, 308)
                    } else {
                        Flow::Done
                    }
                } else {
                    set_flag(ctx.core, owner, "dddlangzhi_lost", 1);
                    let others: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|&p| p != owner)
                        .collect();
                    for p in others {
                        if let Some(c) = he_cards(ctx.core, p).first().copied() {
                            if u01_remove_owned(ctx, p, c) {
                                ctx.core.player_mut(owner).hand.push(c);
                            }
                        }
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddlangzhi",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c_own)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let first_t = ctx
                    .core
                    .alive_players()
                    .find(|&p| p != owner && !he_cards(ctx.core, p).is_empty());
                if let Some(t) = first_t {
                    if let Some(c_other) = he_cards(ctx.core, t).first().copied() {
                        let r1 = u01_remove_owned(ctx, owner, c_own);
                        let r2 = u01_remove_owned(ctx, t, c_other);
                        if r1 {
                            ctx.core.player_mut(t).hand.push(c_own);
                        }
                        if r2 {
                            ctx.core.player_mut(owner).hand.push(c_other);
                        }
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddlangzhi",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddfuyi" => match machine.stage {
            HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectNature, 309),
            HostStage::SelectNature => {
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    let actor = machine.target.unwrap_or(owner);
                    let to_disc: Vec<CardId> =
                        he_cards(ctx.core, actor).into_iter().take(2).collect();
                    for c in to_disc {
                        u01_discard_one(ctx, actor, c);
                    }
                    set_flag(ctx.core, owner, "dddxiaoxing_lost", 0);
                    set_flag(ctx.core, owner, "dddlangzhi_lost", 0);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddfuyi",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddzhuanshe" => {
            if machine.card == Some(1) {
                match machine.stage {
                    HostStage::Act => {
                        let Some(t) = machine.target else {
                            return Flow::Done;
                        };
                        if turn_flag(ctx.core, owner, "dddzhuanshe_given", "dddzhuanshe_stamp")
                            != u32::from(t) + 1
                        {
                            return Flow::Done;
                        }
                        wait_ask_d(machine, ctx, HostStage::SelectNature, 311)
                    }
                    HostStage::SelectNature => {
                        let Some(t) = machine.target else {
                            return Flow::Done;
                        };
                        if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "dddzhuanshe",
                                owner,
                                presentation: None,
                            });
                            ctx.emit(EventData::Damage(crate::machine::damage_data(
                                Some(owner),
                                t,
                                1,
                            )));
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            } else {
                match machine.stage {
                    HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectCard, 310),
                    HostStage::SelectCard => {
                        let (Some(t), Some(ChoiceOption::Card(c))) =
                            (machine.target, ctx.answer().first().copied())
                        else {
                            return Flow::Done;
                        };
                        if u01_remove_owned(ctx, owner, c) {
                            ctx.core.player_mut(t).hand.push(c);
                            set_turn_flag(
                                ctx.core,
                                owner,
                                "dddzhuanshe_given",
                                "dddzhuanshe_stamp",
                                u32::from(t) + 1,
                            );
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "dddzhuanshe",
                                owner,
                                presentation: None,
                            });
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
        }
        "dddweiqiu" => {
            u01_draw_now(ctx, owner, 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddweiqiu",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        "dddlianer" => match machine.stage {
            HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectNature, 312),
            HostStage::SelectNature => {
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    if let Some(c) = machine.card {
                        let rank = u32::from(ctx.catalog.cards[c as usize].rank);
                        set_turn_flag(
                            ctx.core,
                            owner,
                            "dddlianer_rank_cap",
                            "dddlianer_stamp",
                            rank,
                        );
                        if ctx.core.table.contains(&c) {
                            ctx.core.table.retain(|&x| x != c);
                            ctx.core.player_mut(owner).hand.push(c);
                        } else if ctx.core.discard.contains(&c) {
                            ctx.core.discard.retain(|&x| x != c);
                            ctx.core.player_mut(owner).hand.push(c);
                        } else if !ctx.core.player(owner).hand.contains(&c) {
                            u01_draw_now(ctx, owner, 1);
                        }
                    } else {
                        u01_draw_now(ctx, owner, 1);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddlianer",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddanzhi" => {
            if machine.stage == HostStage::Act && machine.card == Some(1) {
                let is_blk = machine
                    .cards
                    .first()
                    .is_some_and(|&c| !is_red(ctx.catalog, c));
                set_flag(ctx.core, owner, "dddanzhi_last_black", u32::from(is_blk));
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => wait_ask_d(machine, ctx, HostStage::TargetChoice, 313),
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                        set_turn_flag(ctx.core, t, "dddanzhi_immune", "dddanzhi_stamp", 1);
                        if t != owner {
                            set_turn_flag(ctx.core, owner, "dddanzhi_immune", "dddanzhi_stamp", 1);
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddanzhi",
                            owner,
                            presentation: None,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddshixing" => {
            let mode = compute_shixing_mode(ctx.core, ctx.catalog);
            set_flag(ctx.core, owner, "dddshixing_mode", mode);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddshixing",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        "ddddanggu" => match machine.stage {
            HostStage::Act => wait_ask_d(machine, ctx, HostStage::TargetChoice, 314),
            HostStage::TargetChoice => {
                let picked: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                for p in picked {
                    if !ctx.core.player(p).linked {
                        ctx.core.player_mut(p).linked = true;
                        ctx.log(LogEvent::Linked {
                            player: p,
                            linked: true,
                        });
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "ddddanggu",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddfuzong" => {
            if machine.card == Some(1) {
                if let Some(c) = he_cards(ctx.core, owner).first().copied() {
                    u01_discard_one(ctx, owner, c);
                }
            } else {
                set_flag(ctx.core, owner, "dddfuzong_init", 1);
                let qun_count = ctx
                    .core
                    .alive_players()
                    .filter(|&p| {
                        ctx.core.player(p).general_slots[0]
                            .and_then(|g| ctx.catalog.general_meta(g))
                            .is_some_and(|(_, fac, _, _, _)| fac == Faction::Qun)
                    })
                    .count();
                u01_draw_now(ctx, owner, qun_count.max(1));
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfuzong",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        "dddlanghuai" => match machine.stage {
            HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectNature, 315),
            HostStage::SelectNature => {
                if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                    set_flag(ctx.core, owner, "dddxuanlun_del", 0);
                    let suits = distinct_suits(&ctx.core.player(owner).hand, ctx.catalog);
                    let st = flag(ctx.core, owner, "dddlanghuai_state");
                    set_flag(ctx.core, owner, "dddlanghuai_state", u32::from(st == 0));
                    let draw_n = if st == 0 {
                        suits.max(1)
                    } else {
                        4usize.saturating_sub(suits).max(1)
                    };
                    u01_draw_now(ctx, owner, draw_n);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddlanghuai",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "dddxuanlun" => {
            if machine.card == Some(1) {
                match machine.stage {
                    HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectNature, 317),
                    HostStage::SelectNature => {
                        set_turn_flag(
                            ctx.core,
                            owner,
                            "dddxuanlun_triggered",
                            "dddxuanlun_stamp",
                            0,
                        );
                        let tag = match ctx.answer().first().copied() {
                            Some(ChoiceOption::Tag(v)) => v,
                            _ => 0,
                        };
                        if tag == 0 {
                            if !he_cards(ctx.core, owner).is_empty() {
                                wait_ask_d(machine, ctx, HostStage::SelectCard, 318)
                            } else {
                                ctx.log(LogEvent::SkillTriggered {
                                    skill: "dddxuanlun",
                                    owner,
                                    presentation: None,
                                });
                                Flow::Done
                            }
                        } else {
                            set_flag(ctx.core, owner, "dddxuanlun_del", 1);
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "dddxuanlun",
                                owner,
                                presentation: None,
                            });
                            Flow::Done
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
                        for c in picked {
                            if u01_remove_owned(ctx, owner, c) {
                                ctx.core.deck.push(c);
                            }
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddxuanlun",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            } else {
                match machine.stage {
                    HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectNature, 316),
                    HostStage::SelectNature => {
                        if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                            u01_draw_now(ctx, owner, 4);
                            set_turn_flag(
                                ctx.core,
                                owner,
                                "dddxuanlun_triggered",
                                "dddxuanlun_stamp",
                                1,
                            );
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "dddxuanlun",
                                owner,
                                presentation: None,
                            });
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
        }
        "dddjiexing" => {
            if machine.target == Some(owner) {
                if flag(ctx.core, owner, "dddjiexing_self_recover") > 0 {
                    set_flag(ctx.core, owner, "dddjiexing_self_recover", 0);
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "dddjiexing_used", 0);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddjiexing",
                    owner,
                    presentation: None,
                });
                return Flow::Done;
            }
            match machine.stage {
                HostStage::Act => wait_ask_d(machine, ctx, HostStage::SelectNature, 319),
                HostStage::SelectNature => {
                    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                        set_flag(ctx.core, owner, "dddjiexing_used", 1);
                        if let (Some(t), Some(c)) = (machine.target, machine.card) {
                            if ctx.core.player(t).equips.contains(&c) {
                                ctx.core.player_mut(t).equips.retain(|&x| x != c);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                if let Some(slot) = ctx.catalog.card_slot_of(ctx.core, c) {
                                    let old: Vec<CardId> = ctx
                                        .core
                                        .player(owner)
                                        .equips
                                        .iter()
                                        .copied()
                                        .filter(|&ec| {
                                            ctx.catalog.card_slot_of(ctx.core, ec) == Some(slot)
                                        })
                                        .collect();
                                    for ec in old {
                                        ctx.core.player_mut(owner).equips.retain(|&x| x != ec);
                                        crate::kernel::revoke_equip_skills(
                                            ctx.core,
                                            ctx.catalog,
                                            owner,
                                            ec,
                                        );
                                        ctx.to_discard(ec);
                                    }
                                }
                                ctx.core.player_mut(owner).equips.push(c);
                                crate::kernel::grant_equip_skills(ctx, owner, c);
                            }
                        } else {
                            set_flag(ctx.core, owner, "dddjiexing_self_recover", 1);
                            ctx.recover(owner, 1, None);
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddjiexing",
                            owner,
                            presentation: None,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            }
        }
        "dddbailei" => match machine.stage {
            HostStage::Act => wait_ask_d(machine, ctx, HostStage::TargetChoice, 320),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.target = Some(t);
                wait_ask_d(machine, ctx, HostStage::SelectNature, 321)
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
                    if let Some(c) = he_cards(ctx.core, t).first().copied() {
                        if u01_remove_owned(ctx, t, c) {
                            ctx.core.player_mut(owner).hand.push(c);
                        }
                    }
                } else {
                    if ctx.core.player(t).hp <= 1 {
                        set_flag(ctx.core, owner, "dddbailei_lost", 1);
                    }
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        t,
                        1,
                    )));
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddbailei",
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
