//! L28 雷霆万钧 Batch A（武将 1–9，`雷霆万钧/extension.js`）。
//!
//! 覆盖武将：
//! 1. `th_guanning`（管宁 `3/7`）：`th_dunshi`
//! 2. `th_lukai`（陆凯）：`th_bushi`、`th_zhongzhuang`
//! 3. `th_caohua`（曹华）：`th_caiyi`、`th_guili`
//! 4. `th_sunhanhua`（孙寒华）：`th_chongxu`、`miaojian`、`shhlianhua`
//! 5. `th_puyuan`（蒲元）：`th_shengong`、`olqisi`
//! 6. `th_pangdegong`（庞德公）：`th_pingcai`、`xinfu_pdgyingshi`、`th_pcaudio_*`
//! 7. `th_zhengxuan`（郑玄）：`th_zhengjing`、`th_zhengjing2`
//! 8. `th_mamidi`（马日磾）：`th_chengye`、`th_buxu`
//! 9. `th_nanhualaoxian`（南华老仙）：`th_yufeng`、`th_tianshu`、`th_taipingyaoshu`、`th_g_taipingyaoshu_ai`

use super::super::*;
use crate::catalog::{CardKind, Catalog, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l28_batch_a_asks.rs"]
mod l28_batch_a_asks;
pub(in crate::content::host) use l28_batch_a_asks::ask_l28_batch_a;
use l28_batch_a_asks::wait_ask;

pub(super) const ASK_DUNSHI_BASIC: u32 = 10;
pub(super) const ASK_DUNSHI_SHA_TARGET: u32 = 11;
pub(super) const ASK_DUNSHI_RESPOND_SHAN: u32 = 12;
pub(super) const ASK_DUNSHI_DAMAGE_CHOICE: u32 = 13;
pub(super) const ASK_DUNSHI_SKILL: u32 = 14;
pub(super) const ASK_BUSHI_POSTURE: u32 = 20;
pub(super) const ASK_CAIYI_TARGET: u32 = 30;
pub(super) const ASK_CAIYI_OPTION: u32 = 31;
pub(super) const ASK_GUILI_TARGET: u32 = 32;
pub(super) const ASK_CHONGXU_ACTION: u32 = 40;
pub(super) const ASK_MIAOJIAN_TARGET: u32 = 41;
pub(super) const ASK_SHENGONG_DISCARD: u32 = 50;
pub(super) const ASK_SHENGONG_TARGET: u32 = 51;
pub(super) const ASK_OLQISI_SLOT: u32 = 52;
pub(super) const ASK_SHENGONG_FORGE: u32 = 53;
pub(super) const ASK_PINGCAI_RELIC: u32 = 60;
pub(super) const ASK_PINGCAI_TARGET: u32 = 61;
pub(super) const ASK_ZHENGJING_COUNT: u32 = 70;
pub(super) const ASK_ZHENGJING_TARGET: u32 = 71;
pub(super) const ASK_CHENGYE_TARGET: u32 = 80;
pub(super) const ASK_BUXU_DISCARD: u32 = 81;
pub(super) const ASK_BUXU_CHAMBER: u32 = 82;
pub(super) const ASK_YUFENG_SCORE: u32 = 90;
pub(super) const ASK_YUFENG_TARGET: u32 = 91;
pub(super) const ASK_TIANSHU_DISCARD: u32 = 92;
pub(super) const ASK_TIANSHU_TARGET: u32 = 93;

pub(super) const DUNSHI_SKILL_POOL: [&str; 6] = [
    "renwang",
    "jianxiong",
    "qingjian",
    "zhiheng",
    "wansha",
    "hongyan",
];

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

pub(super) fn move_cards_to_hand(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId, cards: &[CardId]) {
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

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    let zh_alt = match name {
        "renwang" => Some("仁德"),
        "jianxiong" => Some("奸雄"),
        "zhiheng" => Some("制衡"),
        "wansha" => Some("完杀"),
        "hongyan" => Some("红颜"),
        _ => None,
    };
    for candidate in [Some(name), zh_alt].into_iter().flatten() {
        if let Some(def) = ctx.catalog.skill_by_name(candidate) {
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
}

pub(super) fn is_taipingyaoshu(catalog: &Catalog, c: CardId) -> bool {
    catalog
        .cards
        .get(c as usize)
        .is_some_and(|cd| cd.name == "th_taipingyaoshu")
}

pub(super) fn taipingyaoshu_in_play(core: &GameStateCore, catalog: &Catalog) -> bool {
    core.players
        .iter()
        .any(|p| p.alive && p.equips.iter().any(|&c| is_taipingyaoshu(catalog, c)))
}

pub(super) fn has_non_spade_card(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> bool {
    hand_and_equips(core, p)
        .into_iter()
        .any(|c| catalog.cards[c as usize].suit != Suit::Spade)
}

pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    (core.turn.round.max(1) << 8) | u32::from(core.turn.current)
}

pub(super) fn buxu_cost(core: &GameStateCore, owner: PlayerId) -> usize {
    let used = if flag(core, owner, "th_buxu_turn_stamp") == turn_stamp(core) {
        flag(core, owner, "th_buxu_used_turn")
    } else {
        0
    };
    (1 + used) as usize
}

pub(super) fn bushi_suits(core: &GameStateCore, owner: PlayerId) -> (u32, u32, u32, u32) {
    if flag(core, owner, "th_bushi_suits_assigned") > 0 {
        (
            flag(core, owner, "th_bushi_s0") % 4,
            flag(core, owner, "th_bushi_s1") % 4,
            flag(core, owner, "th_bushi_s2") % 4,
            flag(core, owner, "th_bushi_s3") % 4,
        )
    } else {
        (0, 1, 2, 3)
    }
}

pub(super) fn caiyi_has_options(core: &GameStateCore, owner: PlayerId) -> bool {
    let key = if flag(core, owner, "th_caiyi_state") > 0 {
        "th_caiyi_yin_removed"
    } else {
        "th_caiyi_yang_removed"
    };
    (flag(core, owner, key) & 0b1111) != 0b1111
}

/// 主动技首问取消：退还本回合使用次数（`once_per_turn`），不产生任何效果。
pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn suit_index(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}

pub(super) fn chamber_of_card(catalog: &Catalog, c: CardId) -> u32 {
    let kind = catalog.card_kind(c);
    if kind.is_sha_like() {
        0
    } else if kind == CardKind::Shan {
        1
    } else if kind == CardKind::Tao {
        2
    } else if kind == CardKind::Jiu {
        3
    } else if kind.equip_slot().is_some() {
        4
    } else {
        5
    }
}

pub(super) fn equip_card_onto(ctx: &mut Ctx<'_>, target: PlayerId, card: CardId) {
    ctx.core.deck.retain(|&c| c != card);
    ctx.core.discard.retain(|&c| c != card);
    for p in 0..ctx.core.players.len() as PlayerId {
        ctx.core.player_mut(p).hand.retain(|&c| c != card);
        if ctx.core.player(p).equips.contains(&card) {
            ctx.core.player_mut(p).equips.retain(|&c| c != card);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, card);
        }
    }
    if let Some(slot) = ctx.catalog.card_slot_of(ctx.core, card) {
        let replaced = ctx
            .core
            .player(target)
            .equips
            .iter()
            .copied()
            .find(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(slot));
        if let Some(old) = replaced {
            ctx.core.player_mut(target).equips.retain(|&c| c != old);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, old);
            ctx.to_discard(old);
        }
        ctx.core.player_mut(target).equips.push(card);
        crate::kernel::grant_equip_skills(ctx, target, card);
    } else {
        ctx.core.player_mut(target).hand.push(card);
    }
}

pub(in crate::content::host) fn trigger_meta_l28_batch_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "th_dunshi" => Some(meta(
            EventKind::DamageBefore,
            vec![EventKind::Damage, EventKind::TurnBegin, EventKind::CardUsed],
        )),
        "th_bushi" => Some(meta(
            EventKind::PhaseBegin,
            vec![
                EventKind::CardUsed,
                EventKind::CardResponded,
                EventKind::Equipped,
                EventKind::Damage,
            ],
        )),
        "th_zhongzhuang" => Some(meta(EventKind::Damage, Vec::new())),
        "th_caiyi" | "xinfu_pdgyingshi" | "th_zhengjing2" | "th_yufeng" => {
            Some(meta(EventKind::PhaseBegin, Vec::new()))
        }
        "th_guili" => Some(meta(
            EventKind::TurnPhaseEnd,
            vec![
                EventKind::TurnBegin,
                EventKind::Damage,
                EventKind::TurnEnded,
                EventKind::PhaseEnd,
            ],
        )),
        "shhlianhua" => Some(meta(EventKind::CardUsed, vec![EventKind::Damage])),
        "olqisi" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnBegin])),
        "th_chengye" => Some(meta(EventKind::CardsDiscarded, vec![EventKind::PhaseBegin])),
        "th_taipingyaoshu" => Some(meta(EventKind::CardsLost, vec![EventKind::CardsMoved])),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l28_batch_a(name: &str) -> SkillLimits {
    match name {
        // th_dunshi / th_yufeng 的出牌阶段每回合限一次由各自回合戳记，不走 once_per_turn（避免吞后续触发）；
        // th_buxu 为同回合递增弃牌张数（1, 2, 3...），不设 once_per_turn。
        "th_caiyi" | "th_chongxu" | "miaojian" | "th_shengong" | "th_pingcai" | "th_zhengjing"
        | "th_chengye" | "th_tianshu" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_l28_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "th_dunshi" => match event {
            EventData::TurnBegin { player } => *player == owner,
            EventData::CardUsed(u) => {
                u.targets.contains(&owner)
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && flag(ctx.state, owner, "th_dunshi_used_this_turn") == 0
                    && (flag(ctx.state, owner, "th_dunshi_removed_mask") & 2) == 0
            }
            // `extension.js:3702`：发动后，当前回合角色本回合下次造成伤害时由你选择两项。
            EventData::Damage(d) => {
                flag(ctx.state, owner, "th_dunshi_pending") > 0
                    && d.source == Some(ctx.state.turn.current)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(d.target).alive
            }
            _ => false,
        },
        "th_bushi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::End,
                begin: true,
            } => *player == owner,
            EventData::CardUsed(u) => {
                let (_, s1, s2, _) = bushi_suits(ctx.state, owner);
                let c_suit = u.card.map(|c| suit_index(ctx.catalog.cards[c as usize].suit));
                (u.player == owner && c_suit == Some(s1))
                    || (u.targets.contains(&owner) && c_suit == Some(s2))
            }
            EventData::CardResponded { card, player }
            | EventData::Equipped { card, player, .. } => {
                let (_, s1, _, _) = bushi_suits(ctx.state, owner);
                *player == owner && suit_index(ctx.catalog.cards[*card as usize].suit) == s1
            }
            _ => false,
        },
        "th_zhongzhuang" => matches!(
            event,
            EventData::Damage(d) if d.source == Some(owner) && !d.prevented && d.amount > 0
        ),
        "th_caiyi" => matches!(
            event,
            EventData::Phase { player, phase: Phase::End, begin: true }
                if *player == owner && caiyi_has_options(ctx.state, owner)
        ),
        "th_guili" => match event {
            EventData::TurnBegin { player } => {
                (*player == owner
                    && flag(ctx.state, owner, "th_guili_target") == 0
                    && !alive_others(ctx.state, owner).is_empty())
                    || (u32::from(*player) + 1 == flag(ctx.state, owner, "th_guili_target"))
            }
            EventData::Damage(d) => {
                d.source
                    .is_some_and(|s| u32::from(s) + 1 == flag(ctx.state, owner, "th_guili_target"))
                    && !d.prevented
                    && d.amount > 0
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            }
            | EventData::TurnEnded { player }
            | EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => {
                let gt = flag(ctx.state, owner, "th_guili_target");
                let r = ctx.state.turn.round.max(1);
                let target_no_dmg = u32::from(*player) + 1 == gt
                    && flag(ctx.state, owner, "th_guili_target_first_turn") > 0
                    && flag(ctx.state, owner, "th_guili_target_dealt_damage") == 0
                    && flag(ctx.state, owner, "th_guili_acted_round") != r;
                let owner_extra = *player == owner
                    && ((flag(ctx.state, owner, "th_caiyi_yang_used_turn") > 0
                        && flag(ctx.state, owner, "th_caiyi_yin_used_turn") > 0)
                        || flag(ctx.state, owner, "th_guili_pending_extra") > 0
                        || flag(ctx.state, owner, "th_guili_force") > 0);
                target_no_dmg || owner_extra
            }
            _ => false,
        },
        "shhlianhua" => match event {
            EventData::CardUsed(u) => {
                u.targets.contains(&owner) && u.effective_kind(ctx.catalog).is_sha_like()
            }
            EventData::Damage(d) => d.target == owner && !d.prevented && d.amount > 0,
            _ => false,
        },
        "olqisi" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && flag(ctx.state, owner, "th_olqisi_init_done") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "xinfu_pdgyingshi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::Judge | Phase::End,
                begin: true,
            } if *player == owner
        ),
        "th_zhengjing2" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true } if *player == owner
                && (ctx.state.vm_aux.piles.get(&(owner, "th_jing".to_string())).is_some_and(|p| !p.is_empty())
                    || flag(ctx.state, owner, "th_zhengjing_boom") > 0)
        ),
        "th_chengye" => match event {
            EventData::CardsDiscarded { player, cards } => {
                *player != owner
                    && !cards.is_empty()
                    && (flag(ctx.state, owner, "th_chengye_mask") & 0x3F) != 0x3F
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                *player == owner
                    && (flag(ctx.state, owner, "th_chengye_liujing_count") >= 6
                        || (flag(ctx.state, owner, "th_chengye_mask") & 0x3F) == 0x3F)
            }
            _ => false,
        },
        "th_yufeng" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Prepare, begin: true }
                if flag(ctx.state, *player, "th_yufeng_locked") > 0
                    || (*player == owner && flag(ctx.state, owner, "th_yufeng_locked_target") > 0)
        ),
        "th_tianshu" => matches!(
            event,
            EventData::Phase { player, phase: Phase::Play, begin: true }
                if *player == owner
                    && !taipingyaoshu_in_play(ctx.state, ctx.catalog)
                    && has_non_spade_card(ctx.state, ctx.catalog, owner)
        ),
        // 仅当【太平要术】本身离开装备区时触发（`extension.js:8968` `onLose`）：
        // 失去其他装备或手牌被转移不触发失去效果。
        "th_taipingyaoshu" => match event {
            EventData::CardsLost {
                player,
                cards,
                origins,
            } => {
                *player == owner
                    && cards.iter().zip(origins.iter()).any(|(&c, o)| {
                        *o == LostOrigin::Equip && is_taipingyaoshu(ctx.catalog, c)
                    })
            }
            EventData::CardsMoved { from, to, cards } => {
                *from == owner
                    && from != to
                    && cards.iter().any(|&c| {
                        is_taipingyaoshu(ctx.catalog, c)
                            && !ctx.state.player(owner).equips.contains(&c)
                    })
                    && flag(ctx.state, owner, "l28_taipingyaoshu_revoke_after_lost") > 0
            }
            _ => false,
        },
        _ => false,
    }
}

