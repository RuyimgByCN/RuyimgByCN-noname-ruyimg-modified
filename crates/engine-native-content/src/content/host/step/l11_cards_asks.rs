//! L11（太虚幻境）非武器牌结算的询问构建（`l11_cards_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_ZHIBI_CONFIRM: u32 = 1;
pub(super) const ASK_YIYI_DISCARD: u32 = 2;
pub(super) const ASK_JIEDAO_USE_SHA: u32 = 3;
pub(super) const ASK_CAOMU_DISCARD: u32 = 4;
pub(super) const ASK_SHENGDONG_GIVE1: u32 = 5;
pub(super) const ASK_SHENGDONG_GIVE2: u32 = 6;
pub(super) const ASK_SHUIYAN_DISCARD: u32 = 7;
pub(super) const ASK_JIUCHIROULIN_RESPOND: u32 = 8;
pub(super) const ASK_HUOGONG_SHOW: u32 = 9;
pub(super) const ASK_HUOGONG_DISCARD: u32 = 10;
pub(super) const ASK_ZHAOSHU_GLOBAL_TARGET: u32 = 11;
pub(super) const ASK_ZHAOSHU_GLOBAL_CARDS: u32 = 12;

pub(super) fn build_l11_card_ask(
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
        ASK_ZHIBI_CONFIRM => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::Confirm,
                prompt: "txhj_zhijizhibi:confirm",
                options: vec![ChoiceOption::Confirm],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        ASK_YIYI_DISCARD => {
            let pool = he_cards(core, tgt);
            let cnt = pool.len().min(2);
            card_ask(tgt, "txhj_yiyi:discard_cards", pool, cnt, cnt, false)
        }
        ASK_JIEDAO_USE_SHA => {
            let mut opts = vec![ChoiceOption::Pass];
            for &c in &core.player(tgt).hand {
                if effective_card_kind(core, catalog, c).is_sha_like() {
                    opts.push(ChoiceOption::Card(c));
                }
            }
            spec_ask(tgt, ChoiceKind::Confirm, "txhj_jiedao:choose_sha", opts, 1, 1, true)
        }
        ASK_CAOMU_DISCARD => {
            let pool = he_cards(core, tgt);
            card_ask(tgt, "txhj_caomu:discard_card", pool, 1, 1, false)
        }
        ASK_SHENGDONG_GIVE1 => card_ask(
            owner,
            "txhj_shengdong:give_card1",
            core.player(owner).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_SHENGDONG_GIVE2 => {
            let hand = core.player(tgt).hand.clone();
            let cnt = hand.len().min(2);
            card_ask(tgt, "txhj_shengdong:give_card2", hand, cnt, cnt, false)
        }
        ASK_SHUIYAN_DISCARD => {
            let eqs = core.player(tgt).equips.clone();
            let max = eqs.len();
            card_ask(tgt, "txhj_shuiyan:discard_equips", eqs, 1, max, true)
        }
        ASK_JIUCHIROULIN_RESPOND => {
            let mut opts = vec![ChoiceOption::Pass];
            for &c in &core.player(tgt).hand {
                if effective_card_kind(core, catalog, c) == CardKind::Jiu {
                    opts.push(ChoiceOption::Card(c));
                }
            }
            spec_ask(
                tgt,
                ChoiceKind::Confirm,
                "txhj_jiuchiroulin:respond_jiu",
                opts,
                1,
                1,
                true,
            )
        }
        ASK_HUOGONG_SHOW => card_ask(
            tgt,
            "huogong:show_card",
            core.player(tgt).hand.clone(),
            1,
            1,
            false,
        ),
        ASK_HUOGONG_DISCARD => {
            let show_card = cards.get(1).or(cards.first()).copied()?;
            let show_suit = catalog.effective_card_suit(core, tgt, show_card);
            let use_color = crate::rvm_bridge::modifiers::has_wolongmice(core, catalog, owner);
            let show_red = matches!(show_suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond);
            let matching: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    let s = catalog.effective_card_suit(core, owner, c);
                    if use_color {
                        matches!(s, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond)
                            == show_red
                    } else {
                        s == show_suit
                    }
                })
                .collect();
            card_ask(owner, "huogong:discard_card", matching, 1, 1, true)
        }
        ASK_ZHAOSHU_GLOBAL_TARGET => {
            let holders: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| {
                    core.is_friend_of(catalog, owner, p)
                        && has_named_skill(core, catalog, p, "txzhaoshu_skill")
                })
                .collect();
            player_ask(owner, "txzhaoshu_global:choose_target", holders, 1, 1, true)
        }
        ASK_ZHAOSHU_GLOBAL_CARDS => {
            let pl = core.player(owner);
            let max = if pl.hp < pl.max_hp as i32 { 2 } else { 1 };
            let cap = max.min(pl.hand.len());
            card_ask(
                owner,
                "txzhaoshu_global:give_cards",
                pl.hand.clone(),
                1,
                cap,
                true,
            )
        }
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_trick_card(
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
        build_l11_card_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
