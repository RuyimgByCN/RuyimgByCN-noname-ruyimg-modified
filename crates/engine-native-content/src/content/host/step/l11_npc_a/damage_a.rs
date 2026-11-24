//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 伤害与触发类技执行（14 技：`damage_a.rs`）。

use super::asks_b::*;
use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::CardKind;
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(super) fn handle_damage_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "benyu" => step_benyu(m, ctx),
        "chouce" => step_chouce(m, ctx),
        "dcmoukui" => step_dcmoukui(m, ctx),
        "dcnutao" => step_dcnutao(m, ctx),
        "guju" => step_guju(m, ctx),
        "jieyuan" => step_jieyuan(m, ctx),
        "leiji" => step_leiji(m, ctx),
        "liyu" => step_liyu(m, ctx),
        "mjmouzhi" => step_mjmouzhi(m, ctx),
        "nsxianhai" => step_nsxianhai(m, ctx),
        "qingxian" => step_qingxian(m, ctx),
        "rejinjiu" => step_rejinjiu(m, ctx),
        "repojun" => step_repojun(m, ctx),
        "retianxiang" => step_retianxiang(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_benyu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(src) = m.target else {
        return clear_and_done(m);
    };
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_BENYU_MODE),
        HostStage::SelectNature => match answered_tag(ctx) {
            Some(0) => {
                let want = ctx.core.player(src).hand.len().min(5);
                let have = ctx.core.player(owner).hand.len();
                if want > have {
                    ctx.start_draw(owner, (want - have) as u32);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
                clear_and_done(m)
            }
            Some(1) => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_BENYU_DISCARD),
            _ => clear_and_done(m),
        },
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            let need = ctx.core.player(src).hand.len() + 1;
            if cs.len() == need {
                discard_player_cards(ctx, owner, &cs);
                emit_damage(ctx, Some(owner), src, 1, DamageNature::Normal, None);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_chouce(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.cards.first().copied().unwrap_or(0) == 0 {
                m.cards = vec![1];
            }
            wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_CHOUCE_CONFIRM)
        }
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            m.cards[0] = m.cards[0].saturating_sub(1);
            clear_and_yield(m, HostStage::SelectNature);
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::SelectNature => {
            let suit = flag(ctx.core, owner, "last_judge_suit");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            if matches!(suit, 1 | 3) {
                if alive_all(ctx.core, owner)
                    .into_iter()
                    .any(|p| !hej_cards(ctx.core, p).is_empty())
                {
                    return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_CHOUCE_DISCARD_TARGET);
                }
            } else if matches!(suit, 2 | 4) {
                return wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_CHOUCE_DRAW_TARGET);
            }
            chouce_next_point(m, ctx)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return chouce_next_point(m, ctx);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_CHOUCE_DISCARD_CARD)
        }
        HostStage::SelectCard => {
            if let (Some(t), Some(c)) = (m.target, answered_cards(ctx).into_iter().next()) {
                discard_player_cards(ctx, t, &[c]);
            }
            chouce_next_point(m, ctx)
        }
        HostStage::ShuchuiTarget => {
            if let Some(t) = answered_players(ctx).into_iter().next() {
                if ctx.core.player(t).alive {
                    ctx.start_draw(t, 1);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            chouce_next_point(m, ctx)
        }
        HostStage::DrawSettle => chouce_next_point(m, ctx),
        _ => clear_and_done(m),
    }
}

fn chouce_next_point(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.cards.first().copied().unwrap_or(0) > 0 && ctx.core.player(owner).alive {
        wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_CHOUCE_CONFIRM)
    } else {
        clear_and_done(m)
    }
}

