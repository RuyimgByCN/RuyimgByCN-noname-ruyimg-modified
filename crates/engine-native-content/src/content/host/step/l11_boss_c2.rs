//! L11（太虚幻境）Batch L11-5 领主与通用武将技（下，`l11_boss_c2.rs`）。
//!
//! 承载后半 11 技及辅助：
//! - `boss_moyany`、`boss_danshu`、`boss_modao`、`boss_jueji`、`boss_mojian`、`boss_lianyu`
//! - `boss_skonghun`、`txhj_zhuwei`、`boss_zhennu`、`txhj_chuhai`、`txhj_chuhai2`

use super::l11_boss_c::wait_bc_ask;
use super::l11_boss_c_asks::*;
use super::l11_common::*;
use super::l11_common_b2::emit_virtual_kind_use;
use super::l11_common_b3::gain_first_from_deck_or_discard;
use super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx};
use crate::event::{DamageNature, DrawOutcome, EventData, Frame, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

pub(super) fn pop_deck_with_reshuffle(core: &mut GameStateCore) -> Option<CardId> {
    if core.deck.is_empty() && !core.discard.is_empty() {
        core.rng.shuffle(&mut core.discard);
        core.deck.append(&mut core.discard);
    }
    core.deck.pop()
}

fn card_type2(k: CardKind) -> u8 {
    if k.is_basic_kind() {
        1
    } else if k.equip_slot().is_some() {
        3
    } else {
        2
    }
}

fn empty_chuhai2_slots(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<EquipSlot> {
    let pl = core.player(p);
    let has_liulong = has_equip_named(core, catalog, p, "txhj_liulongcanjia");
    let has_equip5 = has_named_skill(core, catalog, p, "_txhj_equip5");
    let treasure_cap = if has_equip5 { 2 } else { 1 };
    let treasure_cnt = pl
        .equips
        .iter()
        .filter(|&&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Treasure))
        .count();
    let mut out = Vec::new();
    for slot in [
        EquipSlot::Weapon,
        EquipSlot::Armor,
        EquipSlot::HorsePlus,
        EquipSlot::HorseMinus,
        EquipSlot::Treasure,
    ] {
        if slot == EquipSlot::Treasure {
            if treasure_cnt < treasure_cap {
                out.push(slot);
            }
            continue;
        }
        if has_liulong && matches!(slot, EquipSlot::HorsePlus | EquipSlot::HorseMinus) {
            continue;
        }
        let occupied = pl
            .equips
            .iter()
            .any(|&c| catalog.card_slot_of(core, c) == Some(slot));
        if !occupied {
            out.push(slot);
        }
    }
    out
}

/// `phaseUseEnd` 收束清理：`txhj_chuhai` 赢得拼点后授予的 `txhj_chuhai2` 到出牌阶段结束回收。
pub(crate) fn clear_chuhai2_on_play_end(ctx: &mut Ctx<'_>, player: PlayerId) {
    set_flag(ctx.core, player, "skill_turn:l11_chuhai2_target", 0);
    remove_named_skill(ctx, player, "txhj_chuhai2");
}

fn is_l11_card_lost_event(
    state: &GameStateCore,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let fid = frame.map_or(0, |f| f.id);
    let not_deduped = |c: &CardId| {
        flag(state, owner, &format!("l11_loss_dedup:{fid}:{c}")) == 0
            && !frames.iter().any(|f| {
                f.id != fid
                    && f.stage == crate::event::Stage::Commit
                    && matches!(
                        &f.data,
                        EventData::CardsLost { player, cards, .. }
                            if *player == owner && cards.contains(c)
                    )
            })
    };
    match event {
        EventData::CardsLost { player, cards, .. } => *player == owner && !cards.is_empty(),
        EventData::CardsDiscarded { player, cards } => {
            *player == owner && cards.iter().any(not_deduped)
        }
        EventData::CardsMoved { from, to, cards } => {
            *from == owner && to != from && cards.iter().any(not_deduped)
        }
        EventData::CardUsed(u) => {
            u.player == owner
                && !u.cancelled
                && flag(state, owner, &format!("l11_carduse_lost:{fid}")) > 0
        }
        _ => false,
    }
}

pub(super) fn filter_l11_boss_c2_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    match skill.name.as_str() {
        "boss_moyany" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && state.turn.current != owner
                && flag(state, owner, "l11_moyany_ing") == 0
                && is_l11_card_lost_event(state, frames, frame, owner, event)
        }
        "boss_danshu" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && state.turn.current != owner
                && state.player(owner).hp < state.player(owner).max_hp as i32
                && is_l11_card_lost_event(state, frames, frame, owner, event)
        }
        "boss_modao" => {
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Prepare,
                        begin: true
                    } if *player == owner
                )
        }
        "boss_jueji" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::DrawPhase {
                        player,
                        pending,
                        outcome: DrawOutcome::Running,
                        ..
                    } if *player != owner
                        && *pending > 0
                        && state.player(*player).alive
                        && !state.is_friend_of(ctx.catalog, owner, *player)
                        && state.player(*player).hp < state.player(*player).max_hp as i32
                )
        }
        "boss_mojian" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Play,
                        begin: true
                    } if *player == owner
                )
                && !aoe_enemy_targets(state, ctx.catalog, owner, CardKind::ArcheryAttack).is_empty()
        }
        "boss_lianyu" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::End,
                        begin: true
                    } if *player == owner
                )
                && !aoe_enemy_targets(state, ctx.catalog, owner, CardKind::ArcheryAttack).is_empty()
        }
        "boss_skonghun" => {
            if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                || !matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Play,
                        begin: true
                    } if *player == owner
                )
            {
                return false;
            }
            let damaged = (state.player(owner).max_hp as i32 - state.player(owner).hp).max(0) as usize;
            let enemies = enemy_targets(state, ctx.catalog, owner, None).len();
            damaged > 0 && enemies > 0 && damaged >= enemies
        }
        "txhj_zhuwei" => matches!(
            event,
            EventData::OrdinaryJudge {
                owner: j_owner,
                invocation: jc,
            } if *j_owner == owner
                && *jc != u32::MAX
                && state.discard.contains(jc)
                && is_unblocked_l11_owner(state, ctx.catalog, owner)
        ),
        "boss_zhennu" => {
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Prepare,
                        begin: true
                    } if *player == owner
                )
                && !alive_others(state, owner).is_empty()
        }
        "txhj_chuhai2" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                    && flag(state, owner, "skill_turn:l11_chuhai2_target")
                        == u32::from(d.target) + 1
                    && !empty_chuhai2_slots(state, ctx.catalog, owner).is_empty()
        ),
        _ => false,
    }
}

