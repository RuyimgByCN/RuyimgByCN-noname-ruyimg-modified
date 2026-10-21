//! Effect bindings for `ResolveDddJianshuo` and related requests.

use super::*;

pub(super) fn bind_effect(
    host: &EngineVmHost<'_>,
    req: EffectRequest,
) -> Result<EffectRequest, HostTrap> {
    Ok(match req {
        EffectRequest::ResolveDddJianshuo {
            player,
            target,
            action,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddJianshuo {
            player,
            target: if action == 0 {
                host.selected_targets
                    .first()
                    .copied()
                    .map(PlayerId::new)
                    .unwrap_or(target)
            } else {
                target
            },
            action,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddGuanning {
            player,
            actor,
            action,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddGuanning {
            player,
            actor,
            action,
            card: if action == 2 {
                host.selected_cards
                    .first()
                    .copied()
                    .map(CardId::new)
                    .unwrap_or(card)
            } else {
                card
            },
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddCaoshuang {
            player,
            actor,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddCaoshuang {
            player,
            actor,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXuelingyun {
            player,
            actor,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXuelingyun {
            player,
            actor,
            card,
            cards: host.event_cards.iter().copied().map(CardId::new).collect(),
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXiahouxuan { player, action, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDddXiahouxuan {
                player,
                action,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            card,
            cards: host.event_cards.iter().copied().map(CardId::new).collect(),
            amount: host.ctx.event_amount,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiangxi {
            player,
            actor,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiangxi {
            player,
            actor,
            card,
            cards: host.event_cards.iter().copied().map(CardId::new).collect(),
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseOtherDiscardTargetCard { player, scope, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseOtherDiscardTargetCard {
                player,
                scope,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseAddSkillBoundCardTarget { player, card, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseAddSkillBoundCardTarget {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
            player,
            target,
            skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
            player,
            target,
            skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            three_way,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            three_way,
        },

        EffectRequest::ChooseDiscardedBlackBasicAsBingliang { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
                player,
                cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
            player,
            target,
            toggle_link,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                player,
                target,
                toggle_link,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseRevealOtherHandCardRememberCategory {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count,
            ready_to_give,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            ready_to_give,
        },

        EffectRequest::ClearPhaseGiftTarget { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ClearPhaseGiftTarget {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
            player, sha_card, ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
                player,
                sha_card,
                shan_cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                excluded_recipients: host
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
            player, sha_card, ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
                player,
                sha_card,
                shan_cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                excluded_recipients: host
                    .event_targets
                    .iter()
                    .copied()
                    .map(engine_api::PlayerId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { player, amount, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
                player,
                amount,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::RevealUniformHandDrawSelfChooseOthers {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseCardUserOrAddTarget { player, card, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseCardUserOrAddTarget {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseMoveTargetEquipmentOrDraw { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseMoveTargetEquipmentOrDraw {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseMoveTargetEquipmentAndOrDraw { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { player, target, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
            player,
            target,
            ready_to_choose,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
                ready_to_choose,
            }
        }

        EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
            player,
            damage_id: 0,
            amount,
            ..
        } => EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
            player,
            damage_id: host.state.resolution.current_damage_id,
            amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp { player, .. }
            if host.ctx.skill_id != 0 =>
        {
            EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                player,
                remaining: Vec::new(),
                started: false,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }
        req => req,
    })
}
