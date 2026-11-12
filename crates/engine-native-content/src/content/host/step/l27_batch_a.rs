//! L27 选将美化 Batch A（16 个核心技能 + 3 个标记/死代码桩，`选将美化/extension.js`）。
//!
//! 覆盖项：
//! - 3 个标记/死代码桩：`baonvezhi_mark`、`rehuashen_mark`、`huamu`（`if(false)` 死代码）
//! - 16 个核心技能：
//!   1. `regushe`（王朗·鼓舌，`1215`）
//!   2. `kuangcai`（祢衡·狂才，`1340`）
//!   3. `sbtongye`（谋孙权·统业，`1444`）
//!   4. `baonvezhi`（董卓/牛辅·暴虐值，`1817`）
//!   5. `clandianzhan`（族钟会·点盏，`1867`）
//!   6. `rehuashen`（界左慈·化身，`1968`）
//!   7. `rexinsheng`（界左慈·新生，`2259`）
//!   8. `taoluan`（张让·滔乱，`2351`）
//!   9. `shouxi`（曹节·守玺，`2480`）
//!   10. `dinghan`（荀谌·定汉，`2562`）
//!   11. `quanji`（钟会·权计，`2645`）
//!   12. `potzhuangshi`（势魏延·壮誓，`3218`）
//!   13. `qimou`（魏延·奇谋，`5902`）
//!   14. `reqimou`（界魏延·奇谋，`6059`）
//!   15. `sbjianxiong`（谋曹操·奸雄，`6216`）
//!   16. `jinghe`（南华老仙·经合，`7568`）

