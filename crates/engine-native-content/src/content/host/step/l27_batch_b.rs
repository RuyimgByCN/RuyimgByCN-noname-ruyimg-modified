//! L27 选将美化 Batch B（其余 16 个核心技能 + 2 个辅助子技能，`选将美化/extension.js`）。
//!
//! 覆盖项：
//! 1. `bolan`（钟琰·博览，`7747`）与 `bolan_g`
//! 2. `drlt_duorui`（神张辽·夺锐，`8018`）与 `pro_drlt_duorui`（`8170`）
//! 3. `xinfu_xionghuo`（徐荣·凶镬，`8435`）
//! 4. `mbwuling`（神华佗·五灵，`8788`）
//! 5. `sbqingzheng`（谋曹丕·清正，`4263`）
//! 6. `spyanjiao`（SP 张燕·燕教，`4839`）
//! 7–16. 见 `l27_batch_b_tw.rs`（`twxiafeng`、`twxingzhui`、`xingchong`、`dcxiaoxi`、
//!   `twfengji`、`twharvestinori`、`twzhouhu`、`twzuhuo`、`twzhouzu`、`jishe2`、`new_zhixi_recover`）

use super::super::*;
use crate::catalog::{EquipSlot, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l27_batch_b_asks.rs"]
mod l27_batch_b_asks;
#[path = "l27_batch_b_tw.rs"]
mod l27_batch_b_tw;

pub(in crate::content::host) use l27_batch_b_asks::ask_l27_batch_b;
use l27_batch_b_asks::{target_skills_list, wait_ask};

pub(super) const ASK_BOLAN_SKILL: u32 = 201;
pub(super) const ASK_BOLAN_G_SKILL: u32 = 202;
pub(super) const ASK_DUORUI_SLOT: u32 = 210;
pub(super) const ASK_DUORUI_SKILL: u32 = 211;
pub(super) const ASK_PRO_DUORUI_SLOT: u32 = 212;
pub(super) const ASK_PRO_DUORUI_SKILL: u32 = 213;
pub(super) const ASK_XIONGHUO_TARGET: u32 = 220;
pub(super) const ASK_MBWULING_TARGET: u32 = 230;
pub(super) const ASK_MBWULING_ORDER: u32 = 231;
pub(super) const ASK_SBQINGZHENG_SUITS: u32 = 240;
pub(super) const ASK_SBQINGZHENG_TARGET: u32 = 241;
pub(super) const ASK_SBQINGZHENG_TARGET_SUIT: u32 = 242;
pub(super) const ASK_SBQINGZHENG_GAIN_MARK: u32 = 243;
pub(super) const ASK_SPYANJIAO_SUIT: u32 = 250;
pub(super) const ASK_SPYANJIAO_TARGET: u32 = 251;
pub(super) const ASK_TWXIAFENG_COUNT: u32 = 260;
pub(super) const ASK_TWXINGZHUI_TURNS: u32 = 270;
pub(super) const ASK_TWXINGZHUI_TARGET: u32 = 271;
pub(super) const ASK_XINGCHONG_DRAW: u32 = 280;
pub(super) const ASK_XINGCHONG_SHOW: u32 = 281;
pub(super) const ASK_DCXIAOXI_COUNT: u32 = 290;
pub(super) const ASK_DCXIAOXI_TARGET: u32 = 291;
pub(super) const ASK_DCXIAOXI_MODE: u32 = 292;
pub(super) const ASK_TWFENGJI_CARD: u32 = 300;
pub(super) const ASK_TWFENGJI_TURNS: u32 = 301;
pub(super) const ASK_TWHARVESTINORI_CARD: u32 = 310;
pub(super) const ASK_TWHARVESTINORI_TURNS: u32 = 311;
pub(super) const ASK_TWZHOUHU_CARD: u32 = 320;
pub(super) const ASK_TWZHOUHU_TURNS: u32 = 321;
pub(super) const ASK_TWZUHUO_CARD: u32 = 330;
pub(super) const ASK_TWZUHUO_TURNS: u32 = 331;
pub(super) const ASK_TWZHOUZU_TARGET: u32 = 340;
pub(super) const ASK_TWZHOUZU_TURNS: u32 = 341;

pub(super) const BOLAN_SKILL_POOL: [&str; 3] = ["zhiheng", "qiangxi", "quhu"];

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

pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    (core.turn.round.max(1) << 8) | u32::from(core.turn.current)
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

pub(super) fn baonvezhi_count(core: &GameStateCore, p: PlayerId) -> u32 {
    flag(core, p, "baonvezhi_marks").max(flag(core, p, "baonvezhi"))
}

pub(super) fn mbwuling_used_turn(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "mbwuling_turn_stamp") == turn_stamp(core) {
        flag(core, owner, "mbwuling_used_turn")
    } else {
        0
    }
}

