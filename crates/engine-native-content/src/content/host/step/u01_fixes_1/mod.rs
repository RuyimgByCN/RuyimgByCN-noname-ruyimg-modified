//! U01 Sub-batch 1（`3D精选` 前 20 技）上游等价修复模块（按职责拆分子文件，均低于有效行数上限）。

use super::super::*;
use super::u01_batch_a::{
    flag, hand_options, he_cards, he_options, set_flag, set_turn_flag, slot_occupied, turn_flag,
    u01_discard_one, u01_draw_now, ASK_KEY,
};
use super::u01_batch_b::is_male;
use crate::catalog::{CardKind, Catalog, EquipSlot, QueryCtx, SkillLimits, Suit, TriggerMeta};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, DamageNature, EventData, EventKind, LostOrigin, Phase, Timing};
use crate::ids::{CardId, EventId, PlayerId, SkillDefId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

mod asks_a;
mod asks_bcd;
mod helpers;
mod routing;
mod step_a1;
mod step_a2;
mod step_bcd;

pub(crate) use helpers::{can_use_sha_play_phase, consume_sha_play_phase, is_sha_legal_play_target};
pub(super) use asks_a::zhishu_valid_dests;
pub(super) use helpers::*;
pub(crate) use routing::*;
pub(crate) use step_a1::*;
pub(crate) use step_a2::*;
pub(crate) use step_bcd::*;

pub(crate) fn is_u01_fix_1_batch_a(name: &str) -> bool {
    matches!(
        name,
        "dddjuxian"
            | "dddzhishu"
            | "dddjungui"
            | "dddshilie"
            | "dddyeshen"
            | "dddqiaoduan"
            | "ddddiedang"
            | "dddanliu"
            | "dddguiying"
            | "dddbingjian"
            | "dddtongyu"
    )
}

pub(crate) fn is_u01_fix_1_batch_b(name: &str) -> bool {
    matches!(name, "dddzhilian" | "dddjijian" | "dddjijian_old")
}

pub(crate) fn is_u01_fix_1_batch_c(name: &str) -> bool {
    name == "dddfushi"
}

pub(crate) fn is_u01_fix_1_batch_d(name: &str) -> bool {
    matches!(name, "dddlangzhi" | "dddfuzong")
}

pub(super) fn l28_stamp(core: &GameStateCore) -> u32 {
    crate::rvm_bridge::modifiers::d34_l28_mods::l28_turn_stamp(core)
}

pub(super) fn is_scope4_used(core: &GameStateCore, owner: PlayerId, key: &str) -> bool {
    core.vm_aux.once.contains(&(owner, key.to_string(), 4))
}

pub(super) fn mark_scope4_used(core: &mut GameStateCore, owner: PlayerId, key: &str) {
    core.vm_aux.once.insert((owner, key.to_string(), 4));
}

pub(super) fn is_black(catalog: &Catalog, card: CardId) -> bool {
    matches!(
        catalog.cards[card as usize].suit,
        Suit::Spade | Suit::Club
    )
}

pub(super) fn effective_suit_bit(
    core: &GameStateCore,
    catalog: &Catalog,
    holder: PlayerId,
    card: CardId,
) -> u32 {
    match catalog.effective_card_suit(core, holder, card) {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 4,
        Suit::Diamond => 8,
    }
}

pub(super) fn effective_is_red(
    core: &GameStateCore,
    catalog: &Catalog,
    holder: PlayerId,
    card: CardId,
) -> bool {
    matches!(
        catalog.effective_card_suit(core, holder, card),
        Suit::Heart | Suit::Diamond
    )
}

pub(super) fn effective_distinct_suits(
    core: &GameStateCore,
    catalog: &Catalog,
    holder: PlayerId,
    cards: &[CardId],
) -> usize {
    cards
        .iter()
        .fold(0u32, |m, &c| m | effective_suit_bit(core, catalog, holder, c))
        .count_ones() as usize
}

pub(super) fn is_identity_lord(core: &GameStateCore, player: PlayerId) -> bool {
    matches!(core.mode, crate::state::GameMode::Identity { .. }) && core.is_lord(player)
}

pub(crate) fn finish_respond_park_pub(ctx: &mut Ctx<'_>, owner: PlayerId, ok: bool) -> Flow {
    let fid = flag(ctx.core, owner, "u01_respond_fid");
    if fid != 0 {
        set_flag(
            ctx.core,
            owner,
            &format!("u01_respond_result:{fid}"),
            u32::from(ok),
        );
    }
    Flow::Done
}

pub(super) fn wait_ask_fix(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, ask_id);
    if let Some((observer, spec)) = build_ask_fix(
        ask_id,
        machine.owner,
        machine.target,
        &machine.targets,
        machine.card,
        &machine.cards,
        ctx.core,
        ctx.catalog,
    ) {
        machine.stage = stage;
        Flow::WaitChoice { observer, spec }
    } else {
        Flow::Done
    }
}

