//! L10 史诗卡牌十常侍伴生技（`archive/upstream/noname-liuli/noname/character/mobile.js:4669-5374`）。
//!
//! 全部 10 个伴生技能绑定到 `HostBehavior::L10ScsCompanion`（tag 574），按技能名分派：
//! `scstaoluan`, `scschiyan`, `scszimou`, `scspicai`, `scsyaozhuo`,
//! `scsxiaolu`, `scskuiji`, `scschihe`, `scsniqu`, `scsanruo`。

use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

#[path = "l10_scs_asks.rs"]
mod l10_scs_asks;
#[path = "l10_scs_extra.rs"]
mod l10_scs_extra;

pub(in crate::content::host) use self::l10_scs_asks::{
    anruo_usable_cards, anruo_usable_cards_for_ctx, ask_l10_scs,
    scsanruo_steal_pool, scsanruo_steal_target_candidates,
};
use self::l10_scs_extra::{
    after_xiaolu_draw, run_picai_judge, step_picai_judge_settled, step_scsanruo_card,
    step_scsanruo_steal_card, step_scsanruo_steal_confirm, step_scsanruo_steal_target,
    step_scsanruo_target, step_scskuiji_cards, step_scskuiji_target, step_scspicai_continue,
    step_scspicai_target, step_scsxiaolu_cards, step_scsxiaolu_mode, step_scsxiaolu_target,
    step_scsyaozhuo_discard, step_scsyaozhuo_own, step_scsyaozhuo_target, step_scsyaozhuo_tgt,
    start_scsanruo_effect,
};

pub(super) const ASK_SCSNIQU_TARGET: u32 = 1;
pub(super) const ASK_SCSTAOLUAN_CARD: u32 = 2;
pub(super) const ASK_SCSTAOLUAN_NAME: u32 = 3;
pub(super) const ASK_SCSTAOLUAN_TARGETS: u32 = 4;
pub(super) const ASK_SCSCHIYAN_PICK: u32 = 5;
pub(super) const ASK_SCSPICAI_CONTINUE: u32 = 6;
pub(super) const ASK_SCSPICAI_TARGET: u32 = 7;
pub(super) const ASK_SCSYAOZHUO_TARGET: u32 = 8;
pub(super) const ASK_SCSYAOZHUO_OWN_CARD: u32 = 9;
pub(super) const ASK_SCSYAOZHUO_TGT_CARD: u32 = 10;
pub(super) const ASK_SCSYAOZHUO_DISCARD: u32 = 11;
pub(super) const ASK_SCSXIAOLU_MODE: u32 = 12;
pub(super) const ASK_SCSXIAOLU_CARDS: u32 = 13;
pub(super) const ASK_SCSXIAOLU_TARGET: u32 = 14;
pub(super) const ASK_SCSKUIJI_TARGET: u32 = 15;
pub(super) const ASK_SCSKUIJI_CARDS: u32 = 16;
pub(super) const ASK_SCSCHIHE_CONFIRM: u32 = 17;
pub(super) const ASK_SCSANRUO_CARD: u32 = 18;
pub(super) const ASK_SCSANRUO_TARGET: u32 = 19;
pub(super) const ASK_SCS_ORDER: u32 = 20;
pub(super) const ASK_SCSANRUO_STEAL_CONFIRM: u32 = 21;
pub(super) const ASK_SCSANRUO_STEAL_TARGET: u32 = 22;
pub(super) const ASK_SCSANRUO_STEAL_CARD: u32 = 23;

pub(super) fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1 << 0,
        Suit::Heart => 1 << 1,
        Suit::Club => 1 << 2,
        Suit::Diamond => 1 << 3,
    }
}

pub(super) fn is_l10_scs_active_skill(name: &str) -> bool {
    matches!(
        name,
        "scstaoluan" | "scspicai" | "scsyaozhuo" | "scsxiaolu" | "scskuiji" | "scsniqu" | "scsanruo"
    )
}

