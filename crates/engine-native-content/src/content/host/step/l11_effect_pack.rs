//! L11 太虚幻境 `effectPack` 6 个突发技能与 `txhj_yingjiuhandi` SP2 武将技：
//!
//! - `kuanggu`（`shenhua.js:6489-6500`）
//! - `hongyan`（`shenhua.js:6887-6893`，花色改判核心在 `Catalog::effective_card_suit` / `effective_judge_suit_rank`）
//! - `txhj_tanshi`（`extension.js:3187-3199`）
//! - `chanyuan`（`shenhua.js:7693-7724`，技能封锁在 `Catalog::is_chanyuan_sealed`）
//! - `xingshang`（`shenhua.js:4734-4746`）
//! - `hengzheng`（`sp.js:21054-21102`）
//! - `xinfu_yisuan` / `xinfu_lveming` / `xinfu_tunjun` / `xinfu_tanbei` / `xinfu_sidao`（`sp2/skill.js:15630-15950`）

use super::l11_boss_c::wait_bc_ask;
use super::l11_boss_c2::pop_deck_with_reshuffle;
use super::l11_common::*;
use super::l11_effect_pack_asks::*;
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, SkillLimits, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{
    CardUseData, DamageNature, DrawOutcome, EventData, EventKind, Frame, Phase, Timing,
};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(super) fn trigger_meta_l11_effect_pack(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match name {
        "kuanggu" => Some(meta(EventKind::DamageEnd, Timing::Normal, 0)),
        "hongyan" | "chanyuan" => Some(meta(EventKind::DrawPhase, Timing::Normal, 0)),
        "txhj_tanshi" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 0)),
        "xingshang" => Some(meta(EventKind::Death, Timing::Normal, 0)),
        "hengzheng" => Some(meta(EventKind::DrawPhaseBegin1, Timing::Normal, 0)),
        _ => None,
    }
}

pub(super) fn limits_l11_effect_pack(name: &str) -> SkillLimits {
    if matches!(name, "xinfu_yisuan" | "xinfu_lveming" | "xinfu_tanbei") {
        SkillLimits {
            once_per_turn: true,
            active_outside_play: false,
        }
    } else {
        SkillLimits::default()
    }
}

fn is_ordinary_trick_use(catalog: &crate::catalog::Catalog, u: &CardUseData) -> bool {
    let Some(kind) = u.use_as.or_else(|| u.card.map(|cid| catalog.card_kind(cid))) else {
        return false;
    };
    !kind.is_basic_kind()
        && kind.equip_slot().is_none()
        && !matches!(kind, CardKind::Le | CardKind::Bing | CardKind::ShanDian)
}

pub(super) fn filter_l11_effect_pack(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    _frames: &[Frame],
    _frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    match skill.name.as_str() {
        "kuanggu" => {
            if !is_locked_unblocked_l11_owner(state, ctx.catalog, owner) {
                return false;
            }
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && state.player(owner).hp < state.player(owner).max_hp as i32
                && (d.target == owner
                    || state.computed_distance_including_dead(ctx.catalog, owner, d.target) <= 1)
        }
        "hongyan" | "chanyuan" => false,
        "txhj_tanshi" => {
            is_locked_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::End,
                        begin: true
                    } if *player == owner
                )
                && !state.player(owner).hand.is_empty()
        }
        "xingshang" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Death { player, .. }
                        if *player != owner
                            && !state.player(*player).death_cancelled
                            && (!state.player(*player).hand.is_empty()
                                || !state.player(*player).equips.is_empty())
                )
        }
        "hengzheng" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::DrawPhase {
                        player,
                        pending,
                        outcome: DrawOutcome::Running,
                        ..
                    } if *player == owner && *pending > 0
                )
                && (state.player(owner).hp == 1 || state.player(owner).hand.is_empty())
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !hej_cards(state, p).is_empty())
        }
        _ => false,
    }
}

/// `useCardAfter` 弹出钩子：`xinfu_yisuan`（`sp2/skill.js:15630-15670`）与 `xinfu_sidao`（`sp2/skill.js:15876-15944`）。
pub(in crate::content::host) fn on_card_use_after_effect_pack(
    ctx: &mut Ctx<'_>,
    u: &CardUseData,
) {
    if u.cancelled {
        return;
    }
    on_card_use_after_yisuan(ctx, u);
    on_card_use_after_sidao(ctx, u);
}

