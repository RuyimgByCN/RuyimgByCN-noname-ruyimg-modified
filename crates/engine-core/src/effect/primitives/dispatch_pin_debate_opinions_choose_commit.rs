//! 效果原语分派（apply_effect）：choose、commit 族效果。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::ChooseGroup { player, skill } => choose_group(tx, player, skill),

        EffectRequest::ChooseGainOtherSkill { player, skill } => {
            choose_gain_other_skill(tx, player, skill)
        }

        EffectRequest::ChoosePlaceHandXumou { player, skill } => {
            choose_place_hand_xumou(tx, player, skill)
        }

        EffectRequest::ChooseOthersInsertExtraTurn {
            player,
            check,
            skill,
        } => choose_others_insert_extra_turn(tx, player, check, skill),

        EffectRequest::ChooseOthersAddMark {
            player,
            mark,
            delta,
            max_count,
            skill,
        } => choose_others_add_mark(tx, player, mark, delta, max_count, skill),

        EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, skill } => {
            choose_end_extra_draw_or_play_by_used_suits(tx, player, skill)
        }

        EffectRequest::ChooseAddCurrentShaTarget {
            player,
            card,
            skill,
        } => choose_add_current_sha_target(tx, player, card, skill),

        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            skill,
        } => choose_add_current_sha_target_within_distance(tx, player, card, max_distance, skill),

        EffectRequest::CommitAddCurrentShaTarget {
            player,
            card,
            target,
            skill: _,
        } => {
            let targets = crate::command::play::add_saved_sha_target(tx, player, card, target)?;
            Ok(vec![DomainEvent::CardTargetAdded {
                player,
                card,
                target,
                targets,
            }])
        }

        EffectRequest::ChooseStoreHandShaInSkillPile { player, skill } => {
            choose_store_hand_sha_in_skill_pile(tx, player, skill)
        }

        EffectRequest::ChooseSpendSkillPileForDyingJiu {
            player,
            dying,
            skill,
        } => choose_spend_skill_pile_for_dying_jiu(tx, player, dying, skill),

        EffectRequest::CommitStoreHandShaInSkillPile {
            player,
            cards,
            skill,
        } => commit_store_hand_sha_in_skill_pile(tx, player, cards, skill),

        EffectRequest::CommitSpendSkillPileForDyingJiu {
            player,
            dying,
            card,
            skill,
        } => commit_spend_skill_pile_for_dying_jiu(tx, player, dying, card, skill),

        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => choose_compare_selected_hand_card(
            tx,
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        ),

        EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => choose_compare_selected_hand_card_second_failure_seat_damage(
            tx,
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        ),

        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            skill,
        } => choose_hand_compare(tx, player, win_skill_def, lose_skill_def, skill),

        EffectRequest::ChooseHandCompareGrantTargetOnWin {
            player,
            win_grant_def,
            lose_skill_def,
            skill,
        } => choose_hand_compare_grant_target_on_win(
            tx,
            player,
            win_grant_def,
            lose_skill_def,
            skill,
        ),

        EffectRequest::ChooseHandCompareBetween {
            player,
            first,
            skill,
        } => choose_hand_compare_between(tx, player, first, skill),

        EffectRequest::ChooseOtherGiveThenRecover { player, skill } => {
            choose_other_give_then_recover(tx, player, skill)
        }

        EffectRequest::ChooseOtherDamage {
            player,
            amount,
            skill,
        } => choose_other_damage(tx, player, amount, skill),

        EffectRequest::ChooseBoundSourceAttackRangeDamage {
            player,
            source,
            amount,
            skill,
        } => choose_bound_source_attack_range_damage(tx, player, source, amount, skill),

        EffectRequest::ChooseOtherDiscardTargetCard {
            player,
            scope,
            skill,
        } => choose_other_discard_target_card(tx, player, scope, skill),

        EffectRequest::ChooseDrawOrDiscardTargetCard {
            player,
            target,
            draw_count,
            skill,
        } => choose_draw_or_discard_target_card(tx, player, target, draw_count, skill),

        EffectRequest::ChooseSeatPathDamage {
            player,
            first,
            second,
            amount,
            skill,
        } => choose_seat_path_damage(tx, player, first, second, amount, skill),

        EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
            player,
            card,
            skill,
        } => choose_add_or_remove_current_card_target(tx, player, card, skill),

        EffectRequest::ChooseAddSkillBoundCardTarget {
            player,
            card,
            skill,
        } => choose_add_skill_bound_card_target(tx, player, card, skill),

        EffectRequest::CommitAddSkillBoundCardTarget {
            player,
            card,
            target,
            skill: _,
        } => {
            let targets = crate::command::play::add_saved_card_target(tx, player, card, target)?;
            Ok(vec![DomainEvent::CardTargetAdded {
                player,
                card,
                target,
                targets,
            }])
        }

        EffectRequest::ChooseDiscardMatchingEquipColorAddSkill {
            player,
            target,
            skill_def,
            skill,
        } => choose_discard_matching_equip_color_add_skill(tx, player, target, skill_def, skill),

        EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            skill,
        } => choose_current_card_use_enhancements(tx, player, card, helper_skill_def, skill),

        EffectRequest::ChooseDrawOrDiscardAddSkill {
            player,
            draw_skill_def,
            discard_skill_def,
            skill,
            three_way,
        } => choose_draw_or_discard_add_skill(
            tx,
            player,
            draw_skill_def,
            discard_skill_def,
            skill,
            three_way,
        ),

        EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
            player,
            cards,
            skill,
        } => choose_discarded_black_basic_as_bingliang(tx, player, cards, skill),

        EffectRequest::ChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => choose_discard_judge_use_virtual_sha_draw_if_no_damage(tx, player, target, skill),

        EffectRequest::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
            player,
            target,
            toggle_link,
            skill,
        } => choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
            tx,
            player,
            target,
            toggle_link,
            skill,
        ),

        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, skill } => {
            choose_reveal_other_hand_card_remember_category(tx, player, skill)
        }

        EffectRequest::ChooseGainResolvedShaOrShanThenOptionalUseSha {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => choose_gain_resolved_sha_or_shan_then_optional_use_sha(
            tx,
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
            false,
        ),

        EffectRequest::ChooseGainAllResolvedShaShanThenColorRewards {
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
        } => choose_gain_resolved_sha_or_shan_then_optional_use_sha(
            tx,
            player,
            sha_card,
            shan_cards,
            excluded_recipients,
            skill,
            true,
        ),

        EffectRequest::ChooseUseShaAfterResolvedGift {
            owner,
            player,
            skill,
        } => choose_use_sha_after_resolved_gift(tx, owner, player, skill),

        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player,
            amount,
            skill,
        } => choose_other_targets_by_skipped_phase_count_damage(tx, player, amount, skill, false),

        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player,
            amount,
            skill,
        } => choose_other_targets_by_skipped_phase_count_damage(tx, player, amount, skill, true),

        EffectRequest::ChooseCardUserOrAddTarget {
            player,
            card,
            skill,
        } => choose_card_user_or_add_target(tx, player, card, skill),

        EffectRequest::CommitTransferSavedCardUser {
            previous,
            new_user,
            card,
            skill: _,
        } => {
            crate::command::play::transfer_saved_card_user(tx, previous, new_user, card)?;
            Ok(vec![DomainEvent::CardUserChanged {
                previous,
                player: new_user,
                card,
            }])
        }

        EffectRequest::ChooseDiscardEquipmentReduceCurrentDamage {
            player,
            damage_id,
            skill,
        } => choose_discard_equipment_current_damage(tx, player, damage_id, skill, false),

        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
            player,
            damage_id,
            skill,
        } => choose_discard_equipment_current_damage(tx, player, damage_id, skill, true),

        EffectRequest::ChooseDiscardDamageEndpointCardIncreaseIfSuit {
            player,
            target,
            damage_id,
            suit,
            amount,
            skill,
        } => choose_discard_damage_endpoint_card_increase_if_suit(
            tx, player, target, damage_id, suit, amount, skill,
        ),

        EffectRequest::ChooseMoveTargetEquipmentOrDraw {
            player,
            target,
            skill,
        } => choose_move_target_equipment_or_draw(tx, player, target, skill),

        EffectRequest::ChooseMoveTargetEquipmentAndOrDraw {
            player,
            target,
            skill,
        } => choose_move_target_equipment_and_or_draw(tx, player, target, skill),

        EffectRequest::ChooseDrawViewTwoGainOrMoveEquipment {
            player,
            target,
            skill,
        } => choose_draw_view_two_gain_or_move_equipment(tx, player, target, skill),

        EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment {
            owner,
            recipient,
            count,
            skill,
        } => choose_recover_owner_or_damage_by_given_equipment(tx, owner, recipient, count, skill),

        EffectRequest::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => choose_target_discard_weapon_range_hand_or_increase_damage(
            tx, player, target, damage_id, skill,
        ),

        EffectRequest::ChooseTargetDiscardDistanceHandOrIncreaseDamage {
            player,
            target,
            damage_id,
            skill,
        } => choose_target_discard_distance_hand_or_increase_damage(
            tx, player, target, damage_id, skill,
        ),

        EffectRequest::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
            player,
            target,
            damage_id,
            skill,
        } => choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
            tx, player, target, damage_id, skill,
        ),

        EffectRequest::ChooseDrawOrRecastAddSkill {
            player,
            draw_skill_def,
            recast_skill_def,
            skill,
        } => choose_draw_or_recast_add_skill(tx, player, draw_skill_def, recast_skill_def, skill),

        EffectRequest::ChooseGainShaForOpeningExtraPlay { player, skill } => {
            choose_gain_sha_for_opening_extra_play(tx, player, skill)
        }

        EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, skill } => {
            choose_hand_count_different_target_draw_or_random_discard(tx, player, skill)
        }

        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            skill,
        } => choose_other_draw_or_recover(tx, player, draw_count, recover_count, times, skill),

        EffectRequest::ChooseOtherRecover { player, amount } => {
            choose_other_recover(tx, player, amount)
        }

        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count,
            skill,
        } => choose_discard_hand_equip_then_remove_self_skill(tx, player, count, skill),

        EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            skill,
        } => choose_discard_hand_equip_then_recover(tx, player, count, amount, skill),

        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, skill } => {
            choose_target_discard_excess_then_self_discard_or_target_draw(tx, player, skill)
        }

        EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard {
            player,
            target,
            count,
            skill,
        } => choose_self_discard_or_target_draw_after_target_discard(
            tx, player, target, count, skill,
        ),

        EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
            player,
            damage_id,
            amount,
            skill,
        } => choose_prevent_damage_or_lose_max_hp_increase_damage(
            tx, player, damage_id, amount, skill,
        ),

        EffectRequest::ChooseDiscardHandAddSkillByColor {
            player,
            red_skill_def,
            black_skill_def,
        } => choose_discard_hand_add_skill_by_color(tx, player, red_skill_def, black_skill_def),

        EffectRequest::CommitDamage {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => {
            let (mut events, followups) = commit_damage(
                tx,
                damage_id,
                source,
                target,
                amount,
                base_amount,
                is_lose_hp,
                nature,
                card,
                card_kind,
                card_flags,
            )?;
            for followup in followups {
                events.extend(apply_effect(tx, followup)?);
            }
            Ok(events)
        }

        EffectRequest::CommitDamageBanActiveIfDying {
            damage_id,
            source,
            target,
            amount,
            base_amount,
            nature,
            skill,
        } => {
            let (mut events, followups) = commit_damage_ban_active_if_dying(
                tx,
                damage_id,
                source,
                target,
                amount,
                base_amount,
                nature,
                skill,
            )?;
            for followup in followups {
                events.extend(apply_effect(tx, followup)?);
            }
            Ok(events)
        }

        EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            let (events, followups) = commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
                tx, player, target, amount, skill,
            )?;
            debug_assert!(followups.is_empty());
            Ok(events)
        }

        EffectRequest::CommitRecovery {
            recovery_id,
            source,
            player,
            amount,
        } => {
            if recovery_id != 0 {
                if tx.state().resolution.current_recovery_id == recovery_id {
                    tx.state_mut().resolution.current_recovery_id = 0;
                }
                if let Some(index) = tx
                    .state()
                    .resolution
                    .prevented_recovery_ids
                    .iter()
                    .position(|id| *id == recovery_id)
                {
                    tx.state_mut()
                        .resolution
                        .prevented_recovery_ids
                        .remove(index);
                    return Ok(Vec::new());
                }
            }
            recover_hp(tx, source, player, amount)
        }

        EffectRequest::ChooseDiscardOneHandOrEquip { player } => {
            choose_discard_one_hand_or_equip(tx, player)
        }

        EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
            player,
            target,
            skill,
        } => choose_discard_basic_then_target_discard_equip_or_damage(tx, player, target, skill),

        EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
            player,
            target,
            skill,
            stage,
            discarded_basic,
        } => choose_target_lose_hp_discard_hand_sha_if_basic(
            tx,
            player,
            target,
            skill,
            stage,
            discarded_basic,
        ),
        _ => unreachable!("effect routed to a different primitive group"),
    }
}
