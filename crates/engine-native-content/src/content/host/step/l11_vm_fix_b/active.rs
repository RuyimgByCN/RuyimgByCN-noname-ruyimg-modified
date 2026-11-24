//! L11-E2 `taixu` VM Fix B 主动出牌阶段技能（`quhu` / `refanjian` / `reluanwu` /
//! `rexianzhen` / `sbyangwei` / `shanxie` / `shenfen` / `tianyi` / `tiaoxin`）。

use super::super::super::*;
use super::super::l11_common::*;
use super::super::l11_common_b2::{
    deliver_direct_cards_to_hand, emit_virtual_kind_use, in_attack_range,
};
use super::asks::{
    decode_hidden_or_card, nearest_alive_others, reveal_to_all, take_preset_target,
    valid_reluanwu_sha_cards, valid_tiaoxin_sha_cards, wait_vm_b_ask, ASK_QUHU_DAMAGE_TARGET,
    ASK_QUHU_TARGET, ASK_REFANJIAN_CARD, ASK_REFANJIAN_MODE, ASK_REFANJIAN_TARGET,
    ASK_RELUANWU_AFTER_SHA, ASK_RELUANWU_SHA_TARGET, ASK_RELUANWU_USE_SHA, ASK_REXIANZHEN_TARGET,
    ASK_SHENFEN_DISCARD, ASK_TIANYI_TARGET, ASK_TIAOXIN_DISCARD, ASK_TIAOXIN_TARGET,
    ASK_TIAOXIN_USE_SHA,
};
use super::pindian::{step_pindian, PindianStep};
use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::choice::ChoiceOption;
use crate::event::{CardUseData, DamageNature};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow};
use crate::state::GameStateCore;

pub(super) const BAONU_MARK_ID: u32 = 1_252_226_867;

fn sha_nature_for_card(catalog: &Catalog, core: &GameStateCore, c: CardId) -> DamageNature {
    match catalog.resolved_card_kind(core, c) {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    }
}

fn emit_hand_sha_card(ctx: &mut Ctx<'_>, user: PlayerId, cid: CardId, target: PlayerId) {
    let nat = sha_nature_for_card(ctx.catalog, ctx.core, cid);
    ctx.core.player_mut(user).hand.retain(|&x| x != cid);
    ctx.core.table.push(cid);
    ctx.note_cards_lost(user, &[cid]);
    (ctx.catalog.hooks.on_hand_card_removed)(ctx, user, cid);
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(cid),
        player: user,
        targets: vec![target],
        nature: nat,
        use_as: Some(CardKind::Sha),
        cancelled: false,
    }));
}

pub(super) fn handle_quhu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.target.is_none() {
        let o_hp = ctx.core.player(owner).hp;
        if let Some(t) = take_preset_target(ctx.core, owner).filter(|&p| {
            p != owner
                && ctx.core.player(p).alive
                && ctx.core.player(p).hp > o_hp
                && (!ctx.core.player(p).hand.is_empty()
                    || super::super::l11_common_b2::has_txyonglve_ready(ctx.core, ctx.catalog, p))
        }) {
            m.target = Some(t);
        } else {
            return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_QUHU_TARGET);
        }
    }
    if m.stage == HostStage::TargetChoice && m.card == Some(ASK_QUHU_TARGET) {
        m.card = None;
        m.stage = HostStage::Act;
        let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        m.target = Some(t);
    }
    if m.stage == HostStage::TargetChoice && m.card == Some(ASK_QUHU_DAMAGE_TARGET) {
        m.card = None;
        let Some(src) = m.target else {
            return Flow::Done;
        };
        let Some(ChoiceOption::Player(dmg_tgt)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        if ctx.core.player(dmg_tgt).alive {
            emit_damage(ctx, Some(src), dmg_tgt, 1, DamageNature::Normal, None);
        }
        return Flow::Done;
    }
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match step_pindian(m, ctx, owner, t) {
        (PindianStep::Pending, flow) => flow,
        (PindianStep::Aborted, flow) => flow,
        (PindianStep::Resolved { win, .. }, _) => {
            if win {
                let has_in_range = alive_others(ctx.core, t)
                    .into_iter()
                    .any(|p| in_attack_range(ctx.core, ctx.catalog, t, p));
                if has_in_range {
                    wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_QUHU_DAMAGE_TARGET)
                } else {
                    Flow::Done
                }
            } else {
                if ctx.core.player(owner).alive {
                    emit_damage(ctx, Some(t), owner, 1, DamageNature::Normal, None);
                }
                Flow::Done
            }
        }
    }
}

