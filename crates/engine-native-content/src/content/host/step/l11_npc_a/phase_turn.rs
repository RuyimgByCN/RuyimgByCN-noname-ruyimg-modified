//! Phase 3 P3-C1a `L11` 跨包 NPC Part A 阶段/回合与摸牌阶段技能执行（16 技：`phase_turn.rs`）。

use super::asks::{ASK_LIHUN_RETURN, ASK_REFENXUN2_DISCARD};
use super::asks_b::*;
use super::helpers::*;
use super::l11_common::*;
use super::*;
use crate::catalog::CardKind;
use crate::event::{DamageNature, DrawOutcome, EventData};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn handle_phase_turn(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "diancai" => step_diancai(m, ctx),
        "gnsheque" => step_gnsheque(m, ctx),
        "jigong" => step_jigong(m, ctx),
        "lixia" => step_lixia(m, ctx),
        "mozhi" => step_mozhi(m, ctx),
        "nzry_kuizhu" => step_kuizhu(m, ctx),
        "piaoling" => step_piaoling(m, ctx),
        "qingzhongx" => step_qingzhongx(m, ctx),
        "reguanxing" => step_reguanxing(m, ctx),
        "rejuece" => step_rejuece(m, ctx),
        "reluoyi" => step_reluoyi(m, ctx),
        "retuxi" => step_retuxi(m, ctx),
        "rezaiqi" => step_rezaiqi(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_diancai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_DIANCAI_CONFIRM),
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) && ctx.core.player(owner).alive {
                let target_hand = ctx.core.player(owner).max_hp as usize;
                let cur_hand = ctx.core.player(owner).hand.len();
                if cur_hand < target_hand {
                    ctx.start_draw(owner, (target_hand - cur_hand) as u32);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_gnsheque(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_GNSHEQUE_SHA),
        HostStage::SelectCard => {
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return clear_and_done(m);
            };
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return clear_and_done(m);
            };
            set_flag(ctx.core, owner, "l11_mod:gnsheque_pierce", 1);
            ctx.emit(EventData::CardUsed(crate::machine::card_use(c, owner, vec![t])));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => {
            set_flag(ctx.core, owner, "l11_mod:gnsheque_pierce", 0);
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_jigong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_JIGONG_CONFIRM),
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) && ctx.core.player(owner).alive {
                set_flag(ctx.core, owner, "l11_mod:jigong_active", 1);
                ctx.start_draw(owner, 2);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_lixia(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_LIXIA_MODE),
        HostStage::SelectNature => {
            set_flag(ctx.core, owner, "l11_mod:lixia_dist_to_minus1", 1);
            let drawer = if answered_tag(ctx) == Some(1) {
                m.target.unwrap_or(owner)
            } else {
                owner
            };
            if ctx.core.player(drawer).alive {
                ctx.start_draw(drawer, 1);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn mozhi_stop(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    ctx.core
        .vm_aux
        .flags
        .remove(&(m.owner, "l11_mozhi_idx".to_string()));
    clear_and_done(m)
}

fn mozhi_can_auto_target(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Wuzhong
            | CardKind::Tao
            | CardKind::Taoyuan
            | CardKind::Wugu
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Xianluhui
            | CardKind::Chiyuxi
            | CardKind::Jingleishan
            | CardKind::Liufengsan
            | CardKind::Shezhanqunru
    )
}

fn mozhi_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let limit = flag(ctx.core, owner, "l11_mozhi_hist_len").min(2);
    let mut idx = flag(ctx.core, owner, "l11_mozhi_idx");
    while idx < limit && !ctx.core.player(owner).hand.is_empty() && ctx.core.player(owner).alive {
        let enc = flag(ctx.core, owner, &format!("l11_mozhi_hist:{idx}"));
        set_flag(ctx.core, owner, "l11_mozhi_idx", idx + 1);
        let tag = (enc & 0xffff) as u16;
        if let Some(&kind) = CardKind::ALL.get(tag as usize) {
            let can_use = if kind == CardKind::Jiu {
                false
            } else if kind == CardKind::Tao {
                ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            } else if kind == CardKind::Wuzhong {
                true
            } else {
                !ctx.core.legal_targets(ctx.catalog, kind, owner).2.is_empty()
            };
            if !can_use {
                return mozhi_stop(m, ctx);
            }
            m.cards = vec![idx, u32::from(tag), (enc >> 16) & 0xff];
            return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_MOZHI_CARD);
        }
        idx += 1;
    }
    mozhi_stop(m, ctx)
}

