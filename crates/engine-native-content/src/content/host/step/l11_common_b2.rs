//! L11（太虚幻境）Batch 2 共用辅助与生命周期钩子（`l11_common_b2.rs`）。
//!
//! 承载：
//! - `host_before_recover`（`ljxuechi` 血池阻断回复并摸 1）
//! - `host_before_gain_hand`（`zglkongcheng_gain` 空手受他人外部给牌入扩展区）
//! - `on_draw_begin_l11_b2`（`zglkongcheng_got` / `txnianrui` / `txhj_aHaoSkill3` / `txmengtai`）
//! - `on_card_use_after_l11_b2`（`txhj_aleSkill1_after` / `txjuxiang_gain` / `txkuangyi` / 出牌颜色统计）
//! - Pindian（`txyonglve` 牌堆顶拼点 + `txyingyang` 点数 +/-3）与虚拟出牌辅助

use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog, Suit};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{CardUseData, EventData, Phase};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, LogEvent};
use crate::state::GameStateCore;

pub(super) const ZGL_KONGCHENG_PILE: &str = "l11_zglkongcheng";
pub(super) const CHENGYUAN_PILE: &str = "l11_txchengyuan";
pub(super) const GLOBAL_JUXIANG_POOL: &str = "l11_juxiang_pool";

pub(super) fn is_ordinary_trick_kind(k: CardKind) -> bool {
    !k.is_basic_kind()
        && k.equip_slot().is_none()
        && k.is_basic_trick_or_equip()
        && !matches!(
            k,
            CardKind::Le
                | CardKind::Bing
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Guiyoujie
        )
}

pub(super) fn damage_effective_kind(
    state: &GameStateCore,
    frames: &[crate::event::Frame],
    frame: Option<&crate::event::Frame>,
    catalog: &Catalog,
    d: &crate::event::DamageData,
) -> Option<CardKind> {
    let root = frame.and_then(|f| match f.data {
        EventData::DamageWindow { root, .. } => frames.iter().find(|r| r.id == root),
        EventData::Damage(_) => Some(f),
        _ => None,
    });
    root.and_then(|r| crate::event::damage_card_kind(frames, r, catalog))
        .or_else(|| {
            d.card.map(|c| {
                state
                    .declared_kinds
                    .get(&c)
                    .copied()
                    .unwrap_or_else(|| catalog.card_kind(c))
            })
        })
        .or_else(|| match flag(state, d.target, "l28_vdmg_kind") {
            1 => Some(CardKind::Sha),
            2 => Some(CardKind::SavageAttack),
            3 => Some(CardKind::ArcheryAttack),
            _ => None,
        })
}

/// 血池（`extension.js:4751`）：全局回复前，任意未封技 `ljxuechi` 持有者若其手牌数 < 3，取消本次回复并令其摸 1 张牌。
pub(crate) fn host_before_recover(ctx: &mut Ctx<'_>, _recovered: PlayerId) -> bool {
    if ctx.core.turn.phase == Phase::Prepare && ctx.core.turn.round == 0 {
        return true;
    }
    if flag(ctx.core, 0, "l11_xuechi_busy") > 0 {
        return true;
    }
    let seats = ctx.core.seat_order_from(ctx.core.turn.current);
    for o in seats {
        if is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, o)
            && has_named_skill(ctx.core, ctx.catalog, o, "ljxuechi")
            && ctx.core.player(o).hand.len() < 3
        {
            set_flag(ctx.core, 0, "l11_xuechi_busy", 1);
            ctx.start_draw(o, 1);
            set_flag(ctx.core, 0, "l11_xuechi_busy", 0);
            return false;
        }
    }
    true
}

/// 空城·护（`extension.js:5307`）：`zglkongcheng` 持有者非当前回合且手牌数为 0 时，从其他角色处获牌改置入扩展区。
pub(crate) fn host_before_gain_hand(
    ctx: &mut Ctx<'_>,
    from: Option<PlayerId>,
    to: PlayerId,
    cards: &[CardId],
) -> Vec<CardId> {
    if cards.is_empty()
        || !from.is_some_and(|f| f != to)
        || ctx.core.turn.current == to
        || !ctx.core.player(to).hand.is_empty()
        || !is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, to)
        || !has_named_skill(ctx.core, ctx.catalog, to, "zglkongcheng")
    {
        return cards.to_vec();
    }
    let pile = ctx
        .core
        .vm_aux
        .piles
        .entry((to, ZGL_KONGCHENG_PILE.to_string()))
        .or_default();
    pile.extend_from_slice(cards);
    Vec::new()
}

