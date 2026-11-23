//! Batch U05-1 万智牌 (`character/mtg/skill.js`) 武将触发与交互步进实现。

use super::super::*;
use super::l27_batch_a::{alive_others, discard_cards, flag, set_flag};
use super::u05_b1_mtg_asks::{
    is_delayed_trick_kind, jiedao_victims_for, msuoling_auto_targets, target_enabled2_for_card,
    trick_targets_for, wait_ask_mtg,
};
use crate::catalog::{CardKind, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DrawOutcome, EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{card_use, Ctx, Flow};
use crate::state::GameStateCore;

pub(crate) fn apply_msilian_draw_bonus(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    let spent = flag(ctx.core, player, "u05_msilian_extra_active");
    if spent > 0 {
        let bonus = spent as i32 - 2;
        *pending = (*pending + bonus).max(0);
    }
}

pub(in crate::content::host) fn trigger_meta_u05_mtg(name: &str) -> Option<TriggerMeta> {
    match name {
        "mbaizhan" => Some(TriggerMeta {
            event: EventKind::Damage,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        "msilian" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: vec![EventKind::TurnPhaseAfter],
        }),
        "mduohun" => Some(TriggerMeta {
            event: EventKind::DyingSaved,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        "mhuanyi" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 6,
            order_selectable: true,
            optional: false,
            extra_events: vec![EventKind::CardUsed],
        }),
        "msuoling" => Some(TriggerMeta {
            event: EventKind::CardUsed,
            timing: Timing::Normal,
            priority: 5,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        "mtongnian" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: vec![
                EventKind::CardUsed,
                EventKind::CardsLost,
                EventKind::CardsDiscarded,
            ],
        }),
        "mtg_duzhao_skill" | "mtg_lindixiliu_skill" => Some(TriggerMeta {
            event: EventKind::PhaseBegin,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        "mtg_cangbaohaiwan_skill" => Some(TriggerMeta {
            event: EventKind::DrawPhaseBefore,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        "mtg_bingheyaosai_skill" => Some(TriggerMeta {
            event: EventKind::CardUsed,
            timing: Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        "subplayer" => Some(TriggerMeta {
            event: EventKind::Death,
            timing: Timing::Normal,
            priority: -9,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        }),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u05_mtg(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn filter_u05_mtg(
    m: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    if m.name == "subplayer" {
        return matches!(event, EventData::Death { player, .. } if *player == owner)
            && super::u05_b1_mtg_subplayer::has_active_subplayer(ctx.state, owner);
    }
    if ctx.catalog.is_tieji_sealed(ctx.state, owner) {
        return false;
    }
    match m.name.as_str() {
        "mbaizhan" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && !d.prevented && d.amount > 0
        ),
        "msilian" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && ctx.state.player(owner).hujia > 0
                    && flag(ctx.state, owner, "u05_msilian_extra_active") == 0
                    && flag(ctx.state, owner, "u05_msilian_turn_pending") == 0
            }
            EventData::TurnEndWindow { after: true, .. } => {
                flag(ctx.state, owner, "u05_msilian_extra_active") > 0
            }
            _ => false,
        },
        "mduohun" => matches!(
            event,
            EventData::DyingSaved { player, source: Some(s) }
                if *player == owner && *s != owner && ctx.state.player(*s).alive
        ),
        "mhuanyi" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && !alive_others(ctx.state, owner).is_empty()
                    && ctx.state.turn.round.max(1) >= flag(ctx.state, owner, "u05_mhuanyi_ready_round")
            }
            EventData::CardUsed(u) if !u.cancelled && u.targets.contains(&owner) => {
                let tgt_v = flag(ctx.state, owner, "u05_mhuanyi_tgt");
                let cls_v = flag(ctx.state, owner, "u05_mhuanyi_cls");
                if tgt_v == 0 || cls_v == 0 {
                    return false;
                }
                let t = (tgt_v - 1) as PlayerId;
                if (t as usize) >= ctx.state.players.len()
                    || t == owner
                    || t == u.player
                    || !ctx.state.player(t).alive
                {
                    return false;
                }
                let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                let cls_ok = (cls_v == 1 && kind.is_basic_kind()) || (cls_v == 2 && !kind.is_basic_kind());
                cls_ok
                    && target_enabled2_for_card(
                        ctx.state,
                        ctx.catalog,
                        u.card,
                        kind,
                        u.suit(ctx.catalog),
                        u.player,
                        t,
                    )
            }
            _ => false,
        },
        "msuoling" => match event {
            EventData::CardUsed(u) if !u.cancelled && u.player != owner => {
                if ctx.state.turn.round.max(1) < flag(ctx.state, owner, "u05_msuoling_ready_round") {
                    return false;
                }
                let Some(cid) = u.card else {
                    return false;
                };
                let kind = ctx.catalog.card_kind(cid);
                // 注：原生 play_card 对物理过河/顺手会传 `use_as = Some(kind)`；仅在转化声明与目录牌种不一致时视为转化牌。
                if u.use_as.is_some_and(|k| k != kind) {
                    return false;
                }
                if !kind.is_trick_kind() || is_delayed_trick_kind(kind) {
                    return false;
                }
                let suit = ctx.catalog.effective_card_suit(ctx.state, u.player, cid);
                ctx.state
                    .player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .any(|&c| ctx.catalog.effective_card_suit(ctx.state, owner, c) == suit)
            }
            _ => false,
        },
        "mtongnian" => {
            let has_link = ctx
                .state
                .player(owner)
                .hand
                .iter()
                .any(|&c| flag(ctx.state, owner, &format!("mtongnian_link:{c}")) > 0);
            match event {
                EventData::Phase {
                    player,
                    phase: Phase::Play,
                    begin: true,
                } => *player == owner && !alive_enemies_hand_empty(ctx.state, owner),
                EventData::CardUsed(u) if !u.cancelled && u.player == owner => {
                    let uses_link =
                        u.card.is_some_and(|c| flag(ctx.state, owner, &format!("mtongnian_link:{c}")) > 0);
                    uses_link || has_link
                }
                EventData::CardUsed(u) if !u.cancelled && u.player != owner => {
                    has_link && alive_enemies_hand_empty(ctx.state, owner)
                }
                EventData::CardsLost { player, .. } | EventData::CardsDiscarded { player, .. } => {
                    *player != owner && has_link && alive_enemies_hand_empty(ctx.state, owner)
                }
                _ => false,
            }
        }
        "mtg_duzhao_skill" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } if *player == owner && flag(ctx.state, 0, "u05_extra_land") == 1
        ),
        "mtg_cangbaohaiwan_skill" => matches!(
            event,
            EventData::DrawPhase {
                player,
                outcome: DrawOutcome::Running,
                ..
            } if *player == owner && flag(ctx.state, 0, "u05_extra_land") == 2
        ),
        "mtg_lindixiliu_skill" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && ctx.state.current_land == Some(CardKind::Lindixiliu)
                    && !ctx.state.player(owner).judge.is_empty()
                    && ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .chain(ctx.state.player(owner).equips.iter())
                        .any(|&c| ctx.catalog.effective_card_suit(ctx.state, owner, c) == Suit::Heart)
            }
            _ => false,
        },
        "mtg_bingheyaosai_skill" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player == owner
                    && ctx.state.current_land == Some(CardKind::Bingheyaosai)
                    && u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)).is_sha_like()
                    && (!ctx.state.player(owner).hand.is_empty() || !ctx.state.player(owner).equips.is_empty())
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u05_mtg(
    m: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<HostMachine> {
    (m.name == "lingyong"
        && !ctx.catalog.is_tieji_sealed(ctx.state, owner)
        && ctx.state.player(owner).alive
        && !super::u05_b1_mtg_subplayer::has_active_subplayer(ctx.state, owner)
        && !super::u05_b1_mtg_subplayer::active_subplayer_slots(ctx.state, owner).is_empty())
    .then(|| HostMachine {
        def_key: m.name.clone(),
        behavior: m.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    })
}

pub(in crate::content::host) fn alive_enemies_hand_empty(core: &GameStateCore, owner: PlayerId) -> bool {
    alive_others(core, owner)
        .into_iter()
        .all(|p| core.player(p).hand.is_empty())
}

fn rotate_mtongnian_links(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut pool = Vec::new();
    for p in alive_others(ctx.core, owner) {
        for &c in &ctx.core.player(p).hand {
            pool.push((p, c));
        }
    }
    if pool.is_empty() {
        return_mtongnian_links(ctx.core, owner);
        return;
    }
    let links: Vec<CardId> = ctx
        .core
        .player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| flag(ctx.core, owner, &format!("mtongnian_link:{c}")) > 0)
        .collect();
    for fake in links {
        let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
        let (tgt, real_c) = pool[idx];
        set_flag(ctx.core, owner, &format!("mtongnian_link:{fake}"), real_c + 1);
        set_flag(ctx.core, owner, &format!("mtongnian_target:{fake}"), u32::from(tgt) + 1);
        ctx.core.declared_kinds.insert(fake, ctx.catalog.card_kind(real_c));
    }
}

pub(in crate::content::host) fn return_mtongnian_links(core: &mut GameStateCore, owner: PlayerId) {
    let links: Vec<CardId> = core
        .player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| flag(core, owner, &format!("mtongnian_link:{c}")) > 0)
        .collect();
    for fake in links {
        core.player_mut(owner).hand.retain(|&x| x != fake);
        core.declared_kinds.remove(&fake);
        set_flag(core, owner, &format!("mtongnian_link:{fake}"), 0);
        set_flag(core, owner, &format!("mtongnian_target:{fake}"), 0);
        core.vm_aux
            .piles
            .entry((0, "mtg_mirror_supply".to_string()))
            .or_default()
            .push(fake);
    }
}

