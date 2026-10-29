//! VM 效果请求的宿主绑定（bind_effect）：通用效果原语。

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ChooseCurrentTrickTargetsToNullify {
            player,
            source,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseCurrentTrickTargetsToNullify {
            player,
            source,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
            player,
            target,
            ready_to_check,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
                ready_to_check,
            }
        }

        EffectRequest::ChooseGiveOneHandEquipOrRecover {
            owner,
            giver,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGiveOneHandEquipOrRecover {
            owner,
            giver,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
            owner,
            giver,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGiveTwoDifferentColorsOrRecover {
            owner,
            giver,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
            player,
            zone,
            discarded_count,
            ready_to_draw,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
                zone,
                discarded_count,
                ready_to_draw,
            }
        }

        EffectRequest::ChooseBoundSourceAttackRangeDamage {
            player,
            source,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseBoundSourceAttackRangeDamage {
            player,
            source,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseAddCurrentShaTarget { player, card, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseAddCurrentShaTarget {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::RefundCurrentShaUse { player, card, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::RefundCurrentShaUse {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::GainTopCardToSkillPile { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::GainTopCardToSkillPile {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseRepoluTargetsDraw { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseRepoluTargetsDraw {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseStoreHandShaInSkillPile { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseStoreHandShaInSkillPile {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseSpendSkillPileForDyingJiu { player, dying, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseSpendSkillPileForDyingJiu {
                player,
                dying,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::RandomizeCurrentHandCompareOpponent { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::RandomizeCurrentHandCompareOpponent {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GainHighestRankShaFromEventCards { player, cards, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::GainHighestRankShaFromEventCards {
                player,
                cards,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseAddOrRemoveCurrentCardTarget { player, card, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseHandCompareGrantTargetOnWin {
            player,
            win_grant_def,
            lose_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseHandCompareGrantTargetOnWin {
            player,
            win_grant_def,
            lose_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseHandCompareBetween { player, first, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseHandCompareBetween {
                player,
                first,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GuanxingTop { player, count, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::GuanxingTop {
                player,
                count,
                round: 0,
                flipped: Vec::new(),
                top: Vec::new(),
                bottom: Vec::new(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherGiveThenRecover { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseOtherGiveThenRecover {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GuanxingTopByFaluMarks { player, marks, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::GuanxingTopByFaluMarks {
                player,
                marks,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherDamage { player, amount, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseOtherDamage {
                player,
                amount,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::RevealUniformHandChooseDamage { player, amount, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::RevealUniformHandChooseDamage {
                player,
                amount,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainDiscardCardByKind { player, kind, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseGainDiscardCardByKind {
                player,
                kind,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseAddMissingSkill {
            player, skill_defs, ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::AddFirstMissingSkill {
            player, skill_defs, ..
        } if host.ctx.skill_id != 0 => EffectRequest::AddFirstMissingSkill {
            player,
            skill_defs,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::RecordCurrentShaAllTargets { player, card, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::RecordCurrentShaAllTargets {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count,
            scope,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count,
            scope,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherHandStoreSkillPile { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseOtherHandStoreSkillPile {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GainAllSkillPileCards { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::GainAllSkillPileCards {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GainOtherSkillPileCards {
            player,
            target,
            skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::GainOtherSkillPileCards {
            player,
            target,
            skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDiscardHandRange {
            player, min, max, ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardHandRange {
            player,
            min,
            max,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        req => req,
    })
}
