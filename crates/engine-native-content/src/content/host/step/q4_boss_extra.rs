//! Batch Q4 U02/L03 boss/jiange 技能多步交互结算辅助。

use super::super::super::*;
use super::super::super::boss::sixiang_reinit_seat;
use super::{
    jiwu_available_skills, matching_judge_suit_cards, wait_q4_boss, ASK_JIWU_RESUME,
    ASK_JIWU_SKILL, ASK_JINGJIA_NEXT, ASK_KEDING_TARGETS, ASK_LINGFENG_GAIN,
    ASK_LINGFENG_TARGET, ASK_SKONGHUN_NEXT, ASK_TIANYUJG_NEXT, ASK_XIULUO_HAND, ASK_XIULUO_JUDGE,
    ASK_XUANFENG_CARD, ASK_XUANFENG_TARGET, ASK_XUANLEI_NEXT,
};
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData};
use crate::ids::CardId;
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(super) fn start_boss_zhinang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let mut keep = Vec::new();
    let mut rest = Vec::new();
    for _ in 0..5 {
        let Some(c) = ctx.core.deck.pop() else {
            break;
        };
        let kind = ctx.catalog.card_kind(c);
        if kind.is_trick_kind() || kind.is_equip() {
            keep.push(c);
        } else {
            rest.push(c);
        }
    }
    for c in rest {
        ctx.to_discard(c);
    }
    if keep.is_empty() {
        ctx.log(LogEvent::SkillTriggered { skill: "智囊", owner, presentation: None });
        return Flow::Done;
    }
    m.cards = keep.into_iter().map(u32::from).collect();
    m.stage = HostStage::TargetChoice;
    wait_q4_boss(m, ctx)
}

pub(super) fn finish_boss_zhinang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
        for raw in m.cards.drain(..) {
            ctx.to_discard(raw as CardId);
        }
        return Flow::Done;
    };
    ctx.log(LogEvent::SkillTriggered { skill: "智囊", owner, presentation: None });
    let gained: Vec<CardId> = m.cards.drain(..).map(|raw| raw as CardId).collect();
    ctx.core.player_mut(tgt).hand.extend(gained.iter().copied());
    ctx.log(LogEvent::CardsGiven { from: owner, to: tgt, cards: gained });
    Flow::Done
}

pub(super) fn step_keding_cards(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let chosen: Vec<CardId> = ctx
        .answer()
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Card(c) if ctx.core.player(owner).hand.contains(&c) => Some(c),
            _ => None,
        })
        .collect();
    if chosen.is_empty() {
        return Flow::Done;
    }
    let used_card = m.cards.first().copied().unwrap_or(0);
    m.cards = std::iter::once(used_card).chain(chosen.into_iter().map(u32::from)).collect();
    m.card = Some(ASK_KEDING_TARGETS);
    m.stage = HostStage::CompareTargetPick;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_keding_targets(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let need = m.cards.len().saturating_sub(1);
    let extra: Vec<PlayerId> = ctx
        .answer()
        .iter()
        .filter_map(|o| match *o {
            ChoiceOption::Player(p) if ctx.core.player(p).alive => Some(p),
            _ => None,
        })
        .collect();
    if extra.len() != need || need == 0 {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered { skill: "克定", owner, presentation: None });
    let disc: Vec<CardId> = m.cards.drain(1..).map(|raw| raw as CardId).collect();
    ctx.discard_hand_batch(owner, &disc);
    for t in extra {
        let Ok(cur) = ctx.current_card_use() else {
            return Flow::Done;
        };
        if !cur.targets.contains(&t) {
            ctx.current_card_use_mut().targets.push(t);
        }
    }
    Flow::Done
}

