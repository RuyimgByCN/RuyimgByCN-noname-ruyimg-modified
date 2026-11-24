//! Batch Q4 L26 `Fool_skill`（tag 871）：`ziyuan` / `minsi` / `xinfu_fangtong`
//! 实牌点数求和筛选与对局结算（对齐 `l10_scs_asks` 契约）。

use super::super::*;
use crate::catalog::{Catalog, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

const ASK_ZIYUAN_CARDS: u32 = 1;
const ASK_ZIYUAN_TARGET: u32 = 2;
const ASK_MINSI_CARDS: u32 = 3;
const ASK_FANGTONG_HE: u32 = 4;
const ASK_FANGTONG_CARDS: u32 = 5;
const ASK_FANGTONG_TARGET: u32 = 6;
const ASK_MINSI_TAG_DRAW: u32 = 7;

#[inline]
pub(in crate::content::host) fn fool_skill_enabled(core: &GameStateCore) -> bool {
    core.vm_aux
        .flags
        .get(&(0, "l26_fool_skill".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
}

fn card_number(catalog: &Catalog, card: CardId) -> u32 {
    u32::from(catalog.cards[card as usize].rank)
}

/// 当 `Fool_skill` 启用时，过滤出可继续拼合而不超过上限 `13`（资援/敏思）或 `36`（方统）的候选牌。
pub fn filter_cards_by_fool_sum(
    catalog: &Catalog,
    candidates: &[CardId],
    selected: &[CardId],
    limit: u32,
) -> Vec<CardId> {
    let used: u32 = selected.iter().map(|c| card_number(catalog, *c)).sum();
    candidates
        .iter()
        .copied()
        .filter(|c| !selected.contains(c) && used + card_number(catalog, *c) <= limit)
        .collect()
}

pub(in crate::content::host) fn trigger_meta_q4_fool(name: &str) -> Option<TriggerMeta> {
    match name {
        "xinfu_fangtong" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_q4_fool(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn has_phase_use_q4_fool(name: &str) -> bool {
    matches!(name, "ziyuan" | "minsi")
}

pub(in crate::content::host) fn filter_q4_fool(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if skill.name != "xinfu_fangtong" {
        return false;
    }
    let EventData::Phase { player, phase: Phase::End, begin: true } = event else {
        return false;
    };
    if *player != owner {
        return false;
    }
    let pl = ctx.state.player(owner);
    let has_he = !pl.hand.is_empty() || !pl.equips.is_empty();
    let has_fang = ctx
        .state
        .vm_aux
        .piles
        .get(&(owner, "xinfu_jijun".to_string()))
        .is_some_and(|p| !p.is_empty());
    has_he && has_fang
}

pub(in crate::content::host) fn activate_q4_fool(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    let ask_id = match skill.name.as_str() {
        "ziyuan" => ASK_ZIYUAN_CARDS,
        "minsi" => ASK_MINSI_CARDS,
        _ => return None,
    };
    let pl = ctx.state.player(owner);
    let has_cards = if ask_id == ASK_MINSI_CARDS {
        !pl.hand.is_empty() || !pl.equips.is_empty()
    } else {
        !pl.hand.is_empty()
    };
    if ctx.state.vm_aux.once_count(owner, 1, &skill.name) >= 1 || !has_cards {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::Q4FoolSkill,
        owner,
        target: None,
        card: Some(ask_id),
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_q4_fool(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    _event: &EventData,
) -> HostMachine {
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::Q4FoolSkill,
        owner,
        target: None,
        card: Some(ASK_FANGTONG_HE),
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn ask_q4_fool(
    stage: HostStage,
    owner: PlayerId,
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = card?;
    let fool = fool_skill_enabled(core);
    let selected: Vec<CardId> = cards.to_vec();
    match (ask_id, stage) {
        (ASK_ZIYUAN_CARDS, HostStage::SelectCard) => {
            let hand: Vec<CardId> = core.player(owner).hand.clone();
            let filtered = if fool {
                filter_cards_by_fool_sum(catalog, &hand, &selected, 13)
            } else {
                hand.into_iter().filter(|c| !selected.contains(c)).collect()
            };
            let opts: Vec<ChoiceOption> = filtered.into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| ask(owner, "ziyuan:cards", ChoiceKind::SelectCards, opts.clone(), 1, opts.len(), true))
        }
        (ASK_ZIYUAN_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core.alive_players().filter(|&p| p != owner).map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| ask(owner, "ziyuan:target", ChoiceKind::SelectPlayers, opts, 1, 1, false))
        }
        (ASK_MINSI_CARDS, HostStage::SelectCard) => {
            let he: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .collect();
            let filtered = if fool {
                filter_cards_by_fool_sum(catalog, &he, &selected, 13)
            } else {
                he.into_iter().filter(|c| !selected.contains(c)).collect()
            };
            let opts: Vec<ChoiceOption> = filtered.into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| ask(owner, "minsi:cards", ChoiceKind::SelectCards, opts.clone(), 1, opts.len(), true))
        }
        (ASK_FANGTONG_HE, HostStage::SelectCard) => {
            let he: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .collect();
            let filtered = if fool {
                filter_cards_by_fool_sum(catalog, &he, &[], 36)
            } else {
                he
            };
            let opts: Vec<ChoiceOption> = filtered.into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| ask(owner, "xinfu_fangtong:he", ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (ASK_FANGTONG_CARDS, HostStage::SelectCard) => {
            let fang_pile = core
                .vm_aux
                .piles
                .get(&(owner, "xinfu_jijun".to_string()))
                .cloned()
                .unwrap_or_default();
            let filtered = if fool {
                filter_cards_by_fool_sum(catalog, &fang_pile, &selected, 36)
            } else {
                fang_pile.into_iter().filter(|c| !selected.contains(c)).collect()
            };
            let opts: Vec<ChoiceOption> = filtered.into_iter().map(ChoiceOption::Card).collect();
            (!opts.is_empty()).then(|| ask(owner, "xinfu_fangtong:cards", ChoiceKind::SelectCards, opts.clone(), 1, opts.len(), true))
        }
        (ASK_FANGTONG_TARGET, HostStage::TargetChoice) => {
            let opts: Vec<ChoiceOption> = core.alive_players().filter(|&p| p != owner).map(ChoiceOption::Player).collect();
            (!opts.is_empty()).then(|| ask(owner, "xinfu_fangtong:target", ChoiceKind::SelectPlayers, opts, 1, 1, false))
        }
        _ => None,
    }
}

fn wait_q4_fool(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(spec) = ask_q4_fool(m.stage, m.owner, m.card, &m.cards, ctx.core, ctx.catalog) else {
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

pub(in crate::content::host) fn handle_q4_fool(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if m.stage == HostStage::Act {
        m.stage = HostStage::SelectCard;
        return wait_q4_fool(m, ctx);
    }
    let owner = m.owner;
    if m.stage == HostStage::DrawSettle && m.card == Some(ASK_MINSI_TAG_DRAW) {
        let prev_hand: Vec<CardId> = m.cards.drain(..).collect();
        let new_cards: Vec<CardId> = ctx
            .core
            .player(owner)
            .hand
            .iter()
            .copied()
            .filter(|c| !prev_hand.contains(c))
            .collect();
        if !new_cards.is_empty() {
            ctx.core.vm_aux.flags.insert((owner, "q4_minsi2_active".to_string()), 1);
            for cid in new_cards {
                ctx.core.vm_aux.flags.insert((owner, format!("q4_minsi2_card:{cid}")), 1);
            }
        }
        return Flow::Done;
    }
    match m.card {
        Some(ASK_ZIYUAN_CARDS) => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c)
                        if ctx.core.player(owner).hand.contains(&c) && !m.cards.contains(&c) =>
                    {
                        Some(c)
                    }
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            m.cards.extend(picked);
            let sum: u32 = m.cards.iter().map(|c| card_number(ctx.catalog, *c)).sum();
            if sum > 13 {
                return Flow::Done;
            }
            if sum < 13 {
                m.stage = HostStage::SelectCard;
                return wait_q4_fool(m, ctx);
            }
            ctx.core.vm_aux.inc_once(owner, 1, "ziyuan");
            m.card = Some(ASK_ZIYUAN_TARGET);
            m.stage = HostStage::TargetChoice;
            wait_q4_fool(m, ctx)
        }
        Some(ASK_ZIYUAN_TARGET) => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered { skill: "资援", owner, presentation: None });
            let given: Vec<CardId> = m.cards.drain(..).collect();
            for &c in &given {
                ctx.take_hand_card(owner, c);
            }
            ctx.core.player_mut(tgt).hand.extend(given.iter().copied());
            ctx.log(LogEvent::CardsGiven { from: owner, to: tgt, cards: given });
            ctx.recover(tgt, 1, None);
            Flow::Done
        }
        Some(ASK_MINSI_CARDS) => {
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c)
                        if (ctx.core.player(owner).hand.contains(&c)
                            || ctx.core.player(owner).equips.contains(&c))
                            && !m.cards.contains(&c) =>
                    {
                        Some(c)
                    }
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            m.cards.extend(picked);
            let sum: u32 = m.cards.iter().map(|c| card_number(ctx.catalog, *c)).sum();
            if sum > 13 {
                return Flow::Done;
            }
            if sum < 13 {
                m.stage = HostStage::SelectCard;
                return wait_q4_fool(m, ctx);
            }
            ctx.core.vm_aux.inc_once(owner, 1, "minsi");
            ctx.log(LogEvent::SkillTriggered { skill: "敏思", owner, presentation: None });
            let thrown: Vec<CardId> = m.cards.drain(..).collect();
            let draw_n = (thrown.len() as u32) * 2;
            for c in thrown {
                ctx.take_owned_card(owner, c);
                if ctx.catalog.card_kind(c).is_equip() {
                    ctx.core.declared_slots.remove(&c);
                }
                ctx.to_discard(c);
                ctx.emit(EventData::CardsDiscarded { player: owner, cards: vec![c] });
                ctx.log(LogEvent::CardDiscarded { player: owner, cards: vec![c] });
            }
            m.cards = ctx.core.player(owner).hand.clone();
            m.card = Some(ASK_MINSI_TAG_DRAW);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, draw_n);
            Flow::Yield
        }
        Some(ASK_FANGTONG_HE) => {
            let Some(ChoiceOption::Card(he_card)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&he_card)
                && !ctx.core.player(owner).equips.contains(&he_card)
            {
                return Flow::Done;
            }
            m.cards = vec![he_card];
            m.card = Some(ASK_FANGTONG_CARDS);
            m.stage = HostStage::SelectCard;
            wait_q4_fool(m, ctx)
        }
        Some(ASK_FANGTONG_CARDS) => {
            let fang_pile = ctx
                .core
                .vm_aux
                .piles
                .get(&(owner, "xinfu_jijun".to_string()))
                .cloned()
                .unwrap_or_default();
            let picked: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Card(c) if fang_pile.contains(&c) && !m.cards.contains(&c) => Some(c),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            m.cards.extend(picked);
            let sum: u32 = m.cards.iter().map(|c| card_number(ctx.catalog, *c)).sum();
            if sum > 36 {
                return Flow::Done;
            }
            if sum < 36 {
                m.stage = HostStage::SelectCard;
                return wait_q4_fool(m, ctx);
            }
            if m.cards.len() < 2 {
                return Flow::Done;
            }
            m.card = Some(ASK_FANGTONG_TARGET);
            m.stage = HostStage::TargetChoice;
            wait_q4_fool(m, ctx)
        }
        Some(ASK_FANGTONG_TARGET) => {
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered { skill: "方统", owner, presentation: None });
            for c in m.cards.drain(..) {
                if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(owner, "xinfu_jijun".to_string())) {
                    if let Some(pos) = pile.iter().position(|x| *x == c) {
                        pile.remove(pos);
                        ctx.to_discard(c);
                        ctx.emit(EventData::CardsDiscarded { player: owner, cards: vec![c] });
                        continue;
                    }
                }
                ctx.take_owned_card(owner, c);
                if ctx.catalog.card_kind(c).is_equip() {
                    ctx.core.declared_slots.remove(&c);
                }
                ctx.to_discard(c);
                ctx.emit(EventData::CardsDiscarded { player: owner, cards: vec![c] });
                ctx.log(LogEvent::CardDiscarded { player: owner, cards: vec![c] });
            }
            ctx.emit(EventData::Damage(damage_data_with(Some(owner), tgt, 3, DamageNature::Thunder, None)));
            Flow::Done
        }
        _ => Flow::Done,
    }
}
