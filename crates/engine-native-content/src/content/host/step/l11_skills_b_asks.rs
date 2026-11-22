//! L11（太虚幻境）武将技能 Batch B（S44–S57）的询问构建（`l11_skills_b_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::{Catalog, EquipSlot};
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_ZHUILING_CONFIRM: u32 = 1;
pub(super) const ASK_XIESHU_DISCARD: u32 = 2;
pub(super) const ASK_SHIXIN_CONFIRM: u32 = 3;
pub(super) const ASK_MOUDUAN_CONFIRM: u32 = 4;
pub(super) const ASK_MOUDUAN_FROM: u32 = 5;
pub(super) const ASK_MOUDUAN_CARD: u32 = 6;
pub(super) const ASK_MOUDUAN_TO: u32 = 7;
pub(super) const ASK_KAIKANG_CONFIRM: u32 = 8;
pub(super) const ASK_KAIKANG_GIVE: u32 = 9;
pub(super) const ASK_KAIKANG_EQUIP: u32 = 10;
pub(super) const ASK_FEIYAN_CONFIRM: u32 = 11;
pub(super) const ASK_FEIYAN_SHA: u32 = 12;
pub(super) const ASK_LEILI_TARGET: u32 = 13;
pub(super) const ASK_KUANGXI_TARGET: u32 = 14;

pub(super) fn card_name_len(catalog: &Catalog, cid: CardId) -> usize {
    let Some(cd) = catalog.cards.get(cid as usize) else {
        return 2;
    };
    let n = cd.name;
    let ch_count = n.chars().filter(|c| !c.is_ascii()).count();
    if ch_count > 0 {
        return ch_count;
    }
    match n {
        "txhj_xingtianpojunfu" | "txhj_dinglanyemingzhu" | "txhj_chiyanzhenhunqin" => 5,
        "txhj_qilingong" | "txhj_piliche" | "txhj_leijimu" | "txminguangkai" => 3,
        "txhj_baihu" | "txzhaoshu_skill" | "txzhaoshu_global" | "zhaoshu" | "huogong" => 2,
        "guding" | "gudingdao" | "bagua" | "tengjia" | "baiyin" | "zhuque" => 3,
        "zhuge" | "fangtian" | "zhangba" | "muniu" | "shuigong" => 4,
        _ => match cd.kind {
            CardKind::Sha
            | CardKind::FireSha
            | CardKind::LeiSha
            | CardKind::Shan
            | CardKind::Tao
            | CardKind::Jiu => 1,
            CardKind::Duel
            | CardKind::ShanDian
            | CardKind::HongShui
            | CardKind::HorsePlus
            | CardKind::HorseMinus
            | CardKind::Shuigong
            | CardKind::Feibiao
            | CardKind::Caoyao
            | CardKind::Hufu
            | CardKind::Yougeng
            | CardKind::Qingtuan
            | CardKind::Chunbing
            | CardKind::Molicha
            | CardKind::Xiajiao => 2,
            CardKind::Qinggang
            | CardKind::Qilin
            | CardKind::Hanbing
            | CardKind::Guanshi
            | CardKind::Renwang
            | CardKind::Armor
            | CardKind::WuxingPan
            | CardKind::ShouLiJian
            | CardKind::Sanjian
            | CardKind::Suolianjia
            | CardKind::Zhaomingdan
            | CardKind::Jihuocard
            | CardKind::Shujinsan
            | CardKind::Jinlianzhu
            | CardKind::Tanhuadong
            | CardKind::Yuanbaorou
            | CardKind::Gudonggeng
            | CardKind::Liyutang
            | CardKind::Luyugeng => 3,
            CardKind::Qinglong | CardKind::Cixiong => 5,
            _ => 4,
        },
    }
}

pub(super) fn field_cards_of(core: &GameStateCore, p: PlayerId) -> Vec<CardId> {
    core.player(p)
        .equips
        .iter()
        .chain(core.player(p).judge.iter())
        .copied()
        .collect()
}