use super::super::*;
use crate::catalog::{CardKind, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::ChoiceOption;
use crate::event::{EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

#[path = "l27_batch_a_asks.rs"]
mod l27_batch_a_asks;
pub(in crate::content::host) use l27_batch_a_asks::ask_l27_batch_a;
use l27_batch_a_asks::wait_ask;

pub(super) const ASK_REGUSHE_TARGETS: u32 = 10;
pub(super) const ASK_REGUSHE_PINDIAN_CARD: u32 = 11;
pub(super) const ASK_REGUSHE_TARGET_PINDIAN_CARD: u32 = 12;
pub(super) const ASK_REGUSHE_TARGET_DISCARD: u32 = 13;
pub(super) const ASK_REGUSHE_CONFIRM_DRAW: u32 = 14;
pub(super) const ASK_SBTONGYE_PREDICT: u32 = 20;
pub(super) const ASK_REHUASHEN_SKILL: u32 = 30;
pub(super) const ASK_REHUASHEN_ACTION: u32 = 31;
pub(super) const ASK_TAOLUAN_CARD: u32 = 40;
pub(super) const ASK_TAOLUAN_VIRTUAL: u32 = 41;
pub(super) const ASK_TAOLUAN_HELPER: u32 = 42;
pub(super) const ASK_TAOLUAN_HELPER_GIVE: u32 = 43;
pub(super) const ASK_SHOUXI_DECLARE: u32 = 50;
pub(super) const ASK_SHOUXI_ATTACKER_DISCARD: u32 = 51;
pub(super) const ASK_DINGHAN_ADJUST: u32 = 60;
pub(super) const ASK_QUANJI_CONFIRM: u32 = 70;
pub(super) const ASK_QUANJI_PUT_CARD: u32 = 71;
pub(super) const ASK_POTZHUANGSHI_DISCARD: u32 = 80;
pub(super) const ASK_POTZHUANGSHI_LOSE_HP: u32 = 81;
pub(super) const ASK_QIMOU_HP: u32 = 90;
pub(super) const ASK_REQIMOU_HP: u32 = 91;
pub(super) const ASK_SBJIANXIONG_INIT: u32 = 100;
pub(super) const ASK_SBJIANXIONG_CONFIRM: u32 = 101;
pub(super) const ASK_SBJIANXIONG_REMOVE_MARK: u32 = 102;
pub(super) const ASK_JINGHE_SHOW: u32 = 110;
pub(super) const ASK_JINGHE_TARGETS: u32 = 111;

pub(super) const REHUASHEN_SKILLS: [&str; 6] = [
    "yingzi",
    "biyue",
    "zhiheng",
    "qixi",
    "qingnang",
    "guose",
];

pub(super) const JINGHE_SKILL_POOL: [&str; 8] = [
    "releiji",
    "rebiyue",
    "mingce",
    "xinzhiyan",
    "yinbing",
    "huoqi",
    "guizhu",
    "xianshou",
];

pub(super) const TAOLUAN_KINDS: [CardKind; 12] = [
    CardKind::Sha,
    CardKind::Tao,
    CardKind::Jiu,
    CardKind::Wuzhong,
    CardKind::Dismantle,
    CardKind::Snatch,
    CardKind::Duel,
    CardKind::SavageAttack,
    CardKind::ArcheryAttack,
    CardKind::Taoyuan,
    CardKind::Wugu,
    CardKind::Chain,
];

pub(super) const SHOUXI_KINDS: [CardKind; 12] = [
    CardKind::Sha,
    CardKind::Shan,
    CardKind::Tao,
    CardKind::Jiu,
    CardKind::Wuzhong,
    CardKind::Dismantle,
    CardKind::Snatch,
    CardKind::Duel,
    CardKind::SavageAttack,
    CardKind::ArcheryAttack,
    CardKind::Chain,
    CardKind::Nullify,
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

pub(super) fn total_field_equips(core: &GameStateCore) -> u32 {
    core.players
        .iter()
        .filter(|p| p.alive)
        .map(|p| p.equips.len() as u32)
        .sum()
}

pub(super) fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 4,
        Suit::Diamond => 8,
    }
}

pub(super) fn dinghan_trick_bit(kind: CardKind) -> u32 {
    match kind {
        CardKind::Wuzhong => 0,
        CardKind::Dismantle => 1,
        CardKind::Snatch => 2,
        CardKind::Duel => 3,
        CardKind::SavageAttack => 4,
        CardKind::ArcheryAttack => 5,
        CardKind::Taoyuan => 6,
        CardKind::Wugu => 7,
        CardKind::Jiedao => 8,
        CardKind::Chain => 9,
        CardKind::Nullify => 10,
        CardKind::Le => 11,
        CardKind::Bing => 12,
        CardKind::ShanDian => 13,
        CardKind::HongShui => 14,
        _ => 15 + ((kind as u32) & 15),
    }
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

pub(super) fn grant_skill_named(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) -> Option<u16> {
    let zh_alt = match name {
        "yingzi" | "xinzhiyan" => Some("英姿"),
        "biyue" | "rebiyue" => Some("闭月"),
        "zhiheng" | "guizhu" => Some("制衡"),
        "qixi" | "yinbing" => Some("奇袭"),
        "qingnang" | "huoqi" | "xianshou" => Some("青囊"),
        "guose" => Some("国色"),
        "releiji" => Some("雷击"),
        "mingce" => Some("仁德"),
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

pub(super) fn nullify_current_card_on_owner(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    card: Option<CardId>,
) -> bool {
    let fid = ctx.current_event_frame_id();
    let mut matched_frame = false;
    let mut all_cancelled = false;
    if let Some(f) = ctx.res.frame_mut(fid) {
        if let EventData::CardUsed(u) = &mut f.data {
            matched_frame = true;
            u.targets.retain(|&t| t != owner);
            if u.targets.is_empty() {
                u.cancelled = true;
                all_cancelled = true;
            }
        }
    }
    if all_cancelled {
        if let Some(cid) = card {
            if let Some(pos) = ctx.core.table.iter().position(|&x| x == cid) {
                ctx.core.table.remove(pos);
                ctx.core.discard.push(cid);
            }
        }
    }
    matched_frame
}

pub(super) fn refund_once_used(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    if let Some(def) = ctx.catalog.skill_by_name(name) {
        if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
            inst.once_used = false;
        }
    }
}

pub(super) fn regushe_wins_turn(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "regushe_turn_stamp") == turn_stamp(core) {
        flag(core, owner, "regushe_wins_turn")
    } else {
        0
    }
}

pub(super) fn regushe_used_turn(core: &GameStateCore, owner: PlayerId) -> u32 {
    if flag(core, owner, "regushe_turn_stamp") == turn_stamp(core) {
        flag(core, owner, "regushe_used_turn")
    } else {
        0
    }
}

pub(in crate::content::host) fn trigger_meta_l27_batch_a(name: &str) -> Option<TriggerMeta> {
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "kuangcai" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "sbtongye" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "baonvezhi" | "rexinsheng" => Some(meta(EventKind::Damage, Vec::new())),
        "clandianzhan" | "shouxi" => Some(meta(EventKind::CardUsed, Vec::new())),
        "rehuashen" => Some(meta(EventKind::TurnBegin, vec![EventKind::PhaseBegin])),
        "dinghan" => Some(meta(EventKind::CardUsed, vec![EventKind::PhaseBegin])),
        "quanji" => Some(meta(
            EventKind::Damage,
            vec![EventKind::PhaseBegin, EventKind::PhaseEnd, EventKind::LoseHp],
        )),
        "potzhuangshi" => Some(meta(EventKind::PhaseBegin, vec![EventKind::PhaseEnd])),
        "qimou" | "reqimou" | "taoluan" => Some(meta(EventKind::PhaseEnd, Vec::new())),
        "sbjianxiong" => Some(meta(EventKind::Damage, vec![EventKind::TurnBegin])),
        "jinghe" => Some(meta(EventKind::TurnBegin, Vec::new())),
        _ => None,
    }
}

pub(in crate::content::host) fn limits_l27_batch_a(name: &str) -> SkillLimits {
    match name {
        "jinghe" => SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => SkillLimits::default(),
    }
}

pub(in crate::content::host) fn filter_l27_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> bool {
    if !ctx.state.player(owner).alive {
        return false;
    }
    match skill.name.as_str() {
        "kuangcai" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: true
            } if *player == owner
        ),
        "sbtongye" => match event {
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: true,
            } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "sbtongye_predicted") > 0,
            _ => false,
        },
        "baonvezhi" => matches!(
            event,
            EventData::Damage(d)
                if !d.prevented
                    && d.amount > 0
                    && (d.target == owner || d.source == Some(owner))
                    && flag(ctx.state, owner, "baonvezhi") < 5
        ),
        "clandianzhan" => match event {
            EventData::CardUsed(u) if u.player == owner && !u.cancelled => {
                if let Some(cid) = u.card {
                    let s_bit = suit_bit(ctx.catalog.cards[cid as usize].suit);
                    (flag(ctx.state, owner, "clandianzhan_suit_mask") & s_bit) == 0
                } else {
                    false
                }
            }
            _ => false,
        },
        "rehuashen" => match event {
            EventData::TurnBegin { .. } => flag(ctx.state, owner, "rehuashen_init") == 0,
            EventData::Phase {
                player,
                phase: Phase::Prepare | Phase::End,
                begin: true,
            } => *player == owner && flag(ctx.state, owner, "rehuashen_init") > 0,
            _ => false,
        },
        "rexinsheng" => matches!(
            event,
            EventData::Damage(d) if !d.prevented && d.amount > 0 && d.target == owner
        ),
        "taoluan" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false
            } if *player == owner && flag(ctx.state, owner, "taoluan_disabled_turn") > 0
        ),
        "shouxi" => match event {
            EventData::CardUsed(u) => {
                !u.cancelled
                    && u.targets.contains(&owner)
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && (flag(ctx.state, owner, "shouxi_declared_mask") & 0xFFF) != 0xFFF
            }
            _ => false,
        },
        "dinghan" => match event {
            EventData::CardUsed(u) => {
                let kind = u.effective_kind(ctx.catalog);
                let is_trick = !kind.is_basic_kind() && kind.equip_slot().is_none();
                !u.cancelled
                    && is_trick
                    && u.targets.contains(&owner)
                    && (flag(ctx.state, owner, "dinghan_recorded_mask")
                        & (1u32 << dinghan_trick_bit(kind)))
                        == 0
            }
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => *player == owner,
            _ => false,
        },
        "quanji" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "quanji_hp_decreased_play") == 0
                    && ctx.state.player(owner).hp >= flag(ctx.state, owner, "quanji_play_start_hp") as i32
            }
            EventData::Damage(d) => !d.prevented && d.amount > 0 && d.target == owner,
            EventData::LoseHp { player, amount } => {
                *player == owner
                    && *amount > 0
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == Phase::Play
            }
            _ => false,
        },
        "potzhuangshi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player == owner,
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: false,
            } => {
                *player == owner
                    && (flag(ctx.state, owner, "potzhuangshi_unblockable_x") > 0
                        || flag(ctx.state, owner, "potzhuangshi_damage_y") > 0)
            }
            _ => false,
        },
        "qimou" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false
            } if *player == owner && flag(ctx.state, owner, "qimou_x") > 0
        ),
        "reqimou" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::End,
                begin: false
            } if *player == owner && flag(ctx.state, owner, "reqimou_x") > 0
        ),
        "sbjianxiong" => match event {
            EventData::TurnBegin { player } => {
                *player == owner && flag(ctx.state, owner, "sbjianxiong_init") == 0
            }
            EventData::Damage(d) => !d.prevented && d.amount > 0 && d.target == owner,
            _ => false,
        },
        "jinghe" => matches!(
            event,
            EventData::TurnBegin { player }
                if *player == owner && flag(ctx.state, owner, "jinghe_active_round") > 0
        ),
        _ => false,
    }
}