fn step_mozhi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "l11_mozhi_idx", 0);
            mozhi_next(m, ctx)
        }
        HostStage::SelectCard => {
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return mozhi_stop(m, ctx);
            };
            let tag = *m.cards.get(1).unwrap_or(&0) as u16;
            let nat = match m.cards.get(2).copied().unwrap_or(0) {
                1 => DamageNature::Fire,
                2 => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            let Some(kind) = CardKind::ALL.get(tag as usize).copied() else {
                return mozhi_stop(m, ctx);
            };
            if mozhi_can_auto_target(kind) {
                let (_, _, pool) = ctx.core.legal_targets(ctx.catalog, kind, owner);
                let targets = if kind == CardKind::Wuzhong {
                    vec![owner]
                } else if kind == CardKind::Tao {
                    if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                        vec![owner]
                    } else {
                        Vec::new()
                    }
                } else {
                    pool
                };
                if !targets.is_empty() {
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: Some(cid),
                        player: owner,
                        targets,
                        nature: nat,
                        use_as: Some(kind),
                        cancelled: false,
                    }));
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
                return mozhi_stop(m, ctx);
            }
            if m.cards.len() < 4 {
                m.cards.push(cid);
            } else {
                m.cards[3] = cid;
            }
            wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_MOZHI_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(t) = answered_players(ctx).into_iter().next() else {
                return mozhi_stop(m, ctx);
            };
            let Some(&cid) = m.cards.get(3) else {
                return mozhi_stop(m, ctx);
            };
            let tag = *m.cards.get(1).unwrap_or(&0) as u16;
            let nat = match m.cards.get(2).copied().unwrap_or(0) {
                1 => DamageNature::Fire,
                2 => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            let kind = CardKind::ALL
                .get(tag as usize)
                .copied()
                .unwrap_or(CardKind::Sha);
            ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                card: Some(cid),
                player: owner,
                targets: vec![t],
                nature: nat,
                use_as: Some(kind),
                cancelled: false,
            }));
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => mozhi_next(m, ctx),
        _ => clear_and_done(m),
    }
}

