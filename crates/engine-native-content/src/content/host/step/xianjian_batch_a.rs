//! U05 仙剑 Batch A（9 技）：`qiongguang`, `huanlei`, `tianwu`, `yujia`, `yanshi`, `tanhua`, `ywuhun`, `qixia`, `jianzhen`。

use super::xianjian_batch_asks::*;
use super::*;
use crate::catalog::{EquipSlot, QueryCtx, SkillLimits, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::resolution::{Park, Slot};
use crate::state::GameStateCore;

pub(super) fn trigger_meta(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, priority: i32, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "qiongguang" => Some(meta(EventKind::PhaseEnd, 0, Vec::new())),
        "huanlei" => Some(meta(EventKind::DamageEnd, 0, Vec::new())),
        "tianwu" => Some(meta(EventKind::CardUsed, 0, Vec::new())),
        "yujia" => Some(meta(EventKind::Equipped, 0, vec![EventKind::CardUsed])),
        "yanshi" => Some(meta(EventKind::TurnPhaseAfter, 0, Vec::new())),
        "tanhua" => Some(meta(EventKind::Dying, 10, Vec::new())),
        "ywuhun" => Some(meta(EventKind::TurnBegin, 0, vec![EventKind::TurnPhaseEnd])),
        "qixia" => Some(meta(EventKind::TurnPhaseAfter, -50, vec![EventKind::CardResponded])),
        _ => None,
    }
}