fn on_card_use_after_yisuan(ctx: &mut Ctx<'_>, u: &CardUseData) {
    let Some(cid) = u.card else {
        return;
    };
    let owner = u.player;
    if !ctx.core.player(owner).alive
        || !has_named_skill(ctx.core, ctx.catalog, owner, "xinfu_yisuan")
        || !is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
        || ctx.core.turn.current != owner
        || ctx.core.turn.phase != Phase::Play
        || ctx
            .catalog
            .skill_by_name("xinfu_yisuan")
            .is_some_and(|id| ctx.core.skills.get(&(owner, id)).is_some_and(|s| s.once_used))
        || !is_ordinary_trick_use(ctx.catalog, u)
        || (!ctx.core.discard.contains(&cid) && !ctx.core.table.contains(&cid))
    {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "xinfu_yisuan".to_string(),
            behavior: HostBehavior::L11GeneralBossC,
            owner,
            target: None,
            card: None,
            cards: vec![cid],
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
}

fn sidao_card_use_targets(ctx: &Ctx<'_>, u: &CardUseData) -> Vec<PlayerId> {
    if !u.targets.is_empty() {
        return u.targets.clone();
    }
    let eff = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
    match eff {
        CardKind::Wugu | CardKind::Taoyuan => ctx.core.seat_order_from(u.player),
        _ => Vec::new(),
    }
}

fn on_card_use_after_sidao(ctx: &mut Ctx<'_>, u: &CardUseData) {
    let owner = u.player;
    if !has_named_skill(ctx.core, ctx.catalog, owner, "xinfu_sidao") {
        return;
    }
    let cur_targets = sidao_card_use_targets(ctx, u);
    let in_play = ctx.core.turn.current == owner && ctx.core.turn.phase == Phase::Play;
    let prev_packed = flag(ctx.core, owner, "skill_turn:xinfu_sidao_prev_targets");
    let mut cur_mask = 0u32;
    if in_play && !cur_targets.is_empty() {
        for &t in &cur_targets {
            if let Some(bit) = 1u32.checked_shl(u32::from(t)) {
                cur_mask |= bit;
            }
        }
        set_flag(
            ctx.core,
            owner,
            "skill_turn:xinfu_sidao_prev_targets",
            (cur_mask << 1) | 1,
        );
    } else {
        set_flag(ctx.core, owner, "skill_turn:xinfu_sidao_prev_targets", 0);
        return;
    }
    if (prev_packed & 1) == 0 {
        return;
    }
    let prev_mask = prev_packed >> 1;
    let mut seen = 0u32;
    let shared: Vec<PlayerId> = cur_targets
        .into_iter()
        .filter(|&t| {
            let bit = 1u32.checked_shl(u32::from(t)).unwrap_or(0);
            if bit != 0 && (prev_mask & bit) != 0 && (seen & bit) == 0 {
                seen |= bit;
                true
            } else {
                false
            }
        })
        .collect();
    if shared.is_empty()
        || !ctx.core.player(owner).alive
        || !is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
        || flag(ctx.core, owner, "skill_turn:xinfu_sidaoy") != 0
        || ctx.core.player(owner).hand.is_empty()
        || sidao_usable_hand_cards(ctx.core, ctx.catalog, owner, &shared).is_empty()
    {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "xinfu_sidao".to_string(),
            behavior: HostBehavior::L11GeneralBossC,
            owner,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: shared,
            stage: HostStage::Act,
        }),
    });
}

pub(super) fn activate_l11_effect_pack(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    if state.turn.phase != Phase::Play || !is_unblocked_l11_owner(state, ctx.catalog, owner) {
        return None;
    }
    let eligible = match skill.name.as_str() {
        "xinfu_lveming" => {
            let my_eq = state.player(owner).equips.len();
            alive_others(state, owner)
                .into_iter()
                .any(|p| state.player(p).equips.len() < my_eq)
        }
        "xinfu_tunjun" => {
            flag(state, owner, "l11_xinfu_tunjun_used") == 0
                && flag(state, owner, "l11_xinfu_lveming_cnt") > 0
        }
        "xinfu_tanbei" => !alive_others(state, owner).is_empty(),
        _ => false,
    };
    if eligible {
        active_skill_machine(&skill.name, HostBehavior::L11GeneralBossC, owner)
    } else {
        None
    }
}