pub(in crate::content::host) fn trigger_meta_l10_scs(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 0,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "scschiyan" => Some(meta(EventKind::DamageBegin, Vec::new())),
        "scszimou" => Some(meta(EventKind::CardUsed, Vec::new())),
        "scsanruo" => Some(meta(EventKind::CardUsed, vec![EventKind::CardResponded])),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l10_scs(_name: &str) -> SkillLimits {
    // 取消时不可预扣 `once_used`，限一次由各主动技提交时写入 `vm_aux.once` scope=1
    SkillLimits::default()
}

pub(in crate::content::host) fn filter_l10_scs_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[crate::event::Frame],
    frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if skill.name == "scschiyan" {
        if let EventData::Damage(d) = event {
            let root = match frame.data {
                EventData::DamageWindow { root, .. } => frames.iter().find(|f| f.id == root),
                EventData::Damage(_) => Some(frame),
                _ => None,
            };
            let sha_like = root
                .and_then(|r| crate::event::damage_card_kind(frames, r, ctx.catalog))
                .or_else(|| d.card.map(|c| ctx.catalog.card_kind(c)))
                .is_some_and(|k| k.is_sha_like());
            return matches!(d.settlement.progress, crate::event::DamageProgress::Begin)
                && !d.prevented
                && !d.from_chain
                && !d.settlement.notrigger
                && d.amount > 0
                && d.source == Some(owner)
                && sha_like
                && ctx.state.player(owner).hand.len() >= ctx.state.player(d.target).hand.len()
                && ctx.state.player(owner).equips.len()
                    >= ctx.state.player(d.target).equips.len();
        }
        return false;
    }
    filter_l10_scs(skill, ctx, owner, event)
}

pub(in crate::content::host) fn filter_l10_scs(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match (skill.name.as_str(), event) {
        ("scschiyan", EventData::Damage(d)) => {
            matches!(d.settlement.progress, crate::event::DamageProgress::Begin)
                && !d.prevented
                && !d.from_chain
                && !d.settlement.notrigger
                && d.amount > 0
                && d.source == Some(owner)
                && d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
                && ctx.state.player(owner).hand.len() >= ctx.state.player(d.target).hand.len()
                && ctx.state.player(owner).equips.len() >= ctx.state.player(d.target).equips.len()
        }
        ("scszimou", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && ctx.state.turn.current == owner
                && ctx.state.turn.phase == Phase::Play
                && matches!(ctx.state.turn.cards_used_play_phase, 2 | 4 | 6)
        }
        ("scsanruo", EventData::CardUsed(u)) => {
            !u.cancelled
                && u.player == owner
                && u.card.is_some_and(|c| {
                    ctx.state
                        .vm_aux
                        .flags
                        .contains_key(&(owner, format!("l10_scsanruo_effect:{c}")))
                })
        }
        ("scsanruo", EventData::CardResponded { card, player }) => {
            *player == owner
                && ctx
                    .state
                    .vm_aux
                    .flags
                    .contains_key(&(owner, format!("l10_scsanruo_effect:{card}")))
        }
        _ => false,
    }
}

/// 响应窗（1=闪, 2=杀, 3=无懈, 4=濒死桃, 5=自救桃）查询玩家是否可发动【安弱】。
pub(crate) fn scsanruo_response_skill(
    core: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    resp_ctx: u8,
) -> Option<crate::ids::SkillDefId> {
    if !core.player(responder).alive || catalog.blocks_all_cards(core, responder) {
        return None;
    }
    let def = catalog.skill_by_name("scsanruo")?;
    if !core.has_skill(responder, def)
        || catalog.is_tieji_sealed(core, responder)
        || catalog.is_rechanyuan_sealed_def(core, responder, def)
    {
        return None;
    }
    if anruo_usable_cards_for_ctx(core, catalog, responder, u32::from(resp_ctx)).is_empty() {
        return None;
    }
    Some(def)
}

