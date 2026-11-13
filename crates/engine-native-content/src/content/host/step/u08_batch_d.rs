//! U08 英雄杀 Batch D（26 技能 + `qiangxi`，覆盖 8 名武将，`英雄杀/character/skill.js:2407-3175`）。
//!
//! 覆盖武将与技能：
//! 1. `yxs_bole`（伯乐）：`bolehuiyan`、`bolehuiyan2`、`bolehuiyan3`、`oldbolehuiyan`、`oldbolehuiyan2`、`xiangma`
//! 2. `yxs_wuzetian`（武则天）：`yxsweiyi`、`qiandu`、`nvquan`、`nvquan1`、`nvquan2`、`nvquan3`
//! 3. `yxs_mozi`（墨子）：`feigong`、`jianai`、`jieyong`、`jieyong2`、`jieyong3`
//! 4. `yxs_caocao`（曹操）：`zhulu`、`xieling`
//! 5. `yxs_zhuyuanzhang`（朱元璋）：`qiangyun`
//! 6. `yxs_jinke`（荆轲）：`cike`、`qiangxi`
//! 7. `yxs_libai`（李白）：`miaobix`、`miaobi`、`zhexian`
//! 8. `yxs_luban`（鲁班）：`guifu`、`lshengong`

use super::super::*;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::state::GameStateCore;
use crate::vocabulary::generals::Gender;

#[path = "u08_batch_d_asks.rs"]
mod u08_batch_d_asks;

pub(in crate::content::host) use u08_batch_d_asks::ask_u08_batch_d;
use u08_batch_d_asks::*;

// ── Ask IDs ─────────────────────────────────────────────────────────────
pub(super) const ASK_BOLEHUIYAN_PREDICT: u32 = 1;
pub(super) const ASK_BOLEHUIYAN_DISCARD: u32 = 2;
pub(super) const ASK_OLDBOLEHUIYAN_SUIT: u32 = 3;
pub(super) const ASK_YXSWEIYI_CONFIRM: u32 = 4;
pub(super) const ASK_YXSWEIYI_SOURCE_DISCARD: u32 = 5;
pub(super) const ASK_QIANDU_CARD: u32 = 6;
pub(super) const ASK_QIANDU_TARGET: u32 = 7;
pub(super) const ASK_NVQUAN1_RESPONSE: u32 = 8;
pub(super) const ASK_NVQUAN2_SHAN: u32 = 9;
pub(super) const ASK_FEIGONG_DISCARD_SHA: u32 = 10;
pub(super) const ASK_JIANAI_CONFIRM: u32 = 11;
pub(super) const ASK_JIEYONG_DISCARD_BLACK: u32 = 12;
pub(super) const ASK_ZHULU_DISCARD_SAME_SUIT: u32 = 13;
pub(super) const ASK_XIELING_DISCARD_TWO: u32 = 14;
pub(super) const ASK_XIELING_CHOOSE_FROM: u32 = 15;
pub(super) const ASK_XIELING_CHOOSE_CARD: u32 = 16;
pub(super) const ASK_XIELING_CHOOSE_TO: u32 = 17;
pub(super) const ASK_QIANGYUN_CONFIRM: u32 = 18;
pub(super) const ASK_CIKE_CONFIRM: u32 = 19;
pub(super) const ASK_CIKE_DISCARD_CARD: u32 = 20;
pub(super) const ASK_QIANGXI_TARGET: u32 = 21;
pub(super) const ASK_QIANGXI_COST: u32 = 22;
pub(super) const ASK_QIANGXI_WEAPON: u32 = 23;
pub(super) const ASK_MIAOBI_TARGET: u32 = 24;
pub(super) const ASK_MIAOBI_DISCARD_TARGET_CARD: u32 = 25;
pub(super) const ASK_MIAOBI_OWNER_DISCARD: u32 = 26;
pub(super) const ASK_MIAOBIX_CARD: u32 = 27;
pub(super) const ASK_MIAOBIX_TARGET: u32 = 28;
pub(super) const ASK_ZHEXIAN_CONFIRM: u32 = 29;
pub(super) const ASK_GUIFU_TARGET: u32 = 30;
pub(super) const ASK_GUIFU_EQUIP: u32 = 31;
pub(super) const ASK_LSHENGONG_HAND: u32 = 32;
pub(super) const ASK_LSHENGONG_SOURCE: u32 = 33;
pub(super) const ASK_LSHENGONG_EQUIP: u32 = 34;
pub(super) const ASK_LSHENGONG_DEST: u32 = 35;

