use super::super::super::*;
use super::super::l11_common::{
    alive_all, discard_player_cards, emit_damage, emit_lose_hp, flag, he_cards, set_flag,
};
use super::super::l11_common_b2::emit_virtual_kind_use;
use super::helpers::*;
use crate::catalog::{CardKind, EquipSlot, Suit};
use crate::event::{DamageNature, EventData};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "wurong" => step_wurong(m, ctx, owner),
        "yeyan" => step_yeyan(m, ctx, owner),
        "qiangwu" => step_qiangwu(m, ctx, owner),
        "drlt_xiongluan" => step_xiongluan(m, ctx, owner),
        "mingce" => step_mingce(m, ctx, owner),
        "nsyangwu" => step_nsyangwu(m, ctx, owner),
        _ => Flow::Done,
    }
}

fn step_wurong(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_WURONG_TARGET),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive && !ctx.core.player(p).hand.is_empty())
            else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_WURONG_CARD)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(c1) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|&c| ctx.core.player(owner).hand.contains(&c))
            else {
                return Flow::Done;
            };
            let Some(_t) = m
                .target
                .filter(|&p| ctx.core.player(p).alive && !ctx.core.player(p).hand.is_empty())
            else {
                return Flow::Done;
            };
            m.card = Some(c1);
            wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_WURONG_CARD)
        }
        HostStage::CompareTargetPick => {
            clear_ask(ctx, owner);
            let Some(c2) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let (Some(c1), Some(t)) = (m.card, m.target) else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).alive
                || !ctx.core.player(t).alive
                || !ctx.core.player(owner).hand.contains(&c1)
                || !ctx.core.player(t).hand.contains(&c2)
            {
                return Flow::Done;
            }
            mark_once_used(ctx, owner, "wurong");
            ctx.log(LogEvent::SkillTriggered {
                skill: "wurong",
                owner,
                presentation: None,
            });
            let viewers = alive_all(ctx.core, owner);
            ctx.log(LogEvent::CardsRevealed {
                cards: vec![c1],
                to: viewers.clone(),
            });
            ctx.log(LogEvent::CardsRevealed {
                cards: vec![c2],
                to: viewers,
            });
            let is_sha = ctx.catalog.resolved_card_kind(ctx.core, c1).is_sha_like();
            let is_shan = ctx.catalog.resolved_card_kind(ctx.core, c2) == CardKind::Shan;
            if is_sha && !is_shan {
                discard_player_cards(ctx, owner, &[c1]);
                m.stage = HostStage::ShuchuiTargetLoop;
                Flow::Yield
            } else if !is_sha && is_shan {
                discard_player_cards(ctx, owner, &[c1]);
                m.stage = HostStage::JiefenOwnerDiscard;
                Flow::Yield
            } else {
                Flow::Done
            }
        }
        HostStage::ShuchuiTargetLoop => {
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::JiefenOwnerDiscard => {
            let Some(_t) = m
                .target
                .filter(|&p| ctx.core.player(p).alive && !he_cards(ctx.core, p).is_empty())
            else {
                return Flow::Done;
            };
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_WURONG_GAIN)
        }
        HostStage::ComparePlayerPick => {
            clear_ask(ctx, owner);
            if let (Some(t), Some(g)) = (m.target, answered_cards(ctx).into_iter().next()) {
                give_he_cards_to_hand(ctx, t, owner, &[g]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yeyan(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let n = ctx.core.players.len();
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YEYAN_MODE),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            m.cards.clear();
            match answered_tag(ctx) {
                Some(0) => wait_ask(m, ctx, HostStage::SelectNature, ASK_YEYAN_SMALL_TARGETS),
                Some(1) if has_four_distinct_suits_in_hand(ctx.core, ctx.catalog, owner) => {
                    wait_ask(m, ctx, HostStage::SelectCard, ASK_YEYAN_CARD)
                }
                _ => Flow::Done,
            }
        }
        HostStage::SelectNature => {
            clear_ask(ctx, owner);
            let mut ts = answered_players(ctx);
            ts.retain(|&p| ctx.core.player(p).alive);
            if ts.is_empty() {
                return Flow::Done;
            }
            sort_by_seat_from(&mut ts, owner, n);
            mark_once_used(ctx, owner, "yeyan");
            ctx.log(LogEvent::SkillTriggered {
                skill: "yeyan",
                owner,
                presentation: None,
            });
            m.targets = ts;
            m.card = None;
            m.stage = HostStage::ShuchuiTargetLoop;
            step_yeyan_next_dmg(m, ctx, owner)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(picked) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|c| !m.cards.contains(c) && ctx.core.player(owner).hand.contains(c))
            else {
                return Flow::Done;
            };
            m.cards.push(picked);
            if m.cards.len() < 4 {
                wait_ask(m, ctx, HostStage::SelectCard, ASK_YEYAN_CARD)
            } else {
                wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_YEYAN_GREAT_TARGETS)
            }
        }
        HostStage::CompareTargetPick => {
            clear_ask(ctx, owner);
            let mut ts = answered_players(ctx);
            ts.retain(|&p| ctx.core.player(p).alive);
            if ts.is_empty() || ts.len() > 2 {
                return Flow::Done;
            }
            sort_by_seat_from(&mut ts, owner, n);
            m.targets = ts;
            if m.targets.len() == 1 {
                wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_YEYAN_DMG_AMOUNT)
            } else {
                wait_ask(m, ctx, HostStage::CardsAsk, ASK_YEYAN_2DMG_TARGET)
            }
        }
        HostStage::ComparePlayerPick => {
            clear_ask(ctx, owner);
            let amt = match answered_tag(ctx) {
                Some(2) => 2,
                Some(3) => 3,
                _ => return Flow::Done,
            };
            m.card = Some(amt);
            commit_great_yeyan_cost(m, ctx, owner)
        }
        HostStage::CardsAsk => {
            clear_ask(ctx, owner);
            let Some(two_t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|p| m.targets.contains(p))
            else {
                return Flow::Done;
            };
            m.card = Some(100 + u32::from(two_t));
            commit_great_yeyan_cost(m, ctx, owner)
        }
        HostStage::JiefenOwnerDiscard => {
            emit_lose_hp(ctx, owner, 3);
            m.stage = HostStage::ShuchuiTargetLoop;
            Flow::Yield
        }
        HostStage::ShuchuiTargetLoop => step_yeyan_next_dmg(m, ctx, owner),
        _ => Flow::Done,
    }
}

