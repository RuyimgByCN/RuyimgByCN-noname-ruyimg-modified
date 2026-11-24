//! L11-6a 太虚自有 NPC 扩展牌堆/直选/咒雷/游击技能：`baolian`/`modao`/`tunshi`/`chuanyun`/`guihuo`/`luolei`/`guimen`/`dizhou`/`zhoulei`/`youji`。

use super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::event::{DamageNature, EventData, LostOrigin};

pub(super) fn handle_mark_expand(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_baolian" | "tx_modao" => {
            ctx.log(LogEvent::SkillTriggered {
                skill: if m.def_key == "txhj_baolian" { "暴敛" } else { "魔道" },
                owner,
                presentation: None,
            });
            ctx.start_draw(owner, 2);
            Some(Flow::Done)
        }
        "txhj_tunshi" => Some(handle_simple_target(m, ctx, ASK_TUNSHI_TARGET, "吞噬", 1, DamageNature::Normal)),
        "txhj_chuanyun" => Some(handle_simple_target(m, ctx, ASK_CHUANYUN_TARGET, "穿云", 1, DamageNature::Normal)),
        "txhj_guihuo" => Some(handle_simple_target(m, ctx, ASK_GUIHUO_TARGET, "鬼火", 1, DamageNature::Fire)),
        "txhj_luolei" => Some(handle_simple_target(m, ctx, ASK_LUOLEI_TARGET, "落雷", 1, DamageNature::Thunder)),
        "txhj_guimen" => Some(handle_guimen(m, ctx)),
        "txhj_dizhou" => Some(handle_dizhou(m, ctx)),
        "txhj_zhoulei" => {
            if let Some(t) = m.target.filter(|&p| p != owner && ctx.core.player(p).alive) {
                let mark = flag(ctx.core, owner, "l11_zhoulei_marks").min(5);
                let target_h = ctx.core.player(t).hand.len() as i32;
                let diff = (target_h - mark as i32).abs();
                if diff >= (5 - mark as i32) {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "咒雷",
                        owner,
                        presentation: None,
                    });
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
                    if mark < 5 {
                        set_flag(ctx.core, owner, "l11_zhoulei_marks", mark + 1);
                    }
                }
            }
            Some(Flow::Done)
        }
        "txqyyouji" => Some(handle_youji(m, ctx)),
        _ => None,
    }
}

fn handle_simple_target(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    ask_id: u32,
    zh_name: &'static str,
    amt: u32,
    nature: DamageNature,
) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ask_id),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| p != owner && ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: zh_name,
                owner,
                presentation: None,
            });
            emit_damage(ctx, Some(owner), t, amt, nature, None);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_guimen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let spades: Vec<CardId> = m
                .cards
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.resolved_card_suit(ctx.core, c) == Suit::Spade)
                .collect();
            m.cards = spades;
            if m.cards.is_empty() {
                return Flow::Done;
            }
            let first = m.cards.remove(0);
            set_flag(ctx.core, owner, "l11_guimen_cur_rank", u32::from(effective_card_rank(ctx.catalog, first)));
            m.stage = HostStage::SelectNature;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::SelectNature => {
            let j_rank = flag(ctx.core, owner, "last_judge_rank");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            set_flag(ctx.core, owner, "last_judge_card", 0);
            let card_rank = flag(ctx.core, owner, "l11_guimen_cur_rank");
            set_flag(ctx.core, owner, "l11_guimen_cur_rank", 0);
            if card_rank > 0 && j_rank > 0 && (card_rank as i32 - j_rank as i32).abs() <= 1 {
                let c = flag(ctx.core, owner, "l11_guimen_charges");
                set_flag(ctx.core, owner, "l11_guimen_charges", c + 1);
            }
            if !m.cards.is_empty() {
                let next_c = m.cards.remove(0);
                set_flag(ctx.core, owner, "l11_guimen_cur_rank", u32::from(effective_card_rank(ctx.catalog, next_c)));
                m.stage = HostStage::SelectNature;
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                return Flow::Yield;
            }
            let charges = flag(ctx.core, owner, "l11_guimen_charges");
            if charges == 0 {
                return Flow::Done;
            }
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_GUIMEN_TARGET)
        }
        HostStage::TargetChoice => {
            m.card = None;
            let charges = flag(ctx.core, owner, "l11_guimen_charges");
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| p != owner && ctx.core.player(p).alive) else {
                set_flag(ctx.core, owner, "l11_guimen_charges", 0);
                return Flow::Done;
            };
            if charges == 0 {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l11_guimen_charges", charges - 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "鬼门",
                owner,
                presentation: None,
            });
            emit_damage(ctx, Some(owner), t, 2, DamageNature::Thunder, None);
            if charges > 1 {
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            let charges = flag(ctx.core, owner, "l11_guimen_charges");
            if charges == 0 {
                return Flow::Done;
            }
            let res = wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_GUIMEN_TARGET);
            if matches!(res, Flow::Done) {
                set_flag(ctx.core, owner, "l11_guimen_charges", 0);
            }
            res
        }
        _ => Flow::Done,
    }
}