pub(crate) fn active_ask_u01_fix(
    ask_id: u32,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let _ = stage;
    if matches!(ask_id, 1002 | 1032 | 1067 | 1070) {
        let tgt = if ask_id == 1070 { owner } else { target? };
        let suffix = if ask_id == 1002 {
            cards.get(1..)?
        } else if ask_id == 1067 {
            let rev_len = flag(core, owner, "dddlangzhi_rev_len") as usize;
            cards.get(rev_len..)?
        } else {
            cards
        };
        let mut hidden = suffix.to_vec();
        let mut hand = core.player(tgt).hand.clone();
        hidden.sort_unstable();
        hand.sort_unstable();
        if hidden != hand {
            return None;
        }
    }
    let (observer, spec) =
        build_ask_fix(ask_id, owner, target, targets, card, cards, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}

pub(super) fn build_ask_fix(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        1001..=1050 => {
            asks_a::build_ask_a_fix(ask_id, owner, target, targets, card, cards, core, catalog)
        }
        1051..=1099 => {
            asks_bcd::build_ask_bcd_fix(ask_id, owner, target, targets, card, cards, core, catalog)
        }
        _ => None,
    }
}

pub(super) fn u01_remove_owned_ex(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    card: CardId,
) -> Option<LostOrigin> {
    if ctx.core.player(owner).hand.contains(&card) {
        ctx.take_hand_card(owner, card);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![card],
            origins: vec![LostOrigin::Hand],
        });
        return Some(LostOrigin::Hand);
    }
    if ctx.core.player(owner).equips.contains(&card) {
        ctx.core.player_mut(owner).equips.retain(|x| *x != card);
        ctx.core.declared_slots.remove(&card);
        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, card);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![card],
            origins: vec![LostOrigin::Equip],
        });
        return Some(LostOrigin::Equip);
    }
    if ctx.core.player(owner).judge.contains(&card) {
        ctx.core.player_mut(owner).judge.retain(|x| *x != card);
        ctx.core.declared_kinds.remove(&card);
        ctx.emit(EventData::CardsLost {
            player: owner,
            cards: vec![card],
            origins: vec![LostOrigin::Judge],
        });
        return Some(LostOrigin::Judge);
    }
    None
}

pub(super) fn u01_give_card_ex(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    card: CardId,
    public: bool,
) {
    if u01_remove_owned_ex(ctx, from, card).is_none() {
        return;
    }
    ctx.core.player_mut(to).hand.push(card);
    ctx.emit(EventData::CardsMoved {
        from,
        to,
        cards: vec![card],
    });
    if public {
        ctx.log(LogEvent::CardsGivenPublic {
            from,
            to,
            cards: vec![card],
        });
    } else {
        ctx.log(LogEvent::CardsGiven {
            from,
            to,
            cards: vec![card],
        });
    }
}

