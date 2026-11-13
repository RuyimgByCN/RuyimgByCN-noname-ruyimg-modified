//! L20 标记补充 Batch B（20 个核心规则技能，`标记补充/extension.js`）。
//!
//! 覆盖项：
//! 1. `zhengsu`（整肃，`730`）
//! 2. `zhenyi_spade`（真仪·黑桃，`1041`）
//! 3. `tspowei`（破围，`4366`）
//! 4. `qingyu`（清裕，`4534`）
//! 5. `mibei`（密备，`4592`）
//! 6. `fz_new_rewusheng`（武圣，`4676`）
//! 7. `fz_new_longdan`（龙胆，`4860`）
//! 8. `fz_new_tieji`（铁骑，`5089`）
//! 9. `fz_liegong`（烈弓，`5160`）
//! 10. `fz_xinkuanggu`（狂骨，`5199`）
//! 11–20. 见 `l20_batch_b_ext.rs`（`rekuanshi`、`kuanshi`、`kuanshi2`、`rechuhai`、
//!   `xinfu_lingren`、`sbtieji`、`sbduanliang`、`shouye`、`mbmowang`；`zuoxing` 上游整段注释，
//!   L20 不覆盖）

use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l20_batch_b_asks.rs"]
mod l20_batch_b_asks;
#[path = "l20_batch_b_ext.rs"]
mod l20_batch_b_ext;
#[path = "l20_batch_b_mouyi.rs"]
mod l20_batch_b_mouyi;

pub(in crate::content::host) use l20_batch_b_asks::ask_l20_batch_b;
use l20_batch_b_asks::wait_ask;
use l20_batch_b_ext::*;
use l20_batch_b_mouyi::*;

pub(super) const ASK_ZHENGSU_CHOOSE_MODE: u32 = 10;
pub(super) const ASK_ZHENGSU_CHOOSE_REWARD: u32 = 11;
pub(super) const ASK_ZHENYI_SPADE_CONFIRM: u32 = 12;
pub(super) const ASK_TSPOWEI_CHOOSE_ACTION: u32 = 13;
pub(super) const ASK_QINGYU_DISCARD_TWO: u32 = 14;
pub(super) const ASK_TSPOWEI_CHOOSE_CARD: u32 = 15;
pub(super) const ASK_FZ_LIEGONG_CONFIRM: u32 = 16;
pub(super) const ASK_FZ_NEW_LONGDAN_TARGET: u32 = 17;
pub(super) const ASK_FZ_NEW_TIEJI_CONFIRM: u32 = 18;
pub(super) const ASK_FZ_NEW_TIEJI_TARGET_DISCARD: u32 = 19;
pub(super) const ASK_FZ_XINKUANGGU_MODE: u32 = 20;
pub(super) const ASK_REKUANSHI_TARGET: u32 = 21;
pub(super) const ASK_KUANSHI_TARGET: u32 = 22;
pub(super) const ASK_RECHUHAI_TARGET: u32 = 23;
pub(super) const ASK_RECHUHAI_PINDIAN_CARD: u32 = 24;
pub(super) const ASK_RECHUHAI_TARGET_PINDIAN_CARD: u32 = 25;
pub(super) const ASK_XINFU_LINGREN_GUESS_MASK: u32 = 26;
pub(super) const ASK_SBTIEJI_CONFIRM: u32 = 27;
pub(super) const ASK_SBTIEJI_TARGET_GIVE: u32 = 28;
pub(super) const ASK_SBDUANLIANG_TARGET: u32 = 29;
pub(super) const ASK_SHOUYE_TARGETS: u32 = 30;
pub(super) const ASK_XINHONGYAN_SUIT: u32 = 33;
pub(super) const ASK_XINFU_LANGXI_TARGET: u32 = 34;
pub(super) const ASK_CHONGZHU_CARD: u32 = 35;
pub(super) const ASK_XINFU_LINGREN_TARGET: u32 = 36;
pub(super) const ASK_SBTIEJI_MOUYI_OWNER: u32 = 37;
pub(super) const ASK_SBTIEJI_MOUYI_TARGET: u32 = 38;
pub(super) const ASK_SBDUANLIANG_MOUYI_OWNER: u32 = 39;
pub(super) const ASK_SBDUANLIANG_MOUYI_TARGET: u32 = 40;
pub(super) const ASK_SBDUANLIANG_GAIN_CARD: u32 = 41;
pub(super) const ASK_SHOUYE_MOUYI_OWNER: u32 = 42;
pub(super) const ASK_SHOUYE_MOUYI_TARGET: u32 = 43;
pub(super) const ASK_TSPOWEI_GAIN_CARD: u32 = 44;
pub(super) const ASK_FZ_NEW_TIEJI_CHOOSE_SEAL: u32 = 45;