// ── Shared Helpers ──────────────────────────────────────────────────────
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

pub(super) fn has_skill_named(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    name: &str,
) -> bool {
    catalog
        .skill_by_name(name)
        .is_some_and(|def| core.has_skill(p, def))
}

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
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
    }
}

pub(super) fn remove_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        ctx.core.remove_skill_and_r05_mark(ctx.catalog, p, def);
    }
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
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

pub(super) fn suit_code(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
    }
}

pub(super) fn is_red_suit_code(code: u32) -> bool {
    matches!(code, 2 | 4)
}

pub(super) fn is_black_card(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(core, owner, c),
        Suit::Spade | Suit::Club
    )
}

pub(super) fn is_sha_card(core: &GameStateCore, catalog: &Catalog, c: CardId) -> bool {
    core.declared_kinds
        .get(&c)
        .copied()
        .unwrap_or_else(|| catalog.card_kind(c))
        .is_sha_like()
}

pub(super) fn is_sha_use(catalog: &Catalog, u: &CardUseData) -> bool {
    u.use_as
        .unwrap_or_else(|| u.effective_kind(catalog))
        .is_sha_like()
}

pub(super) fn is_delayed_trick_kind(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::Guiyoujie
    )
}

pub(super) fn is_normal_trick_kind(kind: CardKind) -> bool {
    !kind.is_basic_kind() && !kind.is_equip() && !is_delayed_trick_kind(kind)
}

pub(super) fn is_basic_or_normal_trick_kind(kind: CardKind) -> bool {
    !kind.is_equip() && !is_delayed_trick_kind(kind)
}

pub(super) fn player_gender(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Gender {
    core.player(p).general_slots[0]
        .and_then(|def| catalog.general_meta(def).map(|(_, _, _, _, gender)| gender))
        .unwrap_or_else(|| core.effective_gender(p))
}

pub(super) fn is_male(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    player_gender(core, catalog, p) == Gender::Male
}

pub(super) fn mark_sha_unrespondable(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
    cid: Option<CardId>,
) {
    let c = cid.unwrap_or(0);
    set_flag(ctx.core, owner, &format!("sha_unrespondable:{c}:{target}"), 1);
}

pub(super) fn draw_direct(ctx: &mut Ctx<'_>, p: PlayerId, n: u32) -> Vec<CardId> {
    let mut drawn = Vec::new();
    for _ in 0..n {
        if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
            let mut reshuffled = std::mem::take(&mut ctx.core.discard);
            ctx.core.rng.shuffle(&mut reshuffled);
            crate::kernel::elo_on_wash_deck(ctx.core, &reshuffled);
            ctx.core.deck = reshuffled;
            ctx.log(LogEvent::DeckReshuffled);
        }
        if let Some(c) = ctx.core.deck.pop() {
            ctx.core.player_mut(p).hand.push(c);
            drawn.push(c);
        }
    }
    if !drawn.is_empty() {
        ctx.emit(EventData::CardsDrawn {
            player: p,
            cards: drawn.clone(),
        });
    } else if n > 0 {
        ctx.start_draw(p, n);
    }
    drawn
}

pub(super) fn discard_cards(ctx: &mut Ctx<'_>, p: PlayerId, cards: &[CardId]) {
    let mut discarded = Vec::new();
    let mut origins = Vec::new();
    for &c in cards {
        if let Some(pos) = ctx.core.player(p).hand.iter().position(|&x| x == c) {
            ctx.core.player_mut(p).hand.remove(pos);
            ctx.to_discard(Some(c));
            discarded.push(c);
            origins.push(LostOrigin::Hand);
        } else if let Some(pos) = ctx.core.player(p).equips.iter().position(|&x| x == c) {
            ctx.core.player_mut(p).equips.remove(pos);
            ctx.core.declared_slots.remove(&c);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, c);
            ctx.to_discard(Some(c));
            discarded.push(c);
            origins.push(LostOrigin::Equip);
        }
    }
    if !discarded.is_empty() {
        ctx.note_cards_lost(p, &discarded);
        ctx.emit(EventData::CardsLost {
            player: p,
            cards: discarded.clone(),
            origins,
        });
        ctx.emit(EventData::CardsDiscarded {
            player: p,
            cards: discarded.clone(),
        });
        ctx.log(LogEvent::CardDiscarded {
            player: p,
            cards: discarded,
        });
    }
}