pub(in crate::content::host) fn activate_l27_batch_a(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Box<dyn crate::catalog::SkillMachine>> {
    if !ctx.state.player(owner).alive {
        return None;
    }
    let can_activate = match skill.name.as_str() {
        "regushe" => {
            !ctx.state.player(owner).hand.is_empty()
                && alive_others(ctx.state, owner)
                    .into_iter()
                    .any(|p| !ctx.state.player(p).hand.is_empty())
                && regushe_used_turn(ctx.state, owner) <= regushe_wins_turn(ctx.state, owner)
                && flag(ctx.state, owner, "regushe_marks").max(flag(ctx.state, owner, "regushe")) < 7
        }
        "taoluan" => {
            !ctx.state.player(owner).dying
                && flag(ctx.state, owner, "taoluan_disabled_turn") == 0
                && !hand_and_equips(ctx.state, owner).is_empty()
                && (flag(ctx.state, owner, "taoluan_used_mask") & 0xFFF) != 0xFFF
                && !alive_others(ctx.state, owner).is_empty()
        }
        "qimou" => flag(ctx.state, owner, "qimou_used") == 0 && ctx.state.player(owner).hp > 0,
        "reqimou" => flag(ctx.state, owner, "reqimou_used") == 0 && ctx.state.player(owner).hp > 0,
        "jinghe" => !ctx.state.player(owner).hand.is_empty(),
        _ => false,
    };
    if !can_activate {
        return None;
    }
    Some(Box::new(HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L27BeautifyBatchA,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }))
}