pub(super) fn has_shouyue_on_table(core: &GameStateCore, catalog: &Catalog) -> bool {
    let shouyue_def = catalog.skill_by_name("shouyue");
    (0..core.players.len() as PlayerId).any(|p| {
        core.player(p).alive
            && (flag(core, p, "shouyue_active") > 0
                || shouyue_def.is_some_and(|def| core.has_skill(p, def)))
    })
}

pub(super) fn is_recastable_card(
    _core: &GameStateCore,
    catalog: &Catalog,
    _owner: PlayerId,
    cid: CardId,
) -> bool {
    // 上游 `_chongzhu.filterCard`（extension.js:6243-6251）：牌定义 `info.chongzhu`
    // ——原生目录中仅【铁索连环】带重铸标记。
    catalog.card_kind(cid) == CardKind::Chain
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

pub(super) fn alive_others(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive)
        .collect()
}

pub(super) fn alive_all(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| core.player(p).alive)
        .collect()
}

pub(super) fn hand_and_equips(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    let mut out = core.player(p).hand.clone();
    out.extend(core.player(p).equips.iter().copied());
    out
}

pub(super) fn card_rank(catalog: &Catalog, cid: CardId) -> u32 {
    catalog
        .cards
        .get(cid as usize)
        .map_or(1, |c| u32::from(c.rank))
}

pub(super) fn suit_code(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn suit_bit_of(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
) -> u32 {
    match catalog.effective_card_suit(core, owner, cid) {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 4,
        Suit::Diamond => 8,
    }
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    let kind = u
        .use_as
        .or_else(|| u.card.map(|c| catalog.card_kind(c)))
        .unwrap_or(CardKind::Sha);
    kind.is_sha_like()
}

pub(super) fn mark_sha_unrespondable(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
    cid: Option<CardId>,
) {
    if let Some(c) = cid {
        ctx.core
            .vm_aux
            .flags
            .insert((owner, format!("sha_unrespondable:{c}:{target}")), 1);
    }
}

pub(super) fn draw_direct(ctx: &mut Ctx<'_>, p: PlayerId, n: u32) {
    let mut drawn = Vec::new();
    for _ in 0..n {
        if let Some(c) = ctx.core.deck.pop() {
            ctx.core.player_mut(p).hand.push(c);
            drawn.push(c);
        }
    }
    if !drawn.is_empty() {
        ctx.emit(EventData::CardsDrawn {
            player: p,
            cards: drawn,
        });
    } else if n > 0 {
        ctx.start_draw(p, n);
    }
}

pub(super) fn emit_damage(
    ctx: &mut Ctx<'_>,
    source: Option<PlayerId>,
    target: PlayerId,
    amount: i32,
    nature: DamageNature,
) {
    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
        source,
        target,
        amount.max(1),
        nature,
        None,
    )));
}

pub(super) fn discard_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut discarded = Vec::new();
    for &c in cards {
        if ctx.core.player(p).hand.contains(&c) {
            ctx.discard_from_hand(p, c);
            discarded.push(c);
        } else if ctx.core.player(p).equips.contains(&c) {
            ctx.take_owned_card(p, c);
            ctx.to_discard(c);
            discarded.push(c);
        }
    }
    if !discarded.is_empty() {
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: discarded,
        });
    }
}

