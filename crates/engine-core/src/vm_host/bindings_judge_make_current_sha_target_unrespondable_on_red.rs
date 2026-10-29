//! VM 效果请求的宿主绑定（bind_effect）：通用效果原语。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji: true,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji: true,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose: false,
            judgment_suit: 0,
        },

        EffectRequest::TakeOneHandCard { from, to } if from.is_none() => {
            EffectRequest::TakeOneHandCard {
                from: PlayerId::new(*host.event_targets.first().ok_or_else(|| {
                    HostTrap::InvalidRequest {
                        message: "event has no counterpart".into(),
                    }
                })?),
                to,
            }
        }

        EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            amount,
            replacement_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id: host.state.resolution.current_damage_id,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            replacement_skill_def,
            ready_to_replace: false,
        },

        EffectRequest::PreventDamage { damage_id: 0 } => EffectRequest::PreventDamage {
            damage_id: host.state.resolution.current_damage_id,
        },

        EffectRequest::PreventRecovery { recovery_id: 0 } => EffectRequest::PreventRecovery {
            recovery_id: host.state.resolution.current_recovery_id,
        },

        EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
            player,
            target,
            damage_id: 0,
        } => EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
            player,
            target,
            damage_id: host.state.resolution.current_damage_id,
        },

        EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
            player,
            damage_id: 0,
            ..
        } => EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
            player,
            damage_id: host.state.resolution.current_damage_id,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id: 0,
            ..
        } => EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id: host.state.resolution.current_damage_id,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
            player,
            target,
            damage_id: 0,
            suit,
            amount,
            ..
        } => EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
            player,
            target,
            damage_id: host.state.resolution.current_damage_id,
            suit,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
            player,
            target,
            damage_id: 0,
            ..
        } => EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
            player,
            target,
            damage_id: host.state.resolution.current_damage_id,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id: 0,
            ..
        } => EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id: host.state.resolution.current_damage_id,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            player,
            target,
            damage_id: 0,
            ..
        } => EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            player,
            target,
            damage_id: host.state.resolution.current_damage_id,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDrawOrRecastAddSkill {
            player,
            draw_skill_def,
            recast_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDrawOrRecastAddSkill {
            player,
            draw_skill_def,
            recast_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseGainShaForOpeningExtraPlay { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseGainShaForOpeningExtraPlay {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::FinishOpeningExtraPlayShaGain { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::FinishOpeningExtraPlayShaGain {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
            player,
            kind,
            mandatory_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
            player,
            kind,
            mandatory_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseGainOwnLostDiscardCardThisTurn {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseMoveFieldCardTo { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseMoveFieldCardTo {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            ready_to_choose,
            finish,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
            finish,
        },

        EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
            player,
            target,
            cards,
            used_targets,
            total_given,
            continue_only,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
                player,
                target,
                cards,
                used_targets,
                total_given,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
                continue_only,
            }
        }

        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            ready_to_recover,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_recover,
        },

        EffectRequest::GiveSelectedThenRecoverSelf {
            player,
            target,
            cards,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveSelectedThenRecoverSelf {
            player,
            target,
            cards,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { player, mark, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                player,
                mark,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            priority,
            remaining_targets,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player,
                priority,
                remaining_targets,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::SetNonLockedSkillSuppression {
            player, enabled, ..
        } if host.ctx.skill_id != 0 => EffectRequest::SetNonLockedSkillSuppression {
            player,
            enabled,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::DiscardSelectedDrawAllHandBonus { player, cards, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::DiscardSelectedDrawAllHandBonus {
                player,
                cards,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ready_to_discard,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ready_to_discard,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherBizheng {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherBizheng {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherXiashu {
            player,
            target,
            stage,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherXiashu {
            player,
            target,
            stage,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::StoreSelectedInSkillPile { player, cards, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::StoreSelectedInSkillPile {
                player,
                cards,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::RecordActiveTarget { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::RecordActiveTarget {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardWeaponOrLoseHp { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseDiscardWeaponOrLoseHp {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardOneHeOrLoseHp {
            player,
            lose_amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardOneHeOrLoseHp {
            player,
            lose_amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice { source, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::JudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseSourceConfirmOwnerLoseHp {
            owner,
            source,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseSourceConfirmOwnerLoseHp {
            owner,
            source,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            stage,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            stage,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
            player,
            first_skill_def,
            second_skill_def,
            stage,
            cards,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player,
                first_skill_def,
                second_skill_def,
                stage,
                cards,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player,
            target,
            selected,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
                player,
                target,
                selected,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseAllLoseHpOrRecover { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseAllLoseHpOrRecover {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseLinkOther { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseLinkOther {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawCards { player, count } if host.ctx.skill_id != 0 => {
            EffectRequest::DrawCardsFromSkill {
                player,
                count,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            drawn_only,
            use_given_equipment,
            drawn,
            ready_to_give,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            drawn_only,
            use_given_equipment,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            drawn,
            ready_to_give,
        },

        EffectRequest::DrawThenTakeOne {
            player,
            taker,
            count,
            ready_to_take,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DrawThenTakeOne {
            player,
            taker,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_take,
        },

        EffectRequest::MoveEquipmentThenDraw { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::MoveEquipmentThenDraw {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseUseVirtualShaOrDraw { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseUseVirtualShaOrDraw {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DiscardEquipmentPairWithArmor {
            player, targets, ..
        } if host.ctx.skill_id != 0 => EffectRequest::DiscardEquipmentPairWithArmor {
            player,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherDrawByHandCount {
            player,
            normal_count,
            empty_count,
            times,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherDrawByHandCount {
            player,
            normal_count,
            empty_count,
            times,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill { player, count, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
                player,
                count,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player,
            target,
            count,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player,
            target,
            count,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
                player,
                target,
                card,
                draw_count,
                failure_count,
                lose_hp,
                damage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawSelectedTargets { targets, count, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::DrawSelectedTargets {
                targets,
                count,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::RecordSelectedPlayPhaseCategory { card, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::RecordSelectedPlayPhaseCategory {
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },
        req => req,
    })
}
