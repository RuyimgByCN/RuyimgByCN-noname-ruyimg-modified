//! Active (`phaseUse`) Part B skills (`active_b.rs`):
//! `xinfenyue`, `xinfu_kannan`, `xinfu_zengdao`, `xinfu_zhaoxin`, `xinkuangfu`,
//! `yjjianshu`, `yongjin`, `zhanyi`, `paiyi`.

use super::super::l11_common::{
    alive_all, alive_others, discard_player_cards, emit_damage, emit_lose_hp, has_named_skill,
    he_cards, move_cards_to_hand,
};
use super::helpers::{
    card_type_group, clear_ask, flag, gain_hand_cards_from_pile_or_deck, grant_npc_b_skill,
    is_unblocked_npc_b, move_field_card, phase_stamp, pile_cards, pindian_resolve_cards,
    pop_card_from_deck_bottom_or_discard, push_to_pile, set_flag, take_from_pile,
    take_owned_cards_from_zones, turn_stamp, usable_sha_targets_no_range, wait_ask,
};
use super::*;
use crate::catalog::Catalog;
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

pub(super) fn has_phase_use_b(name: &str) -> bool {
    matches!(
        name,
        "xinfenyue"
            | "xinfu_kannan"
            | "xinfu_zengdao"
            | "xinfu_zhaoxin"
            | "xinkuangfu"
            | "yjjianshu"
            | "yongjin"
            | "zhanyi"
            | "paiyi"
    )
}