pub(super) fn finish_boss_baonu(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let form_name = match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(1)) => "boss_lvbu3",
        Some(ChoiceOption::Tag(0)) => "boss_lvbu2",
        _ => return Flow::Done,
    };
    let Some(target_gen) = (0..ctx.catalog.general_count() as u16)
        .find(|&d| ctx.catalog.general_meta(d).map(|g| g.0) == Some(form_name))
    else {
        return Flow::Done;
    };
    let interrupted = ctx.core.turn.current;
    ctx.log(LogEvent::SkillTriggered { skill: "暴怒", owner, presentation: None });
    let preserved_hp = ctx.core.player(owner).hp;
    if let Some(def) = ctx.catalog.skill_by_name("boss_baonu") {
        ctx.remove_skill(owner, def);
    }
    // Unwind all in-flight child frames/parks and processing-zone cards back to phaseLoop,
    // clear extra_phases, and expire the interrupted turn's end-of-turn temp skills/flags.
    crate::kernel::cleanup_interrupted_turn_for_phase_loop(ctx, interrupted, owner);
    crate::resolution::unwind_to_phase_loop(ctx);
    sixiang_reinit_seat(ctx, owner, target_gen, true);
    if preserved_hp > 6 {
        let p = ctx.core.player_mut(owner);
        p.max_hp = preserved_hp as u32;
        p.hp = preserved_hp;
    }
    ctx.core.player_mut(owner).linked = false;
    ctx.core.player_mut(owner).face_down = false;
    let judges: Vec<CardId> = ctx.core.player_mut(owner).judge.drain(..).collect();
    for j in judges {
        ctx.to_discard(j);
    }
    crate::kernel::begin_phase_loop_turn_for(ctx, owner);
    Flow::Done
}

