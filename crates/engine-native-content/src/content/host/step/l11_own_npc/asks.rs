//! L11-6a 太虚自有 NPC 技能询问构建（`l11_own_npc/asks.rs`）。

use super::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_TUNSHI_TARGET: u32 = 1;
pub(super) const ASK_CHUANYUN_TARGET: u32 = 2;
pub(super) const ASK_GUIHUO_TARGET: u32 = 3;
pub(super) const ASK_LUOLEI_TARGET: u32 = 4;
pub(super) const ASK_HUANGJIN_CONFIRM: u32 = 5;
pub(super) const ASK_ZHOUZU_TARGET: u32 = 6;
pub(super) const ASK_ZHOUZU_DUR: u32 = 7;
pub(super) const ASK_ZHOUZU_DISCARD: u32 = 8;
pub(super) const ASK_GUIMEN_TARGET: u32 = 9;
pub(super) const ASK_YAOSHU_TARGET: u32 = 10;
pub(super) const ASK_YAOSHU_DUR: u32 = 11;
pub(super) const ASK_ZHOUFA_TARGET: u32 = 12;
pub(super) const ASK_ZHOUFA_DUR: u32 = 13;
pub(super) const ASK_RENFANG_DUR: u32 = 14;
pub(super) const ASK_RENFANG_TARGET: u32 = 15;
pub(super) const ASK_DIZHOU_CARD: u32 = 16;
pub(super) const ASK_ZHUHUN_TARGET: u32 = 17;
pub(super) const ASK_GUIYAN_TARGET: u32 = 18;
pub(super) const ASK_XIEFENG_DISCARD: u32 = 19;
pub(super) const ASK_YOUJI_CONFIRM: u32 = 20;