pub(super) fn move_cards_to_hand(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    cards: &[CardId],
) {
    if from == to || cards.is_empty() {
        return;
    }
    let mut moved = Vec::new();
    let mut origins = Vec::new();
    for &c in cards {
        if let Some(pos) = ctx.core.player(from).hand.iter().position(|&x| x == c) {
            ctx.core.player_mut(from).hand.remove(pos);
            moved.push(c);
            origins.push(LostOrigin::Hand);
        } else if let Some(pos) = ctx.core.player(from).equips.iter().position(|&x| x == c) {
            ctx.core.player_mut(from).equips.remove(pos);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, c);
            moved.push(c);
            origins.push(LostOrigin::Equip);
        }
    }
    if !moved.is_empty() {
        ctx.note_cards_lost(from, &moved);
        ctx.core.player_mut(to).hand.extend(moved.iter().copied());
        ctx.emit(EventData::CardsLost {
            player: from,
            cards: moved.clone(),
            origins,
        });
        ctx.emit(EventData::CardsMoved {
            cards: moved,
            from,
            to,
        });
    }
}

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) -> Option<u16> {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.skills.insert(
            (p, def),
            crate::state::SkillInstance {
                owner: p,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        return Some(def);
    }
    None
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(in crate::content::host) fn trigger_meta_l20_batch_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "zhengsu" => Some(meta(EventKind::PhaseEnd, vec![EventKind::CardUsed, EventKind::Equipped, EventKind::CardsDiscarded])),
        "zhenyi_spade" => Some(TriggerMeta {
            event: EventKind::OrdinaryJudge,
            timing: Timing::First,
            priority: 2,
            order_selectable: false,
            optional: false,
            extra_events: Vec::new(),
        }),
        "xinhongyan" => Some(meta(EventKind::OrdinaryJudge, Vec::new())),
        "xinfu_langxi" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "fz_gzpaoxiao" => Some(meta(EventKind::CardUsed, Vec::new())),
        "tspowei" => Some(meta(EventKind::TurnBegin, vec![EventKind::PhaseBegin, EventKind::TurnEnded, EventKind::Damage, EventKind::Dying])),
        "qingyu" => Some(meta(EventKind::DamageBefore, vec![EventKind::PhaseBegin, EventKind::Dying])),
        // 装备牌在原生出牌路径只发 `Equipped`（不发 CardUsed）——上游 useCard 同计。
        "mibei" => Some(meta(EventKind::CardUsed, vec![EventKind::PhaseBegin, EventKind::Equipped])),
        "fz_new_longdan" => Some(meta(EventKind::CardResponded, vec![EventKind::ShaDodged, EventKind::CardUsed])),
        "fz_new_tieji" | "fz_liegong" => Some(meta(EventKind::CardUsed, Vec::new())),
        "sbtieji" | "shouye" => Some(meta(EventKind::CardUsed, vec![EventKind::TurnBegin, EventKind::TurnEnded])),
        "fz_xinkuanggu" => Some(meta(EventKind::Damage, Vec::new())),
        "rekuanshi" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin, EventKind::Damage])),
        "kuanshi" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin, EventKind::DamageBefore, EventKind::Death])),
        "kuanshi2" => Some(meta(EventKind::DamageBefore, vec![EventKind::TurnBegin, EventKind::PhaseBegin, EventKind::Death])),
        "rechuhai" => Some(meta(EventKind::Damage, vec![EventKind::Equipped, EventKind::PhaseEnd, EventKind::TurnBegin, EventKind::TurnEnded])),
        "xinfu_lingren" => Some(meta(EventKind::CardUsed, vec![EventKind::TurnBegin, EventKind::PhaseBegin, EventKind::PhaseEnd, EventKind::TurnEnded])),
        "mbmowang" => Some(meta(EventKind::Death, vec![EventKind::TurnBegin, EventKind::PhaseBegin, EventKind::TurnEnded, EventKind::PhaseEnd])),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l20_batch_b(name: &str) -> SkillLimits {
    match name {
        "rechuhai" | "xinfu_lingren" | "sbduanliang" | "shouye" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_l20_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    match skill.name.as_str() {
        "zhengsu" => match event {
            EventData::CardUsed(u) => {
                u.player == owner
                    && ctx.state.turn.phase == Phase::Play
                    && (matches!(flag(ctx.state, owner, "zhengsu_mode"), 1 | 2)
                        || flag(ctx.state, owner, "zhengsu_leijin") > 0
                        || flag(ctx.state, owner, "zhengsu_bianzhen") > 0)
                    && flag(ctx.state, owner, "zhengsu_failed") == 0
            }
            EventData::Equipped { player, .. } => {
                *player == owner
                    && ctx.state.turn.phase == Phase::Play
                    && (matches!(flag(ctx.state, owner, "zhengsu_mode"), 1 | 2)
                        || flag(ctx.state, owner, "zhengsu_leijin") > 0
                        || flag(ctx.state, owner, "zhengsu_bianzhen") > 0)
                    && flag(ctx.state, owner, "zhengsu_failed") == 0
            }
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && ctx.state.turn.phase == Phase::Discard
                    && (flag(ctx.state, owner, "zhengsu_mode") == 3
                        || flag(ctx.state, owner, "zhengsu_mingzhi") > 0)
                    && flag(ctx.state, owner, "zhengsu_failed") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: false,
            } => {
                *player == owner
                    && (flag(ctx.state, owner, "zhengsu_mode") > 0
                        || flag(ctx.state, owner, "zhengsu_leijin") > 0
                        || flag(ctx.state, owner, "zhengsu_bianzhen") > 0
                        || flag(ctx.state, owner, "zhengsu_mingzhi") > 0
                        || flag(ctx.state, owner, "zhengsu_succeeded") > 0
                        || flag(ctx.state, owner, "zhengsu_failed") > 0)
            }
            _ => false,
        },
        "zhenyi_spade" => {
            if flag(ctx.state, owner, "zhenyi_mark") == 0
                && flag(ctx.state, owner, "xinfu_falu_spade") == 0
            {
                return false;
            }
            matches!(
                event,
                EventData::OrdinaryJudge { owner: j_owner, invocation }
                    if *invocation != u32::MAX
                        && flag(ctx.state, *j_owner, &format!("judge_pre_finalized:{invocation}")) > 0
            )
        }
        "tspowei" => {
            if flag(ctx.state, owner, "tspowei_resolved") > 0
                || flag(ctx.state, owner, "tspowei_achieved") > 0
                || flag(ctx.state, owner, "tspowei_failed") > 0
            {
                return false;
            }
            match event {
                EventData::TurnBegin { player } => {
                    if flag(ctx.state, owner, "tspowei_init") == 0 || *player == owner {
                        true
                    } else if flag(ctx.state, *player, "tspowei_wei") > 0 {
                        !ctx.state.player(owner).hand.is_empty()
                            || (ctx.state.player(owner).hp >= ctx.state.player(*player).hp
                                && !ctx.state.player(*player).hand.is_empty())
                    } else {
                        false
                    }
                }
                EventData::Phase {
                    player,
                    phase: Phase::Prepare,
                    begin: true,
                } => {
                    if *player == owner {
                        flag(ctx.state, owner, "tspowei_init") == 0
                            || !alive_others(ctx.state, owner)
                                .iter()
                                .any(|&p| flag(ctx.state, p, "tspowei_wei") > 0)
                    } else {
                        flag(ctx.state, *player, "tspowei_wei") > 0
                            && flag(ctx.state, owner, "tspowei_turn_prompted")
                                != u32::from(*player) + 1
                            && (!ctx.state.player(owner).hand.is_empty()
                                || (ctx.state.player(owner).hp >= ctx.state.player(*player).hp
                                    && !ctx.state.player(*player).hand.is_empty()))
                    }
                }
                EventData::TurnEnded { .. } => flag(ctx.state, owner, "tspowei_in_range") > 0,
                EventData::Damage(d) => {
                    !d.prevented
                        && d.amount > 0
                        && d.settlement.committed_damage.is_some()
                        && flag(ctx.state, d.target, "tspowei_wei") > 0
                }
                EventData::Dying { player, .. } => *player == owner,
                _ => false,
            }
        }
        "qingyu" => match event {
            EventData::Damage(d) => {
                d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.settlement.committed_damage.is_none()
                    && flag(ctx.state, owner, "qingyu_resolved") == 0
                    && hand_and_equips(ctx.state, owner).len() >= 2
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "qingyu_resolved") == 0
                    && ctx.state.player(owner).hp >= ctx.state.player(owner).max_hp as i32
                    && ctx.state.player(owner).hand.is_empty()
            }
            EventData::Dying { player, .. } => {
                *player == owner && flag(ctx.state, owner, "qingyu_resolved") == 0
            }
            _ => false,
        },
        "mibei" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::Play,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "mibei_resolved") == 0,
            EventData::CardUsed(u) => {
                u.player == owner && flag(ctx.state, owner, "mibei_resolved") == 0
            }
            EventData::Equipped { player, .. } => {
                *player == owner
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == Phase::Play
                    && flag(ctx.state, owner, "mibei_resolved") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "mibei_resolved") == 0
                    && flag(ctx.state, owner, "mibei_mark") > 0
                    && flag(ctx.state, owner, "mibei_basic_cnt") == 0
                    && flag(ctx.state, owner, "mibei_trick_cnt") == 0
                    && flag(ctx.state, owner, "mibei_equip_cnt") == 0
                    && flag(ctx.state, owner, "xingqi_count") == 0
            }
            _ => false,
        },
        other => filter_l20_batch_b_ext(other, ctx, owner, event),
    }
}