pub(in crate::content::host) fn machine_l27_batch_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> HostMachine {
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match event {
        EventData::TurnBegin { player } => {
            target = Some(*player);
            card = Some(20);
        }
        EventData::Phase {
            phase,
            begin,
            ..
        } => {
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
        EventData::CardUsed(u) => {
            target = Some(u.player);
            card = u.card;
            targets = u.targets.clone();
            if let Some(k) = u.use_as {
                cards.push(k as u32);
            }
        }
        EventData::Damage(d) => {
            target = d.source;
            card = d.card;
            cards.push(d.amount.max(1) as u32);
            if d.target == owner {
                targets.push(owner);
            }
        }
        EventData::LoseHp { amount, .. } => {
            card = Some(30 + (*amount).max(1).min(20) as u32);
        }
        _ => {}
    }
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L27BeautifyBatchA,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l27_batch_a(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let ask_id = flag(ctx.core, owner, "l27_ask_id");
    if m.stage != HostStage::Act
        && ctx.answer().is_empty()
        && !l27_batch_a_asks::has_custom_cancel(ask_id)
    {
        if l27_batch_a_asks::is_first_cancellable(ask_id) {
            refund_once_used(ctx, owner, &m.def_key);
        }
        set_flag(ctx.core, owner, "l27_ask_id", 0);
        return Flow::Done;
    }
    match m.def_key.as_str() {
        "regushe" => step_regushe(m, ctx),
        "kuangcai" => step_kuangcai(m, ctx),
        "sbtongye" => step_sbtongye(m, ctx),
        "baonvezhi" => step_baonvezhi(m, ctx),
        "clandianzhan" => step_clandianzhan(m, ctx),
        "rehuashen" => step_rehuashen(m, ctx),
        "rexinsheng" => step_rexinsheng(m, ctx),
        "taoluan" => step_taoluan(m, ctx),
        "shouxi" => l27_batch_a_asks::step_shouxi(m, ctx),
        "dinghan" => l27_batch_a_asks::step_dinghan(m, ctx),
        "quanji" => l27_batch_a_asks::step_quanji(m, ctx),
        "potzhuangshi" => l27_batch_a_asks::step_potzhuangshi(m, ctx),
        "qimou" => l27_batch_a_asks::step_qimou(m, ctx),
        "reqimou" => l27_batch_a_asks::step_reqimou(m, ctx),
        "sbjianxiong" => l27_batch_a_asks::step_sbjianxiong(m, ctx),
        "jinghe" => l27_batch_a_asks::step_jinghe(m, ctx),
        "baonvezhi_mark" | "rehuashen_mark" | "huamu" => Flow::Done,
        _ => Flow::Done,
    }
}

fn step_regushe(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_REGUSHE_TARGETS),
        HostStage::TargetChoice => {
            let picked: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match *o {
                    ChoiceOption::Player(p) => Some(p),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            m.targets = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|p| picked.contains(p))
                .collect();
            let stamp = turn_stamp(ctx.core);
            if flag(ctx.core, owner, "regushe_turn_stamp") != stamp {
                set_flag(ctx.core, owner, "regushe_turn_stamp", stamp);
                set_flag(ctx.core, owner, "regushe_wins_turn", 0);
                set_flag(ctx.core, owner, "regushe_used_turn", 0);
            }
            let used = flag(ctx.core, owner, "regushe_used_turn") + 1;
            set_flag(ctx.core, owner, "regushe_used_turn", used);
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_REGUSHE_PINDIAN_CARD)
        }
        HostStage::ComparePlayerPick => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(c);
            m.cards.clear();
            m.target = m.targets.first().copied();
            wait_ask(
                m,
                ctx,
                HostStage::CompareTargetPick,
                ASK_REGUSHE_TARGET_PINDIAN_CARD,
            )
        }
        HostStage::CompareTargetPick => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.cards.push(c);
            if m.cards.len() < m.targets.len() {
                m.target = Some(m.targets[m.cards.len()]);
                return wait_ask(
                    m,
                    ctx,
                    HostStage::CompareTargetPick,
                    ASK_REGUSHE_TARGET_PINDIAN_CARD,
                );
            }
            let owner_cid = m.card.unwrap_or_default();
            let owner_rank = ctx.catalog.cards.get(owner_cid as usize).map_or(1, |d| d.rank);
            discard_cards(ctx, owner, &[owner_cid]);
            let mut losing_targets = Vec::new();
            let mut owner_losses = 0u32;
            let mut owner_won_any = false;
            for (i, &t) in m.targets.clone().iter().enumerate() {
                let t_cid = m.cards[i];
                let t_rank = ctx.catalog.cards.get(t_cid as usize).map_or(1, |d| d.rank);
                discard_cards(ctx, t, &[t_cid]);
                if owner_rank > t_rank {
                    owner_won_any = true;
                } else {
                    owner_losses += 1;
                }
                if owner_rank >= t_rank {
                    losing_targets.push(t);
                }
            }
            if owner_won_any {
                let wins = regushe_wins_turn(ctx.core, owner) + 1;
                set_flag(ctx.core, owner, "regushe_wins_turn", wins);
                set_flag(ctx.core, owner, "gushe_wins", wins);
            }
            if owner_losses > 0 {
                let cur_marks = flag(ctx.core, owner, "regushe_marks")
                    .max(flag(ctx.core, owner, "regushe"))
                    + owner_losses;
                set_flag(ctx.core, owner, "regushe_marks", cur_marks);
                set_flag(ctx.core, owner, "regushe", cur_marks);
                set_flag(ctx.core, owner, "raosuo", cur_marks);
                if cur_marks >= 7 {
                    set_flag(ctx.core, owner, "regushe_died", 1);
                    ctx.emit(EventData::Death {
                        player: owner,
                        killer: None,
                    });
                    return Flow::Done;
                }
            }
            m.targets = losing_targets;
            m.card = Some(owner_losses);
            set_flag(ctx.core, owner, "regushe_loser_idx", 0);
            advance_regushe_losers(m, ctx)
        }
        HostStage::CardsAsk => {
            let idx = flag(ctx.core, owner, "regushe_loser_idx") as usize;
            if let Some(&t) = m.targets.get(idx) {
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Card(c)) => discard_cards(ctx, t, &[c]),
                    _ => ctx.start_draw(owner, 1),
                }
            }
            set_flag(ctx.core, owner, "regushe_loser_idx", (idx + 1) as u32);
            advance_regushe_losers(m, ctx)
        }
        HostStage::DrawSettle => {
            if matches!(
                ctx.answer().first(),
                Some(ChoiceOption::Confirm | ChoiceOption::Tag(0))
            ) || !ctx.answer().is_empty()
            {
                ctx.start_draw(owner, 1);
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn advance_regushe_losers(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let mut idx = flag(ctx.core, owner, "regushe_loser_idx") as usize;
    while idx < m.targets.len() {
        let t = m.targets[idx];
        if ctx.core.player(t).alive && !hand_and_equips(ctx.core, t).is_empty() {
            m.target = Some(t);
            set_flag(ctx.core, owner, "regushe_loser_idx", idx as u32);
            return wait_ask(m, ctx, HostStage::CardsAsk, ASK_REGUSHE_TARGET_DISCARD);
        }
        ctx.start_draw(owner, 1);
        idx += 1;
    }
    set_flag(ctx.core, owner, "regushe_loser_idx", idx as u32);
    if m.card.unwrap_or(0) > 0 && ctx.core.player(owner).alive {
        wait_ask(m, ctx, HostStage::DrawSettle, ASK_REGUSHE_CONFIRM_DRAW)
    } else {
        set_flag(ctx.core, owner, "l27_ask_id", 0);
        Flow::Done
    }
}

fn step_kuangcai(_m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = _m.owner;
    let used_cards = ctx.core.turn.cards_used_play_phase > 0
        || flag(ctx.core, owner, "kuangcai_used_card_turn") > 0;
    let dealt_damage = ctx.core.player(owner).turn_damage > 0
        || flag(ctx.core, owner, "kuangcai_dealt_damage_turn") > 0;
    let mut delta = flag(ctx.core, owner, "kuangcai_delta") as i32;
    if !used_cards {
        delta -= 1;
    } else if dealt_damage {
        delta += 1;
    }
    set_flag(ctx.core, owner, "kuangcai_delta", delta as u32);
    Flow::Done
}

fn step_sbtongye(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(1) {
                let prev = flag(ctx.core, owner, "sbtongye_recorded_equips");
                let cur = total_field_equips(ctx.core);
                let pred = flag(ctx.core, owner, "sbtongye_prediction");
                let matched = (pred == 0 && cur > prev) || (pred == 1 && cur < prev);
                let mut ye = flag(ctx.core, owner, "sbtongye_ye").max(flag(ctx.core, owner, "sbtongye"));
                if matched {
                    ye = (ye + 1).min(2);
                } else {
                    ye = ye.saturating_sub(1);
                }
                set_flag(ctx.core, owner, "sbtongye_ye", ye);
                set_flag(ctx.core, owner, "sbtongye", ye);
                set_flag(ctx.core, owner, "sbtongye_predicted", 0);
                Flow::Done
            } else {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_SBTONGYE_PREDICT)
            }
        }
        HostStage::SelectNature => {
            let opt = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => u32::from(n) & 1,
                _ => 0,
            };
            let eq_count = total_field_equips(ctx.core);
            set_flag(ctx.core, owner, "sbtongye_prediction", opt);
            set_flag(ctx.core, owner, "sbtongye_recorded_equips", eq_count);
            set_flag(ctx.core, owner, "sbtongye_predicted", 1);
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_baonvezhi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let dmg = m.cards.first().copied().unwrap_or(1) as u32;
    let cur = (flag(ctx.core, owner, "baonvezhi") + dmg).min(5);
    set_flag(ctx.core, owner, "baonvezhi", cur);
    set_flag(ctx.core, owner, "baonvezhi_marks", cur);
    Flow::Done
}