pub(super) fn can_receive_moved_card(
    core: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
    card: CardId,
) -> bool {
    if from == to || !core.player(to).alive {
        return false;
    }
    if core.player(from).equips.contains(&card) {
        let Some(slot) = catalog.card_slot_of(core, card) else {
            return false;
        };
        if catalog.cards.get(card as usize).is_some_and(|cd| cd.name == "txhj_liulongcanjia") {
            let has_horse = core.player(to).equips.iter().any(|&c| {
                matches!(
                    catalog.card_slot_of(core, c),
                    Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
                )
            });
            if has_horse {
                return false;
            }
        }
        if matches!(slot, EquipSlot::HorsePlus | EquipSlot::HorseMinus)
            && has_equip_named(core, catalog, to, "txhj_liulongcanjia")
        {
            return false;
        }
        if slot == EquipSlot::Treasure && has_named_skill(core, catalog, to, "_txhj_equip5") {
            let treasure_cnt = core
                .player(to)
                .equips
                .iter()
                .filter(|&&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Treasure))
                .count();
            return treasure_cnt < 2;
        }
        return !core
            .player(to)
            .equips
            .iter()
            .any(|&c| catalog.card_slot_of(core, c) == Some(slot));
    }
    if core.player(from).judge.contains(&card) {
        let kind = catalog.card_kind(card);
        return !core
            .player(to)
            .judge
            .iter()
            .any(|&c| catalog.card_kind(c) == kind);
    }
    false
}

pub(super) fn movable_field_sources(
    core: &GameStateCore,
    catalog: &Catalog,
    start: PlayerId,
) -> Vec<PlayerId> {
    let all = alive_all(core, start);
    all.iter()
        .copied()
        .filter(|&from| {
            field_cards_of(core, from).into_iter().any(|c| {
                all.iter()
                    .any(|&to| can_receive_moved_card(core, catalog, from, to, c))
            })
        })
        .collect()
}

pub(super) fn build_l11_skills_b_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_ZHUILING_CONFIRM => confirm_ask(owner, "diyzhuiling:confirm"),
        ASK_XIESHU_DISCARD => {
            let need = cards.first().copied().unwrap_or(1) as usize;
            card_ask(
                owner,
                "txclanxieshu:discard_cards",
                he_cards(core, owner),
                need,
                need,
                true,
            )
        }
        ASK_SHIXIN_CONFIRM => confirm_ask(owner, "txshixin:confirm"),
        ASK_MOUDUAN_CONFIRM => confirm_ask(owner, "txmouduan:confirm"),
        ASK_MOUDUAN_FROM => player_ask(
            owner,
            "txmouduan:move_from",
            movable_field_sources(core, catalog, owner),
            1,
            1,
            true,
        ),
        ASK_MOUDUAN_CARD => {
            let all = alive_all(core, owner);
            let mv: Vec<CardId> = field_cards_of(core, tgt)
                .into_iter()
                .filter(|&c| {
                    all.iter()
                        .any(|&to| can_receive_moved_card(core, catalog, tgt, to, c))
                })
                .collect();
            card_ask(owner, "txmouduan:move_card", mv, 1, 1, true)
        }
        ASK_MOUDUAN_TO => {
            let moved = cards.first().copied()?;
            let dests: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&to| can_receive_moved_card(core, catalog, tgt, to, moved))
                .collect();
            player_ask(owner, "txmouduan:move_to", dests, 1, 1, false)
        }
        ASK_KAIKANG_CONFIRM => confirm_ask(owner, "txkaikang:confirm"),
        ASK_KAIKANG_GIVE => card_ask(owner, "txkaikang:choose_card", he_cards(core, owner), 1, 1, false),
        ASK_KAIKANG_EQUIP => confirm_ask(tgt, "txkaikang:choose_equip"),
        ASK_FEIYAN_CONFIRM => confirm_ask(owner, "txfeiyan:confirm"),
        ASK_FEIYAN_SHA => {
            let shas: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| effective_card_kind(core, catalog, c).is_sha_like())
                .collect();
            card_ask(owner, "txfeiyan:choose_sha", shas, 1, 1, true)
        }
        ASK_LEILI_TARGET => {
            let enemies: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| Some(p) != target && !core.is_friend_of(catalog, owner, p))
                .collect();
            player_ask(owner, "txleili:choose_target", enemies, 1, 1, true)
        }
        ASK_KUANGXI_TARGET => player_ask(
            owner,
            "cxyKuangXi:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_skills_b(
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
        build_l11_skills_b_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