pub(super) fn regain_card_to_hand(ctx: &mut Ctx<'_>, owner: PlayerId, cid: CardId) {
    let was_in_discard = ctx.core.discard.contains(&cid);
    let was_in_table = ctx.core.table.contains(&cid);
    if was_in_discard || was_in_table {
        ctx.core.discard.retain(|&x| x != cid);
        ctx.core.table.retain(|&x| x != cid);
        if !ctx.core.player(owner).hand.contains(&cid) {
            ctx.core.player_mut(owner).hand.push(cid);
        }
        set_flag(ctx.core, owner, "shouye_pending_gain_card", cid + 1);
    }
}

pub(super) fn can_move_field_card(
    core: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    cid: CardId,
    to: PlayerId,
) -> bool {
    if from == to || !core.player(from).alive || !core.player(to).alive {
        return false;
    }
    if core.player(from).equips.contains(&cid) {
        let Some(slot) = catalog.card_slot_of(core, cid) else {
            return false;
        };
        !core
            .player(to)
            .equips
            .iter()
            .any(|&ec| catalog.card_slot_of(core, ec) == Some(slot))
    } else if core.player(from).judge.contains(&cid) {
        let k = core
            .declared_kinds
            .get(&cid)
            .copied()
            .unwrap_or_else(|| catalog.card_kind(cid));
        !core.player(to).judge.iter().any(|&jc| {
            core.declared_kinds
                .get(&jc)
                .copied()
                .unwrap_or_else(|| catalog.card_kind(jc))
                == k
        })
    } else {
        false
    }
}

pub(super) fn xieling_movable_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
) -> Vec<CardId> {
    let mut out = Vec::new();
    for &c in core
        .player(from)
        .equips
        .iter()
        .chain(core.player(from).judge.iter())
    {
        if (0..core.players.len() as PlayerId)
            .any(|to| can_move_field_card(core, catalog, from, c, to))
        {
            out.push(c);
        }
    }
    out
}

pub(super) fn xieling_from_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&p| !xieling_movable_cards(core, catalog, p).is_empty())
        .collect()
}

pub(super) fn xieling_to_candidates(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    from: PlayerId,
    cid: CardId,
) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&to| can_move_field_card(core, catalog, from, cid, to))
        .collect()
}

pub(super) fn move_field_card_between(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    cid: CardId,
) {
    if from == to || !ctx.core.player(from).alive || !ctx.core.player(to).alive {
        return;
    }
    if let Some(pos) = ctx.core.player(from).equips.iter().position(|&x| x == cid) {
        ctx.core.player_mut(from).equips.remove(pos);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, cid);
        ctx.note_cards_lost(from, &[cid]);
        ctx.emit(EventData::CardsLost {
            player: from,
            cards: vec![cid],
            origins: vec![LostOrigin::Equip],
        });
        ctx.core.player_mut(to).equips.push(cid);
        crate::kernel::grant_equip_skills(ctx, to, cid);
        ctx.emit(EventData::CardsMoved {
            cards: vec![cid],
            from,
            to,
        });
    } else if let Some(pos) = ctx.core.player(from).judge.iter().position(|&x| x == cid) {
        ctx.core.player_mut(from).judge.remove(pos);
        ctx.note_cards_lost(from, &[cid]);
        ctx.emit(EventData::CardsLost {
            player: from,
            cards: vec![cid],
            origins: vec![LostOrigin::Judge],
        });
        ctx.core.player_mut(to).judge.push(cid);
        ctx.emit(EventData::CardsMoved {
            cards: vec![cid],
            from,
            to,
        });
    }
}

pub(super) fn qiangxi_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    let atk = core.attack_range(catalog, owner);
    alive_others(core, owner)
        .into_iter()
        .filter(|&t| core.computed_distance(catalog, owner, t) <= atk)
        .collect()
}

