//! L11-E2 `taixu` VM Fix B 阶段 / 觉醒 / 濒死 / 标记 / 调剂盐梅触发结算
//! （`qiangzhi` / `qiangzhi_draw` / `rejiangchi` / `repolu` / `rezhiyi` / `sbyangwei_counter` /
//! `twshengxi` / `xinfu_qinguo` / `xinfu_qinguo_recover` / `yinghun` / `zaiqi` / `zaoxian` /
//! `zhaohuo` / `zhengjian` / `zhengjian_mark` / `zhiyi` / `tiaojiyanmei`）。

use super::super::super::*;
use super::super::l11_common::*;
use super::super::l11_common_b2::{
    deliver_direct_cards_to_hand, emit_virtual_kind_use, pop_yonglve_deck_top,
};
use super::asks::{
    basic_kind_to_tag, can_use_virtual_basic, decode_hidden_or_card, reveal_to_all,
    tag_to_basic_kind, wait_vm_b_ask, ASK_QIANGZHI_CARD, ASK_QIANGZHI_TARGET,
    ASK_REJIANGCHI_DISCARD, ASK_REJIANGCHI_MODE, ASK_REPOLU_TARGETS, ASK_REZHIYI_KIND,
    ASK_REZHIYI_TARGET, ASK_TIAOJIYANMEI_DISCARD, ASK_TIAOJIYANMEI_GAIN_TARGET,
    ASK_TWSHEGXI_ZHINANG, ASK_TWSHEGXI_ZHUNBEI, ASK_XINFU_QINGUO_RECOVER, ASK_XINFU_QINGUO_SHA,
    ASK_YINGHUN_DISCARD, ASK_YINGHUN_MODE, ASK_YINGHUN_TARGET, ASK_ZHENGJIAN_TARGET,
    ASK_ZHIYI_MODE, ASK_ZHIYI_TARGET,
};
use crate::catalog::{CardKind, Catalog, SkillMachine, Suit};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature, DrawOutcome};
use crate::ids::{CardId, PlayerId};
use crate::kernel::NullifyWindowMachine;
use crate::machine::{Ctx, Flow};
use crate::resolution::{Park, Slot};
use crate::state::GameStateCore;

pub(super) fn is_tiaojiyanmei_name(name: &str) -> bool {
    name == "tiaojiyanmei" || name.starts_with("tiaojiyanmei_")
}

const ZHINANG_KINDS: [CardKind; 3] = [CardKind::Wuzhong, CardKind::Dismantle, CardKind::Nullify];

fn spawn_child_nullify(
    ctx: &mut Ctx<'_>,
    card_id: CardId,
    source: PlayerId,
    effect: Box<dyn SkillMachine>,
) {
    let fid = ctx.current_event_frame_id();
    let order = ctx.core.seat_order_from(source);
    ctx.res.parks.push(Park {
        resume_depth: ctx.res.stack.len(),
        slot: Slot::Intrinsic { frame: fid },
        awaiting_answer: false,
        machine: Box::new(NullifyWindowMachine::new(
            card_id,
            source,
            false,
            order,
            false,
            true,
            false,
            Some(effect),
        )),
    });
}

pub(super) fn card_category_code(kind: CardKind) -> u32 {
    if kind.is_basic_kind() {
        1
    } else if kind.equip_slot().is_some() {
        3
    } else {
        2
    }
}

pub(super) fn record_basic_turn_use_or_resp(
    core: &mut GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
) -> Option<u8> {
    let ts = turn_stamp(core);
    let tag = basic_kind_to_tag(kind)?;
    if catalog.skill_by_name("rezhiyi").is_some() {
        let prev_ts = flag(core, player, "l11_vm_b:rezhiyi_ts");
        let prev_mask = if prev_ts == ts {
            flag(core, player, "l11_vm_b:rezhiyi_mask")
        } else {
            0
        };
        set_flag(core, player, "l11_vm_b:rezhiyi_ts", ts);
        set_flag(core, player, "l11_vm_b:rezhiyi_mask", prev_mask | (1u32 << tag));
    }

    if is_unblocked_l11_owner(core, catalog, player)
        && has_named_skill(core, catalog, player, "zhiyi")
        && flag(core, player, "l11_vm_b:zhiyi_ts") != ts
    {
        set_flag(core, player, "l11_vm_b:zhiyi_ts", ts);
        return Some(tag);
    }
    None
}

