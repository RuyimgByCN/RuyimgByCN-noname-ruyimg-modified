//! Effect bindings for `JudgeMakeCurrentShaTargetUnrespondableOnRed` and related requests.

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

        EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDingpan {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTanlu {
            player,
            target,
            amount,
            stage,
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

        EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            stage,
            hand_cards,
            pile_cards,
            event_cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::SetNonLockedSkillSuppression {
            player, enabled, ..
        } if host.ctx.skill_id != 0 => EffectRequest::SetNonLockedSkillSuppression {
            player,
            enabled,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
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

        EffectRequest::ResolveGuanchaoRank { player, card, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveGuanchaoRank {
                player,
                card,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveGuanwei {
            player,
            target,
            stage,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveLanjiang {
            player,
            stage,
            index,
            target,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRebingyi { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRebingyi {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveRebotu { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveRebotu {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldimeng {
            player,
            stage,
            first,
            second,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRejieyin {
            player,
            target,
            stage,
            card,
            equip,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveReanxu {
            player,
            stage,
            first,
            second,
            card,
            from_hand,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldAnxu {
            player,
            stage,
            giver,
            receiver,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::StoreSelectedInSkillPile { player, cards, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::StoreSelectedInSkillPile {
                player,
                cards,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDcbingji {
            player,
            stage,
            kind,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcbingji {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            stage,
            kind,
        },

        EffectRequest::ResolveQuanjia {
            player,
            source,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQuanjia {
            player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
            source,
            target,
        },

        EffectRequest::ResolveDuwang { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDuwang {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldQinqing {
            player,
            target,
            lord,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldMingjian {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOldXuanfeng {
            player,
            target,
            card,
            stage,
            branch,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJianrou {
            player,
            damage_id: 0,
            ..
        } => EffectRequest::ResolveJianrou {
            player,
            damage_id: host.state.resolution.current_damage_id,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldHuisheng {
            player,
            source,
            damage_id: 0,
            stage,
            cards,
            ..
        } => EffectRequest::ResolveOldHuisheng {
            player,
            source: if source.is_some() {
                source
            } else {
                PlayerId::new(host.ctx.event_source)
            },
            damage_id: host.state.resolution.current_damage_id,
            stage,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveOldJiefan {
            player,
            dying,
            current,
            card,
            damage_id,
            stage,
            ..
        } => EffectRequest::ResolveOldJiefan {
            player,
            dying: if dying.is_some() {
                dying
            } else {
                PlayerId::new(host.ctx.event_player)
            },
            current: if current.is_some() {
                current
            } else {
                host.state.turn.current
            },
            card: if card.is_some() {
                card
            } else {
                engine_api::CardId::new(host.ctx.event_card)
            },
            damage_id: if damage_id != 0 {
                damage_id
            } else {
                host.state.resolution.current_damage_id
            },
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveChegu {
            player,
            card,
            targets,
            cards,
            branch,
            index,
            stage,
            ..
        } => EffectRequest::ResolveChegu {
            player,
            card: if card.is_some() {
                card
            } else {
                engine_api::CardId::new(host.ctx.event_card)
            },
            targets: if targets.is_empty() {
                host.event_targets
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect()
            } else {
                targets
            },
            cards,
            branch,
            index,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            ..
        } => EffectRequest::ResolveJunkguixin {
            player,
            target,
            stage,
            group,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            ..
        } => EffectRequest::ResolveYaoChenwei {
            player,
            target,
            card,
            mode,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltJueyan {
            player,
            stage,
            slot,
            first,
            sha_skill_def,
            weapon_skill_def,
            mount_skill_def,
            armor_skill_def,
            treasure_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRejizhi {
            player,
            stage,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveJiexun {
            player,
            stage,
            target,
            draw_count,
            discard_count,
            funan_helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWenguaSetup {
            player,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWengua {
            player,
            stage,
            target,
            card,
            branch,
            main_skill_def,
            marker_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXinanguo {
            player,
            target,
            stage,
            index,
            remaining,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveGanglieThree { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveGanglieThree {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhongyi {
            player,
            action,
            helper_skill_def,
            card,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMassDamageDiscardTurnOver {
            player,
            mark,
            mark_count,
            damage_amount,
            hand_count,
            stage,
            index,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMassDamageDiscardHandEquip {
            player,
            mark,
            mark_count,
            damage_amount,
            discard_count,
            stage,
            index,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

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

        EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage,
            index,
            target,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
            player,
            damage_amount,
            nature,
            stage,
            index,
            targets,
            ..
        } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                player,
                damage_amount,
                nature,
                stage,
                index,
                targets,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

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

        EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveYeyan {
            player,
            mode,
            stage,
            index,
            targets,
            data,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseAllLoseHpOrRecover { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseAllLoseHpOrRecover {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQixing {
            player,
            initialize,
            stage,
            hand_cards,
            pile_cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQixingWeather {
            player,
            qixing_skill_def,
            helper_skill_def,
            max_targets,
            stage,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNzryLongnu {
            player,
            fire_skill_def,
            thunder_skill_def,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ChooseLinkOther { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ChooseLinkOther {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltPoxi {
            player,
            target,
            hand_limit_skill_def,
            stage,
            own_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltJieying {
            player,
            event_player,
            mark,
            mode,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltDuorui {
            player,
            target,
            helper_skill_def,
            stage,
            selected_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDrltZhiti {
            player,
            event_player,
            event_source,
            event_amount,
            mode,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveShenSimayi {
            player,
            event_player,
            event_amount,
            mark,
            mode,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBeige {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBeigeConfigured {
            player,
            target,
            source,
            stage,
            result_card,
            suit,
            rank,
            rewriters,
            heart_recover_amount,
            diamond_draw_count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

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

        EffectRequest::ResolveOlwenren {
            player, targets, ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveOlwenren {
            player,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

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