fn step_clandianzhan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(cid) = m.card else {
        return Flow::Done;
    };
    let s_bit = suit_bit(ctx.catalog.cards[cid as usize].suit);
    let mut mask = flag(ctx.core, owner, "clandianzhan_suit_mask") | s_bit;
    if m.targets.len() == 1 {
        let t = m.targets[0];
        if ctx.core.player(t).alive && !ctx.core.player(t).linked {
            ctx.emit(EventData::Linked {
                player: t,
                linked: true,
            });
        }
    }
    ctx.start_draw(owner, 1);
    if (mask & 0xF) == 0xF {
        mask = 0;
        set_flag(ctx.core, owner, "clandianzhan_reset_count", flag(ctx.core, owner, "clandianzhan_reset_count") + 1);
    }
    set_flag(ctx.core, owner, "clandianzhan_suit_mask", mask);
    Flow::Done
}

fn step_rehuashen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "rehuashen_init") == 0 {
                set_flag(ctx.core, owner, "rehuashen_init", 1);
                let pool = flag(ctx.core, owner, "rehuashen_pool_count") + 3;
                set_flag(ctx.core, owner, "rehuashen_pool_count", pool);
                set_flag(ctx.core, owner, "rehuashen", pool);
                wait_ask(m, ctx, HostStage::SelectNature, ASK_REHUASHEN_SKILL)
            } else {
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_REHUASHEN_ACTION)
            }
        }
        HostStage::TargetChoice => {
            let act = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => n,
                _ => return Flow::Done,
            };
            if act == 0 {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_REHUASHEN_SKILL)
            } else {
                let rep = u32::from(act).clamp(1, 2);
                let prev_rep = flag(ctx.core, owner, "rehuashen_replaced_count");
                set_flag(ctx.core, owner, "rehuashen_replaced_count", prev_rep + rep);
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                Flow::Done
            }
        }
        HostStage::SelectNature => {
            let idx = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(n)) => (n as usize) % REHUASHEN_SKILLS.len(),
                _ => 0,
            };
            let prev_def = flag(ctx.core, owner, "rehuashen_granted_def");
            if prev_def > 0 {
                ctx.core.skills.remove(&(owner, (prev_def - 1) as u16));
            }
            if let Some(def) = grant_skill_named(ctx, owner, REHUASHEN_SKILLS[idx]) {
                set_flag(ctx.core, owner, "rehuashen_granted_def", u32::from(def) + 1);
            }
            set_flag(ctx.core, owner, "rehuashen_active_skill", (idx as u32) + 1);
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_rexinsheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let dmg = m.cards.first().copied().unwrap_or(1) as u32;
    let pool = flag(ctx.core, owner, "rehuashen_pool_count")
        .max(flag(ctx.core, owner, "rehuashen"))
        + dmg;
    set_flag(ctx.core, owner, "rehuashen_pool_count", pool);
    set_flag(ctx.core, owner, "rehuashen", pool);
    Flow::Done
}