fn ask_spec(
    observer: PlayerId,
    prompt: &'static str,
    kind: ChoiceKind,
    options: Vec<ChoiceOption>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> ExpectedAsk {
    ExpectedAsk {
        observer,
        kind,
        prompt,
        options,
        min,
        max,
        allow_cancel,
        source_skill: None,
    }
}

fn duration_options() -> Vec<ChoiceOption> {
    (0..3).map(ChoiceOption::Tag).collect()
}

pub(in crate::content::host) fn ask_l11_own_npc(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = card?;
    let opt_players = |iter: Vec<PlayerId>| -> Vec<ChoiceOption> {
        iter.into_iter().map(ChoiceOption::Player).collect()
    };
    let opt_cards = |cards: Vec<CardId>| -> Vec<ChoiceOption> {
        cards.into_iter().map(ChoiceOption::Card).collect()
    };
    match (ask_id, stage) {
        (ASK_TUNSHI_TARGET, HostStage::TargetChoice) => {
            let own_h = core.player(owner).hand.len();
            let opts = opt_players(
                alive_others(core, owner)
                    .into_iter()
                    .filter(|&p| !core.is_friend_of(catalog, owner, p) && core.player(p).hand.len() > own_h)
                    .collect(),
            );
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_tunshi:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)
            })
        }
        (ASK_CHUANYUN_TARGET, HostStage::TargetChoice) => {
            let own_hp = core.player(owner).hp;
            let opts = opt_players(
                alive_others(core, owner)
                    .into_iter()
                    .filter(|&p| !core.is_friend_of(catalog, owner, p) && core.player(p).hp > own_hp)
                    .collect(),
            );
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_chuanyun:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        (ASK_GUIHUO_TARGET, HostStage::TargetChoice) => {
            let opts = opt_players(alive_others(core, owner));
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_guihuo:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        (ASK_LUOLEI_TARGET, HostStage::TargetChoice) => {
            let opts = opt_players(alive_others(core, owner));
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_luolei:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        (ASK_HUANGJIN_CONFIRM, HostStage::CardsAsk) => Some(ask_spec(
            owner,
            "txhj_huangjin:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        )),
        (ASK_ZHOUZU_TARGET, HostStage::TargetChoice) => {
            let opts = opt_players(alive_others(core, owner));
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_zhouzu:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        (ASK_ZHOUZU_DUR, HostStage::SelectNature) => Some(ask_spec(
            owner,
            "txhj_zhouzu:choose_duration",
            ChoiceKind::PickTag,
            duration_options(),
            1,
            1,
            false,
        )),
        (ASK_ZHOUZU_DISCARD, HostStage::SelectCard) => {
            let t = target?;
            let he = he_cards(core, t);
            let need = (flag(core, owner, "l11_mahou_zhouzu_disc_n").max(1) as usize).min(he.len());
            if need == 0 {
                return None;
            }
            Some(ask_spec(
                t,
                "txhj_zhouzu:discard_cards",
                ChoiceKind::SelectCards,
                opt_cards(he),
                need,
                need,
                false,
            ))
        }
        (ASK_GUIMEN_TARGET, HostStage::TargetChoice) => {
            let opts = opt_players(alive_others(core, owner));
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_guimen:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)
            })
        }
        (ASK_YAOSHU_TARGET, HostStage::TargetChoice) => {
            let opts = opt_players(
                alive_others(core, owner)
                    .into_iter()
                    .filter(|&p| flag(core, p, "l11_yaoshu_mahou_rem") == 0)
                    .collect(),
            );
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_yaoshu:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        (ASK_YAOSHU_DUR, HostStage::SelectNature) => Some(ask_spec(
            owner,
            "txhj_yaoshu:choose_duration",
            ChoiceKind::PickTag,
            duration_options(),
            1,
            1,
            false,
        )),
        (ASK_ZHOUFA_TARGET, HostStage::TargetChoice) => {
            let opts = opt_players(alive_others(core, owner));
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_zhoufa:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        (ASK_ZHOUFA_DUR, HostStage::SelectNature) => Some(ask_spec(
            owner,
            "txhj_zhoufa:choose_duration",
            ChoiceKind::PickTag,
            duration_options(),
            1,
            1,
            false,
        )),
        (ASK_RENFANG_DUR, HostStage::SelectNature) => Some(ask_spec(
            owner,
            "txhj_renfang:choose_duration",
            ChoiceKind::PickTag,
            duration_options(),
            1,
            1,
            false,
        )),
        (ASK_RENFANG_TARGET, HostStage::TargetChoice) => {
            let opts = opt_players(
                alive_others(core, owner)
                    .into_iter()
                    .filter(|p| !targets.contains(p))
                    .collect(),
            );
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_renfang:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)
            })
        }
        (ASK_DIZHOU_CARD, HostStage::SelectCard) => {
            let opts = opt_cards(he_cards(core, owner));
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_dizhou:choose_card", ChoiceKind::SelectCards, opts, 1, 1, true)
            })
        }
        (ASK_ZHUHUN_TARGET, HostStage::TargetChoice) => {
            let mask = flag(core, owner, "l11_zhuhun_dead_mask");
            let dead: Vec<PlayerId> = (0..core.players.len() as PlayerId)
                .filter(|&p| p != owner && !core.player(p).alive && (mask & (1u32 << p)) != 0)
                .collect();
            let opts = opt_players(dead);
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_zhuhun:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, true)
            })
        }
        (ASK_GUIYAN_TARGET, HostStage::TargetChoice) => {
            let own_hp = core.player(owner).hp;
            let opts = opt_players(
                alive_others(core, owner)
                    .into_iter()
                    .filter(|&p| core.player(p).hp >= own_hp)
                    .collect(),
            );
            (!opts.is_empty()).then(|| {
                ask_spec(owner, "txhj_guiyan:choose_target", ChoiceKind::SelectPlayers, opts, 1, 1, false)
            })
        }
        (ASK_XIEFENG_DISCARD, HostStage::SelectCard) => {
            let opts = opt_cards(he_cards(core, owner));
            let max_n = opts.len().min(3);
            (max_n > 0).then(|| {
                ask_spec(owner, "txhj_xiefeng:discard_cards", ChoiceKind::SelectCards, opts, 1, max_n, true)
            })
        }
        (ASK_YOUJI_CONFIRM, HostStage::CardsAsk) => Some(ask_spec(
            owner,
            "txqyyouji:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        )),
        _ => None,
    }
}