fn step_msuoling(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let Some((cid, caster, cancelled)) = ctx.res.stack.last().and_then(|f| match &f.data {
                EventData::CardUsed(u) => u.card.map(|c| (c, u.player, u.cancelled)),
                _ => None,
            }) else {
                return Flow::Done;
            };
            if cancelled {
                return Flow::Done;
            }
            m.card = Some(cid);
            m.cards = vec![u32::from(caster)];
            wait_ask_mtg(m, ctx, HostStage::SelectCard)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(disc)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(stolen) = m.card else {
                return Flow::Done;
            };
            let Some(top_frame) = ctx.res.stack.last().map(|f| f.id) else {
                return Flow::Done;
            };
            let already_cancelled = matches!(
                &ctx.res.frame(top_frame).map(|f| &f.data),
                Some(EventData::CardUsed(u)) if u.cancelled
            );
            if already_cancelled {
                return Flow::Done;
            }
            let round = ctx.core.turn.round.max(1);
            set_flag(ctx.core, owner, "u05_msuoling_ready_round", round + 1);
            discard_cards(ctx, owner, &[disc]);
            if let Some(f) = ctx.res.frame_mut(top_frame) {
                if let EventData::CardUsed(ref mut u) = f.data {
                    u.cancelled = true;
                    u.targets.clear();
                }
            }
            let kind = ctx.catalog.card_kind(stolen);
            if crate::kernel::jinyan_blocks_black_trick(ctx.core, ctx.catalog, owner, stolen) {
                ctx.core.table.retain(|&x| x != stolen);
                ctx.to_discard(stolen);
                return Flow::Done;
            }
            if let Some(auto_targets) = msuoling_auto_targets(ctx.core, ctx.catalog, owner, kind) {
                if !ctx.core.table.contains(&stolen) {
                    ctx.core.table.push(stolen);
                }
                ctx.emit(EventData::CardUsed(card_use(stolen, owner, auto_targets)));
                return Flow::Done;
            }
            let (opts, min, _max) = trick_targets_for(ctx.core, ctx.catalog, owner, stolen);
            if opts.len() < min {
                ctx.core.table.retain(|&x| x != stolen);
                ctx.to_discard(stolen);
                return Flow::Done;
            }
            wait_ask_mtg(m, ctx, HostStage::TargetChoice)
        }
        HostStage::TargetChoice => {
            let Some(stolen) = m.card else {
                return Flow::Done;
            };
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if targets.is_empty() {
                ctx.core.table.retain(|&x| x != stolen);
                ctx.to_discard(stolen);
                return Flow::Done;
            }
            if ctx.catalog.card_kind(stolen) == CardKind::Jiedao {
                let holder = targets[0];
                if jiedao_victims_for(ctx.core, ctx.catalog, owner, stolen, holder).is_empty() {
                    ctx.core.table.retain(|&x| x != stolen);
                    ctx.to_discard(stolen);
                    return Flow::Done;
                }
                m.targets = vec![holder];
                return wait_ask_mtg(m, ctx, HostStage::CardsAsk);
            }
            if !ctx.core.table.contains(&stolen) {
                ctx.core.table.push(stolen);
            }
            ctx.emit(EventData::CardUsed(card_use(stolen, owner, targets)));
            Flow::Done
        }
        HostStage::CardsAsk => {
            let Some(stolen) = m.card else {
                return Flow::Done;
            };
            let Some(&holder) = m.targets.first() else {
                ctx.core.table.retain(|&x| x != stolen);
                ctx.to_discard(stolen);
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                ctx.core.table.retain(|&x| x != stolen);
                ctx.to_discard(stolen);
                return Flow::Done;
            };
            if !ctx.core.table.contains(&stolen) {
                ctx.core.table.push(stolen);
            }
            ctx.emit(EventData::CardUsed(card_use(stolen, owner, vec![holder, victim])));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_mtongnian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(frame_id) = ctx.res.stack.last().map(|f| f.id) {
        if matches!(
            ctx.res.frame(frame_id).map(|f| &f.data),
            Some(EventData::CardsLost { .. } | EventData::CardsDiscarded { .. })
        ) {
            if alive_enemies_hand_empty(ctx.core, owner) {
                return_mtongnian_links(ctx.core, owner);
            }
            return Flow::Done;
        }
        if let Some(EventData::CardUsed(u)) = ctx.res.frame(frame_id).map(|f| &f.data) {
            if u.player != owner {
                if alive_enemies_hand_empty(ctx.core, owner) {
                    return_mtongnian_links(ctx.core, owner);
                }
                return Flow::Done;
            }
            if let Some(fake) = u.card {
                let link_v = flag(ctx.core, owner, &format!("mtongnian_link:{fake}"));
                if link_v > 0 {
                    let real_c = link_v - 1;
                    let tgt = flag(ctx.core, owner, &format!("mtongnian_target:{fake}")).saturating_sub(1) as PlayerId;
                    set_flag(ctx.core, owner, &format!("mtongnian_link:{fake}"), 0);
                    set_flag(ctx.core, owner, &format!("mtongnian_target:{fake}"), 0);
                    ctx.core.declared_kinds.remove(&fake);
                    ctx.core.table.retain(|&x| x != fake);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((0, "mtg_mirror_supply".to_string()))
                        .or_default()
                        .push(fake);
                    let found_owner = if (tgt as usize) < ctx.core.players.len()
                        && ctx.core.player(tgt).hand.contains(&real_c)
                    {
                        Some(tgt)
                    } else {
                        (0..ctx.core.players.len() as PlayerId)
                            .find(|&p| ctx.core.player(p).hand.contains(&real_c))
                    };
                    if let Some(real_owner) = found_owner {
                        ctx.take_hand_card(real_owner, real_c);
                        ctx.core.table.push(real_c);
                        if let Some(f) = ctx.res.frame_mut(frame_id) {
                            if let EventData::CardUsed(ref mut u2) = f.data {
                                u2.card = Some(real_c);
                            }
                        }
                    } else if let Some(f) = ctx.res.frame_mut(frame_id) {
                        if let EventData::CardUsed(ref mut u2) = f.data {
                            u2.cancelled = true;
                            u2.card = None;
                        }
                    }
                    if alive_enemies_hand_empty(ctx.core, owner) {
                        return_mtongnian_links(ctx.core, owner);
                    }
                    return Flow::Done;
                }
            }
            if u.player != owner && alive_enemies_hand_empty(ctx.core, owner) {
                return_mtongnian_links(ctx.core, owner);
            } else {
                rotate_mtongnian_links(ctx, owner);
            }
            return Flow::Done;
        }
    }
    let mut pool = Vec::new();
    for p in alive_others(ctx.core, owner) {
        for &c in &ctx.core.player(p).hand {
            pool.push((p, c));
        }
    }
    if !pool.is_empty() {
        let fake = ctx
            .core
            .vm_aux
            .piles
            .get_mut(&(0, "mtg_mirror_supply".to_string()))
            .and_then(|v| (!v.is_empty()).then(|| v.remove(0)));
        if let Some(fake) = fake {
            let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
            let (tgt, real_c) = pool[idx];
            set_flag(ctx.core, owner, &format!("mtongnian_link:{fake}"), real_c + 1);
            set_flag(ctx.core, owner, &format!("mtongnian_target:{fake}"), u32::from(tgt) + 1);
            ctx.core.declared_kinds.insert(fake, ctx.catalog.card_kind(real_c));
            ctx.core.player_mut(owner).hand.push(fake);
        }
    }
    Flow::Done
}