pub(super) fn push_vm_b_park(
    ctx: &mut Ctx<'_>,
    def_key: &str,
    owner: PlayerId,
    target: Option<PlayerId>,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
) {
    ctx.res.parks.push(Park {
        resume_depth: ctx.res.stack.len(),
        slot: Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: def_key.to_string(),
            behavior: HostBehavior::L11VmFixB,
            owner,
            target,
            card: None,
            cards,
            targets,
            stage: HostStage::Act,
        }),
    });
}

pub(super) fn handle_qiangzhi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_QIANGZHI_TARGET),
        (HostStage::TargetChoice, Some(ASK_QIANGZHI_TARGET)) => {
            m.card = None;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            m.target = Some(t);
            wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_QIANGZHI_CARD)
        }
        (HostStage::SelectCard, Some(ASK_QIANGZHI_CARD)) => {
            m.card = None;
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(&opt) = ctx.answer().first() else {
                return Flow::Done;
            };
            let Some(cid) = decode_hidden_or_card(ctx.core, t, opt) else {
                return Flow::Done;
            };
            reveal_to_all(ctx, vec![cid]);
            let cat = card_category_code(ctx.catalog.resolved_card_kind(ctx.core, cid));
            set_flag(ctx.core, owner, "l11_vm_b:qiangzhi_cat", cat);
            ensure_perm_skill(ctx, owner, "qiangzhi_draw");
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_qiangzhi_draw(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    ctx.start_draw(m.owner, 1);
    Flow::Done
}

pub(super) fn handle_rejiangchi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_REJIANGCHI_MODE),
        (HostStage::SelectNature, Some(ASK_REJIANGCHI_MODE)) => {
            m.card = None;
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if tag == 0 {
                if he_cards(ctx.core, owner).is_empty() {
                    return Flow::Done;
                }
                wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_REJIANGCHI_DISCARD)
            } else {
                ctx.start_draw(owner, 1);
                ensure_perm_skill(ctx, owner, "rejiangchi3");
                Flow::Done
            }
        }
        (HostStage::SelectCard, Some(ASK_REJIANGCHI_DISCARD)) => {
            m.card = None;
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                return wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_REJIANGCHI_MODE);
            };
            discard_player_cards(ctx, owner, &[cid]);
            ensure_perm_skill(ctx, owner, "jiangchi2");
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_repolu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_REPOLU_TARGETS),
        (HostStage::TargetChoice, Some(ASK_REPOLU_TARGETS)) => {
            m.card = None;
            let picked = answered_players(ctx);
            if picked.is_empty() {
                return Flow::Done;
            }
            let cur = flag(ctx.core, owner, "l11_vm_b:repolu_cnt");
            let draw_n = cur + 1;
            set_flag(ctx.core, owner, "l11_vm_b:repolu_cnt", cur + 1);
            let ordered: Vec<PlayerId> = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|p| picked.contains(p) && ctx.core.player(*p).alive)
                .collect();
            for p in ordered.into_iter().rev() {
                ctx.start_draw(p, draw_n);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_rezhiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => {
            let ts = turn_stamp(ctx.core);
            let mask = if flag(ctx.core, owner, "l11_vm_b:rezhiyi_ts") == ts {
                flag(ctx.core, owner, "l11_vm_b:rezhiyi_mask")
            } else {
                0
            };
            m.cards = vec![mask];
            let has_usable = (0..=4u8).any(|t| {
                (mask & (1u32 << t)) != 0
                    && tag_to_basic_kind(t)
                        .is_some_and(|(k, nat)| can_use_virtual_basic(ctx.core, ctx.catalog, owner, k, nat, true, true))
            });
            if !has_usable {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            }
            wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_REZHIYI_KIND)
        }
        (HostStage::SelectNature, Some(ASK_REZHIYI_KIND)) => {
            m.card = None;
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            };
            let Some((kind, nat)) = tag_to_basic_kind(tag) else {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            };
            if matches!(kind, CardKind::Tao | CardKind::Jiu) {
                emit_virtual_kind_use(ctx, owner, kind, vec![owner], DamageNature::Normal);
                return Flow::Done;
            }
            let mask = m.cards.first().copied().unwrap_or(0);
            m.cards = vec![mask, u32::from(tag)];
            let _ = nat;
            wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_REZHIYI_TARGET)
        }
        (HostStage::TargetChoice, Some(ASK_REZHIYI_TARGET)) => {
            m.card = None;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            };
            let tag = m.cards.get(1).copied().unwrap_or(0) as u8;
            let (kind, nat) = tag_to_basic_kind(tag).unwrap_or((CardKind::Sha, DamageNature::Normal));
            emit_virtual_kind_use(ctx, owner, kind, vec![t], nat);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_sbyangwei_counter(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if flag(ctx.core, owner, "l11_vm_b:sbyangwei_cd_armed") == 0 {
        set_flag(ctx.core, owner, "l11_vm_b:sbyangwei_cd_armed", 1);
    } else {
        set_flag(ctx.core, owner, "l11_vm_b:sbyangwei_cd_armed", 0);
        remove_named_skill(ctx, owner, "sbyangwei_counter");
    }
    Flow::Done
}

