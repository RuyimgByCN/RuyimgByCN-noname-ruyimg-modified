//! U05 `杀海拾遗 · swd` Batch 7b 快照重构询问描述子模块（与 `asks.rs` 分表挂载，禁止超行）。

use super::lingjian::{equip5_targets, forgeable_equips, is_forge_material};
use super::{black_hand_cards, kongxin_sha_targets, respondable_shans};
use super::super::swd_u05_7a::hand_or_equip;
use super::super::super::*;
use crate::catalog::Catalog;
use crate::ids::{CardId, PlayerId};
use crate::snap::{ask as build_ask, ExpectedAsk};
use crate::state::GameStateCore;

pub(super) fn wait_select_cards(
    observer: PlayerId,
    prompt: &'static str,
    cards: Vec<CardId>,
    min: usize,
    max: usize,
    allow_cancel: bool,
) -> Flow {
    Flow::WaitChoice {
        observer,
        spec: ChoiceSpec {
            kind: ChoiceKind::SelectCards,
            prompt,
            options: cards.into_iter().map(ChoiceOption::Card).collect(),
            min,
            max,
            allow_cancel,
        },
    }
}

pub(super) fn wait_select_players(
    observer: PlayerId,
    prompt: &'static str,
    players: Vec<PlayerId>,
    allow_cancel: bool,
) -> Flow {
    Flow::WaitChoice {
        observer,
        spec: ChoiceSpec {
            kind: ChoiceKind::SelectPlayers,
            prompt,
            options: players.into_iter().map(ChoiceOption::Player).collect(),
            min: 1,
            max: 1,
            allow_cancel,
        },
    }
}

pub(super) fn wait_confirm(observer: PlayerId, prompt: &'static str) -> Flow {
    Flow::WaitChoice {
        observer,
        spec: ChoiceSpec {
            kind: ChoiceKind::Confirm,
            prompt,
            options: vec![ChoiceOption::Confirm],
            min: 0,
            max: 1,
            allow_cancel: true,
        },
    }
}