pub(in crate::content::host) fn handle_u05_mtg_char(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "mbaizhan" => {
            if let Ok(d) = ctx.current_damage() {
                let amt = d.amount.max(0) as i32;
                ctx.change_hujia(owner, amt);
            }
            Flow::Done
        }
        "msilian" => match m.stage {
            HostStage::Act => {
                if flag(ctx.core, owner, "u05_msilian_extra_active") > 0 {
                    ctx.core.vm_aux.flags.remove(&(owner, "u05_msilian_extra_active".to_string()));
                    let dmg = flag(ctx.core, owner, "u05_msilian_damaged");
                    set_flag(ctx.core, owner, "u05_msilian_damaged", 0);
                    if dmg == 0 && ctx.core.player(owner).alive {
                        ctx.emit(EventData::LoseHp {
                            player: owner,
                            amount: 1,
                        });
                    }
                    return Flow::Done;
                }
                wait_ask_mtg(m, ctx, HostStage::SelectCard)
            }
            HostStage::SelectCard => {
                if ctx.answer().first().copied() == Some(ChoiceOption::Confirm) {
                    let hujia = u32::from(ctx.core.player(owner).hujia);
                    if hujia > 0 {
                        ctx.core.player_mut(owner).hujia = 0;
                        set_flag(ctx.core, owner, "extra_turn_insert_queued", 1);
                        set_flag(ctx.core, owner, "u05_msilian_turn_pending", hujia);
                        set_flag(ctx.core, owner, "u05_msilian_damaged", 0);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mduohun" => {
            let src = ctx.res.stack.last().and_then(|f| match f.data {
                EventData::DyingSaved { source, .. } => source,
                _ => None,
            });
            if let Some(s) = src {
                if s != owner && ctx.core.player(s).alive {
                    ctx.emit(EventData::LoseHp {
                        player: s,
                        amount: 1,
                    });
                }
            }
            Flow::Done
        }
        "lingyong" => match m.stage {
            HostStage::Act => {
                let slots = super::u05_b1_mtg_subplayer::active_subplayer_slots(ctx.core, owner);
                if slots.is_empty()
                    || super::u05_b1_mtg_subplayer::has_active_subplayer(ctx.core, owner)
                {
                    return Flow::Done;
                }
                if slots.len() == 1 {
                    super::u05_b1_mtg_subplayer::enter_subplayer(ctx, owner, slots[0]);
                    return Flow::Done;
                }
                wait_ask_mtg(m, ctx, HostStage::SelectNature)
            }
            HostStage::SelectNature => {
                if let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() {
                    super::u05_b1_mtg_subplayer::enter_subplayer(ctx, owner, u32::from(idx));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "subplayer" => {
            super::u05_b1_mtg_subplayer::exit_subplayer_permanent(ctx, owner);
            Flow::Done
        }
        "mhuanyi" => match m.stage {
            HostStage::Act => {
                if let Some(frame_id) = ctx.res.stack.last().map(|f| f.id) {
                    if let Some(EventData::CardUsed(u)) = ctx.res.frame(frame_id).map(|f| &f.data) {
                        if !u.cancelled && u.targets.contains(&owner) {
                            let tgt_v = flag(ctx.core, owner, "u05_mhuanyi_tgt");
                            let cls_v = flag(ctx.core, owner, "u05_mhuanyi_cls");
                            if tgt_v > 0 && cls_v > 0 {
                                let t = (tgt_v - 1) as PlayerId;
                                let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                                let cls_ok = (cls_v == 1 && kind.is_basic_kind())
                                    || (cls_v == 2 && !kind.is_basic_kind());
                                if (t as usize) < ctx.core.players.len()
                                    && t != owner
                                    && t != u.player
                                    && ctx.core.player(t).alive
                                    && cls_ok
                                    && target_enabled2_for_card(
                                        ctx.core,
                                        ctx.catalog,
                                        u.card,
                                        kind,
                                        u.suit(ctx.catalog),
                                        u.player,
                                        t,
                                    )
                                {
                                    if let Some(f) = ctx.res.frame_mut(frame_id) {
                                        if let EventData::CardUsed(ref mut u2) = f.data {
                                            let mut new_targets = Vec::with_capacity(u2.targets.len());
                                            for &old_t in &u2.targets {
                                                let rep = if old_t == owner { t } else { old_t };
                                                if !new_targets.contains(&rep) {
                                                    new_targets.push(rep);
                                                }
                                            }
                                            u2.targets = new_targets;
                                        }
                                    }
                                    set_flag(ctx.core, owner, "u05_mhuanyi_tgt", 0);
                                    set_flag(ctx.core, owner, "u05_mhuanyi_cls", 0);
                                }
                            }
                        }
                        return Flow::Done;
                    }
                }
                wait_ask_mtg(m, ctx, HostStage::TargetChoice)
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                m.target = Some(t);
                wait_ask_mtg(m, ctx, HostStage::SelectNature)
            }
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(cls)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if let Some(t) = m.target {
                    let round = ctx.core.turn.round.max(1);
                    set_flag(ctx.core, owner, "u05_mhuanyi_tgt", u32::from(t) + 1);
                    set_flag(ctx.core, owner, "u05_mhuanyi_cls", u32::from(cls) + 1);
                    set_flag(ctx.core, owner, "u05_mhuanyi_ready_round", round + 2);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "msuoling" => step_msuoling(m, ctx),
        "mtongnian" => step_mtongnian(m, ctx),
        _ => Flow::Done,
    }
}