pub(super) fn suit_tag_of(suit: Suit) -> u8 {
    match suit {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
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
    let zh_alt = match name {
        "zhiheng" => Some("制衡"),
        "qiangxi" | "quhu" => Some("奇袭"),
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
            return Some(def);
        }
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

pub(in crate::content::host) fn trigger_meta_l27_batch_b(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "bolan" => Some(meta(EventKind::PhaseBegin, vec![EventKind::PhaseEnd])),
        "drlt_duorui" | "pro_drlt_duorui" => {
            Some(meta(EventKind::Damage, vec![EventKind::PhaseEnd]))
        }
        "xinfu_xionghuo" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::TurnBegin, EventKind::PhaseEnd],
        )),
        "mbwuling" | "spyanjiao" | "twfengji" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "sbqingzheng" | "dcxiaoxi" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "twxiafeng" => Some(meta(EventKind::PhaseBegin, vec![EventKind::PhaseEnd])),
        "twxingzhui" | "twharvestinori" | "twzhouhu" | "twzuhuo" | "twzhouzu" | "jishe2" => {
            Some(meta(EventKind::PhaseEnd, Vec::new()))
        }
        "xingchong" => Some(meta(
            EventKind::TurnBegin,
            vec![
                EventKind::PhaseBegin,
                EventKind::CardsLost,
                EventKind::CardsDiscarded,
            ],
        )),
        "new_zhixi_recover" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l27_batch_b(name: &str) -> SkillLimits {
    match name {
        "bolan_g" | "spyanjiao" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_l27_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "bolan" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } => flag(ctx.state, *player, "bolan_temp_skill") > 0,
            _ => false,
        },
        "drlt_duorui" | "pro_drlt_duorui" => match event {
            EventData::Damage(d) => {
                !d.prevented
                    && d.amount > 0
                    && d.source == Some(owner)
                    && d.target != owner
                    && ctx.state.turn.current == owner
                    && ctx.state.player(d.target).alive
                    && (flag(ctx.state, owner, "drlt_duorui_sealed_slots_mask") & 0x1F) != 0x1F
                    && flag(ctx.state, d.target, "drlt_duorui_disabled_skill") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => flag(ctx.state, *player, "drlt_duorui_disabled_skill") > 0,
            _ => false,
        },
        "xinfu_xionghuo" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && flag(ctx.state, owner, "xinfu_xionghuo_init") == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player != owner && flag(ctx.state, *player, "xinfu_xionghuo_marked") > 0,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => {
                flag(ctx.state, *player, "xinfu_xionghuo_no_sha_turn") > 0
                    || flag(ctx.state, *player, "xinfu_xionghuo_hand_minus") > 0
            }
            _ => false,
        },
        "mbwuling" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if flag(ctx.state, *player, "mbwuling_spirit") > 0
        ),
        "sbqingzheng" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
                && !ctx.state.player(owner).hand.is_empty()
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| !ctx.state.player(p).hand.is_empty())
        ),
        "spyanjiao" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player == owner && flag(ctx.state, owner, "spyanjiao_draw_pending") > 0
        ),
        "twxiafeng" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner && baonvezhi_count(ctx.state, owner) > 0,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } => *player == owner && flag(ctx.state, owner, "twxiafeng_x") > 0,
            _ => false,
        },
        "twxingzhui" => matches!(
            event,
            EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            } if flag(ctx.state, owner, "twxingzhui_rem") > 0
        ),
        "xingchong" => match event {
            EventData::TurnBegin { player }
            | EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player == owner
                    && ctx.state.player(owner).max_hp > 0
                    && flag(ctx.state, owner, "xingchong_round_used")
                        != ctx.state.turn.round.max(1)
            }
            EventData::CardsLost { player, cards, .. }
            | EventData::CardsDiscarded { player, cards } => {
                *player == owner
                    && cards
                        .iter()
                        .any(|c| flag(ctx.state, owner, &format!("xingchong_card:{c}")) > 0)
            }
            _ => false,
        },
        "dcxiaoxi" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } if *player == owner
                && ctx.state.player(owner).max_hp > 1
                && !alive_others(ctx.state, owner).is_empty()
        ),
        "twfengji" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "twfengji_rem") == 0
                    && !hand_and_equips(ctx.state, owner).is_empty()
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "twfengji_rem") > 0,
            _ => false,
        },
        "twharvestinori" => matches!(
            event,
            EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            } if flag(ctx.state, owner, "twharvestinori_rem") > 0
        ),
        "twzhouhu" => matches!(
            event,
            EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            } if flag(ctx.state, owner, "twzhouhu_rem") > 0
        ),
        "twzuhuo" => matches!(
            event,
            EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            } if flag(ctx.state, owner, "twzuhuo_rem") > 0
        ),
        "twzhouzu" => matches!(
            event,
            EventData::Phase {
                phase: Phase::End,
                begin: false,
                ..
            } if flag(ctx.state, owner, "twzhouzu_rem") > 0
        ),
        "jishe2" | "new_zhixi_recover" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false,
            } if *player == owner
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_l27_batch_b(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "bolan" | "bolan_g" => ctx.state.player(owner).hp > 0,
        "xinfu_xionghuo" => {
            (flag(ctx.state, owner, "xinfu_xionghuo_init") == 0
                || flag(ctx.state, owner, "xinfu_xionghuo_marks") > 0)
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| flag(ctx.state, p, "xinfu_xionghuo_marked") == 0)
        }
        "mbwuling" => {
            mbwuling_used_turn(ctx.state, owner) < 2
                && alive_all(ctx.state, owner)
                    .into_iter()
                    .any(|p| flag(ctx.state, p, "mbwuling_spirit") == 0)
        }
        "spyanjiao" => {
            !ctx.state.player(owner).hand.is_empty()
                && !alive_others(ctx.state, owner).is_empty()
        }
        "twxingzhui" => {
            flag(ctx.state, owner, "twxingzhui_rem") == 0 && ctx.state.player(owner).hp > 0
        }
        "twharvestinori" => {
            flag(ctx.state, owner, "twharvestinori_rem") == 0
                && ctx.state.player(owner).hand.iter().any(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| matches!(cd.suit, Suit::Spade | Suit::Club))
                })
        }
        "twzhouhu" => {
            flag(ctx.state, owner, "twzhouhu_rem") == 0
                && ctx.state.player(owner).hand.iter().any(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| matches!(cd.suit, Suit::Heart | Suit::Diamond))
                })
        }
        "twzuhuo" => {
            flag(ctx.state, owner, "twzuhuo_rem") == 0
                && hand_and_equips(ctx.state, owner)
                    .into_iter()
                    .any(|c| !ctx.catalog.card_kind(c).is_basic_kind())
        }
        "twzhouzu" => {
            flag(ctx.state, owner, "twzhouzu_rem") == 0
                && !alive_others(ctx.state, owner).is_empty()
        }
        "jishe2" | "new_zhixi_recover" => true,
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L27BeautifyBatchB,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l27_batch_b(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    match event {
        EventData::TurnBegin { player } => {
            target = Some(*player);
            card = Some(20);
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            target = Some(*player);
            let phase_code = match phase {
                Phase::Prepare => 1u32,
                Phase::Judge => 2,
                Phase::Draw => 3,
                Phase::Play => 4,
                Phase::Discard => 5,
                Phase::End => 6,
            };
            card = Some(if *begin { phase_code } else { phase_code + 10 });
        }
        EventData::Damage(d) => {
            target = Some(d.target);
            card = Some(30);
            cards.push(d.amount.max(1) as u32);
        }
        EventData::CardsLost {
            cards: lost, ..
        }
        | EventData::CardsDiscarded { cards: lost, .. } => {
            card = Some(25);
            cards = lost.clone();
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L27BeautifyBatchB,
        owner,
        target,
        card,
        cards,
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l27_batch_b(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
) -> Flow {
    let owner = m.owner;
    let ask_id = flag(ctx.core, owner, "l27_b_ask_id");
    if m.stage != HostStage::Act
        && ctx.answer().is_empty()
        && !l27_batch_b_asks::has_custom_cancel(ask_id)
    {
        if l27_batch_b_asks::is_first_cancellable(ask_id) {
            refund_once_used(ctx, owner, &m.def_key);
        }
        set_flag(ctx.core, owner, "l27_b_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "bolan" | "bolan_g" => step_bolan(m, ctx),
        "drlt_duorui" | "pro_drlt_duorui" => step_drlt_duorui(m, ctx),
        "xinfu_xionghuo" => l27_batch_b_tw::step_xinfu_xionghuo(m, ctx),
        "mbwuling" => l27_batch_b_tw::step_mbwuling(m, ctx),
        "sbqingzheng" => l27_batch_b_tw::step_sbqingzheng(m, ctx),
        "spyanjiao" => l27_batch_b_tw::step_spyanjiao(m, ctx),
        "twxiafeng" => l27_batch_b_tw::step_twxiafeng(m, ctx),
        "twxingzhui" => l27_batch_b_tw::step_twxingzhui(m, ctx),
        "xingchong" => l27_batch_b_tw::step_xingchong(m, ctx),
        "dcxiaoxi" => l27_batch_b_tw::step_dcxiaoxi(m, ctx),
        "twfengji" => l27_batch_b_tw::step_twfengji(m, ctx),
        "twharvestinori" => l27_batch_b_tw::step_twharvestinori(m, ctx),
        "twzhouhu" => l27_batch_b_tw::step_twzhouhu(m, ctx),
        "twzuhuo" => l27_batch_b_tw::step_twzuhuo(m, ctx),
        "twzhouzu" => l27_batch_b_tw::step_twzhouzu(m, ctx),
        "jishe2" => l27_batch_b_tw::step_jishe2(m, ctx),
        "new_zhixi_recover" => l27_batch_b_tw::step_new_zhixi_recover(m, ctx),
        _ => Flow::Done,
    }
}

fn step_bolan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(14) {
                let p = m.target.unwrap_or(owner);
                let g_def = flag(ctx.core, p, "bolan_granted_def");
                if g_def > 0 {
                    ctx.core.skills.remove(&(p, (g_def - 1) as u16));
                    set_flag(ctx.core, p, "bolan_granted_def", 0);
                }
                set_flag(ctx.core, p, "bolan_temp_skill", 0);
                return Flow::Done;
            }
            if m.def_key == "bolan_g" || (m.card.is_none() && ctx.core.turn.current != owner) {
                let beneficiary = if m.def_key == "bolan_g" {
                    owner
                } else {
                    ctx.core.turn.current
                };
                m.target = Some(beneficiary);
                ctx.emit(EventData::LoseHp {
                    player: beneficiary,
                    amount: 1,
                });
                return wait_ask(m, ctx, HostStage::SelectNature, ASK_BOLAN_G_SKILL);
            }
            m.target = Some(owner);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_BOLAN_SKILL)
        }
        HostStage::SelectNature => {
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => (n as usize) % BOLAN_SKILL_POOL.len(),
                _ => return Flow::Done,
            };
            let beneficiary = m.target.unwrap_or(owner);
            if let Some(def) = grant_skill_named(ctx, beneficiary, BOLAN_SKILL_POOL[idx]) {
                set_flag(ctx.core, beneficiary, "bolan_granted_def", u32::from(def) + 1);
            }
            set_flag(ctx.core, beneficiary, "bolan_temp_skill", (idx as u32) + 1);
            set_flag(ctx.core, owner, "bolan_temp_skill", (idx as u32) + 1);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_drlt_duorui(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let is_pro = m.def_key == "pro_drlt_duorui";
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                let t = m.target.unwrap_or(owner);
                let d_def = flag(ctx.core, t, "drlt_duorui_disabled_def");
                if d_def > 0 {
                    if let Some(inst) = ctx.core.skills.get_mut(&(t, (d_def - 1) as u16)) {
                        inst.enabled = true;
                    }
                    set_flag(ctx.core, t, "drlt_duorui_disabled_def", 0);
                }
                set_flag(ctx.core, t, "drlt_duorui_disabled_skill", 0);
                let s_def = flag(ctx.core, owner, "drlt_duorui_stolen_def");
                if s_def > 0 {
                    ctx.core.skills.remove(&(owner, (s_def - 1) as u16));
                    set_flag(ctx.core, owner, "drlt_duorui_stolen_def", 0);
                }
                set_flag(ctx.core, owner, "drlt_duorui_stolen_skill", 0);
                return Flow::Done;
            }
            let ask_slot = if is_pro {
                ASK_PRO_DUORUI_SLOT
            } else {
                ASK_DUORUI_SLOT
            };
            wait_ask(m, ctx, HostStage::SelectCard, ask_slot)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Tag(slot)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let bit = (slot as u32).min(4);
            let mask = flag(ctx.core, owner, "drlt_duorui_sealed_slots_mask") | (1u32 << bit);
            set_flag(ctx.core, owner, "drlt_duorui_sealed_slots_mask", mask);
            let eq_slot = match bit {
                0 => EquipSlot::Weapon,
                1 => EquipSlot::Armor,
                2 => EquipSlot::HorseMinus,
                3 => EquipSlot::HorsePlus,
                _ => EquipSlot::Treasure,
            };
            let eq_to_disc: Vec<CardId> = ctx
                .core
                .player(owner)
                .equips
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(eq_slot))
                .collect();
            discard_cards(ctx, owner, &eq_to_disc);
            let ask_skill = if is_pro {
                ASK_PRO_DUORUI_SKILL
            } else {
                ASK_DUORUI_SKILL
            };
            wait_ask(m, ctx, HostStage::SelectNature, ask_skill)
        }
        HostStage::SelectNature => {
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n),
                _ => 0,
            };
            let t = m.target.unwrap_or(owner);
            let t_skills = target_skills_list(ctx.core, t);
            if let Some(&def) = t_skills.get(idx as usize) {
                if let Some(inst) = ctx.core.skills.get_mut(&(t, def)) {
                    inst.enabled = false;
                }
                set_flag(ctx.core, t, "drlt_duorui_disabled_def", u32::from(def) + 1);
                ctx.core.skills.insert(
                    (owner, def),
                    crate::state::SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
                set_flag(ctx.core, owner, "drlt_duorui_stolen_def", u32::from(def) + 1);
            } else if let Some(def) = grant_skill_named(ctx, owner, BOLAN_SKILL_POOL[(idx as usize) % 3]) {
                set_flag(ctx.core, owner, "drlt_duorui_stolen_def", u32::from(def) + 1);
            }
            set_flag(ctx.core, t, "drlt_duorui_disabled_skill", idx + 1);
            set_flag(ctx.core, owner, "drlt_duorui_stolen_skill", idx + 1);
            set_flag(ctx.core, owner, "l27_b_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