pub(in crate::content::host) fn scsanruo_spawn_response(
    ctx: &mut Ctx<'_>,
    responder: PlayerId,
    resp_ctx: u8,
    dying: PlayerId,
    steal_target: PlayerId,
) {
    let machine = Box::new(HostMachine {
        def_key: "scsanruo".to_string(),
        behavior: HostBehavior::L10ScsCompanion,
        owner: responder,
        target: (resp_ctx == 4 || resp_ctx == 5).then_some(dying),
        card: Some(ASK_SCSANRUO_CARD),
        cards: vec![u32::from(resp_ctx), u32::from(steal_target)],
        targets: Vec::new(),
        stage: HostStage::Act,
    });
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Trigger {
            frame: crate::resolution::NO_FRAME,
        },
        machine,
        awaiting_answer: false,
    });
}

pub(in crate::content::host) fn activate_l10_scs(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive
        || state.turn.current != owner
        || state.turn.phase != Phase::Play
        || !is_l10_scs_active_skill(&skill.name)
    {
        return None;
    }
    if skill.name != "scsanruo" && state.vm_aux.once.contains(&(owner, skill.name.clone(), 1)) {
        return None;
    }
    let ok = match skill.name.as_str() {
        "scsniqu" => state.alive_players().next().is_some(),
        "scstaoluan" => {
            let can_hand = !catalog.blocks_hand_cards(state, owner);
            !catalog.blocks_card_use_and_save(state, owner)
                && ((can_hand && !state.player(owner).hand.is_empty())
                    || !state.player(owner).equips.is_empty())
                && !super::l10_reguhuo::reguhuo_name_options(state, catalog, owner, 0).is_empty()
        }
        "scspicai" | "scsxiaolu" => true,
        "scsyaozhuo" => {
            !state.player(owner).hand.is_empty()
                && state
                    .alive_players()
                    .any(|p| p != owner && !state.player(p).hand.is_empty())
        }
        "scskuiji" => state
            .alive_players()
            .any(|p| p != owner && !state.player(p).hand.is_empty()),
        "scsanruo" => {
            !catalog.blocks_card_use_and_save(state, owner)
                && !anruo_usable_cards(state, catalog, owner).is_empty()
        }
        _ => false,
    };
    if !ok {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L10ScsCompanion,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l10_scs(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut card = None;
    let mut target = None;
    let mut cards = Vec::new();
    match (skill.name.as_str(), event) {
        ("scschiyan", EventData::Damage(d)) => {
            card = Some(ASK_SCSCHIYAN_PICK + 100); // 伤害加伤瞬时分支哨兵
            target = Some(d.target);
        }
        ("scszimou", EventData::CardUsed(u)) => {
            cards = u.card.into_iter().collect();
        }
        ("scsanruo", EventData::CardUsed(u)) => {
            card = Some(ASK_SCSANRUO_STEAL_CONFIRM);
            cards = u.card.into_iter().collect();
            target = u.targets.first().copied();
        }
        ("scsanruo", EventData::CardResponded { card: c, .. }) => {
            card = Some(ASK_SCSANRUO_STEAL_CONFIRM);
            cards = vec![*c];
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L10ScsCompanion,
        owner,
        target,
        card,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l10_scs(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    if m.card == Some(ASK_SCSCHIYAN_PICK + 100) {
        let dmg = ctx.current_damage_mut();
        if !dmg.prevented && dmg.amount > 0 {
            dmg.amount += 1;
            ctx.log(LogEvent::SkillTriggered {
                skill: "scschiyan",
                owner: m.owner,
                presentation: None,
            });
        }
        return Flow::Done;
    }
    if m.stage == HostStage::Act {
        return start_act(m, ctx);
    }
    match m.card {
        Some(ASK_SCSNIQU_TARGET) => finish_scsniqu(m, ctx),
        Some(ASK_SCSTAOLUAN_CARD) => step_scstaoluan_card(m, ctx),
        Some(ASK_SCSTAOLUAN_NAME) => step_scstaoluan_name(m, ctx),
        Some(ASK_SCSTAOLUAN_TARGETS) => step_scstaoluan_targets(m, ctx),
        Some(ASK_SCSCHIYAN_PICK) => finish_scschiyan_pick(m, ctx),
        Some(ASK_SCSCHIHE_CONFIRM) => finish_scschihe_confirm(m, ctx),
        Some(ASK_SCS_ORDER) => finish_scs_order(m, ctx),
        Some(ASK_SCSPICAI_CONTINUE) => step_scspicai_continue(m, ctx),
        Some(ASK_SCSPICAI_TARGET) => step_scspicai_target(m, ctx),
        Some(ASK_SCSYAOZHUO_TARGET) => step_scsyaozhuo_target(m, ctx),
        Some(ASK_SCSYAOZHUO_OWN_CARD) => step_scsyaozhuo_own(m, ctx),
        Some(ASK_SCSYAOZHUO_TGT_CARD) => step_scsyaozhuo_tgt(m, ctx),
        Some(ASK_SCSYAOZHUO_DISCARD) => step_scsyaozhuo_discard(m, ctx),
        Some(ASK_SCSXIAOLU_MODE) => step_scsxiaolu_mode(m, ctx),
        Some(ASK_SCSXIAOLU_CARDS) => step_scsxiaolu_cards(m, ctx),
        Some(ASK_SCSXIAOLU_TARGET) => step_scsxiaolu_target(m, ctx),
        Some(ASK_SCSKUIJI_TARGET) => step_scskuiji_target(m, ctx),
        Some(ASK_SCSKUIJI_CARDS) => step_scskuiji_cards(m, ctx),
        Some(ASK_SCSANRUO_CARD) => step_scsanruo_card(m, ctx),
        Some(ASK_SCSANRUO_TARGET) => step_scsanruo_target(m, ctx),
        Some(ASK_SCSANRUO_STEAL_CONFIRM) => step_scsanruo_steal_confirm(m, ctx),
        Some(ASK_SCSANRUO_STEAL_TARGET) => step_scsanruo_steal_target(m, ctx),
        Some(ASK_SCSANRUO_STEAL_CARD) => step_scsanruo_steal_card(m, ctx),
        _ => {
            if m.def_key == "scsxiaolu" && m.stage == HostStage::DrawSettle {
                after_xiaolu_draw(m, ctx)
            } else if m.def_key == "scspicai" && m.stage == HostStage::DrawSettle {
                step_picai_judge_settled(m, ctx)
            } else {
                Flow::Done
            }
        }
    }
}

pub(super) fn consume_once(m: &HostMachine, ctx: &mut Ctx<'_>) {
    ctx.core
        .vm_aux
        .once
        .insert((m.owner, m.def_key.clone(), 1));
}

pub(super) fn wait_l10_scs(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(spec) = ask_l10_scs(
        m.stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) else {
        return Flow::Done;
    };
    Flow::WaitChoice {
        observer: spec.observer,
        spec: ChoiceSpec {
            kind: spec.kind,
            prompt: spec.prompt,
            options: spec.options,
            min: spec.min,
            max: spec.max,
            allow_cancel: spec.allow_cancel,
        },
    }
}

fn start_act(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.card == Some(ASK_SCS_ORDER) {
        m.stage = HostStage::SelectNature;
        return wait_l10_scs(m, ctx);
    }
    if m.def_key == "scsanruo" && m.card == Some(ASK_SCSANRUO_STEAL_CONFIRM) {
        return start_scsanruo_effect(m, ctx);
    }
    if m.def_key == "scsanruo" && m.card == Some(ASK_SCSANRUO_CARD) {
        m.stage = HostStage::SelectCard;
        return wait_l10_scs(m, ctx);
    }
    if m.card == Some(ASK_SCSCHIYAN_PICK) && m.target.is_none() {
        return Flow::Done;
    }
    if matches!(m.card, Some(ASK_SCSCHIYAN_PICK) | Some(ASK_SCSCHIHE_CONFIRM)) {
        m.stage = HostStage::SelectCard;
        return wait_l10_scs(m, ctx);
    }
    match m.def_key.as_str() {
        "scsniqu" => {
            m.card = Some(ASK_SCSNIQU_TARGET);
            m.stage = HostStage::TargetChoice;
        }
        "scstaoluan" => {
            m.card = Some(ASK_SCSTAOLUAN_CARD);
            m.stage = HostStage::SelectCard;
        }
        "scszimou" => {
            return step_scszimou(m, ctx);
        }
        "scspicai" => {
            consume_once(m, ctx);
            return run_picai_judge(m, ctx);
        }
        "scsyaozhuo" => {
            m.card = Some(ASK_SCSYAOZHUO_TARGET);
            m.stage = HostStage::TargetChoice;
        }
        "scsxiaolu" => {
            consume_once(m, ctx);
            ctx.log(LogEvent::SkillTriggered {
                skill: "scsxiaolu",
                owner: m.owner,
                presentation: None,
            });
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(m.owner, 2);
            return Flow::Yield;
        }
        "scskuiji" => {
            m.card = Some(ASK_SCSKUIJI_TARGET);
            m.stage = HostStage::TargetChoice;
        }
        "scsanruo" => {
            m.cards = vec![0];
            m.card = Some(ASK_SCSANRUO_CARD);
            m.stage = HostStage::SelectCard;
        }
        _ => return Flow::Done,
    }
    wait_l10_scs(m, ctx)
}

fn finish_scsniqu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    consume_once(m, ctx);
    ctx.log(LogEvent::SkillTriggered {
        skill: "scsniqu",
        owner: m.owner,
        presentation: None,
    });
    ctx.emit(EventData::Damage(damage_data_with(
        Some(m.owner),
        t,
        1,
        DamageNature::Fire,
        None,
    )));
    Flow::Done
}

fn step_scstaoluan_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(m.owner).hand.contains(&c) && !ctx.core.player(m.owner).equips.contains(&c) {
        return Flow::Done;
    }
    m.cards = vec![c];
    m.card = Some(ASK_SCSTAOLUAN_NAME);
    m.stage = HostStage::SelectNature;
    wait_l10_scs(m, ctx)
}

fn step_scstaoluan_name(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    let Some(c) = m.cards.first().copied() else {
        return Flow::Done;
    };
    let Some(kind) = super::l10_reguhuo::reguhuo_kind_at_catalog(ctx.catalog, idx) else {
        return Flow::Done;
    };
    let (min, max, targets) = ctx.core.legal_targets(ctx.catalog, kind, m.owner);
    if (min == 0 && max == 0)
        || matches!(
            kind,
            CardKind::Tao | CardKind::Jiu | CardKind::Tianxianjiu | CardKind::Dinvxuanshuang
        )
    {
        if matches!(
            kind,
            CardKind::Tao | CardKind::Jiu | CardKind::Tianxianjiu | CardKind::Dinvxuanshuang
        ) && targets.is_empty()
        {
            return Flow::Done;
        }
        consume_once(m, ctx);
        ctx.log(LogEvent::SkillTriggered {
            skill: "scstaoluan",
            owner: m.owner,
            presentation: None,
        });
        let u = if kind.is_sha_like() {
            super::sha_card_use_data(ctx, m.owner, c, targets, Some(kind))
        } else {
            crate::machine::card_use_as(c, m.owner, targets, Some(kind))
        };
        ctx.emit(EventData::CardUsed(u));
        return Flow::Done;
    }
    if targets.len() < min as usize {
        return Flow::Done;
    }
    m.target = None;
    m.cards = vec![c, u32::from(idx)];
    m.card = Some(ASK_SCSTAOLUAN_TARGETS);
    m.stage = HostStage::TargetChoice;
    wait_l10_scs(m, ctx)
}

fn step_scstaoluan_targets(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(c) = m.cards.first().copied() else {
        return Flow::Done;
    };
    let Some(idx) = m.cards.get(1).copied() else {
        return Flow::Done;
    };
    let Some(kind) = super::l10_reguhuo::reguhuo_kind_at_catalog(ctx.catalog, idx as u8) else {
        return Flow::Done;
    };
    let targets: Vec<PlayerId> = if kind == CardKind::Jiedao {
        if let Some(holder) = m.target {
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                m.target = None;
                return Flow::Done;
            };
            m.target = None;
            if !ctx
                .core
                .legal_jiedao_victims(ctx.catalog, holder)
                .contains(&victim)
            {
                return Flow::Done;
            }
            vec![holder, victim]
        } else {
            let Some(ChoiceOption::Player(holder)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let victims = ctx.core.legal_jiedao_victims(ctx.catalog, holder);
            if victims.is_empty() {
                return Flow::Done;
            }
            m.target = Some(holder);
            return wait_l10_scs(m, ctx);
        }
    } else {
        let picked: Vec<PlayerId> = ctx
            .answer()
            .iter()
            .filter_map(|o| match o {
                ChoiceOption::Player(p) => Some(*p),
                _ => None,
            })
            .collect();
        let (min, max, _) = ctx.core.legal_targets(ctx.catalog, kind, m.owner);
        if picked.len() < min as usize || picked.len() > max as usize {
            return Flow::Done;
        }
        picked
    };
    consume_once(m, ctx);
    ctx.log(LogEvent::SkillTriggered {
        skill: "scstaoluan",
        owner: m.owner,
        presentation: None,
    });
    let u = if kind.is_sha_like() {
        super::sha_card_use_data(ctx, m.owner, c, targets, Some(kind))
    } else {
        crate::machine::card_use_as(c, m.owner, targets, Some(kind))
    };
    ctx.emit(EventData::CardUsed(u));
    Flow::Done
}

fn finish_scschiyan_pick(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    let origin = if ctx.core.player(t).hand.contains(&cid) {
        ctx.take_hand_card(t, cid);
        crate::event::LostOrigin::Hand
    } else if ctx.core.player(t).equips.contains(&cid) {
        ctx.core.player_mut(t).equips.retain(|&c| c != cid);
        ctx.core.declared_slots.remove(&cid);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, cid);
        crate::event::LostOrigin::Equip
    } else {
        return Flow::Done;
    };
    ctx.note_cards_lost(t, &[cid]);
    ctx.core
        .vm_aux
        .piles
        .entry((t, "scschiyan_get".to_string()))
        .or_default()
        .push(cid);
    ctx.log(LogEvent::SkillTriggered {
        skill: "scschiyan",
        owner: m.owner,
        presentation: None,
    });
    ctx.emit(EventData::CardsLost {
        player: t,
        cards: vec![cid],
        origins: vec![origin],
    });
    Flow::Done
}

fn finish_scschihe_confirm(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
        return Flow::Done;
    }
    let Some(t) = m.target else {
        return Flow::Done;
    };
    let sha_frame_id = ctx
        .core
        .vm_aux
        .flags
        .get(&(m.owner, format!("sha_l10_ctx_frame:{t}")))
        .copied()
        .unwrap_or(0);
    let sha_cid = m.cards.first().copied();
    ctx.log(LogEvent::SkillTriggered {
        skill: "scschihe",
        owner: m.owner,
        presentation: None,
    });
    let sha_suit = sha_cid.map(|cid| ctx.catalog.effective_card_suit(ctx.core, m.owner, cid));
    let mut suit_mask = 0u32;
    let mut extra_dmg = 0u32;
    for _ in 0..2 {
        let judged = ctx.draw_judge_card();
        let s = ctx.catalog.cards[judged as usize].suit;
        suit_mask |= suit_bit(s);
        if Some(s) == sha_suit {
            extra_dmg += 1;
        }
        ctx.to_discard(judged);
    }
    let cur_mask = ctx
        .core
        .vm_aux
        .flags
        .get(&(t, "l10_scschihe_block_suits".to_string()))
        .copied()
        .unwrap_or(0);
    ctx.core
        .vm_aux
        .flags
        .insert((t, "l10_scschihe_block_suits".to_string()), cur_mask | suit_mask);
    if extra_dmg > 0 {
        let key = if let Some(cid) = sha_cid {
            (m.owner, format!("card_damage_bonus:{cid}"))
        } else {
            (m.owner, format!("sha_frame_damage_bonus:{sha_frame_id}"))
        };
        let prev = ctx.core.vm_aux.flags.get(&key).copied().unwrap_or(0);
        ctx.core.vm_aux.flags.insert(key, prev + extra_dmg);
    }
    Flow::Done
}

fn finish_scs_order(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    let sha_frame_id = ctx
        .core
        .vm_aux
        .flags
        .get(&(m.owner, format!("sha_l10_ctx_frame:{t}")))
        .copied()
        .unwrap_or(0);
    let first = match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(1)) => 1,
        _ => 0,
    };
    ctx.core.vm_aux.flags.insert(
        (m.owner, format!("sha_l10_order_done:{sha_frame_id}:{t}")),
        1,
    );
    ctx.core.vm_aux.flags.insert(
        (m.owner, format!("sha_l10_order_first:{sha_frame_id}:{t}")),
        first,
    );
    Flow::Done
}

