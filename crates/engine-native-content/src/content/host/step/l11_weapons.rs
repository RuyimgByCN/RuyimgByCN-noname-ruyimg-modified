//! L11（太虚幻境）Batch L11-5 武器覆写技（`L11Weapons` tag 740：`l11_weapons.rs`）。
//!
//! 承载 `extension_skill.js:658..1136` 的 6 个武器技：
//! - `hanbing_skill`（`damageBegin3` 寒冰剑：弃2 / 拂尘荡魔弃3 + 深寒雪饮获得）
//! - `cixiong_skill`（`useCardToPlayered` 雌雄双股剑：异性目标弃1 / 拂尘荡魔弃2 + 深寒雪饮）
//! - `guanshi_skill`（`shaMiss` 贯石斧：弃2 / 上将的膂力弃1 强制命中含酒加成及火雷属性）
//! - `sanjian_skill`（`damageSource` 三尖两刃刀：弃1 / 上将的膂力免弃 对距离<=1另一目标1伤）
//! - `yitianjian`（`damageSource` 倚天剑：受伤且有手牌时弃1或上将免弃回1血）
//! - `pyzhuren_diamond`（`damageBegin1` 烈淬刃：每回合限2次，弃1/上将免弃+1伤，取消退还次数）

use super::l11_common::*;
use super::l11_common_b2::damage_effective_kind;
use super::l11_weapons_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{DamageData, DamageNature, EventData, EventKind, Frame, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::rvm_bridge::modifiers::{has_fuchendangmo, has_shangjiangdelvli, has_shenhanxueyin};
use crate::state::GameStateCore;

fn is_sha_damage(
    state: &GameStateCore,
    frames: &[Frame],
    frame: Option<&Frame>,
    catalog: &crate::catalog::Catalog,
    d: &DamageData,
) -> bool {
    damage_effective_kind(state, frames, frame, catalog, d).is_some_and(CardKind::is_sha_like)
}

fn gain_discarded_if_shenhanxueyin(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    cards: &[CardId],
) {
    if !has_shenhanxueyin(ctx.core, ctx.catalog, owner) || !ctx.core.player(owner).alive {
        return;
    }
    let mut gained = Vec::new();
    for &c in cards {
        if let Some(pos) = ctx.core.discard.iter().position(|&dc| dc == c) {
            ctx.core.discard.remove(pos);
            ctx.core.player_mut(owner).hand.push(c);
            gained.push(c);
        }
    }
    if !gained.is_empty() {
        ctx.out.push(LogEvent::CardsDrawn {
            player: owner,
            cards: gained.clone(),
        });
        ctx.emit(EventData::CardsDrawn {
            player: owner,
            cards: gained,
        });
    }
}

fn wait_w_ask(m: &mut HostMachine, ctx: &mut Ctx<'_>, stage: HostStage, ask_id: u32) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_weapons_ask(
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

pub(in crate::content::host) fn trigger_meta_l11_weapons(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match name {
        "hanbing_skill" => Some(meta(EventKind::DamageBegin3, Timing::Normal, 1)),
        "cixiong_skill" => Some(meta(EventKind::CardUsed, Timing::Normal, 1)),
        "guanshi_skill" => Some(meta(EventKind::ShaDodged, Timing::Normal, 1)),
        "sanjian_skill" | "yitianjian" => Some(meta(EventKind::DamageSource, Timing::Normal, 1)),
        "pyzhuren_diamond" => Some(meta(EventKind::DamageBegin1, Timing::Normal, 1)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_weapons(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn filter_l11_weapons_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: Option<&Frame>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !is_unblocked_l11_owner(state, ctx.catalog, owner) {
        return false;
    }
    match skill.name.as_str() {
        "hanbing_skill" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && !d.from_chain
                    && d.amount > 0
                    && state.player(d.target).alive
                    && !he_cards(state, d.target).is_empty()
                    && is_sha_damage(state, frames, frame, ctx.catalog, d)
        ),
        "cixiong_skill" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && is_sha_use(ctx.catalog, u)
                    && u.targets.iter().any(|&t| {
                        t != owner
                            && state.player(t).alive
                            && state.different_gender_from(ctx.catalog, owner, t)
                    })
        ),
        "guanshi_skill" => {
            let EventData::ShaDodged { source, target, .. } = event else {
                return false;
            };
            let need = if has_shangjiangdelvli(state, ctx.catalog, owner) {
                1
            } else {
                2
            };
            *source == owner
                && state.player(*target).alive
                && guanshi_discard_pool(state, ctx.catalog, owner).len() >= need
        }
        "sanjian_skill" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && !d.from_chain
                    && d.amount > 0
                    && state.player(d.target).alive
                    && is_sha_damage(state, frames, frame, ctx.catalog, d)
                    && !sanjian_targets(state, ctx.catalog, owner, d.target).is_empty()
                    && (has_shangjiangdelvli(state, ctx.catalog, owner)
                        || !state.player(owner).hand.is_empty())
        ),
        "yitianjian" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && is_sha_damage(state, frames, frame, ctx.catalog, d)
                    && state.player(owner).hp < state.player(owner).max_hp as i32
                    && !state.player(owner).hand.is_empty()
        ),
        "pyzhuren_diamond" => matches!(
            event,
            EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && flag(state, owner, "skill_turn:l11_pyzhuren_used") < 2
                    && is_sha_damage(state, frames, frame, ctx.catalog, d)
                    && (has_shangjiangdelvli(state, ctx.catalog, owner)
                        || !pyzhuren_discard_pool(state, ctx.catalog, owner).is_empty())
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_weapons(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, card, cards, targets) = match event {
        EventData::Damage(d) => (Some(d.target), d.card, Vec::new(), Vec::new()),
        EventData::CardUsed(u) => (None, u.card, Vec::new(), u.targets.clone()),
        EventData::ShaDodged { target, card, .. } => {
            (Some(*target), *card, card.into_iter().copied().collect(), Vec::new())
        }
        _ => (None, None, Vec::new(), Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11Weapons,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn activate_l11_weapons(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    None
}

pub(in crate::content::host) fn handle_l11_weapons(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "hanbing_skill" => handle_hanbing(m, ctx),
        "cixiong_skill" => handle_cixiong(m, ctx),
        "guanshi_skill" => handle_guanshi(m, ctx),
        "sanjian_skill" => handle_sanjian(m, ctx),
        "yitianjian" => handle_yitianjian(m, ctx),
        "pyzhuren_diamond" => handle_pyzhuren(m, ctx),
        _ => Flow::Done,
    }
}

fn handle_hanbing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
        set_flag(ctx.core, owner, "l11_hanbing_rem", 0);
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => wait_w_ask(m, ctx, HostStage::TargetChoice, ASK_HANBING_CONFIRM),
        HostStage::TargetChoice => {
            m.card = None;
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            with_current_damage_mut(ctx, |dm| {
                dm.prevented = true;
                dm.amount = 0;
            });
            let total = if has_fuchendangmo(ctx.core, ctx.catalog, owner) {
                3
            } else {
                2
            };
            set_flag(ctx.core, owner, "l11_hanbing_rem", total);
            if he_cards(ctx.core, t).is_empty() {
                set_flag(ctx.core, owner, "l11_hanbing_rem", 0);
                return Flow::Done;
            }
            wait_w_ask(m, ctx, HostStage::SelectCard, ASK_HANBING_PICK)
        }
        HostStage::SelectCard => {
            m.card = None;
            let pick = answered_cards(ctx).into_iter().next();
            if let Some(cid) = pick {
                if he_cards(ctx.core, t).contains(&cid) {
                    discard_player_cards(ctx, t, &[cid]);
                    gain_discarded_if_shenhanxueyin(ctx, owner, &[cid]);
                }
            }
            let rem = flag(ctx.core, owner, "l11_hanbing_rem").saturating_sub(1);
            set_flag(ctx.core, owner, "l11_hanbing_rem", rem);
            if rem == 0 || !ctx.core.player(t).alive || he_cards(ctx.core, t).is_empty() {
                set_flag(ctx.core, owner, "l11_hanbing_rem", 0);
                return Flow::Done;
            }
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if flag(ctx.core, owner, "l11_hanbing_rem") == 0
                || !ctx.core.player(t).alive
                || he_cards(ctx.core, t).is_empty()
            {
                set_flag(ctx.core, owner, "l11_hanbing_rem", 0);
                return Flow::Done;
            }
            wait_w_ask(m, ctx, HostStage::SelectCard, ASK_HANBING_PICK)
        }
        _ => {
            set_flag(ctx.core, owner, "l11_hanbing_rem", 0);
            Flow::Done
        }
    }
}

fn handle_cixiong(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            while let Some(next_t) = m.targets.first().copied() {
                m.targets.remove(0);
                if next_t != owner
                    && ctx.core.player(next_t).alive
                    && ctx.core.different_gender_from(ctx.catalog, owner, next_t)
                {
                    m.target = Some(next_t);
                    return wait_w_ask(m, ctx, HostStage::TargetChoice, ASK_CIXIONG_CONFIRM);
                }
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                m.stage = HostStage::Act;
                return handle_cixiong(m, ctx);
            };
            if !is_confirmed(ctx) {
                m.stage = HostStage::Act;
                return handle_cixiong(m, ctx);
            }
            let need = if has_fuchendangmo(ctx.core, ctx.catalog, owner) {
                2
            } else {
                1
            };
            if ctx.core.player(t).hand.len() < need {
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(owner, 1);
                return Flow::Yield;
            }
            wait_w_ask(m, ctx, HostStage::SelectCard, ASK_CIXIONG_TARGET_DISCARD)
        }
        HostStage::SelectCard => {
            m.card = None;
            let Some(t) = m.target else {
                m.stage = HostStage::Act;
                return handle_cixiong(m, ctx);
            };
            let need = if has_fuchendangmo(ctx.core, ctx.catalog, owner) {
                2
            } else {
                1
            };
            let picks: Vec<CardId> = answered_cards(ctx)
                .into_iter()
                .filter(|c| ctx.core.player(t).hand.contains(c))
                .collect();
            if picks.len() == need {
                discard_player_cards(ctx, t, &picks);
                gain_discarded_if_shenhanxueyin(ctx, owner, &picks);
                m.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            m.stage = HostStage::Act;
            handle_cixiong(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn handle_guanshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_w_ask(m, ctx, HostStage::SelectCard, ASK_GUANSHI_DISCARD),
        HostStage::SelectCard => {
            m.card = None;
            let need = if has_shangjiangdelvli(ctx.core, ctx.catalog, owner) {
                1
            } else {
                2
            };
            let pool = guanshi_discard_pool(ctx.core, ctx.catalog, owner);
            let picks: Vec<CardId> = answered_cards(ctx)
                .into_iter()
                .filter(|c| pool.contains(c))
                .collect();
            if picks.len() != need {
                return Flow::Done;
            }
            discard_player_cards(ctx, owner, &picks);
            if m.target.is_some_and(|t| ctx.core.player(t).alive) {
                set_flag(ctx.core, owner, "l11_guanshi_force_hit", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_sanjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if has_shangjiangdelvli(ctx.core, ctx.catalog, owner) {
                wait_w_ask(m, ctx, HostStage::TargetChoice, ASK_SANJIAN_TARGET)
            } else {
                wait_w_ask(m, ctx, HostStage::SelectCard, ASK_SANJIAN_DISCARD)
            }
        }
        HostStage::SelectCard => {
            m.card = None;
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&cid) {
                return Flow::Done;
            }
            m.cards = vec![cid];
            wait_w_ask(m, ctx, HostStage::TargetChoice, ASK_SANJIAN_TARGET)
        }
        HostStage::TargetChoice => {
            m.card = None;
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| ctx.core.player(p).alive)
            else {
                return Flow::Done;
            };
            let Some(damaged) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            if !sanjian_targets(ctx.core, ctx.catalog, owner, damaged).contains(&t) {
                return Flow::Done;
            }
            if !has_shangjiangdelvli(ctx.core, ctx.catalog, owner) {
                let Some(cid) = m.cards.first().copied().filter(|c| ctx.core.player(owner).hand.contains(c)) else {
                    return Flow::Done;
                };
                discard_player_cards(ctx, owner, &[cid]);
            }
            emit_damage(ctx, Some(owner), t, 1, DamageNature::Normal, None);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn handle_yitianjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if ctx.core.player(owner).hp >= ctx.core.player(owner).max_hp as i32
                || ctx.core.player(owner).hand.is_empty()
            {
                return Flow::Done;
            }
            if has_shangjiangdelvli(ctx.core, ctx.catalog, owner) {
                ctx.recover(owner, 1, None);
                return Flow::Done;
            }
            wait_w_ask(m, ctx, HostStage::SelectCard, ASK_YITIANJIAN_DISCARD)
        }
        HostStage::SelectCard => {
            m.card = None;
            let Some(cid) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|c| ctx.core.player(owner).hand.contains(c))
            else {
                return Flow::Done;
            };
            discard_player_cards(ctx, owner, &[cid]);
            if ctx.core.player(owner).alive {
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn handle_pyzhuren(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let used = flag(ctx.core, owner, "skill_turn:l11_pyzhuren_used");
    match m.stage {
        HostStage::Act => {
            if used >= 2 {
                return Flow::Done;
            }
            if has_shangjiangdelvli(ctx.core, ctx.catalog, owner) {
                set_flag(ctx.core, owner, "skill_turn:l11_pyzhuren_used", used + 1);
                with_current_damage_mut(ctx, |dm| dm.amount += 1);
                return Flow::Done;
            }
            wait_w_ask(m, ctx, HostStage::SelectCard, ASK_PYZHUREN_DISCARD)
        }
        HostStage::SelectCard => {
            m.card = None;
            let pool = pyzhuren_discard_pool(ctx.core, ctx.catalog, owner);
            let Some(cid) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|c| pool.contains(c))
            else {
                return Flow::Done;
            };
            let cur_used = flag(ctx.core, owner, "skill_turn:l11_pyzhuren_used");
            if cur_used >= 2 {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "skill_turn:l11_pyzhuren_used", cur_used + 1);
            with_current_damage_mut(ctx, |dm| dm.amount += 1);
            discard_player_cards(ctx, owner, &[cid]);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