fn step_kuizhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_KUIZHU_MODE),
        HostStage::SelectNature => match answered_tag(ctx) {
            Some(0) => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_KUIZHU_DRAW_TARGETS),
            Some(1) => wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_KUIZHU_DMG_TARGETS),
            _ => clear_and_done(m),
        },
        HostStage::TargetChoice => {
            let ts = answered_players(ctx);
            for t in ts {
                if ctx.core.player(t).alive {
                    draw_direct_cards(ctx, t, 1);
                }
            }
            clear_and_done(m)
        }
        HostStage::ShuchuiTarget => {
            let ts = answered_players(ctx);
            if ts.is_empty() {
                return clear_and_done(m);
            }
            let target_sum = flag(ctx.core, owner, "l11_kuizhu_discard_cnt") as i32;
            let sum: i32 = ts.iter().map(|&p| ctx.core.player(p).hp).sum();
            if sum != target_sum {
                return wait_npc_ask(m, ctx, HostStage::ShuchuiTarget, ASK_KUIZHU_DMG_TARGETS);
            }
            for &t in ts.iter().rev() {
                if ctx.core.player(t).alive {
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                }
            }
            clear_and_yield(m, HostStage::DrawSettle)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn step_piaoling(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_PIAOLING_CONFIRM),
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            clear_and_yield(m, HostStage::SelectNature);
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::SelectNature => {
            let suit = flag(ctx.core, owner, "last_judge_suit");
            let jcard = flag(ctx.core, owner, "last_judge_card");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            if suit == 2 && ctx.core.discard.contains(&jcard) {
                m.cards = vec![jcard];
                return wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_PIAOLING_GIVE_TARGET);
            }
            clear_and_done(m)
        }
        HostStage::TargetChoice => {
            let Some(&jcard) = m.cards.first() else {
                return clear_and_done(m);
            };
            if let Some(t) = answered_players(ctx).into_iter().next() {
                if take_from_table_discard_or_equip(ctx, jcard) {
                    let delivered =
                        (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, t, &[jcard]);
                    ctx.core
                        .player_mut(t)
                        .hand
                        .extend(delivered.iter().copied());
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: t,
                            cards: delivered,
                        });
                    }
                }
                if t == owner && !he_cards(ctx.core, owner).is_empty() {
                    return wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_PIAOLING_SELF_DISCARD);
                }
            } else if take_from_table_discard_or_equip(ctx, jcard) {
                put_on_top_of_deck(ctx, jcard);
            }
            clear_and_done(m)
        }
        HostStage::SelectCard => {
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                discard_player_cards(ctx, owner, &[c]);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_qingzhongx(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let is_begin = (m.cards.first().copied().unwrap_or(0) >> 8) > 0;
            if is_begin {
                wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_QINGZHONGX_CONFIRM)
            } else {
                set_flag(ctx.core, owner, "l11_qingzhongx_armed", 0);
                wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_QINGZHONGX_SWAP_TARGET)
            }
        }
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) && ctx.core.player(owner).alive {
                set_flag(ctx.core, owner, "l11_qingzhongx_armed", 1);
                ctx.start_draw(owner, 2);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::TargetChoice => {
            if let Some(t) = answered_players(ctx).into_iter().next() {
                let oh = std::mem::take(&mut ctx.core.player_mut(owner).hand);
                let th = std::mem::take(&mut ctx.core.player_mut(t).hand);
                ctx.core.player_mut(owner).hand = th;
                ctx.core.player_mut(t).hand = oh;
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

pub(super) fn step_refenxun_end(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let targets: Vec<PlayerId> = ctx
                .core
                .alive_players()
                .filter(|&t| {
                    flag(ctx.core, owner, &format!("l11_mod:fenxun_to:{t}")) > 0
                        && flag(ctx.core, owner, &format!("l11_refenxun_dealt:{t}")) == 0
                })
                .collect();
            m.targets = targets;
            if !m.targets.is_empty() && !he_cards(ctx.core, owner).is_empty() {
                m.targets.remove(0);
                return wait_npc_ask(
                    m,
                    ctx,
                    HostStage::JiefenOwnerDiscard,
                    ASK_REFENXUN2_DISCARD,
                );
            }
            clear_and_done(m)
        }
        HostStage::JiefenOwnerDiscard => {
            if let Some(c) = answered_cards(ctx).into_iter().next() {
                discard_player_cards(ctx, owner, &[c]);
            }
            if !m.targets.is_empty() && !he_cards(ctx.core, owner).is_empty() {
                m.targets.remove(0);
                return wait_npc_ask(
                    m,
                    ctx,
                    HostStage::JiefenOwnerDiscard,
                    ASK_REFENXUN2_DISCARD,
                );
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_reguanxing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let is_end_phase = (m.cards.first().copied().unwrap_or(0) & 0xff) == 6;
    match m.stage {
        HostStage::Act => {
            m.target = Some(u32::from(is_end_phase) as PlayerId);
            wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_REGUANXING_CONFIRM)
        }
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            let n = if ctx.core.alive_players().count() < 4 {
                3
            } else {
                5
            };
            let mut peek = Vec::with_capacity(n);
            for _ in 0..n {
                if let Some(c) = pop_deck_reshuffle(ctx) {
                    peek.push(c);
                }
            }
            if peek.is_empty() {
                return clear_and_done(m);
            }
            ctx.core
                .vm_aux
                .piles
                .insert((owner, "l11_reguanxing_peek".to_string()), peek);
            wait_npc_ask(m, ctx, HostStage::SelectCard, ASK_REGUANXING_TOP)
        }
        HostStage::SelectCard => {
            let peek = ctx
                .core
                .vm_aux
                .piles
                .remove(&(owner, "l11_reguanxing_peek".to_string()))
                .unwrap_or_default();
            let top = answered_cards(ctx);
            let bottom: Vec<CardId> = peek
                .iter()
                .copied()
                .filter(|c| !top.contains(c))
                .collect();
            for c in bottom {
                put_on_bottom_of_deck(ctx, c);
            }
            for &c in top.iter().rev() {
                if peek.contains(&c) {
                    put_on_top_of_deck(ctx, c);
                }
            }
            if m.target == Some(0) && top.is_empty() {
                set_flag(ctx.core, owner, "l11_reguanxing_end_ok", 1);
            } else if m.target == Some(1) {
                set_flag(ctx.core, owner, "l11_reguanxing_end_ok", 0);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_rejuece(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REJUECE_TARGET),
        HostStage::TargetChoice => {
            if let Some(t) = answered_players(ctx).into_iter().next() {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
                return clear_and_yield(m, HostStage::DrawSettle);
            }
            clear_and_done(m)
        }
        HostStage::DrawSettle => clear_and_done(m),
        _ => clear_and_done(m),
    }
}

fn skip_current_draw_phase(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if let Some(fr) = ctx.res.stack.iter_mut().rev().find(|f| {
        matches!(
            f.data,
            EventData::DrawPhase {
                player,
                outcome: DrawOutcome::Running,
                ..
            } if player == owner
        )
    }) {
        if let EventData::DrawPhase {
            pending, outcome, ..
        } = &mut fr.data
        {
            *pending = 0;
            *outcome = DrawOutcome::Skipped;
        }
    }
}

fn step_reluoyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::JiefenTarget, ASK_RELUOYI_CONFIRM),
        HostStage::JiefenTarget => {
            if is_confirmed(ctx) {
                skip_current_draw_phase(ctx, owner);
                let mut shown = Vec::new();
                for _ in 0..3 {
                    if let Some(c) = pop_deck_reshuffle(ctx) {
                        shown.push(c);
                    }
                }
                if !shown.is_empty() {
                    ctx.out.push(LogEvent::CardsRevealed {
                        cards: shown.clone(),
                        to: ctx.core.alive_players().collect(),
                    });
                }
                let mut gained = Vec::new();
                for c in shown {
                    let k = ctx.catalog.card_kind(c);
                    let keep = k.is_basic_kind()
                        || ctx.catalog.card_slot_of(ctx.core, c)
                            == Some(crate::catalog::EquipSlot::Weapon)
                        || k == CardKind::Duel;
                    if keep {
                        gained.push(c);
                    } else {
                        ctx.to_discard(c);
                    }
                }
                if !gained.is_empty() {
                    let delivered =
                        (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &gained);
                    ctx.core
                        .player_mut(owner)
                        .hand
                        .extend(delivered.iter().copied());
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::CardsDrawn {
                            player: owner,
                            cards: delivered,
                        });
                    }
                }
                set_flag(ctx.core, owner, "l11_mod:reluoyi_dmg_plus1", 1);
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

pub(super) fn reshuangxiong_collectible_duel_shas(
    catalog: &crate::catalog::Catalog,
    state: &crate::state::GameStateCore,
    frames: &[crate::event::Frame],
    owner: PlayerId,
) -> Option<(u32, Vec<CardId>)> {
    let (duel_idx, fid) = frames
        .iter()
        .enumerate()
        .rev()
        .find(|(_, f)| matches!(f.data, EventData::CardUsed(_)))
        .and_then(|(i, f)| match &f.data {
            EventData::CardUsed(u)
                if u.effective_kind(catalog) == CardKind::Duel
                    && (u.player == owner || u.targets.contains(&owner)) =>
            {
                Some((i, f.id))
            }
            _ => None,
        })?;
    let mut shas = state
        .vm_aux
        .piles
        .get(&(owner, format!("l11_shuangxiong_duel:{fid}")))
        .cloned()
        .unwrap_or_default();
    for f in &frames[duel_idx + 1..] {
        if let EventData::CardResponded { player, card } = f.data {
            if player != owner && !shas.contains(&card) {
                shas.push(card);
            }
        }
    }
    shas.retain(|c| state.discard.contains(c) || state.table.contains(c));
    (!shas.is_empty()).then_some((fid, shas))
}

pub(super) fn step_reshuangxiong_trigger(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if let Some(top) = ctx.res.stack.last() {
                match &top.data {
                    EventData::DrawPhase { .. } => {
                        return wait_npc_ask(
                            m,
                            ctx,
                            HostStage::JiefenTarget,
                            ASK_RESHUANGXIONG_CONFIRM,
                        );
                    }
                    EventData::CardResponded { player, card } => {
                        if *player != owner
                            && (ctx.core.discard.contains(card) || ctx.core.table.contains(card))
                        {
                            let c = *card;
                            if let Some(fid) = ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                                EventData::CardUsed(u) => Some(f.id).filter(|_| {
                                    u.effective_kind(ctx.catalog) == CardKind::Duel
                                        && (u.player == owner || u.targets.contains(&owner))
                                }),
                                _ => None,
                            }) {
                                let entry = ctx
                                    .core
                                    .vm_aux
                                    .piles
                                    .entry((owner, format!("l11_shuangxiong_duel:{fid}")))
                                    .or_default();
                                if !entry.contains(&c) {
                                    entry.push(c);
                                }
                            }
                        }
                        return clear_and_done(m);
                    }
                    EventData::Damage(_) | EventData::DamageWindow { .. } => {
                        if let Some((fid, shas)) = reshuangxiong_collectible_duel_shas(
                            ctx.catalog,
                            ctx.core,
                            &ctx.res.stack,
                            owner,
                        ) {
                            ctx.core
                                .vm_aux
                                .piles
                                .insert((owner, format!("l11_shuangxiong_duel:{fid}")), shas);
                            set_flag(ctx.core, owner, "l11_shuangxiong_duel_fid", fid);
                            return wait_npc_ask(
                                m,
                                ctx,
                                HostStage::CardsAsk,
                                ASK_RESHUANGXIONG_DUEL_CONFIRM,
                            );
                        }
                    }
                    _ => {}
                }
            }
            clear_and_done(m)
        }
        HostStage::JiefenTarget => {
            if !is_confirmed(ctx) {
                return clear_and_done(m);
            }
            skip_current_draw_phase(ctx, owner);
            let mut peek = Vec::new();
            for _ in 0..2 {
                if let Some(c) = pop_deck_reshuffle(ctx) {
                    peek.push(c);
                }
            }
            if peek.is_empty() {
                return clear_and_done(m);
            }
            ctx.out.push(LogEvent::CardsRevealed {
                cards: peek.clone(),
                to: ctx.core.alive_players().collect(),
            });
            ctx.core
                .vm_aux
                .piles
                .insert((owner, "l11_reshuangxiong_peek".to_string()), peek);
            wait_npc_ask(m, ctx, HostStage::HuanrenPick, ASK_RESHUANGXIONG_PICK)
        }
        HostStage::HuanrenPick => {
            let peek = ctx
                .core
                .vm_aux
                .piles
                .remove(&(owner, "l11_reshuangxiong_peek".to_string()))
                .unwrap_or_default();
            let pick = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|c| peek.contains(c))
                .or_else(|| peek.first().copied());
            if let Some(cid) = pick {
                let red = is_red_card(ctx.catalog, ctx.core, owner, cid);
                let delivered = (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &[cid]);
                ctx.core
                    .player_mut(owner)
                    .hand
                    .extend(delivered.iter().copied());
                if !delivered.is_empty() {
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: owner,
                        cards: delivered,
                    });
                }
                for &other in &peek {
                    if other != cid {
                        ctx.to_discard(other);
                    }
                }
                set_flag(ctx.core, owner, "shuangxiong_color", if red { 1 } else { 2 });
                set_flag(ctx.core, owner, "l11_mod:reshuangxiong_duel", 1);
            }
            clear_and_done(m)
        }
        HostStage::CardsAsk => {
            let fid = flag(ctx.core, owner, "l11_shuangxiong_duel_fid");
            set_flag(ctx.core, owner, "l11_shuangxiong_duel_fid", 0);
            if is_confirmed(ctx) && fid != 0 && ctx.core.player(owner).alive {
                if let Some(shas) = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, format!("l11_shuangxiong_duel:{fid}")))
                {
                    let mut gained = Vec::new();
                    for cid in shas {
                        if (ctx.core.discard.contains(&cid) || ctx.core.table.contains(&cid))
                            && take_from_table_discard_or_equip(ctx, cid)
                        {
                            gained.push(cid);
                        }
                    }
                    if !gained.is_empty() {
                        let delivered = (ctx.catalog.hooks.on_before_gain_hand)(
                            ctx, None, owner, &gained,
                        );
                        ctx.core
                            .player_mut(owner)
                            .hand
                            .extend(delivered.iter().copied());
                        if !delivered.is_empty() {
                            ctx.out.push(LogEvent::CardsDrawn {
                                player: owner,
                                cards: delivered,
                            });
                        }
                    }
                }
            } else if fid != 0 {
                ctx.core
                    .vm_aux
                    .piles
                    .remove(&(owner, format!("l11_shuangxiong_duel:{fid}")));
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn step_retuxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_RETUXI_TARGETS),
        HostStage::TargetChoice => {
            let ts = answered_players(ctx);
            if ts.is_empty() {
                return clear_and_done(m);
            }
            if let Some(fr) = ctx.res.stack.iter_mut().rev().find(|f| {
                matches!(
                    f.data,
                    EventData::DrawPhase {
                        player,
                        outcome: DrawOutcome::Running,
                        ..
                    } if player == owner
                )
            }) {
                if let EventData::DrawPhase { pending, .. } = &mut fr.data {
                    let take = (*pending).max(0).min(ts.len() as i32);
                    *pending -= take;
                }
            }
            for t in ts {
                if !ctx.core.player(t).hand.is_empty() && ctx.core.player(owner).alive {
                    let idx =
                        ctx.core.rng.gen_range(ctx.core.player(t).hand.len() as u32) as usize;
                    let cid = ctx.core.player(t).hand[idx];
                    move_cards_to_hand(ctx, t, owner, &[cid]);
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}

fn rezaiqi_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(t).alive && ctx.core.player(owner).alive {
            m.target = Some(t);
            return wait_npc_ask(m, ctx, HostStage::SelectNature, ASK_REZAIQI_TARGET_MODE);
        }
    }
    clear_and_done(m)
}