/// 将牌直接交予 `to` 手牌（不经 `host_before_gain_hand` 拦截，如 `txjuxiang` `.giver = player`）。
pub(super) fn deliver_direct_cards_to_hand(
    ctx: &mut Ctx<'_>,
    to: PlayerId,
    cards: &[CardId],
) {
    if cards.is_empty() || !ctx.core.player(to).alive {
        return;
    }
    let delivered = cards.to_vec();
    ctx.core.player_mut(to).hand.extend(delivered.iter().copied());
    ctx.out.push(LogEvent::CardsDrawn {
        player: to,
        cards: delivered.clone(),
    });
    ctx.emit(EventData::CardsDrawn {
        player: to,
        cards: delivered,
    });
}

/// 摸牌阶段开始修正钩子：`zglkongcheng_got`、`txnianrui`、`txhj_aHaoSkill3`、`txmengtai`。
pub(super) fn on_draw_begin_l11_b2(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    if !ctx.core.player(player).alive {
        return;
    }
    // `zglkongcheng_got` (`extension.js:5325`): 摸牌阶段开始取回扩展区「空城」牌
    if let Some(exp) = ctx
        .core
        .vm_aux
        .piles
        .remove(&(player, ZGL_KONGCHENG_PILE.to_string()))
    {
        if !exp.is_empty() {
            ctx.core.player_mut(player).hand.extend(exp.iter().copied());
            ctx.out.push(LogEvent::CardsDrawn {
                player,
                cards: exp.clone(),
            });
            ctx.emit(EventData::CardsDrawn {
                player,
                cards: exp,
            });
        }
    }
    set_flag(ctx.core, player, "skill_turn:l11_mengtai_no_draw", 0);
    if is_locked_unblocked_l11_owner(ctx.core, ctx.catalog, player)
        && has_named_skill(ctx.core, ctx.catalog, player, "txnianrui")
    {
        *pending += 2;
    }
    if has_named_skill(ctx.core, ctx.catalog, player, "txhj_aHaoSkill3") {
        *pending += ctx.core.player(player).equips.len() as i32;
    }
}

