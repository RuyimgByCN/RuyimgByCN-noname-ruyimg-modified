//! Combat, damage, and card-use triggered skills for `HostBehavior::L11VmFixC`.

use super::filter::{damage_root_is_sha, mieji_extra_targets, qiaoshui3_add_targets};
use super::helpers::*;
use super::*;
use crate::catalog::{CardKind, Suit};
use crate::choice::ChoiceOption;
use crate::event::EventData;
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn step_combat(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let tstamp = turn_stamp(ctx.core);
    match m.def_key.as_str() {
        "anjian" => {
            let mut boosted = false;
            if let Some(f) = ctx
                .res
                .stack
                .iter_mut()
                .rev()
                .find(|f| matches!(f.data, EventData::Damage(_)))
            {
                let in_range = matches!(f.data, EventData::Damage(ref d)
                    if ctx.core.in_attack_range(ctx.catalog, d.target, owner));
                if let EventData::Damage(ref mut d) = f.data {
                    if d.source == Some(owner)
                        && !d.prevented
                        && d.amount > 0
                        && !d.from_chain
                        && !in_range
                    {
                        d.amount += 1;
                        boosted = true;
                    }
                }
            }
            if boosted {
                log_skill(ctx, owner, "anjian");
            }
            Flow::Done
        }
        "baonu" => {
            let add = if m.cards.is_empty() {
                if flag(ctx.core, owner, "l11c_baonu_init") == 0 {
                    set_flag(ctx.core, owner, "l11c_baonu_init", 1);
                    2
                } else {
                    0
                }
            } else {
                m.cards.first().copied().unwrap_or(0)
            };
            if add > 0 {
                log_skill(ctx, owner, "baonu");
                let cur = flag(ctx.core, owner, "l11c_baonu_marks");
                sync_mark_count(ctx.core, owner, "baonu", cur + add);
            }
            Flow::Done
        }
        "chongzhen" => match m.stage {
            HostStage::Act => {
                let Some(cp) = find_chongzhen_counterpart(ctx, owner, m) else {
                    return Flow::Done;
                };
                if ctx.core.player(cp).hand.is_empty() {
                    return Flow::Done;
                }
                m.target = Some(cp);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_CHONGZHEN_CONFIRM)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let cp = m.target.unwrap_or(owner);
                if !ctx.core.player(cp).alive || ctx.core.player(cp).hand.is_empty() {
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_CHONGZHEN_CARD)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let cp = m.target.unwrap_or(owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    log_skill(ctx, owner, "chongzhen");
                    move_cards_to_hand(ctx, cp, owner, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mieji" => match m.stage {
            HostStage::Act => {
                let extra = {
                    let Ok(u) = ctx.current_card_use() else {
                        return Flow::Done;
                    };
                    mieji_extra_targets(ctx.core, ctx.catalog, owner, u)
                };
                if extra.is_empty() {
                    return Flow::Done;
                }
                m.cards = extra.into_iter().map(u32::from).collect();
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_MIEJI_TARGET)
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if let Some(f) = ctx
                        .res
                        .stack
                        .iter_mut()
                        .rev()
                        .find(|f| matches!(f.data, EventData::CardUsed(_)))
                    {
                        if let EventData::CardUsed(ref mut u) = f.data {
                            if !u.targets.contains(&t) {
                                u.targets.push(t);
                            }
                        }
                    }
                    log_skill(ctx, owner, "mieji");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qiaoshui3" => match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "l11c_qiaoshui3_turn", 0);
                remove_named_skill(ctx, owner, "qiaoshui3");
                let (add_ts, cur_targets) = {
                    let Ok(u) = ctx.current_card_use() else {
                        return Flow::Done;
                    };
                    (
                        qiaoshui3_add_targets(ctx.core, ctx.catalog, owner, u),
                        u.targets.clone(),
                    )
                };
                if !add_ts.is_empty() {
                    m.cards = add_ts.into_iter().map(u32::from).collect();
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIAOSHUI3_ADD_TARGET)
                } else if cur_targets.len() > 1 {
                    m.cards = cur_targets.into_iter().map(u32::from).collect();
                    wait_ask(m, ctx, HostStage::SelectNature, ASK_QIAOSHUI3_REMOVE_TARGET)
                } else {
                    Flow::Done
                }
            }
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(add_t)) = ctx.answer().first().copied() {
                    if let Some(f) = ctx
                        .res
                        .stack
                        .iter_mut()
                        .rev()
                        .find(|f| matches!(f.data, EventData::CardUsed(_)))
                    {
                        if let EventData::CardUsed(ref mut u) = f.data {
                            if !u.targets.contains(&add_t) {
                                u.targets.push(add_t);
                            }
                        }
                    }
                    log_skill(ctx, owner, "qiaoshui3");
                    return Flow::Done;
                }
                let cur_targets = ctx.current_card_use()
                    .map(|u| u.targets.clone())
                    .unwrap_or_default();
                if cur_targets.len() > 1 {
                    m.cards = cur_targets.into_iter().map(u32::from).collect();
                    wait_ask(m, ctx, HostStage::SelectNature, ASK_QIAOSHUI3_REMOVE_TARGET)
                } else {
                    Flow::Done
                }
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(rem_t)) = ctx.answer().first().copied() {
                    if let Some(f) = ctx
                        .res
                        .stack
                        .iter_mut()
                        .rev()
                        .find(|f| matches!(f.data, EventData::CardUsed(_)))
                    {
                        if let EventData::CardUsed(ref mut u) = f.data {
                            u.targets.retain(|&p| p != rem_t);
                        }
                    }
                    log_skill(ctx, owner, "qiaoshui3");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "ranshang" => {
            let add = m.cards.first().copied().unwrap_or(0);
            if add > 0 {
                log_skill(ctx, owner, "ranshang");
                let cur = flag(ctx.core, owner, "l11c_ranshang_marks");
                sync_mark_count(ctx.core, owner, "ranshang", cur + add);
                ensure_perm_skill(ctx, owner, "ranshang2");
            }
            Flow::Done
        }
        "rejizhi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_REJIZHI_CONFIRM),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                log_skill(ctx, owner, "rejizhi");
                m.cards = ctx.core.player(owner).hand.clone();
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 1);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if !ctx.core.player(owner).alive {
                    return Flow::Done;
                }
                let new_card = ctx
                    .core
                    .player(owner)
                    .hand
                    .iter()
                    .rev()
                    .copied()
                    .find(|c| !m.cards.contains(c));
                if let Some(c) = new_card {
                    if ctx.catalog.resolved_card_kind(ctx.core, c).is_basic_kind() {
                        m.card = Some(c);
                        return wait_ask(m, ctx, HostStage::SelectCard, ASK_REJIZHI_DISCARD);
                    }
                }
                Flow::Done
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, owner, &[c]);
                    if flag(ctx.core, owner, "l11c_rejizhi_hand_turn") != tstamp {
                        set_flag(ctx.core, owner, "l11c_rejizhi_hand_turn", tstamp);
                        set_flag(ctx.core, owner, "l11c_rejizhi_hand_bonus", 1);
                    } else {
                        let prev = flag(ctx.core, owner, "l11c_rejizhi_hand_bonus");
                        set_flag(ctx.core, owner, "l11c_rejizhi_hand_bonus", prev + 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "retieji" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_RETIEJI_CONFIRM),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                let t = m.target.unwrap_or(owner);
                log_skill(ctx, owner, "retieji");
                set_flag(ctx.core, t, "l11c_retieji_sealed", 1);
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let t = m.target.unwrap_or(owner);
                let suit_code = flag(ctx.core, owner, "last_judge_suit");
                set_flag(ctx.core, owner, "last_judge_suit", 0);
                set_flag(ctx.core, owner, "last_judge_rank", 0);
                set_flag(ctx.core, owner, "last_judge_card", 0);
                let suit = match suit_code {
                    1 => Suit::Spade,
                    2 => Suit::Heart,
                    3 => Suit::Club,
                    _ => Suit::Diamond,
                };
                let sbit = suit_bit(suit);
                let sha_cid = m.card.unwrap_or(0);
                let can_disc = ctx.core.player(t).alive
                    && he_cards(ctx.core, t)
                        .into_iter()
                        .any(|c| suit_bit(card_suit(ctx.core, ctx.catalog, t, c)) == sbit);
                if can_disc {
                    m.card = Some(sbit);
                    m.cards = vec![sha_cid];
                    wait_ask(m, ctx, HostStage::SelectCard, ASK_RETIEJI_DISCARD)
                } else {
                    set_flag(
                        ctx.core,
                        owner,
                        &format!("sha_unrespondable:{sha_cid}:{t}"),
                        1,
                    );
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let sha_cid = m.cards.first().copied().unwrap_or(0);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    discard_player_cards(ctx, t, &[c]);
                } else {
                    set_flag(
                        ctx.core,
                        owner,
                        &format!("sha_unrespondable:{sha_cid}:{t}"),
                        1,
                    );
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "rezhenxing" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_REZHENXING_CONFIRM),
            HostStage::TargetChoice => {
                clear_ask(ctx, owner);
                if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    return Flow::Done;
                }
                log_skill(ctx, owner, "rezhenxing");
                let mut top3 = Vec::new();
                for _ in 0..3 {
                    if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
                        ctx.reshuffle_discard_into_deck();
                    }
                    if let Some(c) = ctx.core.deck.pop() {
                        top3.push(c);
                    }
                }
                if top3.is_empty() {
                    return Flow::Done;
                }
                m.cards = top3;
                wait_ask(m, ctx, HostStage::SelectCard, ASK_REZHENXING_PICK)
            }
            HostStage::SelectCard => {
                clear_ask(ctx, owner);
                let picked = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Card(c)) => Some(c),
                    _ => None,
                };
                if let Some(c) = picked {
                    if ctx.core.player(owner).alive {
                        ctx.core.player_mut(owner).hand.push(c);
                        ctx.log(LogEvent::CardsDrawn {
                            player: owner,
                            cards: vec![c],
                        });
                    } else {
                        ctx.to_discard(c);
                    }
                }
                for &c in m.cards.iter().rev() {
                    if Some(c) != picked {
                        ctx.core.deck.push(c);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shibei" => {
            log_skill(ctx, owner, "shibei");
            let cnt = if flag(ctx.core, owner, "l11c_turn_dmg_taken_stamp") == tstamp {
                flag(ctx.core, owner, "l11c_turn_dmg_taken_count")
            } else {
                0
            };
            if cnt == 0 {
                if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    ctx.recover(owner, 1, None);
                }
            } else {
                emit_lose_hp(ctx, owner, 1);
            }
            Flow::Done
        }
        "twkujian_draw" => {
            let user = m.target.unwrap_or(owner);
            log_skill(ctx, owner, "twkujian");
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
            }
            if user != owner && ctx.core.player(user).alive {
                ctx.start_draw(user, 1);
            }
            Flow::Done
        }
        "xinanjian" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_XINANJIAN_MODE),
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let t = m.target.unwrap_or(owner);
                let cid = m.card.unwrap_or(0);
                let mode = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(tag)) => tag,
                    _ => 0,
                };
                log_skill(ctx, owner, "xinanjian");
                if mode == 0 {
                    set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{t}"), 1);
                } else {
                    let cur = flag(ctx.core, owner, &format!("sha_target_damage_bonus:{cid}:{t}"));
                    set_flag(
                        ctx.core,
                        owner,
                        &format!("sha_target_damage_bonus:{cid}:{t}"),
                        cur + 1,
                    );
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xinkuanggu" => match m.stage {
            HostStage::Act => {
                let rem = m.cards.first().copied().unwrap_or(1).max(1);
                m.card = Some(rem);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_XINKUANGGU_MODE)
            }
            HostStage::SelectNature => {
                clear_ask(ctx, owner);
                let rem = m.card.unwrap_or(1);
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(0)) => {
                        log_skill(ctx, owner, "xinkuanggu");
                        ctx.start_draw(owner, 1);
                    }
                    Some(ChoiceOption::Tag(1)) => {
                        log_skill(ctx, owner, "xinkuanggu");
                        if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                            ctx.recover(owner, 1, None);
                        }
                    }
                    _ => return Flow::Done,
                }
                if rem > 1 && has_live_l11c_skill(ctx.core, ctx.catalog, owner, "xinkuanggu", false) {
                    m.card = Some(rem - 1);
                    m.stage = HostStage::DrawSettle;
                    Flow::Yield
                } else {
                    Flow::Done
                }
            }
            HostStage::DrawSettle => {
                if has_live_l11c_skill(ctx.core, ctx.catalog, owner, "xinkuanggu", false) {
                    wait_ask(m, ctx, HostStage::SelectNature, ASK_XINKUANGGU_MODE)
                } else {
                    Flow::Done
                }
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}

fn find_chongzhen_counterpart(
    ctx: &Ctx<'_>,
    owner: PlayerId,
    m: &HostMachine,
) -> Option<PlayerId> {
    if let Ok(u) = ctx.current_card_use() {
        if u.player == owner {
            let eff = u.effective_kind(ctx.catalog);
            if eff.is_sha_like() {
                return u.targets.first().copied().filter(|&t| t != owner && ctx.core.player(t).alive);
            }
            if eff == CardKind::Shan {
                return super::filter::chongzhen_counterpart_on_use(
                    ctx.core,
                    ctx.catalog,
                    &ctx.res.stack,
                    owner,
                    u,
                );
            }
        }
    }
    if let Some(c) = m.card {
        return super::filter::chongzhen_counterpart_on_respond(
            ctx.core,
            ctx.catalog,
            &ctx.res.stack,
            owner,
            c,
        );
    }
    m.targets.first().copied().filter(|&t| t != owner && ctx.core.player(t).alive)
}

#[allow(dead_code)]
fn _unused_dmg_check(frames: &[crate::event::Frame], d: &crate::event::DamageData, catalog: &crate::catalog::Catalog) -> bool {
    damage_root_is_sha(frames, d, catalog)
}
