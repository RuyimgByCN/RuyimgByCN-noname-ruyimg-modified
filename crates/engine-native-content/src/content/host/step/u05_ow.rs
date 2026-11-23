//! U05-3 (`shaai-ow`) 守望先锋 109 项宿主技分派与共享原语。

use super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{DamageData, EventData, EventKind, Frame, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::{GameStateCore, SkillInstance};

#[path = "u05_ow_a.rs"]
pub(super) mod u05_ow_a;
#[path = "u05_ow_asks.rs"]
pub(super) mod u05_ow_asks;
#[path = "u05_ow_b.rs"]
pub(super) mod u05_ow_b;
#[path = "u05_ow_c.rs"]
pub(super) mod u05_ow_c;
#[path = "u05_ow_d.rs"]
pub(super) mod u05_ow_d;
#[path = "u05_ow_hooks.rs"]
pub(super) mod u05_ow_hooks;

pub(in crate::content::host) use u05_ow_asks::*;
pub(crate) use u05_ow_hooks::{
    ensure_ow_player_inited, host_before_facedown_turn_skip_ow as host_before_facedown_turn_skip,
    host_card_use_after_ow, host_draw_begin_step_ow, host_on_game_start_ow,
    host_on_player_died_ow, host_on_shanxian_turnover_ow as host_on_shanxian_turnover,
    host_sha_begin_step_ow, on_skill_removed_ow,
};

pub(crate) fn ow_enabled_in_catalog(catalog: &Catalog) -> bool {
    catalog.skill_by_name("woliu").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior() == Some(HostBehavior::U05OwBatchA)
    }) || catalog.skill_by_name("zhongdun").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior() == Some(HostBehavior::U05OwBatchB)
    }) || catalog.skill_by_name("jijia").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior() == Some(HostBehavior::U05OwBatchC)
    }) || catalog.skill_by_name("xie").is_some_and(|d| {
        catalog.skills[d as usize].behavior.as_host_behavior() == Some(HostBehavior::U05OwBatchD)
    })
}

pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn has_skill_u05(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog.skill_by_name(name).is_some_and(|d| {
        matches!(
            catalog.skills[d as usize].behavior.as_host_behavior(),
            Some(
                HostBehavior::U05OwBatchA
                    | HostBehavior::U05OwBatchB
                    | HostBehavior::U05OwBatchC
                    | HostBehavior::U05OwBatchD
            )
        ) && core.has_skill(p, d)
    })
}

pub(super) fn grant_skill_u05(ctx: &mut Ctx<'_>, target: PlayerId, name: &str, temp: bool) {
    let Some(def) = ctx.catalog.skill_by_name(name) else {
        return;
    };
    ctx.insert_skill(
        (target, def),
        SkillInstance {
            owner: target,
            enabled: true,
            once_used: false,
            expires_end_of_turn: temp,
            rounds: 0,
        },
    );
}

pub(super) fn grant_temp_skill(ctx: &mut Ctx<'_>, target: PlayerId, name: &str) {
    grant_skill_u05(ctx, target, name, true);
}

pub(super) fn remove_skill_u05(ctx: &mut Ctx<'_>, target: PlayerId, name: &str) {
    let Some(def) = ctx.catalog.skill_by_name(name) else {
        return;
    };
    if ctx.core.skills.remove(&(target, def)).is_some() {
        let reclaimed = (ctx.catalog.hooks.on_skill_removed)(ctx.core, ctx.catalog, target, name);
        for c in reclaimed {
            ctx.to_discard(c);
        }
    }
}

pub(super) fn alive_others(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(core.next_alive(owner))
        .into_iter()
        .filter(|p| *p != owner && core.player(*p).alive)
        .collect()
}

pub(super) fn hand_and_equips(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let pl = core.player(p);
    pl.hand.iter().copied().chain(pl.equips.iter().copied()).collect()
}

pub(super) fn suit_of(core: &GameStateCore, catalog: &Catalog, p: PlayerId, c: CardId) -> Suit {
    catalog.effective_card_suit(core, p, c)
}

pub(super) fn is_red_card(core: &GameStateCore, catalog: &Catalog, p: PlayerId, c: CardId) -> bool {
    matches!(suit_of(core, catalog, p, c), Suit::Heart | Suit::Diamond)
}

pub(super) fn is_black_card(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    c: CardId,
) -> bool {
    matches!(suit_of(core, catalog, p, c), Suit::Spade | Suit::Club)
}