/// `useCardAfter` / `useCardEnd` 钩子：记录本回合出牌色/南蛮/首杀统计，并结算 `txhj_aleSkill1_after`、`txjuxiang_gain`、`txkuangyi`。
pub(super) fn on_card_use_after_l11_b2(
    ctx: &mut Ctx<'_>,
    cur_fid: crate::ids::EventId,
    u: &CardUseData,
) {
    let kind = u.effective_kind(ctx.catalog);
    let color_bit = u.card.map(|c| {
        if matches!(
            ctx.catalog.effective_card_suit(ctx.core, u.player, c),
            Suit::Heart | Suit::Diamond
        ) {
            1u32
        } else {
            2u32
        }
    });
    if u.player == ctx.core.turn.current {
        if ctx.core.turn.phase == Phase::Play && ctx.catalog.skill_by_name("txkeji").is_some() {
            if let Some(bit) = color_bit {
                let prev = flag(ctx.core, u.player, "skill_turn:l11_play_color_mask");
                set_flag(ctx.core, u.player, "skill_turn:l11_play_color_mask", prev | bit);
            }
        }
        if ctx.catalog.skill_by_name("txchengyuan").is_some() {
            if let Some(bit) = color_bit {
                let prev = flag(ctx.core, u.player, "skill_turn:l11_chengyuan_used");
                set_flag(ctx.core, u.player, "skill_turn:l11_chengyuan_used", prev | bit);
            }
        }
        if kind == CardKind::SavageAttack && ctx.catalog.skill_by_name("txjuxiang").is_some() {
            set_flag(ctx.core, u.player, "skill_turn:l11_used_nanman", 1);
        }
        if kind.is_sha_like() && has_named_skill(ctx.core, ctx.catalog, u.player, "txhj_luluSkill2")
        {
            set_flag(ctx.core, u.player, "skill_turn:l11_lulu2_sha_seen", 1);
        }
    }
    if u.cancelled {
        return;
    }

    // `txhj_aleSkill1_after` (`extension.js:2356`): 被基本牌指定目标但该牌未造成任何伤害 -> 摸 1 (`charlotte: true`)
    let seats = ctx.core.seat_order_from(ctx.core.turn.current);
    let armed_key = format!("skill_turn:l11_ale1_armed:{cur_fid}");
    let hit_key = format!("skill_turn:l11_ale1_hit:{cur_fid}");
    for p in seats.iter().copied() {
        if flag(ctx.core, p, &armed_key) > 0 {
            set_flag(ctx.core, p, &armed_key, 0);
            let dealt_dmg = flag(ctx.core, p, &hit_key) > 0;
            set_flag(ctx.core, p, &hit_key, 0);
            if !dealt_dmg
                && ctx.core.player(p).alive
                && has_named_skill(ctx.core, ctx.catalog, p, "txhj_aleSkill1")
            {
                ctx.start_draw(p, 1);
            }
        }
    }

    // `txjuxiang_gain` (`extension.js:5445`): 他人使用【南蛮入侵】结算后，自弃牌堆/桌面获得该牌
    if kind == CardKind::SavageAttack {
        if let Some(cid) = u.card {
            for p in seats.iter().copied() {
                if p != u.player
                    && ctx.core.player(p).alive
                    && has_named_skill(ctx.core, ctx.catalog, p, "txjuxiang")
                    && (ctx.core.discard.contains(&cid) || ctx.core.table.contains(&cid))
                {
                    ctx.core.discard.retain(|&c| c != cid);
                    ctx.core.table.retain(|&c| c != cid);
                    ctx.core.player_mut(p).hand.push(cid);
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: p,
                        cards: vec![cid],
                    });
                    ctx.emit(EventData::CardsDrawn {
                        player: p,
                        cards: vec![cid],
                    });
                    break;
                }
            }
        }
    }

    // `txkuangyi` (`extension.js:4664`): 出牌阶段使用基本牌结算后仍在弃牌堆/桌面 -> 询问横置并收回该牌（限 1/回合）
    if ctx.core.turn.phase == Phase::Play
        && u.player == ctx.core.turn.current
        && kind.is_basic_kind()
        && flag(ctx.core, u.player, "skill_turn:l11_kuangyi_used") == 0
        && is_unblocked_l11_owner(ctx.core, ctx.catalog, u.player)
        && has_named_skill(ctx.core, ctx.catalog, u.player, "txkuangyi")
    {
        if let Some(cid) = u.card {
            if ctx.core.discard.contains(&cid) || ctx.core.table.contains(&cid) {
                ctx.res.parks.push(crate::resolution::Park {
                    resume_depth: ctx.res.stack.len(),
                    slot: crate::resolution::Slot::Intrinsic {
                        frame: crate::resolution::NO_FRAME,
                    },
                    awaiting_answer: false,
                    machine: Box::new(HostMachine {
                        def_key: "txkuangyi".to_string(),
                        behavior: HostBehavior::L11GeneralSkillsD,
                        owner: u.player,
                        target: None,
                        card: Some(cid),
                        cards: vec![cid],
                        targets: Vec::new(),
                        stage: HostStage::Act,
                    }),
                });
            }
        }
    }
}

pub(super) fn suit_is_red(catalog: &Catalog, core: &GameStateCore, owner: PlayerId, cid: CardId) -> bool {
    matches!(
        catalog.effective_card_suit(core, owner, cid),
        Suit::Heart | Suit::Diamond
    )
}

pub(super) fn in_attack_range(
    core: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
) -> bool {
    from != to
        && core.player(from).alive
        && core.player(to).alive
        && core.computed_distance(catalog, from, to) <= core.attack_range(catalog, from)
}