pub(super) fn handle_l11_boss_c2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "boss_moyany" => handle_moyany(m, ctx),
        "boss_danshu" => handle_danshu(m, ctx),
        "boss_modao" => {
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "boss_jueji" => handle_jueji(m, ctx),
        "boss_mojian" => match m.stage {
            HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_MOJIAN_CONFIRM),
            HostStage::TargetChoice => {
                m.card = None;
                if !is_confirmed(ctx) {
                    return Flow::Done;
                }
                let tgts = aoe_enemy_targets(ctx.core, ctx.catalog, owner, CardKind::ArcheryAttack);
                if tgts.is_empty() {
                    return Flow::Done;
                }
                emit_virtual_kind_use(
                    ctx,
                    owner,
                    CardKind::ArcheryAttack,
                    tgts,
                    DamageNature::Normal,
                );
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            _ => Flow::Done,
        },
        "boss_lianyu" => handle_lianyu(m, ctx),
        "boss_skonghun" => handle_skonghun(m, ctx),
        "txhj_zhuwei" => handle_zhuwei(m, ctx),
        "boss_zhennu" => handle_zhennu(m, ctx),
        "txhj_chuhai" => handle_chuhai(m, ctx),
        "txhj_chuhai2" => handle_chuhai2(m, ctx),
        "xueji" => handle_xueji(m, ctx),
        _ => Flow::Done,
    }
}