pub(super) fn step_xiuluo_hand(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(ChoiceOption::Card(hc)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(owner).hand.contains(&hc) {
        return Flow::Done;
    }
    let suit = ctx.catalog.effective_card_suit(ctx.core, owner, hc);
    let judges: Vec<CardId> = ctx
        .core
        .player(owner)
        .judge
        .iter()
        .copied()
        .filter(|j| ctx.catalog.effective_card_suit(ctx.core, owner, *j) == suit)
        .collect();
    let Some(&first_j) = judges.first() else {
        return Flow::Done;
    };
    if judges.len() == 1 {
        discard_xiuluo_pair(ctx, owner, hc, first_j);
        return repeat_or_finish_xiuluo(m, ctx, owner);
    }
    m.cards = vec![u32::from(hc)];
    m.card = Some(ASK_XIULUO_JUDGE);
    m.stage = HostStage::CardsAsk;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_xiuluo_judge(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let (Some(&raw_hc), Some(ChoiceOption::Card(jc))) = (m.cards.first(), ctx.answer().first().copied()) else {
        return Flow::Done;
    };
    if !ctx.core.player(owner).judge.contains(&jc) {
        return Flow::Done;
    }
    discard_xiuluo_pair(ctx, owner, raw_hc as CardId, jc);
    repeat_or_finish_xiuluo(m, ctx, owner)
}

fn discard_xiuluo_pair(ctx: &mut Ctx<'_>, owner: PlayerId, hc: CardId, jc: CardId) {
    ctx.log(LogEvent::SkillTriggered { skill: "修罗", owner, presentation: None });
    ctx.discard_from_hand(owner, hc);
    ctx.core.player_mut(owner).judge.retain(|x| *x != jc);
    ctx.to_discard(jc);
}

fn repeat_or_finish_xiuluo(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    if ctx.core.player(owner).judge.is_empty()
        || matching_judge_suit_cards(ctx.core, ctx.catalog, owner).is_empty()
    {
        Flow::Done
    } else {
        m.cards.clear();
        m.card = Some(ASK_XIULUO_HAND);
        m.stage = HostStage::SelectCard;
        wait_q4_boss(m, ctx)
    }
}

pub(super) fn discard_owned_card_and_emit(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    ctx.take_owned_card(player, card);
    if ctx.catalog.card_kind(card).is_equip() {
        ctx.core.declared_slots.remove(&card);
    }
    ctx.to_discard(card);
    ctx.emit(EventData::CardsDiscarded { player, cards: vec![card] });
    ctx.log(LogEvent::CardDiscarded { player, cards: vec![card] });
}

pub(super) fn step_jiwu_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    let in_hand = ctx.core.player(owner).hand.contains(&c);
    let in_equip = ctx.core.player(owner).equips.contains(&c);
    if !in_hand && !in_equip {
        return Flow::Done;
    }
    discard_owned_card_and_emit(ctx, owner, c);
    m.card = Some(ASK_JIWU_RESUME);
    m.stage = HostStage::DrawSettle;
    Flow::Yield
}

pub(super) fn finish_jiwu_skill_step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let avail = jiwu_available_skills(ctx.core, ctx.catalog, owner);
    if avail.len() == 1 {
        grant_jiwu_skill(ctx, owner, avail[0]);
        return Flow::Done;
    }
    if avail.is_empty() {
        return Flow::Done;
    }
    m.card = Some(ASK_JIWU_SKILL);
    m.stage = HostStage::CompareTargetPick;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_jiwu_skill(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(ChoiceOption::Skill(def)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    grant_jiwu_skill(ctx, m.owner, def);
    Flow::Done
}

fn grant_jiwu_skill(ctx: &mut Ctx<'_>, owner: PlayerId, def: crate::ids::SkillDefId) {
    ctx.log(LogEvent::SkillTriggered { skill: "极武", owner, presentation: None });
    ctx.insert_skill(
        (owner, def),
        crate::state::SkillInstance {
            owner,
            enabled: true,
            once_used: false,
            expires_end_of_turn: true,
            rounds: 0,
        },
    );
}

pub(super) fn finish_shenqu_tao(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if !ctx.core.player(owner).hand.contains(&c) || ctx.catalog.card_kind(c) != CardKind::Tao {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered { skill: "神躯", owner, presentation: None });
    ctx.emit(EventData::CardUsed(crate::machine::card_use(c, owner, vec![owner])));
    Flow::Done
}

pub(super) fn start_boss_lingfeng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    for frame in ctx.res.stack.iter_mut().rev() {
        if let EventData::DrawPhase { player, pending, outcome, .. } = &mut frame.data {
            if *player == owner {
                *pending = 0;
                *outcome = crate::event::DrawOutcome::Cancelled;
                break;
            }
        }
    }
    ctx.log(LogEvent::SkillTriggered { skill: "灵锋", owner, presentation: None });
    let c1 = ctx.draw_judge_card();
    let c2 = ctx.draw_judge_card();
    let red1 = matches!(ctx.catalog.cards[c1 as usize].suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
    let red2 = matches!(ctx.catalog.cards[c2 as usize].suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
    ctx.core.table.push(c1);
    ctx.core.table.push(c2);
    ctx.log(LogEvent::CardsRevealed { cards: vec![c1, c2], to: Vec::new() });
    m.cards = vec![u32::from(c1), u32::from(c2)];
    let has_enemy = (0..ctx.core.players.len() as PlayerId)
        .any(|p| p != owner && ctx.core.player(p).alive && ctx.core.is_hostile_to(ctx.catalog, owner, p));
    if red1 != red2 && has_enemy {
        m.card = Some(ASK_LINGFENG_TARGET);
        m.stage = HostStage::TargetChoice;
        wait_q4_boss(m, ctx)
    } else {
        finish_lingfeng_gain(m, ctx)
    }
}

pub(super) fn step_lingfeng_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() {
        m.card = Some(ASK_LINGFENG_GAIN);
        m.stage = HostStage::DrawSettle;
        ctx.emit(EventData::LoseHp { player: tgt, amount: 1 });
        return Flow::Yield;
    }
    finish_lingfeng_gain(m, ctx)
}

pub(super) fn finish_lingfeng_gain(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let cards: Vec<CardId> = m.cards.drain(..).map(|raw| raw as CardId).collect();
    let mut gained = Vec::new();
    for c in cards {
        ctx.core.table.retain(|x| *x != c);
        if ctx.core.player(owner).alive {
            ctx.core.player_mut(owner).hand.push(c);
            gained.push(c);
        } else {
            ctx.to_discard(c);
        }
    }
    if !gained.is_empty() {
        ctx.log(LogEvent::CardsDrawn { player: owner, cards: gained });
    }
    Flow::Done
}

pub(super) fn step_xuanfeng_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    let pl = ctx.core.player(tgt);
    if tgt == owner || !pl.alive || (pl.hand.is_empty() && pl.equips.is_empty()) {
        return Flow::Done;
    }
    m.target = Some(tgt);
    m.card = Some(ASK_XUANFENG_CARD);
    m.stage = HostStage::SelectCard;
    wait_q4_boss(m, ctx)
}

pub(super) fn step_xuanfeng_card(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let (Some(tgt), Some(ChoiceOption::Card(c))) = (m.target, ctx.answer().first().copied()) else {
        return Flow::Done;
    };
    let pl = ctx.core.player(tgt);
    if !pl.hand.contains(&c) && !pl.equips.contains(&c) {
        return Flow::Done;
    }
    let rem_before = m.cards.first().copied().unwrap_or(2);
    if rem_before == 2 {
        ctx.log(LogEvent::SkillTriggered { skill: "旋风", owner, presentation: None });
    }
    discard_owned_card_and_emit(ctx, tgt, c);
    let rem = rem_before.saturating_sub(1);
    if rem == 0 {
        return Flow::Done;
    }
    m.cards = vec![rem];
    m.target = None;
    m.card = Some(ASK_XUANFENG_TARGET);
    m.stage = HostStage::DrawSettle;
    Flow::Yield
}

pub(super) fn resume_xuanfeng_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    if !ctx.core.player(m.owner).alive {
        return Flow::Done;
    }
    m.stage = HostStage::TargetChoice;
    wait_q4_boss(m, ctx)
}