pub(super) fn discard_he_card(ctx: &mut Ctx<'_>, p: PlayerId, c: CardId) {
    ctx.take_owned_card(p, c);
    ctx.to_discard(c);
    ctx.log(LogEvent::CardDiscarded {
        player: p,
        cards: vec![c],
    });
}

pub(super) fn random_discard_he(ctx: &mut Ctx<'_>, p: PlayerId) -> Option<CardId> {
    let he = hand_and_equips(ctx.core, p);
    if he.is_empty() {
        return None;
    }
    let idx = ctx.core.rng.gen_range(he.len() as u32) as usize;
    let c = he[idx];
    discard_he_card(ctx, p, c);
    Some(c)
}

pub(super) fn draw_direct(ctx: &mut Ctx<'_>, p: PlayerId, count: u32) {
    if count > 0 && ctx.core.player(p).alive {
        ctx.start_draw(p, count);
    }
}

pub(super) fn turn_over_to_back(ctx: &mut Ctx<'_>, p: PlayerId) {
    if ctx.core.player(p).alive && !ctx.core.player(p).face_down {
        ctx.emit(EventData::TurnOverBefore {
            player: p,
            cancelled: false,
        });
    }
}

pub(super) fn toggle_turn_over(ctx: &mut Ctx<'_>, p: PlayerId) {
    if ctx.core.player(p).alive {
        ctx.emit(EventData::TurnOverBefore {
            player: p,
            cancelled: false,
        });
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, owner: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn claim_off_deck_supply(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    kind: CardKind,
    prefix: &str,
    random_pick: bool,
) -> Option<CardId> {
    if !ctx.core.player(owner).alive {
        return None;
    }
    let det_matches: Vec<usize> = ctx
        .core
        .detached_cards
        .iter()
        .enumerate()
        .filter(|(_, (cid, _))| {
            ctx.catalog.card_kind(*cid) == kind
                && ctx.catalog.cards[*cid as usize].name.starts_with(prefix)
        })
        .map(|(i, _)| i)
        .collect();
    let cid = if !det_matches.is_empty() {
        let pick = if random_pick {
            det_matches[ctx.core.rng.gen_range(det_matches.len() as u32) as usize]
        } else {
            det_matches[0]
        };
        ctx.core.detached_cards.remove(pick).0
    } else {
        let disc_matches: Vec<usize> = ctx
            .core
            .discard
            .iter()
            .enumerate()
            .filter(|(_, cid)| {
                ctx.catalog.card_kind(**cid) == kind
                    && ctx.catalog.cards[**cid as usize].name.starts_with(prefix)
            })
            .map(|(i, _)| i)
            .collect();
        if !disc_matches.is_empty() {
            let pick = if random_pick {
                disc_matches[ctx.core.rng.gen_range(disc_matches.len() as u32) as usize]
            } else {
                disc_matches[0]
            };
            ctx.core.discard.remove(pick)
        } else if kind == CardKind::Sha {
            let deck_idx = ctx
                .core
                .deck
                .iter()
                .position(|cid| ctx.catalog.card_kind(*cid) == CardKind::Sha);
            if let Some(i) = deck_idx {
                ctx.core.deck.remove(i)
            } else if let Some(i) = ctx
                .core
                .discard
                .iter()
                .position(|cid| ctx.catalog.card_kind(*cid) == CardKind::Sha)
            {
                ctx.core.discard.remove(i)
            } else {
                return None;
            }
        } else {
            return None;
        }
    };
    ctx.core.player_mut(owner).hand.push(cid);
    // 上游 `game.createCard(...); player.gain(..., "gain2")` 走非摸牌得牌通道（不发 CardsDrawn、不触发连营/摸牌钩子）
    ctx.log(LogEvent::CardsTaken {
        from: owner,
        to: owner,
        cards: vec![cid],
        public: true,
    });
    ctx.emit(EventData::CardsMoved {
        cards: vec![cid],
        from: owner,
        to: owner,
    });
    Some(cid)
}

pub(super) fn with_current_damage_mut<F: FnOnce(&mut DamageData)>(ctx: &mut Ctx<'_>, f: F) {
    let fid = ctx.current_event_frame_id();
    let root_id = match ctx.res.frame(fid).map(|fr| &fr.data) {
        Some(EventData::Damage(_)) => Some(fid),
        Some(EventData::DamageWindow { root, .. }) => Some(*root),
        _ => None,
    };
    if let Some(rid) = root_id {
        if let Some(frame) = ctx.res.frame_mut(rid) {
            if let EventData::Damage(dm) = &mut frame.data {
                f(dm);
            }
        }
    }
}

pub(super) fn with_current_card_use_mut<F: FnOnce(&mut crate::event::CardUseData)>(
    ctx: &mut Ctx<'_>,
    f: F,
) {
    let fid = ctx.current_event_frame_id();
    if let Some(frame) = ctx.res.frame_mut(fid) {
        if let EventData::CardUsed(u) = &mut frame.data {
            f(u);
        }
    }
}

pub(super) fn damage_is_sha_like(
    frames: &[crate::event::Frame],
    frame: &crate::event::Frame,
    catalog: &Catalog,
    d: &DamageData,
) -> bool {
    let root = match frame.data {
        EventData::DamageWindow { root, .. } => frames.iter().find(|f| f.id == root),
        EventData::Damage(_) => Some(frame),
        _ => None,
    };
    root.and_then(|r| crate::event::damage_card_kind(frames, r, catalog))
        .or_else(|| d.card.map(|c| catalog.card_kind(c)))
        .is_some_and(|k| k.is_sha_like())
}

pub(super) fn damage_card_suit(
    frames: &[crate::event::Frame],
    frame: &crate::event::Frame,
    core: &GameStateCore,
    catalog: &Catalog,
    d: &DamageData,
) -> Option<Suit> {
    let root = match frame.data {
        EventData::DamageWindow { root, .. } => frames.iter().find(|f| f.id == root),
        EventData::Damage(_) => Some(frame),
        _ => None,
    };
    let parent_use = root.and_then(|r| {
        r.parent
            .and_then(|p| frames.iter().find(|f| f.id == p))
    });
    if let Some(pf) = parent_use {
        if let EventData::CardUsed(u) = &pf.data {
            if core
                .vm_aux
                .flags
                .get(&(u.player, format!("ow_card_use_suit_spade:{}", pf.id)))
                .copied()
                .unwrap_or(0)
                > 0
            {
                return Some(Suit::Spade);
            }
        }
    }
    if let Some(cid) = d.card {
        let p = d.source.unwrap_or(d.target);
        return Some(suit_of(core, catalog, p, cid));
    }
    parent_use.and_then(|pf| match &pf.data {
        EventData::CardUsed(u) => u.suit(catalog),
        _ => None,
    })
}

pub(super) fn prev_next_targets_clear_of(
    core: &GameStateCore,
    catalog: &Catalog,
    t: PlayerId,
) -> bool {
    let prev = core
        .seat_order_from(core.next_alive(t))
        .into_iter()
        .last()
        .unwrap_or(t);
    let next = core.next_alive(t);
    [t, prev, next].into_iter().all(|p| {
        !has_skill_u05(core, catalog, p, "bingqiang2")
            && !has_skill_u05(core, catalog, p, "bingqiang5")
    })
}

pub(super) fn bingqiang_valid_targets(core: &GameStateCore, catalog: &Catalog) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|p| prev_next_targets_clear_of(core, catalog, *p))
        .collect()
}