fn xinfenyue_limit(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> u32 {
    let alive = state.alive_players().count() as u32;
    let friends = state
        .alive_players()
        .filter(|&p| state.is_friend_of(catalog, owner, p))
        .count() as u32;
    alive.saturating_sub(friends)
}

pub(super) fn activate_b(
    skill: &HostSkillBehavior,
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<HostMachine> {
    if !is_unblocked_npc_b(state, catalog, owner) {
        return None;
    }
    let pstamp = phase_stamp(state);
    let tstamp = turn_stamp(state);
    let eligible = match skill.name.as_str() {
        "xinfenyue" => {
            let used = if flag(state, owner, "l11b_xinfenyue_stamp") == pstamp {
                flag(state, owner, "l11b_xinfenyue_used")
            } else {
                0
            };
            let lim = xinfenyue_limit(state, catalog, owner);
            used < lim
                && !state.player(owner).hand.is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "xinfu_kannan" => {
            let used = if flag(state, owner, "l11b_kannan_phase_stamp") == pstamp {
                flag(state, owner, "l11b_kannan_used")
            } else {
                0
            };
            flag(state, owner, "l11b_kannan_phase_used") != pstamp
                && (used as i32) < state.player(owner).hp
                && !state.player(owner).hand.is_empty()
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| {
                        !state.player(p).hand.is_empty()
                            && flag(state, p, "l11b_kannan_phase_used") != pstamp
                    })
        }
        "xinfu_zengdao" => {
            flag(state, owner, "l11b_limited_xinfu_zengdao") == 0
                && !state.player(owner).equips.is_empty()
                && !alive_others(state, owner).is_empty()
        }
        "xinfu_zhaoxin" => {
            flag(state, owner, "l11b_zhaoxin_stamp") != pstamp
                && !state.player(owner).hand.is_empty()
                && pile_cards(state, owner, "l11b_zhaoxin").len() < 3
        }
        "xinkuangfu" => {
            flag(state, owner, "l11b_xinkuangfu_stamp") != pstamp
                && alive_all(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).equips.is_empty())
        }
        "yjjianshu" => {
            flag(state, owner, "l11b_yjjianshu_stamp") != pstamp
                && !state.player(owner).hand.is_empty()
                && alive_others(state, owner).len() >= 2
        }
        "yongjin" => {
            flag(state, owner, "l11b_limited_yongjin") == 0
                && alive_all(state, owner).into_iter().any(|src| {
                    state.player(src).equips.iter().any(|&c| {
                        alive_all(state, owner)
                            .into_iter()
                            .any(|dst| dst != src && helpers::can_move_card_to(state, catalog, c, dst))
                    })
                })
        }
        "zhanyi" => flag(state, owner, "l11b_zhanyi_stamp") != pstamp && !he_cards(state, owner).is_empty(),
        "paiyi" => {
            flag(state, owner, "l11b_paiyi_stamp") != pstamp
                && !pile_cards(state, owner, "l11b_quanji").is_empty()
                && !alive_all(state, owner).is_empty()
        }
        _ => false,
    };
    let _ = tstamp;
    eligible.then(|| HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11CrossNpcB,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(super) fn step_active_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let pstamp = phase_stamp(ctx.core);
    let tstamp = turn_stamp(ctx.core);
    match m.def_key.as_str() {
        "xinfenyue" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_XINFENYUE_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                let prev = if flag(ctx.core, owner, "l11b_xinfenyue_stamp") == pstamp {
                    flag(ctx.core, owner, "l11b_xinfenyue_used")
                } else {
                    0
                };
                set_flag(ctx.core, owner, "l11b_xinfenyue_stamp", pstamp);
                set_flag(ctx.core, owner, "l11b_xinfenyue_used", prev + 1);
                helpers::log_skill(ctx, owner, "xinfenyue");
                wait_ask(
                    m,
                    ctx,
                    HostStage::ComparePlayerPick,
                    ASK_XINFENYUE_PINDIAN_OWNER,
                )
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c1);
                wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_XINFENYUE_PINDIAN_TARGET,
                )
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t = m.target.unwrap_or(owner);
                let c1 = m.card.unwrap_or(0);
                let (won, r1, _) = pindian_resolve_cards(ctx, owner, c1, t, c2);
                if !won || !ctx.core.player(owner).alive {
                    return Flow::Done;
                }
                m.card = Some(u32::from(r1));
                if r1 <= 5 && ctx.core.player(t).alive && !he_cards(ctx.core, t).is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_XINFENYUE_GAIN);
                }
                finish_xinfenyue_post_steal(ctx, owner, t, r1);
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let r1 = m.card.unwrap_or(13) as u8;
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    move_cards_to_hand(ctx, t, owner, &[c]);
                }
                finish_xinfenyue_post_steal(ctx, owner, t, r1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinfu_kannan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_XINFU_KANNAN_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                let prev = if flag(ctx.core, owner, "l11b_kannan_phase_stamp") == pstamp {
                    flag(ctx.core, owner, "l11b_kannan_used")
                } else {
                    0
                };
                set_flag(ctx.core, owner, "l11b_kannan_phase_stamp", pstamp);
                set_flag(ctx.core, owner, "l11b_kannan_used", prev + 1);
                helpers::log_skill(ctx, owner, "xinfu_kannan");
                wait_ask(
                    m,
                    ctx,
                    HostStage::ComparePlayerPick,
                    ASK_XINFU_KANNAN_PINDIAN_OWNER,
                )
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c1);
                wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_XINFU_KANNAN_PINDIAN_TARGET,
                )
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t = m.target.unwrap_or(owner);
                let c1 = m.card.unwrap_or(0);
                let (won, _, _) = pindian_resolve_cards(ctx, owner, c1, t, c2);
                let winner = if won { owner } else { t };
                set_flag(ctx.core, winner, "l11b_kannan_phase_used", pstamp);
                let cur = flag(ctx.core, winner, "l11b_kannan_eff");
                set_flag(ctx.core, winner, "l11b_kannan_eff", cur + 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinfu_zengdao" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XINFU_ZENGDAO_CARDS),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) if ctx.core.player(owner).equips.contains(&c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if cs.is_empty() {
                    return Flow::Done;
                }
                m.cards = cs;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_XINFU_ZENGDAO_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11b_limited_xinfu_zengdao", 1);
                helpers::log_skill(ctx, owner, "xinfu_zengdao");
                let moved = take_owned_cards_from_zones(ctx, owner, &m.cards);
                push_to_pile(ctx.core, t, "l11b_zengdao", &moved);
                grant_npc_b_skill(ctx, t, "xinfu_zengdao2");
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinfu_zhaoxin" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XINFU_ZHAOXIN_CARDS),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) if ctx.core.player(owner).hand.contains(&c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if cs.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11b_zhaoxin_stamp", pstamp);
                helpers::log_skill(ctx, owner, "xinfu_zhaoxin");
                let moved = take_owned_cards_from_zones(ctx, owner, &cs);
                let n = moved.len() as u32;
                push_to_pile(ctx.core, owner, "l11b_zhaoxin", &moved);
                if n > 0 {
                    ctx.start_draw(owner, n);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinkuangfu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_XINKUANGFU_TARGET),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                set_flag(ctx.core, owner, "l11b_xinkuangfu_stamp", pstamp);
                helpers::log_skill(ctx, owner, "xinkuangfu");
                wait_ask(
                    m,
                    ctx,
                    HostStage::SelectCard,
                    ASK_XINKUANGFU_DISCARD_EQUIP,
                )
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t = m.target.unwrap_or(owner);
                discard_player_cards(ctx, t, &[c]);
                set_flag(ctx.core, owner, "l11b_xinkuangfu_dealt_dmg", 0);
                set_flag(ctx.core, owner, "l11b_xinkuangfu_sha_fid", 0);
                if usable_sha_targets_no_range(ctx.core, ctx.catalog, owner).is_empty() {
                    return finish_xinkuangfu_check(m, ctx, owner, t);
                }
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_XINKUANGFU_SHA_TARGET)
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let Some(ChoiceOption::Player(st)) = ctx.answer().first().copied() else {
                    return finish_xinkuangfu_check(m, ctx, owner, t);
                };
                if !super::helpers::can_target_sha_no_range_npc(ctx.core, ctx.catalog, owner, st) {
                    return finish_xinkuangfu_check(m, ctx, owner, t);
                }
                let prev_len = ctx.res.stack.len();
                helpers::emit_virtual_sha(ctx, owner, vec![st], true);
                if ctx.res.stack.len() > prev_len {
                    if let Some(top) = ctx.res.stack.last() {
                        set_flag(ctx.core, owner, "l11b_xinkuangfu_sha_fid", top.id);
                    }
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
                finish_xinkuangfu_check(m, ctx, owner, t)
            }
            HostStage::DrawSettle => {
                let t = m.target.unwrap_or(owner);
                set_flag(ctx.core, owner, "l11b_xinkuangfu_sha_fid", 0);
                finish_xinkuangfu_check(m, ctx, owner, t)
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) if ctx.core.player(owner).hand.contains(&c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if !cs.is_empty() {
                    discard_player_cards(ctx, owner, &cs);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yjjianshu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_YJJIANSHU_CARD),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_YJJIANSHU_TARGETS)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let ts: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Player(p) => Some(p),
                        _ => None,
                    })
                    .collect();
                if ts.len() != 2 {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11b_yjjianshu_stamp", pstamp);
                helpers::log_skill(ctx, owner, "yjjianshu");
                let give_c = m.card.unwrap_or(0);
                move_cards_to_hand(ctx, owner, ts[0], &[give_c]);
                m.targets = ts;
                if ctx.core.player(m.targets[0]).hand.is_empty()
                    || ctx.core.player(m.targets[1]).hand.is_empty()
                {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_YJJIANSHU_PINDIAN_1)
            }
            HostStage::ComparePlayerPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c1);
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_YJJIANSHU_PINDIAN_2)
            }
            HostStage::CompareTargetPick => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let t0 = m.targets[0];
                let t1 = m.targets[1];
                let c1 = m.card.unwrap_or(0);
                let (_, r1, r2) = pindian_resolve_cards(ctx, t0, c1, t1, c2);
                if r1 > r2 {
                    emit_lose_hp(ctx, t1, 1);
                } else if r2 > r1 {
                    emit_lose_hp(ctx, t0, 1);
                } else {
                    emit_lose_hp(ctx, t0, 1);
                    emit_lose_hp(ctx, t1, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yongjin" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "l11b_limited_yongjin", 1);
                helpers::log_skill(ctx, owner, "yongjin");
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_YONGJIN_SRC)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(src)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(src);
                wait_ask(m, ctx, HostStage::SelectCard, ASK_YONGJIN_CARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(cid);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_YONGJIN_DST)
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(dst)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let src = m.target.unwrap_or(owner);
                let cid = m.card.unwrap_or(0);
                if move_field_card(ctx, src, cid, dst) {
                    m.cards.push(cid);
                }
                m.target = None;
                m.card = None;
                if m.cards.len() >= 3 {
                    return Flow::Done;
                }
                if ASK_YONGJIN_SRC > 0 {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_YONGJIN_SRC)
                } else {
                    Flow::Done
                }
            }
            _ => Flow::Done,
        },
        "zhanyi" => match m.stage {
            HostStage::Act => {
                if m.targets.len() == 2 {
                    // Triggered via `ShaBegin` (`zhanyi_equip`)
                    let t = m.target.unwrap_or(owner);
                    if !he_cards(ctx.core, t).is_empty() {
                        return wait_ask(m, ctx, HostStage::CardsAsk, ASK_ZHANYI_EQUIP_DISCARD);
                    }
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_ZHANYI_DISCARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let grp = card_type_group(ctx.catalog.resolved_card_kind(ctx.core, c));
                set_flag(ctx.core, owner, "l11b_zhanyi_stamp", pstamp);
                helpers::log_skill(ctx, owner, "zhanyi");
                discard_player_cards(ctx, owner, &[c]);
                m.card = Some(u32::from(grp));
                m.stage = HostStage::DrawSettle;
                emit_lose_hp(ctx, owner, 1);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if !ctx.core.player(owner).alive {
                    return Flow::Done;
                }
                match m.card.unwrap_or(1) {
                    0 => {
                        set_flag(ctx.core, owner, "l11b_zhanyi_basic", 1);
                    }
                    2 => {
                        set_flag(ctx.core, owner, "l11b_zhanyi_equip_stamp", pstamp);
                    }
                    _ => {
                        set_flag(ctx.core, owner, "l11b_zhanyi_trick_stamp", pstamp);
                        ctx.start_draw(owner, 2);
                    }
                }
                Flow::Done
            }
            HostStage::CardsAsk => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let cs: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match *o {
                        ChoiceOption::Card(c) => Some(c),
                        _ => None,
                    })
                    .collect();
                if !cs.is_empty() {
                    discard_player_cards(ctx, t, &cs);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "paiyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_PAIYI_PILE),
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.card = Some(c);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_PAIYI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let c = m.card.unwrap_or(0);
                let taken = take_from_pile(ctx.core, owner, "l11b_quanji", &[c]);
                if taken.is_empty() {
                    return Flow::Done;
                }
                set_flag(ctx.core, owner, "l11b_paiyi_stamp", pstamp);
                helpers::log_skill(ctx, owner, "paiyi");
                for tc in &taken {
                    ctx.to_discard(*tc);
                }
                ctx.emit(EventData::CardsDiscarded {
                    player: owner,
                    cards: taken,
                });
                let cur_marks = flag(ctx.core, owner, "quanji_marks").max(flag(ctx.core, owner, "quanji"));
                if cur_marks > 0 {
                    set_flag(ctx.core, owner, "quanji_marks", cur_marks - 1);
                    set_flag(ctx.core, owner, "quanji", cur_marks - 1);
                }
                let _ = tstamp;
                m.target = Some(t);
                ctx.start_draw(t, 2);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let t = m.target.unwrap_or(owner);
                if ctx.core.player(t).alive
                    && ctx.core.player(t).hand.len() > ctx.core.player(owner).hand.len()
                {
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn finish_xinfenyue_post_steal(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId, r1: u8) {
    if r1 <= 9 {
        if let Some(sha_c) = pop_card_from_deck_bottom_or_discard(ctx.core, |c| {
            ctx.catalog.card_kind(c).is_sha_like()
        }) {
            gain_hand_cards_from_pile_or_deck(ctx, owner, vec![sha_c]);
        }
    }
    if r1 <= 13 && ctx.core.player(target).alive && has_named_skill(ctx.core, ctx.catalog, owner, "xinfenyue") {
        helpers::emit_virtual_thunder_sha(ctx, owner, target);
    }
}

fn finish_xinkuangfu_check(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    t: PlayerId,
) -> Flow {
    let dealt = flag(ctx.core, owner, "l11b_xinkuangfu_dealt_dmg") > 0;
    set_flag(ctx.core, owner, "l11b_xinkuangfu_dealt_dmg", 0);
    if t == owner && dealt {
        ctx.start_draw(owner, 2);
        return Flow::Done;
    }
    if t != owner && !dealt && !ctx.core.player(owner).hand.is_empty() {
        return wait_ask(m, ctx, HostStage::CardsAsk, ASK_XINKUANGFU_DISCARD_TWO);
    }
    Flow::Done
}