pub(super) fn handle_l11_effect_pack(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "kuanggu" => {
            if let Some(EventData::Damage(d)) = ctx
                .res
                .stack
                .iter()
                .rev()
                .find_map(|f| match &f.data {
                    EventData::Damage(_) => Some(&f.data),
                    _ => None,
                })
                .cloned()
            {
                let cur_hp = ctx.core.player(owner).hp;
                let max_hp = ctx.core.player(owner).max_hp as i32;
                if cur_hp < max_hp && d.amount > 0 {
                    ctx.recover(owner, d.amount as u32, None);
                }
            }
            Flow::Done
        }
        "txhj_tanshi" => handle_tanshi(m, ctx),
        "xingshang" => handle_xingshang(m, ctx),
        "hengzheng" => handle_hengzheng(m, ctx),
        "xinfu_yisuan" => handle_xinfu_yisuan(m, ctx),
        "xinfu_lveming" => handle_xinfu_lveming(m, ctx),
        "xinfu_tunjun" => handle_xinfu_tunjun(m, ctx),
        "xinfu_tanbei" => handle_xinfu_tanbei(m, ctx),
        "xinfu_sidao" => handle_xinfu_sidao(m, ctx),
        _ => Flow::Done,
    }
}

fn handle_tanshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::CardsAsk, ASK_EP_TANSHI_DISCARD),
        HostStage::CardsAsk => {
            m.card = None;
            let picked = answered_cards(ctx);
            if !picked.is_empty() {
                discard_player_cards(ctx, owner, &picked);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_xingshang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_EP_XINGSHANG_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            let Some(victim) = m.target else {
                return Flow::Done;
            };
            let he = he_cards(ctx.core, victim);
            if !he.is_empty() {
                move_cards_to_hand(ctx, victim, owner, &he);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_hengzheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_EP_HENGZHENG_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
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
                if let EventData::DrawPhase {
                    pending, outcome, ..
                } = &mut fr.data
                {
                    *pending = 0;
                    *outcome = DrawOutcome::Skipped;
                }
            }
            m.targets = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| !hej_cards(ctx.core, p).is_empty())
                .collect();
            hengzheng_next_target(m, ctx)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(ChoiceOption::Tag(zone)) = ctx.answer().first().copied() else {
                return hengzheng_next_target(m, ctx);
            };
            let Some(t) = m.target else {
                return hengzheng_next_target(m, ctx);
            };
            hengzheng_resolve_zone(m, ctx, t, u16::from(zone))
        }
        HostStage::CardsAsk => {
            m.card = None;
            m.cards.clear();
            let Some(t) = m.target else {
                return hengzheng_next_target(m, ctx);
            };
            let picked = answered_cards(ctx);
            if !picked.is_empty() && ctx.core.player(t).alive {
                move_cards_to_hand(ctx, t, owner, &picked);
            }
            hengzheng_next_target(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn hengzheng_resolve_zone(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    t: PlayerId,
    zone: u16,
) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(t).alive {
        return hengzheng_next_target(m, ctx);
    }
    match zone {
        0 => {
            let hand_len = ctx.core.player(t).hand.len();
            if hand_len > 0 {
                let idx = ctx.core.rng.gen_range(hand_len as u32) as usize;
                let cid = ctx.core.player(t).hand[idx];
                move_cards_to_hand(ctx, t, owner, &[cid]);
            }
            hengzheng_next_target(m, ctx)
        }
        1 | 2 => {
            m.cards = vec![u32::from(zone)];
            wait_bc_ask(m, ctx, HostStage::CardsAsk, ASK_EP_HENGZHENG_CARD)
        }
        _ => hengzheng_next_target(m, ctx),
    }
}

fn hengzheng_next_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while !m.targets.is_empty() {
        let t = m.targets.remove(0);
        if !ctx.core.player(t).alive {
            continue;
        }
        let (has_hand, has_equip, has_judge, hand_len) = {
            let pl = ctx.core.player(t);
            (
                !pl.hand.is_empty(),
                !pl.equips.is_empty(),
                !pl.judge.is_empty(),
                pl.hand.len(),
            )
        };
        let non_empty_zones = (has_hand as u8) + (has_equip as u8) + (has_judge as u8);
        if non_empty_zones == 0 {
            continue;
        }
        m.target = Some(t);
        if non_empty_zones == 1 {
            if hand_len > 0 {
                let idx = ctx.core.rng.gen_range(hand_len as u32) as usize;
                let cid = ctx.core.player(t).hand[idx];
                move_cards_to_hand(ctx, t, m.owner, &[cid]);
                continue;
            }
            let zone = if has_equip { 1 } else { 2 };
            return hengzheng_resolve_zone(m, ctx, t, zone);
        }
        return wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_EP_HENGZHENG_ZONE);
    }
    Flow::Done
}