pub(in crate::content::host) fn activate_l20_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive
        || ctx.state.turn.current != owner
        || ctx.state.turn.phase != Phase::Play
        || ctx.catalog.has_diaohulishan(ctx.state, owner)
    {
        return None;
    }
    match skill.name.as_str() {
        "rechuhai" => {
            let stamp = (ctx.state.turn.round.max(1) << 8) | u32::from(ctx.state.turn.current);
            if flag(ctx.state, owner, "rechuhai_resolved") != 0
                || flag(ctx.state, owner, "rechuhai_play_stamp") == stamp
                || !alive_others(ctx.state, owner)
                    .iter()
                    .any(|&p| !ctx.state.player(p).hand.is_empty())
            {
                return None;
            }
        }
        "sbduanliang" => {
            if !alive_others(ctx.state, owner)
                .into_iter()
                .any(|p| !ctx.catalog.has_diaohulishan(ctx.state, p))
            {
                return None;
            }
        }
        "_chongzhu" => {
            if !ctx
                .state
                .player(owner)
                .hand
                .iter()
                .any(|&cid| is_recastable_card(ctx.state, ctx.catalog, owner, cid))
            {
                return None;
            }
        }
        _ => return None,
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

pub(in crate::content::host) fn machine_l20_batch_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("zhengsu", EventData::CardUsed(u)) => {
            card = Some(1);
            cards.extend(u.card);
        }
        ("zhengsu", EventData::Equipped { card: eq, .. }) => {
            card = Some(1);
            cards.push(*eq);
        }
        ("zhengsu", EventData::CardsDiscarded { cards: disc, .. }) => {
            card = Some(2);
            cards = disc.clone();
        }
        ("zhengsu", EventData::Phase { phase: Phase::Discard, begin: false, .. }) => card = Some(3),
        ("zhenyi_spade", EventData::OrdinaryJudge { owner: j_owner, invocation }) => {
            target = Some(*j_owner);
            cards.push(*invocation);
        }
        ("tspowei", EventData::TurnBegin { player }) => {
            card = Some(0);
            target = Some(*player);
        }
        ("tspowei", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            card = Some(2);
            target = Some(*player);
        }
        ("tspowei", EventData::TurnEnded { .. }) => card = Some(4),
        ("tspowei", EventData::Damage(d)) => {
            card = Some(1);
            target = Some(d.target);
        }
        ("tspowei", EventData::Dying { .. }) => card = Some(3),
        ("qingyu", EventData::Damage(_)) => card = Some(0),
        ("qingyu", EventData::Phase { phase: Phase::Prepare, begin: true, .. }) => card = Some(1),
        ("qingyu", EventData::Dying { .. }) => card = Some(2),
        ("mibei", EventData::Phase { phase: Phase::Prepare | Phase::Play, begin: true, .. }) => card = Some(0),
        ("mibei", EventData::CardUsed(u)) => {
            card = Some(1);
            cards.extend(u.card);
        }
        ("mibei", EventData::Equipped { card: eq, .. }) => {
            card = Some(1);
            cards.push(*eq);
        }
        ("mibei", EventData::Phase { phase: Phase::End, begin: true, .. }) => card = Some(4),
        ("fz_new_longdan", EventData::CardResponded { card: cid, .. }) => {
            card = Some(1);
            cards.push(*cid);
        }
        ("fz_new_longdan", EventData::CardUsed(u)) => {
            card = if u.use_as == Some(CardKind::Shan) {
                Some(1)
            } else {
                Some(3)
            };
            cards.extend(u.card);
        }
        ("fz_new_longdan", EventData::ShaDodged { target: t, card: cid, .. }) => {
            card = Some(2);
            target = Some(*t);
            cards.extend(*cid);
        }
        ("fz_new_tieji" | "fz_liegong", EventData::CardUsed(u)) => {
            targets = u.targets.clone();
            target = targets.first().copied();
            cards.extend(u.card);
        }
        ("sbtieji", EventData::CardUsed(u)) => {
            targets = u.targets.iter().copied().filter(|&t| t != owner).collect();
            target = targets.first().copied();
            cards.extend(u.card);
        }
        ("sbtieji", EventData::TurnBegin { .. } | EventData::TurnEnded { .. }) => card = Some(1),
        ("xinfu_lingren", EventData::CardUsed(u)) => {
            targets = u.targets.clone();
            target = targets.first().copied();
            cards.extend(u.card);
        }
        ("fz_xinkuanggu", EventData::Damage(d)) => {
            card = Some(d.amount.max(1) as u32);
            cards.push(d.amount.max(1) as u32);
        }
        ("xinfu_lingren", EventData::TurnBegin { .. } | EventData::Phase { phase: Phase::Prepare, begin: true, .. }) => card = Some(1),
        ("xinfu_lingren", EventData::Phase { begin: false, .. } | EventData::TurnEnded { .. }) => card = Some(2),
        ("rekuanshi" | "kuanshi", EventData::Phase { phase: Phase::End, begin: true, .. }) => card = Some(0),
        ("rekuanshi", EventData::TurnBegin { player }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("rekuanshi", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            card = Some(3);
            target = Some(*player);
        }
        ("kuanshi", EventData::Phase { phase: Phase::Prepare | Phase::Judge, begin: true, .. }) => card = Some(1),
        ("kuanshi", EventData::Death { .. }) => card = Some(3),
        ("rekuanshi", EventData::Damage(d)) => {
            card = Some(2);
            target = Some(d.target);
            cards.push(d.amount.max(1) as u32);
        }
        ("kuanshi", EventData::Damage(d)) => {
            card = Some(2);
            target = Some(d.target);
        }
        ("kuanshi2", EventData::Damage(_)) => card = Some(2),
        ("kuanshi2", EventData::Phase { player, phase: Phase::Prepare, begin: true } | EventData::Death { player, .. }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("rechuhai", EventData::Damage(d)) => {
            card = Some(2);
            target = Some(d.target);
        }
        ("rechuhai", EventData::Equipped { .. }) => card = Some(3),
        ("rechuhai", EventData::Phase { phase: Phase::Play, begin: false, .. } | EventData::TurnBegin { .. } | EventData::TurnEnded { .. }) => card = Some(4),
        ("shouye", EventData::CardUsed(u)) => {
            target = Some(u.player);
            cards.extend(u.card);
        }
        ("shouye", EventData::TurnBegin { .. } | EventData::TurnEnded { .. }) => card = Some(1),
        ("mbmowang", EventData::Death { player, .. }) => {
            card = Some(0);
            target = Some(*player);
        }
        ("mbmowang", EventData::TurnBegin { player } | EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("mbmowang", EventData::TurnEnded { player } | EventData::Phase { player, phase: Phase::End, begin: false }) => {
            card = Some(2);
            target = Some(*player);
        }
        ("xinhongyan", EventData::OrdinaryJudge { owner: j_owner, invocation }) => {
            target = Some(*j_owner);
            cards.push(*invocation);
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
        targets,
        stage: HostStage::Act,
    }
}

fn clear_zhengsu_flags(core: &mut GameStateCore, owner: PlayerId) {
    for k in [
        "zhengsu_mode",
        "zhengsu_leijin",
        "zhengsu_bianzhen",
        "zhengsu_mingzhi",
        "zhengsu_count",
        "zhengsu_last_rank",
        "zhengsu_suit",
        "zhengsu_suit_mask",
        "zhengsu_failed",
        "zhengsu_succeeded",
        "zhengsu_used_cnt",
        "zhengsu_discard_cnt",
    ] {
        set_flag(core, owner, k, 0);
    }
}

pub(in crate::content::host) fn handle_l20_batch_b(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "zhengsu" => match m.stage {
            HostStage::Act => match m.card {
                Some(1) => {
                    let mut mode = flag(ctx.core, owner, "zhengsu_mode");
                    if mode == 0 {
                        if flag(ctx.core, owner, "zhengsu_leijin") > 0 {
                            mode = 1;
                        } else if flag(ctx.core, owner, "zhengsu_bianzhen") > 0 {
                            mode = 2;
                        }
                    }
                    let Some(&cid) = m.cards.first() else {
                        set_flag(ctx.core, owner, "zhengsu_failed", 1);
                        set_flag(ctx.core, owner, "zhengsu_succeeded", 0);
                        return Flow::Done;
                    };
                    let Some(cdef) = ctx.catalog.cards.get(cid as usize) else {
                        set_flag(ctx.core, owner, "zhengsu_failed", 1);
                        set_flag(ctx.core, owner, "zhengsu_succeeded", 0);
                        return Flow::Done;
                    };
                    if mode == 1 {
                        let rank = u32::from(cdef.rank);
                        let prev = flag(ctx.core, owner, "zhengsu_last_rank");
                        if rank > 0 && rank > prev {
                            set_flag(ctx.core, owner, "zhengsu_last_rank", rank);
                            let cnt = flag(ctx.core, owner, "zhengsu_count") + 1;
                            set_flag(ctx.core, owner, "zhengsu_count", cnt);
                            set_flag(ctx.core, owner, "zhengsu_used_cnt", cnt);
                            if cnt >= 3 && flag(ctx.core, owner, "zhengsu_failed") == 0 {
                                set_flag(ctx.core, owner, "zhengsu_succeeded", 1);
                            }
                        } else {
                            set_flag(ctx.core, owner, "zhengsu_failed", 1);
                            set_flag(ctx.core, owner, "zhengsu_succeeded", 0);
                        }
                    } else if mode == 2 {
                        let suit = suit_code(ctx.catalog.effective_card_suit(ctx.core, owner, cid));
                        let prev = flag(ctx.core, owner, "zhengsu_suit");
                        if suit > 0 && (prev == 0 || prev == suit) {
                            set_flag(ctx.core, owner, "zhengsu_suit", suit);
                            let cnt = flag(ctx.core, owner, "zhengsu_count") + 1;
                            set_flag(ctx.core, owner, "zhengsu_count", cnt);
                            set_flag(ctx.core, owner, "zhengsu_used_cnt", cnt);
                            if cnt >= 2 && flag(ctx.core, owner, "zhengsu_failed") == 0 {
                                set_flag(ctx.core, owner, "zhengsu_succeeded", 1);
                            }
                        } else {
                            set_flag(ctx.core, owner, "zhengsu_failed", 1);
                            set_flag(ctx.core, owner, "zhengsu_succeeded", 0);
                        }
                    }
                    Flow::Done
                }
                Some(2) => {
                    for &cid in &m.cards {
                        let bit = suit_bit_of(ctx.core, ctx.catalog, owner, cid);
                        let mask = flag(ctx.core, owner, "zhengsu_suit_mask");
                        if (mask & bit) != 0 {
                            set_flag(ctx.core, owner, "zhengsu_failed", 1);
                            set_flag(ctx.core, owner, "zhengsu_succeeded", 0);
                        } else {
                            set_flag(ctx.core, owner, "zhengsu_suit_mask", mask | bit);
                            let cnt = flag(ctx.core, owner, "zhengsu_count") + 1;
                            set_flag(ctx.core, owner, "zhengsu_count", cnt);
                            set_flag(ctx.core, owner, "zhengsu_discard_cnt", cnt);
                            if cnt >= 2 && flag(ctx.core, owner, "zhengsu_failed") == 0 {
                                set_flag(ctx.core, owner, "zhengsu_succeeded", 1);
                            }
                        }
                    }
                    Flow::Done
                }
                Some(3) => {
                    let ok = flag(ctx.core, owner, "zhengsu_succeeded") > 0
                        && flag(ctx.core, owner, "zhengsu_failed") == 0;
                    clear_zhengsu_flags(ctx.core, owner);
                    if !ok {
                        return Flow::Done;
                    }
                    if ctx.core.player(owner).hp >= ctx.core.player(owner).max_hp as i32 {
                        draw_direct(ctx, owner, 2);
                        set_flag(ctx.core, owner, "zhengsu_rewarded", 1);
                        return Flow::Done;
                    }
                    wait_ask(m, ctx, ASK_ZHENGSU_CHOOSE_REWARD)
                }
                _ => Flow::Done,
            },
            HostStage::SelectCard => {
                let ask_code = m.card.unwrap_or(0);
                let ans = ctx.answer();
                let chosen = ans.iter().find_map(|o| match o {
                    ChoiceOption::Tag(t) => Some(*t),
                    _ => None,
                });
                let Some(tag) = chosen else {
                    return Flow::Done;
                };
                if ask_code == ASK_ZHENGSU_CHOOSE_REWARD {
                    if tag == 0 {
                        draw_direct(ctx, owner, 2);
                        set_flag(ctx.core, owner, "zhengsu_rewarded", 1);
                    } else {
                        ctx.recover(owner, 1, None);
                        set_flag(ctx.core, owner, "zhengsu_rewarded", 2);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        "zhenyi_spade" => match m.stage {
            HostStage::Act => wait_ask(m, ctx, ASK_ZHENYI_SPADE_CONFIRM),
            HostStage::SelectCard => {
                let ans = ctx.answer();
                let chosen_suit = ans
                    .iter()
                    .find_map(|o| match o {
                        // 上游 `['spade','heart','diamond','club']`：
                        // Tag(0)=♠(1), Tag(1)=♥(2), Tag(2)=♦(4), Tag(3)=♣(3)。
                        ChoiceOption::Tag(0) => Some(1),
                        ChoiceOption::Tag(1) => Some(2),
                        ChoiceOption::Tag(2) => Some(4),
                        ChoiceOption::Tag(_) => Some(3),
                        ChoiceOption::Confirm => Some(1),
                        _ => None,
                    });
                let Some(suit_val) = chosen_suit else {
                    return Flow::Done;
                };
                let rem_zy = flag(ctx.core, owner, "zhenyi_mark").saturating_sub(1);
                set_flag(ctx.core, owner, "zhenyi_mark", rem_zy);
                let rem_alt = flag(ctx.core, owner, "xinfu_falu_spade").saturating_sub(1);
                set_flag(ctx.core, owner, "xinfu_falu_spade", rem_alt);
                let j_owner = m.target.unwrap_or(owner);
                set_flag(ctx.core, owner, "zhenyi_spade_applied", 1);
                // 判定改写只写到判定者——`effective_judge_suit_rank` 按判定者消费；
                // 若同时写给持有者，持有者下次判定会误读残留改写。
                set_flag(ctx.core, j_owner, "zhenyi_judge_suit", suit_val);
                set_flag(ctx.core, j_owner, "zhenyi_judge_number", 5);
                Flow::Done
            }
            _ => Flow::Done,
        },
        "tspowei" | "qingyu" | "mibei" => handle_l20_duty_three(m, ctx),
        "fz_new_longdan" => handle_fz_new_longdan(m, ctx),
        "fz_new_tieji" => handle_fz_new_tieji(m, ctx),
        "fz_liegong" => handle_fz_liegong(m, ctx),
        "fz_xinkuanggu" => handle_fz_xinkuanggu(m, ctx),
        "rekuanshi" => handle_rekuanshi(m, ctx),
        "kuanshi" => handle_kuanshi(m, ctx),
        "kuanshi2" => handle_kuanshi2(m, ctx),
        "rechuhai" => handle_rechuhai(m, ctx),
        "xinfu_lingren" => handle_xinfu_lingren(m, ctx),
        "sbtieji" => handle_sbtieji(m, ctx),
        "sbduanliang" => handle_sbduanliang(m, ctx),
        "shouye" => handle_shouye(m, ctx),
        "mbmowang" => handle_mbmowang(m, ctx),
        "xinhongyan" | "xinfu_langxi" | "fz_gzpaoxiao" | "_chongzhu" => {
            l20_batch_b_asks::handle_l20_extra_four(m, ctx)
        }
        _ => Flow::Done,
    }
}
