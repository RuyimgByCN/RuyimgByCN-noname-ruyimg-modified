//! U08 英雄杀 Batch D 后半部分步骤状态机（`u08_batch_d_ext.rs`）。

use super::*;
use crate::catalog::{CardKind, EquipSlot};
use crate::event::{CardUseData, DamageNature, EventData, LostOrigin};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

fn emit_miaobix_virtual_use(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    use_cid: CardId,
    copy_kind: CardKind,
    chosen_target: Option<PlayerId>,
) {
    let targets = if let Some(t) = chosen_target {
        vec![t]
    } else {
        match copy_kind {
            CardKind::Wuzhong | CardKind::Tao | CardKind::Jiu => vec![owner],
            CardKind::Wugu | CardKind::Taoyuan => alive_all(ctx.core, owner),
            CardKind::SavageAttack | CardKind::ArcheryAttack => alive_others(ctx.core, owner),
            _ => Vec::new(),
        }
    };
    let nature = match copy_kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    ctx.core.declared_kinds.insert(use_cid, copy_kind);
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(use_cid),
        player: owner,
        targets,
        nature,
        use_as: Some(copy_kind),
        cancelled: false,
    }));
}

fn guifu_draw_both(ctx: &mut Ctx<'_>, owner: PlayerId, target: PlayerId) {
    for p in ctx.core.seat_order_from(ctx.core.turn.current) {
        if (p == owner || p == target) && ctx.core.player(p).alive {
            draw_direct(ctx, p, 1);
        }
    }
}

fn equip_handcard_as_template(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    dest: PlayerId,
    hand_cid: CardId,
    template_eq: CardId,
) {
    if !ctx.core.player(owner).hand.contains(&hand_cid) || !ctx.core.player(dest).alive {
        return;
    }
    let template_kind = ctx
        .core
        .declared_kinds
        .get(&template_eq)
        .copied()
        .unwrap_or_else(|| ctx.catalog.card_kind(template_eq));
    let template_slot = ctx
        .catalog
        .card_slot_of(ctx.core, template_eq)
        .or_else(|| template_kind.equip_slot())
        .unwrap_or(EquipSlot::Weapon);

    ctx.take_hand_card(owner, hand_cid);
    ctx.note_cards_lost(owner, &[hand_cid]);
    ctx.emit(EventData::CardsLost {
        player: owner,
        cards: vec![hand_cid],
        origins: vec![LostOrigin::Hand],
    });

    ctx.core.declared_kinds.insert(hand_cid, template_kind);
    ctx.core.declared_slots.insert(hand_cid, template_slot);

    let replaced = ctx
        .core
        .player(dest)
        .equips
        .iter()
        .copied()
        .find(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(template_slot));
    if let Some(old) = replaced {
        ctx.core.player_mut(dest).equips.retain(|&c| c != old);
        ctx.core.declared_slots.remove(&old);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, dest, old);
        ctx.note_cards_lost(dest, &[old]);
        ctx.to_discard(Some(old));
        ctx.emit(EventData::CardsLost {
            player: dest,
            cards: vec![old],
            origins: vec![LostOrigin::Equip],
        });
    }

    ctx.core.player_mut(dest).equips.push(hand_cid);
    crate::kernel::grant_equip_skills(ctx, dest, template_eq);
    ctx.log(LogEvent::Equipped {
        player: dest,
        card: hand_cid,
        replaced,
    });
}

