//! 效果原语分派（apply_effect）：通用效果原语。

use super::*;

pub(super) fn apply_effect(
    tx: &mut Transaction,
    req: EffectRequest,
) -> Result<Vec<DomainEvent>, EffectError> {
    match req {
        EffectRequest::SetSkillContentFlag {
            player,
            bit,
            value,
            skill,
        } => set_skill_content_flag(tx, player, bit, value, skill),

        EffectRequest::DiscardEquipmentSlot { player, slot } => {
            discard_equipment_slot(tx, player, slot)
        }

        EffectRequest::StoreSelectedInSkillPile {
            player,
            cards,
            skill,
        } => store_selected_in_skill_pile(tx, player, cards, skill),

        EffectRequest::GainTopCardToSkillPile { player, skill } => {
            gain_top_card_to_skill_pile(tx, player, skill)
        }

        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        } => choose_discard_skill_pile_card_add_marked_skill(
            tx,
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        ),

        EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill,
        } => gain_one_hand_from_selected_targets(tx, player, targets, index, selected, skill),

        EffectRequest::ChooseRepoluTargetsDraw { player, skill } => {
            choose_repolu_targets_draw(tx, player, skill)
        }

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
            player,
            sha_user,
            card,
            restore_skill_def,
            skill,
        } => choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(
            tx,
            player,
            sha_user,
            card,
            restore_skill_def,
            skill,
        ),

        EffectRequest::EndMatchForCampIfSkillPileAtLeast {
            player,
            pile_skill_def,
            count,
        } => end_match_for_camp_if_skill_pile_at_least(tx, player, pile_skill_def, count),

        EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            stage,
            cards,
            skill,
        } => replace_draw_with_distinct_suit_selection(tx, player, count, stage, cards, skill),

        EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage,
            card,
            skill,
        } => {
            inspect_target_hand_choose_heart_discard_or_top(tx, player, target, stage, card, skill)
        }

        EffectRequest::ChooseAllLoseHpOrRecover { player, skill } => {
            choose_all_lose_hp_or_recover(tx, player, skill)
        }

        EffectRequest::ChooseLinkOther { player, skill } => choose_link_other(tx, player, skill),

        EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
            player,
            kind,
            mandatory_skill_def,
            skill,
        } => lose_hp_gain_discard_card_for_opening_extra_play(
            tx,
            player,
            kind,
            mandatory_skill_def,
            skill,
        ),

        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            draw_for_lifetime_card_action_divisors(tx, player, divisors)
        }

        EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player,
            divisors,
            groups,
        } => gain_draw_cards_for_lifetime_card_action_divisors(tx, player, divisors, groups),

        EffectRequest::ChooseAddMissingSkill {
            player,
            skill_defs,
            skill,
        } => choose_add_missing_skill(tx, player, skill_defs, skill),

        EffectRequest::AddFirstMissingSkill {
            player,
            skill_defs,
            skill,
        } => add_first_missing_skill(tx, player, skill_defs, skill),

        EffectRequest::RecordCurrentShaAllTargets {
            player,
            card,
            skill,
        } => record_current_sha_all_targets(tx, player, card, skill),

        EffectRequest::ChooseGiveOrDiscardHand {
            player,
            payment_count,
            scope,
            skill,
        } => {
            if hand_cards_sorted(tx, player).len() < payment_count as usize {
                Ok(Vec::new())
            } else {
                draw_then_choose_give_or_discard_hand(
                    tx,
                    player,
                    0,
                    payment_count,
                    skill,
                    true,
                    scope,
                )
            }
        }

        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            skill,
        } => {
            choose_discard_hand_equip_then_recover_target(tx, chooser, target, count, amount, skill)
        }

        EffectRequest::JudgeRecoverOnSuit {
            source,
            player,
            suit,
            amount,
        } => judge_recover_on_suit(tx, source, player, suit, amount),

        EffectRequest::JudgeRecoverOtherOnSuit {
            source,
            judge_player,
            recover_player,
            suit,
            amount,
        } => judge_recover_other_on_suit(tx, source, judge_player, recover_player, suit, amount),

        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player,
            remaining,
            started,
            skill,
        } => force_others_use_sha_on_nearest_or_lose_hp(tx, player, remaining, started, skill),

        EffectRequest::JudgeDamageOtherOnSuit {
            player,
            suit,
            amount,
            nature,
        } => judge_damage_other_on_suit(tx, player, suit, amount, nature),

        EffectRequest::JudgeSourceDiscardTwoHandOrDamage { player, source } => {
            judge_source_discard_two_hand_or_damage(tx, player, source)
        }

        EffectRequest::GiveSelectedRecoverAfterPhaseTotalTwo {
            player,
            target,
            cards,
            skill,
            ready_to_recover,
        } => give_selected_recover_after_phase_total_two(
            tx,
            player,
            target,
            cards,
            skill,
            ready_to_recover,
        ),

        EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
            player,
            targets,
            index,
            selected,
            skill,
        } => choose_other_hand_targets_gain_one_replace_draw(
            tx, player, targets, index, selected, skill, None,
        ),

        EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
            player,
            targets,
            index,
            selected,
            draw_count,
            skill,
        } => choose_other_hand_targets_gain_one_replace_draw(
            tx,
            player,
            targets,
            index,
            selected,
            skill,
            Some(draw_count),
        ),

        EffectRequest::DistributeTopTwoCards {
            player,
            cards,
            selected,
            target,
            skill,
            ready_to_choose,
        } => distribute_top_two_cards(tx, player, cards, selected, target, skill, ready_to_choose),

        EffectRequest::RecastCards { player, cards } => recast_cards(tx, player, cards),

        EffectRequest::DisableEquipSlots { player, slots } => {
            disable_equip_slots(tx, player, slots)
        }

        EffectRequest::MoveEquipmentThenDraw { player, skill } => {
            move_equipment_then_draw(tx, player, skill)
        }

        EffectRequest::DiscardEquipmentPairWithArmor {
            player,
            targets,
            skill,
        } => discard_equipment_pair_with_armor(tx, player, targets, skill),

        EffectRequest::DiscardTargetEquipmentGainMount { player, target } => {
            discard_target_equipment_gain_mount(tx, player, target)
        }

        EffectRequest::DiscardTargetEquipmentDrawWeaponRange {
            player,
            target,
            skill,
        } => discard_target_equipment_draw_weapon_range(tx, player, target, skill),

        EffectRequest::ChooseTargetRevealHandDamageIfSha {
            player,
            amount,
            skill,
        } => choose_target_reveal_hand_damage_if_sha(tx, player, amount, skill),

        EffectRequest::ChooseOtherGiveOneHandEquip { player, skill } => {
            choose_other_give_one_hand_equip(tx, player, skill)
        }

        EffectRequest::ChooseGiveOrRecastOneHand {
            player,
            skill,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
        } => choose_give_or_recast_one_hand(
            tx,
            player,
            skill,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
        ),

        EffectRequest::ChooseRevealHandCardBindCategoryTarget {
            player,
            target,
            skill,
        } => choose_reveal_hand_card_bind_category_target(tx, player, target, skill),

        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        } => recover_to_max_then_optional_exchange_skill(
            tx,
            player,
            skill,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose,
        ),

        EffectRequest::DiscardTargetCardGainMount {
            player,
            target,
            scope,
        } => discard_target_card_gain_mount(tx, player, target, scope),

        EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        } => choose_discard_target_card(tx, player, target, scope),

        EffectRequest::JudgePreventDamageLoseMaxHpOnNonHeart {
            player,
            target,
            damage_id,
        } => judge_prevent_damage_lose_max_hp_on_nonheart(tx, player, target, damage_id),

        EffectRequest::ChooseGainTopCardsBySuit {
            player,
            count,
            suit,
        } => choose_gain_top_cards_by_suit(tx, player, count, suit),

        EffectRequest::ChooseOtherDrawByHandCount {
            player,
            normal_count,
            empty_count,
            times,
            skill,
        } => choose_other_draw_by_hand_count(tx, player, normal_count, empty_count, times, skill),

        EffectRequest::ChooseDiscardOneThenGainTargetEquipment {
            player,
            target,
            slot,
        } => choose_discard_one_then_gain_target_equipment(tx, player, target, slot),

        EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            skill,
        } => choose_target_draw(tx, player, count, scope, skill),

        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            skill,
        } => choose_targets_draw(tx, player, count, target_count, scope, skill),

        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            skill,
        } => choose_targets_damage(tx, player, min_count, max_count, scope, amount, skill),

        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            skill,
        } => choose_optional_targets_draw(tx, player, count, min_count, max_count, scope, skill),

        EffectRequest::RevealTargetHandShaOrDiscardBind {
            player,
            target,
            helper_skill_def,
            skill,
            stage,
        } => reveal_target_hand_sha_or_discard_bind(
            tx,
            player,
            target,
            helper_skill_def,
            skill,
            stage,
        ),

        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, skill } => {
            choose_target_draw_to_max_hp_cap(tx, player, cap, skill)
        }

        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, skill } => {
            choose_other_draw_owner_lost_hp_turn_over(tx, player, skill)
        }

        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. } => {
            let count = tx.state().turn.play_card_category_count(player);
            draw_cards(tx, player, count)
        }

        EffectRequest::RefundCurrentShaUseAndDrawIfRed {
            player,
            sha_user,
            card,
            ..
        } => refund_current_sha_use_and_draw_if_red(tx, player, sha_user, card),

        EffectRequest::ChooseDiscardOneForShaRefundAndDrawIfRed {
            player,
            sha_user,
            card,
            skill,
        } => choose_discard_one_for_sha_refund_and_draw_if_red(tx, player, sha_user, card, skill),

        EffectRequest::ChooseTargetGainMaxHp { player, skill } => {
            choose_target_gain_max_hp(tx, player, skill)
        }

        EffectRequest::GainTargetHandCard { player, skill } => {
            gain_target_hand_card(tx, player, skill)
        }

        EffectRequest::ChooseClanNotMaxHandLimitGain { player, skill } => {
            choose_clan_not_max_hand_limit_gain(tx, player, skill)
        }

        EffectRequest::ExtraResolveCurrentTrick {
            player,
            card,
            skill,
        } => extra_resolve_current_trick(tx, player, card, skill),

        EffectRequest::PlaceHandCardToExpansion { player, skill } => {
            place_hand_card_to_expansion(tx, player, skill)
        }

        EffectRequest::GainFromSkillPile { player, skill } => {
            gain_from_skill_pile(tx, player, skill)
        }

        EffectRequest::DiscardSkillPile {
            player,
            skill,
            pile_def,
        } => discard_skill_pile(tx, player, skill, pile_def),

        EffectRequest::PlaceTopDeckToExpansion {
            player,
            skill,
            pile_def,
            count,
        } => place_top_deck_to_expansion(tx, player, skill, pile_def, count),

        EffectRequest::ChooseOneFromSkillPileDiscard {
            player,
            skill,
            pile_def,
        } => choose_one_from_skill_pile_discard(tx, player, skill, pile_def),

        EffectRequest::GainWholeSkillPile {
            player,
            skill,
            pile_def,
        } => gain_whole_skill_pile(tx, player, skill, pile_def),

        EffectRequest::RearrangePileToDeckTop {
            player,
            skill,
            pile_def,
            count,
        } => rearrange_pile_to_deck_top(tx, player, skill, pile_def, count),

        EffectRequest::ChooseHandCardsToSkillPile {
            player,
            skill,
            min_count,
            max_count,
        } => choose_hand_cards_to_skill_pile(tx, player, skill, min_count, max_count),

        EffectRequest::ResetLinkedSelectedTargets { source, targets } => {
            reset_linked_selected_targets(tx, source, &targets)
        }

        EffectRequest::CommitGaoyuanRedirect {
            player,
            source,
            card,
            target,
        } => {
            let candidates = gaoyuan_redirect_candidates(tx, player, source, card)?;
            if !candidates.contains(&target) {
                return Ok(Vec::new());
            }
            let targets =
                crate::command::play::redirect_saved_sha_target(tx, source, card, player, target)?;
            Ok(vec![DomainEvent::CardTargetAdded {
                player: source,
                card,
                target,
                targets,
            }])
        }

        EffectRequest::PlaceHandCardsToDeckTop {
            player,
            skill,
            count,
        } => place_hand_cards_to_deck_top(tx, player, skill, count),

        EffectRequest::ChooseTargetHandCardToPile {
            player,
            skill,
            pile_def,
        } => choose_target_hand_card_to_pile(tx, player, skill, pile_def),

        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            skill,
        } => choose_other_add_mark(tx, player, mark, delta, skill),

        EffectRequest::StartDebateAllOthers { player, skill } => {
            start_debate_all_others(tx, player, skill)
        }

        EffectRequest::DiscardBasicDrawEqual { player, skill } => {
            discard_basic_draw_equal(tx, player, skill)
        }

        EffectRequest::StartDebateTargets { player, skill } => {
            start_debate_targets(tx, player, skill)
        }
        _ => unreachable!("effect routed to a different primitive group"),
    }
}