fn step_dcmoukui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let Some(&first_t) = m.targets.iter().find(|&&t| ctx.core.player(t).alive) else {
                return clear_and_done(m);
            };
            m.target = Some(first_t);
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_DCMOUKUI_MODE)
        }
        HostStage::SelectNature => {
            let Some(mode) = answered_tag(ctx) else {
                return clear_and_done(m);
            };
            let Some(t) = m.target else {
                return clear_and_done(m);
            };
            let sha_fid = ctx
                .res
                .stack
                .iter()
                .rev()
                .find_map(|f| {
                    matches!(&f.data, EventData::CardUsed(u) if u.player == owner && u.effective_kind(ctx.catalog).is_sha_like())
                        .then_some(f.id)
                })
                .unwrap_or(0);
            if mode == 2 && sha_fid != 0 {
                set_flag(
                    ctx.core,
                    owner,
                    &format!("l11_dcmoukui_both:{sha_fid}:{t}"),
                    1,
                );
            }
            if matches!(mode, 1 | 2) {
                if let Some(c) = he_cards(ctx.core, t).into_iter().next() {
                    discard_player_cards(ctx, t, &[c]);
                }
            }
            if matches!(mode, 0 | 2) && ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_dcnutao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        if let Some(top) = ctx.res.stack.last() {
            if matches!(top.data, EventData::Damage(_) | EventData::DamageWindow { .. }) {
                inc_flag(ctx.core, owner, "l11_mod:dcnutao_sha_plus", 1);
                return clear_and_done(m);
            }
        }
        let others: Vec<PlayerId> = m
            .targets
            .iter()
            .copied()
            .filter(|&t| t != owner && ctx.core.player(t).alive)
            .collect();
        if !others.is_empty() {
            let idx = ctx.core.rng.gen_range(others.len() as u32) as usize;
            emit_damage(
                ctx,
                Some(owner),
                others[idx],
                1,
                DamageNature::Thunder,
                None,
            );
            return clear_and_yield(m, HostStage::DrawSettle);
        }
    }
    clear_and_done(m)
}

fn step_guju(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        ctx.start_draw(owner, 1);
        return clear_and_yield(m, HostStage::DrawSettle);
    }
    clear_and_done(m)
}

fn step_jieyuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let is_source_case = m.targets.get(1).copied() == Some(owner)
        && m.targets.first().copied() != Some(owner)
        && m.targets
            .first()
            .copied()
            .is_some_and(|t| ctx.core.player(t).hp >= ctx.core.player(owner).hp);
    let ask_id = if is_source_case {
        ASK_JIEYUAN_MORE_CARD
    } else {
        ASK_JIEYUAN_LESS_CARD
    };
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ask_id),
        HostStage::SelectCard => {
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                discard_player_cards(ctx, owner, &[c]);
                with_current_damage_mut(ctx, |d| {
                    if is_source_case {
                        d.amount += 1;
                    } else {
                        d.amount = (d.amount - 1).max(0);
                        if d.amount == 0 {
                            d.prevented = true;
                        }
                    }
                });
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_leiji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_LEIJI_TARGET),
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            clear_and_yield(m, HostStage::SelectNature);
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::SelectNature => {
            let suit = flag(ctx.core, owner, "last_judge_suit");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            if suit == 1 {
                if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                    emit_damage(ctx, Some(owner), t, 2, DamageNature::Thunder, None);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_liyu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_LIYU_CONFIRM_TARGET),
        HostStage::TargetChoice => {
            let Some(duel_t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(vict) = m.target else {
                return clear_and_done(m);
            };
            m.targets = vec![duel_t];
            if he_cards(ctx.core, vict).is_empty() {
                return clear_and_done(m);
            }
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_LIYU_GAIN_CARD)
        }
        HostStage::SelectCard => {
            if let (Some(vict), Some(c)) = (m.target, answered_cards(ctx).into_iter().next()) {
                move_cards_to_hand(ctx, vict, owner, &[c]);
            }
            if let Some(&duel_t) = m.targets.first() {
                if ctx.core.player(owner).alive && ctx.core.player(duel_t).alive {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![duel_t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Duel),
                        cancelled: false,
                    }));
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_mjmouzhi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        if let Some(cid) = m.cards.get(1).copied().and_then(|v| v.checked_sub(1)) {
            let sbit = suit_bit(card_suit(ctx.catalog, ctx.core, owner, cid));
            if ctx
                .res
                .stack
                .iter()
                .rev()
                .any(|f| matches!(&f.data, EventData::Damage(d) if d.settlement.committed_damage.is_none()))
            {
                if sbit != 0 && flag(ctx.core, owner, "l11_mjmouzhi_suit") == sbit {
                    with_current_damage_mut(ctx, |d| {
                        d.prevented = true;
                        d.amount = 0;
                    });
                }
            } else if sbit != 0 {
                set_flag(ctx.core, owner, "l11_mjmouzhi_suit", sbit);
            }
        }
    }
    clear_and_done(m)
}

