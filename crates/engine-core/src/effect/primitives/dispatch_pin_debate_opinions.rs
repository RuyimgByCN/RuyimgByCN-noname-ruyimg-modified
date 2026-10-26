//! Primitive dispatch for `PinDebateOpinions` and related requests.

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::PinDebateOpinions { player, skill } => {
            pin_debate_opinions(tx, player, skill)
        }

        EffectRequest::AlignDebateOpinions { player, skill } => {
            align_debate_opinions(tx, player, skill)
        }

        EffectRequest::GainShadowCards {
            player,
            count,
            skill,
        } => gain_shadow_cards(tx, player, count, skill),

        EffectRequest::ChangeGroup {
            player,
            group,
            skill,
        } => change_group(tx, player, group, skill),

        EffectRequest::DistributeShadowCards {
            player,
            count,
            skill,
        } => distribute_shadow_cards(tx, player, count, skill),

        EffectRequest::RegisterTurnEndDebate { player, skill } => {
            register_turn_end_debate(tx, player, skill)
        }

        EffectRequest::ResolveTurnEndDebates { player, skill } => {
            resolve_turn_end_debates(tx, player, skill)
        }

        EffectRequest::ChooseGroup { player, skill } => choose_group(tx, player, skill),

        EffectRequest::ChooseGainOtherSkill { player, skill } => {
            choose_gain_other_skill(tx, player, skill)
        }

        EffectRequest::LoseGainedSkill { player, skill } => lose_gained_skill(tx, player, skill),

        EffectRequest::ResolvePotheyun { player, skill } => resolve_potheyun(tx, player, skill),

        EffectRequest::PlaceXumou {
            player,
            card,
            skill,
        } => place_xumou(tx, player, card, skill),

        EffectRequest::PlaceDeckTopXumou { player, skill } => {
            place_deck_top_xumou(tx, player, skill)
        }

        EffectRequest::ChoosePlaceHandXumou { player, skill } => {
            choose_place_hand_xumou(tx, player, skill)
        }

        EffectRequest::DiscardOneOwnXumou { player, skill } => {
            discard_one_own_xumou(tx, player, skill)
        }

        EffectRequest::PlaceDeckTopExpansion {
            player,
            count,
            skill,
        } => place_deck_top_expansion(tx, player, count, skill),

        EffectRequest::CityConsume { player, skill } => city_consume(tx, player, skill),

        EffectRequest::QueueExtraTurn {
            player,
            check,
            skill,
        } => queue_extra_turn(tx, player, check, skill),

        EffectRequest::ChooseOthersInsertExtraTurn {
            player,
            check,
            skill,
        } => choose_others_insert_extra_turn(tx, player, check, skill),

        EffectRequest::QueueExtraPhases {
            player,
            phases_mask,
            skill,
        } => queue_extra_phases(tx, player, phases_mask, skill),

        EffectRequest::ResolveTwjieqiu {
            player,
            target,
            subaction,
            skill,
        } => resolve_twjieqiu(tx, player, target, subaction, skill),

        EffectRequest::SkipRemainingPhases {
            player,
            phases_mask,
            skill,
        } => skip_remaining_phases(tx, player, phases_mask, skill),

        EffectRequest::ChooseOthersAddMark {
            player,
            mark,
            delta,
            max_count,
            skill,
        } => choose_others_add_mark(tx, player, mark, delta, max_count, skill),

        EffectRequest::ResolveJsrgsheju {
            player,
            counterpart,
            skill,
        } => resolve_jsrgsheju(tx, player, counterpart, skill),

        EffectRequest::QueueOpeningExtraPlayPhase { player, skill } => {
            queue_opening_extra_play_phase(tx, player, skill)
        }

        EffectRequest::ChooseEndExtraDrawOrPlayByUsedSuits { player, skill } => {
            choose_end_extra_draw_or_play_by_used_suits(tx, player, skill)
        }

        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        } => lose_hp_double_current_damage_then_replace(
            tx,
            player,
            damage_id,
            amount,
            skill,
            replacement_skill_def,
            ready_to_replace,
        ),

        EffectRequest::BeginTurn { player } => begin_turn(tx, player),

        EffectRequest::FinishTurnBegin { player } => finish_turn_begin(tx, player),

        EffectRequest::AdvanceOpeningExtraPlay { player } => advance_opening_extra_play(tx, player),

        EffectRequest::ConvertCurrentShaToFireLoseHpIfDamage { player, card, .. } => {
            crate::command::play::convert_saved_sha_to_fire(tx, player, card)?;
            Ok(Vec::new())
        }

        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            ..
        } => {
            crate::command::play::increase_saved_sha_base_damage(tx, player, card, amount)?;
            Ok(Vec::new())
        }

        EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            ..
        } => {
            crate::command::play::increase_saved_card_base_damage(tx, player, card, amount)?;
            Ok(Vec::new())
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

        EffectRequest::RefundCurrentShaUse {
            player,
            card,
            skill: _,
        } => {
            crate::command::play::refund_resolving_sha_use(tx, player, card)?;
            Ok(Vec::new())
        }

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

        EffectRequest::GuanxingTop {
            player,
            count,
            round: _,
            flipped: _,
            top: _,
            bottom: _,
            skill,
        } => guanxing_top(tx, player, count, skill),

        EffectRequest::ChooseOtherGiveThenRecover { player, skill } => {
            choose_other_give_then_recover(tx, player, skill)
        }

        EffectRequest::GuanxingTopByFaluMarks {
            player,
            marks,
            skill,
        } => guanxing_top_by_falu_marks(tx, player, marks, skill),

        EffectRequest::GiveSelectedThenRecoverSelf {
            player,
            target,
            cards,
            amount,
            skill,
        } => give_selected_then_recover_self(tx, player, target, cards, amount, skill),

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

        EffectRequest::ContinueChooseCompareTargetHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        } => continue_choose_compare_target_hand_card(
            tx,
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        ),

        EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
            player,
            target,
            card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => continue_choose_compare_target_hand_card_second_failure_seat_damage(
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

        EffectRequest::RecordCompareFailureSeatDamage {
            player,
            target,
            failure_count,
            lose_hp,
            damage,
            skill,
        } => {
            let effects = record_compare_failure_seat_damage(
                tx,
                player,
                target,
                failure_count,
                lose_hp,
                damage,
                skill,
            )?;
            super::super::run_effects(tx, effects).map(|_| Vec::new())
        }

        EffectRequest::DealDamageEachInOrder {
            source,
            targets,
            amount,
            skill,
        } => {
            let effects = continue_damage_each_in_order(tx, source, targets, amount, skill)?;
            super::super::run_effects(tx, effects).map(|_| Vec::new())
        }

        EffectRequest::RandomizeCurrentHandCompareOpponent { player, skill } => {
            randomize_current_hand_compare_opponent(tx, player, skill)
        }

        EffectRequest::EmitHandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        } => Ok(vec![DomainEvent::HandCompareFinished {
            player,
            target,
            player_card,
            target_card,
            winner,
        }]),

        EffectRequest::GainHighestRankShaFromEventCards {
            player,
            cards,
            skill,
        } => gain_highest_rank_sha_from_event_cards(tx, player, &cards, skill),

        EffectRequest::GainHandCompareResultCard {
            player,
            initiator,
            target,
            winner,
            cards,
            skill,
        } => gain_hand_compare_result_card(tx, player, initiator, target, winner, &cards, skill),

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

        EffectRequest::AddCardColorLockSkill {
            player,
            source,
            color_mask,
            skill_def,
        } => add_card_color_lock_skill(tx, player, source, color_mask, skill_def),

        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, skill } => {
            use_virtual_sha_from_card_color_lock_source_if_no_sha(tx, player, skill)
        }

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

        EffectRequest::UseVirtualShaDrawIfNoDamage {
            player,
            target,
            skill,
        } => crate::command::play::use_virtual_sha_on_draw_if_no_damage(tx, player, target, skill)
            .map(|_| Vec::new()),

        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            skill,
        } => discard_selected_record_unique_category_target(tx, player, target, card, skill),

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

        EffectRequest::FinishLinkIfTargetWounded {
            player,
            target,
            toggle_link,
            skill: _,
        } => finish_link_if_target_wounded(tx, player, target, toggle_link),

        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, skill } => {
            choose_reveal_other_hand_card_remember_category(tx, player, skill)
        }

        EffectRequest::DrawThenGiveExactRecordPhaseTarget {
            player,
            target,
            count,
            skill,
            ready_to_give,
        } => draw_then_give_exact_record_phase_target(
            tx,
            player,
            target,
            count,
            skill,
            ready_to_give,
        ),

        EffectRequest::ClearPhaseGiftTarget { player, skill } => {
            clear_phase_gift_target(tx, player, skill)
        }

        EffectRequest::EmitShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        } => Ok(vec![DomainEvent::ShaResolved {
            source,
            card,
            targets,
            response_cards,
            card_flags,
        }]),

        EffectRequest::EmitCardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        } => Ok(vec![DomainEvent::CardResolved {
            source,
            card,
            kind,
            targets,
            damaged_targets,
            source_cards,
            all_sources_were_hand,
        }]),

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

        EffectRequest::SkipCurrentPhase { player } => skip_current_phase(tx, player),

        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage {
            player,
            amount,
            skill,
        } => choose_other_targets_by_skipped_phase_count_damage(tx, player, amount, skill, false),

        EffectRequest::ResolveOlJieDanshou { player, skill } => {
            resolve_ol_jie_danshou(tx, player, skill)
        }

        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw {
            player,
            amount,
            skill,
        } => choose_other_targets_by_skipped_phase_count_damage(tx, player, amount, skill, true),

        EffectRequest::RevealUniformHandDrawSelfChooseOthers { player, skill } => {
            reveal_uniform_hand_draw_self_choose_others(tx, player, skill)
        }

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

        EffectRequest::ModifyCurrentDamage {
            player,
            delta,
            skill: _,
        } => modify_current_damage(tx, player, delta),

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

        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage {
            player,
            target,
            skill,
        } => give_all_equipment_choose_recover_or_damage(tx, player, target, skill),

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

        EffectRequest::FinishOpeningExtraPlayShaGain { player, skill } => {
            finish_opening_extra_play_sha_gain(tx, player, skill)
        }

        EffectRequest::RevealTargetHandGainShanArmorOrDrawTwo {
            player,
            target,
            skill,
        } => reveal_target_hand_gain_shan_armor_or_draw_two(tx, player, target, skill),

        EffectRequest::ChooseHandCountDifferentTargetDrawOrRandomDiscard { player, skill } => {
            choose_hand_count_different_target_draw_or_random_discard(tx, player, skill)
        }

        EffectRequest::IncreaseCurrentDrawCount { player, count } => {
            increase_current_draw_count(tx, player, count)
        }

        EffectRequest::SkipJudgeAndDrawPhases { player, skill } => {
            skip_judge_and_draw_phases(tx, player, skill)
        }

        EffectRequest::SkipNextPlayPhase { player } => skip_next_play_phase(tx, player),

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

        EffectRequest::EnableEquipSlot {
            player,
            slot,
            source,
        } => enable_equip_slot(tx, player, slot, source),

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

        EffectRequest::DrawSelectedTargets {
            targets,
            count,
            skill: _,
        } => draw_selected_targets(tx, &targets, count),

        EffectRequest::DamageThenDrawSelectedTargets {
            source,
            targets,
            damage,
            draw,
            skill: _,
        } => damage_then_draw_selected_targets(tx, source, &targets, damage, draw),

        EffectRequest::ResolveYoudi {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_youdi(tx, player, target, card, stage, skill),

        EffectRequest::ResolveKuangfu {
            player,
            target,
            card,
            stage,
            skill,
        } => resolve_kuangfu(tx, player, target, card, stage, skill),

        EffectRequest::ResolveShanjia {
            player,
            count,
            stage,
            skill,
        } => resolve_shanjia(tx, player, count, stage, skill),

        EffectRequest::ResolveNewYajiao {
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        } => resolve_new_yajiao(
            tx,
            player,
            source_card,
            revealed,
            target,
            source_category,
            stage,
            skill,
        ),

        EffectRequest::ResolveDingpin {
            player,
            target,
            skill,
        } => resolve_dingpin(tx, player, target, skill),

        EffectRequest::FinishCurrentPhaseAbortResolution { player, stage } => {
            finish_current_phase_abort_resolution(tx, player, stage)
        }

        EffectRequest::RecoverSelectedTargets {
            source,
            targets,
            amount,
            skill: _,
        } => recover_selected_targets(tx, source, &targets, amount),

        EffectRequest::RecordSelectedPlayPhaseCategory { card, skill } => {
            record_selected_play_phase_category(tx, card, skill)
        }

        EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
            recover_if_min_current_hp(tx, player, amount)
        }

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

        EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
            player,
            target,
            skill,
            ready_to_choose,
        } => draw_then_choose_gain_one_from_damage_target_and_neighbors(
            tx,
            player,
            target,
            skill,
            ready_to_choose,
        ),

        EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
            player,
            targets,
            index,
            selected,
            skill,
        } => continue_choose_gain_one_from_damage_target_and_neighbors(
            tx, player, targets, index, selected, skill,
        ),

        EffectRequest::ChoosePreventDamageOrLoseMaxHpIncreaseDamage {
            player,
            damage_id,
            amount,
            skill,
        } => choose_prevent_damage_or_lose_max_hp_increase_damage(
            tx, player, damage_id, amount, skill,
        ),

        EffectRequest::ContinueIncreaseBoundDamage {
            player,
            damage_id,
            amount,
            ..
        } => {
            if !bound_damage_source_matches(tx, damage_id, player) {
                return Err(EffectError::Illegal(
                    "bound damage increase context changed".into(),
                ));
            }
            increase_bound_damage(tx, damage_id, amount)?;
            Ok(Vec::new())
        }

        EffectRequest::AddCardUsePolicySkill {
            player,
            card,
            policy_mask,
            skill_def,
        } => add_card_use_policy_skill(tx, player, card, policy_mask, skill_def),

        EffectRequest::DiscardTargetCards {
            actor,
            target,
            cards,
            scope,
        } => discard_target_cards(tx, actor, target, cards, scope),

        EffectRequest::RemoveSkillByDef { player, skill_def } => {
            remove_skill_by_def(tx, player, skill_def)
        }

        EffectRequest::ChooseDiscardHandAddSkillByColor {
            player,
            red_skill_def,
            black_skill_def,
        } => choose_discard_hand_add_skill_by_color(tx, player, red_skill_def, black_skill_def),

        EffectRequest::RevealTopGainNonBasicOrSwapBasic { player } => {
            reveal_top_gain_non_basic_or_swap_basic(tx, player)
        }

        EffectRequest::PreventDamage { damage_id } => {
            if damage_id == 0 {
                return Err(EffectError::ContentFault(
                    "prevent_current_damage used outside damage_before".into(),
                ));
            }
            if !tx
                .state()
                .resolution
                .prevented_damage_ids
                .contains(&damage_id)
            {
                tx.state_mut()
                    .resolution
                    .prevented_damage_ids
                    .push(damage_id);
            }
            Ok(Vec::new())
        }

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

        EffectRequest::MoveCards { cards, to, from } => move_cards(tx, cards, to, from),

        EffectRequest::MoveCardsWithReason {
            cards,
            to,
            from,
            reason,
        } => move_cards_with_reason(tx, cards, to, from, reason),

        EffectRequest::GiveCards { cards, to, from } => give_cards(tx, cards, to, from),

        EffectRequest::UseCard {
            player,
            card,
            targets,
        } => use_card(tx, player, card, targets),

        EffectRequest::UseFieldCard {
            player,
            card,
            targets,
        } => use_field_card(tx, player, card, targets),

        EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
        } => deal_damage(
            tx,
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            false,
            CardId::NONE,
            0,
            0,
        ),

        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            let Some(EffectRequest::CommitDamage {
                damage_id,
                source,
                target,
                amount,
                base_amount,
                ..
            }) = prepare_damage(
                tx.state(),
                source,
                target,
                amount,
                false,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
            )?
            else {
                return Ok(Vec::new());
            };
            let (mut events, followups) = commit_damage_ban_active_if_dying(
                tx,
                damage_id,
                source,
                target,
                amount,
                base_amount,
                rule_api::damage_nature::NORMAL,
                skill,
            )?;
            for followup in followups {
                events.extend(apply_effect(tx, followup)?);
            }
            Ok(events)
        }

        EffectRequest::SpjungongDiscardOrLoseThenSha {
            player,
            target,
            skill,
        } => spjungong_discard_or_lose_then_sha(tx, player, target, skill),

        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            ban_active_until_turn_end(tx, player, skill)
        }

        EffectRequest::SpwanweiActive { player, target } => spwanwei_active(tx, player, target),

        EffectRequest::SpwanweiDying { player, target } => spwanwei_dying(tx, player, target),

        EffectRequest::ResolveSpxizhan {
            player,
            phase_player,
            fangzong_skill_def,
        } => resolve_spxizhan(tx, player, phase_player, fangzong_skill_def),

        EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying {
            player,
            target,
            amount,
            skill,
        } => {
            let Some(EffectRequest::CommitDamage { amount, .. }) = prepare_damage(
                tx.state(),
                PlayerId::NONE,
                player,
                amount,
                true,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
            )?
            else {
                return Ok(Vec::new());
            };
            let (events, followups) = commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
                tx, player, target, amount, skill,
            )?;
            debug_assert!(followups.is_empty());
            Ok(events)
        }

        EffectRequest::DealShaDamage {
            source,
            target,
            card,
            amount,
            nature,
            card_flags,
        } => deal_damage(
            tx,
            source,
            target,
            amount,
            false,
            nature,
            true,
            card,
            CardKind::Sha.tag(),
            card_flags,
        ),

        EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            card,
            card_kind,
            card_flags,
        } => deal_damage(
            tx, source, target, amount, is_lose_hp, nature, false, card, card_kind, card_flags,
        ),

        EffectRequest::RecoverHp {
            source,
            player,
            amount,
        } => recover_hp(tx, source, player, amount),

        EffectRequest::PreventRecovery { recovery_id } => {
            if recovery_id == 0 || tx.state().resolution.current_recovery_id != recovery_id {
                return Err(EffectError::ContentFault(
                    "prevent recovery requested outside recover_before".into(),
                ));
            }
            if !tx
                .state()
                .resolution
                .prevented_recovery_ids
                .contains(&recovery_id)
            {
                tx.state_mut()
                    .resolution
                    .prevented_recovery_ids
                    .push(recovery_id);
            }
            Ok(Vec::new())
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

        EffectRequest::ChangeMaxHp { player, delta } => change_max_hp(tx, player, delta),

        EffectRequest::AddSkill { player, skill_def } => add_skill(tx, player, skill_def),

        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => add_bound_skill(tx, player, skill_def, bound_target),

        EffectRequest::RemoveSkill { skill } => remove_skill(tx, skill),

        EffectRequest::RefreshSkill { skill } => refresh_skill(tx, skill),

        EffectRequest::TagCard { cards, tag } => tag_cards(tx, cards, tag),

        EffectRequest::UntagCard { cards, tag } => untag_cards(tx, cards, tag),

        EffectRequest::ClanshenjunReveal {
            player,
            event_card,
            skill,
        } => clanshenjun_reveal(tx, player, event_card, skill),

        EffectRequest::ClanshenjunUse { player, skill } => clanshenjun_use(tx, player, skill),

        EffectRequest::StorageSet { skill, key, value } => storage_set(tx, skill, key, value),

        EffectRequest::DrawAllEventTargets { players, count } => {
            draw_all_event_targets(tx, players, count)
        }

        EffectRequest::RoundEnded { round, current } => {
            Ok(vec![DomainEvent::RoundEnded { round, current }])
        }

        EffectRequest::RoundStarted { round, current } => {
            Ok(vec![DomainEvent::RoundStarted { round, current }])
        }

        EffectRequest::ClanyuzhiReveal { player, skill } => clanyuzhi_reveal(tx, player, skill),

        EffectRequest::ClanyuzhiRoundEnd { player, skill } => {
            clanyuzhi_round_end(tx, player, skill)
        }

        EffectRequest::QieyiPhaseStart { player, skill } => qieyi_phase_start(tx, player, skill),

        EffectRequest::QieyiUseAfter {
            player,
            card,
            skill,
        } => qieyi_use_after(tx, player, card, skill),

        EffectRequest::JianzhiPhaseEnd { player, skill } => jianzhi_phase_end(tx, player, skill),

        EffectRequest::AddMark {
            player,
            mark,
            delta,
        } => add_mark(tx, player, mark.raw(), delta),

        EffectRequest::RemoveMark {
            player,
            mark,
            amount,
        } => remove_mark(tx, player, mark.raw(), amount),

        EffectRequest::RevealCards { cards, to } => {
            Ok(vec![DomainEvent::CardsRevealed { cards, to }])
        }

        EffectRequest::RevealHandDiscardKind { player, kind } => {
            reveal_hand_discard_kind(tx, player, kind)
        }

        EffectRequest::RequestChoice {
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
        } => request_choice(
            tx,
            observer,
            min_count,
            max_count,
            allow_cancel,
            option_tokens,
            ChoiceKind::Custom,
        ),

        EffectRequest::RequestDyingSave { player, source } => {
            request_dying_save(tx, player, source)
        }

        EffectRequest::ContinueDyingSave {
            player,
            source,
            responders,
        } => request_dying_save_from_queue(tx, player, source, responders),

        EffectRequest::EnterDying { player } => enter_dying(tx, player),

        EffectRequest::KillPlayer { player, killer } => kill_player(tx, player, killer),

        EffectRequest::SetTurnState { current, phase } => set_turn(tx, current, phase),

        EffectRequest::AdvanceTurn => {
            crate::command::turn::finish_turn(tx)?;
            Ok(Vec::new())
        }

        EffectRequest::EmitRuleEvent { code, payload } => {
            Ok(vec![DomainEvent::RuleNote { code, payload }])
        }

        EffectRequest::EmitDebateResolved {
            initiator,
            red,
            black,
            result,
        } => Ok(vec![DomainEvent::DebateResolved {
            initiator,
            red,
            black,
            result,
        }]),

        EffectRequest::DiscardFromHand { player, count } => discard_from_hand(tx, player, count),

        EffectRequest::DiscardFromHandEquip { player, count } => {
            discard_from_hand_equip(tx, player, count)
        }

        EffectRequest::AdjustHandTo { player, count } => adjust_hand_to(tx, player, count),

        EffectRequest::DiscardCards { player, cards } => discard_cards(tx, player, cards),

        EffectRequest::ChooseDiscardOneHandOrEquip { player } => {
            choose_discard_one_hand_or_equip(tx, player)
        }

        EffectRequest::DiscardAllHandAndEquip { player } => discard_all_hand_and_equip(tx, player),

        EffectRequest::RemoveAllSkills { player } => remove_all_skills(tx, player),

        EffectRequest::JudgeGainEquipmentByColor { player } => {
            judge_gain_equipment_by_color(tx, player)
        }

        EffectRequest::JudgeCancelCurrentShaTargetOnRed {
            player,
            card,
            target,
        } => judge_cancel_current_sha_target_on_red(tx, player, card, target),

        EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        } => make_current_sha_target_unrespondable(tx, source, card, target),

        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        } => judge_make_current_sha_target_unrespondable_on_red(
            tx,
            player,
            card,
            target,
            retieji,
            skill,
            ready_to_choose,
            judgment_suit,
        ),

        EffectRequest::ChooseDiscardBasicThenTargetDiscardEquipOrDamage {
            player,
            target,
            skill,
        } => choose_discard_basic_then_target_discard_equip_or_damage(tx, player, target, skill),

        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner {
            player,
            target,
            skill,
        } => target_discard_then_duel_if_non_sha_and_hp_gte_owner(tx, player, target, skill),

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