fn handle_moyany(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_MOYANY_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l11_moyany_ing", 1);
            m.stage = HostStage::SelectNature;
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
            if matches!(suit, 2 | 4) && !alive_others(ctx.core, owner).is_empty() {
                return wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_MOYANY_TARGET);
            }
            set_flag(ctx.core, owner, "l11_moyany_ing", 0);
            Flow::Done
        }
        HostStage::SelectCard => {
            m.card = None;
            if let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive)
            {
                m.stage = HostStage::DrawSettle;
                emit_damage(ctx, Some(owner), t, 2, DamageNature::Fire, None);
                return Flow::Yield;
            }
            set_flag(ctx.core, owner, "l11_moyany_ing", 0);
            Flow::Done
        }
        _ => {
            set_flag(ctx.core, owner, "l11_moyany_ing", 0);
            Flow::Done
        }
    }
}

fn handle_danshu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_DANSHU_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            m.stage = HostStage::SelectNature;
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
            if matches!(suit, 2 | 4) && ctx.core.player(owner).alive {
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_jueji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let root = m.card.take();
            let found = root
                .and_then(|id| ctx.res.frame(id))
                .or_else(|| {
                    ctx.res.stack.iter().rev().find(|f| {
                        matches!(
                            f.data,
                            EventData::DrawPhase {
                                player,
                                pending,
                                outcome: DrawOutcome::Running,
                                ..
                            } if player != owner && pending > 0
                        )
                    })
                });
            let Some(fr) = found else {
                return Flow::Done;
            };
            let EventData::DrawPhase {
                player,
                pending,
                outcome: DrawOutcome::Running,
                ..
            } = fr.data
            else {
                return Flow::Done;
            };
            if pending <= 0
                || player == owner
                || !ctx.core.player(player).alive
                || ctx.core.is_friend_of(ctx.catalog, owner, player)
                || ctx.core.player(player).hp >= ctx.core.player(player).max_hp as i32
            {
                return Flow::Done;
            }
            m.target = Some(player);
            set_flag(ctx.core, owner, "l11_jueji_draw_root", fr.id + 1);
            wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_JUEJI_CONFIRM)
        }
        HostStage::TargetChoice => {
            m.card = None;
            let root = flag(ctx.core, owner, "l11_jueji_draw_root").checked_sub(1);
            set_flag(ctx.core, owner, "l11_jueji_draw_root", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let tgt = m.target;
            let frame_idx = root
                .and_then(|id| ctx.res.stack.iter().position(|f| f.id == id))
                .or_else(|| {
                    ctx.res.stack.iter().rposition(|f| {
                        matches!(
                            f.data,
                            EventData::DrawPhase {
                                player,
                                pending,
                                outcome: DrawOutcome::Running,
                                ..
                            } if Some(player) == tgt && pending > 0
                        )
                    })
                });
            if let Some(fr) = frame_idx.map(|i| &mut ctx.res.stack[i]) {
                if let EventData::DrawPhase {
                    pending,
                    outcome: DrawOutcome::Running,
                    ..
                } = &mut fr.data
                {
                    if *pending > 0 {
                        *pending -= 1;
                    }
                }
            }
            Flow::Done
        }
        _ => {
            set_flag(ctx.core, owner, "l11_jueji_draw_root", 0);
            Flow::Done
        }
    }
}

fn handle_lianyu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_LIANYU_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            m.targets = enemy_targets(ctx.core, ctx.catalog, owner, None);
            m.stage = HostStage::DrawSettle;
            handle_lianyu(m, ctx)
        }
        HostStage::DrawSettle => {
            while let Some(t) = m.targets.first().copied() {
                m.targets.remove(0);
                if ctx.core.player(t).alive {
                    emit_damage(ctx, Some(owner), t, 1, DamageNature::Fire, None);
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_skonghun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        let tgts = enemy_targets(ctx.core, ctx.catalog, owner, None);
        if tgts.is_empty() {
            return Flow::Done;
        }
        set_flag(ctx.core, owner, "l11_skonghun_num", tgts.len() as u32);
        m.targets = tgts;
        m.stage = HostStage::DrawSettle;
    }
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(t).alive {
            emit_damage(ctx, Some(owner), t, 1, DamageNature::Thunder, None);
            return Flow::Yield;
        }
    }
    let rec = flag(ctx.core, owner, "l11_skonghun_num");
    set_flag(ctx.core, owner, "l11_skonghun_num", 0);
    if rec > 0 && ctx.core.player(owner).alive {
        ctx.recover(owner, rec, None);
    }
    Flow::Done
}