pub(super) fn clear_bingqiang_by_owner(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let seats = ctx.core.players.len() as PlayerId;
    for p in 0..seats {
        if flag(ctx.core, p, "ow_bingqiang2_owner") == u32::from(owner) + 1 {
            set_flag(ctx.core, p, "ow_bingqiang2", 0);
            set_flag(ctx.core, p, "ow_bingqiang2_owner", 0);
            remove_skill_u05(ctx, p, "bingqiang2");
        }
        if flag(ctx.core, p, "ow_bingqiang5_owner") == u32::from(owner) + 1 {
            set_flag(ctx.core, p, "ow_bingqiang5", 0);
            set_flag(ctx.core, p, "ow_bingqiang5_owner", 0);
            remove_skill_u05(ctx, p, "bingqiang5");
        }
    }
}

pub(super) fn qinru_valid_card_suit(s: Suit) -> bool {
    s != Suit::Heart
}

pub(super) fn valid_single_sha_target(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    t: PlayerId,
) -> bool {
    if t == owner || !core.player(t).alive {
        return false;
    }
    if catalog
        .skill_by_name(crate::content::standard::KONG_CHENG)
        .is_some_and(|d| core.has_skill(t, d))
        && core.player(t).hand.is_empty()
    {
        return false;
    }
    if core.player(t).jidong_active
        || core.player(t).yinshen_active
        || core.player(t).jileng_active
        || core.player(t).yinmi_active
        || has_skill_u05(core, catalog, t, "sheng2")
        || has_skill_u05(core, catalog, t, "yihun2")
    {
        return false;
    }
    true
}