pub(super) fn owner_weapon_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    hand_and_equips(core, owner)
        .into_iter()
        .filter(|&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Weapon))
        .collect()
}

pub(super) fn miaobix_needs_target(kind: CardKind) -> bool {
    !matches!(
        kind,
        CardKind::Wuzhong
            | CardKind::Tao
            | CardKind::Jiu
            | CardKind::Wugu
            | CardKind::Taoyuan
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
    )
}

pub(super) fn miaobix_targets_for_kind(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
    carrier: CardId,
) -> Vec<PlayerId> {
    let nature = match kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    if kind.is_sha_like() {
        let atk = core.attack_range(catalog, owner);
        alive_others(core, owner)
            .into_iter()
            .filter(|&t| {
                core.computed_distance(catalog, owner, t) <= atk
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, owner, t, carrier, kind, nature,
                    )
            })
            .collect()
    } else {
        match kind {
            CardKind::Duel => alive_others(core, owner)
                .into_iter()
                .filter(|&t| {
                    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        core, catalog, owner, t, carrier, kind, nature,
                    )
                })
                .collect(),
            CardKind::Dismantle => alive_others(core, owner)
                .into_iter()
                .filter(|&t| {
                    (!core.player(t).hand.is_empty()
                        || !core.player(t).equips.is_empty()
                        || !core.player(t).judge.is_empty())
                        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            core, catalog, owner, t, carrier, kind, nature,
                        )
                })
                .collect(),
            CardKind::Snatch => alive_others(core, owner)
                .into_iter()
                .filter(|&t| {
                    core.computed_distance(catalog, owner, t) <= 1
                        && (!core.player(t).hand.is_empty() || !core.player(t).equips.is_empty())
                        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            core, catalog, owner, t, carrier, kind, nature,
                        )
                })
                .collect(),
            CardKind::Chain => alive_all(core, owner),
            _ => alive_others(core, owner),
        }
    }
}

pub(super) fn miaobix_can_use_kind(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
) -> bool {
    if !is_basic_or_normal_trick_kind(kind)
        || kind == CardKind::Shan
        || kind == CardKind::Nullify
    {
        return false;
    }
    if !miaobix_needs_target(kind) {
        return match kind {
            CardKind::Tao => core.player(owner).hp < core.player(owner).max_hp as i32,
            CardKind::Jiu => !core.turn.jiu_active,
            CardKind::SavageAttack | CardKind::ArcheryAttack => !alive_others(core, owner).is_empty(),
            _ => true,
        };
    }
    let sample_cid = core.player(owner).hand.first().copied().unwrap_or(0);
    !miaobix_targets_for_kind(core, catalog, owner, kind, sample_cid).is_empty()
}