pub(super) fn limits(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(super) fn filter_with_frame(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frame: &crate::event::Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "qiongguang" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Discard, begin: false }
            if *player == owner
                && ctx.state.turn.discard_phase_discarded.max(flag(ctx.state, owner, "xj_last_discard_count")) > 1
                && !alive_enemies(ctx.state, owner).is_empty()
        ),
        "huanlei" => matches!(
            event,
            EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0 && !d.settlement.notrigger
                && d.source.is_some_and(|s| s != owner && ctx.state.player(s).alive && ctx.state.player(s).hp > ctx.state.player(owner).hp)
        ),
        "tianwu" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player == owner && !u.cancelled && u.targets.len() == 1
                && ctx.state.player(u.targets[0]).alive && !is_friendly(ctx.state, owner, u.targets[0])
        ),
        "yujia" => match event {
            EventData::Equipped { player, card, .. } => {
                *player == owner
                    && !ctx.catalog.cards[*card as usize].off_deck
                    && flag(ctx.state, owner, &format!("u05_play_equip_frame:{}", frame.id)) > 0
                    && ctx.catalog.card_kind(*card).equip_slot().is_some()
                    && !yujia_jiguan_pool(ctx.state, ctx.catalog).is_empty()
            }
            EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && u.card
                        .is_some_and(|c| !ctx.catalog.cards[c as usize].off_deck)
                    && u.effective_kind(ctx.catalog).equip_slot().is_some()
                    && !yujia_jiguan_pool(ctx.state, ctx.catalog).is_empty()
            }
            _ => false,
        },
        "yanshi" => matches!(
            event,
            EventData::TurnEndWindow { player, after: true }
            if *player == owner && flag(ctx.state, owner, "xj_yanshi_count") >= 4 && flag(ctx.state, owner, "xj_yanshi_done") == 0
        ),
        "tanhua" => matches!(event, EventData::Dying { player, .. } if *player == owner && ctx.state.player(owner).hp <= 0),
        "ywuhun" => match event {
            EventData::TurnBegin { player } if *player == owner => flag(ctx.state, owner, "ywuhun_active") == 0,
            EventData::TurnEndWindow { player, after: false } if *player == owner => flag(ctx.state, owner, "ywuhun_active") > 0,
            _ => false,
        },
        "qixia" => match event {
            EventData::CardResponded { player, card } if *player == owner => {
                let b = suit_bit(ctx.catalog.effective_card_suit(ctx.state, owner, *card));
                (flag(ctx.state, owner, "xj_qixia_suits") & b) == 0
            }
            EventData::TurnEndWindow { after: true, .. } => flag(ctx.state, owner, "xj_qixia_suits").count_ones() >= 4,
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn machine(skill: &HostSkillBehavior, owner: PlayerId, event: &EventData) -> HostMachine {
    let mut target = None;
    let mut cards = Vec::new();
    match (skill.name.as_str(), event) {
        ("huanlei", EventData::Damage(d)) => target = d.source,
        ("tianwu", EventData::CardUsed(u)) => target = u.targets.first().copied(),
        ("qixia", EventData::CardResponded { card, .. }) => cards.push(*card),
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::U05XianjianBatchA,
        owner,
        target,
        card: None,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "qiongguang" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_QIONGGUANG_CONFIRM),
            HostStage::SelectNature => {
                if answered_confirm(ctx) {
                    for enemy in alive_enemies(ctx.core, owner) {
                        u05_apply_debuff(ctx, enemy);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "huanlei" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_HUANLEI_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let Some(src) = m.target.filter(|&s| ctx.core.player(s).alive) else {
                    return Flow::Done;
                };
                m.stage = HostStage::DrawSettle;
                m.card = None;
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    src,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if let Some(src) = m.target.filter(|&s| ctx.core.player(s).alive) {
                    ctx.start_draw(src, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "tianwu" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_TIANWU_CONFIRM),
            HostStage::SelectNature => {
                if answered_confirm(ctx) {
                    if let Some(t) = m.target.filter(|&t| ctx.core.player(t).alive) {
                        u05_apply_debuff(ctx, t);
                        ensure_temp_skill(ctx, owner, "tianwu2", true);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yujia" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, HostStage::SelectNature, ASK_YUJIA_CONFIRM),
            HostStage::SelectNature => {
                if !answered_confirm(ctx) {
                    return Flow::Done;
                }
                let pool = yujia_jiguan_pool(ctx.core, ctx.catalog);
                if pool.is_empty() {
                    return Flow::Done;
                }
                let cnt = flag(ctx.core, owner, "xj_yujia_count") as usize;
                if cnt > 1 {
                    let mut uniq = Vec::new();
                    let mut rem = pool;
                    while !rem.is_empty() && uniq.len() < cnt {
                        let i = ctx.core.rng.gen_range(rem.len() as u32) as usize;
                        uniq.push(rem.remove(i));
                    }
                    m.cards = uniq;
                    return wait_ask(m, ctx, HostStage::SelectCard, ASK_YUJIA_PICK);
                }
                let i = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                give_pile_card(ctx, owner, pool[i]);
                Flow::Done
            }
            HostStage::SelectCard => {
                if let Some(c) = answered_cards(ctx).into_iter().next() {
                    give_pile_card(ctx, owner, c);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "yanshi" => {
            if flag(ctx.core, owner, "xj_yanshi_done") != 0 {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "xj_yanshi_done", 1);
            ctx.core.player_mut(owner).max_hp += 1;
            ctx.recover(owner, 1, None);
            let slots = [
                EquipSlot::Weapon,
                EquipSlot::Armor,
                EquipSlot::HorsePlus,
                EquipSlot::HorseMinus,
                EquipSlot::Treasure,
            ];
            for slot in slots {
                if ctx.core.player(owner).equips.iter().any(|&e| ctx.catalog.card_slot_of(ctx.core, e) == Some(slot)) {
                    continue;
                }
                let matching: Vec<CardId> = ctx
                    .core
                    .deck
                    .iter()
                    .chain(ctx.core.discard.iter())
                    .copied()
                    .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(slot))
                    .collect();
                if matching.is_empty() {
                    continue;
                }
                let i = ctx.core.rng.gen_range(matching.len() as u32) as usize;
                let eq = matching[i];
                remove_from_deck_or_discard(ctx.core, eq);
                ctx.core.player_mut(owner).equips.push(eq);
                ctx.log(LogEvent::Equipped { player: owner, card: eq, replaced: None });
                crate::kernel::grant_equip_skills(ctx, owner, eq);
            }
            Flow::Done
        }
        "tanhua" => {
            if ctx.core.player(owner).alive && ctx.core.player(owner).hp <= 0 {
                ctx.recover(owner, 1, None);
                remove_skill_named(ctx, owner, "tanhua");
            }
            Flow::Done
        }
        "ywuhun" => {
            if flag(ctx.core, owner, "ywuhun_active") == 0 {
                snapshot_ywuhun(ctx.core, owner);
            } else {
                restore_ywuhun(ctx, owner);
            }
            Flow::Done
        }
        "qixia" => {
            if let Some(c) = m.cards.pop() {
                let prev = flag(ctx.core, owner, "xj_qixia_suits");
                let b = suit_bit(ctx.catalog.effective_card_suit(ctx.core, owner, c));
                set_flag(ctx.core, owner, "xj_qixia_suits", prev | b);
                return Flow::Done;
            }
            if flag(ctx.core, owner, "xj_qixia_suits").count_ones() >= 4 {
                set_flag(ctx.core, owner, "xj_qixia_suits", 0);
                set_flag(ctx.core, owner, "extra_turn_insert_queued", 1);
            }
            Flow::Done
        }
        "jianzhen" => step_jianzhen(m, ctx),
        _ => Flow::Done,
    }
}

fn step_jianzhen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let Some(t) = m.target else {
        return Flow::Done;
    };
    if !ctx.core.player(t).alive {
        return Flow::Done;
    }
    if m.stage == HostStage::TargetChoice {
        let Some(&cand) = m.targets.first() else {
            return Flow::Done;
        };
        if let Some(sha) = answered_cards(ctx).into_iter().next() {
            if ctx.core.player(cand).hand.contains(&sha) && ctx.catalog.card_kind(sha).is_sha_like() {
                ctx.emit(EventData::CardUsed(sha_card_use_data(ctx, cand, sha, vec![t], None)));
                return Flow::Done;
            }
        }
        m.targets.remove(0);
    }
    while let Some(&cand) = m.targets.first() {
        if ctx.core.player(cand).alive
            && !sha_user_blocked(ctx.core, ctx.catalog, cand, t)
            && sha_target_legal(ctx.core, ctx.catalog, cand, t, false)
            && ctx.core.player(cand).hand.iter().any(|&c| ctx.catalog.card_kind(c).is_sha_like())
        {
            return wait_ask(m, ctx, HostStage::TargetChoice, ASK_JIANZHEN_SHA);
        }
        m.targets.remove(0);
    }
    Flow::Done
}

pub(crate) fn host_sha_end_xianjian(
    ctx: &mut Ctx<'_>,
    sha_frame_id: EventId,
    source: PlayerId,
    target: PlayerId,
) -> bool {
    let Some(def) = ctx.catalog.skill_by_name("jianzhen") else {
        return false;
    };
    if !ctx.core.has_skill(source, def) || ctx.catalog.is_tieji_sealed(ctx.core, source) || !ctx.core.player(target).alive {
        return false;
    }
    let key = format!("sha_jianzhen_done:{sha_frame_id}:{target}");
    if flag(ctx.core, source, &key) != 0 {
        return false;
    }
    let candidates: Vec<PlayerId> = ctx
        .core
        .seat_order_from(source)
        .into_iter()
        .filter(|&p| {
            p != source
                && ctx.core.player(p).alive
                && sha_target_legal(ctx.core, ctx.catalog, p, target, false)
                && ctx.core.player(p).hand.iter().any(|&c| ctx.catalog.card_kind(c).is_sha_like())
        })
        .collect();
    if candidates.is_empty() {
        return false;
    }
    set_flag(ctx.core, source, &key, 1);
    ctx.out.push(LogEvent::SkillTriggered {
        skill: ctx.catalog.skills[def as usize].name,
        owner: source,
        presentation: None,
    });
    ctx.res.parks.push(Park {
        resume_depth: ctx.res.stack.len(),
        slot: Slot::Trigger { frame: sha_frame_id },
        machine: Box::new(HostMachine {
            def_key: "jianzhen".to_string(),
            behavior: HostBehavior::U05XianjianBatchA,
            owner: source,
            target: Some(target),
            card: None,
            cards: Vec::new(),
            targets: candidates,
            stage: HostStage::Act,
        }),
        awaiting_answer: false,
    });
    true
}

fn remove_from_deck_or_discard(core: &mut GameStateCore, card: CardId) {
    if let Some(pos) = core.deck.iter().rposition(|&c| c == card) {
        core.deck.remove(pos);
    } else if let Some(pos) = core.discard.iter().rposition(|&c| c == card) {
        core.discard.remove(pos);
    } else {
        core.detached_cards.retain(|&(c, _)| c != card);
    }
}

fn give_pile_card(ctx: &mut Ctx<'_>, owner: PlayerId, card: CardId) {
    remove_from_deck_or_discard(ctx.core, card);
    ctx.core.player_mut(owner).hand.push(card);
    ctx.log(LogEvent::CardsDrawn { player: owner, cards: vec![card] });
}

fn snapshot_ywuhun(core: &mut GameStateCore, owner: PlayerId) {
    let seats = core.players.len() as PlayerId;
    set_flag(core, owner, "ywuhun_active", 1);
    set_flag(core, owner, "ywuhun_deck_len", core.deck.len() as u32);
    let deck_len = core.deck.len();
    for i in 0..deck_len {
        let c = core.deck[i];
        set_flag(core, owner, &format!("ywuhun_d:{i}"), c + 1);
    }
    for s in 0..seats {
        let p = core.player(s).clone();
        set_flag(core, s, "ywuhun_h_len", p.hand.len() as u32);
        for (i, &c) in p.hand.iter().enumerate() {
            set_flag(core, s, &format!("ywuhun_h:{i}"), c + 1);
        }
        set_flag(core, s, "ywuhun_e_len", p.equips.len() as u32);
        for (i, &c) in p.equips.iter().enumerate() {
            set_flag(core, s, &format!("ywuhun_e:{i}"), c + 1);
        }
        set_flag(core, s, "ywuhun_j_len", p.judge.len() as u32);
        for (i, &c) in p.judge.iter().enumerate() {
            set_flag(core, s, &format!("ywuhun_j:{i}"), c + 1);
        }
        set_flag(core, s, "ywuhun_oc_len", p.off_court.len() as u32);
        for (i, &c) in p.off_court.iter().enumerate() {
            set_flag(core, s, &format!("ywuhun_oc:{i}"), c + 1);
        }
    }
    let pile_snapshot: Vec<((PlayerId, String), Vec<CardId>)> = core
        .vm_aux
        .piles
        .iter()
        .map(|((s, k), v)| ((*s, k.clone()), v.clone()))
        .collect();
    for ((s, pname), cards) in pile_snapshot {
        set_flag(core, s, &format!("ywuhun_pile:{pname}:len"), cards.len() as u32);
        for (i, &c) in cards.iter().enumerate() {
            set_flag(core, s, &format!("ywuhun_pile:{pname}:{i}"), c + 1);
        }
    }
}

fn restore_ywuhun(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "ywuhun_active", 0);
    set_flag(ctx.core, owner, "xj_ywuhun_abort", 0);
    let seats = ctx.core.players.len() as PlayerId;
    let saved_piles: Vec<(PlayerId, String)> = ctx
        .core
        .vm_aux
        .flags
        .keys()
        .filter_map(|&(s, ref k)| {
            k.strip_prefix("ywuhun_pile:")
                .and_then(|rest| rest.strip_suffix(":len"))
                .map(|pname| (s, pname.to_string()))
        })
        .collect();
    let mut pool = Vec::new();
    pool.extend(ctx.core.deck.drain(..));
    pool.extend(ctx.core.discard.drain(..));
    pool.extend(ctx.core.table.drain(..));
    for pile in ctx.core.vm_aux.piles.values_mut() {
        pool.extend(pile.drain(..));
    }
    ctx.core.vm_aux.piles.clear();
    for s in 0..seats {
        let equips: Vec<CardId> = ctx.core.player(s).equips.clone();
        for c in equips {
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, s, c);
        }
        let pl = ctx.core.player_mut(s);
        pool.extend(pl.hand.drain(..));
        pool.extend(pl.equips.drain(..));
        pool.extend(pl.judge.drain(..));
        pool.extend(pl.off_court.drain(..));
    }
    let mut take_card = |c: CardId| -> Option<CardId> {
        pool.iter().position(|&x| x == c).map(|pos| pool.remove(pos))
    };
    for s in 0..seats {
        let h_len = flag(ctx.core, s, "ywuhun_h_len") as usize;
        let e_len = flag(ctx.core, s, "ywuhun_e_len") as usize;
        let j_len = flag(ctx.core, s, "ywuhun_j_len") as usize;
        let oc_len = flag(ctx.core, s, "ywuhun_oc_len") as usize;
        if ctx.core.player(s).alive {
            for i in 0..h_len {
                if let Some(c) = flag(ctx.core, s, &format!("ywuhun_h:{i}")).checked_sub(1).and_then(&mut take_card) {
                    ctx.core.player_mut(s).hand.push(c);
                }
            }
            let mut rest_eq = Vec::new();
            for i in 0..e_len {
                if let Some(c) = flag(ctx.core, s, &format!("ywuhun_e:{i}")).checked_sub(1).and_then(&mut take_card) {
                    ctx.core.player_mut(s).equips.push(c);
                    rest_eq.push(c);
                }
            }
            for c in rest_eq {
                crate::kernel::grant_equip_skills(ctx, s, c);
            }
            for i in 0..j_len {
                if let Some(c) = flag(ctx.core, s, &format!("ywuhun_j:{i}")).checked_sub(1).and_then(&mut take_card) {
                    ctx.core.player_mut(s).judge.push(c);
                }
            }
            for i in 0..oc_len {
                if let Some(c) = flag(ctx.core, s, &format!("ywuhun_oc:{i}")).checked_sub(1).and_then(&mut take_card) {
                    ctx.core.player_mut(s).off_court.push(c);
                }
            }
        }
    }
    for (s, pname) in saved_piles {
        let p_len = flag(ctx.core, s, &format!("ywuhun_pile:{pname}:len")) as usize;
        let mut restored = Vec::with_capacity(p_len);
        for i in 0..p_len {
            if let Some(c) = flag(ctx.core, s, &format!("ywuhun_pile:{pname}:{i}")).checked_sub(1).and_then(&mut take_card) {
                restored.push(c);
            }
        }
        if !restored.is_empty() {
            ctx.core.vm_aux.piles.insert((s, pname), restored);
        }
    }
    let d_len = flag(ctx.core, owner, "ywuhun_deck_len") as usize;
    for i in 0..d_len {
        if let Some(c) = flag(ctx.core, owner, &format!("ywuhun_d:{i}")).checked_sub(1).and_then(&mut take_card) {
            ctx.core.deck.push(c);
        }
    }
    ctx.core.discard.extend(pool);
    ctx.core.vm_aux.flags.retain(|(_, k), _| !k.starts_with("ywuhun_"));
}