pub(super) fn u01_discard_batch(ctx: &mut Ctx<'_>, player: PlayerId, cards: &[CardId]) {
    let mut removed = Vec::new();
    let mut origins = Vec::new();
    for &card in cards {
        if ctx.core.player(player).hand.contains(&card) {
            ctx.take_hand_card(player, card);
            ctx.to_discard(card);
            removed.push(card);
            origins.push(LostOrigin::Hand);
        } else if ctx.core.player(player).equips.contains(&card) {
            ctx.core.player_mut(player).equips.retain(|&x| x != card);
            ctx.core.declared_slots.remove(&card);
            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, player, card);
            ctx.to_discard(card);
            removed.push(card);
            origins.push(LostOrigin::Equip);
        } else if ctx.core.player(player).judge.contains(&card) {
            ctx.core.player_mut(player).judge.retain(|&x| x != card);
            ctx.core.declared_kinds.remove(&card);
            ctx.to_discard(card);
            removed.push(card);
            origins.push(LostOrigin::Judge);
        }
    }
    if removed.is_empty() {
        return;
    }
    ctx.note_cards_lost(player, &removed);
    ctx.emit(EventData::CardsLost {
        player,
        cards: removed.clone(),
        origins,
    });
    ctx.log(LogEvent::CardDiscarded {
        player,
        cards: removed.clone(),
    });
    ctx.emit(EventData::CardsDiscarded {
        player,
        cards: removed,
    });
}

pub(super) fn u01_swap_handcards_simultaneous(
    ctx: &mut Ctx<'_>,
    a: PlayerId,
    a_cards: &[CardId],
    b: PlayerId,
    b_cards: &[CardId],
) {
    let take_list = |ctx: &mut Ctx<'_>, holder: PlayerId, want: &[CardId]| -> (Vec<CardId>, Vec<LostOrigin>) {
        let mut got = Vec::new();
        let mut org = Vec::new();
        for &c in want {
            if ctx.core.player(holder).hand.contains(&c) {
                ctx.take_hand_card(holder, c);
                got.push(c);
                org.push(LostOrigin::Hand);
            } else if ctx.core.player(holder).equips.contains(&c) {
                ctx.core.player_mut(holder).equips.retain(|&x| x != c);
                ctx.core.declared_slots.remove(&c);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, holder, c);
                got.push(c);
                org.push(LostOrigin::Equip);
            }
        }
        (got, org)
    };
    let (taken_a, org_a) = take_list(ctx, a, a_cards);
    let (taken_b, org_b) = take_list(ctx, b, b_cards);
    if !taken_a.is_empty() {
        ctx.core.player_mut(b).hand.extend(taken_a.iter().copied());
        ctx.note_cards_lost(a, &taken_a);
        ctx.emit(EventData::CardsLost {
            player: a,
            cards: taken_a.clone(),
            origins: org_a,
        });
        ctx.emit(EventData::CardsMoved {
            from: a,
            to: b,
            cards: taken_a.clone(),
        });
        ctx.log(LogEvent::CardsGiven {
            from: a,
            to: b,
            cards: taken_a,
        });
    }
    if !taken_b.is_empty() {
        ctx.core.player_mut(a).hand.extend(taken_b.iter().copied());
        ctx.note_cards_lost(b, &taken_b);
        ctx.emit(EventData::CardsLost {
            player: b,
            cards: taken_b.clone(),
            origins: org_b,
        });
        ctx.emit(EventData::CardsMoved {
            from: b,
            to: a,
            cards: taken_b.clone(),
        });
        ctx.log(LogEvent::CardsGiven {
            from: b,
            to: a,
            cards: taken_b,
        });
    }
}