fn handle_zhennu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act {
        m.targets = alive_others(ctx.core, owner);
        m.stage = HostStage::DrawSettle;
    }
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(t).alive {
            emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
            return Flow::Yield;
        }
    }
    Flow::Done
}

fn handle_zhuwei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let Some(jc) = m.card.take().filter(|&c| c != u32::MAX) else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "l11_zhuwei_card", jc + 1);
            wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_ZHUWEI_CONFIRM_GAIN)
        }
        HostStage::TargetChoice => {
            m.card = None;
            let jc = flag(ctx.core, owner, "l11_zhuwei_card").checked_sub(1);
            set_flag(ctx.core, owner, "l11_zhuwei_card", 0);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            if let Some(c) = jc {
                if let Some(pos) = ctx.core.discard.iter().position(|&d| d == c) {
                    ctx.core.discard.remove(pos);
                    let delivered = (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &[c]);
                    for &dc in &delivered {
                        ctx.core.player_mut(owner).hand.push(dc);
                    }
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::JudgementGained {
                            player: owner,
                            card: delivered[0],
                        });
                        ctx.emit(EventData::CardsDrawn {
                            player: owner,
                            cards: delivered,
                        });
                    }
                }
            }
            wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_ZHUWEI_CONFIRM_BUFF)
        }
        HostStage::SelectNature => {
            m.card = None;
            if is_confirmed(ctx) {
                let cur = ctx.core.turn.current;
                if ctx.core.player(cur).alive {
                    let prev = flag(ctx.core, cur, "skill_turn:l11_zhuwei_eff");
                    set_flag(ctx.core, cur, "skill_turn:l11_zhuwei_eff", prev + 1);
                }
            }
            Flow::Done
        }
        _ => {
            set_flag(ctx.core, owner, "l11_zhuwei_card", 0);
            Flow::Done
        }
    }
}

fn handle_chuhai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_CHUHAI_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive && !ctx.core.player(p).hand.is_empty())
            else {
                refund_once_used(ctx, owner, "txhj_chuhai");
                return Flow::Done;
            };
            m.target = Some(t);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            if ctx.core.player(owner).hand.is_empty() || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_CHUHAI_PINDIAN_OWNER)
        }
        HostStage::SelectCard => {
            m.card = None;
            let Some(_t) = m.target.filter(|&p| ctx.core.player(p).alive && !ctx.core.player(p).hand.is_empty()) else {
                return Flow::Done;
            };
            let Some(pc) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|c| ctx.core.player(owner).hand.contains(c))
            else {
                return Flow::Done;
            };
            m.cards = vec![pc];
            wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_CHUHAI_PINDIAN_TARGET)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                m.cards.clear();
                return Flow::Done;
            };
            let Some(pc) = m.cards.pop() else {
                return Flow::Done;
            };
            let Some(tc) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|c| ctx.core.player(t).hand.contains(c))
            else {
                return Flow::Done;
            };
            if ctx.core.player(owner).hand.contains(&pc) {
                discard_player_cards(ctx, owner, &[pc]);
            }
            if ctx.core.player(t).hand.contains(&tc) {
                discard_player_cards(ctx, t, &[tc]);
            }
            let my_raw = effective_card_rank(ctx.catalog, pc);
            let their_raw = effective_card_rank(ctx.catalog, tc);
            let eq_cnt = ctx.core.player(owner).equips.len();
            let bonus = if eq_cnt < 4 && my_raw < 13 {
                (4 - eq_cnt) as u8
            } else {
                0
            };
            let my_rank = my_raw.saturating_add(bonus).min(13);
            if my_rank > their_raw {
                set_flag(ctx.core, owner, "compare_win_turn", 1);
                set_flag(ctx.core, owner, "skill_turn:l11_chuhai2_target", u32::from(t) + 1);
                if let Some(def) = ctx.catalog.skill_by_name("txhj_chuhai2") {
                    ctx.ensure_skill(
                        (owner, def),
                        SkillInstance {
                            owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: true,
                            rounds: 0,
                        },
                    );
                }
                let mut types = Vec::new();
                for &hc in &ctx.core.player(t).hand {
                    let ty = card_type2(ctx.catalog.card_kind(hc));
                    if !types.contains(&ty) {
                        types.push(ty);
                    }
                }
                for ty in types {
                    let _ = gain_first_from_deck_or_discard(ctx, owner, |k| card_type2(k) == ty);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_chuhai2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage != HostStage::Act {
        return Flow::Done;
    }
    let owner = m.owner;
    let slots = empty_chuhai2_slots(ctx.core, ctx.catalog, owner);
    for slot in slots {
        let pick_idx = ctx
            .core
            .deck
            .iter()
            .rposition(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(slot))
            .map(|i| (true, i))
            .or_else(|| {
                ctx.core
                    .discard
                    .iter()
                    .rposition(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(slot))
                    .map(|i| (false, i))
            });
        if let Some((from_deck, pos)) = pick_idx {
            let cid = if from_deck {
                ctx.core.deck.remove(pos)
            } else {
                ctx.core.discard.remove(pos)
            };
            ctx.core.player_mut(owner).hand.push(cid);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::Equipped {
                player: owner,
                card: cid,
                replaced: None,
            });
            return Flow::Yield;
        }
    }
    Flow::Done
}

