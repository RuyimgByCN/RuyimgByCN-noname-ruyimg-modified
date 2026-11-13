//! U08 英雄杀 Batch B 后半部分步骤状态机（`u08_batch_b_ext.rs`）。

use super::*;
use crate::catalog::CardKind;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(super) fn clear_heqin_pair(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let partner_raw = flag(ctx.core, owner, "heqin_partner");
    set_flag(ctx.core, owner, "heqin_partner", 0);
    set_flag(ctx.core, owner, "heqin2_active", 0);
    remove_skill_named(ctx, owner, "heqin2");
    remove_skill_named(ctx, owner, "heqin3");
    if partner_raw > 0 {
        let partner = (partner_raw - 1) as PlayerId;
        if (partner as usize) < ctx.core.players.len() {
            set_flag(ctx.core, partner, "heqin_partner", 0);
            set_flag(ctx.core, partner, "heqin2_active", 0);
            remove_skill_named(ctx, partner, "heqin2");
            remove_skill_named(ctx, partner, "heqin3");
        }
    }
}

pub(super) fn trigger_sanbanfu_counter(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: Option<PlayerId>,
) -> Flow {
    let recorded = flag(ctx.core, owner, "sanbanfu_counter_target");
    if recorded == 0 {
        return Flow::Done;
    }
    let t = target.unwrap_or((recorded - 1) as PlayerId);
    if recorded != u32::from(t) + 1 {
        return Flow::Done;
    }
    set_flag(ctx.core, owner, "sanbanfu_counter_target", 0);
    if ctx.core.player(t).alive && ctx.core.player(owner).alive {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(t),
            owner,
            1,
            DamageNature::Normal,
            None,
        )));
    }
    Flow::Done
}