fn handle_xinfu_yisuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let used_card = m.card;
            if let Some(c) = used_card {
                m.cards = vec![c];
            }
            wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_EP_YISUAN_CONFIRM)
        }
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            if let Some(def) = ctx.catalog.skill_by_name("xinfu_yisuan") {
                if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
                    inst.once_used = true;
                }
            }
            ctx.lose_max_hp(owner, 1);
            if ctx.core.player(owner).max_hp == 0 && ctx.core.player(owner).alive {
                m.stage = HostStage::SelectNature;
                ctx.emit(EventData::Death {
                    player: owner,
                    killer: None,
                });
                return Flow::Yield;
            }
            if let Some(&cid) = m.cards.first() {
                let mut gained = false;
                if let Some(pos) = ctx.core.discard.iter().position(|&c| c == cid) {
                    ctx.core.discard.remove(pos);
                    gained = true;
                } else if let Some(pos) = ctx.core.table.iter().position(|&c| c == cid) {
                    ctx.core.table.remove(pos);
                    gained = true;
                }
                if gained && ctx.core.player(owner).alive {
                    let delivered = (ctx.catalog.hooks.on_before_gain_hand)(ctx, None, owner, &[cid]);
                    ctx.core.player_mut(owner).hand.extend(delivered.iter().copied());
                    if !delivered.is_empty() {
                        ctx.out.push(LogEvent::CardsTaken {
                            from: owner,
                            to: owner,
                            cards: delivered,
                            public: true,
                        });
                    }
                }
            }
            Flow::Done
        }
        HostStage::SelectNature => Flow::Done,
        _ => Flow::Done,
    }
}

fn handle_xinfu_lveming(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_EP_LVEMING_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "xinfu_lveming");
                return Flow::Done;
            };
            m.target = Some(tgt);
            wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_EP_LVEMING_RANK)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let chosen_rank = ((idx.min(12)) + 1) as u32;
            m.cards = vec![chosen_rank];
            let cnt = flag(ctx.core, owner, "l11_xinfu_lveming_cnt") + 1;
            set_flag(ctx.core, owner, "l11_xinfu_lveming_cnt", cnt);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            if !ctx.core.player(tgt).alive {
                return Flow::Done;
            }
            let chosen_rank = m.cards.first().copied().unwrap_or(1);
            let judge_rank = flag(ctx.core, owner, "last_judge_rank");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            if judge_rank == chosen_rank {
                emit_damage(ctx, Some(owner), tgt, 2, DamageNature::Normal, None);
            } else {
                let pool = hej_cards(ctx.core, tgt);
                if !pool.is_empty() {
                    let pick = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
                    move_cards_to_hand(ctx, tgt, owner, &[pick]);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_xinfu_tunjun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_EP_TUNJUN_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let count = flag(ctx.core, owner, "l11_xinfu_lveming_cnt") as usize;
            if count == 0 || !ctx.core.player(tgt).alive {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "l11_xinfu_tunjun_used", 1);
            m.target = Some(tgt);
            let mut chosen_slots: Vec<EquipSlot> = Vec::new();
            let mut collected = Vec::new();
            for _ in 0..count {
                let pos = ctx.core.deck.iter().rposition(|&c| {
                    let Some(slot) = ctx.catalog.card_slot_of(ctx.core, c) else {
                        return false;
                    };
                    if chosen_slots.contains(&slot) || collected.contains(&c) {
                        return false;
                    }
                    let tgt_has_slot = ctx
                        .core
                        .player(tgt)
                        .equips
                        .iter()
                        .any(|&e| ctx.catalog.card_slot_of(ctx.core, e) == Some(slot));
                    !tgt_has_slot
                });
                let Some(idx) = pos else { break };
                let cid = ctx.core.deck.remove(idx);
                if let Some(slot) = ctx.catalog.card_slot_of(ctx.core, cid) {
                    chosen_slots.push(slot);
                }
                collected.push(cid);
            }
            if collected.is_empty() {
                let _ = pop_deck_with_reshuffle(ctx.core).map(|c| ctx.core.deck.push(c));
                return Flow::Done;
            }
            m.cards = collected;
            tunjun_equip_next(m, ctx)
        }
        HostStage::DrawSettle => tunjun_equip_next(m, ctx),
        _ => Flow::Done,
    }
}