// ── Public Entry Points ─────────────────────────────────────────────────
pub(in crate::content::host) fn trigger_meta_u08_batch_d(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "bolehuiyan" => Some(meta(
            EventKind::CardUsed,
            vec![
                EventKind::Damage,
                EventKind::ShaDodged,
                EventKind::TurnBegin,
                EventKind::TurnEnded,
            ],
        )),
        "bolehuiyan2" => Some(meta(EventKind::ShaDodged, vec![EventKind::TurnEnded])),
        "bolehuiyan3" => Some(meta(EventKind::Damage, Vec::new())),
        "oldbolehuiyan" => Some(meta(EventKind::PhaseBegin, vec![EventKind::OrdinaryJudge])),
        "oldbolehuiyan2" => Some(meta(EventKind::OrdinaryJudge, vec![EventKind::PhaseEnd])),
        "yxsweiyi" => Some(meta(EventKind::Damage, Vec::new())),
        "nvquan" | "nvquan1" | "nvquan2" | "feigong" | "cike" => {
            Some(meta(EventKind::CardUsed, Vec::new()))
        }
        "jianai" => Some(meta(EventKind::Recover, Vec::new())),
        "jieyong" | "zhulu" => Some(meta(
            EventKind::CardUsed,
            vec![EventKind::TurnBegin, EventKind::TurnEnded],
        )),
        "jieyong2" | "jieyong3" => Some(meta(EventKind::TurnBegin, vec![EventKind::TurnEnded])),
        "qiangyun" => Some(meta(EventKind::CardsLost, vec![EventKind::CardsDiscarded])),
        "zhexian" => Some(meta(
            EventKind::CardsLost,
            vec![
                EventKind::CardsDiscarded,
                EventKind::TurnBegin,
                EventKind::TurnEnded,
            ],
        )),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_u08_batch_d(name: &str) -> SkillLimits {
    match name {
        "qiandu" | "xieling" | "qiangxi" | "miaobi" | "miaobix" | "guifu" | "lshengong" => {
            SkillLimits {
                once_per_turn: true,
                ..Default::default()
            }
        }
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_u08_batch_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "bolehuiyan" => match event {
            EventData::CardUsed(u) => {
                flag(ctx.state, owner, "bolehuiyan_used_turn") == 0
                    && u.player != owner
                    && is_sha_use(ctx.catalog, u)
                    && !u.targets.contains(&owner)
                    && u.targets
                        .first()
                        .is_some_and(|&t| !ctx.state.player(t).hand.is_empty())
            }
            EventData::Damage(d) => {
                let pred = flag(ctx.state, owner, "bolehuiyan_pred");
                let atk = flag(ctx.state, owner, "bolehuiyan_attacker");
                let tgt = flag(ctx.state, owner, "bolehuiyan_target");
                pred > 0
                    && !d.prevented
                    && d.amount > 0
                    && (atk == 0 || d.source == Some((atk - 1) as PlayerId))
                    && (tgt == 0 || u32::from(d.target) + 1 == tgt)
            }
            EventData::ShaDodged { source, target, .. } => {
                let pred = flag(ctx.state, owner, "bolehuiyan_pred");
                let atk = flag(ctx.state, owner, "bolehuiyan_attacker");
                let tgt = flag(ctx.state, owner, "bolehuiyan_target");
                pred > 0
                    && (atk == 0 || u32::from(*source) + 1 == atk)
                    && (tgt == 0 || u32::from(*target) + 1 == tgt)
            }
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. } => {
                flag(ctx.state, owner, "bolehuiyan_used_turn") > 0
                    || flag(ctx.state, owner, "bolehuiyan_pred") > 0
            }
            _ => false,
        },
        "bolehuiyan2" => match event {
            EventData::ShaDodged { .. } | EventData::TurnEnded { .. } => {
                flag(ctx.state, owner, "bolehuiyan_pred") > 0
            }
            _ => false,
        },
        "bolehuiyan3" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented && d.amount > 0 && flag(ctx.state, owner, "bolehuiyan_pred") > 0
        ),
        "oldbolehuiyan" => match event {
            EventData::Phase {
                player,
                phase: Phase::Judge,
                begin: true,
            } => *player != owner && !ctx.state.player(*player).judge.is_empty(),
            EventData::OrdinaryJudge {
                owner: j_owner,
                invocation,
            } => {
                *j_owner != owner
                    && *invocation != u32::MAX
                    && (*invocation as usize) < ctx.catalog.cards.len()
            }
            _ => false,
        },
        "oldbolehuiyan2" => match event {
            EventData::OrdinaryJudge {
                owner: j_owner,
                invocation,
            } => {
                *j_owner != owner
                    && *invocation != u32::MAX
                    && (*invocation as usize) < ctx.catalog.cards.len()
                    && flag(ctx.state, owner, "oldbolehuiyan_pred") > 0
            }
            EventData::Phase {
                player,
                phase: Phase::Judge,
                begin: false,
            } => *player != owner && flag(ctx.state, owner, "oldbolehuiyan_pred") > 0,
            _ => false,
        },
        "yxsweiyi" => matches!(
            event,
            EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.is_some_and(|src| {
                        ctx.state.player(src).alive && !hand_and_equips(ctx.state, src).is_empty()
                    })
        ),
        "nvquan" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && (is_sha_use(ctx.catalog, u)
                        || u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog))
                            == CardKind::Duel)
                    && u.targets
                        .iter()
                        .any(|&t| ctx.state.player(t).alive && is_male(ctx.state, ctx.catalog, t))
        ),
        "nvquan1" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !has_skill_named(ctx.state, ctx.catalog, owner, "nvquan")
                    && is_sha_use(ctx.catalog, u)
                    && u.targets
                        .iter()
                        .any(|&t| ctx.state.player(t).alive && is_male(ctx.state, ctx.catalog, t))
        ),
        "nvquan2" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner
                    && !has_skill_named(ctx.state, ctx.catalog, owner, "nvquan")
                    && u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)) == CardKind::Duel
                    && u.targets
                        .iter()
                        .any(|&t| ctx.state.player(t).alive && is_male(ctx.state, ctx.catalog, t))
        ),
        "feigong" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player != owner
                    && !u.targets.contains(&owner)
                    && is_sha_use(ctx.catalog, u)
                    && ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .any(|&c| is_sha_card(ctx.state, ctx.catalog, c))
        ),
        "jianai" => matches!(
            event,
            EventData::Recover { player, amount, .. }
                if *player == owner
                    && *amount > 0
                    && alive_others(ctx.state, owner)
                        .into_iter()
                        .any(|p| ctx.state.player(p).hp < ctx.state.player(p).max_hp as i32)
        ),
        "jieyong" => match event {
            EventData::CardUsed(u) => {
                let Some(cid) = u.card else {
                    return false;
                };
                u.player == owner
                    && flag(ctx.state, owner, "jieyong2_used") == 0
                    && !has_skill_named(ctx.state, ctx.catalog, owner, "jieyong2")
                    && is_basic_or_normal_trick_kind(ctx.catalog.card_kind(cid))
                    && (ctx.state.discard.contains(&cid) || ctx.state.table.contains(&cid))
                    && hand_and_equips(ctx.state, owner)
                        .into_iter()
                        .any(|c| c != cid && is_black_card(ctx.state, ctx.catalog, owner, c))
            }
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. } => {
                flag(ctx.state, owner, "jieyong2_used") > 0
                    || has_skill_named(ctx.state, ctx.catalog, owner, "jieyong2")
            }
            _ => false,
        },
        "jieyong2" | "jieyong3" => matches!(
            event,
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. }
                if flag(ctx.state, owner, "jieyong2_used") > 0
                    || has_skill_named(ctx.state, ctx.catalog, owner, "jieyong2")
        ),
        "zhulu" => match event {
            EventData::CardUsed(u) => {
                let Some(cid) = u.card else {
                    return false;
                };
                let eff_kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                ctx.state.turn.current != owner
                    && u.player != owner
                    && flag(ctx.state, owner, "zhulu_used_turn") == 0
                    && is_normal_trick_kind(eff_kind)
                    && (ctx.state.discard.contains(&cid) || ctx.state.table.contains(&cid))
                    && {
                        let trick_suit =
                            ctx.catalog.effective_card_suit(ctx.state, u.player, cid);
                        hand_and_equips(ctx.state, owner).into_iter().any(|c| {
                            c != cid
                                && ctx.catalog.effective_card_suit(ctx.state, owner, c)
                                    == trick_suit
                        })
                    }
            }
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. } => {
                flag(ctx.state, owner, "zhulu_used_turn") > 0
            }
            _ => false,
        },
        "qiangyun" => match event {
            EventData::CardsLost {
                player,
                cards,
                origins,
            } => {
                *player == owner
                    && !cards.is_empty()
                    && origins.contains(&LostOrigin::Hand)
                    && ctx.state.player(owner).hand.is_empty()
                    && cards
                        .first()
                        .map_or(true, |&c| flag(ctx.state, owner, "qiangyun_declined_cid") != c + 1)
            }
            EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && ctx.state.player(owner).hand.is_empty()
                    && cards
                        .first()
                        .map_or(true, |&c| flag(ctx.state, owner, "qiangyun_declined_cid") != c + 1)
            }
            _ => false,
        },
        "cike" => matches!(
            event,
            EventData::CardUsed(u)
                if u.player == owner && is_sha_use(ctx.catalog, u) && !u.targets.is_empty()
        ),
        "zhexian" => match event {
            EventData::CardsLost { player, cards, .. }
            | EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && !cards.is_empty()
                    && ctx.state.turn.current != owner
                    && flag(ctx.state, owner, "zhexian_used_turn") == 0
                    && cards
                        .first()
                        .map_or(true, |&c| flag(ctx.state, owner, "zhexian_declined_cid") != c + 1)
            }
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. } => {
                flag(ctx.state, owner, "zhexian_used_turn") > 0
                    || flag(ctx.state, owner, "zhexian_declined_cid") > 0
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn activate_u08_batch_d(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    let state = ctx.state;
    let catalog = ctx.catalog;
    if !state.player(owner).alive {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "qiandu" => {
            state
                .player(owner)
                .hand
                .iter()
                .any(|&c| is_black_card(state, catalog, owner, c))
                && !alive_others(state, owner).is_empty()
        }
        "xieling" => {
            state.player(owner).hand.len() >= 2
                && !xieling_from_candidates(state, catalog, owner).is_empty()
        }
        "qiangxi" => !qiangxi_targets(state, catalog, owner).is_empty(),
        "miaobi" | "miaobix" => alive_others(state, owner)
            .into_iter()
            .any(|p| !hand_and_equips(state, p).is_empty()),
        "guifu" => alive_others(state, owner)
            .into_iter()
            .any(|p| !state.player(p).equips.is_empty()),
        "lshengong" => {
            !state.player(owner).hand.is_empty()
                && alive_all(state, owner)
                    .into_iter()
                    .any(|p| !state.player(p).equips.is_empty())
        }
        _ => false,
    };
    if !can_activate {
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

pub(in crate::content::host) fn machine_u08_batch_d(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("bolehuiyan", EventData::CardUsed(u)) => {
            card = Some(1);
            target = u.targets.first().copied();
            targets = vec![u.player];
        }
        ("bolehuiyan" | "bolehuiyan3", EventData::Damage(_)) => {
            card = Some(2);
        }
        ("bolehuiyan" | "bolehuiyan2", EventData::ShaDodged { .. }) => {
            card = Some(3);
        }
        ("bolehuiyan" | "bolehuiyan2", EventData::TurnBegin { .. } | EventData::TurnEnded { .. }) => {
            card = Some(4);
        }
        ("oldbolehuiyan", EventData::Phase { player, .. }) => {
            card = Some(1);
            target = Some(*player);
        }
        (
            "oldbolehuiyan" | "oldbolehuiyan2",
            EventData::OrdinaryJudge {
                owner: j_owner,
                invocation,
            },
        ) => {
            card = Some(2);
            target = Some(*j_owner);
            cards = vec![*invocation];
        }
        ("oldbolehuiyan2", EventData::Phase { player, .. }) => {
            card = Some(3);
            target = Some(*player);
        }
        ("yxsweiyi", EventData::Damage(d)) => {
            target = d.source;
        }
        ("nvquan" | "nvquan1" | "nvquan2", EventData::CardUsed(u)) => {
            target = u.targets.first().copied();
            targets = u.targets.clone();
            cards = vec![u.card.unwrap_or(0)];
        }
        ("feigong", EventData::CardUsed(u)) => {
            target = Some(u.player);
            cards = u.card.into_iter().collect();
        }
        ("jieyong", EventData::CardUsed(u)) => {
            card = Some(1);
            cards = u.card.into_iter().collect();
        }
        (
            "jieyong" | "jieyong2" | "jieyong3",
            EventData::TurnBegin { .. } | EventData::TurnEnded { .. },
        ) => {
            card = Some(2);
        }
        ("zhulu", EventData::CardUsed(u)) => {
            card = Some(1);
            target = Some(u.player);
            cards = u.card.into_iter().collect();
        }
        ("zhulu", EventData::TurnBegin { .. } | EventData::TurnEnded { .. }) => {
            card = Some(2);
        }
        ("qiangyun", EventData::CardsLost { cards: lost, .. })
        | ("qiangyun", EventData::CardsDiscarded { cards: lost, .. }) => {
            cards = lost.clone();
        }
        ("cike", EventData::CardUsed(u)) => {
            target = u.targets.first().copied();
            cards = vec![u.card.unwrap_or(0)];
        }
        ("zhexian", EventData::CardsLost { cards: lost, .. })
        | ("zhexian", EventData::CardsDiscarded { cards: lost, .. }) => {
            card = Some(1);
            cards = lost.clone();
        }
        ("zhexian", EventData::TurnBegin { .. } | EventData::TurnEnded { .. }) => {
            card = Some(2);
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

pub(in crate::content::host) fn handle_u08_batch_d(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    handle_u08_batch_d_impl(m, ctx)
}