fn handle_dizhou(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(MODE_SHIELD) {
        if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(owner, "txhj_dizhou".to_string())) {
            if !pile.is_empty() {
                let idx = ctx.core.rng.gen_range(pile.len() as u32) as usize;
                let removed = pile.remove(idx);
                ctx.to_discard(removed);
                (ctx.catalog.hooks.on_cards_to_discard)(ctx, owner, &[removed], false);
            }
        }
        return Flow::Done;
    }
    if m.card == Some(MODE_SECONDARY) {
        let Some(user) = m.target.filter(|&p| ctx.core.player(p).alive) else {
            return Flow::Done;
        };
        if m.stage == HostStage::Act {
            ctx.log(LogEvent::SkillTriggered {
                skill: "地咒",
                owner,
                presentation: None,
            });
            m.stage = HostStage::SelectNature;
            ctx.emit(EventData::OrdinaryJudge {
                owner: user,
                invocation: u32::MAX,
            });
            return Flow::Yield;
        }
        if m.stage == HostStage::SelectNature {
            let suit = flag(ctx.core, user, "last_judge_suit");
            let rank = flag(ctx.core, user, "last_judge_rank");
            set_flag(ctx.core, user, "last_judge_suit", 0);
            set_flag(ctx.core, user, "last_judge_rank", 0);
            set_flag(ctx.core, user, "last_judge_card", 0);
            if matches!(suit, 1 | 3) {
                super::super::l11_common_b3::random_discard_he(ctx, user);
            }
            if suit == 1 {
                let mut discarded = None;
                if let Some(u) = current_use_card_mut(ctx) {
                    u.cancelled = true;
                    u.targets.clear();
                    discarded = u.card;
                }
                if let Some(cid) = discarded {
                    finish_table_card(ctx, cid);
                }
                if (2..=9).contains(&rank) && ctx.core.player(user).alive {
                    emit_lose_hp(ctx, user, 1);
                }
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_DIZHOU_CARD),
        HostStage::SelectCard => {
            m.card = None;
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let in_hand = ctx.core.player(owner).hand.contains(&cid);
            let in_equip = ctx.core.player(owner).equips.contains(&cid);
            if !in_hand && !in_equip {
                return Flow::Done;
            }
            ctx.take_owned_card(owner, cid);
            if in_equip {
                ctx.core.declared_slots.remove(&cid);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, cid);
            }
            ctx.core
                .vm_aux
                .piles
                .entry((owner, "txhj_dizhou".to_string()))
                .or_default()
                .push(cid);
            ctx.emit(EventData::CardsLost {
                player: owner,
                cards: vec![cid],
                origins: vec![if in_hand { LostOrigin::Hand } else { LostOrigin::Equip }],
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_youji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::CardsAsk, ASK_YOUJI_CONFIRM),
        HostStage::CardsAsk => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let Some(fid) = current_use_frame_id(ctx) else {
                return Flow::Done;
            };
            let catalog = ctx.catalog;
            let eff_kind = current_use_card_mut(ctx).and_then(|u| effective_kind_of_use(catalog, u));
            let dup = current_use_card_mut(ctx).and_then(|u| {
                if u.cancelled || !is_youji_eligible_use(catalog, u) {
                    return None;
                }
                if u.targets.len() == 1 {
                    let t = u.targets[0];
                    u.targets.push(t);
                    Some(false)
                } else if u.targets.is_empty() && eff_kind == Some(CardKind::Wuzhong) {
                    Some(true)
                } else {
                    None
                }
            });
            if let Some(is_self_wuzhong) = dup {
                let flag_key = if is_self_wuzhong {
                    format!("card_use_extra_effect:{fid}")
                } else {
                    format!("txqyyouji_dup:{fid}")
                };
                set_flag(ctx.core, owner, &flag_key, 1);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "游击",
                    owner,
                    presentation: None,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(in crate::content::host) fn is_youji_eligible_use(
    catalog: &Catalog,
    u: &crate::event::CardUseData,
) -> bool {
    let Some(k) = effective_kind_of_use(catalog, u) else {
        return false;
    };
    if u.targets.is_empty() {
        // Engine-native `Wuzhong` uses empty targets with implicit self-target.
        let is_plain_wuzhong = k == CardKind::Wuzhong
            && u.card.is_none_or(|cid| {
                catalog.cards.get(cid as usize).is_some_and(|cd| {
                    cd.name == "wuzhong"
                        || cd.name == "无中生有"
                        || cd.name.starts_with("wuzhong_")
                })
            });
        return is_plain_wuzhong;
    }
    if !is_single_target_trick_use(catalog, u) {
        return false;
    }
    super::super::l11_common_b2::is_ordinary_trick_kind(k)
        && k.aoe_need().is_none()
        && !matches!(k, CardKind::Taoyuan | CardKind::Wugu | CardKind::Jiedao | CardKind::Chain)
}