fn step_taoluan(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            if m.card == Some(16) {
                set_flag(ctx.core, owner, "taoluan_disabled_turn", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_TAOLUAN_CARD)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.card = Some(c);
            wait_ask(m, ctx, HostStage::SelectNature, ASK_TAOLUAN_VIRTUAL)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let bit = (tag as usize).min(TAOLUAN_KINDS.len() - 1);
            let mask = flag(ctx.core, owner, "taoluan_used_mask") | (1u32 << bit);
            set_flag(ctx.core, owner, "taoluan_used_mask", mask);
            set_flag(ctx.core, owner, "taoluan_last_kind", (bit as u32) + 1);
            if let Some(c) = m.card {
                discard_cards(ctx, owner, &[c]);
            }
            match TAOLUAN_KINDS[bit] {
                CardKind::Tao if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 => {
                    ctx.recover(owner, 1, None);
                }
                CardKind::Jiu => {
                    ctx.core.turn.jiu_active = true;
                }
                CardKind::Wuzhong => {
                    ctx.start_draw(owner, 2);
                }
                _ => {}
            }
            m.cards = vec![bit as u32];
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_TAOLUAN_HELPER)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(helper)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(helper);
            if hand_and_equips(ctx.core, helper).is_empty() {
                set_flag(ctx.core, owner, "taoluan_disabled_turn", 1);
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
                set_flag(ctx.core, owner, "l27_ask_id", 0);
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::CardsAsk, ASK_TAOLUAN_HELPER_GIVE)
        }
        HostStage::CardsAsk => {
            let helper = m.target.unwrap_or(owner);
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Card(c)) => {
                    move_cards_to_hand(ctx, helper, owner, &[c]);
                }
                _ => {
                    set_flag(ctx.core, owner, "taoluan_disabled_turn", 1);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                }
            }
            set_flag(ctx.core, owner, "l27_ask_id", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}