fn step_rezaiqi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_npc_ask(m, ctx, HostStage::TargetChoice, ASK_REZAIQI_TARGETS),
        HostStage::TargetChoice => {
            m.targets = answered_players(ctx);
            rezaiqi_next(m, ctx)
        }
        HostStage::SelectNature => {
            if let Some(t) = m.target {
                if answered_tag(ctx) == Some(1) {
                    emit_recover(ctx, owner, 1);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
                if ctx.core.player(t).alive {
                    ctx.start_draw(t, 1);
                    return clear_and_yield(m, HostStage::DrawSettle);
                }
            }
            rezaiqi_next(m, ctx)
        }
        HostStage::DrawSettle => rezaiqi_next(m, ctx),
        _ => clear_and_done(m),
    }
}

pub(super) fn step_lihun_phase_end(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let raw = flag(ctx.core, owner, "l11_lihun_target");
            set_flag(ctx.core, owner, "l11_lihun_target", 0);
            let Some(t) = raw.checked_sub(1).map(|v| v as PlayerId) else {
                return clear_and_done(m);
            };
            if !ctx.core.player(t).alive
                || ctx.core.player(t).hp <= 0
                || he_cards(ctx.core, owner).is_empty()
            {
                return clear_and_done(m);
            }
            m.target = Some(t);
            wait_npc_ask(m, ctx, HostStage::JiefenOwnerDiscard, ASK_LIHUN_RETURN)
        }
        HostStage::JiefenOwnerDiscard => {
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let cs = answered_cards(ctx);
                if !cs.is_empty() {
                    move_cards_to_hand(ctx, owner, t, &cs);
                }
            }
            clear_and_done(m)
        }
        _ => clear_and_done(m),
    }
}