pub(in crate::content::host) fn activate_l28_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "th_dunshi" => {
            flag(ctx.state, owner, "th_dunshi_used_this_turn") == 0
                && (flag(ctx.state, owner, "th_dunshi_removed_mask") & 0b1111) != 0b1111
        }
        "th_caiyi" => caiyi_has_options(ctx.state, owner),
        "th_chongxu" | "th_pingcai" | "th_zhengjing" => true,
        "th_yufeng" => flag(ctx.state, owner, "th_yufeng_turn_stamp") != turn_stamp(ctx.state),
        "miaojian" => !alive_others(ctx.state, owner).is_empty(),
        "th_shengong" => hand_and_equips(ctx.state, owner)
            .into_iter()
            .any(|c| ctx.catalog.card_kind(c).equip_slot().is_some()),
        "th_buxu" => {
            (flag(ctx.state, owner, "th_chengye_mask") & 0x3F) != 0x3F
                && hand_and_equips(ctx.state, owner).len() >= buxu_cost(ctx.state, owner)
        }
        "th_tianshu" => {
            !taipingyaoshu_in_play(ctx.state, ctx.catalog)
                && has_non_spade_card(ctx.state, ctx.catalog, owner)
        }
        "th_chengye" => {
            flag(ctx.state, owner, "th_chengye_liujing_count") >= 6
                || (flag(ctx.state, owner, "th_chengye_mask") & 0x3F) == 0x3F
        }
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L28ThunderBatchA,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l28_batch_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match event {
        // 遁世：回合开始只清本回合计数（`targets` 标记），不弹询问。
        EventData::TurnBegin { .. } if skill.name == "th_dunshi" => targets.push(owner),
        EventData::TurnBegin { player } if skill.name == "th_guili" => {
            target = Some(*player);
            targets.push(0);
        }
        EventData::TurnBegin { .. } => target = Some(owner),
        // 遁世：成为【杀】目标时视为使用基本牌（与主动发动同一询问）。
        EventData::CardUsed(u) if skill.name == "th_dunshi" => card = u.card,
        EventData::Phase {
            player,
            phase: Phase::Prepare,
            ..
        } if skill.name == "th_yufeng" => {
            target = Some(*player);
            targets.push(*player);
        }
        EventData::Phase {
            phase: Phase::Prepare,
            ..
        } => target = Some(0),
        EventData::Phase {
            phase: Phase::Judge,
            ..
        } => target = Some(1),
        EventData::Phase {
            phase: Phase::Draw, ..
        } => target = Some(2),
        EventData::Phase {
            phase: Phase::Play, ..
        } => target = Some(3),
        EventData::Phase {
            player,
            phase: Phase::End,
            begin: false,
        } if skill.name == "th_guili" => {
            target = Some(*player);
            targets.push(2);
        }
        EventData::Phase {
            phase: Phase::End,
            ..
        } => target = Some(5),
        EventData::TurnEndWindow { player, .. } | EventData::TurnEnded { player }
            if skill.name == "th_guili" =>
        {
            target = Some(*player);
            targets.push(2);
        }
        EventData::CardUsed(u) => {
            target = if skill.name == "shhlianhua" || skill.name == "th_dunshi" {
                Some(u.player)
            } else {
                u.targets.first().copied()
            };
            card = u.card;
            targets.push(u.player);
        }
        EventData::CardResponded { card: c, player }
        | EventData::Equipped { card: c, player, .. } => {
            card = Some(*c);
            targets.push(*player);
        }
        EventData::Damage(d) if skill.name == "th_guili" => {
            target = d.source;
            targets.push(1);
        }
        EventData::Damage(d) => {
            target = Some(d.target);
            card = d.card;
        }
        EventData::CardsDiscarded {
            player,
            cards: disc,
        } => {
            target = Some(*player);
            cards = disc.clone();
        }
        EventData::CardsLost { cards: lost, .. } | EventData::CardsMoved { cards: lost, .. } => {
            cards = lost.clone();
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L28ThunderBatchA,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l28_batch_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage != HostStage::Act && ctx.answer().is_empty() {
        // 可取消询问被放弃：首问取消退还次数，其余阶段直接结束。
        if l28_batch_a_asks::is_first_cancellable(flag(ctx.core, owner, "l28_ask_id")) {
            refund_once_used(ctx, owner, &m.def_key);
        }
        set_flag(ctx.core, owner, "l28_ask_id", 0);
        return Flow::Done;
    }
    set_flag(ctx.core, owner, &format!("th_skill:{}", m.def_key), 1);
    match m.def_key.as_str() {
        "th_dunshi" => step_th_dunshi(m, ctx),
        "th_bushi" => step_th_bushi(m, ctx),
        "th_zhongzhuang" => {
            set_flag(ctx.core, owner, "th_zhongzhuang_mounted", 1);
            Flow::Done
        }
        "th_caiyi" => l28_batch_a_asks::step_th_caiyi(m, ctx),
        "th_guili" => l28_batch_a_asks::step_th_guili(m, ctx),
        "th_chongxu" => l28_batch_a_asks::step_th_chongxu(m, ctx),
        "miaojian" => l28_batch_a_asks::step_miaojian(m, ctx),
        "shhlianhua" => {
            set_flag(ctx.core, owner, "th_shhlianhua_ward", 1);
            ctx.start_draw(owner, 1);
            if flag(ctx.core, owner, "th_shhlianhua_level") >= 1 {
                if let Some(src) = m.target.filter(|&p| p != owner && ctx.core.player(p).alive) {
                    if let Some(&c) = hand_and_equips(ctx.core, src).first() {
                        move_cards_to_hand(ctx, src, owner, &[c]);
                    }
                }
            }
            Flow::Done
        }
        "th_shengong" => l28_batch_a_asks::step_th_shengong(m, ctx),
        "olqisi" => l28_batch_a_asks::step_olqisi(m, ctx),
        "th_pingcai" => l28_batch_a_asks::step_th_pingcai(m, ctx),
        "xinfu_pdgyingshi" => l28_batch_a_asks::step_xinfu_pdgyingshi(m, ctx),
        "th_pcaudio_wolong_card"
        | "th_pcaudio_fengchu_card"
        | "th_pcaudio_shuijing_card"
        | "th_pcaudio_xuanjian_card" => {
            set_flag(ctx.core, owner, &format!("th_pcaudio:{}", m.def_key), 1);
            Flow::Done
        }
        "th_zhengjing" => l28_batch_a_asks::step_th_zhengjing(m, ctx),
        "th_zhengjing2" => l28_batch_a_asks::step_th_zhengjing2(m, ctx),
        "th_chengye" => l28_batch_a_asks::step_th_chengye(m, ctx),
        "th_buxu" => l28_batch_a_asks::step_th_buxu(m, ctx),
        "th_yufeng" => l28_batch_a_asks::step_th_yufeng(m, ctx),
        "th_tianshu" => l28_batch_a_asks::step_th_tianshu(m, ctx),
        "th_g_taipingyaoshu_ai" => {
            set_flag(ctx.core, owner, "th_g_taipingyaoshu_ai_checked", 1);
            Flow::Done
        }
        "th_taipingyaoshu" => l28_batch_a_asks::step_th_taipingyaoshu(m, ctx),
        _ => Flow::Done,
    }
}

fn step_th_dunshi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if !m.targets.is_empty() {
                set_flag(ctx.core, owner, "th_dunshi_used_this_turn", 0);
                set_flag(ctx.core, owner, "th_dunshi_pending", 0);
                set_flag(ctx.core, owner, "th_dunshi_first_opt", 0);
                return Flow::Done;
            }
            if m.target.is_none() {
                return wait_ask(m, ctx, HostStage::SelectNature, ASK_DUNSHI_BASIC);
            }
            let damaged = m.target.unwrap_or(owner);
            set_flag(ctx.core, owner, "th_dunshi_pending", 0);
            set_flag(ctx.core, owner, "th_dunshi_first_opt", 0);
            if !ctx.core.player(damaged).alive {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_DUNSHI_DAMAGE_CHOICE)
        }
        HostStage::SelectNature => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Tag(tag)) = ans.first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, "th_dunshi_used_this_turn", 1);
            set_flag(ctx.core, owner, "th_dunshi_pending", 1);
            set_flag(ctx.core, owner, "th_dunshi_used_kind", u32::from(tag) + 1);
            match tag {
                0 => wait_ask(m, ctx, HostStage::ShuchuiTarget, ASK_DUNSHI_SHA_TARGET),
                1 => {
                    set_flag(ctx.core, owner, "th_dunshi_virtual_shan", 1);
                    Flow::Done
                }
                2 => {
                    if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                        ctx.recover(owner, 1, None);
                    }
                    Flow::Done
                }
                _ => {
                    ctx.core.turn.jiu_active = true;
                    Flow::Done
                }
            }
        }
        HostStage::ShuchuiTarget => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Player(t)) = ans.first().copied() else {
                return Flow::Done;
            };
            let boost = i32::from(ctx.core.turn.jiu_active);
            ctx.core.turn.jiu_active = false;
            ctx.emit(EventData::Damage(crate::machine::damage_data(
                Some(owner),
                t,
                1 + boost,
            )));
            Flow::Done
        }
        HostStage::TargetChoice => {
            let ans = ctx.answer().clone();
            let Some(ChoiceOption::Tag(choice)) = ans.first().copied() else {
                return Flow::Done;
            };
            let is_first = flag(ctx.core, owner, "th_dunshi_first_opt") == 0;
            set_flag(
                ctx.core,
                owner,
                "th_dunshi_first_opt",
                if is_first { u32::from(choice) + 1 } else { 0 },
            );
            match choice {
                0 => {
                    if ctx.current_damage().is_ok() {
                        ctx.current_damage_mut().prevented = true;
                    }
                    set_flag(ctx.core, owner, "th_dunshi_damage_prevented", 1);
                    return wait_ask(m, ctx, HostStage::HuanrenPick, ASK_DUNSHI_SKILL);
                }
                1 => {
                    ctx.lose_max_hp(owner, 1);
                    let x = flag(ctx.core, owner, "th_dunshi_removed_mask").count_ones();
                    if x > 0 {
                        ctx.start_draw(owner, x);
                    }
                }
                _ => {
                    let mut mask = flag(ctx.core, owner, "th_dunshi_removed_mask");
                    let used_kind = flag(ctx.core, owner, "th_dunshi_used_kind");
                    let bit = if (1..=4).contains(&used_kind)
                        && (mask & (1 << (used_kind - 1))) == 0
                    {
                        used_kind - 1
                    } else {
                        (0..4).find(|&b| (mask & (1 << b)) == 0).unwrap_or(0)
                    };
                    mask |= 1 << bit;
                    set_flag(ctx.core, owner, "th_dunshi_removed_mask", mask);
                }
            }
            if is_first {
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_DUNSHI_DAMAGE_CHOICE)
            } else {
                Flow::Done
            }
        }
        HostStage::HuanrenPick => {
            let ans = ctx.answer().clone();
            let idx = match ans.first().copied() {
                Some(ChoiceOption::Tag(t)) => (t as usize) % DUNSHI_SKILL_POOL.len(),
                _ => 0,
            };
            let cur_turn = ctx.core.turn.current;
            grant_skill_named(ctx, cur_turn, DUNSHI_SKILL_POOL[idx]);
            set_flag(ctx.core, owner, "th_dunshi_gained_skill", (idx + 1) as u32);
            if flag(ctx.core, owner, "th_dunshi_first_opt") > 0 {
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_DUNSHI_DAMAGE_CHOICE)
            } else {
                Flow::Done
            }
        }
        _ => Flow::Done,
    }
}

