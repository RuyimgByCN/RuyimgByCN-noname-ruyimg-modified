//! L11（太虚幻境）侍从全局钩子（`L11ServantHooks` tag 728）与侍从技能 Group B（`L11ServantSkillsB` tag 729）：
//!
//! - `L11ServantHooks`（`extension_servant.js:11, 205, 244, 261`）：`_slEnterGame`、`\_slDamage`、`\_slUseCard`、`\_slHurt`
//! - `L11ServantSkillsB`（`extension.js:2031-2702`）：`txhj_dundunSkill1`、`txhj_datongSkill1`、`txhj_luluSkill1..2`、
//!   `txhj_aleSkill1..2`、`txhj_ruiSkill2..3`、`txhj_yueerSkill1..2`、`txhj_yanSkill1..3`、`txhj_liuliSkill1..2`、
//!   `txhj_manmanSkill1..2`、`txhj_xiaoxiaoSkill1..2`、`txhj_xuerenSkill1`、`txhj_xuanwuSkill1..3`、`txhj_aHaoSkill1..3`

use super::l11_common::*;
use super::l11_common_b2::*;
use super::l11_servants_b_asks::*;
use super::*;
use crate::catalog::{CardKind, EquipSlot, QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::{DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};
use crate::vocabulary::generals::Gender;

fn wait_sb_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.card = Some(ask_id);
    if let Some((observer, spec)) = build_l11_servants_b_ask(
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

fn is_ordinary_trick_use(catalog: &crate::catalog::Catalog, u: &crate::event::CardUseData) -> bool {
    is_ordinary_trick_kind(u.effective_kind(catalog))
}

fn is_trick_use(catalog: &crate::catalog::Catalog, u: &crate::event::CardUseData) -> bool {
    let k = u.effective_kind(catalog);
    !k.is_basic_kind() && k.equip_slot().is_none() && k.is_basic_trick_or_equip()
}

pub(in crate::content::host) fn trigger_meta_l11_servant_hooks(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "_slEnterGame" => Some(meta(EventKind::TurnBegin, Timing::First, 10, Vec::new())),
        "_slDamage" => Some(meta(EventKind::DamageBegin1, Timing::Normal, 1, Vec::new())),
        "_slUseCard" => Some(meta(EventKind::CardUsed, Timing::First, 1, Vec::new())),
        "_slHurt" => Some(meta(
            EventKind::DamageBegin4,
            Timing::Normal,
            1,
            vec![EventKind::LoseHp],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn trigger_meta_l11_servants_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, timing: Timing, priority: i32, extra_events| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "txhj_dundunSkill1" => Some(meta(EventKind::CardUsed, Timing::Normal, 1, Vec::new())),
        "txhj_datongSkill1" => Some(meta(EventKind::Dying, Timing::Normal, 1, Vec::new())),
        "txhj_luluSkill1" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txhj_luluSkill2" => Some(meta(
            EventKind::CardUsed,
            Timing::First,
            5,
            vec![EventKind::CardsDrawn, EventKind::CardsMoved, EventKind::CardsGiven],
        )),
        "txhj_aleSkill1" => Some(meta(
            EventKind::CardUsed,
            Timing::First,
            2,
            vec![EventKind::DamageEnd],
        )),
        "txhj_aleSkill2" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txhj_ruiSkill2" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            1,
            vec![EventKind::TurnEnded],
        )),
        "txhj_ruiSkill3" | "txhj_yanSkill1" | "txhj_xuanwuSkill3" => {
            Some(meta(EventKind::CardUsed, Timing::Normal, 4, Vec::new()))
        }
        "txhj_yueerSkill1" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_yueerSkill2" => Some(meta(EventKind::PhaseEnd, Timing::Normal, 1, Vec::new())),
        "txhj_yanSkill2" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_yanSkill3" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_liuliSkill1" => Some(meta(
            EventKind::CardsLost,
            Timing::Normal,
            1,
            vec![EventKind::CardsDiscarded],
        )),
        "txhj_liuliSkill2" => Some(meta(
            EventKind::PhaseEnd,
            Timing::Normal,
            1,
            vec![EventKind::CardsDiscarded],
        )),
        "txhj_manmanSkill1" => Some(meta(
            EventKind::TurnBegin,
            Timing::Normal,
            1,
            vec![EventKind::DamageEnd],
        )),
        "txhj_manmanSkill2" => Some(meta(EventKind::DamageEnd, Timing::Normal, 1, Vec::new())),
        "txhj_xiaoxiaoSkill1" => Some(meta(
            EventKind::CardUsed,
            Timing::Normal,
            1,
            vec![EventKind::DamageEnd],
        )),
        "txhj_xiaoxiaoSkill2" => Some(meta(EventKind::PhaseBegin, Timing::Normal, 1, Vec::new())),
        "txhj_xuerenSkill1" => Some(meta(
            EventKind::TurnEnded,
            Timing::Normal,
            1,
            vec![EventKind::DamageEnd],
        )),
        "txhj_xuanwuSkill1" => Some(meta(EventKind::Recover, Timing::Normal, 1, Vec::new())),
        "txhj_xuanwuSkill2" => Some(meta(EventKind::CardsDiscarded, Timing::Normal, 1, Vec::new())),
        "txhj_aHaoSkill2" => Some(meta(EventKind::CardsLost, Timing::Normal, 1, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l11_servants_b(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_servants_b(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

pub(in crate::content::host) fn filter_l11_servant_hooks(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "_slEnterGame" => {
            matches!(event, EventData::TurnBegin { .. })
                && flag(ctx.state, owner, "l11_sl_enter_game") == 0
        }
        "_slDamage" => matches!(
            event,
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.source == Some(owner)
        ),
        "_slUseCard" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player == owner
                    && (is_ordinary_trick_use(ctx.catalog, u)
                        || matches!(
                            u.effective_kind(ctx.catalog),
                            CardKind::Tao | CardKind::Jiu
                        ))
        ),
        "_slHurt" => match event {
            EventData::Damage(d) => !d.prevented && d.amount > 0 && d.target == owner,
            EventData::LoseHp { player, amount } => *player == owner && *amount > 0,
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn filter_l11_servants_b_with_frames(
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
        "txhj_dundunSkill1" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player == owner
                    && is_sha_use(ctx.catalog, u)
                    && !u.targets.is_empty()
        ),
        "txhj_datongSkill1" => matches!(
            event,
            EventData::Dying { player, .. }
                if *player == owner
                    && state.player(owner).hp < 1
                    && flag(state, owner, "l11_datong1_used") == 0
        ),
        "txhj_luluSkill1" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true
            } if *player == owner
        ),
        "txhj_luluSkill2" => match event {
            EventData::CardsDrawn { player, cards } if *player == owner && !cards.is_empty() => true,
            EventData::CardsMoved { to, cards, .. } if *to == owner && !cards.is_empty() => true,
            EventData::CardsGiven {
                recipient, cards, ..
            } if *recipient == owner && !cards.is_empty() => true,
            EventData::CardUsed(u) => {
                if u.player != owner || u.cancelled || !is_sha_use(ctx.catalog, u) {
                    return false;
                }
                if flag(state, owner, "skill_turn:l11_lulu2_sha_seen") > 0 {
                    return false;
                }
                let gained_cnt = state
                    .player(owner)
                    .hand
                    .iter()
                    .filter(|c| flag(state, owner, &format!("skill_turn:l11_gained:{c}")) > 0)
                    .count();
                gained_cnt >= state.player(owner).hp.max(0) as usize && !u.targets.is_empty()
            }
            _ => false,
        },
        "txhj_aleSkill1" => match event {
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.target == owner => {
                frames.iter().any(|f| {
                    matches!(f.data, EventData::CardUsed(_))
                        && flag(state, owner, &format!("skill_turn:l11_ale1_armed:{}", f.id)) > 0
                })
            }
            EventData::CardUsed(u) => {
                !u.cancelled
                    && u.targets.contains(&owner)
                    && u.effective_kind(ctx.catalog).is_basic_kind()
                    && flag(state, owner, "skill_turn:l11_ale1_used") == 0
            }
            _ => false,
        },
        "txhj_aleSkill2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true
            } if *player == owner
                && state.player(owner).hp < state.player(owner).max_hp as i32
                && alive_others(state, owner)
                    .into_iter()
                    .any(|p| state.player(p).hp > state.player(owner).hp)
        ),
        "txhj_ruiSkill2" => match event {
            EventData::TurnBegin { player } | EventData::TurnEnded { player } => {
                *player == owner && !state.player(owner).hand.is_empty()
            }
            _ => false,
        },
        "txhj_ruiSkill3" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player != owner
                    && u.targets.contains(&owner)
                    && is_ordinary_trick_use(ctx.catalog, u)
        ),
        "txhj_yanSkill1" => matches!(
            event,
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player != owner
                    && u.targets.len() == 1
                    && u.targets[0] == owner
                    && is_ordinary_trick_use(ctx.catalog, u)
        ),
        "txhj_xuanwuSkill3" => match event {
            EventData::CardUsed(u)
                if !u.cancelled
                    && u.player != owner
                    && u.targets.contains(&owner)
                    && is_trick_use(ctx.catalog, u) =>
            {
                let rank = u
                    .card
                    .map(|c| effective_card_rank(ctx.catalog, c))
                    .unwrap_or(0);
                usize::from(rank) <= state.player(owner).hand.len()
            }
            _ => false,
        },
        "txhj_yueerSkill1" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && state.player(d.target).alive
                    && state.matches_gender(ctx.catalog, d.target, Gender::Male)
                    && flag(state, owner, "skill_round:l11_yueer1_used") == 0
        ),
        "txhj_yueerSkill2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: false
            } if *player == owner
        ),
        "txhj_yanSkill2" => match event {
            EventData::Damage(d) if !d.prevented && d.amount > 0 => {
                if d.target == owner
                    && d.source.is_some_and(|s| s != owner && state.player(s).alive)
                    && flag(state, owner, "skill_turn:l11_yan2_used") == 0
                {
                    return true;
                }
                d.source == Some(owner) && flag(state, owner, "skill_turn:l11_yan2_armed") > 0
            }
            _ => false,
        },
        "txhj_yanSkill3" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && flag(state, owner, "skill_turn:l11_yan3_used") == 0
        ),
        "txhj_liuliSkill1" => match event {
            EventData::CardsLost { player, cards, .. }
            | EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && state.turn.current != owner
                    && state.player(state.turn.current).alive
                    && flag(state, owner, "skill_turn:l11_liuli1_used") == 0
            }
            _ => false,
        },
        "txhj_liuliSkill2" => match event {
            EventData::CardsDiscarded { player, cards } => {
                state.turn.phase == Phase::Discard && *player != owner && !cards.is_empty()
            }
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: false,
            } => {
                *player != owner
                    && flag(state, owner, "skill_turn:l11_liuli2_used") == 0
                    && flag(state, owner, "skill_turn:l11_discard_phase_actor")
                        == u32::from(*player) + 1
                    && !state.discard.is_empty()
            }
            _ => false,
        },
        "txhj_manmanSkill1" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && !alive_others(state, owner).is_empty()
            }
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && damage_effective_kind(state, frames, frame, ctx.catalog, d)
                        == Some(CardKind::SavageAttack)
            }
            _ => false,
        },
        "txhj_manmanSkill2" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && d.target != owner
                    && state.player(d.target).alive
                    && !he_cards(state, d.target).is_empty()
                    && flag(state, owner, "skill_turn:l11_manman2_used") == 0
        ),
        "txhj_xiaoxiaoSkill1" => match event {
            EventData::Damage(d) if !d.prevented && d.amount > 0 => true,
            EventData::CardUsed(u) => {
                !u.cancelled
                    && u.player != owner
                    && u.targets.contains(&owner)
                    && is_trick_use(ctx.catalog, u)
                    && flag(state, owner, "skill_turn:l11_xiaoxiao1_used") == 0
                    && flag(state, owner, "skill_turn:l11_any_damage_seen") == 0
                    && !ctx.state.players.iter().any(|p| p.turn_damage > 0)
            }
            _ => false,
        },
        "txhj_xiaoxiaoSkill2" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true
            } if *player == owner
                && alive_others(state, owner).into_iter().any(|p| {
                    state.player(p).equips.len() <= state.player(owner).equips.len()
                        && !state.player(p).hand.is_empty()
                })
        ),
        "txhj_xuerenSkill1" => match event {
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.source == Some(owner) => true,
            EventData::TurnEnded { player } => {
                *player == owner
                    && (state.player(owner).turn_damage > 0
                        || flag(state, owner, "skill_turn:l11_xueren_dmg") > 0)
            }
            _ => false,
        },
        "txhj_xuanwuSkill1" => matches!(
            event,
            EventData::Recover { player, amount, .. }
                if *player != owner
                    && *amount > 0
                    && state.player(owner).hp < state.player(owner).max_hp as i32
                    && flag(state, 0, "l11_xuanwu1_busy") == 0
        ),
        "txhj_xuanwuSkill2" => matches!(
            event,
            EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && cards.iter().any(|&c| {
                        flag(state, *player, &format!("l11_discard_non_hand:{c}")) == 0
                    })
                    && flag(state, owner, "skill_turn:l11_xuanwu2_used") == 0
        ),
        "txhj_aHaoSkill2" => matches!(
            event,
            EventData::CardsLost { player, origins, .. }
                if *player == owner && origins.contains(&LostOrigin::Equip)
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn machine_l11_servants_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    match event {
        EventData::CardUsed(u) => {
            target = Some(u.player);
            card = Some(1);
            if let Some(c) = u.card {
                cards.push(c);
            }
        }
        EventData::Damage(d) => {
            target = Some(d.target);
            card = Some(0);
            cards.push(d.amount.max(0) as u32);
            if let Some(s) = d.source {
                cards.push(u32::from(s));
            }
        }
        EventData::CardsDrawn { cards: cs, .. }
        | EventData::CardsMoved { cards: cs, .. }
        | EventData::CardsGiven { cards: cs, .. }
        | EventData::CardsDiscarded { cards: cs, .. } => {
            card = Some(0);
            cards.extend_from_slice(cs);
            if let EventData::CardsDiscarded { player, .. } = event {
                target = Some(*player);
            }
        }
        EventData::TurnBegin { player } | EventData::TurnEnded { player } => {
            target = Some(*player);
            card = Some(u32::from(matches!(event, EventData::TurnEnded { .. })));
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

pub(in crate::content::host) fn handle_l11_servant_hooks(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "_slEnterGame" => {
            set_flag(ctx.core, owner, "l11_sl_enter_game", 1);
            set_flag(ctx.core, owner, "l11_sl_desk_ready", 1);
        }
        "_slDamage" => {
            let n = flag(ctx.core, owner, "l11_sl_attack_anim") + 1;
            set_flag(ctx.core, owner, "l11_sl_attack_anim", n);
        }
        "_slUseCard" => {
            let n = flag(ctx.core, owner, "l11_sl_usecard_anim") + 1;
            set_flag(ctx.core, owner, "l11_sl_usecard_anim", n);
        }
        "_slHurt" => {
            let n = flag(ctx.core, owner, "l11_sl_hurt_anim") + 1;
            set_flag(ctx.core, owner, "l11_sl_hurt_anim", n);
        }
        _ => {}
    }
    Flow::Done
}

pub(in crate::content::host) fn handle_l11_servants_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "txhj_dundunSkill1" => {
            let cnt = match current_use_card_mut(ctx) {
                Some(u) => u.targets.len().max(1) as u32,
                None => 1,
            };
            ctx.start_draw(owner, cnt);
            Flow::Done
        }
        "txhj_datongSkill1" => {
            set_flag(ctx.core, owner, "l11_datong1_used", 1);
            let hp = ctx.core.player(owner).hp;
            if hp < 1 {
                ctx.recover(owner, (1 - hp) as u32, None);
            }
            Flow::Done
        }
        "txhj_luluSkill1" => {
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "txhj_luluSkill2" => {
            if m.card == Some(0) {
                for &c in &m.cards {
                    set_flag(ctx.core, owner, &format!("skill_turn:l11_gained:{c}"), 1);
                }
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "skill_turn:l11_lulu2_sha_seen", 1);
            let Some(u) = current_use_card_mut(ctx) else {
                return Flow::Done;
            };
            let cid = u.card.unwrap_or(0);
            let tgts = u.targets.clone();
            for t in tgts {
                set_flag(ctx.core, owner, &format!("sha_unrespondable:{cid}:{t}"), 1);
                let prev = flag(ctx.core, owner, &format!("sha_target_damage_bonus:{cid}:{t}"));
                set_flag(
                    ctx.core,
                    owner,
                    &format!("sha_target_damage_bonus:{cid}:{t}"),
                    prev + 1,
                );
            }
            Flow::Done
        }
        "txhj_aleSkill1" => {
            let cur_is_card = ctx
                .res
                .stack
                .iter()
                .rev()
                .find_map(|f| matches!(f.data, EventData::CardUsed(_)).then_some(f.id));
            if m.card == Some(0) {
                for f in &ctx.res.stack {
                    if matches!(f.data, EventData::CardUsed(_))
                        && flag(ctx.core, owner, &format!("skill_turn:l11_ale1_armed:{}", f.id)) > 0
                    {
                        set_flag(ctx.core, owner, &format!("skill_turn:l11_ale1_hit:{}", f.id), 1);
                    }
                }
                return Flow::Done;
            }
            if let Some(fid) = cur_is_card {
                set_flag(ctx.core, owner, "skill_turn:l11_ale1_used", 1);
                set_flag(ctx.core, owner, &format!("skill_turn:l11_ale1_armed:{fid}"), 1);
            }
            Flow::Done
        }
        "txhj_aleSkill2" => {
            ctx.recover(owner, 1, None);
            Flow::Done
        }
        "txhj_ruiSkill2" => {
            let hand_len = ctx.core.player(owner).hand.len();
            if hand_len == 0 {
                return Flow::Done;
            }
            let friends: Vec<PlayerId> = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| ctx.core.is_friend_of(ctx.catalog, owner, p))
                .collect();
            if hand_len % 2 == 1 && !friends.is_empty() {
                let pick = friends[ctx.core.rng.gen_range(friends.len() as u32) as usize];
                ctx.start_draw(pick, 1);
            } else {
                let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                    .into_iter()
                    .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                    .collect();
                if !enemies.is_empty() {
                    let pick_p = enemies[ctx.core.rng.gen_range(enemies.len() as u32) as usize];
                    let pool = he_cards(ctx.core, pick_p);
                    if !pool.is_empty() {
                        let pick_c = pool[ctx.core.rng.gen_range(pool.len() as u32) as usize];
                        discard_player_cards(ctx, pick_p, &[pick_c]);
                    }
                }
            }
            Flow::Done
        }
        "txhj_ruiSkill3" => step_txhj_rui_skill3(m, ctx),
        "txhj_yanSkill1" => step_txhj_yan_skill1(m, ctx),
        "txhj_xuanwuSkill3" => {
            if let Some(u) = current_use_card_mut(ctx) {
                u.targets.retain(|&p| p != owner);
                if u.targets.is_empty() {
                    u.cancelled = true;
                }
            }
            Flow::Done
        }
        "txhj_yueerSkill1" => {
            set_flag(ctx.core, owner, "skill_round:l11_yueer1_used", 1);
            ctx.recover(owner, 1, None);
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "txhj_yueerSkill2" => {
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        "txhj_yanSkill2" => step_txhj_yan_skill2(m, ctx),
        "txhj_yanSkill3" => step_txhj_yan_skill3(m, ctx),
        "txhj_liuliSkill1" => {
            set_flag(ctx.core, owner, "skill_turn:l11_liuli1_used", 1);
            let tgt = ctx.core.turn.current;
            emit_damage(ctx, Some(owner), tgt, 1, DamageNature::Normal, None);
            Flow::Done
        }
        "txhj_liuliSkill2" => {
            if let (Some(actor), false) = (m.target, m.cards.is_empty()) {
                set_flag(
                    ctx.core,
                    owner,
                    "skill_turn:l11_discard_phase_actor",
                    u32::from(actor) + 1,
                );
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "skill_turn:l11_liuli2_used", 1);
            if let Some(c) = ctx.core.discard.last().copied() {
                gain_card_from_deck_or_discard(ctx, owner, c);
            }
            Flow::Done
        }
        "txhj_manmanSkill1" => step_txhj_manman_skill1(m, ctx),
        "txhj_manmanSkill2" => step_txhj_manman_skill2(m, ctx),
        "txhj_xiaoxiaoSkill1" => step_txhj_xiaoxiao_skill1(m, ctx),
        "txhj_xiaoxiaoSkill2" => {
            let my_eq = ctx.core.player(owner).equips.len();
            for p in alive_others(ctx.core, owner) {
                if ctx.core.player(p).equips.len() <= my_eq && !ctx.core.player(p).hand.is_empty() {
                    let hlen = ctx.core.player(p).hand.len() as u32;
                    let idx = ctx.core.rng.gen_range(hlen) as usize;
                    let cid = ctx.core.player(p).hand[idx];
                    move_cards_to_hand(ctx, p, owner, &[cid]);
                }
            }
            Flow::Done
        }
        "txhj_xuerenSkill1" => {
            if m.card == Some(0) {
                set_flag(ctx.core, owner, "skill_turn:l11_xueren_dmg", 1);
                return Flow::Done;
            }
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        "txhj_xuanwuSkill1" => {
            set_flag(ctx.core, 0, "l11_xuanwu1_busy", 1);
            ctx.recover(owner, 1, None);
            set_flag(ctx.core, 0, "l11_xuanwu1_busy", 0);
            Flow::Done
        }
        "txhj_xuanwuSkill2" => {
            let hs_count = m
                .cards
                .iter()
                .filter(|&&c| flag(ctx.core, owner, &format!("l11_discard_non_hand:{c}")) == 0)
                .count();
            if hs_count == 0 {
                return Flow::Done;
            }
            set_flag(ctx.core, owner, "skill_turn:l11_xuanwu2_used", 1);
            let count = hs_count.div_ceil(2);
            ctx.start_draw(owner, count as u32);
            let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                .take(count)
                .collect();
            for e in enemies {
                emit_damage(ctx, Some(owner), e, 1, DamageNature::Normal, None);
            }
            Flow::Done
        }
        "txhj_aHaoSkill2" => {
            let enemies: Vec<PlayerId> = alive_others(ctx.core, owner)
                .into_iter()
                .filter(|&p| !ctx.core.is_friend_of(ctx.catalog, owner, p))
                .collect();
            for e in enemies {
                emit_damage(ctx, Some(owner), e, 1, DamageNature::Normal, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_rui_skill3(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::TargetChoice;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::TargetChoice => {
            let suit_code = flag(ctx.core, owner, "last_judge_suit");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            let won = matches!(suit_code, 2 | 4);
            if won {
                let gained_cid = current_use_card_mut(ctx).and_then(|u| {
                    u.targets.retain(|&p| p != owner);
                    if u.targets.is_empty() {
                        u.cancelled = true;
                    }
                    u.card
                });
                if let Some(cid) = gained_cid {
                    if ctx.core.table.contains(&cid) {
                        ctx.core.table.retain(|&c| c != cid);
                        ctx.core.player_mut(owner).hand.push(cid);
                        ctx.emit(EventData::CardsDrawn {
                            player: owner,
                            cards: vec![cid],
                        });
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_yan_skill1(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::TargetChoice;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::TargetChoice => {
            let suit_code = flag(ctx.core, owner, "last_judge_suit");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            let won = matches!(suit_code, 1 | 3);
            if !won {
                return Flow::Done;
            }
            let (orig_user, reflect_card, reflect_use_as) = match current_use_card_mut(ctx) {
                Some(u) => {
                    let user = u.player;
                    let card = u.card;
                    let use_as = u.use_as;
                    u.targets.retain(|&p| p != owner);
                    if u.targets.is_empty() {
                        u.cancelled = true;
                    }
                    (user, card, use_as)
                }
                None => return Flow::Done,
            };
            let reflect_kind = match (reflect_use_as, reflect_card) {
                (Some(k), _) => k,
                (None, Some(c)) => ctx.catalog.card_kind(c),
                (None, None) => return Flow::Done,
            };
            let is_jiedao = reflect_card
                .and_then(|c| ctx.catalog.cards.get(c as usize))
                .is_some_and(|cd| matches!(cd.name, "jiedao" | "txhj_jiedao"));
            if !ctx.core.player(orig_user).alive {
                return Flow::Done;
            }
            if is_jiedao && equip_slot_card(ctx.core, ctx.catalog, orig_user, EquipSlot::Weapon).is_none() {
                return Flow::Done;
            }
            let targets = if is_jiedao {
                let Some(victim) = alive_others(ctx.core, orig_user)
                    .into_iter()
                    .find(|&v| in_attack_range(ctx.core, ctx.catalog, orig_user, v))
                else {
                    return Flow::Done;
                };
                vec![orig_user, victim]
            } else {
                vec![orig_user]
            };
            emit_virtual_kind_use(ctx, owner, reflect_kind, targets, DamageNature::Normal);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_yan_skill2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::DrawSettle {
        set_flag(ctx.core, owner, "skill_turn:l11_yan2_armed", 0);
        return Flow::Done;
    }
    let src = m.cards.get(1).map(|&v| v as PlayerId);
    if src == Some(owner) && flag(ctx.core, owner, "skill_turn:l11_yan2_armed") > 0 {
        ctx.recover(owner, 1, None);
        return Flow::Done;
    }
    if let Some(attacker) = src {
        if attacker != owner && ctx.core.player(attacker).alive {
            set_flag(ctx.core, owner, "skill_turn:l11_yan2_used", 1);
            set_flag(ctx.core, owner, "skill_turn:l11_yan2_armed", 1);
            m.stage = HostStage::DrawSettle;
            emit_virtual_kind_use(ctx, owner, CardKind::Sha, vec![attacker], DamageNature::Normal);
            return Flow::Yield;
        }
    }
    Flow::Done
}

fn step_txhj_yan_skill3(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "skill_turn:l11_yan3_used", 1);
            m.stage = HostStage::TargetChoice;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::TargetChoice => {
            let suit_code = flag(ctx.core, owner, "last_judge_suit");
            set_flag(ctx.core, owner, "last_judge_suit", 0);
            set_flag(ctx.core, owner, "last_judge_rank", 0);
            if suit_code == 0 {
                return Flow::Done;
            }
            if matches!(suit_code, 2 | 4) {
                ctx.start_draw(owner, 2);
                Flow::Done
            } else {
                let Some(tgt) = m.target else {
                    return Flow::Done;
                };
                if tgt == owner || !ctx.core.player(tgt).alive || he_cards(ctx.core, tgt).is_empty() {
                    return Flow::Done;
                }
                wait_sb_ask(m, ctx, HostStage::CardsAsk, ASK_YAN3_CHOOSE_CARD)
            }
        }
        HostStage::CardsAsk => {
            if let (Some(tgt), Some(cid)) = (m.target, answered_cards(ctx).into_iter().next()) {
                move_cards_to_hand(ctx, tgt, owner, &[cid]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_manman_skill1(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !m.cards.is_empty() {
        let amt = m.cards[0].max(1);
        ctx.start_draw(owner, amt);
        return Flow::Done;
    }
    let others = alive_others(ctx.core, owner);
    if others.is_empty() {
        return Flow::Done;
    }
    emit_virtual_kind_use(ctx, owner, CardKind::SavageAttack, others, DamageNature::Normal);
    Flow::Done
}

fn step_txhj_manman_skill2(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            set_flag(ctx.core, owner, "skill_turn:l11_manman2_used", 1);
            wait_sb_ask(m, ctx, HostStage::CardsAsk, ASK_MANMAN2_CHOOSE_CARD)
        }
        HostStage::CardsAsk => {
            if let (Some(tgt), Some(cid)) = (m.target, answered_cards(ctx).into_iter().next()) {
                move_cards_to_hand(ctx, tgt, owner, &[cid]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_txhj_xiaoxiao_skill1(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(0) && m.stage == HostStage::Act {
        set_flag(ctx.core, owner, "skill_turn:l11_any_damage_seen", 1);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => wait_sb_ask(m, ctx, HostStage::TargetChoice, ASK_XIAOXIAO1_CONFIRM),
        HostStage::TargetChoice => {
            if is_confirmed(ctx) {
                set_flag(ctx.core, owner, "skill_turn:l11_xiaoxiao1_used", 1);
                let p = ctx.core.player(owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(owner, 1, None);
                } else {
                    ctx.start_draw(owner, 1);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