fn tunjun_equip_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(tgt) = m.target else {
        return Flow::Done;
    };
    if !ctx.core.player(tgt).alive || m.cards.is_empty() {
        for c in m.cards.drain(..) {
            ctx.to_discard(c);
        }
        return Flow::Done;
    }
    let cid = m.cards.remove(0);
    ctx.core.player_mut(tgt).hand.push(cid);
    m.stage = HostStage::DrawSettle;
    ctx.emit(EventData::Equipped {
        player: tgt,
        card: cid,
        replaced: None,
    });
    Flow::Yield
}

fn handle_xinfu_tanbei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_EP_TANBEI_TARGET),
        HostStage::TargetChoice => {
            m.card = None;
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "xinfu_tanbei");
                return Flow::Done;
            };
            m.target = Some(tgt);
            if hej_cards(ctx.core, tgt).is_empty() {
                set_flag(
                    ctx.core,
                    owner,
                    &format!("skill_turn:xinfu_tanbei_unlimited:{tgt}"),
                    1,
                );
                return Flow::Done;
            }
            wait_bc_ask(m, ctx, HostStage::SelectNature, ASK_EP_TANBEI_MODE)
        }
        HostStage::SelectNature => {
            m.card = None;
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(0)) => 0,
                _ => 1,
            };
            if mode == 0 {
                let pool = hej_cards(ctx.core, tgt);
                if !pool.is_empty() {
                    let cid = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
                    move_cards_to_hand(ctx, tgt, owner, &[cid]);
                }
                set_flag(
                    ctx.core,
                    owner,
                    &format!("skill_turn:xinfu_tanbei_forbid:{tgt}"),
                    1,
                );
            } else {
                set_flag(
                    ctx.core,
                    owner,
                    &format!("skill_turn:xinfu_tanbei_unlimited:{tgt}"),
                    1,
                );
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn sidao_range_free(
    core: &GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    cid: CardId,
    target: PlayerId,
) -> bool {
    if crate::rvm_bridge::modifiers::tanbei_unlimited_target(core, catalog, owner, target) {
        return true;
    }
    if crate::rvm_bridge::modifiers::eval_card_distance_unlimited(
        core,
        catalog,
        owner,
        cid,
        CardKind::Snatch,
        DamageNature::Normal,
    ) {
        return true;
    }
    if catalog
        .skill_by_name(crate::content::standard::QI_CAI)
        .is_some_and(|d| core.has_skill(owner, d))
        || catalog
            .skill_by_name(crate::content::standard::BOSS_SHOUYI)
            .is_some_and(|d| core.has_skill(owner, d))
    {
        return true;
    }
    if catalog
        .skill_by_name(crate::content::standard::BOSS_SHENEN)
        .is_some_and(|d| core.skills.keys().any(|(_, sd)| *sd == d))
        && core.is_boss_side(owner)
    {
        return true;
    }
    catalog
        .skill_by_name(crate::content::standard::BOSS_FENTIAN)
        .is_some_and(|d| {
            core.has_skill(owner, d)
                && matches!(
                    catalog.effective_card_suit(core, owner, cid),
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                )
        })
}

fn sidao_can_use_card(
    core: &GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    cid: CardId,
) -> bool {
    core.player(owner).hand.contains(&cid)
        && !core.player(owner).jileng_active
        && !crate::rvm_bridge::modifiers::eval_card_use_canceled(
            core,
            catalog,
            owner,
            cid,
            CardKind::Snatch,
            DamageNature::Normal,
        )
}

fn sidao_can_target_with_card(
    core: &GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    cid: CardId,
    target: PlayerId,
) -> bool {
    if target == owner || !core.player(target).alive {
        return false;
    }
    let tp = core.player(target);
    if tp.hand.is_empty() && tp.equips.is_empty() {
        return false;
    }
    if tp.jidong_active || tp.yinshen_active || tp.jileng_active || tp.yinmi_active {
        return false;
    }
    if catalog
        .skill_by_name(crate::content::standard::QIAN_XUN)
        .is_some_and(|d| core.has_skill(target, d))
    {
        return false;
    }
    if catalog
        .skill_by_name(crate::content::host::GJ_FOOD_SHIHUIFEN_BAN_KEY)
        .and_then(|d| core.skills.get(&(owner, d)))
        .is_some_and(|s| s.enabled)
    {
        return false;
    }
    if !sidao_range_free(core, catalog, owner, cid, target)
        && core.computed_distance(catalog, owner, target) > 1
    {
        return false;
    }
    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core,
        catalog,
        owner,
        target,
        cid,
        CardKind::Snatch,
        DamageNature::Normal,
    )
}

