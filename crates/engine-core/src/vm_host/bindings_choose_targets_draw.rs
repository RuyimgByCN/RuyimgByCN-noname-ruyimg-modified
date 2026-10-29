//! VM 效果请求的宿主绑定（bind_effect）：通用效果原语。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            stage,
        },

        EffectRequest::DrawRememberedHandCards { player, count, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::DrawRememberedHandCards {
                player,
                count,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DiscardRememberedHandCards { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::DiscardRememberedHandCards {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DiscardTargetEquipmentDrawWeaponRange { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseTargetRevealHandDamageIfSha { player, amount, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseTargetRevealHandDamageIfSha {
                player,
                amount,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherGiveOneHandEquip { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseOtherGiveOneHandEquip {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }
        EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
        },

        EffectRequest::ChooseRevealHandCardBindCategoryTarget { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseRevealHandCardBindCategoryTarget {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        },

        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseTargetDrawToMaxHpCap {
                player,
                cap,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::DrawByPlayPhaseCardCategoryCount {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                player,
                sha_user,
                card,
                restore_skill_def,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::QueueOpeningExtraPlayPhase { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::QueueOpeningExtraPlayPhase {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { player, card, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
            player,
            card,
            draw_count,
            damage_amount,
            mark_def,
            mark_count,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                player,
                card,
                draw_count,
                damage_amount,
                mark_def,
                mark_count,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player,
            draw_count,
            payment_count,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DrawThenChooseGiveOrDiscardHand {
            player,
            draw_count,
            payment_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
        },

        EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target,
            count,
            ready_to_check,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_check,
        },

        EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            ready_to_discard,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_discard,
        },

        EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
            player,
            target,
            card,
            suit,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                card,
                suit,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
                ready_to_choose,
            }
        }

        EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::TargetChooseSuitGainOwnerHandDamageIfMismatch {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            ready_to_recover,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_recover,
        },

        EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards,
            targets,
            stage,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
            player,
            card,
            target,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                player,
                card,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseLoseHpUseVirtualShaByBasicVariants {
            player,
            card,
            targets,
            stage,
            damage_bonus,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
            player,
            reveal_count,
            recipient_cap,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                player,
                reveal_count,
                recipient_cap,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGainTargetHandCeilHalfDifference {
            player,
            target,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_choose,
        },

        EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
            player,
            target,
            card,
            targets,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
            player,
            target,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                player,
                target,
                damage_id: host.state.resolution.current_damage_id,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
            player,
            source,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player,
                source,
                card: engine_api::CardId::NONE,
                stage: 0,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        req => req,
    })
}
