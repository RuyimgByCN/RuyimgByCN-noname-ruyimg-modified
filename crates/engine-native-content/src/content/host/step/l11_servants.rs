//! L11（太虚幻境）待援（S01–S04）、侍灵战斗技（S05–S28）与 `L11SubStates`（`l11_servants.rs`）：
//!
//! - `txhj_daiyuan` / `txhj_daiyuan2` / `txhj_daiyuanA` / `txhj_daiyuanB`（`extension.js:425-555`）
//! - `txhj_ruiSkill1`、`txhj_dundunSkill2`、`txhj_jiuweiSkill1..3`、`txhj_jiuweiSkill1_disable`
//! - `txhj_tengsheSkill1..3`、`txhj_yayaSkill1..2`、`txhj_yayaSkill2_protected`
//! - `txhj_youyouSkill1..2`、`txhj_qilinSkill1..3`、`txhj_minminSkill1..2`
//! - `txhj_ditingSkill1..3`、`txhj_qiaoqiaoSkill1..2`

use super::l11_common::*;
use super::l11_servants_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, DrawWindowKind, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

fn wait_srv_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_servant_ask(
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

pub(in crate::content::host) fn trigger_meta_l11_servants(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "txhj_daiyuan" => Some(meta(
            EventKind::TurnBegin,
            Timing::First,
            5,
            vec![EventKind::PhaseBegin],
        )),
        "txhj_daiyuan2" => Some(meta(EventKind::TurnEnded, Timing::Normal, 1, Vec::new())),
        "txhj_ruiSkill1" => None,
        "txhj_dundunSkill2" => Some(meta(EventKind::DamageBegin4, Timing::Normal, 1, Vec::new())),
        "txhj_jiuweiSkill1" => Some(meta(EventKind::TurnBegin, Timing::First, 5, Vec::new())),
        "txhj_jiuweiSkill1_disable" => {
            Some(meta(EventKind::CardUsed, Timing::First, 12, Vec::new()))
        }
        "txhj_jiuweiSkill2" => Some(meta(EventKind::CardUsed, Timing::First, 15, Vec::new())),
        "txhj_jiuweiSkill3" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_tengsheSkill1" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_tengsheSkill2" => Some(meta(
            EventKind::TurnPhaseAfter,
            Timing::Normal,
            1,
            Vec::new(),
        )),
        "txhj_tengsheSkill3" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_yayaSkill1" => Some(meta(EventKind::CardUsed, Timing::Normal, 1, Vec::new())),
        "txhj_yayaSkill2" => Some(meta(EventKind::Dying, Timing::Normal, 1, Vec::new())),
        "txhj_yayaSkill2_protected" => Some(meta(
            EventKind::TurnBegin,
            Timing::First,
            10,
            vec![EventKind::DamageBefore],
        )),
        "txhj_youyouSkill1" => Some(meta(EventKind::DamageAfter, Timing::Normal, 1, Vec::new())),
        "txhj_youyouSkill2" => Some(meta(
            EventKind::CardsDrawn,
            Timing::Normal,
            1,
            vec![EventKind::CardsMoved],
        )),
        "txhj_qilinSkill1" => Some(meta(EventKind::DrawPhaseAfter, Timing::Normal, 1, Vec::new())),
        "txhj_qilinSkill2" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_qilinSkill3" => Some(meta(
            EventKind::PhaseBegin,
            Timing::Normal,
            1,
            vec![EventKind::PhaseEnd],
        )),
        "txhj_minminSkill1" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_minminSkill2" => Some(meta(EventKind::CardsDrawn, Timing::Normal, 1, Vec::new())),
        "txhj_ditingSkill1" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_ditingSkill2" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_ditingSkill3" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        "txhj_qiaoqiaoSkill2" => Some(meta(EventKind::TurnPhaseEnd, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_servants(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_servants(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    if skill.name != "txhj_daiyuan2" {
        return None;
    }
    let state = ctx.state;
    let catalog = ctx.catalog;
    if state.turn.phase != Phase::Play || !is_unblocked_l11_owner(state, catalog, owner) {
        return None;
    }
    if has_named_skill(state, catalog, owner, "txhj_daiyuan") {
        return None;
    }
    if damaged_daiyuan_holders(state, catalog, owner).is_empty() {
        return None;
    }
    let can_a = !has_named_skill(state, catalog, owner, "txhj_daiyuanA")
        && has_diff_color_pair(state, catalog, owner);
    let can_b = !has_named_skill(state, catalog, owner, "txhj_daiyuanB")
        && has_diff_type2_pair(state, catalog, owner);
    if !can_a && !can_b {
        return None;
    }
    active_skill_machine("txhj_daiyuan2", HostBehavior::L11Daiyuan, owner)
}

pub(in crate::content::host) fn filter_l11_servants_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "txhj_youyouSkill2" => {
            if flag(state, owner, "l11_youyou2_ts") == turn_stamp(state) {
                return false;
            }
            let in_draw_phase = frames
                .iter()
                .any(|f| matches!(f.data, EventData::DrawPhase { player, .. } if player == owner));
            if in_draw_phase {
                return false;
            }
            let gained_gt1 = match event {
                EventData::CardsDrawn { player, cards } => *player == owner && cards.len() > 1,
                EventData::CardsMoved { to, cards, .. } => *to == owner && cards.len() > 1,
                _ => false,
            };
            gained_gt1
                && alive_all(state, owner).into_iter().any(|p| {
                    state.is_friend_of(catalog, owner, p)
                        && state.player(p).hp < state.player(p).max_hp as i32
                })
        }
        "txhj_qilinSkill1" => matches!(
            event,
            EventData::DrawPhase {
                player,
                actual_gain,
                ..
            } if *player == owner && *actual_gain > 0
        ),
        _ => filter_l11_servants(skill, ctx, owner, event),
    }
}

pub(in crate::content::host) fn filter_l11_servants(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "txhj_daiyuan" => match event {
            EventData::TurnBegin { .. } => true,
            EventData::Phase { begin: true, .. } => alive_all(state, owner).into_iter().any(|p| {
                !has_named_skill(state, catalog, p, "txhj_daiyuan")
                    && !has_named_skill(state, catalog, p, "txhj_daiyuan2")
            }),
            _ => false,
        },
        // `txhj_ruiSkill1` runs solely via `play_end_rui_skill1` (`ContentHooks::play_end_step`)
        // on real `Phase::Play { begin: false }` completion before `Phase::Discard`.
        "txhj_ruiSkill1" => false,
        "txhj_dundunSkill2" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner && !d.prevented && d.amount > 1
        }
        "txhj_jiuweiSkill1" => matches!(event, EventData::TurnBegin { .. }),
        "txhj_jiuweiSkill1_disable" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            u.player == owner
                && !u.cancelled
                && is_single_target_trick_use(catalog, u)
                && u.targets.first().copied().is_some_and(|t| {
                    state.player(t).alive
                        && has_named_skill(state, catalog, t, "txhj_jiuweiSkill1")
                        && !catalog.is_tieji_sealed(state, t)
                })
        }
        "txhj_jiuweiSkill2" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            state.turn.current == owner
                && u.player != owner
                && !u.cancelled
                && flag(state, owner, &format!("l11_jiuwei2_ts:{}", u.player)) != turn_stamp(state)
                && effective_kind_of_use(catalog, u)
                    .is_some_and(|k| k.is_basic_kind() || k.equip_slot().is_some())
        }
        "txhj_jiuweiSkill3" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && flag(state, owner, "l11_jiuwei3_count") < 9
        }
        "txhj_tengsheSkill1" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && d.nature == DamageNature::Normal
                && state.turn.current != owner
                && flag(state, owner, "l11_tengshe1_ts") != turn_stamp(state)
                && d.source.is_some_and(|s| s != owner && state.player(s).alive)
        }
        "txhj_tengsheSkill2" => {
            let EventData::TurnEndWindow {
                player,
                after: true,
            } = event
            else {
                return false;
            };
            *player == owner
                && alive_others(state, owner).into_iter().any(|p| {
                    !state.is_friend_of(catalog, owner, p)
                        && state.player(p).hp < state.player(p).max_hp as i32
                })
        }
        "txhj_tengsheSkill3" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && d.target != owner
                && d.nature == DamageNature::Thunder
                && !d.prevented
                && d.amount > 0
                && flag(state, owner, "l11_tengshe3_ts") != turn_stamp(state)
        }
        "txhj_yayaSkill1" => {
            let EventData::CardUsed(u) = event else {
                return false;
            };
            if u.cancelled || !(u.player == owner || u.targets.contains(&owner)) {
                return false;
            }
            let Some(cid) = u.card else {
                return false;
            };
            if !effective_color_is_red(state, catalog, u.player, cid) {
                return false;
            }
            let ts = turn_stamp(state);
            let used_turn = if flag(state, owner, "l11_yaya1_ts") == ts {
                flag(state, owner, "l11_yaya1_turn_cnt")
            } else {
                0
            };
            used_turn < 4
        }
        "txhj_yayaSkill2" => {
            let EventData::Dying { player, .. } = event else {
                return false;
            };
            *player == owner && flag(state, owner, "l11_yaya2_used") == 0
        }
        "txhj_yayaSkill2_protected" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::Damage(d) => d.target == owner && !d.prevented,
            _ => false,
        },
        "txhj_youyouSkill1" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && state.turn.current != owner
                && flag(state, owner, "l11_youyou1_ts") != turn_stamp(state)
        }
        "txhj_qilinSkill2" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            if d.source != Some(owner)
                || d.nature != DamageNature::Fire
                || d.prevented
                || d.amount <= 0
                || !state.player(d.target).alive
            {
                return false;
            }
            let ts = turn_stamp(state);
            let used = if flag(state, owner, "l11_qilin2_ts") == ts {
                flag(state, owner, "l11_qilin2_cnt")
            } else {
                0
            };
            used < 2
        }
        "txhj_qilinSkill3" => {
            let EventData::Phase {
                player,
                phase: Phase::Play,
                ..
            } = event
            else {
                return false;
            };
            *player == owner
                && alive_others(state, owner).into_iter().any(|p| {
                    !state.is_friend_of(catalog, owner, p) && !state.player(p).hand.is_empty()
                })
        }
        "txhj_minminSkill1" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner
                && !d.prevented
                && d.amount > 0
                && flag(state, owner, "l11_minmin1_ts") != turn_stamp(state)
        }
        "txhj_minminSkill2" => {
            let EventData::CardsDrawn { player, cards } = event else {
                return false;
            };
            if *player != owner || cards.len() < 2 {
                return false;
            }
            let ts = turn_stamp(state);
            let used = if flag(state, owner, "l11_minmin2_ts") == ts {
                flag(state, owner, "l11_minmin2_cnt")
            } else {
                0
            };
            used < 2
                && alive_all(state, owner).into_iter().any(|p| {
                    state.is_friend_of(catalog, owner, p)
                        && state.player(p).hp < state.player(p).max_hp as i32
                })
        }
        "txhj_ditingSkill1" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && state.player(owner).hp < state.player(owner).max_hp as i32
        }
        "txhj_ditingSkill2" => {
            let EventData::Damage(d) = event else {
                return false;
            };
            d.target == owner && !d.prevented && d.amount > 0 && !state.player(owner).infinite_hp()
        }
        "txhj_ditingSkill3" => {
            let EventData::TurnEndWindow {
                player,
                after: false,
            } = event
            else {
                return false;
            };
            *player == owner && flag(state, owner, "l11_turn_used_ranks_mask").count_ones() > 0
        }
        "txhj_qiaoqiaoSkill2" => {
            let EventData::TurnEndWindow {
                player,
                after: false,
            } = event
            else {
                return false;
            };
            *player == owner
                && flag(state, owner, "l11_qiaoqiao2_ts") != turn_stamp(state)
                && (state.player(owner).hand.len()
                    < (state.player(owner).hp.max(0) as usize).min(5))
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_servants(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let behavior = match skill.name.as_str() {
        "txhj_daiyuan" | "txhj_daiyuan2" | "txhj_daiyuanA" | "txhj_daiyuanB" => {
            HostBehavior::L11Daiyuan
        }
        "txhj_yayaSkill2_protected" => HostBehavior::L11SubStates,
        _ => HostBehavior::L11ServantSkills,
    };
    let mut target = None;
    let mut cards = Vec::new();
    match event {
        EventData::Damage(d) => {
            target = Some(d.target);
            cards.push(d.amount.max(0) as u32);
            if let Some(s) = d.source {
                cards.push(u32::from(s));
            }
        }
        EventData::TurnBegin { player } => {
            target = Some(*player);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior,
        owner,
        target,
        card: None,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

fn daiyuan_mode_flag(core: &GameStateCore, owner: PlayerId) -> u32 {
    flag(core, owner, "l11_daiyuan_chosen_mode")
}

pub(in crate::content::host) fn handle_l11_servants(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_daiyuan" => {
            for p in alive_all(ctx.core, owner) {
                if !has_named_skill(ctx.core, ctx.catalog, p, "txhj_daiyuan") {
                    ensure_perm_skill(ctx, p, "txhj_daiyuan2");
                }
            }
            Flow::Done
        }
        "txhj_daiyuan2" => match m.stage {
            HostStage::Act => {
                let can_a = !has_named_skill(ctx.core, ctx.catalog, owner, "txhj_daiyuanA")
                    && has_diff_color_pair(ctx.core, ctx.catalog, owner);
                let can_b = !has_named_skill(ctx.core, ctx.catalog, owner, "txhj_daiyuanB")
                    && has_diff_type2_pair(ctx.core, ctx.catalog, owner);
                if !can_a && !can_b {
                    return Flow::Done;
                }
                if can_a && can_b {
                    return wait_srv_ask(m, ctx, HostStage::SelectNature, ASK_DAIYUAN_MODE);
                }
                let mode = if can_a { 1 } else { 2 };
                set_flag(ctx.core, owner, "l11_daiyuan_chosen_mode", mode);
                wait_srv_ask(m, ctx, HostStage::SelectCard, ASK_DAIYUAN_CARDS)
            }
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(t)) = ctx.answer().first().copied() else {
                    set_flag(ctx.core, owner, "l11_daiyuan_chosen_mode", 0);
                    return Flow::Done;
                };
                set_flag(ctx.core, owner, "l11_daiyuan_chosen_mode", u32::from(t) + 1);
                wait_srv_ask(m, ctx, HostStage::SelectCard, ASK_DAIYUAN_CARDS)
            }
            HostStage::SelectCard => {
                let chosen = answered_cards(ctx);
                if chosen.len() != 2 {
                    set_flag(ctx.core, owner, "l11_daiyuan_chosen_mode", 0);
                    return Flow::Done;
                }
                let mode = daiyuan_mode_flag(ctx.core, owner);
                let valid = if mode == 1 {
                    effective_color_is_red(ctx.core, ctx.catalog, owner, chosen[0])
                        != effective_color_is_red(ctx.core, ctx.catalog, owner, chosen[1])
                } else {
                    card_type2(ctx.catalog, chosen[0]) != card_type2(ctx.catalog, chosen[1])
                };
                if !valid {
                    return wait_srv_ask(m, ctx, HostStage::SelectCard, ASK_DAIYUAN_CARDS);
                }
                let holders = damaged_daiyuan_holders(ctx.core, ctx.catalog, owner);
                if holders.is_empty() {
                    set_flag(ctx.core, owner, "l11_daiyuan_chosen_mode", 0);
                    return Flow::Done;
                }
                discard_player_cards(ctx, owner, &chosen);
                if mode == 1 {
                    ensure_perm_skill(ctx, owner, "txhj_daiyuanA");
                } else {
                    ensure_perm_skill(ctx, owner, "txhj_daiyuanB");
                }
                set_flag(ctx.core, owner, "l11_daiyuan_chosen_mode", 0);
                if holders.len() == 1 {
                    ctx.recover(holders[0], 1, None);
                    return Flow::Done;
                }
                wait_srv_ask(m, ctx, HostStage::TargetChoice, ASK_DAIYUAN_TARGET)
            }
            HostStage::TargetChoice => {
                if let Some(t) = answered_players(ctx).into_iter().next() {
                    ctx.recover(t, 1, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_ruiSkill1" => {
            let my_hand = ctx.core.player(owner).hand.len();
            let targets: Vec<PlayerId> = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| {
                    !ctx.core.is_friend_of(ctx.catalog, owner, p)
                        && ctx.core.player(p).hand.len() <= my_hand
                })
                .collect();
            for t in targets {
                emit_damage(ctx, Some(owner), t, 1, DamageNature::Fire, None);
            }
            Flow::Done
        }
        "txhj_dundunSkill2" => {
            with_current_damage_mut(ctx, |dm| {
                if dm.amount > 1 {
                    dm.amount -= 1;
                }
            });
            Flow::Done
        }
        "txhj_jiuweiSkill1" => {
            let tp = m.target.unwrap_or(ctx.core.turn.current);
            set_flag(ctx.core, tp, "l11_jiuwei1_holder", u32::from(owner) + 1);
            if tp != owner {
                ensure_perm_skill(ctx, tp, "txhj_jiuweiSkill1_disable");
            }
            Flow::Done
        }
        "txhj_jiuweiSkill1_disable" => {
            if let Some(u) = current_use_card_mut(ctx) {
                u.targets.clear();
                u.cancelled = true;
            }
            Flow::Done
        }
        "txhj_jiuweiSkill2" => {
            if let Some(u) = current_use_card_mut(ctx) {
                let actor = u.player;
                u.cancelled = true;
                let ts = turn_stamp(ctx.core);
                set_flag(ctx.core, owner, &format!("l11_jiuwei2_ts:{actor}"), ts);
            }
            Flow::Done
        }
        "txhj_jiuweiSkill3" => {
            let cnt = flag(ctx.core, owner, "l11_jiuwei3_count");
            if cnt < 9 {
                let dmg = m.cards.first().copied().unwrap_or(1).max(1);
                let draw_n = (9 - cnt) * dmg;
                set_flag(ctx.core, owner, "l11_jiuwei3_count", cnt + 1);
                ctx.start_draw(owner, draw_n);
            }
            Flow::Done
        }
        "txhj_tengsheSkill1" => {
            set_flag(ctx.core, owner, "l11_tengshe1_ts", turn_stamp(ctx.core));
            if let Some(src) = m.cards.get(1).copied().map(|v| v as PlayerId) {
                if ctx.core.player(src).alive {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![src],
                        nature: DamageNature::Thunder,
                        use_as: Some(CardKind::LeiSha),
                        cancelled: false,
                    }));
                }
            }
            Flow::Done
        }
        "txhj_tengsheSkill2" => {
            let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                .collect();
            let dmg = if enemies.len() == 1 { 2 } else { 1 };
            for e in enemies {
                emit_damage(ctx, Some(owner), e, dmg, DamageNature::Thunder, None);
            }
            Flow::Done
        }
        "txhj_tengsheSkill3" => {
            set_flag(ctx.core, owner, "l11_tengshe3_ts", turn_stamp(ctx.core));
            ctx.recover(owner, 1, None);
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "txhj_yayaSkill1" => {
            let ts = turn_stamp(ctx.core);
            let turn_cnt = if flag(ctx.core, owner, "l11_yaya1_ts") == ts {
                flag(ctx.core, owner, "l11_yaya1_turn_cnt")
            } else {
                0
            };
            set_flag(ctx.core, owner, "l11_yaya1_ts", ts);
            set_flag(ctx.core, owner, "l11_yaya1_turn_cnt", turn_cnt + 1);
            let total = flag(ctx.core, owner, "l11_yaya1_total") + 1;
            ctx.start_draw(owner, 1);
            if total >= 4 {
                set_flag(ctx.core, owner, "l11_yaya1_total", 0);
                let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                    .collect();
                if !enemies.is_empty() {
                    let pick = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
                    emit_damage(
                        ctx,
                        Some(owner),
                        enemies[pick],
                        1,
                        DamageNature::Normal,
                        None,
                    );
                }
            } else {
                set_flag(ctx.core, owner, "l11_yaya1_total", total);
            }
            Flow::Done
        }
        "txhj_yayaSkill2" => {
            set_flag(ctx.core, owner, "l11_yaya2_used", 1);
            let cur_hp = ctx.core.player(owner).hp;
            if cur_hp < 1 {
                let need = (1 - cur_hp) as u32;
                ctx.recover(owner, need, None);
            }
            ensure_perm_skill(ctx, owner, "txhj_yayaSkill2_protected");
            set_flag(ctx.core, owner, "l11_yaya_protected", 1);
            Flow::Done
        }
        "txhj_yayaSkill2_protected" => {
            let fid = ctx.current_event_frame_id();
            if let Some(fr) = ctx.res.frame_mut(fid) {
                match &mut fr.data {
                    EventData::TurnBegin { player } if *player == owner => {
                        set_flag(ctx.core, owner, "l11_yaya_protected", 0);
                        remove_named_skill(ctx, owner, "txhj_yayaSkill2_protected");
                    }
                    EventData::Damage(d) if d.target == owner => {
                        d.prevented = true;
                        ctx.out.push(LogEvent::DamagePrevented {
                            target: owner,
                            presentation: None,
                        });
                    }
                    _ => {}
                }
            }
            Flow::Done
        }
        "txhj_youyouSkill1" => {
            set_flag(ctx.core, owner, "l11_youyou1_ts", turn_stamp(ctx.core));
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "txhj_youyouSkill2" => {
            set_flag(ctx.core, owner, "l11_youyou2_ts", turn_stamp(ctx.core));
            let injured: Vec<PlayerId> = alive_all(ctx.core, owner)
                .into_iter()
                .filter(|&p| {
                    ctx.core.is_friend_of(ctx.catalog, owner, p)
                        && ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32
                })
                .collect();
            if !injured.is_empty() {
                let pick = ctx.core.rng.gen_range(injured.len() as u32) as usize;
                ctx.recover(injured[pick], 1, None);
            }
            Flow::Done
        }
        "txhj_qilinSkill1" => {
            let gain = ctx
                .res
                .stack
                .iter()
                .rev()
                .find_map(|f| match f.data {
                    EventData::DrawWindow {
                        root,
                        window: DrawWindowKind::After,
                    } => ctx.res.frame(root).and_then(|rf| match rf.data {
                        EventData::DrawPhase { actual_gain, .. } => Some(actual_gain),
                        _ => None,
                    }),
                    EventData::DrawPhase { actual_gain, .. } => Some(actual_gain),
                    _ => None,
                })
                .unwrap_or(0);
            if gain > 0 {
                ctx.start_draw(owner, gain);
            }
            Flow::Done
        }
        "txhj_qilinSkill2" => match m.stage {
            HostStage::Act => {
                let ts = turn_stamp(ctx.core);
                let cnt = if flag(ctx.core, owner, "l11_qilin2_ts") == ts {
                    flag(ctx.core, owner, "l11_qilin2_cnt")
                } else {
                    0
                };
                set_flag(ctx.core, owner, "l11_qilin2_ts", ts);
                set_flag(ctx.core, owner, "l11_qilin2_cnt", cnt + 1);
                ctx.recover(owner, 1, None);
                let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                    return Flow::Done;
                };
                let pool = he_cards(ctx.core, t);
                if pool.is_empty() {
                    return Flow::Done;
                }
                if pool.len() == 1 {
                    move_cards_to_hand(ctx, t, owner, &[pool[0]]);
                    return Flow::Done;
                }
                wait_srv_ask(m, ctx, HostStage::SelectCard, ASK_QILIN2_CARD)
            }
            HostStage::SelectCard => {
                if let (Some(t), Some(c)) = (
                    m.target.filter(|&p| ctx.core.player(p).alive),
                    answered_cards(ctx).into_iter().next(),
                ) {
                    move_cards_to_hand(ctx, t, owner, &[c]);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_qilinSkill3" => {
            let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| {
                    !ctx.core.is_friend_of(ctx.catalog, owner, p)
                        && !ctx.core.player(p).hand.is_empty()
                })
                .collect();
            if !enemies.is_empty() {
                let pick = ctx.core.rng.gen_range(enemies.len() as u32) as usize;
                set_flag(ctx.core, owner, VIRTUAL_HUOGONG_FLAG, 1);
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![enemies[pick]],
                    nature: DamageNature::Fire,
                    use_as: Some(CardKind::Wuzhong),
                    cancelled: false,
                }));
            }
            Flow::Done
        }
        "txhj_minminSkill1" => {
            set_flag(ctx.core, owner, "l11_minmin1_ts", turn_stamp(ctx.core));
            let dmg = m.cards.first().copied().unwrap_or(1).max(1);
            ctx.start_draw(owner, dmg);
            Flow::Done
        }
        "txhj_minminSkill2" => {
            let ts = turn_stamp(ctx.core);
            let cnt = if flag(ctx.core, owner, "l11_minmin2_ts") == ts {
                flag(ctx.core, owner, "l11_minmin2_cnt")
            } else {
                0
            };
            set_flag(ctx.core, owner, "l11_minmin2_ts", ts);
            set_flag(ctx.core, owner, "l11_minmin2_cnt", cnt + 1);
            let injured: Vec<PlayerId> = alive_all(ctx.core, owner)
                .into_iter()
                .filter(|&p| {
                    ctx.core.is_friend_of(ctx.catalog, owner, p)
                        && ctx.core.player(p).hp < ctx.core.player(p).max_hp as i32
                })
                .collect();
            if !injured.is_empty() {
                let pick = ctx.core.rng.gen_range(injured.len() as u32) as usize;
                ctx.recover(injured[pick], 1, None);
            }
            Flow::Done
        }
        "txhj_ditingSkill1" => {
            ctx.recover(owner, 1, None);
            Flow::Done
        }
        "txhj_ditingSkill2" => {
            if !ctx.core.player(owner).infinite_hp() {
                ctx.core.player_mut(owner).max_hp += 1;
            }
            Flow::Done
        }
        "txhj_ditingSkill3" => match m.stage {
            HostStage::Act => {
                let x = flag(ctx.core, owner, "l11_turn_used_ranks_mask").count_ones();
                if x == 0 {
                    return Flow::Done;
                }
                m.cards = vec![x];
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, x);
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let x = m.cards.first().copied().unwrap_or(0) as usize;
                let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                    .take(x)
                    .collect();
                for e in enemies {
                    emit_damage(ctx, Some(owner), e, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txhj_qiaoqiaoSkill2" => {
            set_flag(ctx.core, owner, "l11_qiaoqiao2_ts", turn_stamp(ctx.core));
            let target_n = (ctx.core.player(owner).hp.max(0) as usize).min(5);
            let cur_n = ctx.core.player(owner).hand.len();
            if cur_n < target_n {
                ctx.start_draw(owner, (target_n - cur_n) as u32);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

/// `phaseUseEnd` 钩子（`extension.js:2053`）：仅在真实出牌阶段收束压入下一阶段后、
/// 弃牌/下阶段运行前（LIFO 栈顶 `Park`）触发 `txhj_ruiSkill1`，跳过出牌阶段不会触发。
pub(crate) fn play_end_rui_skill1(ctx: &mut Ctx<'_>, player: PlayerId) {
    super::l11_boss_c2::clear_chuhai2_on_play_end(ctx, player);
    let Some(def) = ctx.catalog.skill_by_name("txhj_ruiSkill1") else {
        return;
    };
    if !ctx.core.has_skill(player, def)
        || !ctx.core.player(player).alive
        || ctx.catalog.is_tieji_sealed(ctx.core, player)
    {
        return;
    }
    let my_hand = ctx.core.player(player).hand.len();
    let has_fewer = alive_others(ctx.core, player).into_iter().any(|p| {
        !ctx.core.is_friend_of(ctx.catalog, player, p)
            && ctx.core.player(p).hand.len() < my_hand
    });
    if !has_fewer {
        return;
    }
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: "txhj_ruiSkill1".to_string(),
            behavior: HostBehavior::L11ServantSkills,
            owner: player,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }),
    });
}