pub(crate) fn u01_card_use_after_hook(ctx: &mut Ctx<'_>, u: &CardUseData) {
    // 0. `dddtongyu` discard paid cards carried on table until Wugu finishes
    if u.use_as == Some(CardKind::Wugu) {
        let paid_n = ctx
            .core
            .vm_aux
            .flags
            .remove(&(u.player, "dddtongyu_paid_count".to_string()))
            .unwrap_or(0);
        for idx in 0..paid_n {
            if let Some(c) = ctx
                .core
                .vm_aux
                .flags
                .remove(&(u.player, format!("dddtongyu_paid:{idx}")))
            {
                ctx.core.table.retain(|&x| x != c);
                ctx.to_discard(c);
            }
        }
    }
    // 1. `dddtongyu` remaining Wugu cards after Wugu resolves (`wuguRemained`)
    if u.use_as == Some(CardKind::Wugu) {
        if let Some(frame) = ctx
            .core
            .vm_aux
            .flags
            .remove(&(u.player, "dddtongyu_rem_cards_frame".to_string()))
        {
            let prefix_rem = format!("dddtongyu_rem_card:{frame}:");
            let mut rem: Vec<CardId> = ctx
                .core
                .table
                .iter()
                .copied()
                .filter(|&c| {
                    ctx.core
                        .vm_aux
                        .flags
                        .contains_key(&(u.player, format!("{prefix_rem}{c}")))
                })
                .collect();
            ctx.core
                .vm_aux
                .flags
                .retain(|(p, k), _| !(*p == u.player && k.starts_with(&prefix_rem)));
            let prefix_ex = format!("dddtongyu_excluded:{frame}:");
            let excluded: Vec<PlayerId> = ctx
                .core
                .seat_order_from(u.player)
                .into_iter()
                .filter(|&p| {
                    ctx.core.player(p).alive
                        && ctx
                            .core
                            .vm_aux
                            .flags
                            .contains_key(&(u.player, format!("{prefix_ex}{p}")))
                })
                .collect();
            ctx.core
                .vm_aux
                .flags
                .retain(|(p, k), _| !(*p == u.player && k.starts_with(&prefix_ex)));
            if !rem.is_empty() && !excluded.is_empty() && ctx.core.player(u.player).alive {
                for &c in &rem {
                    ctx.core.table.retain(|&x| x != c);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((u.player, "dddtongyu_wugu_rem".to_string()))
                        .or_default()
                        .push(c);
                }
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Intrinsic {
                        frame: crate::resolution::NO_FRAME,
                    },
                    awaiting_answer: false,
                    machine: Box::new(HostMachine {
                        def_key: "dddtongyu".to_string(),
                        behavior: HostBehavior::U01DddBatchA,
                        owner: u.player,
                        target: None,
                        card: Some(2),
                        cards: rem,
                        targets: excluded,
                        stage: HostStage::Act,
                    }),
                });
            } else {
                for c in rem.drain(..) {
                    ctx.core.table.retain(|&x| x != c);
                    ctx.to_discard(c);
                }
            }
        }
    }
    // 2. `dddfushi` `useCardAfter` (`skill.js:3727-3864`)
    if !u.cancelled && ctx.core.player(u.player).alive {
        let eff_kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
        let owner = u.player;
        if eff_kind.is_basic_kind()
            && ctx
                .catalog
                .skill_by_name("dddfushi")
                .is_some_and(|def| ctx.core.has_skill(owner, def))
            && !ctx.catalog.is_tieji_sealed(ctx.core, owner)
            && turn_flag(ctx.core, owner, "dddfushi_used_turn", "dddfushi_stamp") == 0
        {
            let p1 = ctx
                .core
                .vm_aux
                .piles
                .get(&(owner, "dddmiaoxing_1".to_string()))
                .is_some_and(|v| !v.is_empty());
            let p2 = ctx
                .core
                .vm_aux
                .piles
                .get(&(owner, "dddmiaoxing_2".to_string()))
                .is_some_and(|v| !v.is_empty());
            if !ctx.core.player(owner).hand.is_empty() || p1 || p2 {
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Intrinsic {
                        frame: crate::resolution::NO_FRAME,
                    },
                    awaiting_answer: false,
                    machine: Box::new(HostMachine {
                        def_key: "dddfushi".to_string(),
                        behavior: HostBehavior::U01DddBatchC,
                        owner,
                        target: None,
                        card: None,
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }),
                });
            }
        }
    }
}

pub(super) fn maybe_trigger_fushi(ctx: &mut Ctx<'_>, owner: PlayerId, eff_kind: CardKind) {
    if !eff_kind.is_basic_kind() || !ctx.core.player(owner).alive {
        return;
    }
    if ctx
        .catalog
        .skill_by_name("dddfushi")
        .is_some_and(|def| ctx.core.has_skill(owner, def))
        && !ctx.catalog.is_tieji_sealed(ctx.core, owner)
        && turn_flag(ctx.core, owner, "dddfushi_used_turn", "dddfushi_stamp") == 0
    {
        let p1 = ctx
            .core
            .vm_aux
            .piles
            .get(&(owner, "dddmiaoxing_1".to_string()))
            .is_some_and(|v| !v.is_empty());
        let p2 = ctx
            .core
            .vm_aux
            .piles
            .get(&(owner, "dddmiaoxing_2".to_string()))
            .is_some_and(|v| !v.is_empty());
        if !ctx.core.player(owner).hand.is_empty() || p1 || p2 {
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "dddfushi".to_string(),
                    behavior: HostBehavior::U01DddBatchC,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
            });
        }
    }
}