pub(super) fn mujing_sha_usable(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    if catalog.blocks_card_use_and_save(core, owner)
        || has_skill_u05(core, catalog, owner, "shouhu")
        || has_skill_u05(core, catalog, owner, "guangshu_club")
    {
        return false;
    }
    let paoxiao = !catalog.is_rechanyuan_sealed_name(core, owner, crate::content::standard::PAO_XIAO)
        && catalog
            .skill_by_name(crate::content::standard::PAO_XIAO)
            .is_some_and(|d| core.has_skill(owner, d));
    let zhuge = !crate::kernel::mbcmfangzhu_baiban(core, owner)
        && core
            .player(owner)
            .equips
            .iter()
            .any(|c| catalog.card_kind(*c) == CardKind::Zhuge);
    let baolei = has_skill_u05(core, catalog, owner, "bshaowei")
        && core.player(owner).baolei_mode == 1;
    let zhanlong2 = has_skill_u05(core, catalog, owner, "zhanlong2");
    let land = core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used;
    paoxiao || zhuge || baolei || zhanlong2 || !core.turn.sha_used_play_phase || land
}

pub(super) fn mujing_valid_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    if !mujing_sha_usable(core, catalog, owner) {
        return Vec::new();
    }
    let seats = core.players.len() as u8;
    let chaofeng = (0..seats).find(|&p| {
        p != owner && core.player(p).alive && core.player(p).chaofeng_active && {
            let d = (u16::from(p) + u16::from(seats) - u16::from(owner)) % u16::from(seats);
            d == 1 || d == u16::from(seats) - 1
        }
    });
    let baolei_free =
        has_skill_u05(core, catalog, owner, "bshaowei") && core.player(owner).baolei_mode == 1;
    let feiren_free = has_skill_u05(core, catalog, owner, "feiren");
    let feidan = has_skill_u05(core, catalog, owner, "feidan");
    let sx_flag = flag(core, owner, "ow_shanxian_target");
    let rng = core.attack_range(catalog, owner);
    alive_others(core, owner)
        .into_iter()
        .filter(|&t| {
            if !valid_single_sha_target(core, catalog, owner, t) {
                return false;
            }
            if let Some(cf) = chaofeng {
                if t != cf {
                    return false;
                }
            }
            let dist = core.computed_distance(catalog, owner, t);
            if feidan && dist <= 1 {
                return false;
            }
            let sx_free = sx_flag == u32::from(t) + 1;
            baolei_free || feiren_free || feidan || sx_free || dist <= rng
        })
        .collect()
}

pub(super) fn mujing_max_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
) -> usize {
    let fangtian = !crate::kernel::mbcmfangzhu_baiban(core, owner)
        && core.player(owner).hand.len() == 1
        && core.player(owner).hand.contains(&cid)
        && core
            .player(owner)
            .equips
            .iter()
            .any(|c| catalog.card_kind(*c) == CardKind::Fangtian);
    let feiren_club = has_skill_u05(core, catalog, owner, "feiren")
        && suit_of(core, catalog, owner, cid) == Suit::Club;
    (if fangtian { 3 } else { 1 }) + usize::from(feiren_club)
}

pub(super) fn wait_ow(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    next_stage: HostStage,
    ask_id: u32,
) -> Flow {
    m.stage = next_stage;
    m.card = Some(ask_id);
    let exp = build_ow_expected_ask(
        ask_id,
        m.owner,
        m.target,
        &m.targets,
        &m.cards,
        ctx.core,
        ctx.catalog,
    );
    Flow::WaitChoice {
        observer: exp.observer,
        spec: crate::choice::ChoiceSpec {
            kind: exp.kind,
            prompt: exp.prompt,
            options: exp.options,
            min: exp.min,
            max: exp.max,
            allow_cancel: exp.allow_cancel,
        },
    }
}

pub(super) fn answered_cards(ctx: &Ctx<'_>) -> Vec<CardId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            crate::choice::ChoiceOption::Card(c) => Some(*c),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_players(ctx: &Ctx<'_>) -> Vec<PlayerId> {
    ctx.answer()
        .iter()
        .filter_map(|o| match o {
            crate::choice::ChoiceOption::Player(p) => Some(*p),
            _ => None,
        })
        .collect()
}