fn finish_nsxianhai_after_slot(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    src: PlayerId,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return clear_and_done(m);
    }
    if let Some(def) = ctx.catalog.skill_by_name("nsxingchu") {
        if ctx.core.has_skill(owner, def) {
            set_flag(ctx.core, owner, "l11_nsxingchu_revived", 1);
        }
    }
    if ctx.core.player(src).alive {
        let shans: Vec<crate::ids::CardId> = ctx
            .core
            .player(src)
            .hand
            .iter()
            .copied()
            .filter(|&c| ctx.catalog.resolved_card_kind(ctx.core, c) == CardKind::Shan)
            .collect();
        if !shans.is_empty() {
            discard_player_cards(ctx, src, &shans);
        }
    }
    clear_and_done(m)
}

fn nsxianhai_src_has_enabled_slot(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    src: PlayerId,
) -> bool {
    [
        crate::catalog::EquipSlot::Weapon,
        crate::catalog::EquipSlot::Armor,
        crate::catalog::EquipSlot::HorsePlus,
        crate::catalog::EquipSlot::HorseMinus,
        crate::catalog::EquipSlot::Treasure,
    ]
    .into_iter()
    .any(|slot| catalog.is_equip_slot_enabled(core, src, slot))
}

fn step_nsxianhai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(src) = m.target else {
        return clear_and_done(m);
    };
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_NSXIANHAI_CONFIRM),
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            set_flag(ctx.core, owner, "l11_nsxianhai_round", ctx.core.turn.round);
            ctx.lose_max_hp(owner, 1);
            if ctx.core.player(owner).max_hp == 0 && ctx.core.player(owner).alive {
                m.stage = HostStage::DrawSettle;
                ctx.emit(EventData::Death {
                    player: owner,
                    killer: None,
                });
                return Flow::Yield;
            }
            if !ctx.core.player(owner).alive {
                return clear_and_done(m);
            }
            if ctx.core.player(src).alive && nsxianhai_src_has_enabled_slot(ctx.core, ctx.catalog, src) {
                wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_NSXIANHAI_SLOT)
            } else {
                finish_nsxianhai_after_slot(m, ctx, owner, src)
            }
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive {
                return clear_and_done(m);
            }
            if ctx.core.player(src).alive && nsxianhai_src_has_enabled_slot(ctx.core, ctx.catalog, src) {
                wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_NSXIANHAI_SLOT)
            } else {
                finish_nsxianhai_after_slot(m, ctx, owner, src)
            }
        }
        HostStage::SelectNature => {
            let Some(slot_tag) = answered_tag(ctx) else {
                return clear_and_done(m);
            };
            let slots: &[u32] = match slot_tag {
                1 => &[1],
                2 => &[2],
                3 => &[3, 4],
                _ => &[5],
            };
            let mut mask = flag(ctx.core, src, "disabled_equip_slots");
            for &s in slots {
                mask |= 1u32 << s;
                set_flag(ctx.core, src, &format!("equip_slot_disabled:{s}"), 1);
                let want = match s {
                    1 => crate::catalog::EquipSlot::Weapon,
                    2 => crate::catalog::EquipSlot::Armor,
                    3 => crate::catalog::EquipSlot::HorsePlus,
                    4 => crate::catalog::EquipSlot::HorseMinus,
                    _ => crate::catalog::EquipSlot::Treasure,
                };
                let matching: Vec<crate::ids::CardId> = ctx
                    .core
                    .player(src)
                    .equips
                    .iter()
                    .copied()
                    .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(want))
                    .collect();
                if !matching.is_empty() {
                    discard_player_cards(ctx, src, &matching);
                }
            }
            set_flag(ctx.core, src, "disabled_equip_slots", mask);
            finish_nsxianhai_after_slot(m, ctx, owner, src)
        }
        _ => clear_and_done(m),
    }
}