pub(crate) fn u01_respond_skill_options(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    need: CardKind,
    is_use: bool,
) -> Vec<ChoiceOption> {
    let mut out = Vec::new();
    if catalog.is_tieji_sealed(core, player) {
        return out;
    }
    if need == CardKind::Tao && is_use {
        if let Some(def) = catalog.skill_by_name("dddxiaxing") {
            let round_tag = core.turn.round + 1000;
            if core.has_skill(player, def)
                && flag(core, player, "dddxiaxing_round") != round_tag
                && (!core.player(player).hand.is_empty() || !core.player(player).equips.is_empty())
            {
                out.push(ChoiceOption::Skill(def));
            }
        }
        return out;
    }
    if matches!(need, CardKind::Shan | CardKind::Sha) {
        if is_use {
            if let Some(def) = catalog.skill_by_name("dddxiaxing") {
                if core.has_skill(player, def) && flag(core, player, "dddxiaxing_xia") > 0 {
                    out.push(ChoiceOption::Skill(def));
                }
            }
        }
        if let Some(def) = catalog.skill_by_name("dddbingjian") {
            if core.has_skill(player, def)
                && turn_flag(core, player, "dddbingjian_used_count", "dddbingjian_stamp") < 2
                && core.player(player).hand.len() != 2
            {
                out.push(ChoiceOption::Skill(def));
            }
        }
        if is_use {
            if let Some(def) = catalog.skill_by_name("dddshilie") {
                if core.has_skill(player, def)
                    && turn_flag(core, player, "dddshilie_used_turn", "dddshilie_stamp") == 0
                    && !asks_a::dddshilie_unshown_hand(core, player).is_empty()
                    && asks_a::dddshilie_max_unshown_sum(core, catalog, player)
                        >= asks_a::dddshilie_threshold(core, player)
                {
                    out.push(ChoiceOption::Skill(def));
                }
            }
        }
    }
    out
}

pub(crate) fn u01_try_activate_respond_skill(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
    def: SkillDefId,
    need: CardKind,
    fid: EventId,
) -> Option<bool> {
    let name = ctx.catalog.skill_name(def);
    match name {
        "dddxiaxing" => {
            if need == CardKind::Tao {
                set_flag(ctx.core, player, "u01_respond_fid", fid);
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Trigger { frame: fid },
                    awaiting_answer: false,
                    machine: Box::new(HostMachine {
                        def_key: "dddxiaxing".to_string(),
                        behavior: HostBehavior::DddXiaxing,
                        owner: player,
                        target: None,
                        card: Some(1),
                        cards: Vec::new(),
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }),
                });
                return Some(false);
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(player, "dddxiaxing_xia".to_string()));
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddxiaxing",
                owner: player,
                presentation: None,
            });
            Some(true)
        }
        "dddbingjian" => {
            set_flag(ctx.core, player, "u01_respond_fid", fid);
            let mode_code = if need == CardKind::Shan {
                let vs_sha = ctx
                    .res
                    .stack
                    .last()
                    .is_some_and(|f| matches!(&f.data, EventData::CardUsed(u) if u.effective_kind(ctx.catalog).is_sha_like()));
                if vs_sha { 10 } else { 12 }
            } else {
                11
            };
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger { frame: fid },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "dddbingjian".to_string(),
                    behavior: HostBehavior::U01DddBatchA,
                    owner: player,
                    target: None,
                    card: Some(mode_code),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
            });
            Some(false)
        }
        "dddshilie" => {
            set_flag(ctx.core, player, "u01_respond_fid", fid);
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger { frame: fid },
                awaiting_answer: false,
                machine: Box::new(HostMachine {
                    def_key: "dddshilie".to_string(),
                    behavior: HostBehavior::U01DddBatchA,
                    owner: player,
                    target: None,
                    card: Some(10),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }),
            });
            Some(false)
        }
        _ => None,
    }
}
