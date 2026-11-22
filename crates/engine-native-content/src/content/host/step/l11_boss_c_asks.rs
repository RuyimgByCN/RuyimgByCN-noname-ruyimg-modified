//! L11（太虚幻境）Batch L11-5 领主与通用武将技询问构建（`l11_boss_c_asks.rs`）。

use super::l11_common::*;
use super::l11_skills_b_asks::{can_receive_moved_card, movable_field_sources};
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceSpec;
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_MOUDUAN_CONFIRM: u32 = 1;
pub(super) const ASK_MOUDUAN_FROM: u32 = 2;
pub(super) const ASK_MOUDUAN_CARD: u32 = 3;
pub(super) const ASK_MOUDUAN_TO: u32 = 4;
pub(super) const ASK_OLSHANJIA_CONFIRM: u32 = 5;
pub(super) const ASK_OLSHANJIA_DISCARD: u32 = 6;
pub(super) const ASK_OLSHANJIA_SHA_TARGET: u32 = 7;
pub(super) const ASK_XUANLVE_TARGET: u32 = 8;
pub(super) const ASK_XUANLVE_CARD: u32 = 9;
pub(super) const ASK_LINGFENG_CONFIRM: u32 = 10;
pub(super) const ASK_LINGFENG_TARGET: u32 = 11;
pub(super) const ASK_LEILI_TARGET: u32 = 12;
pub(super) const ASK_FENGXING_TARGET: u32 = 13;
pub(super) const ASK_YUSHOU_CONFIRM: u32 = 14;
pub(super) const ASK_XIXING_TARGET: u32 = 15;
pub(super) const ASK_MOYANY_CONFIRM: u32 = 16;
pub(super) const ASK_MOYANY_TARGET: u32 = 17;
pub(super) const ASK_DANSHU_CONFIRM: u32 = 18;
pub(super) const ASK_JUEJI_CONFIRM: u32 = 19;
pub(super) const ASK_MOJIAN_CONFIRM: u32 = 20;
pub(super) const ASK_LIANYU_CONFIRM: u32 = 21;
pub(super) const ASK_ZHUWEI_CONFIRM_GAIN: u32 = 22;
pub(super) const ASK_ZHUWEI_CONFIRM_BUFF: u32 = 23;
pub(super) const ASK_CHUHAI_TARGET: u32 = 24;
pub(super) const ASK_CHUHAI_PINDIAN_OWNER: u32 = 25;
pub(super) const ASK_CHUHAI_PINDIAN_TARGET: u32 = 26;
pub(super) const ASK_XUEJI_CARD: u32 = 27;
pub(super) const ASK_XUEJI_TARGET: u32 = 28;

pub(super) fn mouduan_from_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
) -> Vec<CardId> {
    let all = alive_all(core, from);
    let pl = core.player(from);
    pl.equips
        .iter()
        .chain(pl.judge.iter())
        .copied()
        .filter(|&c| all.iter().any(|&to| can_receive_moved_card(core, catalog, from, to, c)))
        .collect()
}

pub(super) fn mouduan_to_players(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    from: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&to| can_receive_moved_card(core, catalog, from, to, card))
        .collect()
}

pub(super) fn enemy_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    exclude: Option<PlayerId>,
) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| Some(p) != exclude && !core.is_friend_of(catalog, owner, p))
        .collect()
}

pub(super) fn sha_target_valid(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    p: PlayerId,
) -> bool {
    if !core.player(p).alive {
        return false;
    }
    let kongcheng = has_named_skill(core, catalog, p, "zglkongcheng")
        || has_named_skill(core, catalog, p, crate::vocabulary::names::KONG_CHENG);
    if kongcheng && core.player(p).hand.is_empty() {
        return false;
    }
    !crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core,
        catalog,
        actor,
        p,
        0,
        CardKind::Sha,
        DamageNature::Normal,
    )
}

pub(super) fn aoe_enemy_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    kind: CardKind,
) -> Vec<PlayerId> {
    enemy_targets(core, catalog, owner, None)
        .into_iter()
        .filter(|&t| {
            !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                core,
                catalog,
                owner,
                t,
                0,
                kind,
                DamageNature::Normal,
            )
        })
        .collect()
}

pub(super) fn chuhai_targets(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| !core.player(p).hand.is_empty())
        .collect()
}

pub(super) fn xueji_red_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| {
            matches!(
                catalog.effective_card_suit(core, owner, c),
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            )
        })
        .collect()
}

pub(super) fn xueji_max_targets(core: &GameStateCore, owner: PlayerId) -> usize {
    let p = core.player(owner);
    let damaged = (p.max_hp as i32 - p.hp).max(1) as usize;
    damaged.min(core.alive_players().count()).max(1)
}