fn sidao_legal_targets_for_card(
    core: &GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    cid: CardId,
    shared_targets: &[PlayerId],
) -> Vec<PlayerId> {
    shared_targets
        .iter()
        .copied()
        .filter(|&t| sidao_can_target_with_card(core, catalog, owner, cid, t))
        .collect()
}

fn sidao_usable_hand_cards(
    core: &GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    shared_targets: &[PlayerId],
) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&cid| {
            sidao_can_use_card(core, catalog, owner, cid)
                && !sidao_legal_targets_for_card(core, catalog, owner, cid, shared_targets)
                    .is_empty()
        })
        .collect()
}

fn handle_xinfu_sidao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if !ctx.core.player(owner).alive
                || !is_unblocked_l11_owner(ctx.core, ctx.catalog, owner)
                || flag(ctx.core, owner, "skill_turn:xinfu_sidaoy") != 0
            {
                return Flow::Done;
            }
            let usable = sidao_usable_hand_cards(ctx.core, ctx.catalog, owner, &m.targets);
            if usable.is_empty() {
                return Flow::Done;
            }
            m.cards = usable;
            wait_bc_ask(m, ctx, HostStage::SelectCard, ASK_EP_SIDAO_CARD)
        }
        HostStage::SelectCard => {
            m.card = None;
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !sidao_can_use_card(ctx.core, ctx.catalog, owner, cid) {
                return Flow::Done;
            }
            let legal = sidao_legal_targets_for_card(ctx.core, ctx.catalog, owner, cid, &m.targets);
            if legal.is_empty() {
                return Flow::Done;
            }
            m.cards = vec![cid];
            m.targets = legal;
            wait_bc_ask(m, ctx, HostStage::TargetChoice, ASK_EP_SIDAO_TARGET)
        }
        HostStage::TargetChoice => {
            m.card = None;
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let Some(cid) = m.cards.first().copied() else {
                return Flow::Done;
            };
            if !sidao_can_use_card(ctx.core, ctx.catalog, owner, cid)
                || !sidao_can_target_with_card(ctx.core, ctx.catalog, owner, cid, tgt)
            {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "skill_turn:xinfu_sidaoy", 1);
            if let Some(def) = ctx.catalog.skill_by_name("xinfu_sidao") {
                ctx.out.push(LogEvent::SkillTriggered {
                    skill: ctx.catalog.skills[def as usize].name,
                    owner,
                    presentation: None,
                });
            }
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                cid,
                owner,
                vec![tgt],
                Some(CardKind::Snatch),
            )));
            Flow::Yield
        }
        HostStage::DrawSettle => Flow::Done,
        _ => Flow::Done,
    }
}