pub(super) fn is_equip_slot_open(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    slot: EquipSlot,
) -> bool {
    let idx: u8 = match slot {
        EquipSlot::Weapon => 1,
        EquipSlot::Armor => 2,
        EquipSlot::HorsePlus => 3,
        EquipSlot::HorseMinus => 4,
        EquipSlot::Treasure => 5,
    };
    let mask = core
        .vm_aux
        .flags
        .get(&(owner, "disabled_equip_slots".to_string()))
        .copied()
        .unwrap_or(0);
    let flagged = core
        .vm_aux
        .flags
        .get(&(owner, format!("equip_slot_disabled:{idx}")))
        .copied()
        .unwrap_or(0);
    let bit_disabled = (mask & (1u32 << idx)) != 0 || (idx == 1 && (mask & 1) != 0);
    !bit_disabled
        && flagged == 0
        && !core
            .player(owner)
            .equips
            .iter()
            .any(|c| catalog.card_slot_of(core, *c) == Some(slot))
}

pub(super) fn start_boss_jingjia(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    ctx.core.vm_aux.flags.insert((owner, "q4_jingjia_done".to_string()), 1);
    let mut candidates: Vec<CardId> = ctx
        .core
        .deck
        .iter()
        .copied()
        .filter(|&c| {
            matches!(
                ctx.catalog.card_slot_of(ctx.core, c),
                Some(EquipSlot::Weapon | EquipSlot::Armor | EquipSlot::Treasure)
            )
        })
        .collect();
    if candidates.is_empty() {
        return Flow::Done;
    }
    ctx.core.rng.shuffle(&mut candidates);
    ctx.log(LogEvent::SkillTriggered { skill: "精甲", owner, presentation: None });
    let mut first = true;
    let mut used_slots = Vec::new();
    let mut chosen = Vec::new();
    for c in candidates {
        let Some(slot) = ctx.catalog.card_slot_of(ctx.core, c) else {
            continue;
        };
        if used_slots.contains(&slot) || !is_equip_slot_open(ctx.core, ctx.catalog, owner, slot) {
            continue;
        }
        let pass = if first {
            let _ = ctx.core.rng.gen_range(2);
            first = false;
            true
        } else {
            ctx.core.rng.gen_range(2) == 0
        };
        if pass {
            ctx.core.deck.retain(|x| *x != c);
            ctx.core.table.push(c);
            used_slots.push(slot);
            chosen.push(u32::from(c));
        }
    }
    if chosen.is_empty() {
        return Flow::Done;
    }
    m.cards = chosen;
    step_jingjia_next(m, ctx)
}

pub(super) fn step_jingjia_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.cards.is_empty() {
        return Flow::Done;
    }
    if !ctx.core.player(owner).alive {
        for raw in m.cards.drain(..) {
            let c = raw as CardId;
            ctx.core.table.retain(|x| *x != c);
            ctx.to_discard(c);
        }
        return Flow::Done;
    }
    let eq = m.cards.remove(0) as CardId;
    ctx.core.table.retain(|x| *x != eq);
    ctx.core.player_mut(owner).hand.push(eq);
    ctx.emit(EventData::Equipped { player: owner, card: eq, replaced: None });
    if m.cards.is_empty() {
        Flow::Done
    } else {
        m.card = Some(ASK_JINGJIA_NEXT);
        m.stage = HostStage::DrawSettle;
        Flow::Yield
    }
}

pub(super) fn start_boss_tianyujg(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let queue: Vec<PlayerId> = ctx
        .core
        .seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            ctx.core.player(p).alive
                && !ctx.core.player(p).linked
                && ctx.core.is_hostile_to(ctx.catalog, owner, p)
        })
        .collect();
    if queue.is_empty() {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered { skill: "天狱", owner, presentation: None });
    m.targets = queue;
    step_tianyujg_next(m, ctx)
}