pub(super) fn build_l11_boss_c_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_MOUDUAN_CONFIRM => confirm_ask(owner, "txhj_mouduan:confirm"),
        ASK_MOUDUAN_FROM => {
            let froms = movable_field_sources(core, catalog, owner);
            if froms.is_empty() {
                return None;
            }
            player_ask(owner, "txhj_mouduan:move_from", froms, 0, 1, true)
        }
        ASK_MOUDUAN_CARD => {
            let from = target.filter(|&p| core.player(p).alive)?;
            let pool = mouduan_from_cards(core, catalog, from);
            if pool.is_empty() {
                return None;
            }
            card_ask(owner, "txhj_mouduan:move_card", pool, 0, 1, true)
        }
        ASK_MOUDUAN_TO => {
            let from = target.filter(|&p| core.player(p).alive)?;
            let cid = cards.first().copied()?;
            let tos = mouduan_to_players(core, catalog, owner, from, cid);
            if tos.is_empty() {
                return None;
            }
            player_ask(owner, "txhj_mouduan:move_to", tos, 0, 1, true)
        }
        ASK_OLSHANJIA_CONFIRM => confirm_ask(owner, "olshanjia:confirm"),
        ASK_OLSHANJIA_DISCARD => {
            let need = (flag(core, owner, "l11_olshanjia_need") as usize).clamp(1, 3);
            let pool = he_cards(core, owner);
            if pool.len() < need {
                return None;
            }
            card_ask(owner, "olshanjia:discard", pool, need, need, false)
        }
        ASK_OLSHANJIA_SHA_TARGET => {
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| sha_target_valid(core, catalog, owner, p))
                .collect();
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "olshanjia:sha_target", tgts, 0, 1, true)
        }
        ASK_XUANLVE_TARGET => {
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .collect();
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "txhj_xuanlve:target", tgts, 0, 1, true)
        }
        ASK_XUANLVE_CARD => {
            let t = target.filter(|&p| core.player(p).alive)?;
            let pool = he_cards(core, t);
            if pool.is_empty() {
                return None;
            }
            card_ask(owner, "txhj_xuanlve:card", pool, 1, 1, false)
        }
        ASK_LINGFENG_CONFIRM => confirm_ask(owner, "boss_lingfeng:confirm"),
        ASK_LINGFENG_TARGET => {
            let tgts = enemy_targets(core, catalog, owner, None);
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "boss_lingfeng:target", tgts, 0, 1, true)
        }
        ASK_LEILI_TARGET => {
            let tgts = enemy_targets(core, catalog, owner, target);
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "boss_leili:target", tgts, 0, 1, true)
        }
        ASK_FENGXING_TARGET => {
            let tgts: Vec<PlayerId> = enemy_targets(core, catalog, owner, None)
                .into_iter()
                .filter(|&p| sha_target_valid(core, catalog, owner, p))
                .collect();
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "boss_fengxing:target", tgts, 0, 1, true)
        }
        ASK_YUSHOU_CONFIRM => confirm_ask(owner, "boss_yushou:confirm"),
        ASK_XIXING_TARGET => {
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).linked)
                .collect();
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "boss_xixing:target", tgts, 0, 1, true)
        }
        ASK_MOYANY_CONFIRM => confirm_ask(owner, "boss_moyany:confirm"),
        ASK_MOYANY_TARGET => {
            let tgts = alive_others(core, owner);
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "boss_moyany:target", tgts, 1, 1, false)
        }
        ASK_DANSHU_CONFIRM => confirm_ask(owner, "boss_danshu:confirm"),
        ASK_JUEJI_CONFIRM => confirm_ask(owner, "boss_jueji:confirm"),
        ASK_MOJIAN_CONFIRM => confirm_ask(owner, "boss_mojian:confirm"),
        ASK_LIANYU_CONFIRM => confirm_ask(owner, "boss_lianyu:confirm"),
        ASK_ZHUWEI_CONFIRM_GAIN => confirm_ask(owner, "txhj_zhuwei:confirm_gain"),
        ASK_ZHUWEI_CONFIRM_BUFF => confirm_ask(owner, "txhj_zhuwei:confirm_buff"),
        ASK_CHUHAI_TARGET => {
            let tgts = chuhai_targets(core, owner);
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "txhj_chuhai:target", tgts, 0, 1, true)
        }
        ASK_CHUHAI_PINDIAN_OWNER => {
            let hand = core.player(owner).hand.clone();
            if hand.is_empty() {
                return None;
            }
            card_ask(owner, "txhj_chuhai:pindian_card", hand, 1, 1, false)
        }
        ASK_CHUHAI_PINDIAN_TARGET => {
            let t = target.filter(|&p| core.player(p).alive)?;
            let hand = core.player(t).hand.clone();
            if hand.is_empty() {
                return None;
            }
            card_ask(t, "txhj_chuhai:target_pindian_card", hand, 1, 1, false)
        }
        ASK_XUEJI_CARD => {
            let pool = xueji_red_cards(core, catalog, owner);
            if pool.is_empty() {
                return None;
            }
            card_ask(owner, "xueji:card", pool, 1, 1, true)
        }
        ASK_XUEJI_TARGET => {
            let tgts = alive_all(core, owner);
            let max_t = xueji_max_targets(core, owner);
            player_ask(owner, "xueji:target", tgts, 1, max_t, true)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l11_boss_c(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle {
        return None;
    }
    let ask_id = card?;
    let (observer, spec) =
        build_l11_boss_c_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