pub(super) fn handle_twshengxi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.cards.first().copied() == Some(1) || m.card == Some(ASK_TWSHEGXI_ZHINANG) {
        // Phase::End 分支
        match (m.stage, m.card) {
            (HostStage::Act, None) => {
                let mut mask = 0u32;
                for (i, &k) in ZHINANG_KINDS.iter().enumerate() {
                    if ctx
                        .core
                        .deck
                        .iter()
                        .any(|&c| ctx.catalog.resolved_card_kind(ctx.core, c) == k)
                    {
                        mask |= 1u32 << i;
                    }
                }
                if mask == 0 {
                    ctx.start_draw(owner, 1);
                    return Flow::Done;
                }
                m.cards = vec![mask];
                wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_TWSHEGXI_ZHINANG)
            }
            (HostStage::SelectNature, Some(ASK_TWSHEGXI_ZHINANG)) => {
                m.card = None;
                if let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() {
                    if let Some(&want_kind) = ZHINANG_KINDS.get(tag as usize) {
                        if let Some(pos) = ctx
                            .core
                            .deck
                            .iter()
                            .rposition(|&c| ctx.catalog.resolved_card_kind(ctx.core, c) == want_kind)
                        {
                            let cid = ctx.core.deck.remove(pos);
                            deliver_direct_cards_to_hand(ctx, owner, &[cid]);
                        }
                    }
                }
                ctx.start_draw(owner, 1);
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        // Phase::Prepare 分支：从游戏外（detached_cards）获得【调剂盐梅】
        match (m.stage, m.card) {
            (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_TWSHEGXI_ZHUNBEI),
            (HostStage::TargetChoice, Some(ASK_TWSHEGXI_ZHUNBEI)) => {
                m.card = None;
                if is_confirmed(ctx) {
                    if let Some(pos) = ctx.core.detached_cards.iter().position(|&(c, _)| {
                        ctx.catalog
                            .cards
                            .get(c as usize)
                            .is_some_and(|d| is_tiaojiyanmei_name(d.name))
                    }) {
                        let (cid, _) = ctx.core.detached_cards.remove(pos);
                        deliver_direct_cards_to_hand(ctx, owner, &[cid]);
                    } else if let Some(pos) = ctx.core.deck.iter().position(|&c| {
                        ctx.catalog
                            .cards
                            .get(c as usize)
                            .is_some_and(|d| is_tiaojiyanmei_name(d.name))
                    }) {
                        let cid = ctx.core.deck.remove(pos);
                        deliver_direct_cards_to_hand(ctx, owner, &[cid]);
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

pub(super) fn handle_xinfu_qinguo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_XINFU_QINGUO_SHA),
        (HostStage::TargetChoice, Some(ASK_XINFU_QINGUO_SHA)) => {
            m.card = None;
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
                emit_virtual_kind_use(ctx, owner, CardKind::Sha, vec![t], DamageNature::Normal);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_xinfu_qinguo_recover(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_XINFU_QINGUO_RECOVER),
        (HostStage::TargetChoice, Some(ASK_XINFU_QINGUO_RECOVER)) => {
            m.card = None;
            if is_confirmed(ctx)
                && ctx.core.player(owner).alive
                && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
            {
                ctx.recover(owner, 1, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_yinghun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_YINGHUN_TARGET),
        (HostStage::TargetChoice, Some(ASK_YINGHUN_TARGET)) => {
            m.card = None;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            if lost <= 1 {
                m.cards = vec![1];
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(t, 1);
                return Flow::Yield;
            }
            wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_YINGHUN_MODE)
        }
        (HostStage::SelectNature, Some(ASK_YINGHUN_MODE)) => {
            m.card = None;
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let (draw_n, disc_n) = if tag == 0 {
                (lost, 1u32)
            } else {
                (1u32, lost)
            };
            m.cards = vec![disc_n];
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(t, draw_n);
            Flow::Yield
        }
        (HostStage::DrawSettle, _) => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            if he_cards(ctx.core, t).is_empty() {
                return Flow::Done;
            }
            wait_vm_b_ask(m, ctx, HostStage::CardsAsk, ASK_YINGHUN_DISCARD)
        }
        (HostStage::CardsAsk, Some(ASK_YINGHUN_DISCARD)) => {
            m.card = None;
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let picked = answered_cards(ctx);
                if !picked.is_empty() {
                    discard_player_cards(ctx, t, &picked);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_zaiqi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(fr) = ctx.res.stack.iter_mut().rev().find(|f| {
        matches!(
            f.data,
            EventData::DrawPhase {
                player,
                outcome: DrawOutcome::Running,
                ..
            } if player == owner
        )
    }) {
        if let EventData::DrawPhase {
            pending, outcome, ..
        } = &mut fr.data
        {
            *pending = 0;
            *outcome = DrawOutcome::Skipped;
        }
    }
    let count = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as usize;
    let mut shown = Vec::new();
    let mut hearts = 0u32;
    let mut gained = Vec::new();
    for _ in 0..count {
        let Some(cid) = pop_yonglve_deck_top(ctx) else {
            break;
        };
        shown.push(cid);
        if ctx.catalog.resolved_card_suit(ctx.core, cid) == Suit::Heart {
            hearts += 1;
            ctx.to_discard(cid);
        } else {
            gained.push(cid);
        }
    }
    reveal_to_all(ctx, shown);
    if hearts > 0 && ctx.core.player(owner).alive {
        ctx.recover(owner, hearts, None);
    }
    if !gained.is_empty() && ctx.core.player(owner).alive {
        deliver_direct_cards_to_hand(ctx, owner, &gained);
    }
    Flow::Done
}

pub(super) fn handle_zaoxian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if flag(ctx.core, owner, "l11_vm_b:zaoxian_used") > 0 {
        return Flow::Done;
    }
    set_flag(ctx.core, owner, "l11_vm_b:zaoxian_used", 1);
    ctx.lose_max_hp(owner, 1);
    ensure_perm_skill(ctx, owner, "jixi");
    Flow::Done
}

pub(super) fn handle_zhaohuo(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let lost = ctx.core.player(owner).max_hp.saturating_sub(1);
    if lost > 0 {
        ctx.lose_max_hp(owner, lost);
        ctx.start_draw(owner, lost);
    }
    Flow::Done
}

pub(super) fn handle_zhengjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.cards.first().copied() == Some(1) {
        // TurnBegin（phaseBegin）结算并清除全场 zhengjian_mark
        let order = ctx.core.seat_order_from(owner);
        let mut draws = Vec::new();
        for p in order {
            if ctx.core.player(p).alive && has_named_skill(ctx.core, ctx.catalog, p, "zhengjian_mark") {
                let cnt = flag(ctx.core, p, "l11_vm_b:zhengjian_cnt");
                let cap = cnt.min(5).min(ctx.core.player(p).max_hp);
                if cap > 0 {
                    draws.push((p, cap));
                }
            }
        }
        let seats = ctx.core.players.len() as PlayerId;
        for p in 0..seats {
            set_flag(ctx.core, p, "l11_vm_b:zhengjian_cnt", 0);
            remove_named_skill(ctx, p, "zhengjian_mark");
        }
        for (p, n) in draws.into_iter().rev() {
            ctx.start_draw(p, n);
        }
        return Flow::Done;
    }
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_ZHENGJIAN_TARGET),
        (HostStage::TargetChoice, Some(ASK_ZHENGJIAN_TARGET)) => {
            m.card = None;
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                if ctx.core.player(t).alive {
                    ensure_perm_skill(ctx, t, "zhengjian_mark");
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_zhengjian_mark(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let cur = flag(ctx.core, owner, "l11_vm_b:zhengjian_cnt");
    set_flag(ctx.core, owner, "l11_vm_b:zhengjian_cnt", cur + 1);
    Flow::Done
}

pub(super) fn handle_zhiyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => {
            let tag = m.cards.first().copied().unwrap_or(0) as u8;
            if tag >= 5 {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            }
            wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_ZHIYI_MODE)
        }
        (HostStage::SelectNature, Some(ASK_ZHIYI_MODE)) => {
            m.card = None;
            let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            };
            if mode == 0 {
                ctx.start_draw(owner, 1);
                return Flow::Done;
            }
            let tag = m.cards.first().copied().unwrap_or(0) as u8;
            let Some((kind, nat)) = tag_to_basic_kind(tag) else {
                return Flow::Done;
            };
            if !can_use_virtual_basic(ctx.core, ctx.catalog, owner, kind, nat, true, false) {
                return Flow::Done;
            }
            if matches!(kind, CardKind::Tao | CardKind::Jiu) {
                emit_virtual_kind_use(ctx, owner, kind, vec![owner], DamageNature::Normal);
                return Flow::Done;
            }
            wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_ZHIYI_TARGET)
        }
        (HostStage::TargetChoice, Some(ASK_ZHIYI_TARGET)) => {
            m.card = None;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let tag = m.cards.first().copied().unwrap_or(0) as u8;
            let (kind, nat) = tag_to_basic_kind(tag).unwrap_or((CardKind::Sha, DamageNature::Normal));
            if kind.is_sha_like() {
                set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
            }
            emit_virtual_kind_use(ctx, owner, kind, vec![t], nat);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(in crate::content::host) fn try_tiaojiyanmei_intrinsic(
    ctx: &mut Ctx<'_>,
    u: &CardUseData,
    effective_kind: CardKind,
) -> Option<Option<Box<dyn SkillMachine>>> {
    let cid = u.card?;
    if effective_kind != CardKind::Wuzhong
        || !ctx
            .catalog
            .resolved_card_def(ctx.core, cid)
            .is_some_and(|d| is_tiaojiyanmei_name(d.name))
    {
        return None;
    }
    let owner = u.player;
    let sum_h: u32 = u
        .targets
        .iter()
        .map(|&t| ctx.core.player(t).hand.len() as u32)
        .sum();
    set_flag(ctx.core, owner, &format!("l11_vm_b:tiaoji_avg2:{cid}"), sum_h);
    for &t in &u.targets {
        set_flag(
            ctx.core,
            t,
            &format!("l11_vm_b:tiaoji_h:{cid}"),
            ctx.core.player(t).hand.len() as u32,
        );
    }
    ctx.core
        .vm_aux
        .piles
        .remove(&(owner, format!("l11_vm_b:tiaoji_disc:{cid}")));
    let orig: Vec<u32> = u.targets.iter().map(|&p| u32::from(p)).collect();
    Some(Some(Box::new(HostMachine {
        def_key: "tiaojiyanmei__loop".to_string(),
        behavior: HostBehavior::L11VmFixB,
        owner,
        target: None,
        card: Some(cid),
        cards: orig,
        targets: u.targets.clone(),
        stage: HostStage::Act,
    })))
}

pub(super) fn handle_tiaojiyanmei_loop(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::TargetChoice && m.card == Some(ASK_TIAOJIYANMEI_GAIN_TARGET) {
        m.card = None;
        let orig_cid = m.cards.last().copied().unwrap_or(0);
        let disc = ctx
            .core
            .vm_aux
            .piles
            .remove(&(owner, format!("l11_vm_b:tiaoji_disc:{orig_cid}")))
            .unwrap_or_default();
        if let Some(ChoiceOption::Player(gain_t)) = ctx.answer().first().copied() {
            if ctx.core.player(gain_t).alive {
                let in_d: Vec<CardId> = disc
                    .into_iter()
                    .filter(|&c| ctx.core.discard.contains(&c))
                    .collect();
                for &c in &in_d {
                    ctx.core.discard.retain(|&x| x != c);
                }
                if !in_d.is_empty() {
                    deliver_direct_cards_to_hand(ctx, gain_t, &in_d);
                }
            }
        }
        finish_tiaoji_cleanup(ctx, owner, orig_cid, &m.cards);
        return Flow::Done;
    }

    let orig_cid = m.card.unwrap_or(0);
    if let Some(next_t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(next_t).alive {
            let eff = Box::new(HostMachine {
                def_key: "tiaojiyanmei__one".to_string(),
                behavior: HostBehavior::L11VmFixB,
                owner,
                target: Some(next_t),
                card: None,
                cards: vec![orig_cid],
                targets: vec![next_t],
                stage: HostStage::Act,
            });
            spawn_child_nullify(ctx, orig_cid, owner, eff);
            return Flow::Yield;
        }
        return handle_tiaojiyanmei_loop(m, ctx);
    }

    let orig_targets: Vec<PlayerId> = m.cards.iter().map(|&x| x as PlayerId).collect();
    if ctx.core.player(owner).alive && orig_targets.len() >= 2 {
        let first_h = ctx.core.player(orig_targets[0]).hand.len();
        let all_equal = orig_targets
            .iter()
            .skip(1)
            .all(|&p| ctx.core.player(p).hand.len() == first_h);
        let in_d: Vec<CardId> = ctx
            .core
            .vm_aux
            .piles
            .get(&(owner, format!("l11_vm_b:tiaoji_disc:{orig_cid}")))
            .map(|cs| {
                cs.iter()
                    .copied()
                    .filter(|c| ctx.core.discard.contains(c))
                    .collect()
            })
            .unwrap_or_default();
        if all_equal && !in_d.is_empty() {
            m.cards.push(orig_cid);
            return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_TIAOJIYANMEI_GAIN_TARGET);
        }
    }
    finish_tiaoji_cleanup(ctx, owner, orig_cid, &orig_targets_u32(&m.cards));
    Flow::Done
}

fn orig_targets_u32(cards: &[CardId]) -> Vec<u32> {
    cards.to_vec()
}

fn finish_tiaoji_cleanup(ctx: &mut Ctx<'_>, owner: PlayerId, cid: CardId, orig_u32: &[u32]) {
    set_flag(ctx.core, owner, &format!("l11_vm_b:tiaoji_avg2:{cid}"), 0);
    for &p in orig_u32 {
        if let Some(pl) = ctx.core.players.get(p as usize) {
            let _ = pl;
            set_flag(ctx.core, p as PlayerId, &format!("l11_vm_b:tiaoji_h:{cid}"), 0);
        }
    }
    ctx.core
        .vm_aux
        .piles
        .remove(&(owner, format!("l11_vm_b:tiaoji_disc:{cid}")));
    finish_table_card(ctx, cid);
}

pub(super) fn handle_tiaojiyanmei_one(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
        return Flow::Done;
    };
    let cid = m.cards.first().copied().unwrap_or(0);
    match (m.stage, m.card) {
        (HostStage::Act, None) => {
            let avg2 = flag(ctx.core, owner, &format!("l11_vm_b:tiaoji_avg2:{cid}"));
            let t_len = ctx
                .core
                .vm_aux
                .flags
                .get(&(t, format!("l11_vm_b:tiaoji_h:{cid}")))
                .copied()
                .unwrap_or(ctx.core.player(t).hand.len() as u32);
            let h2 = t_len.saturating_mul(2);
            if h2 > avg2 {
                if he_cards(ctx.core, t).is_empty() {
                    return Flow::Done;
                }
                wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_TIAOJIYANMEI_DISCARD)
            } else if h2 < avg2 {
                ctx.start_draw(t, 1);
                Flow::Done
            } else {
                Flow::Done
            }
        }
        (HostStage::SelectCard, Some(ASK_TIAOJIYANMEI_DISCARD)) => {
            m.card = None;
            if let Some(ChoiceOption::Card(dcid)) = ctx.answer().first().copied() {
                discard_player_cards(ctx, t, &[dcid]);
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, format!("l11_vm_b:tiaoji_disc:{cid}")))
                    .or_default()
                    .push(dcid);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
