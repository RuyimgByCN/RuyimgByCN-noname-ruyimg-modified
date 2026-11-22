//! L11（太虚幻境）武将技能 Group F（`L11GeneralSkillsF` tag 732）：
//!
//! - `txhj_suoming`、`txhj_huangmen`、`txhj_mouqiang`、`txhj_suzhi`、`txmanjia`、`txmanjia1`、`txmanjia2`
//! - `txmanzhen`、`txfusha`、`txleizhou`、`txzhennu`、`txyongguan`、`txyongguan_fate`
//! - `txchengyuan`、`txbenji`、`txyonglve`、`txjiang`、`txhj_oldhuxiao`

use super::l11_common::*;
use super::l11_common_b2::*;
use super::l11_skills_e_asks::find_red_or_black_color_val;
use super::l11_skills_f_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

fn wait_f_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_skills_f_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        m.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        m.card = None;
        Flow::Done
    }
}

pub(in crate::content::host) fn trigger_meta_l11_skills_f(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "txhj_suoming" | "txhj_huangmen" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new()))
        }
        "txhj_mouqiang" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_suzhi" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            2,
            vec![
                EventKind::PhaseBegin,
                EventKind::DamageBegin1,
                EventKind::CardUsed,
                EventKind::CardsDiscarded,
            ],
        )),
        "txmanzhen" => Some(meta(EventKind::DamageBegin1, Timing::Normal, 1, Vec::new())),
        "txfusha" => Some(meta(EventKind::DamageBegin1, Timing::Normal, 1, Vec::new())),
        "txleizhou" | "txzhennu" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txyongguan" => Some(meta(EventKind::DamageBegin4, Timing::Normal, 1, Vec::new())),
        "txyongguan_fate" => Some(meta(EventKind::TurnEnded, Timing::Normal, 1, Vec::new())),
        "txchengyuan" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txbenji" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            1,
            vec![EventKind::Dying],
        )),
        "txjiang" => Some(meta(EventKind::CardUsed, Timing::Normal, 1, Vec::new())),
        "txhj_oldhuxiao" => Some(meta(EventKind::ShaDodged, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_skills_f(_name: &str) -> SkillLimits {
    // `txmanzhen` active once-per-turn is gated by `skill_turn:l11_manzhen_used` so its
    // `DamageBegin1` trigger still runs after activation (`trigger_order` checks `once_used`).
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_skills_f(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    if skill.name != "txmanzhen" {
        return None;
    }
    let state = ctx.state;
    if state.turn.phase != Phase::Play
        || !is_unblocked_l11_owner(state, ctx.catalog, owner)
        || flag(state, owner, "skill_turn:l11_manzhen_used") > 0
        || state.player(owner).hand.is_empty()
        || manzhen_valid_targets(state, ctx.catalog, owner).is_empty()
    {
        return None;
    }
    active_skill_machine(
        "txmanzhen",
        HostBehavior::L11GeneralSkillsF,
        owner,
    )
}

pub(in crate::content::host) fn filter_l11_skills_f_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "txhj_suoming" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::End,
                        begin: true
                    } if *player == owner
                )
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).linked)
        }
        "txhj_huangmen" => {
            matches!(
                event,
                EventData::Phase {
                    player,
                    phase: Phase::Prepare | Phase::End,
                    begin: true
                } if *player == owner
            ) && state.player(owner).hand.is_empty()
        }
        "txhj_mouqiang" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Damage(d)
                        if !d.prevented && d.target == owner && d.amount > 1 && d.source.is_some()
                )
        }
        "txhj_suzhi" => {
            let under_cap = flag(state, owner, "skill_turn:l11_suzhi_marks") < 3;
            match event {
                EventData::TurnBegin { player } => {
                    *player == owner && flag(state, owner, "l11_suzhi_fankui") > 0
                }
                EventData::Phase {
                    player,
                    phase: Phase::End,
                    begin: true,
                } => *player == owner && under_cap,
                EventData::Damage(d) => {
                    under_cap
                        && !d.prevented
                        && d.amount > 0
                        && state.turn.current == owner
                        && d.source == Some(owner)
                        && matches!(
                            damage_effective_kind(state, frames, frame, ctx.catalog, d),
                            Some(
                                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Duel
                            )
                        )
                }
                EventData::CardUsed(u) => {
                    under_cap
                        && !u.cancelled
                        && u.player == owner
                        && state.turn.current == owner
                        && {
                            let k = u.effective_kind(ctx.catalog);
                            !k.is_basic_kind() && k.equip_slot().is_none() && k.is_basic_trick_or_equip()
                        }
                }
                EventData::CardsDiscarded { player, cards } => {
                    under_cap
                        && state.turn.current == owner
                        && *player != owner
                        && state.player(*player).alive
                        && !cards.is_empty()
                        && !he_cards(state, *player).is_empty()
                }
                _ => false,
            }
        }
        "txmanzhen" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Damage(d)
                        if !d.prevented
                            && d.amount > 0
                            && d.source == Some(owner)
                            && state.player(d.target).hp >= state.player(owner).hp
                            && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                                == Some(CardKind::SavageAttack)
                )
        }
        "txfusha" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && !d.from_chain
                    && d.source == Some(owner)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        .is_some_and(|k| k.is_sha_like())
        ),
        "txleizhou" => {
            matches!(
                event,
                EventData::Phase {
                    player,
                    phase: Phase::Prepare,
                    begin: true
                } if *player == owner
            ) && !alive_others(state, owner).is_empty()
        }
        "txzhennu" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
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
        "txyongguan" => matches!(
            event,
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.target == owner
        ),
        "txyongguan_fate" => {
            matches!(event, EventData::TurnEnded { player } if *player == owner)
                && flag(state, owner, "l11_yongguan_marks") > 0
        }
        "txchengyuan" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(state, owner, "skill_turn:l11_chengyuan_used") > 0,
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: true,
            } => {
                *player == owner
                    && is_unblocked_l11_owner(state, ctx.catalog, owner)
                    && {
                        let used_mask = flag(state, owner, "skill_turn:l11_chengyuan_used");
                        state.player(owner).hand.iter().any(|&c| {
                            (used_mask
                                & find_red_or_black_color_val(ctx.catalog, state, owner, c))
                                == 0
                        })
                    }
            }
            _ => false,
        },
        "txbenji" => match event {
            EventData::Dying { player, .. } => {
                flag(state, owner, "skill_turn:l11_benji_watch_tgt") == u32::from(*player) + 1
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && !alive_all(state, owner).is_empty(),
            _ => false,
        },
        "txjiang" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && flag(state, owner, "skill_turn:l11_txjiang_used") == 0
                && matches!(
                    event,
                    EventData::CardUsed(u)
                        if !u.cancelled
                            && matches!(
                                u.effective_kind(ctx.catalog),
                                CardKind::Sha | CardKind::FireSha | CardKind::LeiSha | CardKind::Duel
                            )
                            && (u.targets.contains(&owner)
                                || (u.player == owner && !u.targets.is_empty()))
                )
        }
        "txhj_oldhuxiao" => matches!(event, EventData::ShaDodged { source, .. } if *source == owner),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_skills_f(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    match event {
        EventData::Damage(d) => {
            target = Some(d.target);
            card = Some(3);
            cards.push(d.amount.max(0) as u32);
            if let Some(s) = d.source {
                cards.push(u32::from(s));
            }
        }
        EventData::CardUsed(_) => card = Some(4),
        EventData::CardsDiscarded { player, .. } => {
            target = Some(*player);
            card = Some(5);
        }
        EventData::TurnBegin { .. } => card = Some(1),
        EventData::Phase { .. } => card = Some(2),
        EventData::Dying { player, .. } => {
            target = Some(*player);
            card = Some(6);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_skills_f(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_suoming" => step_txhj_suoming(m, ctx),
        "txhj_huangmen" => step_txhj_huangmen(m, ctx),
        "txhj_mouqiang" => step_txhj_mouqiang(m, ctx),
        "txhj_suzhi" => step_txhj_suzhi(m, ctx),
        "txmanzhen" => step_txmanzhen(m, ctx),
        "txfusha" => {
            with_current_damage_mut(ctx, |dm| dm.amount += 1);
            Flow::Done
        }
        "txleizhou" => {
            let others = alive_others(ctx.core, owner);
            if !others.is_empty() {
                let tgt = others[ctx.core.rng.gen_range(others.len() as u32) as usize];
                emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Thunder, None);
            }
            Flow::Done
        }
        "txzhennu" => match m.stage {
            HostStage::Act => wait_f_ask(m, ctx, HostStage::TargetChoice, ASK_ZHENNU_TARGET),
            HostStage::TargetChoice => {
                if let Some(tgt) = answered_players(ctx).into_iter().next() {
                    emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txyongguan" => {
            let amt = m.cards.first().copied().unwrap_or(0);
            with_current_damage_mut(ctx, |dm| {
                dm.prevented = true;
                dm.amount = 0;
            });
            if amt > 0 {
                let prev = flag(ctx.core, owner, "l11_yongguan_marks");
                set_flag(ctx.core, owner, "l11_yongguan_marks", prev + amt);
            }
            Flow::Done
        }
        "txyongguan_fate" => step_txyongguan_fate(m, ctx),
        "txchengyuan" => step_txchengyuan(m, ctx),
        "txbenji" => step_txbenji(m, ctx),
        "txjiang" => match m.stage {
            HostStage::Act => wait_f_ask(m, ctx, HostStage::TargetChoice, ASK_TXJIANG_CONFIRM),
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    set_flag(ctx.core, owner, "skill_turn:l11_txjiang_used", 1);
                    ctx.start_draw(owner, 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_oldhuxiao" => {
            refund_one_sha_use(ctx, owner);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_suoming(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_f_ask(m, ctx, HostStage::TargetChoice, ASK_SUOMING_TARGETS),
        HostStage::TargetChoice => {
            for t in answered_players(ctx) {
                if ctx.core.player(t).alive && !ctx.core.player(t).linked {
                    ctx.emit(EventData::Linked {
                        player: t,
                        linked: true,
                    });
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_huangmen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_f_ask(m, ctx, HostStage::SelectNature, ASK_HUANGMEN_MODE),
        HostStage::SelectNature => {
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(t)) => t,
                _ => 0,
            };
            if mode == 0 {
                ctx.start_draw(owner, 2);
            } else {
                let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                    .collect();
                if !enemies.is_empty() {
                    let tgt = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
                    let pool = he_cards(ctx.core, tgt);
                    if !pool.is_empty() {
                        let cid = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
                        move_cards_to_hand(ctx, tgt, owner, &[cid]);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_mouqiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_f_ask(m, ctx, HostStage::TargetChoice, ASK_MOUQIANG_TARGET),
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            let want = m.cards.first().copied().unwrap_or(2) / 2;
            if want > 0 && !he_cards(ctx.core, tgt).is_empty() {
                return wait_f_ask(m, ctx, HostStage::CardsAsk, ASK_MOUQIANG_HE);
            }
            emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Normal, None);
            Flow::Done
        }
        HostStage::CardsAsk => {
            let Some(tgt) = m.target else {
                return Flow::Done;
            };
            let picked = answered_cards(ctx);
            let dmg_src = m.cards.get(1).copied().map(|v| v as PlayerId);
            if tgt != owner {
                move_cards_to_hand(ctx, tgt, owner, &picked);
            }
            m.target = dmg_src;
            m.cards = picked;
            m.stage = HostStage::DrawSettle;
            emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Normal, None);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let dmg_src = m.target;
            let picked = std::mem::take(&mut m.cards);
            for c in picked {
                if ctx.catalog.card_kind(c).is_basic_kind() {
                    if ctx.core.player(owner).alive {
                        ctx.recover(owner, 1, None);
                    }
                } else if let Some(src) = dmg_src {
                    if ctx.core.player(src).alive {
                        emit_damage(ctx, Some(owner), src, 1, DamageNature::Normal, None);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_suzhi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::CardsAsk {
        if let (Some(tgt), Some(cid)) = (m.target, answered_cards(ctx).into_iter().next()) {
            move_cards_to_hand(ctx, tgt, owner, &[cid]);
        }
        return Flow::Done;
    }
    match m.card {
        Some(1) => {
            if flag(ctx.core, owner, "l11_suzhi_fankui") > 0 {
                set_flag(ctx.core, owner, "l11_suzhi_fankui", 0);
                if let Some(def) = ctx
                    .catalog
                    .skill_by_name("fankui")
                    .or_else(|| ctx.catalog.skill_by_name(crate::vocabulary::names::FAN_KUI))
                {
                    ctx.remove_skill(owner, def);
                }
            }
            Flow::Done
        }
        Some(2) => {
            let fankui_def = ctx
                .catalog
                .skill_by_name("fankui")
                .or_else(|| ctx.catalog.skill_by_name(crate::vocabulary::names::FAN_KUI));
            if let Some(def) = fankui_def {
                if !ctx.core.has_skill(owner, def) {
                    set_flag(ctx.core, owner, "l11_suzhi_fankui", 1);
                    ctx.ensure_skill(
                        (owner, def),
                        crate::state::SkillInstance {
                            owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
            Flow::Done
        }
        Some(3) => {
            let marks = flag(ctx.core, owner, "skill_turn:l11_suzhi_marks");
            if marks < 3 {
                with_current_damage_mut(ctx, |dm| dm.amount += 1);
                set_flag(ctx.core, owner, "skill_turn:l11_suzhi_marks", marks + 1);
            }
            Flow::Done
        }
        Some(4) => {
            let marks = flag(ctx.core, owner, "skill_turn:l11_suzhi_marks");
            if marks < 3 {
                set_flag(ctx.core, owner, "skill_turn:l11_suzhi_marks", marks + 1);
                ctx.start_draw(owner, 1);
            }
            Flow::Done
        }
        Some(5) => {
            let marks = flag(ctx.core, owner, "skill_turn:l11_suzhi_marks");
            if marks < 3 {
                set_flag(ctx.core, owner, "skill_turn:l11_suzhi_marks", marks + 1);
                return wait_f_ask(m, ctx, HostStage::CardsAsk, ASK_SUZHI_HE);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txmanzhen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.card == Some(3) {
        return wait_f_ask(m, ctx, HostStage::SelectNature, ASK_MANZHEN_CONFIRM_DMG);
    }
    match m.stage {
        HostStage::Act => wait_f_ask(m, ctx, HostStage::CardsAsk, ASK_MANZHEN_CARDS),
        HostStage::CardsAsk => {
            let picked = answered_cards(ctx);
            if picked.is_empty() {
                set_flag(ctx.core, owner, "skill_turn:l11_manzhen_used", 0);
                return Flow::Done;
            }
            m.cards = picked;
            wait_f_ask(m, ctx, HostStage::TargetChoice, ASK_MANZHEN_TARGETS)
        }
        HostStage::TargetChoice => {
            let tgts = answered_players(ctx);
            if tgts.len() != m.cards.len() || tgts.is_empty() {
                set_flag(ctx.core, owner, "skill_turn:l11_manzhen_used", 0);
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "skill_turn:l11_manzhen_used", 1);
            let cards = std::mem::take(&mut m.cards);
            for c in cards {
                if ctx.core.player(owner).hand.contains(&c) {
                    ctx.take_hand_card(owner, c);
                    ctx.to_discard(c);
                }
            }
            emit_virtual_kind_use(ctx, owner, CardKind::SavageAttack, tgts, DamageNature::Normal);
            Flow::Done
        }
        HostStage::SelectNature => {
            if is_confirmed(ctx) {
                with_current_damage_mut(ctx, |dm| dm.amount += 1);
                emit_lose_hp(ctx, owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txyongguan_fate(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.core.player(owner).hand.is_empty() {
                let marks = flag(ctx.core, owner, "l11_yongguan_marks");
                set_flag(ctx.core, owner, "l11_yongguan_marks", 0);
                if marks > 0 {
                    emit_lose_hp(ctx, owner, marks);
                }
                return Flow::Done;
            }
            wait_f_ask(m, ctx, HostStage::CardsAsk, ASK_YONGGUAN_DISCARD)
        }
        HostStage::CardsAsk => {
            let picked = answered_cards(ctx);
            let marks = flag(ctx.core, owner, "l11_yongguan_marks");
            let remain = marks.saturating_sub(picked.len() as u32);
            set_flag(ctx.core, owner, "l11_yongguan_marks", 0);
            if !picked.is_empty() {
                discard_player_cards(ctx, owner, &picked);
            }
            if remain > 0 && ctx.core.player(owner).alive {
                emit_lose_hp(ctx, owner, remain);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txchengyuan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && ctx.core.turn.phase == Phase::Prepare {
        set_flag(ctx.core, owner, "skill_turn:l11_chengyuan_used", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_f_ask(m, ctx, HostStage::CardsAsk, ASK_CHENGYUAN_CARD),
        HostStage::CardsAsk => {
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&cid) {
                return Flow::Done;
            }
            ctx.take_hand_card(owner, cid);
            let pile = ctx
                .core
                .vm_aux
                .piles
                .entry((owner, CHENGYUAN_PILE.to_string()))
                .or_default();
            pile.push(cid);
            if pile.len() < 3 {
                return Flow::Done;
            }
            let thrown = std::mem::take(pile);
            for &c in &thrown {
                ctx.to_discard(c);
            }
            ctx.out.push(LogEvent::CardDiscarded {
                player: owner,
                cards: thrown.clone(),
            });
            ctx.emit(EventData::CardsDiscarded {
                player: owner,
                cards: thrown,
            });
            if alive_others(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_f_ask(m, ctx, HostStage::TargetChoice, ASK_CHENGYUAN_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            m.target = Some(tgt);
            m.stage = HostStage::DrawSettle;
            emit_damage(ctx, Some(owner), tgt, 2, DamageNature::Normal, None);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if let Some(tgt) = m.target {
                if ctx.core.player(tgt).alive {
                    let eq = ctx.core.player(tgt).equips.clone();
                    if !eq.is_empty() {
                        discard_player_cards(ctx, tgt, &eq);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txbenji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.card == Some(6) {
        set_flag(ctx.core, owner, "skill_turn:l11_benji_saw_dying", 1);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_f_ask(m, ctx, HostStage::TargetChoice, ASK_BENJI_TARGET),
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
            if lost == 0 {
                emit_lose_hp(ctx, owner, 1);
                return Flow::Done;
            }
            m.target = Some(tgt);
            set_flag(ctx.core, owner, "skill_turn:l11_benji_saw_dying", 0);
            set_flag(
                ctx.core,
                owner,
                "skill_turn:l11_benji_watch_tgt",
                u32::from(tgt) + 1,
            );
            m.stage = HostStage::DrawSettle;
            emit_damage(ctx, Some(owner), tgt, lost, DamageNature::Normal, None);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let saw_dying = flag(ctx.core, owner, "skill_turn:l11_benji_saw_dying") > 0;
            set_flag(ctx.core, owner, "skill_turn:l11_benji_watch_tgt", 0);
            set_flag(ctx.core, owner, "skill_turn:l11_benji_saw_dying", 0);
            if let Some(tgt) = m.target {
                if ctx.core.player(tgt).alive && !saw_dying && ctx.core.player(owner).alive {
                    emit_lose_hp(ctx, owner, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