pub(in crate::content::host) fn ask(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let cards_ask = |obs: PlayerId, prompt: &'static str, opts: Vec<CardId>, min: usize, max: usize, cancel: bool| {
        Some(build_ask(
            obs,
            prompt,
            ChoiceKind::SelectCards,
            opts.into_iter().map(ChoiceOption::Card).collect(),
            min,
            max,
            cancel,
        ))
    };
    let players_ask = |obs: PlayerId, prompt: &'static str, opts: Vec<PlayerId>, cancel: bool| {
        Some(build_ask(
            obs,
            prompt,
            ChoiceKind::SelectPlayers,
            opts.into_iter().map(ChoiceOption::Player).collect(),
            1,
            1,
            cancel,
        ))
    };
    let confirm_ask = |obs: PlayerId, prompt: &'static str| {
        Some(build_ask(
            obs,
            prompt,
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            0,
            1,
            true,
        ))
    };
    match (behavior, stage) {
        (HostBehavior::U05SwdXiangyuyeCard, HostStage::SelectCard) => {
            let t = target?;
            cards_ask(t, "u05_xiangyuye_discard", black_hand_cards(core, catalog, t), 1, 1, true)
        }
        (HostBehavior::U05SwdKongxin, HostStage::TargetChoice) => {
            let opts: Vec<PlayerId> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).hand.is_empty())
                .collect();
            players_ask(owner, "u05_kongxin_target", opts, true)
        }
        (HostBehavior::U05SwdKongxin, HostStage::ComparePlayerPick) => {
            cards_ask(owner, "u05_kongxin_pindian_own", core.player(owner).hand.clone(), 1, 1, false)
        }
        (HostBehavior::U05SwdKongxin, HostStage::CompareTargetPick) => {
            let t = target?;
            cards_ask(t, "u05_kongxin_pindian_target", core.player(t).hand.clone(), 1, 1, false)
        }
        (HostBehavior::U05SwdKongxin, HostStage::SelectNature) => {
            let t = target?;
            players_ask(owner, "u05_kongxin_sha_target", kongxin_sha_targets(core, catalog, owner, t), true)
        }
        (HostBehavior::U05SwdKongxin, HostStage::CardsAsk) => {
            let t = target?;
            cards_ask(t, "u05_kongxin_lose_discard", hand_or_equip(core, owner), 1, 1, false)
        }
        (HostBehavior::U05SwdLingjianDmgSrc, HostStage::SelectNature) => {
            confirm_ask(owner, "u05_zhongxing_equip1_confirm")
        }
        (HostBehavior::U05SwdLingjianDmgSrc, HostStage::DrawSettle) => {
            confirm_ask(owner, "u05_jileng_equip1_confirm")
        }
        (HostBehavior::U05SwdLingjianDmgSrc, HostStage::TargetChoice) => {
            let opts: Vec<PlayerId> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).yinmi_active)
                .collect();
            players_ask(owner, "u05_yinmi_equip1_target", opts, true)
        }
        (HostBehavior::U05SwdLingjianDmgTgt, HostStage::SelectNature) => {
            confirm_ask(owner, "u05_zhongxing_equip2_confirm")
        }
        (HostBehavior::U05SwdLingjianDmgTgt, HostStage::DrawSettle) => {
            confirm_ask(owner, "u05_jileng_equip2_confirm")
        }
        (HostBehavior::U05SwdLingjianPhaseEnd, HostStage::TargetChoice) => {
            let opts: Vec<PlayerId> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).face_down)
                .collect();
            players_ask(owner, "u05_jileng_equip5_target", opts, true)
        }
        (HostBehavior::U05SwdLingjianPhaseUse, HostStage::SelectCard) => {
            cards_ask(owner, "u05_lingjian_e5_cost", hand_or_equip(core, owner), 1, 1, true)
        }
        (HostBehavior::U05SwdLingjianPhaseUse, HostStage::CardsAsk) => {
            cards_ask(owner, "u05_lingjian_e5_cost", hand_or_equip(core, owner), 2, 2, true)
        }
        (HostBehavior::U05SwdLingjianPhaseUse, HostStage::TargetChoice) => {
            let opts = equip5_targets(core, owner, "hslingjian_shijianhuisu_equip5");
            players_ask(owner, "u05_lingjian_e5_target", opts, true)
        }
        (HostBehavior::U05SwdLingjianPhaseUse, HostStage::CompareTargetPick) => {
            let opts = equip5_targets(core, owner, "hslingjian_xuanfengzhiren_equip5");
            players_ask(owner, "u05_lingjian_e5_target", opts, true)
        }
        (HostBehavior::U05SwdLingjianPhaseUse, HostStage::ComparePlayerPick) => {
            let opts = equip5_targets(core, owner, "hslingjian_zhongxinghujia_equip5");
            players_ask(owner, "u05_lingjian_e5_target", opts, true)
        }
        (HostBehavior::U05SwdForgeAction, HostStage::SelectCard) => {
            cards_ask(owner, "u05_duanzao_equip", forgeable_equips(core, catalog, owner), 1, 1, true)
        }
        (HostBehavior::U05SwdForgeAction, HostStage::CardsAsk) => {
            let mats: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| is_forge_material(catalog, *c))
                .collect();
            cards_ask(owner, "u05_duanzao_mat", mats, 1, 1, true)
        }
        (HostBehavior::U05SwdForgeAction, HostStage::TargetChoice) => {
            let opts: Vec<PlayerId> = core
                .alive_players()
                .filter(|&p| core.computed_distance(catalog, owner, p) <= 1)
                .collect();
            players_ask(owner, "u05_duanzao_target", opts, true)
        }
        (HostBehavior::U05SwdQinglonglingzhu, HostStage::SelectCard) => {
            let t = target?;
            cards_ask(owner, "u05_qinglonglingzhu_gain", hand_or_equip(core, t), 0, 1, true)
        }
        (HostBehavior::U05SwdBaihupifeng, HostStage::SelectNature) => {
            confirm_ask(owner, "u05_baihupifeng_recover")
        }
        (HostBehavior::U05SwdFengxueren, HostStage::SelectNature) => {
            confirm_ask(owner, "u05_fengxueren_confirm")
        }
        (HostBehavior::U05SwdOldLongfan, HostStage::TargetChoice) => {
            players_ask(owner, "u05_old_longfan_target", core.alive_players().collect(), true)
        }
        (HostBehavior::U05SwdSadengjinhuan, HostStage::SelectNature) => {
            confirm_ask(owner, "sadengjinhuan_judge_confirm")
        }
        (HostBehavior::U05SwdSadengjinhuan, HostStage::CardsAsk) => {
            let t = target?;
            cards_ask(t, "sadengjinhuan_extra_shan", respondable_shans(core, catalog, t), 0, 1, false)
        }
        (HostBehavior::U05SwdGuiyanfadao, HostStage::SelectNature) => {
            confirm_ask(owner, "u05_guiyanfadao_confirm")
        }
        _ => None,
    }
}