pub(super) fn step_tianyujg_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    while !m.targets.is_empty() {
        let tgt = m.targets.remove(0);
        if ctx.core.player(tgt).alive
            && !ctx.core.player(tgt).linked
            && ctx.core.is_hostile_to(ctx.catalog, owner, tgt)
        {
            ctx.emit(EventData::Linked { player: tgt, linked: true });
            return if m.targets.is_empty() {
                Flow::Done
            } else {
                m.card = Some(ASK_TIANYUJG_NEXT);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            };
        }
    }
    Flow::Done
}

pub(super) fn start_boss_xuanlei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let queue: Vec<PlayerId> = ctx
        .core
        .seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            ctx.core.player(p).alive
                && !ctx.core.player(p).judge.is_empty()
                && ctx.core.is_hostile_to(ctx.catalog, owner, p)
        })
        .collect();
    if queue.is_empty() {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered { skill: "玄雷", owner, presentation: None });
    m.targets = queue;
    step_xuanlei_next(m, ctx)
}

pub(super) fn step_xuanlei_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    while !m.targets.is_empty() {
        let tgt = m.targets.remove(0);
        if ctx.core.player(tgt).alive {
            ctx.emit(EventData::Damage(damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Thunder,
                None,
            )));
            return if m.targets.is_empty() {
                Flow::Done
            } else {
                m.card = Some(ASK_XUANLEI_NEXT);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            };
        }
    }
    Flow::Done
}

pub(super) fn start_boss_skonghun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let queue: Vec<PlayerId> = ctx
        .core
        .seat_order_from(owner)
        .into_iter()
        .filter(|&p| ctx.core.player(p).alive && ctx.core.is_hostile_to(ctx.catalog, owner, p))
        .collect();
    if queue.is_empty() {
        return Flow::Done;
    }
    ctx.log(LogEvent::SkillTriggered { skill: "控魂", owner, presentation: None });
    m.cards = vec![queue.len() as u32];
    m.targets = queue;
    step_skonghun_next(m, ctx)
}

pub(super) fn step_skonghun_next(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    while !m.targets.is_empty() {
        let tgt = m.targets.remove(0);
        if ctx.core.player(tgt).alive {
            m.card = Some(ASK_SKONGHUN_NEXT);
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::Damage(damage_data_with(
                Some(owner),
                tgt,
                1,
                DamageNature::Thunder,
                None,
            )));
            return Flow::Yield;
        }
    }
    let count = m.cards.first().copied().unwrap_or(0);
    if count > 0 {
        ctx.recover(owner, count, None);
    }
    Flow::Done
}

fn card_savable_tao(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    dying_tgt: PlayerId,
) -> bool {
    if !core.player(owner).alive || core.player(dying_tgt).hp >= 1 {
        return false;
    }
    if catalog.blocks_all_cards(core, owner) {
        return false;
    }
    if catalog.wansha_blocks_rescue(core, owner, dying_tgt) {
        return false;
    }
    if let Some(tao_idx) = catalog.cards.iter().position(|c| c.kind == CardKind::Tao) {
        let tao_id = tao_idx as CardId;
        if crate::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
            core, catalog, owner, dying_tgt, tao_id,
        ) || crate::rvm_bridge::modifiers::eval_card_use_canceled(
            core,
            catalog,
            owner,
            tao_id,
            CardKind::Tao,
            DamageNature::Normal,
        ) {
            return false;
        }
    } else if crate::rvm_bridge::modifiers::d34_flags::apply_dying_peach_target_canceled(
        core,
        catalog,
        owner,
        dying_tgt,
        0,
        CardKind::Tao,
        false,
    ) {
        return false;
    }
    true
}

pub(super) fn step_danjing_save(m: &HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(dying_tgt) = m.target else {
        return Flow::Done;
    };
    if !ctx.core.player(dying_tgt).alive || !card_savable_tao(ctx.core, ctx.catalog, owner, dying_tgt) {
        return Flow::Done;
    }
    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
        card: None,
        player: owner,
        targets: vec![dying_tgt],
        nature: DamageNature::Normal,
        use_as: Some(CardKind::Tao),
        cancelled: false,
    }));
    Flow::Done
}