fn step_qingxian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if let Some(top) = ctx.res.stack.last() {
                match &top.data {
                    EventData::Recover { .. } => {
                        if ctx.core.alive_players().any(|p| ctx.core.player(p).dying) {
                            inc_flag(ctx.core, owner, "l11_qingxian_queued", 1);
                            return clear_and_done(m);
                        }
                        return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_QINGXIAN_TARGET);
                    }
                    EventData::DyingSaved { .. } => {
                        let q = flag(ctx.core, owner, "l11_qingxian_queued");
                        if q > 0 {
                            set_flag(ctx.core, owner, "l11_qingxian_queued", q - 1);
                            return wait_npc_ask(
                                m,
                                ctx,
                                HostStage::TargetChoice,
                                ASK_QINGXIAN_TARGET,
                            );
                        }
                        return clear_and_done(m);
                    }
                    EventData::Damage(_) => {
                        if m.target.is_some() {
                            return wait_npc_ask(
                                m,
                                ctx,
                                HostStage::SelectNature,
                                ASK_QINGXIAN_MODE,
                            );
                        }
                    }
                    _ => {}
                }
            }
            clear_and_done(m)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_QINGXIAN_MODE)
        }
        HostStage::SelectNature => {
            let Some(mode) = answered_tag(ctx) else {
                return clear_and_done(m);
            };
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return clear_and_done(m);
            };
            if mode == 1 {
                emit_recover(ctx, t, 1);
                return clear_and_yield(m, HostStage::JiefenTargetDiscard);
            }
            emit_lose_hp(ctx, t, 1);
            clear_and_yield(m, HostStage::JiefenTarget)
        }
        HostStage::JiefenTarget => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return clear_and_done(m);
            };
            let equips: Vec<_> = ctx
                .core
                .deck
                .iter()
                .chain(ctx.core.discard.iter())
                .copied()
                .filter(|&c| {
                    ctx.catalog
                        .card_slot_of(ctx.core, c)
                        .is_some_and(|sl| ctx.catalog.is_equip_slot_enabled(ctx.core, t, sl))
                })
                .collect();
            if equips.is_empty() {
                return clear_and_done(m);
            }
            let pick = equips[ctx.core.rng.gen_range(equips.len() as u32) as usize];
            take_from_table_discard_or_equip(ctx, pick);
            ctx.core.deck.retain(|&c| c != pick);
            let club = card_suit(ctx.catalog, ctx.core, t, pick) == crate::catalog::Suit::Club;
            equip_card_direct(ctx, t, pick);
            if club && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                emit_recover(ctx, owner, 1);
            }
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::JiefenTargetDiscard => {
            if m.target.is_some_and(|p| !ctx.core.player(p).equips.is_empty()) {
                return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_QINGXIAN_DISCARD_EQUIP);
            }
            clear_and_done(m)
        }
        HostStage::SelectCard => {
            if let (Some(t), Some(c)) = (m.target, answered_cards(ctx).into_iter().next()) {
                let club = card_suit(ctx.catalog, ctx.core, t, c) == crate::catalog::Suit::Club;
                discard_player_cards(ctx, t, &[c]);
                if club && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                    emit_recover(ctx, owner, 1);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => {
            if flag(ctx.core, owner, "l11_qingxian_queued") > 0
                && !ctx.core.alive_players().any(|p| ctx.core.player(p).dying)
            {
                let q = flag(ctx.core, owner, "l11_qingxian_queued");
                set_flag(ctx.core, owner, "l11_qingxian_queued", q - 1);
                return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_QINGXIAN_TARGET);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_rejinjiu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        let Some(src) = m.targets.get(1).copied() else {
            return clear_and_done(m);
        };
        let jiu_add = ctx
            .res
            .stack
            .iter()
            .rev()
            .find_map(|f| match &f.data {
                EventData::CardUsed(u) if u.player == src && u.effective_kind(ctx.catalog).is_sha_like() => {
                    let v = flag(ctx.core, src, &format!("sha_used_jiu:{}", f.id));
                    (v > 0).then_some(v as i32)
                }
                _ => None,
            })
            .unwrap_or(0);
        if jiu_add > 0 {
            with_current_damage_mut(ctx, |d| {
                d.amount = (d.amount - jiu_add).max(0);
                if d.amount == 0 {
                    d.prevented = true;
                }
            });
        }
        let _ = owner;
    }
    clear_and_done(m)
}

fn step_repojun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.res.stack.last().is_some_and(|f| {
                matches!(f.data, EventData::Damage(_) | EventData::DamageWindow { .. })
            }) {
                with_current_damage_mut(ctx, |d| d.amount += 1);
                return clear_and_done(m);
            }
            let Some(&next_t) = m
                .targets
                .iter()
                .find(|&&t| ctx.core.player(t).alive && ctx.core.player(t).hp > 0 && !he_cards(ctx.core, t).is_empty())
            else {
                return clear_and_done(m);
            };
            m.target = Some(next_t);
            m.targets.retain(|&t| t != next_t);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REPOJUN_CARDS)
        }
        HostStage::SelectCard => {
            let cs = answered_cards(ctx);
            if let Some(t) = m.target {
                let mut moved = Vec::new();
                for c in cs {
                    if ctx.core.player(t).hand.contains(&c) {
                        ctx.take_hand_card(t, c);
                        moved.push(c);
                    } else if ctx.core.player(t).equips.contains(&c) {
                        ctx.take_owned_card(t, c);
                        moved.push(c);
                    }
                }
                if !moved.is_empty() {
                    ctx.note_cards_lost(t, &moved);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((owner, format!("l11_repojun:{t}")))
                        .or_default()
                        .extend(moved);
                }
            }
            if let Some(&next_t) = m
                .targets
                .iter()
                .find(|&&t| ctx.core.player(t).alive && ctx.core.player(t).hp > 0 && !he_cards(ctx.core, t).is_empty())
            {
                m.target = Some(next_t);
                m.targets.retain(|&t| t != next_t);
                return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REPOJUN_CARDS);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_retianxiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let dmg_src = m.targets.get(1).copied();
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_RETIANXIANG_CARD),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            m.cards = vec![c];
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_RETIANXIANG_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(&c) = m.cards.first() else {
                return clear_and_done(m);
            };
            m.target = Some(t);
            discard_player_cards(ctx, owner, &[c]);
            with_current_damage_mut(ctx, |d| {
                d.prevented = true;
                d.amount = 0;
            });
            wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_RETIANXIANG_MODE)
        }
        HostStage::SelectNature => {
            let mode = answered_tag(ctx).unwrap_or(0);
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return clear_and_done(m);
            };
            if mode == 1 {
                emit_lose_hp(ctx, t, 1);
                if let Some(&c) = m.cards.first() {
                    if ctx.core.discard.contains(&c) {
                        ctx.core.discard.retain(|&x| x != c);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                }
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            m.card = None;
            emit_damage(ctx, dmg_src, t, 1, DamageNature::Normal, None);
            clear_and_yield(m, HostStage::ShuchuiTarget)
        }
        HostStage::ShuchuiTarget => {
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let lost = ((ctx.core.player(t).max_hp as i32 - ctx.core.player(t).hp).max(0) as u32)
                    .min(5);
                if lost > 0 {
                    ctx.start_draw(t, lost);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}