/// 关银屏 `xueji`（`sp.js:21742-21777`）：出牌阶段限一次，弃置一张红色牌，
/// 选择 1..=max(1, 已损体力值) 名角色横置，然后对其中首名角色造成 1 点火焰伤害。
fn handle_xueji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_XUEJI_CARD),
        HostStage::SelectCard => {
            m.card = None;
            let reds = xueji_red_cards(ctx.core, ctx.catalog, owner);
            let Some(cid) = answered_cards(ctx).into_iter().find(|c| reds.contains(c)) else {
                refund_once_used(ctx, owner, "xueji");
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_XUEJI_TARGET)
        }
        HostStage::TargetChoice => {
            m.card = None;
            let max_t = xueji_max_targets(ctx.core, owner);
            let mut chosen = Vec::new();
            for t in answered_players(ctx) {
                if ctx.core.player(t).alive && !chosen.contains(&t) {
                    chosen.push(t);
                    if chosen.len() >= max_t {
                        break;
                    }
                }
            }
            if chosen.is_empty() {
                m.cards.clear();
                refund_once_used(ctx, owner, "xueji");
                return Flow::Done;
            }
            let Some(cid) = m.cards.pop() else {
                refund_once_used(ctx, owner, "xueji");
                return Flow::Done;
            };
            if !he_cards(ctx.core, owner).contains(&cid) {
                refund_once_used(ctx, owner, "xueji");
                return Flow::Done;
            }
            m.targets = chosen;
            discard_player_cards(ctx, owner, &[cid]);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let mut any_linked = false;
            for &t in m.targets.iter().rev() {
                if ctx.core.player(t).alive && !ctx.core.player(t).linked {
                    ctx.emit(EventData::Linked {
                        player: t,
                        linked: true,
                    });
                    any_linked = true;
                }
            }
            if any_linked {
                m.stage = HostStage::SelectNature;
                Flow::Yield
            } else {
                if let Some(&t0) = m.targets.first() {
                    if ctx.core.player(t0).alive {
                        emit_damage(ctx, Some(owner), t0, 1, DamageNature::Fire, None);
                    }
                }
                Flow::Done
            }
        }
        HostStage::SelectNature => {
            if let Some(&t0) = m.targets.first() {
                if ctx.core.player(t0).alive {
                    emit_damage(ctx, Some(owner), t0, 1, DamageNature::Fire, None);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
