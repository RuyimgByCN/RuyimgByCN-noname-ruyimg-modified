//! PA12 批⑰：60 种效果与结算器（tags 209..=254 + ext_tag 256..=269）的入队参数校验。

use super::super::*;

pub(super) fn accept_batch17_effect(
    req: &EffectRequest,
    seats: u8,
) -> Option<Result<(), HostTrap>> {
    let ok_player = |p: rule_api::PlayerId, what: &str| -> Result<PlayerId, HostTrap> {
        seat_of(p, seats).ok_or_else(|| reject(what))
    };
    let opt_player = |p: rule_api::PlayerId, what: &str| -> Result<(), HostTrap> {
        if p.is_some() {
            ok_player(p, what)?;
        }
        Ok(())
    };
    let ok_players = |ps: &[rule_api::PlayerId], what: &str| -> Result<(), HostTrap> {
        if ps.len() > u8::MAX as usize {
            return Err(reject(what));
        }
        for &p in ps {
            ok_player(p, what)?;
        }
        Ok(())
    };
    let ok_cards = |cs: &[rule_api::CardId], what: &str| -> Result<(), HostTrap> {
        if cs.len() > u8::MAX as usize || cs.iter().any(|c| !c.is_some()) {
            return Err(reject(what));
        }
        Ok(())
    };
    let res = (|| -> Result<(), HostTrap> {
        match req {
            EffectRequest::ChooseCardUserOrAddTarget { player, .. }
            | EffectRequest::ChooseCurrentCardUseEnhancements { player, .. }
            | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, .. }
            | EffectRequest::ChooseDiscardWeaponOrLoseHp { player, .. }
            | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                player, ..
            }
            | EffectRequest::StartDebateAllOthers { player, .. }
            | EffectRequest::StartDebateTargets { player, .. }
            | EffectRequest::ResolveJsrgchaozheng { player, .. }
            | EffectRequest::ResolveRebingyi { player, .. }
            | EffectRequest::ChooseAddSkillBoundCardTarget { player, .. }
            | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, .. }
            | EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, .. }
            | EffectRequest::ChooseTargetHandCardToPile { player, .. }
            | EffectRequest::CityConsume { player, .. }
            | EffectRequest::DiscardOneOwnXumou { player, .. }
            | EffectRequest::DiscardRememberedHandCards { player, .. }
            | EffectRequest::ExtraResolveCurrentTrick { player, .. }
            | EffectRequest::MoveEquipmentThenDraw { player, .. } => {
                ok_player(*player, "Batch17.player")?;
            }
            EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { player, target, .. }
            | EffectRequest::ChooseOptionalDiscardTargetCard { player, target, .. }
            | EffectRequest::ChooseMoveTargetEquipmentOrDraw { player, target, .. }
            | EffectRequest::ChooseOtherHandStoreSkillPile { player, target, .. }
            | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { player, target, .. }
            | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { player, target, .. }
            | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { player, target, .. }
            | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                player,
                target,
                ..
            }
            | EffectRequest::ChooseRevealHandCardBindCategoryTarget { player, target, .. }
            | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { player, target, .. } => {
                ok_player(*player, "Batch17.player")?;
                opt_player(*target, "Batch17.target")?;
            }
            EffectRequest::ChooseUseVirtualCard { player, kind, .. } => {
                ok_player(*player, "ChooseUseVirtualCard.player")?;
                if *kind == 0 {
                    return Err(reject("ChooseUseVirtualCard.kind"));
                }
            }
            EffectRequest::ChooseDiscardHandEquipThenRecover {
                player,
                count,
                amount,
                ..
            } => {
                ok_player(*player, "ChooseDiscardHandEquipThenRecover.player")?;
                if *count == 0 || *amount == 0 {
                    return Err(reject("ChooseDiscardHandEquipThenRecover.counts"));
                }
            }
            EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
                chooser,
                target,
                count,
                amount,
                ..
            } => {
                ok_player(*chooser, "ChooseDiscardHandEquipThenRecoverTarget.chooser")?;
                opt_player(*target, "ChooseDiscardHandEquipThenRecoverTarget.target")?;
                if *count == 0 || *amount == 0 {
                    return Err(reject("ChooseDiscardHandEquipThenRecoverTarget.counts"));
                }
            }
            EffectRequest::ChooseOtherYinghun {
                player,
                target,
                branch,
                ..
            } => {
                ok_player(*player, "ChooseOtherYinghun.player")?;
                opt_player(*target, "ChooseOtherYinghun.target")?;
                if *branch > 2 {
                    return Err(reject("ChooseOtherYinghun.branch"));
                }
            }
            EffectRequest::ResolveDiliYuheng {
                player,
                action,
                event_cards,
                ..
            } => {
                ok_player(*player, "ResolveDiliYuheng.player")?;
                ok_cards(event_cards, "ResolveDiliYuheng.event_cards")?;
                if *action > 4 {
                    return Err(reject("ResolveDiliYuheng.action"));
                }
            }
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { player, amount, .. } => {
                ok_player(
                    *player,
                    "ChooseOtherTargetsBySkippedPhaseCountDamage.player",
                )?;
                if *amount == 0 {
                    return Err(reject(
                        "ChooseOtherTargetsBySkippedPhaseCountDamage.amount",
                    ));
                }
            }
            EffectRequest::LoseHpDoubleCurrentDamageThenReplace { player, .. } => {
                ok_player(*player, "LoseHpDoubleCurrentDamageThenReplace.player")?;
            }
            EffectRequest::DrawThenOptionallyDistributeHandCards {
                player,
                draw_count,
                give_cap,
                cards,
                targets,
                ..
            } => {
                ok_player(*player, "DrawThenOptionallyDistributeHandCards.player")?;
                ok_cards(cards, "DrawThenOptionallyDistributeHandCards.cards")?;
                ok_players(targets, "DrawThenOptionallyDistributeHandCards.targets")?;
                if *draw_count == 0 || *give_cap == 0 {
                    return Err(reject("DrawThenOptionallyDistributeHandCards.counts"));
                }
            }
            EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                player,
                count,
                stage,
                cards,
                ..
            } => {
                ok_player(*player, "ReplaceDrawWithDistinctSuitSelection.player")?;
                ok_cards(cards, "ReplaceDrawWithDistinctSuitSelection.cards")?;
                if *count == 0 || *stage > 2 {
                    return Err(reject("ReplaceDrawWithDistinctSuitSelection.args"));
                }
            }
            EffectRequest::ResolveBailingyun {
                player,
                target,
                action,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveBailingyun.player")?;
                opt_player(*target, "ResolveBailingyun.target")?;
                ok_cards(cards, "ResolveBailingyun.cards")?;
                if !matches!(*action, 0..=4 | 10 | 11) {
                    return Err(reject("ResolveBailingyun.action"));
                }
            }
            EffectRequest::ResolveJieliangGain {
                player,
                mover,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveJieliangGain.player")?;
                opt_player(*mover, "ResolveJieliangGain.mover")?;
                ok_cards(cards, "ResolveJieliangGain.cards")?;
            }
            EffectRequest::ResolveStarcuxiaWeiming {
                player,
                source,
                variant,
            } => {
                ok_player(*player, "ResolveStarcuxiaWeiming.player")?;
                opt_player(*source, "ResolveStarcuxiaWeiming.source")?;
                if *variant > 1 {
                    return Err(reject("ResolveStarcuxiaWeiming.variant"));
                }
            }
            EffectRequest::ResolveReanxu {
                player,
                stage,
                first,
                second,
                ..
            } => {
                ok_player(*player, "ResolveReanxu.player")?;
                opt_player(*first, "ResolveReanxu.first")?;
                opt_player(*second, "ResolveReanxu.second")?;
                if *stage > 3 {
                    return Err(reject("ResolveReanxu.stage"));
                }
            }
            EffectRequest::ResolveRejizhi { player, stage, .. } => {
                ok_player(*player, "ResolveRejizhi.player")?;
                if *stage > 2 {
                    return Err(reject("ResolveRejizhi.stage"));
                }
            }
            EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                player, remaining, ..
            } => {
                ok_player(*player, "ForceOthersUseShaOnNearestOrLoseHp.player")?;
                ok_players(remaining, "ForceOthersUseShaOnNearestOrLoseHp.remaining")?;
            }
            EffectRequest::ResolveHoufeng {
                player,
                target,
                mode,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveHoufeng.player")?;
                opt_player(*target, "ResolveHoufeng.target")?;
                if *mode > 1 || *stage > 1 {
                    return Err(reject("ResolveHoufeng.args"));
                }
            }
            EffectRequest::ResolveHuzun {
                player,
                source,
                action,
                cards,
                targets,
                ..
            } => {
                ok_player(*player, "ResolveHuzun.player")?;
                opt_player(*source, "ResolveHuzun.source")?;
                ok_cards(cards, "ResolveHuzun.cards")?;
                ok_players(targets, "ResolveHuzun.targets")?;
                if *action > 2 {
                    return Err(reject("ResolveHuzun.action"));
                }
            }
            EffectRequest::ResolveMalingli {
                player,
                action,
                source,
                target,
                event_cards,
                event_targets,
                ..
            } => {
                ok_player(*player, "ResolveMalingli.player")?;
                opt_player(*source, "ResolveMalingli.source")?;
                opt_player(*target, "ResolveMalingli.target")?;
                ok_cards(event_cards, "ResolveMalingli.event_cards")?;
                ok_players(event_targets, "ResolveMalingli.event_targets")?;
                if !matches!(*action, 0..=2 | 10..=14 | 20) {
                    return Err(reject("ResolveMalingli.action"));
                }
            }
            EffectRequest::ResolveZhugemengxue {
                player,
                target,
                action,
                cards,
                ..
            } => {
                ok_player(*player, "ResolveZhugemengxue.player")?;
                opt_player(*target, "ResolveZhugemengxue.target")?;
                ok_cards(cards, "ResolveZhugemengxue.cards")?;
                if !matches!(*action, 0..=4 | 10..=12) {
                    return Err(reject("ResolveZhugemengxue.action"));
                }
            }
            EffectRequest::ResolveTwyuyan { player, source, .. } => {
                ok_player(*player, "ResolveTwyuyan.player")?;
                opt_player(*source, "ResolveTwyuyan.source")?;
            }
            EffectRequest::ResolveTwzhibian {
                player,
                target,
                winner,
                stage,
                ..
            } => {
                ok_player(*player, "ResolveTwzhibian.player")?;
                opt_player(*target, "ResolveTwzhibian.target")?;
                opt_player(*winner, "ResolveTwzhibian.winner")?;
                if *stage > 3 {
                    return Err(reject("ResolveTwzhibian.stage"));
                }
            }
            EffectRequest::ChooseDiscardTargetCardResolveCurrentSha { source, target, .. }
            | EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                source,
                target,
                ..
            } => {
                ok_player(*source, "ChooseDiscardTargetCardResolveCurrentSha.source")?;
                opt_player(*target, "ChooseDiscardTargetCardResolveCurrentSha.target")?;
            }
            EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
                player,
                target,
                count,
                amount,
                ..
            } => {
                ok_player(*player, "ChooseDiscardTargetCardsRecoverSameCategory.player")?;
                opt_player(*target, "ChooseDiscardTargetCardsRecoverSameCategory.target")?;
                if *count == 0 || *amount == 0 {
                    return Err(reject("ChooseDiscardTargetCardsRecoverSameCategory.counts"));
                }
            }
            EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
                player,
                remaining_targets,
                ..
            }
            | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                player,
                remaining_targets,
                ..
            }
            | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player,
                remaining_targets,
                ..
            } => {
                ok_player(*player, "ChooseGainCardFromEachOther.player")?;
                ok_players(remaining_targets, "ChooseGainCardFromEachOther.targets")?;
            }
            EffectRequest::ChooseGainTopCardsBySuit {
                player,
                count,
                suit,
            } => {
                ok_player(*player, "ChooseGainTopCardsBySuit.player")?;
                if *count == 0 || matches!(suit, rule_api::Suit::None) {
                    return Err(reject("ChooseGainTopCardsBySuit.args"));
                }
            }
            EffectRequest::DiscardEquipmentPairWithArmor {
                player, targets, ..
            } => {
                ok_player(*player, "DiscardEquipmentPairWithArmor.player")?;
                ok_players(targets, "DiscardEquipmentPairWithArmor.targets")?;
            }
            EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
                player, divisors, ..
            } => {
                ok_player(*player, "GainDrawCardsForLifetimeCardActionDivisors.player")?;
                if divisors.iter().any(|&d| d == 0) {
                    return Err(reject(
                        "GainDrawCardsForLifetimeCardActionDivisors.divisors",
                    ));
                }
            }
            EffectRequest::GainHandCompareResultCard {
                player,
                initiator,
                target,
                winner,
                cards,
                ..
            } => {
                ok_player(*player, "GainHandCompareResultCard.player")?;
                opt_player(*initiator, "GainHandCompareResultCard.initiator")?;
                opt_player(*target, "GainHandCompareResultCard.target")?;
                opt_player(*winner, "GainHandCompareResultCard.winner")?;
                ok_cards(cards, "GainHandCompareResultCard.cards")?;
            }
            EffectRequest::GainHighestRankShaFromEventCards { player, cards, .. } => {
                ok_player(*player, "GainHighestRankShaFromEventCards.player")?;
                ok_cards(cards, "GainHighestRankShaFromEventCards.cards")?;
            }
            _ => return Ok(()),
        }
        Ok(())
    })();
    match req {
        EffectRequest::ChooseCardUserOrAddTarget { .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { .. }
        | EffectRequest::ChooseOptionalDiscardTargetCard { .. }
        | EffectRequest::ChooseUseVirtualCard { .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { .. }
        | EffectRequest::ChooseDiscardWeaponOrLoseHp { .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { .. }
        | EffectRequest::ChooseOtherYinghun { .. }
        | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { .. }
        | EffectRequest::ResolveDiliYuheng { .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { .. }
        | EffectRequest::ReplaceDrawWithDistinctSuitSelection { .. }
        | EffectRequest::StartDebateAllOthers { .. }
        | EffectRequest::StartDebateTargets { .. }
        | EffectRequest::ResolveBailingyun { .. }
        | EffectRequest::ResolveJieliangGain { .. }
        | EffectRequest::ResolveJsrgchaozheng { .. }
        | EffectRequest::ResolveStarcuxiaWeiming { .. }
        | EffectRequest::ResolveReanxu { .. }
        | EffectRequest::ResolveRebingyi { .. }
        | EffectRequest::ResolveRejizhi { .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { .. }
        | EffectRequest::ResolveHoufeng { .. }
        | EffectRequest::ResolveHuzun { .. }
        | EffectRequest::ResolveMalingli { .. }
        | EffectRequest::ResolveZhugemengxue { .. }
        | EffectRequest::ResolveTwyuyan { .. }
        | EffectRequest::ResolveTwzhibian { .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentSha { .. }
        | EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus { .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { .. }
        | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { .. }
        | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { .. }
        | EffectRequest::ChooseGainTopCardsBySuit { .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { .. }
        | EffectRequest::ChooseTargetHandCardToPile { .. }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver { .. }
        | EffectRequest::CityConsume { .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { .. }
        | EffectRequest::DiscardOneOwnXumou { .. }
        | EffectRequest::DiscardRememberedHandCards { .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { .. }
        | EffectRequest::ExtraResolveCurrentTrick { .. }
        | EffectRequest::GainDrawCardsForLifetimeCardActionDivisors { .. }
        | EffectRequest::GainHandCompareResultCard { .. }
        | EffectRequest::GainHighestRankShaFromEventCards { .. }
        | EffectRequest::MoveEquipmentThenDraw { .. } => Some(res),
        _ => None,
    }
}

pub(crate) fn backfill_batch17_skill(
    req: &mut EffectRequest,
    skill_id: u32,
    event_source: u32,
    event_cards: &[u32],
) {
    if let EffectRequest::ResolveJieliangGain { mover, cards, .. } = req {
        if !mover.is_some() && event_source != 0 {
            *mover = rule_api::PlayerId::new(event_source);
        }
        if cards.is_empty() && !event_cards.is_empty() {
            *cards = event_cards
                .iter()
                .copied()
                .map(rule_api::CardId::new)
                .collect();
        }
    }
    if skill_id == 0 {
        return;
    }
    match req {
        EffectRequest::ChooseCardUserOrAddTarget { skill, .. }
        | EffectRequest::ChooseCurrentCardUseEnhancements { skill, .. }
        | EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha { skill, .. }
        | EffectRequest::ChooseUseVirtualCard { skill, .. }
        | EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { skill, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecover { skill, .. }
        | EffectRequest::ChooseDiscardHandEquipThenRecoverTarget { skill, .. }
        | EffectRequest::ChooseDiscardWeaponOrLoseHp { skill, .. }
        | EffectRequest::ChooseMoveTargetEquipmentOrDraw { skill, .. }
        | EffectRequest::ChooseOtherHandStoreSkillPile { skill, .. }
        | EffectRequest::ChooseOtherYinghun { skill, .. }
        | EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { skill, .. }
        | EffectRequest::ResolveDiliYuheng { skill, .. }
        | EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { skill, .. }
        | EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { skill, .. }
        | EffectRequest::LoseHpDoubleCurrentDamageThenReplace { skill, .. }
        | EffectRequest::DrawThenOptionallyDistributeHandCards { skill, .. }
        | EffectRequest::ReplaceDrawWithDistinctSuitSelection { skill, .. }
        | EffectRequest::StartDebateAllOthers { skill, .. }
        | EffectRequest::StartDebateTargets { skill, .. }
        | EffectRequest::ResolveBailingyun { skill, .. }
        | EffectRequest::ResolveJieliangGain { skill, .. }
        | EffectRequest::ResolveJsrgchaozheng { skill, .. }
        | EffectRequest::ResolveReanxu { skill, .. }
        | EffectRequest::ResolveRebingyi { skill, .. }
        | EffectRequest::ResolveRejizhi { skill, .. }
        | EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { skill, .. }
        | EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { skill, .. }
        | EffectRequest::ResolveHoufeng { skill, .. }
        | EffectRequest::ResolveHuzun { skill, .. }
        | EffectRequest::ResolveMalingli { skill, .. }
        | EffectRequest::ResolveZhugemengxue { skill, .. }
        | EffectRequest::ResolveTwyuyan { skill, .. }
        | EffectRequest::ResolveTwzhibian { skill, .. }
        | EffectRequest::ChooseAddSkillBoundCardTarget { skill, .. }
        | EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill { skill, .. }
        | EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory { skill, .. }
        | EffectRequest::ChooseGainCardFromEachOtherReplaceDraw { skill, .. }
        | EffectRequest::ChooseGainCardFromEachOtherThenTurnOver { skill, .. }
        | EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { skill, .. }
        | EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw { skill, .. }
        | EffectRequest::ChooseRevealHandCardBindCategoryTarget { skill, .. }
        | EffectRequest::ChooseRevealOtherHandCardRememberCategory { skill, .. }
        | EffectRequest::ChooseTargetHandCardToPile { skill, .. }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            skill, ..
        }
        | EffectRequest::CityConsume { skill, .. }
        | EffectRequest::DiscardEquipmentPairWithArmor { skill, .. }
        | EffectRequest::DiscardOneOwnXumou { skill, .. }
        | EffectRequest::DiscardRememberedHandCards { skill, .. }
        | EffectRequest::DiscardTargetEquipmentDrawWeaponRange { skill, .. }
        | EffectRequest::ExtraResolveCurrentTrick { skill, .. }
        | EffectRequest::GainHandCompareResultCard { skill, .. }
        | EffectRequest::GainHighestRankShaFromEventCards { skill, .. }
        | EffectRequest::MoveEquipmentThenDraw { skill, .. }
            if !skill.is_some() =>
        {
            *skill = rule_api::SkillId::new(skill_id);
        }
        _ => {}
    }
}
