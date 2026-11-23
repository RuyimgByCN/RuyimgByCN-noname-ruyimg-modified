use super::chars_c::*;
use super::chars_e::*;
use super::chars_e2::*;
use super::helpers::*;
use super::super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(in crate::content::host) fn ask_char_c(
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = current_ask_id(core, owner, HostBehavior::U05GujianCharBatchC);
    let obs = current_ask_obs(core, owner, HostBehavior::U05GujianCharBatchC);
    match (stage, ask_id) {
        (HostStage::SelectCard, ASK_CHIZHEN_DISCARD) => {
            let n = card.unwrap_or(1) as usize;
            let opts: Vec<ChoiceOption> = hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let cnt = n.min(opts.len());
            build_ask(obs, "gj_chizhen_discard", ChoiceKind::SelectCards, opts, cnt, cnt, false)
        }
        (HostStage::TargetChoice, ASK_CHIZHEN_DUEL_TARGET) => build_ask(
            obs,
            "gj_chizhen_duel_target",
            ChoiceKind::SelectPlayers,
            core.alive_players()
                .filter(|&p| p != owner && can_target_card(core, catalog, owner, p, CardKind::Duel))
                .map(ChoiceOption::Player)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_DANGPING_CARD) => build_ask(
            obs,
            "gj_dangping_card",
            ChoiceKind::SelectCards,
            hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_DANGPING_TARGET) => {
            let injured = card.unwrap_or(0) as PlayerId;
            build_ask(
                obs,
                "gj_dangping_target",
                ChoiceKind::SelectPlayers,
                dangping_targets(core, catalog, owner, injured)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect(),
                1,
                1,
                true,
            )
        }
        (HostStage::SelectCard, ASK_YUNYIN_CARD) => build_ask(
            obs,
            "gj_yunyin_card",
            ChoiceKind::SelectCards,
            yunyin_discardable_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_YUNYIN_VCARD) => build_ask(
            obs,
            "gj_yunyin_vcard",
            ChoiceKind::PickTag,
            YUNYIN_VCARDS
                .iter()
                .enumerate()
                .filter_map(|(i, &k)| {
                    if yunyin_vcard_targets(core, catalog, owner, k).is_empty() {
                        None
                    } else {
                        Some(ChoiceOption::Tag(i as u8))
                    }
                })
                .collect(),
            1,
            1,
            false,
        ),
        (HostStage::DrawSettle, ASK_YUNYIN_TARGET) => {
            let kind = YUNYIN_VCARDS.get(card.unwrap_or(0) as usize).copied().unwrap_or(CardKind::Sha);
            build_ask(
                obs,
                "gj_yunyin_target",
                ChoiceKind::SelectPlayers,
                yunyin_vcard_targets(core, catalog, owner, kind)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect(),
                1,
                1,
                false,
            )
        }
        (HostStage::TargetChoice, ASK_DUHUN_TARGET) => build_ask(
            obs,
            "gj_duhun_target",
            ChoiceKind::SelectPlayers,
            duhun_candidates(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::ComparePlayerPick, ASK_DUHUN_COMPARE_OWN) => build_ask(
            obs,
            GJ_BINGPO_SHOW_OWN,
            ChoiceKind::SelectCards,
            core.player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            false,
        ),
        (HostStage::CompareTargetPick, ASK_DUHUN_COMPARE_TARGET) => build_ask(
            _target.unwrap_or(obs),
            GJ_BINGPO_SHOW_TARGET,
            ChoiceKind::SelectCards,
            core.player(_target.unwrap_or(obs))
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_XUANNING1_CARD) => build_ask(
            obs,
            "gj_xuanning1_card",
            ChoiceKind::SelectCards,
            core.player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_LIUGUANG_TARGETS) => {
            let opts = push_non_self_alive(core, owner);
            let max = opts.len().min(3);
            build_ask(obs, "gj_liuguang_targets", ChoiceKind::SelectPlayers, opts, 1, max, true)
        }
        (HostStage::SelectCard, ASK_LIUGUANG_RESPOND) => build_ask(
            _target.unwrap_or(obs),
            "gj_liuguang_respond",
            ChoiceKind::SelectCards,
            hand_and_equips(core, _target.unwrap_or(obs))
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_YANGMING_CARD) => build_ask(
            obs,
            "gj_yangming_card",
            ChoiceKind::SelectCards,
            core.player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| is_red(catalog, c))
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_YANGMING2_TARGETS) => {
            let cands = yangming2_targets(core, catalog, owner);
            let max = cands.len();
            build_ask(
                obs,
                "gj_yangming2_targets",
                ChoiceKind::SelectPlayers,
                cands.into_iter().map(ChoiceOption::Player).collect(),
                1,
                max,
                true,
            )
        }
        _ => {
            let _ = cards;
            None
        }
    }
}