pub(super) fn handle_rexianzhen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.target.is_none() {
        if let Some(t) = take_preset_target(ctx.core, owner).filter(|&p| {
            p != owner
                && ctx.core.player(p).alive
                && (!ctx.core.player(p).hand.is_empty()
                    || super::super::l11_common_b2::has_txyonglve_ready(ctx.core, ctx.catalog, p))
        }) {
            m.target = Some(t);
        } else {
            return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_REXIANZHEN_TARGET);
        }
    }
    if m.stage == HostStage::TargetChoice && m.card == Some(ASK_REXIANZHEN_TARGET) {
        m.card = None;
        m.stage = HostStage::Act;
        let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        m.target = Some(t);
    }
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match step_pindian(m, ctx, owner, t) {
        (PindianStep::Pending, flow) => flow,
        (PindianStep::Aborted, flow) => flow,
        (PindianStep::Resolved { owner_card, win, .. }, _) => {
            if ctx.catalog.resolved_card_kind(ctx.core, owner_card).is_sha_like() {
                ensure_perm_skill(ctx, owner, "rexianzhen4");
            }
            if win {
                set_flag(ctx.core, owner, "bound_target", u32::from(t) + 1);
                ensure_perm_skill(ctx, owner, "rexianzhen2");
            } else {
                ensure_perm_skill(ctx, owner, "rexianzhen3");
            }
            Flow::Done
        }
    }
}

pub(super) fn handle_tianyi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.target.is_none() {
        if let Some(t) = take_preset_target(ctx.core, owner).filter(|&p| {
            p != owner
                && ctx.core.player(p).alive
                && (!ctx.core.player(p).hand.is_empty()
                    || super::super::l11_common_b2::has_txyonglve_ready(ctx.core, ctx.catalog, p))
        }) {
            m.target = Some(t);
        } else {
            return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_TIANYI_TARGET);
        }
    }
    if m.stage == HostStage::TargetChoice && m.card == Some(ASK_TIANYI_TARGET) {
        m.card = None;
        m.stage = HostStage::Act;
        let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        m.target = Some(t);
    }
    let Some(t) = m.target else {
        return Flow::Done;
    };
    match step_pindian(m, ctx, owner, t) {
        (PindianStep::Pending, flow) => flow,
        (PindianStep::Aborted, flow) => flow,
        (PindianStep::Resolved { win, .. }, _) => {
            if win {
                ensure_perm_skill(ctx, owner, "tianyi2");
            } else {
                ensure_perm_skill(ctx, owner, "tianyi3");
            }
            Flow::Done
        }
    }
}

pub(super) fn handle_refanjian(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match (m.stage, m.card) {
        (HostStage::Act, None) => wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_REFANJIAN_CARD),
        (HostStage::SelectCard, Some(ASK_REFANJIAN_CARD)) => {
            m.card = None;
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c) {
                return Flow::Done;
            }
            m.cards = vec![c];
            wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_REFANJIAN_TARGET)
        }
        (HostStage::TargetChoice, Some(ASK_REFANJIAN_TARGET)) => {
            m.card = None;
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(&cid) = m.cards.first() else {
                return Flow::Done;
            };
            m.target = Some(t);
            move_cards_to_hand(ctx, owner, t, &[cid]);
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            if ctx.core.player(t).hand.is_empty() {
                emit_lose_hp(ctx, t, 1);
                return Flow::Done;
            }
            wait_vm_b_ask(m, ctx, HostStage::SelectNature, ASK_REFANJIAN_MODE)
        }
        (HostStage::SelectNature, Some(ASK_REFANJIAN_MODE)) => {
            m.card = None;
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if tag == 0 {
                let hand = ctx.core.player(t).hand.clone();
                reveal_to_all(ctx, hand);
                let given_cid = m.cards.first().copied();
                let target_suit = given_cid.map(|c| ctx.catalog.effective_card_suit(ctx.core, t, c));
                if let Some(suit) = target_suit {
                    let matching: Vec<CardId> = he_cards(ctx.core, t)
                        .into_iter()
                        .filter(|&c| ctx.catalog.effective_card_suit(ctx.core, t, c) == suit)
                        .collect();
                    if !matching.is_empty() {
                        discard_player_cards(ctx, t, &matching);
                    }
                }
            } else {
                emit_lose_hp(ctx, t, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_reluanwu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.card.is_none() && m.cards.is_empty() {
        m.targets = alive_others(ctx.core, owner);
        m.cards = vec![0];
    }
    match (m.stage, m.card) {
        (HostStage::SelectCard, Some(ASK_RELUANWU_USE_SHA)) => {
            m.card = None;
            let Some(p) = m.target else {
                return reluanwu_advance(m, ctx);
            };
            let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() else {
                emit_lose_hp(ctx, p, 1);
                m.stage = HostStage::Act;
                return Flow::Yield;
            };
            if !ctx.core.player(p).hand.contains(&cid) {
                emit_lose_hp(ctx, p, 1);
                m.stage = HostStage::Act;
                return Flow::Yield;
            }
            let kind = ctx.catalog.resolved_card_kind(ctx.core, cid);
            let nat = sha_nature_for_card(ctx.catalog, ctx.core, cid);
            let nearest: Vec<PlayerId> = nearest_alive_others(ctx.core, ctx.catalog, p)
                .into_iter()
                .filter(|&q| {
                    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        ctx.core,
                        ctx.catalog,
                        p,
                        q,
                        cid,
                        kind,
                        nat,
                    )
                })
                .collect();
            if nearest.is_empty() {
                emit_lose_hp(ctx, p, 1);
                m.stage = HostStage::Act;
                return Flow::Yield;
            }
            if nearest.len() == 1 {
                emit_hand_sha_card(ctx, p, cid, nearest[0]);
                m.stage = HostStage::Act;
                return Flow::Yield;
            }
            m.cards = vec![0, cid];
            wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_RELUANWU_SHA_TARGET)
        }
        (HostStage::TargetChoice, Some(ASK_RELUANWU_SHA_TARGET)) => {
            m.card = None;
            let Some(p) = m.target else {
                return reluanwu_advance(m, ctx);
            };
            let Some(cid) = m.cards.get(1).copied() else {
                return reluanwu_advance(m, ctx);
            };
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                emit_hand_sha_card(ctx, p, cid, t);
                m.stage = HostStage::Act;
                return Flow::Yield;
            }
            emit_lose_hp(ctx, p, 1);
            m.stage = HostStage::Act;
            Flow::Yield
        }
        (HostStage::TargetChoice, Some(ASK_RELUANWU_AFTER_SHA)) => {
            m.card = None;
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                emit_virtual_kind_use(ctx, owner, CardKind::Sha, vec![t], DamageNature::Normal);
            }
            Flow::Done
        }
        _ => reluanwu_advance(m, ctx),
    }
}