pub(super) fn emit_virtual_kind_use(
    ctx: &mut Ctx<'_>,
    user: PlayerId,
    kind: CardKind,
    targets: Vec<PlayerId>,
    nature: crate::event::DamageNature,
) {
    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
        card: None,
        player: user,
        targets,
        nature,
        use_as: Some(kind),
        cancelled: false,
    }));
}

/// 退还 1 次出杀次数（`txhj_oldhuxiao` / `extension.js:6078`）。
pub(super) fn refund_one_sha_use(ctx: &mut Ctx<'_>, player: PlayerId) {
    let extra_used = flag(ctx.core, player, "vm_sha_extra_used");
    if extra_used > 0 {
        set_flag(ctx.core, player, "vm_sha_extra_used", extra_used - 1);
    } else if ctx.core.turn.land_sha_used {
        ctx.core.turn.land_sha_used = false;
    } else if ctx.core.turn.sha_used_play_phase {
        ctx.core.turn.sha_used_play_phase = false;
    }
}

// ---- `txyonglve` & `txyingyang` Pindian hooks ----

pub(crate) fn has_txyonglve_ready(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    (!core.deck.is_empty() || !core.discard.is_empty())
        && is_unblocked_l11_owner(core, catalog, player)
        && has_named_skill(core, catalog, player, "txyonglve")
}

pub(crate) fn has_txyingyang_ready(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    is_unblocked_l11_owner(core, catalog, player)
        && has_named_skill(core, catalog, player, "txyingyang")
}

pub(crate) fn pop_yonglve_deck_top(ctx: &mut Ctx<'_>) -> Option<CardId> {
    if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
        let mut reshuffled = std::mem::take(&mut ctx.core.discard);
        ctx.core.rng.shuffle(&mut reshuffled);
        ctx.core.deck = reshuffled;
    }
    let c = ctx.core.deck.pop()?;
    ctx.to_discard(c);
    Some(c)
}

#[inline]
pub(crate) fn apply_yingyang_mode(rank: u8, mode: u32) -> u8 {
    match mode {
        0 => rank.saturating_add(3).min(13),
        1 => rank.saturating_sub(3).max(1),
        _ => rank,
    }
}

pub(crate) fn ask_txyonglve_confirm(chooser: PlayerId) -> crate::snap::ExpectedAsk {
    crate::snap::ask(
        chooser,
        "txyonglve:confirm",
        ChoiceKind::Confirm,
        vec![ChoiceOption::Confirm, ChoiceOption::Pass],
        1,
        1,
        true,
    )
}

pub(crate) fn ask_txyingyang_mode(chooser: PlayerId) -> crate::snap::ExpectedAsk {
    crate::snap::ask(
        chooser,
        "txyingyang:choose_mode",
        ChoiceKind::PickTag,
        vec![
            ChoiceOption::Tag(0),
            ChoiceOption::Tag(1),
            ChoiceOption::Tag(2),
        ],
        1,
        1,
        true,
    )
}

fn vm_pindian_wait(ask: crate::snap::ExpectedAsk) -> crate::machine::Flow {
    crate::machine::Flow::WaitChoice {
        observer: ask.observer,
        spec: ChoiceSpec {
            kind: ask.kind,
            prompt: ask.prompt,
            options: ask.options,
            min: ask.min,
            max: ask.max,
            allow_cancel: ask.allow_cancel,
        },
    }
}

pub(crate) fn vm_pindian_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    fixed_target: rule_api::PlayerId,
) -> Vec<ChoiceOption> {
    let owner_ok = !core.player(owner).hand.is_empty() || has_txyonglve_ready(core, catalog, owner);
    if !owner_ok {
        return Vec::new();
    }
    let tgt_ok = |p: PlayerId| !core.player(p).hand.is_empty() || has_txyonglve_ready(core, catalog, p);
    if fixed_target.is_some() {
        let Some(idx) = fixed_target.raw().checked_sub(1) else {
            return Vec::new();
        };
        let t = idx as PlayerId;
        if (idx as usize) < core.players.len() && t != owner && core.player(t).alive && tgt_ok(t) {
            return vec![ChoiceOption::Player(t)];
        }
        return Vec::new();
    }
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive && tgt_ok(p))
        .map(ChoiceOption::Player)
        .collect()
}