fn gain_suit_card(ctx: &mut Ctx<'_>, owner: PlayerId, suit_idx: u32) {
    if let Some(pos) = ctx
        .core
        .deck
        .iter()
        .position(|&c| suit_index(ctx.catalog.cards[c as usize].suit) == suit_idx)
    {
        let c = ctx.core.deck.remove(pos);
        ctx.core.player_mut(owner).hand.push(c);
    } else if let Some(pos) = ctx
        .core
        .discard
        .iter()
        .position(|&c| suit_index(ctx.catalog.cards[c as usize].suit) == suit_idx)
    {
        let c = ctx.core.discard.remove(pos);
        ctx.core.player_mut(owner).hand.push(c);
    } else {
        ctx.start_draw(owner, 1);
    }
}

fn step_th_bushi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => match m.target {
            Some(0) if m.card.is_none() => {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_BUSHI_POSTURE)
            }
            Some(5) if m.card.is_none() => {
                let (_, _, _, s3) = bushi_suits(ctx.core, owner);
                gain_suit_card(ctx, owner, s3);
                Flow::Done
            }
            _ => {
                let (_, s1, s2, s3) = bushi_suits(ctx.core, owner);
                let c_suit = m.card.map(|c| suit_index(ctx.catalog.cards[c as usize].suit));
                if m.targets.first().copied() == Some(owner) && c_suit == Some(s1) {
                    ctx.start_draw(owner, 2);
                }
                if m.target == Some(owner) && c_suit == Some(s2) {
                    gain_suit_card(ctx, owner, s3);
                }
                Flow::Done
            }
        },
        HostStage::SelectNature => {
            let ans = ctx.answer().clone();
            let s0 = match ans.first().copied() {
                Some(ChoiceOption::Tag(t)) => u32::from(t) % 4,
                _ => 0,
            };
            let (s1, s2, s3) = ((s0 + 1) % 4, (s0 + 2) % 4, (s0 + 3) % 4);
            set_flag(ctx.core, owner, "th_bushi_s0", s0);
            set_flag(ctx.core, owner, "th_bushi_s1", s1);
            set_flag(ctx.core, owner, "th_bushi_s2", s2);
            set_flag(ctx.core, owner, "th_bushi_s3", s3);
            set_flag(ctx.core, owner, "th_bushi_tao", s0);
            set_flag(ctx.core, owner, "th_bushi_sha", s1);
            set_flag(ctx.core, owner, "th_bushi_ying", s2);
            set_flag(ctx.core, owner, "th_bushi_zhi", s3);
            set_flag(ctx.core, owner, "th_bushi_unlimited_suit", s0 + 1);
            set_flag(ctx.core, owner, "th_bushi_suits_assigned", 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