pub(super) fn answered_confirm(ctx: &Ctx<'_>) -> bool {
    matches!(
        ctx.answer().first(),
        Some(crate::choice::ChoiceOption::Confirm)
    )
}

pub(super) fn answered_tag(ctx: &Ctx<'_>) -> Option<u8> {
    ctx.answer().iter().find_map(|o| match o {
        crate::choice::ChoiceOption::Tag(t) => Some(*t),
        _ => None,
    })
}

pub(in crate::content::host) fn trigger_meta_u05_ow(
    behavior: HostBehavior,
    name: &str,
) -> Option<TriggerMeta> {
    match behavior {
        HostBehavior::U05OwBatchA => u05_ow_a::trigger_meta_a(name),
        HostBehavior::U05OwBatchB => u05_ow_b::trigger_meta_b(name),
        HostBehavior::U05OwBatchC => u05_ow_c::trigger_meta_c(name),
        HostBehavior::U05OwBatchD => u05_ow_d::trigger_meta_d(name),
        _ => None,
    }
}

pub(in crate::content::host) fn window_trigger_meta_u05_ow(
    behavior: HostBehavior,
    name: &str,
    kind: EventKind,
) -> Option<TriggerMeta> {
    let forced = |event: EventKind, timing: Timing, priority: i32| TriggerMeta {
        event,
        timing,
        priority,
        order_selectable: false,
        optional: false,
        extra_events: Vec::new(),
    };
    match (behavior, name, kind) {
        (HostBehavior::U05OwBatchA, "woliu", EventKind::TurnPhaseEnd) => {
            Some(forced(EventKind::TurnPhaseEnd, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchA, "woliu", EventKind::TurnBegin)
        | (HostBehavior::U05OwBatchA, "woliu", EventKind::Death) => {
            Some(forced(kind, Timing::Normal, 2))
        }
        (HostBehavior::U05OwBatchA, "pingzhang", EventKind::DamageBefore) => {
            Some(forced(EventKind::DamageBefore, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchA, "pingzhang", EventKind::TurnBegin) => {
            Some(forced(EventKind::TurnBegin, Timing::Normal, 2))
        }
        (HostBehavior::U05OwBatchB, "bfengshi", EventKind::DamageBegin) => {
            Some(forced(EventKind::DamageBegin, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchB, "bfengshi", EventKind::CardUsed)
        | (HostBehavior::U05OwBatchB, "bfengshi3", EventKind::CardUsed) => {
            Some(forced(EventKind::CardUsed, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchB, "bingqiang" | "bingqiang_remove", EventKind::TurnBegin)
        | (HostBehavior::U05OwBatchB, "bingqiang" | "bingqiang_remove", EventKind::Death) => {
            Some(forced(kind, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchB, "paotai", EventKind::TurnPhaseEnd | EventKind::DamageEnd) => {
            Some(forced(kind, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchB, "shoujia2", EventKind::CardUsed | EventKind::DamageEnd) => {
            Some(forced(kind, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchC, "duwen", EventKind::DamageBegin) => {
            Some(forced(EventKind::DamageBegin, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchC, "duwen2", EventKind::DamageEnd) => {
            Some(forced(EventKind::DamageEnd, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchC, "yihun2", EventKind::TurnBegin | EventKind::Death) => {
            Some(forced(kind, Timing::First, 3))
        }
        (HostBehavior::U05OwBatchD, "shanxian", EventKind::TurnBegin) => {
            Some(forced(EventKind::TurnBegin, Timing::First, 5))
        }
        (HostBehavior::U05OwBatchD, "shanhui", EventKind::DamageEnd) => {
            Some(forced(kind, Timing::Normal, 1))
        }
        (HostBehavior::U05OwBatchD, "shanxian2", _) => Some(forced(kind, Timing::Normal, 1)),
        (HostBehavior::U05OwBatchD, "xie2", EventKind::TurnPhaseEnd | EventKind::Death) => {
            Some(forced(kind, Timing::Normal, 1))
        }
        (
            HostBehavior::U05OwBatchD,
            "luan2",
            EventKind::DamageEnd | EventKind::DyingSaved | EventKind::Death,
        ) => Some(forced(kind, Timing::Normal, 1)),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u05_ow(name: &str) -> SkillLimits {
    match name {
        "qianggu" | "dianji" | "liangou" | "zhongdun" | "yinbo" | "aqianghua" | "shoujia"
        | "shanguang" | "mianzhen" | "juji" | "tuijin" | "liangou2" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn has_phase_use_activation_u05_ow(
    behavior: HostBehavior,
    name: &str,
) -> bool {
    match behavior {
        HostBehavior::U05OwBatchA => matches!(name, "qianggu" | "dianji" | "liangou"),
        HostBehavior::U05OwBatchB => matches!(
            name,
            "zhongdun"
                | "maoding"
                | "maoding2"
                | "paotai"
                | "yinbo"
                | "aqianghua"
                | "shoujia"
                | "shanguang"
                | "baoxue"
                | "mianzhen"
                | "bingqiang"
        ),
        HostBehavior::U05OwBatchC => {
            matches!(name, "juji" | "dulei" | "tuijin" | "zihui" | "mujing")
        }
        HostBehavior::U05OwBatchD => matches!(
            name,
            "xie" | "luan" | "sheng" | "huoyu" | "huhuan" | "kuoyin" | "guangshu" | "shouhu"
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn is_bolan_eligible_u05_ow(name: &str) -> bool {
    !matches!(name, "baoxue" | "sheng" | "huoyu" | "shouhu")
}

pub(in crate::content::host) fn activate_u05_ow(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let ok = match skill.behavior {
        HostBehavior::U05OwBatchA => u05_ow_a::can_activate_a(&skill.name, ctx, owner),
        HostBehavior::U05OwBatchB => u05_ow_b::can_activate_b(&skill.name, ctx, owner),
        HostBehavior::U05OwBatchC => u05_ow_c::can_activate_c(&skill.name, ctx, owner),
        HostBehavior::U05OwBatchD => u05_ow_d::can_activate_d(&skill.name, ctx, owner),
        _ => false,
    };
    if !ok {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_u05_ow(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let (target, targets, cards) = match event {
        EventData::TurnBegin { player }
        | EventData::Death { player, .. }
        | EventData::DyingSaved { player, .. }
        | EventData::TurnOvered { player, .. } => (Some(*player), Vec::new(), Vec::new()),
        EventData::Phase { player, begin, .. } => {
            (Some(*player), Vec::new(), vec![u32::from(*begin)])
        }
        EventData::TurnEndWindow { player, after } => {
            (Some(*player), Vec::new(), vec![u32::from(*after)])
        }
        EventData::Damage(d) => (
            Some(d.target),
            d.source.into_iter().collect(),
            vec![d.card.map(|c| c + 1).unwrap_or(0), d.amount.max(0) as u32],
        ),
        EventData::CardUsed(u) => (
            Some(u.player),
            u.targets.clone(),
            u.card.into_iter().collect(),
        ),
        EventData::CardsLost { player, cards, .. }
        | EventData::CardsDiscarded { player, cards }
        | EventData::CardsDrawn { player, cards } => (Some(*player), Vec::new(), cards.clone()),
        _ => (None, Vec::new(), Vec::new()),
    };
    HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card: None,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn filter_u05_ow_with_frames(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    frames: &[Frame],
    frame: &Frame,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.behavior {
        HostBehavior::U05OwBatchA => {
            u05_ow_a::filter_a(&skill.name, ctx, frames, frame, owner, event)
        }
        HostBehavior::U05OwBatchB => {
            u05_ow_b::filter_b(&skill.name, ctx, frames, frame, owner, event)
        }
        HostBehavior::U05OwBatchC => {
            u05_ow_c::filter_c(&skill.name, ctx, frames, frame, owner, event)
        }
        HostBehavior::U05OwBatchD => {
            u05_ow_d::filter_d(&skill.name, ctx, frames, frame, owner, event)
        }
        _ => false,
    }
}

pub(in crate::content::host) fn step_u05_ow(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    ensure_ow_player_inited(ctx.core, ctx.catalog, m.owner);
    match m.behavior {
        HostBehavior::U05OwBatchA => u05_ow_a::step_a(m, ctx),
        HostBehavior::U05OwBatchB => u05_ow_b::step_b(m, ctx),
        HostBehavior::U05OwBatchC => u05_ow_c::step_c(m, ctx),
        HostBehavior::U05OwBatchD => u05_ow_d::step_d(m, ctx),
        _ => Flow::Done,
    }
}
