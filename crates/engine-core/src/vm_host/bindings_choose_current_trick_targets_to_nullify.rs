//! Effect bindings for `ChooseCurrentTrickTargetsToNullify` and related requests.

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

        EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbzhuguo {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbanxian {
            player,
            cards,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card,
            helper_skill_def,
            stage,
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

        EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDingpin { player, target, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDingpin {
                player,
                target,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

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

        EffectRequest::ResolveZhuiyi { player, killer, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveZhuiyi {
                player,
                killer,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDecadezhenjun {
            player,
            target,
            stage,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveSpXiaoguo {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

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

        EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXinfuGuolun {
            player,
            target,
            player_card,
            target_card,
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

        EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDccunsi {
            player,
            target,
            guixiu_def,
            yongjue_def,
            stage,
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

        EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcjue {
            player,
            target,
            stage,
            remaining,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveRuanji {
            player,
            source,
            card,
            action,
            cards: if host.event_cards.is_empty() {
                card.is_some().then_some(card).into_iter().collect()
            } else {
                host.event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            },
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhengbi {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveFengying {
            player,
            helper_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            split: 0,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXicha {
            player,
            source,
            card,
            action,
            cards: if host.event_cards.is_empty() {
                card.is_some().then_some(card).into_iter().collect()
            } else {
                host.event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            },
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveHuzun {
            player,
            source,
            card,
            action,
            cards: if host.event_cards.is_empty() {
                card.is_some().then_some(card).into_iter().collect()
            } else {
                host.event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect()
            },
            targets: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveChenglve { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveChenglve {
                player,
                stage,
                suits: 0,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveShicai {
            player,
            card,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveShicai {
            player,
            card,
            placed: Vec::new(),
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveXiongluan {
            player,
            target,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveGzbuqu {
            player,
            delta,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSbbiyue { player, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveSbbiyue {
                player,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveSxrmqiewang {
            player,
            event_player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveSxrmqianliu {
            player,
            source,
            stage,
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

        EffectRequest::ResolveGzbuquRecover { player, stage, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveGzbuquRecover {
                player,
                stage,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveTwjuxiang {
            player,
            event_player,
            stage,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhugemengxue {
            player,
            target,
            card,
            action,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBailingyun {
            player,
            target,
            card,
            action,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcshaowei { player, action, .. } if host.ctx.skill_id != 0 => {
            EffectRequest::ResolveDcshaowei {
                player,
                action,
                cards: host
                    .event_cards
                    .iter()
                    .copied()
                    .map(engine_api::CardId::new)
                    .collect(),
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcdichou {
            player,
            target,
            card,
            action,
            helper_skill_def,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcdufeng {
            player,
            action,
            helper_skill_def,
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

        EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcctjiuxian {
            player,
            event_player,
            target,
            card,
            action,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveBianyue {
            player,
            target,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhupeilan {
            player,
            target,
            card,
            helper_skill_def,
            action,
            event_cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcSbZhangxiu {
            player,
            target,
            action,
            cards: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcSbZhouyu {
            player,
            source,
            card,
            action,
            event_targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            damaged_targets: host
                .event_targets
                .iter()
                .enumerate()
                .filter(|(index, _)| {
                    1u32.checked_shl(*index as u32)
                        .is_some_and(|bit| u32::from(host.event_nature) & bit != 0)
                })
                .map(|(_, target)| PlayerId::new(*target))
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDcSbLusu {
            player,
            source,
            card,
            action,
            cards: host.event_cards.iter().copied().map(CardId::new).collect(),
            amount: host.ctx.event_amount,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhangjian {
            player,
            card,
            grant_skill_def,
            action,
            target,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card,
            action,
            remaining,
            used_mask,
            repeated,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveMalingli {
            player,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveMalingli {
            player,
            card,
            helper_skill_def,
            action,
            source: PlayerId::new(host.ctx.event_source),
            target: PlayerId::new(host.ctx.event_player),
            nature: host.event_nature,
            damage_id: 0,
            event_cards: host.event_cards.iter().copied().map(CardId::new).collect(),
            event_targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveNsshuaiyan {
            player,
            recovered_player,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbHanlong {
            player,
            target,
            card,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbHanlong {
            player,
            target,
            card,
            helper_skill_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbLiufu {
            player,
            target,
            card,
            zhucheng_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiufu {
            player,
            target,
            card,
            zhucheng_skill_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbLiuyan {
            player,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiuyan {
            player,
            card: host
                .selected_cards
                .first()
                .copied()
                .map(engine_api::CardId::new)
                .unwrap_or(card),
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbLiuzan {
            player,
            target,
            helper_skill_def,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbLiuzan {
            player,
            target: host
                .selected_targets
                .first()
                .copied()
                .map(engine_api::PlayerId::new)
                .unwrap_or(target),
            helper_skill_def,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbTianyu {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbTianyu {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbXizhenxihong {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveWdbYangyi {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => {
            let event_card = host
                .event_cards
                .first()
                .copied()
                .map(engine_api::CardId::new);
            let gained_target = event_card
                .and_then(|card| host.state.cards.get(card))
                .map(|card| card.owner);
            EffectRequest::ResolveWdbYangyi {
                player,
                target: if action == 1 {
                    gained_target.unwrap_or(target)
                } else {
                    target
                },
                card: if action == 1 {
                    event_card.unwrap_or(card)
                } else {
                    host.selected_cards
                        .first()
                        .copied()
                        .map(engine_api::CardId::new)
                        .unwrap_or(card)
                },
                action,
                skill: engine_api::SkillId::new(host.ctx.skill_id),
            }
        }

        EffectRequest::ResolveWdbZaozhiRenjun {
            player,
            target,
            card,
            cards,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveWdbZaozhiRenjun {
            player,
            target,
            card,
            cards,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXianxi {
            player,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXianxi {
            player,
            card,
            action,
            event_targets: host
                .event_targets
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect(),
            damaged_targets: host
                .event_targets
                .iter()
                .enumerate()
                .filter(|(index, _)| {
                    1u32.checked_shl(*index as u32)
                        .is_some_and(|bit| u32::from(host.event_nature) & bit != 0)
                })
                .map(|(_, target)| PlayerId::new(*target))
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXiaheng {
            player,
            discard_target,
            damage_target,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXiaheng {
            player,
            discard_target,
            damage_target,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiuchong {
            player,
            actor,
            card,
            action,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuchong {
            player,
            actor,
            card,
            action,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddDuanbing {
            player,
            target,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddDuanbing {
            player,
            target,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddWuzhi {
            player,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddWuzhi {
            player,
            card,
            action,
            cards: host
                .event_cards
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXujing {
            player,
            source,
            card,
            action,
            targets: host
                .event_targets
                .iter()
                .copied()
                .map(engine_api::PlayerId::new)
                .collect(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddCaomao {
            player,
            action,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddCaomao {
            player,
            action,
            cards,
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXinxianying {
            player,
            actor,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXinxianying {
            player,
            actor,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddXianglang {
            player,
            actor,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddXianglang {
            player,
            actor,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddYujin {
            player,
            actor,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddYujin {
            player,
            actor,
            target: host.event_move_target,
            move_mask: host.event_move_mask,
            action,
            candidates: Vec::new(),
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuye {
            player,
            actor,
            card,
            action,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddBaosanniang {
            player,
            target,
            card,
            action,
            cards,
            targets,
            used_kind,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddBaosanniang {
            player,
            target,
            card,
            action,
            cards,
            targets,
            used_kind,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddZhenji {
            player,
            target,
            action,
            stage,
            piles,
            cards,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage,
            card,
            kind,
            count,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddZhaoang {
            player,
            action,
            stage,
            card,
            kind,
            count,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },

        EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards,
            targets,
            ..
        } if host.ctx.skill_id != 0 => EffectRequest::ResolveDddLiuba {
            player,
            actor,
            action,
            cards: if action == 0 {
                host.selected_cards
                    .iter()
                    .copied()
                    .map(CardId::new)
                    .collect()
            } else {
                cards
            },
            targets,
            skill: engine_api::SkillId::new(host.ctx.skill_id),
        },
        req => req,
    })
}
