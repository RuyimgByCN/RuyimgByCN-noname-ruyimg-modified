//! L11（太虚幻境）武将技能 Group E（`L11GeneralSkillsE` tag 731）：
//!
//! - `txmengtai`、`txshouyi`、`txrenxing`、`txyinli`、`txzhulian`、`txkeji`、`txkeji_add`
//! - `txnangce`、`txduanzui`、`txyingyang`、`mdqianxi7`、`mdqianxi72`、`mdqianxi`
//! - `zylongdan`、`zglkongcheng`、`txjuxiang`、`txdaoji`

use super::l11_common::*;
use super::l11_common_b2::*;
use super::l11_skills_e_asks::*;
use super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

fn wait_e_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_skills_e_ask(
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

fn has_shouyue_active(state: &GameStateCore, catalog: &crate::catalog::Catalog) -> bool {
    state.alive_players().any(|p| {
        has_named_skill(state, catalog, p, "shouyue") || flag(state, p, "shouyue_active") > 0
    })
}

pub(in crate::content::host) fn trigger_meta_l11_skills_e(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "txmengtai" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txrenxing" => Some(meta(
            EventKind::DamageEnd,
            Timing::Normal,
            1,
            vec![EventKind::Recover],
        )),
        "txyinli" => Some(meta(
            EventKind::TurnEnded,
            Timing::Normal,
            1,
            vec![EventKind::CardsDiscarded],
        )),
        "txzhulian" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            1,
            vec![EventKind::Linked, EventKind::DamageBegin1],
        )),
        "txkeji" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txnangce" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            vec![EventKind::CardResponded],
        )),
        "txduanzui" => Some(meta(EventKind::TurnEnded, Timing::Normal, 1, Vec::new())),
        "mdqianxi7" | "mdqianxi" => {
            Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new()))
        }
        "zylongdan" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            vec![EventKind::CardResponded, EventKind::ShaDodged],
        )),
        "txjuxiang" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_skills_e(name: &str) -> SkillLimits {
    match name {
        "txdaoji" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn activate_l11_skills_e(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    if skill.name != "txdaoji" {
        return None;
    }
    let state = ctx.state;
    if state.turn.phase != Phase::Play || !is_unblocked_l11_owner(state, ctx.catalog, owner) {
        return None;
    }
    let has_nonbasic = he_cards(state, owner)
        .into_iter()
        .any(|c| !ctx.catalog.card_kind(c).is_basic_kind());
    let has_tgt = alive_others(state, owner)
        .into_iter()
        .any(|p| !he_cards(state, p).is_empty());
    if has_nonbasic && has_tgt {
        active_skill_machine("txdaoji", HostBehavior::L11GeneralSkillsE, owner)
    } else {
        None
    }
}

pub(in crate::content::host) fn filter_l11_skills_e(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    let state = ctx.state;
    if !state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "txmengtai" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::Play | Phase::End,
                begin: true
            } if *player == owner
        ),
        "txrenxing" => {
            state.turn.current != owner
                && match event {
                    EventData::Damage(d) => !d.prevented && d.amount > 0,
                    EventData::Recover { amount, .. } => *amount > 0,
                    _ => false,
                }
        }
        "txyinli" => match event {
            EventData::CardsDiscarded { cards, .. } => cards
                .iter()
                .any(|&c| ctx.catalog.card_kind(c).is_equip()),
            EventData::TurnEnded { player } => {
                let tag = u32::from(*player) + 1;
                *player != owner
                    && state.discard.iter().any(|&c| {
                        flag(state, owner, &format!("l11_yinli_eq:{c}")) == tag
                    })
            }
            _ => false,
        },
        "txzhulian" => match event {
            EventData::TurnBegin { .. } => !state.player(owner).linked,
            EventData::Linked { player, linked } => *player == owner && !*linked,
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.target == owner
                    && alive_others(state, owner).into_iter().any(|p| {
                        !state.player(p).linked && in_attack_range(state, ctx.catalog, owner, p)
                    })
            }
            _ => false,
        },
        "txkeji" => {
            matches!(
                event,
                EventData::Phase {
                    player,
                    phase: Phase::Discard,
                    begin: true
                } if *player == owner
            ) && flag(state, owner, "skill_turn:l11_play_color_mask").count_ones() <= 1
        }
        "txnangce" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && state.player(state.turn.current).alive
                && match event {
                    EventData::CardUsed(u) => {
                        !u.cancelled
                            && u.player == owner
                            && is_ordinary_trick_kind(u.effective_kind(ctx.catalog))
                    }
                    EventData::CardResponded { player, card } => {
                        *player == owner && ctx.catalog.card_kind(*card) == CardKind::Nullify
                    }
                    _ => false,
                }
        }
        "txduanzui" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(event, EventData::TurnEnded { player } if *player == owner)
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).hand.is_empty())
        }
        "mdqianxi7" | "mdqianxi" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner)
                && matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: Phase::Prepare,
                        begin: true
                    } if *player == owner
                )
        }
        "zylongdan" => {
            is_unblocked_l11_owner(state, ctx.catalog, owner) && match event {
                EventData::CardResponded { player, card } => {
                    if *player != owner {
                        return false;
                    }
                    let ev_vas = flag(state, owner, "event_view_as_skill");
                    let shouyue = has_shouyue_active(state, ctx.catalog);
                    let raw_kind = ctx.catalog.card_kind(*card);
                    if raw_kind == CardKind::Shan && ev_vas == 1_926_900_051 {
                        return shouyue;
                    }
                    if raw_kind.is_sha_like() && ev_vas == 1_926_900_052 {
                        let sha_src_enc = flag(state, owner, "responding_to_sha_from");
                        let has_recover_tgt = sha_src_enc > 0 && {
                            let exc = (sha_src_enc - 1) as PlayerId;
                            alive_others(state, owner).into_iter().any(|p| {
                                p != exc && state.player(p).hp < state.player(p).max_hp as i32
                            })
                        };
                        return shouyue || has_recover_tgt;
                    }
                    false
                }
                EventData::CardUsed(u) => {
                    u.player == owner
                        && !u.cancelled
                        && u.card.is_some_and(|c| {
                            let raw = ctx.catalog.card_kind(c);
                            let eff = u.effective_kind(ctx.catalog);
                            (raw == CardKind::Shan && eff.is_sha_like())
                                || (raw.is_sha_like()
                                    && eff == CardKind::Shan
                                    && has_shouyue_active(state, ctx.catalog))
                        })
                }
                EventData::ShaDodged {
                    source,
                    target,
                    card,
                } => {
                    *source == owner
                        && card.is_some_and(|c| {
                            ctx.catalog.card_kind(c) == CardKind::Shan
                                || flag(state, owner, &format!("skill_turn:l11_longdan_sha:{c}"))
                                    > 0
                        })
                        && alive_others(state, owner).into_iter().any(|p| p != *target)
                }
                _ => false,
            }
        }
        "txjuxiang" => {
            matches!(
                event,
                EventData::Phase {
                    player,
                    phase: Phase::End,
                    begin: true
                } if *player == owner
            ) && flag(state, owner, "skill_turn:l11_used_nanman") == 0
                && flag(state, 0, GLOBAL_JUXIANG_POOL) != 3
        }
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_skills_e(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    match event {
        EventData::Phase { phase, .. } => {
            card = Some(match phase {
                Phase::Prepare => 0,
                Phase::Play => 2,
                Phase::End => 4,
                _ => 1,
            });
        }
        EventData::CardsDiscarded { cards: cs, .. } => {
            card = Some(0);
            cards.extend_from_slice(cs);
        }
        EventData::TurnEnded { player } => {
            target = Some(*player);
            card = Some(1);
        }
        EventData::CardUsed(u) => {
            card = Some(1);
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        EventData::CardResponded { card: c, .. } => {
            card = Some(2);
            cards.push(*c);
        }
        EventData::ShaDodged {
            target: tgt,
            card: c,
            ..
        } => {
            target = Some(*tgt);
            card = Some(3);
            if let Some(cid) = c {
                cards.push(*cid);
            }
        }
        EventData::Damage(_) => card = Some(2),
        EventData::Linked { .. } => card = Some(1),
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

pub(in crate::content::host) fn handle_l11_skills_e(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txmengtai" => {
            match m.card {
                Some(0) => {
                    set_flag(ctx.core, owner, "skill_turn:l11_mengtai_no_draw", 1);
                    ctx.core.player_mut(owner).skip_discard_phase = true;
                }
                Some(2) => {
                    ctx.core.player_mut(owner).skip_discard_phase = false;
                }
                Some(4) => {
                    if flag(ctx.core, owner, "skill_turn:l11_mengtai_no_draw") > 0 {
                        set_flag(ctx.core, owner, "skill_turn:l11_mengtai_no_draw", 0);
                        ctx.start_draw(owner, 3);
                    }
                }
                _ => {}
            }
            Flow::Done
        }
        "txrenxing" => {
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "txyinli" => step_txyinli(m, ctx),
        "txzhulian" => {
            if matches!(m.card, Some(2)) {
                let to_link: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| {
                        !ctx.core.player(p).linked
                            && in_attack_range(ctx.core, ctx.catalog, owner, p)
                    })
                    .collect();
                for p in to_link {
                    ctx.emit(EventData::Linked {
                        player: p,
                        linked: true,
                    });
                }
            } else if !ctx.core.player(owner).linked {
                ctx.emit(EventData::Linked {
                    player: owner,
                    linked: true,
                });
            }
            Flow::Done
        }
        "txkeji" => {
            if let Some(def) = ctx.catalog.skill_by_name("txkeji_add") {
                ctx.add_skill_until_turn_end(owner, def);
            }
            Flow::Done
        }
        "txnangce" => match m.stage {
            HostStage::Act => wait_e_ask(m, ctx, HostStage::TargetChoice, ASK_TXNANGCE_CONFIRM),
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    let cur = ctx.core.turn.current;
                    if ctx.core.player(cur).alive {
                        ctx.start_draw(cur, 1);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "txduanzui" => match m.stage {
            HostStage::Act => wait_e_ask(m, ctx, HostStage::TargetChoice, ASK_TXDUANZUI_CONFIRM),
            HostStage::TargetChoice => {
                if is_confirmed(ctx) {
                    for p in alive_others(ctx.core, owner) {
                        let hs = ctx.core.player(p).hand.len();
                        if hs > 0 && hs < 3 {
                            emit_damage(ctx, Some(owner), p, (3 - hs) as u32, DamageNature::Fire, None);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "mdqianxi7" => step_mdqianxi7(m, ctx),
        "mdqianxi" => step_mdqianxi(m, ctx),
        "zylongdan" => step_zylongdan(m, ctx),
        "txjuxiang" => step_txjuxiang(m, ctx),
        "txdaoji" => step_txdaoji(m, ctx),
        _ => Flow::Done,
    }
}

fn step_txyinli(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.card == Some(0) {
        // `TurnEnded::Commit` runs `turn_handoff` before `TurnEnded::AfterTriggers`,
        // so `turn_stamp` advances before `TurnEnded` triggers fire. Key each equip by
        // `turn.current + 1` and clear stale entries whenever `l11_yinli_ts` rotates.
        let ts = turn_stamp(ctx.core);
        if flag(ctx.core, owner, "l11_yinli_ts") != ts {
            let stale: Vec<String> = ctx
                .core
                .vm_aux
                .flags
                .keys()
                .filter(|(p, k)| *p == owner && k.starts_with("l11_yinli_eq:"))
                .map(|(_, k)| k.clone())
                .collect();
            for k in stale {
                set_flag(ctx.core, owner, &k, 0);
            }
            set_flag(ctx.core, owner, "l11_yinli_ts", ts);
        }
        let actor_tag = u32::from(ctx.core.turn.current) + 1;
        for &c in &m.cards {
            if ctx.catalog.card_kind(c).is_equip() {
                set_flag(ctx.core, owner, &format!("l11_yinli_eq:{c}"), actor_tag);
            }
        }
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_e_ask(m, ctx, HostStage::CardsAsk, ASK_TXYINLI_CARDS),
        HostStage::CardsAsk => {
            let picked = answered_cards(ctx);
            let mut gained = Vec::new();
            for c in picked {
                if let Some(pos) = ctx.core.discard.iter().position(|&x| x == c) {
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
            let clear_keys: Vec<String> = ctx
                .core
                .vm_aux
                .flags
                .keys()
                .filter(|(p, k)| *p == owner && k.starts_with("l11_yinli_eq:"))
                .map(|(_, k)| k.clone())
                .collect();
            for k in clear_keys {
                set_flag(ctx.core, owner, &k, 0);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_mdqianxi7(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_e_ask(m, ctx, HostStage::TargetChoice, ASK_MDQIANXI7_CONFIRM),
        HostStage::TargetChoice => {
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 1);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if he_cards(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_e_ask(m, ctx, HostStage::CardsAsk, ASK_MDQIANXI7_DISCARD)
        }
        HostStage::CardsAsk => {
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let color = find_red_or_black_color_val(ctx.catalog, ctx.core, owner, cid);
            discard_player_cards(ctx, owner, &[cid]);
            m.cards = vec![color];
            wait_e_ask(m, ctx, HostStage::SelectNature, ASK_MDQIANXI7_TARGET)
        }
        HostStage::SelectNature => {
            if let Some(tgt) = answered_players(ctx).into_iter().next() {
                let color = m.cards.first().copied().unwrap_or(1);
                set_flag(ctx.core, tgt, "l11_mdqianxi72_color", color);
                if let Some(def) = ctx.catalog.skill_by_name("mdqianxi72") {
                    ctx.add_skill_until_turn_end(tgt, def);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_mdqianxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_e_ask(m, ctx, HostStage::TargetChoice, ASK_MDQIANXI_CONFIRM),
        HostStage::TargetChoice => {
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            m.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let suit_code = flag(ctx.core, owner, "last_judge_suit");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            if suit_code == 0 {
                return Flow::Done;
            }
            let color = if matches!(suit_code, 2 | 4) {
                1
            } else {
                2
            };
            m.cards = vec![color];
            wait_e_ask(m, ctx, HostStage::SelectNature, ASK_MDQIANXI_TARGET)
        }
        HostStage::SelectNature => {
            if let Some(tgt) = answered_players(ctx).into_iter().next() {
                let color = m.cards.first().copied().unwrap_or(1);
                set_flag(ctx.core, tgt, "l11_mdqianxi72_color", color);
                ensure_perm_skill(ctx, tgt, "mdqianxi72");
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_zylongdan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(3) {
                // `ShaDodged`
                let sha_tgt = m.target.unwrap_or(owner);
                m.cards = vec![0, u32::from(sha_tgt)];
                return wait_e_ask(m, ctx, HostStage::TargetChoice, ASK_LONGDAN_DAMAGE);
            }
            let cid = m.cards.first().copied();
            if has_shouyue_active(ctx.core, ctx.catalog) {
                ctx.start_draw(owner, 1);
            }
            let as_sha = (m.card == Some(1) || m.card == Some(2))
                && cid.is_some_and(|c| ctx.catalog.card_kind(c) == CardKind::Shan);
            if as_sha {
                if m.card == Some(1) {
                    if let Some(c) = cid {
                        set_flag(ctx.core, owner, &format!("skill_turn:l11_longdan_sha:{c}"), 1);
                    }
                }
                return Flow::Done;
            }
            let as_shan = (m.card == Some(2) || m.card == Some(1))
                && cid.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like());
            if as_shan {
                let raw_src = flag(ctx.core, owner, "responding_to_sha_from");
                set_flag(ctx.core, owner, "responding_to_sha_from", 0);
                if raw_src > 0 {
                    let sha_src = (raw_src - 1) as PlayerId;
                    m.cards = vec![0, u32::from(sha_src)];
                    return wait_e_ask(m, ctx, HostStage::SelectNature, ASK_LONGDAN_RECOVER);
                }
            }
            Flow::Done
        }
        HostStage::TargetChoice => {
            if let Some(tgt) = answered_players(ctx).into_iter().next() {
                emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Normal, None);
            }
            Flow::Done
        }
        HostStage::SelectNature => {
            if let Some(tgt) = answered_players(ctx).into_iter().next() {
                ctx.recover(tgt, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txjuxiang(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.stage {
        HostStage::Act => wait_e_ask(m, ctx, HostStage::TargetChoice, ASK_TXJUXIANG_TARGET),
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let mask = flag(ctx.core, 0, GLOBAL_JUXIANG_POOL);
            if mask == 3 {
                return Flow::Done;
            }
            if let Some((pos, bit)) =
                find_txjuxiang_pool_card(ctx.catalog, &ctx.core.deck, mask)
            {
                let cid = ctx.core.deck.remove(pos);
                set_flag(ctx.core, 0, GLOBAL_JUXIANG_POOL, mask | bit);
                deliver_direct_cards_to_hand(ctx, tgt, &[cid]);
            } else if let Some((pos, bit)) =
                find_txjuxiang_pool_card(ctx.catalog, &ctx.core.discard, mask)
            {
                let cid = ctx.core.discard.remove(pos);
                set_flag(ctx.core, 0, GLOBAL_JUXIANG_POOL, mask | bit);
                deliver_direct_cards_to_hand(ctx, tgt, &[cid]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txdaoji(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_e_ask(m, ctx, HostStage::SelectCard, ASK_TXDAOJI_DISCARD),
        HostStage::SelectCard => {
            let Some(cid) = answered_cards(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "txdaoji");
                return Flow::Done;
            };
            m.cards = vec![cid];
            wait_e_ask(m, ctx, HostStage::TargetChoice, ASK_TXDAOJI_TARGET)
        }
        HostStage::TargetChoice => {
            let Some(tgt) = answered_players(ctx).into_iter().next() else {
                refund_once_used(ctx, owner, "txdaoji");
                return Flow::Done;
            };
            if let Some(&pay) = m.cards.first() {
                discard_player_cards(ctx, owner, &[pay]);
            }
            m.target = Some(tgt);
            wait_e_ask(m, ctx, HostStage::CardsAsk, ASK_TXDAOJI_HE)
        }
        HostStage::CardsAsk => {
            let (Some(tgt), Some(stolen)) = (m.target, answered_cards(ctx).into_iter().next())
            else {
                return Flow::Done;
            };
            move_cards_to_hand(ctx, tgt, owner, &[stolen]);
            if ctx.core.player(owner).hand.contains(&stolen) {
                let kind = ctx.catalog.card_kind(stolen);
                if kind.is_basic_kind() {
                    ctx.start_draw(owner, 1);
                } else if kind.is_equip() {
                    m.target = Some(tgt);
                    m.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::Equipped {
                        player: owner,
                        card: stolen,
                        replaced: None,
                    });
                    return Flow::Yield;
                }
            }
            Flow::Done
        }
        HostStage::DrawSettle => {
            if let Some(tgt) = m.target {
                if ctx.core.player(tgt).alive {
                    emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Normal, None);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