pub(crate) fn vm_pindian_active_ask(
    owner: PlayerId,
    fixed_target: rule_api::PlayerId,
    allow_cancel: bool,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<crate::snap::ExpectedAsk> {
    match state.first().copied() {
        Some(1) => {
            let options = vm_pindian_targets(core, catalog, owner, fixed_target);
            if options.is_empty() {
                return None;
            }
            Some(crate::snap::ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                allow_cancel,
            ))
        }
        Some(2) => Some(ask_txyonglve_confirm(owner)),
        Some(3) => Some(ask_txyonglve_confirm(*state.get(1)? as PlayerId)),
        Some(4) => Some(ask_txyingyang_mode(owner)),
        Some(5) => Some(ask_txyingyang_mode(*state.get(1)? as PlayerId)),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn vm_finish_hand_compare(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
    pc: CardId,
    tc: CardId,
    pc_from_hand: bool,
    tc_from_hand: bool,
    my_yy: u32,
    their_yy: u32,
    win_skill_def: rule_api::SkillDefId,
    lose_skill_def: rule_api::SkillDefId,
) -> crate::machine::Flow {
    let raw_my = ctx.catalog.cards.get(pc as usize).map(|cd| cd.rank).unwrap_or(0);
    let raw_their = ctx.catalog.cards.get(tc as usize).map(|cd| cd.rank).unwrap_or(0);
    let my_rank = apply_yingyang_mode(raw_my, my_yy);
    let their_rank = apply_yingyang_mode(raw_their, their_yy);
    if pc_from_hand && ctx.core.player(owner).hand.contains(&pc) {
        ctx.discard_from_hand(owner, pc);
    }
    if tc_from_hand && ctx.core.player(target).hand.contains(&tc) {
        ctx.discard_from_hand(target, tc);
    }
    crate::rvm_ops::commit_hand_compare_outcome(
        ctx,
        owner,
        target,
        pc,
        my_rank > their_rank,
        win_skill_def,
        lose_skill_def,
    );
    crate::machine::Flow::Done
}

#[allow(clippy::too_many_arguments)]
fn vm_advance_hand_compare(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    target: PlayerId,
    mut pc: Option<(CardId, bool)>,
    mut tc: Option<(CardId, bool)>,
    mut my_yy: Option<u32>,
    mut their_yy: Option<u32>,
    win_skill_def: rule_api::SkillDefId,
    lose_skill_def: rule_api::SkillDefId,
    state: &mut Vec<u32>,
) -> crate::machine::Flow {
    let enc_card = |c: Option<(CardId, bool)>| match c {
        None => (0u32, 0u32),
        Some((id, h)) => (id + 1, u32::from(h)),
    };
    let enc_yy = |y: Option<u32>| y.map_or(0, |m| m + 1);
    if pc.is_none() {
        if has_txyonglve_ready(ctx.core, ctx.catalog, owner) {
            *state = vec![2, target as u32, 0, 0, 0, 0, enc_yy(my_yy), enc_yy(their_yy)];
            return vm_pindian_wait(ask_txyonglve_confirm(owner));
        }
        let Some(hc) = ctx.core.player(owner).hand.first().copied() else {
            return crate::machine::Flow::Done;
        };
        pc = Some((hc, true));
    }
    if tc.is_none() {
        if has_txyonglve_ready(ctx.core, ctx.catalog, target) {
            let (p_id, p_h) = enc_card(pc);
            *state = vec![3, target as u32, p_id, p_h, 0, 0, enc_yy(my_yy), enc_yy(their_yy)];
            return vm_pindian_wait(ask_txyonglve_confirm(target));
        }
        let Some(hc) = ctx.core.player(target).hand.first().copied() else {
            return crate::machine::Flow::Done;
        };
        tc = Some((hc, true));
    }
    if my_yy.is_none() {
        if has_txyingyang_ready(ctx.core, ctx.catalog, owner) {
            let (p_id, p_h) = enc_card(pc);
            let (t_id, t_h) = enc_card(tc);
            *state = vec![4, target as u32, p_id, p_h, t_id, t_h, 0, enc_yy(their_yy)];
            return vm_pindian_wait(ask_txyingyang_mode(owner));
        }
        my_yy = Some(2);
    }
    if their_yy.is_none() {
        if has_txyingyang_ready(ctx.core, ctx.catalog, target) {
            let (p_id, p_h) = enc_card(pc);
            let (t_id, t_h) = enc_card(tc);
            *state = vec![5, target as u32, p_id, p_h, t_id, t_h, enc_yy(my_yy), 0];
            return vm_pindian_wait(ask_txyingyang_mode(target));
        }
        their_yy = Some(2);
    }
    let (pc_id, pc_h) = pc.unwrap();
    let (tc_id, tc_h) = tc.unwrap();
    vm_finish_hand_compare(
        ctx,
        owner,
        target,
        pc_id,
        tc_id,
        pc_h,
        tc_h,
        my_yy.unwrap_or(2),
        their_yy.unwrap_or(2),
        win_skill_def,
        lose_skill_def,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn vm_step_hand_compare(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    fixed_target: rule_api::PlayerId,
    selected_card: rule_api::CardId,
    win_skill_def: rule_api::SkillDefId,
    lose_skill_def: rule_api::SkillDefId,
    state: &mut Vec<u32>,
) -> crate::machine::Flow {
    let dec_card = |id: u32, h: u32| (id > 0).then(|| (id - 1, h > 0));
    let dec_yy = |v: u32| (v > 0).then(|| v - 1);
    match state.first().copied() {
        Some(1) => {
            let valid = vm_pindian_targets(ctx.core, ctx.catalog, owner, fixed_target);
            let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                return crate::machine::Flow::Done;
            };
            if !valid.contains(&ChoiceOption::Player(target)) {
                return crate::machine::Flow::Done;
            }
            let pc = if selected_card.is_some()
                && ctx.core.player(owner).hand.contains(&selected_card.raw())
            {
                Some((selected_card.raw(), true))
            } else {
                None
            };
            vm_advance_hand_compare(
                ctx,
                owner,
                target,
                pc,
                None,
                None,
                None,
                win_skill_def,
                lose_skill_def,
                state,
            )
        }
        Some(2) => {
            let target = state[1] as PlayerId;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            let pc = if confirmed {
                pop_yonglve_deck_top(ctx).map(|c| (c, false))
            } else {
                ctx.core.player(owner).hand.first().copied().map(|c| (c, true))
            };
            let Some(pc) = pc else {
                return crate::machine::Flow::Done;
            };
            vm_advance_hand_compare(
                ctx,
                owner,
                target,
                Some(pc),
                dec_card(state[4], state[5]),
                dec_yy(state[6]),
                dec_yy(state[7]),
                win_skill_def,
                lose_skill_def,
                state,
            )
        }
        Some(3) => {
            let target = state[1] as PlayerId;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            let tc = if confirmed {
                pop_yonglve_deck_top(ctx).map(|c| (c, false))
            } else {
                ctx.core.player(target).hand.first().copied().map(|c| (c, true))
            };
            let Some(tc) = tc else {
                return crate::machine::Flow::Done;
            };
            vm_advance_hand_compare(
                ctx,
                owner,
                target,
                dec_card(state[2], state[3]),
                Some(tc),
                dec_yy(state[6]),
                dec_yy(state[7]),
                win_skill_def,
                lose_skill_def,
                state,
            )
        }
        Some(4) | Some(5) => {
            let target = state[1] as PlayerId;
            let is_owner = state[0] == 4;
            let mode = match ctx.answer().first().copied() {
                Some(ChoiceOption::Tag(m)) if m <= 2 => m,
                _ => 2,
            };
            vm_advance_hand_compare(
                ctx,
                owner,
                target,
                dec_card(state[2], state[3]),
                dec_card(state[4], state[5]),
                if is_owner { Some(u32::from(mode)) } else { dec_yy(state[6]) },
                if is_owner { dec_yy(state[7]) } else { Some(u32::from(mode)) },
                win_skill_def,
                lose_skill_def,
                state,
            )
        }
        _ => crate::machine::Flow::Done,
    }
}