pub(super) fn handle_u08_batch_d_ext(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "yxsweiyi" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_YXSWEIYI_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let Some(src) = m.target else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if !ctx.core.player(src).alive {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let he = hand_and_equips(ctx.core, src);
                if he.is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                if he.len() <= 2 {
                    discard_cards(ctx, src, &he);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(
                    m,
                    ctx,
                    HostStage::SelectCard,
                    ASK_YXSWEIYI_SOURCE_DISCARD,
                )
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(src) = m.target {
                    let chosen = answered_cards(ctx);
                    if !chosen.is_empty() {
                        discard_cards(ctx, src, &chosen);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qiandu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_QIANDU_CARD),
            HostStage::SelectCard => {
                let Some(cid) = answered_cards(ctx).into_iter().next() else {
                    refund_once_used(ctx, owner, "qiandu");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIANDU_TARGET)
            }
            HostStage::TargetChoice => {
                m.stage = HostStage::Act;
                let (Some(&cid), Some(t)) = (m.cards.first(), answered_player(ctx)) else {
                    refund_once_used(ctx, owner, "qiandu");
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &[cid]);
                ctx.core.swap_seat(owner, t);
                set_flag(ctx.core, owner, "yxs_qiandu_seat_swapped", 1);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "nvquan" | "nvquan1" | "nvquan2" => {
            let is_duel = m.def_key == "nvquan2"
                || (m.def_key == "nvquan"
                    && ctx
                        .current_card_use()
                        .ok()
                        .map(|u| u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)))
                        == Some(CardKind::Duel));
            if is_duel {
                match m.stage {
                    HostStage::Act => {
                        let Some(t) = m
                            .targets
                            .iter()
                            .copied()
                            .find(|&p| ctx.core.player(p).alive && is_male(ctx.core, ctx.catalog, p))
                            .or(m.target)
                        else {
                            return Flow::Done;
                        };
                        m.target = Some(t);
                        set_flag(ctx.core, owner, "nvquan2_triggered", 1);
                        let has_sha = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .any(|&c| is_sha_card(ctx.core, ctx.catalog, c));
                        if !has_sha {
                            return Flow::Done;
                        }
                        wait_ask(m, ctx, HostStage::SelectCard, ASK_NVQUAN2_SHAN)
                    }
                    HostStage::SelectCard => {
                        m.stage = HostStage::Act;
                        let Some(t) = m.target else {
                            return Flow::Done;
                        };
                        let duel_cid = m.cards.first().copied().filter(|&c| c > 0);
                        if let Some(sha_cid) = answered_cards(ctx).into_iter().next() {
                            ctx.discard_from_hand(t, sha_cid);
                            ctx.emit(EventData::CardResponded {
                                card: sha_cid,
                                player: t,
                            });
                        } else {
                            if ctx.current_card_use().is_ok() {
                                ctx.current_card_use_mut().cancelled = true;
                            }
                            if let Some(c) = duel_cid {
                                ctx.to_discard(Some(c));
                                ctx.core.table.retain(|&x| x != c);
                            }
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(owner),
                                t,
                                1,
                                DamageNature::Normal,
                                duel_cid,
                            )));
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            } else {
                match m.stage {
                    HostStage::Act => {
                        let Some(t) = m
                            .targets
                            .iter()
                            .copied()
                            .find(|&p| ctx.core.player(p).alive && is_male(ctx.core, ctx.catalog, p))
                            .or(m.target)
                        else {
                            return Flow::Done;
                        };
                        m.target = Some(t);
                        let sha_cid = m.cards.first().copied();
                        let has_shan = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .any(|&c| ctx.catalog.card_kind(c) == CardKind::Shan);
                        if !has_shan {
                            mark_sha_unrespondable(ctx, owner, t, sha_cid);
                            set_flag(ctx.core, owner, "nvquan1_direct_hit", 1);
                            return Flow::Done;
                        }
                        wait_ask(m, ctx, HostStage::SelectCard, ASK_NVQUAN1_RESPONSE)
                    }
                    HostStage::SelectCard => {
                        m.stage = HostStage::Act;
                        let Some(t) = m.target else {
                            return Flow::Done;
                        };
                        let sha_cid = m.cards.first().copied();
                        if let Some(shan_cid) = answered_cards(ctx).into_iter().next() {
                            ctx.discard_from_hand(t, shan_cid);
                            ctx.emit(EventData::CardResponded {
                                card: shan_cid,
                                player: t,
                            });
                        } else {
                            mark_sha_unrespondable(ctx, owner, t, sha_cid);
                            set_flag(ctx.core, owner, "nvquan1_direct_hit", 1);
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
        }
        "feigong" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_FEIGONG_DISCARD_SHA),
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(sha_cid) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, owner, &[sha_cid]);
                    if ctx.current_card_use().is_ok() {
                        ctx.current_card_use_mut().cancelled = true;
                    }
                    if let Some(&used_sha) = m.cards.first() {
                        ctx.to_discard(Some(used_sha));
                        ctx.core.table.retain(|&x| x != used_sha);
                    }
                    set_flag(ctx.core, owner, "feigong_nullified", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jianai" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_JIANAI_CONFIRM),
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    for p in alive_others(ctx.core, owner) {
                        if ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32 {
                            ctx.recover(p, 1, None);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jieyong" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    set_flag(ctx.core, owner, "jieyong2_used", 0);
                    remove_skill_named(ctx, owner, "jieyong2");
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_JIEYONG_DISCARD_BLACK)
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(black_cid) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, owner, &[black_cid]);
                    if let Some(&used_cid) = m.cards.first() {
                        regain_card_to_hand(ctx, owner, used_cid);
                    }
                    set_flag(ctx.core, owner, "jieyong2_used", 1);
                    grant_skill_named(ctx, owner, "jieyong2");
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "jieyong2" | "jieyong3" => {
            set_flag(ctx.core, owner, "jieyong2_used", 0);
            remove_skill_named(ctx, owner, "jieyong2");
            Flow::Done
        }
        "zhulu" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    set_flag(ctx.core, owner, "zhulu_used_turn", 0);
                    return Flow::Done;
                }
                wait_ask(
                    m,
                    ctx,
                    HostStage::SelectCard,
                    ASK_ZHULU_DISCARD_SAME_SUIT,
                )
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let Some(disc_cid) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, owner, &[disc_cid]);
                    if let Some(&trick_cid) = m.cards.first() {
                        regain_card_to_hand(ctx, owner, trick_cid);
                    }
                    set_flag(ctx.core, owner, "zhulu_used_turn", 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "xieling" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_XIELING_DISCARD_TWO),
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if chosen.len() != 2 {
                    refund_once_used(ctx, owner, "xieling");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                discard_cards(ctx, owner, &chosen);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_XIELING_CHOOSE_FROM)
            }
            HostStage::TargetChoice => {
                let Some(from) = answered_player(ctx) else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(from);
                wait_ask(
                    m,
                    ctx,
                    HostStage::ComparePlayerPick,
                    ASK_XIELING_CHOOSE_CARD,
                )
            }
            HostStage::ComparePlayerPick => {
                let Some(move_cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![move_cid];
                wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_XIELING_CHOOSE_TO,
                )
            }
            HostStage::CompareTargetPick => {
                m.stage = HostStage::Act;
                if let (Some(from), Some(&move_cid), Some(to)) =
                    (m.target, m.cards.first(), answered_player(ctx))
                {
                    move_field_card_between(ctx, from, to, move_cid);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qiangyun" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_QIANGYUN_CONFIRM),
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    set_flag(ctx.core, owner, "qiangyun_declined_cid", 0);
                    draw_direct(ctx, owner, 2);
                } else if let Some(&cid) = m.cards.first() {
                    set_flag(ctx.core, owner, "qiangyun_declined_cid", cid + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "cike" => match m.stage {
            HostStage::Act => {
                if m.card == Some(99) {
                    let suit_val = flag(ctx.core, owner, "last_judge_suit");
                    let is_red = is_red_suit_code(suit_val);
                    let t = m.target.unwrap_or(owner);
                    let sha_cid = m.cards.first().copied();
                    if is_red {
                        mark_sha_unrespondable(ctx, owner, t, sha_cid);
                        set_flag(ctx.core, owner, "cike_direct_hit", 1);
                        set_flag(ctx.core, t, "cike_direct_hit", 1);
                        return Flow::Done;
                    }
                    let he = hand_and_equips(ctx.core, t);
                    if he.is_empty() || !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    if he.len() == 1 {
                        discard_cards(ctx, t, &[he[0]]);
                        return Flow::Done;
                    }
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_CIKE_DISCARD_CARD);
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_CIKE_CONFIRM)
            }
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                m.stage = HostStage::Act;
                m.card = Some(99);
                ctx.emit(EventData::OrdinaryJudge {
                    owner,
                    invocation: u32::MAX,
                });
                Flow::Yield
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let (Some(t), Some(cid)) = (m.target, answered_cards(ctx).into_iter().next()) {
                    discard_cards(ctx, t, &[cid]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "qiangxi" => match m.stage {
            HostStage::TargetChoice => {
                let Some(t) = answered_player(ctx) else {
                    refund_once_used(ctx, owner, "qiangxi");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(t);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_QIANGXI_COST)
            }
            HostStage::SelectNature => {
                let Some(tag) = answered_tag(ctx) else {
                    refund_once_used(ctx, owner, "qiangxi");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if tag == 2 {
                    let weapons = owner_weapon_cards(ctx.core, ctx.catalog, owner);
                    if weapons.is_empty() {
                        refund_once_used(ctx, owner, "qiangxi");
                        m.stage = HostStage::Act;
                        return Flow::Done;
                    }
                    if weapons.len() == 1 {
                        discard_cards(ctx, owner, &[weapons[0]]);
                        m.stage = HostStage::Act;
                        m.card = Some(98);
                        return Flow::Yield;
                    }
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_QIANGXI_WEAPON);
                }
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
                m.stage = HostStage::Act;
                m.card = Some(98);
                Flow::Yield
            }
            HostStage::SelectCard => {
                let Some(w_cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                discard_cards(ctx, owner, &[w_cid]);
                m.stage = HostStage::Act;
                m.card = Some(98);
                Flow::Yield
            }
            HostStage::Act => {
                if m.card == Some(98) {
                    m.card = None;
                    if let Some(t) = m.target {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(owner),
                                t,
                                1,
                                DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                    Flow::Done
                } else {
                    wait_ask(m, ctx, HostStage::TargetChoice, ASK_QIANGXI_TARGET)
                }
            }
            _ => Flow::Done,
        },
        "miaobi" | "miaobix" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_MIAOBI_TARGET),
            HostStage::TargetChoice => {
                let Some(t) = answered_player(ctx) else {
                    refund_once_used(ctx, owner, &m.def_key);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(t);
                let he = hand_and_equips(ctx.core, t);
                if he.is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(
                    m,
                    ctx,
                    HostStage::SelectCard,
                    ASK_MIAOBI_DISCARD_TARGET_CARD,
                )
            }
            HostStage::SelectCard => {
                let (Some(t), Some(disc_cid)) =
                    (m.target, answered_cards(ctx).into_iter().next())
                else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                let disc_kind = ctx
                    .core
                    .declared_kinds
                    .get(&disc_cid)
                    .copied()
                    .unwrap_or_else(|| ctx.catalog.card_kind(disc_cid));
                discard_cards(ctx, t, &[disc_cid]);
                m.cards = vec![disc_cid];
                if is_basic_or_normal_trick_kind(disc_kind) {
                    if !ctx.core.player(owner).hand.is_empty()
                        && miaobix_can_use_kind(ctx.core, ctx.catalog, owner, disc_kind)
                    {
                        return wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_MIAOBIX_CARD);
                    }
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                let owner_he = hand_and_equips(ctx.core, owner);
                if owner_he.is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                if owner_he.len() == 1 {
                    discard_cards(ctx, owner, &[owner_he[0]]);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(
                    m,
                    ctx,
                    HostStage::JiefenOwnerDiscard,
                    ASK_MIAOBI_OWNER_DISCARD,
                )
            }
            HostStage::JiefenOwnerDiscard => {
                m.stage = HostStage::Act;
                if let Some(c) = answered_cards(ctx).into_iter().next() {
                    discard_cards(ctx, owner, &[c]);
                }
                Flow::Done
            }
            HostStage::ComparePlayerPick => {
                let Some(use_cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                let Some(&disc_cid) = m.cards.first() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                let copy_kind = ctx
                    .core
                    .declared_kinds
                    .get(&disc_cid)
                    .copied()
                    .unwrap_or_else(|| ctx.catalog.card_kind(disc_cid));
                if miaobix_needs_target(copy_kind) {
                    m.cards = vec![disc_cid, use_cid];
                    return wait_ask(m, ctx, HostStage::CompareTargetPick, ASK_MIAOBIX_TARGET);
                }
                m.stage = HostStage::Act;
                emit_miaobix_virtual_use(ctx, owner, use_cid, copy_kind, None);
                Flow::Done
            }
            HostStage::CompareTargetPick => {
                m.stage = HostStage::Act;
                let Some(use_target) = answered_player(ctx) else {
                    return Flow::Done;
                };
                let Some(&disc_cid) = m.cards.first() else {
                    return Flow::Done;
                };
                let Some(&use_cid) = m.cards.get(1) else {
                    return Flow::Done;
                };
                let copy_kind = ctx
                    .core
                    .declared_kinds
                    .get(&disc_cid)
                    .copied()
                    .unwrap_or_else(|| ctx.catalog.card_kind(disc_cid));
                emit_miaobix_virtual_use(ctx, owner, use_cid, copy_kind, Some(use_target));
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhexian" => match m.stage {
            HostStage::Act => {
                if m.card == Some(2) {
                    set_flag(ctx.core, owner, "zhexian_used_turn", 0);
                    set_flag(ctx.core, owner, "zhexian_declined_cid", 0);
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectNature, ASK_ZHEXIAN_CONFIRM)
            }
            HostStage::SelectNature => {
                m.stage = HostStage::Act;
                if answered_confirm(ctx) {
                    set_flag(ctx.core, owner, "zhexian_used_turn", 1);
                    set_flag(ctx.core, owner, "zhexian_declined_cid", 0);
                    draw_direct(ctx, owner, 1);
                } else if let Some(&cid) = m.cards.first() {
                    set_flag(ctx.core, owner, "zhexian_declined_cid", cid + 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "guifu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_GUIFU_TARGET),
            HostStage::TargetChoice => {
                let Some(t) = answered_player(ctx) else {
                    refund_once_used(ctx, owner, "guifu");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(t);
                let equips = ctx.core.player(t).equips.clone();
                if equips.is_empty() {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                if equips.len() == 1 {
                    discard_cards(ctx, t, &[equips[0]]);
                    guifu_draw_both(ctx, owner, t);
                    m.stage = HostStage::Act;
                    return Flow::Done;
                }
                wait_ask(m, ctx, HostStage::SelectCard, ASK_GUIFU_EQUIP)
            }
            HostStage::SelectCard => {
                m.stage = HostStage::Act;
                if let (Some(t), Some(eq_cid)) = (m.target, answered_cards(ctx).into_iter().next())
                {
                    discard_cards(ctx, t, &[eq_cid]);
                    guifu_draw_both(ctx, owner, t);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "lshengong" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_LSHENGONG_HAND),
            HostStage::SelectCard => {
                let Some(hand_cid) = answered_cards(ctx).into_iter().next() else {
                    refund_once_used(ctx, owner, "lshengong");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.cards = vec![hand_cid];
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_LSHENGONG_SOURCE)
            }
            HostStage::TargetChoice => {
                let Some(src) = answered_player(ctx) else {
                    refund_once_used(ctx, owner, "lshengong");
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                m.target = Some(src);
                wait_ask(
                    m,
                    ctx,
                    HostStage::ComparePlayerPick,
                    ASK_LSHENGONG_EQUIP,
                )
            }
            HostStage::ComparePlayerPick => {
                let Some(eq_cid) = answered_cards(ctx).into_iter().next() else {
                    m.stage = HostStage::Act;
                    return Flow::Done;
                };
                if m.cards.len() >= 2 {
                    m.cards[1] = eq_cid;
                } else {
                    m.cards.push(eq_cid);
                }
                wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_LSHENGONG_DEST,
                )
            }
            HostStage::CompareTargetPick => {
                m.stage = HostStage::Act;
                let Some(dest) = answered_player(ctx) else {
                    return Flow::Done;
                };
                let Some(&hand_cid) = m.cards.first() else {
                    return Flow::Done;
                };
                let Some(&template_eq) = m.cards.get(1) else {
                    return Flow::Done;
                };
                equip_handcard_as_template(ctx, owner, dest, hand_cid, template_eq);
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