fn step_scszimou(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let cnt = ctx.core.turn.cards_used_play_phase;
    let found = match cnt {
        2 => ctx
            .core
            .deck
            .iter()
            .rev()
            .copied()
            .find(|c| ctx.catalog.card_kind(*c) == CardKind::Jiu),
        4 => ctx
            .core
            .deck
            .iter()
            .rev()
            .copied()
            .find(|c| ctx.catalog.card_kind(*c).is_sha_like()),
        6 => ctx
            .core
            .deck
            .iter()
            .rev()
            .copied()
            .find(|c| ctx.catalog.card_kind(*c) == CardKind::Duel),
        _ => None,
    };
    if let Some(card) = found {
        ctx.core.deck.retain(|&c| c != card);
        ctx.core.player_mut(m.owner).hand.push(card);
        ctx.log(LogEvent::SkillTriggered {
            skill: "scszimou",
            owner: m.owner,
            presentation: None,
        });
        ctx.log(LogEvent::CardsGiven {
            from: m.owner,
            to: m.owner,
            cards: vec![card],
        });
    }
    Flow::Done
}

/// 杀目标结算前钩子（`u08_batch_d_hooks` 调用）：`scschiyan`（目标 `hp>0 && he` 非空）与 `scschihe`（单目标杀，含虚拟杀与顺序选择）。
pub(in crate::content::host) fn sha_begin_l10_scs(
    ctx: &mut Ctx<'_>,
    sha_frame_id: crate::ids::EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
) -> bool {
    let chiyan_done_key = (source, format!("sha_scschiyan_done:{sha_frame_id}:{target}"));
    let t_he = !ctx.core.player(target).hand.is_empty()
        || !ctx.core.player(target).equips.is_empty();
    let chiyan_eligible = ctx
        .catalog
        .skill_by_name("scschiyan")
        .is_some_and(|def| {
            let sealed = ctx.catalog.is_tieji_sealed(ctx.core, source)
                || ctx.catalog.is_rechanyuan_sealed_def(ctx.core, source, def);
            !sealed
                && ctx.core.has_skill(source, def)
                && ctx.core.player(target).hp > 0
                && t_he
                && !ctx.core.vm_aux.flags.contains_key(&chiyan_done_key)
        });
    let chihe_done_key = (source, format!("sha_scschihe_done:{sha_frame_id}:{target}"));
    let chihe_eligible = ctx
        .catalog
        .skill_by_name("scschihe")
        .is_some_and(|def| {
            let sealed = ctx.catalog.is_tieji_sealed(ctx.core, source)
                || ctx.catalog.is_rechanyuan_sealed_def(ctx.core, source, def);
            let single_tgt = ctx
                .res
                .frame(sha_frame_id)
                .is_some_and(|f| matches!(&f.data, EventData::CardUsed(u) if u.targets.len() == 1));
            !sealed
                && single_tgt
                && ctx.core.has_skill(source, def)
                && !ctx.core.vm_aux.flags.contains_key(&chihe_done_key)
        });
    if !chiyan_eligible && !chihe_eligible {
        ctx.core
            .vm_aux
            .flags
            .remove(&(source, format!("sha_l10_ctx_frame:{target}")));
        return false;
    }
    ctx.core
        .vm_aux
        .flags
        .insert((source, format!("sha_l10_ctx_frame:{target}")), sha_frame_id);
    if chiyan_eligible && chihe_eligible {
        let order_done_key = (source, format!("sha_l10_order_done:{sha_frame_id}:{target}"));
        if !ctx.core.vm_aux.flags.contains_key(&order_done_key) {
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger {
                    frame: sha_frame_id,
                },
                machine: Box::new(HostMachine {
                    def_key: "scschiyan".to_string(),
                    behavior: HostBehavior::L10ScsCompanion,
                    owner: source,
                    target: Some(target),
                    card: Some(ASK_SCS_ORDER),
                    cards: sha_card.into_iter().collect(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
                awaiting_answer: false,
            });
            return true;
        }
    }
    let first = ctx
        .core
        .vm_aux
        .flags
        .get(&(source, format!("sha_l10_order_first:{sha_frame_id}:{target}")))
        .copied()
        .unwrap_or(0);
    if chihe_eligible && (first == 1 || !chiyan_eligible) {
        ctx.core.vm_aux.flags.insert(chihe_done_key, 1);
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Trigger {
                frame: sha_frame_id,
            },
            machine: Box::new(HostMachine {
                def_key: "scschihe".to_string(),
                behavior: HostBehavior::L10ScsCompanion,
                owner: source,
                target: Some(target),
                card: Some(ASK_SCSCHIHE_CONFIRM),
                cards: sha_card.into_iter().collect(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }),
            awaiting_answer: false,
        });
        return true;
    }
    if chiyan_eligible {
        ctx.core.vm_aux.flags.insert(chiyan_done_key, 1);
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Trigger {
                frame: sha_frame_id,
            },
            machine: Box::new(HostMachine {
                def_key: "scschiyan".to_string(),
                behavior: HostBehavior::L10ScsCompanion,
                owner: source,
                target: Some(target),
                card: Some(ASK_SCSCHIYAN_PICK),
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }),
            awaiting_answer: false,
        });
        return true;
    }
    false
}

/// 回合结束（`Phase::End { begin: false }`）归还所有 `scschiyan_get` 暂扣牌并清理本回合计数。
pub(crate) fn on_phase_end_l10_scs(ctx: &mut Ctx<'_>) {
    super::u05_b2_gujian::on_phase_end_gujian(ctx);
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        if let Some(cards) = ctx
            .core
            .vm_aux
            .piles
            .remove(&(p, "scschiyan_get".to_string()))
        {
            if !cards.is_empty() && ctx.core.player(p).alive {
                ctx.core.player_mut(p).hand.extend(cards.iter().copied());
                ctx.log(LogEvent::CardsGiven {
                    from: p,
                    to: p,
                    cards,
                });
            } else {
                for c in cards {
                    ctx.to_discard(c);
                }
            }
        }
    }
}
