//! Batch Q4 U02/L03 boss/jiange 技能家族（tag 870）交互询问构造（对齐 `l10_scs_asks` 契约）。

use super::{
    is_mech_seat, jiwu_available_skills, matching_judge_suit_cards, q4_boss_jiwu,
    q4_fengxing_targets, q4_keding_extra_targets, ASK_BAONU_FORM, ASK_BASHI_CONFIRM,
    ASK_DANJING_CONFIRM, ASK_FENGXING_TARGET, ASK_HUODI_TARGET, ASK_JIAOXIE_GIVE,
    ASK_JIAOXIE_TARGET, ASK_JIWU_CARD, ASK_JIWU_SKILL, ASK_KEDING_CARDS, ASK_KEDING_TARGETS,
    ASK_LEILI_TARGET, ASK_LINGFENG_TARGET, ASK_QIANGXIX_CARD, ASK_QIANGXIX_TARGET,
    ASK_QIWU_TARGET, ASK_RETIEJI_CONFIRM, ASK_RETIEJI_DISCARD, ASK_RETIEJI_GIVE,
    ASK_REXUANFENG_MODE,
    ASK_REXUANFENG_MOVE_CARD, ASK_REXUANFENG_MOVE_FROM, ASK_REXUANFENG_MOVE_TO,
    ASK_SHENQU_TAO, ASK_XIULUO_HAND, ASK_XIULUO_JUDGE, ASK_XUANFENG_CARD, ASK_XUANFENG_TARGET,
    ASK_ZHINANG_TARGET,
};
use crate::catalog::{Catalog, CardKind};
use crate::choice::{ChoiceKind, ChoiceOption};
use crate::content::host::HostStage;
use crate::ids::{CardId, PlayerId};
use crate::snap::{ask, ExpectedAsk};
use crate::state::GameStateCore;

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_q4_boss(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let ask_id = card?;
    match (ask_id, stage) {
        (ASK_ZHINANG_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = (0..core.players.len() as PlayerId)
                .filter(|&p| core.player(p).alive && core.is_same_team_or_friend(catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "boss_zhinang:target", ChoiceKind::SelectPlayers, opts, 1, 1, false))
        }
        (ASK_QIWU_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = (0..core.players.len() as PlayerId)
                .filter(|&p| {
                    let pl = core.player(p);
                    pl.alive && pl.hp < pl.max_hp as i32 && core.is_same_team_or_friend(catalog, owner, p)
                })
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "boss_qiwu:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_HUODI_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = (0..core.players.len() as PlayerId)
                .filter(|&p| core.player(p).alive && core.is_hostile_to(catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "boss_huodi:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_BASHI_CONFIRM, HostStage::CardsAsk) => Some(ask(
            owner,
            "boss_bashi:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        (ASK_DANJING_CONFIRM, HostStage::CardsAsk) => Some(ask(
            owner,
            "boss_danjing:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        (ASK_JIAOXIE_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = (0..core.players.len() as PlayerId)
                .filter(|&p| {
                    let pl = core.player(p);
                    pl.alive
                        && core.is_hostile_to(catalog, owner, p)
                        && is_mech_seat(core, catalog, p)
                        && (!pl.hand.is_empty() || !pl.equips.is_empty())
                })
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "boss_jiaoxie:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_JIAOXIE_GIVE, HostStage::SelectCard) => {
            let tgt = target?;
            let mut opts: Vec<ChoiceOption> = core
                .player(tgt)
                .hand
                .iter()
                .chain(core.player(tgt).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(tgt, "boss_jiaoxie:give_card", ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        (ASK_KEDING_CARDS, HostStage::SelectCard) => {
            let used_card = cards.first().copied();
            let max_extra = q4_keding_extra_targets(core, catalog, owner, used_card, targets).len();
            let cap = max_extra.min(core.player(owner).hand.len());
            if cap == 0 {
                return None;
            }
            let mut opts: Vec<ChoiceOption> = core.player(owner).hand.iter().copied().map(ChoiceOption::Card).collect();
            opts.truncate(64);
            Some(ask(owner, "boss_keding:cards", ChoiceKind::SelectCards, opts, 1, cap, true))
        }
        (ASK_KEDING_TARGETS, HostStage::CompareTargetPick) => {
            let used_card = cards.first().copied();
            let k = cards.len().saturating_sub(1);
            if k == 0 {
                return None;
            }
            let mut opts: Vec<ChoiceOption> = q4_keding_extra_targets(core, catalog, owner, used_card, targets)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (opts.len() >= k).then(|| ask(owner, "boss_keding:targets", ChoiceKind::SelectPlayers, opts, k, k, false))
        }
        (ASK_BAONU_FORM, HostStage::SelectNature) => Some(ask(
            owner,
            "boss_baonu:form",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        (ASK_XIULUO_HAND, HostStage::SelectCard) => {
            let mut opts: Vec<ChoiceOption> = matching_judge_suit_cards(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "xiuluo_q4:hand", ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (ASK_XIULUO_JUDGE, HostStage::CardsAsk) => {
            let chosen = cards.first().copied()?;
            let suit = catalog.effective_card_suit(core, owner, chosen);
            let mut opts: Vec<ChoiceOption> = core
                .player(owner)
                .judge
                .iter()
                .copied()
                .filter(|j| catalog.effective_card_suit(core, owner, *j) == suit)
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "xiuluo_q4:judge", ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        (ASK_SHENQU_TAO, HostStage::SelectCard) => {
            let mut opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.card_kind(*c) == CardKind::Tao)
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "shenqu:tao", ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (ASK_JIWU_CARD, HostStage::SelectCard) => {
            let mut opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "jiwu:card", ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (ASK_JIWU_SKILL, HostStage::CompareTargetPick) => {
            let opts: Vec<ChoiceOption> = jiwu_available_skills(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Skill)
                .collect();
            (!opts.is_empty()).then(|| ask(owner, "jiwu:skill", ChoiceKind::Control, opts, 1, 1, false))
        }
        (ASK_LINGFENG_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = (0..core.players.len() as PlayerId)
                .filter(|&p| p != owner && core.player(p).alive && core.is_hostile_to(catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "boss_lingfeng:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_LEILI_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = (0..core.players.len() as PlayerId)
                .filter(|&p| {
                    target != Some(p) && core.player(p).alive && core.is_hostile_to(catalog, owner, p)
                })
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "boss_leili:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_FENGXING_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = q4_fengxing_targets(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "boss_fengxing:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_QIANGXIX_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = q4_boss_jiwu::qiangxix_targets(core, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "qiangxix:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_QIANGXIX_CARD, HostStage::SelectCard) => {
            let mut opts: Vec<ChoiceOption> = q4_boss_jiwu::qiangxix_weapons(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            let hp = core.player(owner).hp;
            if hp < 1 {
                (!opts.is_empty()).then(|| ask(owner, "qiangxix:card", ChoiceKind::SelectCards, opts, 1, 1, true))
            } else {
                (!opts.is_empty()).then(|| ask(owner, "qiangxix:card", ChoiceKind::SelectCards, opts, 0, 1, false))
            }
        }
        (ASK_RETIEJI_CONFIRM, HostStage::CardsAsk) => Some(ask(
            owner,
            "retieji:confirm",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm],
            1,
            1,
            true,
        )),
        (ASK_RETIEJI_GIVE | ASK_RETIEJI_DISCARD, HostStage::SelectCard) => {
            let tgt = target?;
            let suit = cards.get(1).copied().and_then(q4_boss_jiwu::code_to_suit)?;
            let mut opts: Vec<ChoiceOption> = q4_boss_jiwu::retieji_matching_cards(core, catalog, tgt, suit)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            let prompt = if ask_id == ASK_RETIEJI_DISCARD {
                "retieji:discard"
            } else {
                "retieji:give"
            };
            (!opts.is_empty()).then(|| ask(tgt, prompt, ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (ASK_REXUANFENG_MODE, HostStage::SelectNature) => Some(ask(
            owner,
            "rexuanfeng:mode",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            true,
        )),
        (ASK_REXUANFENG_MOVE_FROM, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = q4_boss_jiwu::rexuanfeng_move_from_candidates(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "rexuanfeng:move_from", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_REXUANFENG_MOVE_CARD, HostStage::SelectCard) => {
            let from = target?;
            let mut opts: Vec<ChoiceOption> = q4_boss_jiwu::rexuanfeng_movable_cards(core, catalog, owner, from)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "rexuanfeng:move_card", ChoiceKind::SelectCards, opts, 1, 1, true))
        }
        (ASK_REXUANFENG_MOVE_TO, HostStage::CompareTargetPick) => {
            let from = target?;
            let eq = cards.first().copied()?;
            let mut opts: Vec<ChoiceOption> = q4_boss_jiwu::rexuanfeng_move_to_candidates(core, catalog, owner, from, eq)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "rexuanfeng:move_to", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_XUANFENG_TARGET, HostStage::TargetChoice) => {
            let mut opts: Vec<ChoiceOption> = (0..core.players.len() as PlayerId)
                .filter(|&p| {
                    p != owner
                        && core.player(p).alive
                        && (!core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
                })
                .map(ChoiceOption::Player)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "xuanfeng:target", ChoiceKind::SelectPlayers, opts, 1, 1, true))
        }
        (ASK_XUANFENG_CARD, HostStage::SelectCard) => {
            let tgt = target?;
            let mut opts: Vec<ChoiceOption> = core
                .player(tgt)
                .hand
                .iter()
                .chain(core.player(tgt).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            opts.truncate(64);
            (!opts.is_empty()).then(|| ask(owner, "xuanfeng:card", ChoiceKind::SelectCards, opts, 1, 1, false))
        }
        _ => None,
    }
}