fn reluanwu_advance(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.cards.first().copied() == Some(1) {
        return Flow::Done;
    }
    while let Some(next_p) = m.targets.first().copied() {
        m.targets.remove(0);
        if !ctx.core.player(next_p).alive {
            continue;
        }
        m.target = Some(next_p);
        let valid = valid_reluanwu_sha_cards(ctx.core, ctx.catalog, next_p);
        if valid.is_empty() {
            emit_lose_hp(ctx, next_p, 1);
            m.stage = HostStage::Act;
            return Flow::Yield;
        }
        return wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_RELUANWU_USE_SHA);
    }
    m.cards = vec![1];
    m.target = None;
    if ctx.core.player(owner).alive
        && !crate::rvm_bridge::modifiers::eval_card_use_canceled(
            ctx.core,
            ctx.catalog,
            owner,
            CardId::MAX,
            CardKind::Sha,
            DamageNature::Normal,
        )
        && alive_others(ctx.core, owner).into_iter().any(|q| {
            !crate::rvm_bridge::modifiers::eval_card_target_canceled_opt(
                ctx.core,
                ctx.catalog,
                owner,
                q,
                None,
                CardKind::Sha,
                DamageNature::Normal,
            )
        })
    {
        return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_RELUANWU_AFTER_SHA);
    }
    Flow::Done
}

pub(super) fn handle_sbyangwei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, 2);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            ensure_perm_skill(ctx, owner, "sbyangwei_effect");
            ensure_perm_skill(ctx, owner, "sbyangwei_counter");
            set_flag(ctx.core, owner, "l11_vm_b:sbyangwei_cd_armed", 0);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn handle_shanxie(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if let Some(pos) = ctx.core.deck.iter().rposition(|&c| {
        ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Weapon)
    }) {
        let cid = ctx.core.deck.remove(pos);
        deliver_direct_cards_to_hand(ctx, owner, &[cid]);
        return Flow::Done;
    }
    let weapon_holders: Vec<PlayerId> = alive_all(ctx.core, owner)
        .into_iter()
        .filter(|&p| {
            ctx.core
                .player(p)
                .equips
                .iter()
                .any(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Weapon))
        })
        .collect();
    if !weapon_holders.is_empty() {
        let idx = ctx.core.rng.gen_range(weapon_holders.len() as u32) as usize;
        let chosen = weapon_holders[idx];
        let weps: Vec<CardId> = ctx
            .core
            .player(chosen)
            .equips
            .iter()
            .copied()
            .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(EquipSlot::Weapon))
            .collect();
        if chosen != owner {
            move_cards_to_hand(ctx, chosen, owner, &weps);
        } else {
            for &c in &weps {
                ctx.core.player_mut(owner).equips.retain(|&x| x != c);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
            }
            ctx.note_cards_lost(owner, &weps);
            deliver_direct_cards_to_hand(ctx, owner, &weps);
        }
    }
    Flow::Done
}