pub(in crate::content::host) fn ask_char_e(
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = current_ask_id(core, owner, HostBehavior::U05GujianCharBatchE);
    let obs = current_ask_obs(core, owner, HostBehavior::U05GujianCharBatchE);
    match (stage, ask_id) {
        (HostStage::SelectCard, ASK_LINGYAN_CARD) => build_ask(
            obs,
            "gj_lingyan_card",
            ChoiceKind::SelectCards,
            lingyan_eligible_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_XUANCI_CARD) => build_ask(
            obs,
            "gj_xuanci_card",
            ChoiceKind::SelectCards,
            hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| catalog.cards[c as usize].suit == crate::catalog::Suit::Club)
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_XUANCI_TARGET) => build_ask(
            obs,
            "gj_xuanci_target",
            ChoiceKind::SelectPlayers,
            core.alive_players()
                .filter(|&p| p != owner && can_target_card(core, catalog, owner, p, CardKind::Feibiao))
                .map(ChoiceOption::Player)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_WOXUE_CONFIRM) => build_ask(
            obs,
            "gj_woxue_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_XUANZHEN_CONFIRM) => build_ask(
            obs,
            "gj_xuanzhen_confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::TargetChoice, ASK_YANGE_TARGET) => build_ask(
            obs,
            "gj_yange_target",
            ChoiceKind::SelectPlayers,
            push_non_self_alive(core, owner),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_LIANJING_TARGETS) => {
            let opts = push_non_self_alive(core, owner);
            let max = opts.len().min(2);
            build_ask(obs, "gj_lianjing_targets", ChoiceKind::SelectPlayers, opts, 1, max, true)
        }
        (HostStage::TargetChoice, ASK_CIHONG_TARGET) => build_ask(
            obs,
            "gj_cihong_target",
            ChoiceKind::SelectPlayers,
            core.alive_players()
                .filter(|&p| p != owner && can_target_card(core, catalog, owner, p, CardKind::Sha))
                .map(ChoiceOption::Player)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::SelectCard, ASK_CIHONG_RED) => build_ask(
            obs,
            "gj_cihong_red",
            ChoiceKind::SelectCards,
            hand_and_equips(core, owner)
                .into_iter()
                .filter(|&c| is_red(catalog, c))
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::ComparePlayerPick, ASK_CIHONG_LOSEHP) => build_ask(
            obs,
            "gj_cihong_losehp",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::CompareTargetPick, ASK_CIHONG_TURNOVER) => build_ask(
            obs,
            "gj_cihong_turnover",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_ZHENYING_DISCARD) => build_ask(
            obs,
            "gj_zhenying_discard",
            ChoiceKind::SelectCards,
            hand_and_equips(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect(),
            1,
            1,
            false,
        ),
        (HostStage::SelectCard, ASK_QINGSHU_VCARD) => build_ask(
            obs,
            "gj_qingshu_vcard",
            ChoiceKind::PickTag,
            qingshu_available_lands(core, owner)
                .into_iter()
                .filter_map(|k| land_kind_to_idx(k).map(|i| ChoiceOption::Tag(i as u8)))
                .collect(),
            1,
            1,
            true,
        ),
        (HostStage::TargetChoice, ASK_QINGSHU_TARGET) => {
            let kind = _card.and_then(idx_to_land_kind)?;
            let keys = land_skill_keys(kind);
            build_ask(
                obs,
                "gj_qingshu_target",
                ChoiceKind::SelectPlayers,
                core.alive_players()
                    .filter(|&p| keys.first().is_some_and(|k| !has_skill(core, catalog, p, k)))
                    .map(ChoiceOption::Player)
                    .collect(),
                1,
                1,
                true,
            )
        }
        (HostStage::SelectCard, ASK_YUNYOU_DISCOVER) => build_ask(
            obs,
            "gj_yunyou_discover",
            ChoiceKind::PickTag,
            cards.iter().copied().map(|v| ChoiceOption::Tag(v as u8)).collect(),
            1,
            1,
            false,
        ),
        (HostStage::TargetChoice, ASK_XUANZHEN_VCARD) => build_ask(
            obs,
            "gj_xuanzhen_vcard",
            ChoiceKind::PickTag,
            cards.iter().copied().map(|v| ChoiceOption::Tag(v as u8)).collect(),
            1,
            1,
            false,
        ),
        _ => None,
    }
}