fn commit_great_yeyan_cost(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let owned = &ctx.core.player(owner).hand;
    if m.cards.len() != 4 || !m.cards.iter().all(|c| owned.contains(c)) {
        return Flow::Done;
    }
    mark_once_used(ctx, owner, "yeyan");
    ctx.log(LogEvent::SkillTriggered {
        skill: "yeyan",
        owner,
        presentation: None,
    });
    discard_player_cards(ctx, owner, &m.cards);
    m.cards.clear();
    m.stage = HostStage::JiefenOwnerDiscard;
    Flow::Yield
}

fn step_yeyan_next_dmg(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if !ctx.core.player(t).alive {
            continue;
        }
        let dmg = match m.card {
            None => 1,
            Some(v) if v >= 100 => {
                if u32::from(t) == v - 100 { 2 } else { 1 }
            }
            Some(v) => v,
        };
        let src = ctx.core.player(owner).alive.then_some(owner);
        emit_damage(ctx, src, t, dmg, DamageNature::Fire, None);
        return Flow::Yield;
    }
    Flow::Done
}

fn step_qiangwu(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                return Flow::Done;
            }
            mark_once_used(ctx, owner, "qiangwu");
            ctx.log(LogEvent::SkillTriggered {
                skill: "qiangwu",
                owner,
                presentation: None,
            });
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "last_judge_card".to_string()));
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(jc) = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "last_judge_card".to_string()))
            {
                let rank = ctx.catalog.cards.get(jc as usize).map_or(0, |cd| u32::from(cd.rank));
                set_flag(ctx.core, owner, "l11_qiangwu_active", 1);
                set_flag(ctx.core, owner, "l11_qiangwu_rank", rank);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_xiongluan(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_XIONGLUAN_TARGET),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive)
            else {
                return Flow::Done;
            };
            mark_once_used(ctx, owner, "drlt_xiongluan");
            ctx.log(LogEvent::SkillTriggered {
                skill: "drlt_xiongluan",
                owner,
                presentation: None,
            });
            set_flag(ctx.core, owner, "l11_judge_slot_disabled", 1);
            discard_judge_cards(ctx, owner);
            let slots: [(u8, EquipSlot); 5] = [
                (1, EquipSlot::Weapon),
                (2, EquipSlot::Armor),
                (3, EquipSlot::HorsePlus),
                (4, EquipSlot::HorseMinus),
                (5, EquipSlot::Treasure),
            ];
            let mut to_disc = Vec::new();
            let mut mask = flag(ctx.core, owner, "disabled_equip_slots");
            for (code, slot) in slots {
                if !ctx.catalog.is_equip_slot_enabled(ctx.core, owner, slot) {
                    continue;
                }
                mask |= 1u32 << code;
                set_flag(ctx.core, owner, &format!("equip_slot_disabled:{code}"), 1);
                for &c in &ctx.core.player(owner).equips {
                    if ctx.catalog.card_slot_of(ctx.core, c) == Some(slot) && !to_disc.contains(&c)
                    {
                        to_disc.push(c);
                    }
                }
            }
            set_flag(ctx.core, owner, "disabled_equip_slots", mask);
            if !to_disc.is_empty() {
                discard_player_cards(ctx, owner, &to_disc);
            }
            set_flag(ctx.core, owner, "l11_xiongluan1_active", 1);
            set_flag(ctx.core, owner, "l11_xiongluan_target", u32::from(t) + 1);
            let cur_ban = flag(ctx.core, t, "l11_xiongluan2_hand_ban");
            set_flag(ctx.core, t, "l11_xiongluan2_hand_ban", cur_ban + 1);
            set_flag(ctx.core, t, &format!("l11_xiongluan2_from:{owner}"), 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_mingce(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_MINGCE_CARD),
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let eligible = mingce_eligible_cards(ctx.core, ctx.catalog, owner);
            let Some(c) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|cid| eligible.contains(cid))
            else {
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_MINGCE_GIVE_TARGET)
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            let Some(t1) = answered_players(ctx).into_iter().next().filter(|&p| {
                is_mingce_give_candidate(ctx.core, ctx.catalog, owner, p)
            }) else {
                return Flow::Done;
            };
            m.target = Some(t1);
            wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_MINGCE_SHA_TARGET)
        }
        HostStage::CompareTargetPick => {
            clear_ask(ctx, owner);
            let (Some(c), Some(t1)) = (m.card, m.target) else {
                return Flow::Done;
            };
            let Some(t2) = answered_players(ctx).into_iter().next().filter(|&p| {
                p != t1 && ctx.core.player(p).alive && ctx.core.in_attack_range(ctx.catalog, t1, p)
            }) else {
                return Flow::Done;
            };
            if !mingce_eligible_cards(ctx.core, ctx.catalog, owner).contains(&c)
                || !ctx.core.player(t1).alive
            {
                return Flow::Done;
            }
            mark_once_used(ctx, owner, "mingce");
            ctx.log(LogEvent::SkillTriggered {
                skill: "mingce",
                owner,
                presentation: None,
            });
            m.targets = vec![t2];
            give_he_cards_to_hand(ctx, owner, t1, &[c]);
            m.stage = HostStage::SelectNature;
            Flow::Yield
        }
        HostStage::SelectNature => {
            let Some(t1) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let can_sha = m
                .targets
                .first()
                .copied()
                .is_some_and(|t2| can_virtual_sha_target(ctx.core, ctx.catalog, t1, t2, false));
            if !can_sha {
                ctx.start_draw(t1, 1);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_MINGCE_CHOICE)
        }
        HostStage::ComparePlayerPick => {
            clear_ask(ctx, owner);
            let Some(t1) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let tag = answered_tag(ctx);
            if tag == Some(1) {
                if let Some(t2) = m
                    .targets
                    .first()
                    .copied()
                    .filter(|&p| can_virtual_sha_target(ctx.core, ctx.catalog, t1, p, false))
                {
                    emit_virtual_kind_use(ctx, t1, CardKind::Sha, vec![t2], DamageNature::Normal);
                    m.stage = HostStage::DrawSettle;
                    return Flow::Yield;
                }
            }
            ctx.start_draw(t1, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_nsyangwu(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_NSYANGWU_TARGET),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            let my_len = ctx.core.player(owner).hand.len();
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| {
                p != owner && ctx.core.player(p).alive && ctx.core.player(p).hand.len() > my_len
            }) else {
                return Flow::Done;
            };
            m.target = Some(t);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_NSYANGWU_CARD)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(c) = answered_cards(ctx).into_iter().next().filter(|&cid| {
                ctx.core.player(owner).hand.contains(&cid)
                    && ctx.catalog.effective_card_suit(ctx.core, owner, cid) == Suit::Heart
            }) else {
                return Flow::Done;
            };
            let Some(_t) = m.target.filter(|&p| {
                p != owner
                    && ctx.core.player(p).alive
                    && ctx.core.player(p).hand.len() > ctx.core.player(owner).hand.len()
            }) else {
                return Flow::Done;
            };
            mark_once_used(ctx, owner, "nsyangwu");
            ctx.log(LogEvent::SkillTriggered {
                skill: "nsyangwu",
                owner,
                presentation: None,
            });
            discard_player_cards(ctx, owner, &[c]);
            m.stage = HostStage::JiefenOwnerDiscard;
            Flow::Yield
        }
        HostStage::JiefenOwnerDiscard => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let th = ctx.core.player(t).hand.len();
            let oh = ctx.core.player(owner).hand.len();
            let num = th.saturating_sub(oh).div_ceil(2).min(th);
            if num == 0 {
                return Flow::Done;
            }
            m.card = Some(num as u32);
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_NSYANGWU_GAIN)
        }
        HostStage::ComparePlayerPick => {
            clear_ask(ctx, owner);
            let picks = answered_cards(ctx);
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                if !picks.is_empty() {
                    give_he_cards_to_hand(ctx, t, owner, &picks);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