pub(super) fn handle_shenfen(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.card.is_none() && m.cards.is_empty() {
        let cur = ctx.core.vm_aux.marks.get(&(owner, BAONU_MARK_ID)).copied().unwrap_or(0);
        if cur < 6 {
            return Flow::Done;
        }
        ctx.core.vm_aux.marks.insert((owner, BAONU_MARK_ID), cur - 6);
        let others = alive_others(ctx.core, owner);
        m.cards = std::iter::once(0u32)
            .chain(others.iter().map(|&p| u32::from(p)))
            .collect();
        m.targets = others;
    }
    if m.stage == HostStage::CardsAsk && m.card == Some(ASK_SHENFEN_DISCARD) {
        m.card = None;
        m.stage = HostStage::Act;
        if let Some(t) = m.target {
            let picked = answered_cards(ctx);
            if !picked.is_empty() && ctx.core.player(t).alive {
                discard_player_cards(ctx, t, &picked);
            }
        }
    }
    let phase = m.cards.first().copied().unwrap_or(0);
    if phase == 0 {
        while let Some(next_p) = m.targets.first().copied() {
            m.targets.remove(0);
            if !ctx.core.player(next_p).alive {
                continue;
            }
            emit_damage(ctx, Some(owner), next_p, 1, DamageNature::Normal, None);
            m.stage = HostStage::Act;
            return Flow::Yield;
        }
        if let Some(p0) = m.cards.first_mut() {
            *p0 = 1;
        }
        m.targets = m.cards.iter().skip(1).map(|&x| x as PlayerId).collect();
    }
    while let Some(next_p) = m.targets.first().copied() {
        m.targets.remove(0);
        if !ctx.core.player(next_p).alive || he_cards(ctx.core, next_p).is_empty() {
            continue;
        }
        m.target = Some(next_p);
        return wait_vm_b_ask(m, ctx, HostStage::CardsAsk, ASK_SHENFEN_DISCARD);
    }
    Flow::Done
}

pub(super) fn handle_tiaoxin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.stage == HostStage::Act && m.target.is_none() {
        if let Some(t) = take_preset_target(ctx.core, owner).filter(|&p| {
            p != owner
                && ctx.core.player(p).alive
                && in_attack_range(ctx.core, ctx.catalog, p, owner)
                && !he_cards(ctx.core, p).is_empty()
        }) {
            m.target = Some(t);
        } else {
            return wait_vm_b_ask(m, ctx, HostStage::TargetChoice, ASK_TIAOXIN_TARGET);
        }
    }
    if m.stage == HostStage::TargetChoice && m.card == Some(ASK_TIAOXIN_TARGET) {
        m.card = None;
        m.stage = HostStage::Act;
        let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        m.target = Some(t);
    }
    if m.stage == HostStage::SelectCard && m.card == Some(ASK_TIAOXIN_USE_SHA) {
        m.card = None;
        let Some(t) = m.target else {
            return Flow::Done;
        };
        if let Some(ChoiceOption::Card(cid)) = ctx.answer().first().copied() {
            if ctx.core.player(t).hand.contains(&cid) {
                emit_hand_sha_card(ctx, t, cid, owner);
                return Flow::Done;
            }
        }
        if !ctx.core.player(t).alive || he_cards(ctx.core, t).is_empty() {
            return Flow::Done;
        }
        return wait_vm_b_ask(m, ctx, HostStage::CardsAsk, ASK_TIAOXIN_DISCARD);
    }
    if m.stage == HostStage::CardsAsk && m.card == Some(ASK_TIAOXIN_DISCARD) {
        m.card = None;
        let Some(t) = m.target else {
            return Flow::Done;
        };
        if let Some(&opt) = ctx.answer().first() {
            if let Some(cid) = decode_hidden_or_card(ctx.core, t, opt) {
                discard_player_cards(ctx, t, &[cid]);
            }
        }
        return Flow::Done;
    }
    let Some(t) = m.target else {
        return Flow::Done;
    };
    if !valid_tiaoxin_sha_cards(ctx.core, ctx.catalog, t, owner).is_empty() {
        wait_vm_b_ask(m, ctx, HostStage::SelectCard, ASK_TIAOXIN_USE_SHA)
    } else if !he_cards(ctx.core, t).is_empty() {
        wait_vm_b_ask(m, ctx, HostStage::CardsAsk, ASK_TIAOXIN_DISCARD)
    } else {
        Flow::Done
    }
}