pub(super) fn handle_u08_batch_b_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "sanbanfu" => match m.stage {
            HostStage::Act => {
                if m.card == Some(1) {
                    if flag(ctx.core, owner, "sanbanfu2_boost") > 0 {
                        set_flag(ctx.core, owner, "sanbanfu2_boost", 0);
                        with_current_damage_mut(ctx, |dm| {
                            dm.amount = dm.amount.saturating_add(1);
                        });
                    }
                    return Flow::Done;
                }
                if m.card == Some(2) {
                    return trigger_sanbanfu_counter(ctx, owner, m.target);
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_SANBANFU_CONFIRM)
            }
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let Some(t) = m.target else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if target_shan_cards(ctx.core, ctx.catalog, t).is_empty() {
                    let cid = m.cards.first().copied().unwrap_or(0);
                    set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{t}"), 1);
                    set_flag(ctx.core, owner, "sanbanfu2_boost", 1);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_SANBANFU_TARGET_SHAN,
                )
            }
            HostStage::CompareTargetPick => {
                m.stage = HostStage::Act;
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                let cid = m.cards.first().copied().unwrap_or(0);
                if let Some(shan_cid) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, t, &[shan_cid]);
                    ctx.emit(EventData::CardResponded {
                        card: shan_cid,
                        player: t,
                    });
                    set_flag(
                        ctx.core,
                        owner,
                        "sanbanfu_counter_target",
                        u32::from(t) + 1,
                    );
                } else {
                    set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{t}"), 1);
                    set_flag(ctx.core, owner, "sanbanfu2_boost", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "sanbanfu2" => trigger_sanbanfu_counter(ctx, owner, m.target),
        "sanbanfu3" => {
            if flag(ctx.core, owner, "sanbanfu2_boost") > 0 {
                set_flag(ctx.core, owner, "sanbanfu2_boost", 0);
                with_current_damage_mut(ctx, |dm| {
                    dm.amount = dm.amount.saturating_add(1);
                });
            }
            Flow::Done
        }
        "bingsheng" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_BINGSHENG_CARDS),
            HostStage::SelectCard => {
                let cards = answered_cards(ctx);
                if cards.len() != 2 {
                    refund_once_used(ctx, owner, "bingsheng");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let s0 = ctx.catalog.effective_card_suit(ctx.core, owner, cards[0]);
                let s1 = ctx.catalog.effective_card_suit(ctx.core, owner, cards[1]);
                if s0 == s1 {
                    refund_once_used(ctx, owner, "bingsheng");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                m.cards = cards;
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_BINGSHENG_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let Some(t) = answered_player(ctx) else {
                    refund_once_used(ctx, owner, "bingsheng");
                    return Flow::Done;
                };
                let cards = std::mem::take(&mut m.cards);
                discard_cards(ctx, owner, &cards);
                let diff = ctx.core.player(t).hp - ctx.core.player(owner).hp;
                if diff > 0 {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        diff.min(2),
                        DamageNature::Normal,
                        None,
                    )));
                } else if diff < 0 {
                    ctx.recover(t, (-diff).min(2) as u32, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "shentan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_SHENTAN_CARD),
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    refund_once_used(ctx, owner, "shentan");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_SHENTAN_TARGET)
            }
            HostStage::TargetChoice => {
                let (Some(&cid), Some(t)) = (m.cards.first(), answered_player(ctx)) else {
                    refund_once_used(ctx, owner, "shentan");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(t);
                discard_cards(ctx, owner, &[cid]);
                let thand = ctx.core.player(t).hand.clone();
                if thand.is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                if thand.len() == 1 {
                    let gained = thand[0];
                    move_cards_to_hand(ctx, t, owner, &[gained]);
                    m.cards = vec![gained];
                    if !alive_others(ctx.core, owner).is_empty() {
                        return wait_ask(m, ctx, HostStage::SelectNature, ASK_SHENTAN_GIVE_TARGET);
                    }
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_SHENTAN_GAIN_CARD,
                )
            }
            HostStage::CompareTargetPick => {
                let (Some(t), Some(gained)) = (m.target, answered_cards(ctx).into_iter().next())
                else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                move_cards_to_hand(ctx, t, owner, &[gained]);
                m.cards = vec![gained];
                if !alive_others(ctx.core, owner).is_empty() {
                    return wait_ask(m, ctx, HostStage::SelectNature, ASK_SHENTAN_GIVE_TARGET);
                }
                m.stage = HostStage::Act;
                Flow::Done
            }
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if let (Some(&gained), Some(dest)) = (m.cards.first(), answered_player(ctx)) {
                    if dest != owner && ctx.core.player(owner).hand.contains(&gained) {
                        move_cards_to_hand(ctx, owner, dest, &[gained]);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "biaoqi" => {
            if let Some(t) = m.target {
                let range = ctx.core.attack_range(ctx.catalog, owner) as i32;
                let thp = ctx.core.player(t).hp;
                if range > thp {
                    let cid = m.card.unwrap_or(0);
                    set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{t}"), 1);
                } else if range < thp {
                    draw_direct(ctx, owner, 1);
                }
            }
            Flow::Done
        }
        "wluoyan" => {
            with_current_damage_mut(ctx, |dm| {
                dm.prevented = true;
                dm.amount = 0;
                dm.settlement.outcome = crate::event::DamageOutcome::ReplacedByLoseHp;
            });
            ctx.emit(EventData::LoseHp {
                player: owner,
                amount: 1,
            });
            Flow::Done
        }
        "heqin" => match m.stage {
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let Some(t) = answered_player(ctx) else {
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "heqin_used", 1);
                set_flag(ctx.core, owner, "heqin_partner", u32::from(t) + 1);
                set_flag(ctx.core, t, "heqin_partner", u32::from(owner) + 1);
                set_flag(ctx.core, owner, "heqin2_active", 1);
                set_flag(ctx.core, t, "heqin2_active", 1);
                grant_skill_named(ctx, owner, "heqin2");
                grant_skill_named(ctx, owner, "heqin3");
                grant_skill_named(ctx, t, "heqin2");
                grant_skill_named(ctx, t, "heqin3");
                Flow::Done
            }
            HostStage::Act => {
                if m.card == Some(1) {
                    if let Some(p) = m.target {
                        draw_direct(ctx, p, 1);
                    }
                    Flow::Done
                } else if m.card == Some(2) {
                    clear_heqin_pair(ctx, owner);
                    Flow::Done
                } else {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_HEQIN_TARGET)
                }
            }
            _ => Flow::Done,
        },
        "heqin2" => {
            draw_direct(ctx, owner, 1);
            Flow::Done
        }
        "heqin3" => {
            clear_heqin_pair(ctx, owner);
            Flow::Done
        }
        "chajue" => {
            if m.card == Some(2) {
                set_flag(ctx.core, owner, "chajue2_active", 0);
                set_flag(ctx.core, owner, "yxs_chajue_immune", 0);
                remove_skill_named(ctx, owner, "chajue2");
            } else {
                set_flag(ctx.core, owner, "chajue2_active", 1);
                set_flag(ctx.core, owner, "yxs_chajue_immune", 1);
                grant_skill_named(ctx, owner, "chajue2");
            }
            Flow::Done
        }
        "tiewan" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_TIEWAN_CARD),
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_TIEWAN_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let (Some(&cid), Some(t)) = (m.cards.first(), answered_player(ctx)) else {
                    return Flow::Done;
                };
                if (ctx.core.player(owner).hand.contains(&cid)
                    || ctx.core.player(owner).equips.contains(&cid))
                    && ctx.core.player(t).alive
                {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: Some(cid),
                        player: owner,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Le),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => {
            m.stage = HostStage::Act;
            Flow::Done
        }
    }
}
