//! L11 太虚幻境 `effectPack` 与 `txhj_yingjiuhandi` SP2 武将技询问构造（`l11_effect_pack_asks.rs`）。

use super::l11_common::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

pub(super) const ASK_EP_TANSHI_DISCARD: u32 = 101;
pub(super) const ASK_EP_XINGSHANG_CONFIRM: u32 = 102;
pub(super) const ASK_EP_HENGZHENG_CONFIRM: u32 = 103;
pub(super) const ASK_EP_HENGZHENG_ZONE: u32 = 104;
pub(super) const ASK_EP_HENGZHENG_CARD: u32 = 105;
pub(super) const ASK_EP_YISUAN_CONFIRM: u32 = 106;
pub(super) const ASK_EP_LVEMING_TARGET: u32 = 107;
pub(super) const ASK_EP_LVEMING_RANK: u32 = 108;
pub(super) const ASK_EP_TUNJUN_TARGET: u32 = 109;
pub(super) const ASK_EP_TANBEI_TARGET: u32 = 110;
pub(super) const ASK_EP_TANBEI_MODE: u32 = 111;
pub(super) const ASK_EP_SIDAO_CARD: u32 = 112;
pub(super) const ASK_EP_SIDAO_TARGET: u32 = 113;

pub(super) fn build_l11_effect_pack_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    cards: &[CardId],
    core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_EP_TANSHI_DISCARD => {
            let hand = core.player(owner).hand.clone();
            if hand.is_empty() {
                return None;
            }
            card_ask(owner, "txhj_tanshi:discard_card", hand, 1, 1, false)
        }
        ASK_EP_XINGSHANG_CONFIRM => confirm_ask(owner, "xingshang:confirm"),
        ASK_EP_HENGZHENG_CONFIRM => confirm_ask(owner, "hengzheng:confirm"),
        ASK_EP_HENGZHENG_ZONE => {
            let t = target.filter(|&p| core.player(p).alive)?;
            let pl = core.player(t);
            let mut opts = Vec::new();
            if !pl.hand.is_empty() {
                opts.push(ChoiceOption::Tag(0));
            }
            if !pl.equips.is_empty() {
                opts.push(ChoiceOption::Tag(1));
            }
            if !pl.judge.is_empty() {
                opts.push(ChoiceOption::Tag(2));
            }
            if opts.is_empty() {
                return None;
            }
            spec_ask(
                owner,
                ChoiceKind::PickTag,
                "hengzheng:choose_zone",
                opts,
                1,
                1,
                false,
            )
        }
        ASK_EP_HENGZHENG_CARD => {
            let t = target.filter(|&p| core.player(p).alive)?;
            let zone = cards.first().copied().unwrap_or(1);
            let pl = core.player(t);
            let pool = match zone {
                1 => pl.equips.clone(),
                2 => pl.judge.clone(),
                _ => Vec::new(),
            };
            if pool.is_empty() {
                return None;
            }
            card_ask(owner, "hengzheng:choose_card", pool, 1, 1, false)
        }
        ASK_EP_YISUAN_CONFIRM => confirm_ask(owner, "xinfu_yisuan:confirm"),
        ASK_EP_LVEMING_TARGET => {
            let my_eq = core.player(owner).equips.len();
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).equips.len() < my_eq)
                .collect();
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "xinfu_lveming:choose_target", tgts, 1, 1, true)
        }
        ASK_EP_LVEMING_RANK => {
            let t = target.filter(|&p| core.player(p).alive)?;
            tag_ask(t, "xinfu_lveming:choose_rank", 0..=12, false)
        }
        ASK_EP_TUNJUN_TARGET => {
            let tgts = alive_all(core, owner);
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "xinfu_tunjun:choose_target", tgts, 1, 1, true)
        }
        ASK_EP_TANBEI_TARGET => {
            let tgts = alive_others(core, owner);
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "xinfu_tanbei:choose_target", tgts, 1, 1, true)
        }
        ASK_EP_TANBEI_MODE => {
            let t = target.filter(|&p| core.player(p).alive)?;
            tag_ask(t, "xinfu_tanbei:choose_mode", 0..=1, false)
        }
        ASK_EP_SIDAO_CARD => {
            if cards.is_empty() {
                return None;
            }
            card_ask(owner, "xinfu_sidao:choose_card", cards.to_vec(), 1, 1, true)
        }
        ASK_EP_SIDAO_TARGET => {
            if targets.is_empty() {
                return None;
            }
            player_ask(
                owner,
                "xinfu_sidao:choose_target",
                targets.to_vec(),
                1,
                1,
                true,
            )
        }
        _ => None,
    }
}
