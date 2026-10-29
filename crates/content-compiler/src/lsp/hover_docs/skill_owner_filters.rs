//! LSP 悬停文档链：持有者条件（owner_*）说明。

pub(in super::super) fn documentation(word: &str) -> Option<&'static str> {
    Some(match word {
            "owner_missing_skill" => {
                "Active-skill clause: activation is legal only while the owner does not have the given stable skill definition. Requires Rule VM ABI 234."
            }
            "owner_alive_turn_ended_rehengjiang_pending" => {
                "横江 turn-end filter: the living helper owner has pending state for the ended turn. Requires Rule VM ABI 572."
            }
            "owner_gained_cards_potzhouyu" => {
                "炽沄 gain filter: the living owner gained cards and still has hand cards. Requires Rule VM ABI 561."
            }
            "owner_source_first_target_has_hand_potyanhui" => {
                "焰洄 target filter: the owner used a card, this is its first target event, and some target has hand cards. Requires Rule VM ABI 561."
            }
            "owner_alive_fire_damage_potfentao" => {
                "焚涛 fire-damage filter; target and chain eligibility are revalidated by the resolver. Requires Rule VM ABI 561."
            }
            "owner_alive_potfentao_pending_damage" => {
                "焚涛 post-damage filter for its pending relink target. Requires Rule VM ABI 561."
            }
            "owner_source_sha_damage_before_potyinzhan_condition" => {
                "饮战 Damage-before filter: the owner dealt Sha damage and has no more HP or hand cards than the target. Requires Rule VM ABI 558."
            }
            "owner_source_damage_within_one" => {
                "狂骨 Damage filter: the owner dealt positive damage to a living target at distance 1 or less. Requires Rule VM ABI 558."
            }
            "owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused" => {
                "诫节 Play-phase filter: owner and current actor are alive, the actor has hand cards, and no 诫节 has been invoked by that actor this phase. Requires Rule VM ABI 554."
            }
            "owner_lost_non_damage_card_pottuntian_charge_not_full" => {
                "Cards-moved filter: the owner lost at least one non-damage card from a personal zone, excluding an equipment card consumed by using it, while 势屯田 charge is below its stored maximum. Requires Rule VM ABI 543."
            }
            "owner_in_target_attack_range" => {
                "Active-target constraint: the skill owner must currently be inside every selected target's attack range. Requires Rule VM ABI 544."
            }
            "owner_hp" => {
                "Query-time modifier value: the skill owner's current HP, clamped to zero."
            }
            "owner_max_hp" => "Query-time modifier value: the skill owner's maximum HP.",
            "owner_hp_minus_one" => {
                "Query-time modifier value: the skill owner's current HP minus one, clamped to zero."
            }
            "owner_lost_hp" => {
                "Query-time modifier value: the skill owner's current lost HP, clamped to zero."
            }
            "owner_lost_hp_minus_one" => {
                "Query-time modifier value: the skill owner's current lost HP minus one, clamped to zero."
            }
            "owner_lost_hp_min_one" => {
                "Query-time modifier value: the skill owner's current lost HP, with a minimum of one."
            }
            "owner_skill_mark" => {
                "Query-time modifier value: the owner's mark count whose id equals this skill definition id."
            }
            "owner_skill_state_u32" => {
                "Query-time modifier value: the first little-endian u32 in this skill instance's state blob. ABI 522."
            }
            "owner_skill_state_u32_negative" => {
                "Query-time modifier value: the negative first little-endian u32 in this skill instance's state blob. ABI 612."
            }
            "owner_mark_2058376721" => {
                "Parameterized query-time modifier value: the owner's mark count whose stable definition id is the numeric suffix."
            }
            "owner_has_weapon" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains a weapon, otherwise zero."
            }
            "owner_no_weapon" => {
                "Query-time modifier value: one while the skill owner's equipment zone lacks a weapon, otherwise zero."
            }
            "owner_has_armor_equip" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains an armor, otherwise zero."
            }
            "owner_unique_max_hand" => {
                "Query-time modifier value: one while the skill owner uniquely has the most hand cards among living players, otherwise zero."
            }
            "owner_hand_gt_other" => {
                "Contextual damage modifier value: one while the skill owner's hand count is greater than the other damage participant's, otherwise zero."
            }
            "owner_hand_lt_other" => {
                "Contextual damage modifier value: one while the skill owner's hand count is less than the other damage participant's, otherwise zero."
            }
            "owner_qun_other_shu" => {
                "Contextual damage modifier value: one while the skill owner is a living qun character and the other damage participant is a living shu character, otherwise zero. Requires Rule VM ABI 81."
            }
            "owner_turned_over_diff_other" => {
                "Contextual damage modifier value: one while the skill owner's face-up/face-down state differs from the other damage participant's, otherwise zero."
            }
            "owner_linked_diff_other" => {
                "Contextual damage modifier value: one while the skill owner and the other damage participant have different linked states, otherwise zero."
            }
            "owner_hand_equip_gte_other" => {
                "Contextual damage modifier value: one while the skill owner's hand and equipment counts are each no smaller than the other damage participant's corresponding count, otherwise zero."
            }
            "owner_sha_damage_dealt_zero_this_turn" => {
                "Turn-scoped damage modifier value: one before the skill owner has dealt any logical-Sha damage event in the current turn, otherwise zero."
            }
            "owner_no_mount" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains no mount, otherwise zero."
            }
            "owner_armor_empty" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains no armor, otherwise zero."
            }
            "owner_healthy" => {
                "Query-time modifier value: one while the skill owner is at full health, otherwise zero."
            }
            "owner_hand_empty" => {
                "Trigger filter or query-time modifier value: matches while the skill owner's hand is empty."
            }
            "owner_has_treasure" => {
                "Query-time modifier value: one while the skill owner's equipment zone contains a treasure, otherwise zero."
            }
            "owner_equip_count_odd" => {
                "Query-time modifier value: one while the skill owner's equipment-card count is odd, otherwise zero."
            }
            "owner_equip_count_even" => {
                "Query-time modifier value: one while the skill owner's equipment-card count is even (including zero), otherwise zero."
            }
            "owner_equip_count_half_floor" => {
                "Query-time modifier value: half the skill owner's equipment-card count, rounded down."
            }
            "owner_equip_count_exactly_one" => {
                "Query-time modifier value: one while the skill owner has exactly one equipment card, otherwise zero."
            }
            "owner_equip_count" => {
                "Query-time modifier value: the number of cards in the skill owner's equipment zone."
            }
            "owner_hp_gt_two" => {
                "Query-time modifier value: one while the skill owner's current HP is greater than two, otherwise zero."
            }
            "owner_hp_le_two" => {
                "Query-time modifier value: one while the skill owner's current HP is at most two, otherwise zero."
            }
            "owner_lost_hp_at_least_two" => {
                "Query-time modifier value: one while the skill owner's lost HP is at least two, otherwise zero."
            }
            "owner_all_others_in_attack_range" => {
                "Query-time modifier value: one while every other alive player is within the skill owner's attack range, otherwise zero."
            }
            "owner_current_turn_all_others_distance_at_most_one" => {
                "Query-time modifier value: one during the owner's current turn while every other living player's final distance from the owner is at most one; otherwise zero. Requires Rule VM ABI 281."
            }
            "owner_sha_used_zero" => {
                "Query-time modifier value: one before the first Sha is used in the current turn, otherwise zero."
            }
            "owner_sha_used_nonzero" => {
                "Query-time modifier value: one after the skill owner uses at least one Sha in the current turn, otherwise zero."
            }
            "owner_sha_used_exactly_one" => {
                "Query-time modifier value: one after the skill owner uses exactly one Sha in their current turn, otherwise zero."
            }
            "owner_sha_used_exactly_two" => {
                "Query-time modifier value: one after the skill owner uses exactly two Sha in their current turn, otherwise zero."
            }
            "owner_max_hand_before_skill_fires_this_turn" => {
                "Query-time modifier value: one while the skill owner ties for maximum hand size and this skill has not fired in the current turn, otherwise zero."
            }
            "owner_non_damage_hand_type_count" => {
                "Query-time modifier value: number of distinct basic, trick, and equipment categories among the owner's non-damage hand cards."
            }
            "owner_sha_hand_count" => {
                "Query-time modifier value: number of Sha cards in the skill owner's hand (任城 pxrencheng)."
            }
            "owner_turn_damage_dealt" => {
                "Query-time modifier value: total damage dealt by the skill owner during their current turn."
            }
            "owner_current_turn_card_use_count" => {
                "Query-time modifier value: cards proactively used by the skill owner during their current turn; zero outside that owner's turn. Requires Rule VM ABI 49."
            }
            "owner_play_phase_card_use_count_zero" => {
                "Query-time modifier value: one before the skill owner proactively uses a card in their current Play phase; zero otherwise. Requires Rule VM ABI 115."
            }
            "owner_skill_play_phase_fire_count_below_2" => {
                "Parameterized query-time modifier value: one during the owner's Play phase while this skill instance has fired fewer than the positive numeric suffix; zero otherwise. Requires Rule VM ABI 220."
            }
            "owner_skill_use_count" => {
                "Query-time modifier value: originating active skill invocations in the current turn. Requires Rule VM ABI 143."
            }
            "owner_has_heart_equip" => {
                "Query-time modifier value: one while the owner's equipment zone contains a card whose effective suit is heart."
            }
            "owner_outside_turn" => {
                "View-as filter: matches any legal source card only while the skill owner is not the current turn player. Requires Rule VM ABI 289."
            }
            "owner_alive_is_source" => {
                "Trigger filter: the skill owner is the event source and remains alive after the event. Requires Rule VM ABI 291."
            }
            "owner_alive_shan_used_by_owner_or_during_owner_turn" => {
                "Card-used trigger filter: the owner is alive, the logical card is Shan, and either the owner used it or it was used during the owner's current turn. Requires Rule VM ABI 287."
            }
            "owner_alive_other_phase_play_skill_pile_nonempty" => {
                "Play-phase trigger filter: the owner and entering player are distinct and alive, and the originating skill's public pile is nonempty. Requires Rule VM ABI 287."
            }
            "owner_is_target_other_hand_nonempty" => {
                "Phase trigger filter: the owner is the entering Play- or Draw-phase player and at least one other living player has a hand card. Requires Rule VM ABI 53."
            }
            "owner_is_target_positive_draw_other_hand_nonempty" => {
                "Draw-phase trigger filter: the living owner is entering an unfixed Draw phase with a positive pending draw count and at least one other living player has a hand card. Requires Rule VM ABI 311."
            }
            "owner_source_first_target_black_card_any_target_other_discardable_hand_equip" => {
                "Card-target trigger filter: the living owner used a black card, this is its first target event, and at least one other target has a discardable hand/equipment card. Requires Rule VM ABI 313."
            }
            "owner_source_play_card_matches_remembered_category" => {
                "Card-used trigger filter: during the owner's Play phase, the used card's effective basic/trick/equipment category matches the category remembered by this skill instance. Requires Rule VM ABI 53."
            }
            "owner_phase_gift_target_is_source" => {
                "Death trigger filter: the killer matches this skill instance's delayed Play-phase gift target. Requires Rule VM ABI 53."
            }
            "owner_phase_gift_target_is_target" => {
                "Play-phase-end trigger filter: the ending phase player matches this skill instance's delayed gift target. Requires Rule VM ABI 53."
            }
            "owner_source_sha_resolved_has_gainable_cards_and_recipient" => {
                "Sha-resolved trigger filter: the living owner used this Sha, at least one physical Sha/Shan group remains in discard, and at least one living recipient is neither the owner nor an original target. Requires Rule VM ABI 54."
            }
            "owner_is_target_tied_max_hand" => {
                "Phase trigger filter: the living owner is the event player and ties for the largest hand among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_tied_max_hp" => {
                "Phase trigger filter: the living owner is the event player and ties for the highest current HP among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_tied_max_equip_nonempty" => {
                "Phase trigger filter: the living owner is the event player, has at least one equipment card, and ties for the largest equipment count among living players. Requires Rule VM ABI 55."
            }
            "owner_is_target_skipped_phase_count_positive" => {
                "Phase trigger filter: the living owner is the event player and at least one phase has been skipped this turn. Requires Rule VM ABI 55."
            }
            "owner_discarded_has_hand_once_per_phase" => {
                "Cards-moved trigger filter: the living owner discarded at least one hand/equipment card, still has hand cards, and has not accepted this trigger in the current phase. Requires Rule VM ABI 56."
            }
            "owner_alive_lost_hand_this_turn" => {
                "Phase-end trigger filter: the living owner lost at least one hand/equipment card during the current turn. Requires Rule VM ABI 405."
            }
            "owner_source_first_sha_this_play_phase" => {
                "Card-used trigger filter: the living owner used the first logical Sha of their current Play phase. Requires Rule VM ABI 407."
            }
            "owner_source_damage_target_distance_at_most_one_empty_equip_slot_unused" => {
                "Damage trigger filter: the owner dealt damage to a target within distance 1, has an enabled empty equipment slot, and has not fired this turn. Requires Rule VM ABI 379."
            }
            "owner_source_first_sha_target_this_turn" => {
                "Card-targeted trigger filter: this target belongs to the owner's first logical Sha of the current turn. Requires Rule VM ABI 379."
            }
            "owner_alive_other_phase_play_end_twzhengjian_condition" => {
                "Play-phase-end filter: another living player failed the current Zhengjian branch (no non-basic use or no gain), and can pay unless Zhengjian has awakened. Requires Rule VM ABI 381."
            }
            "owner_source_sha_target_twzhenxi_available_once_per_turn" => {
                "Card-targeted filter: once per turn, the owner's Sha target has discardable hand cards or a field card that can move to another player. Requires Rule VM ABI 382."
            }
            "owner_source_first_target_damage_card_once_per_turn" => {
                "Card-targeted filter: the first declared target of the owner's Sha or damage trick, once per turn. Requires Rule VM ABI 384."
            }
            "owner_outside_turn_discarded_red_cards_in_discard" => {
                "Cards-moved filter: outside the owner's turn, at least one red card discarded by the owner remains in the shared discard pile. Requires Rule VM ABI 384."
            }
            "owner_source_single_target_sha_or_black_ordinary_trick_current_play_phase_unused" => {
                "Card-used trigger filter: during the owner's Play phase, the owner used a single-target Sha or black ordinary trick and this skill has not completed in the current phase. Requires Rule VM ABI 57."
            }
            "owner_is_target_male_source_has_discardable_equipment" => {
                "Damage-before trigger filter: the living owner is the damage target, the living source is male/double-sex, and the owner can discard an effective equipment card from hand/equipment. Requires Rule VM ABI 57."
            }
            "owner_alive_other_turn_ended_no_card_targeted_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player used no card targeting another player during the ended turn. Requires Rule VM ABI 58."
            }
            "owner_alive_other_turn_ended_card_targeted_other_no_damage_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player targeted another player with a card but dealt no damage to another player during the ended turn. Requires Rule VM ABI 146."
            }
            "owner_alive_phase_targeted_hp_changed_can_use_wanjian" => {
                "Phase-any-end filter: the living owner was named as a card target and their actual HP changed during the ending phase, and a source-free Wanjian is currently legal. Requires Rule VM ABI 98."
            }
            "owner_alive_phase_targeted_hp_unchanged_can_use_juedou" => {
                "Phase-any-end filter: the living owner was named as a card target and their actual HP did not change during the ending phase, and a source-free Juedou is currently legal. Requires Rule VM ABI 98."
            }
            "owner_source_sha_direct_damage_before" => {
                "Damage-before trigger filter: the living owner is the source of direct logical-Sha damage and the event player is its living target. Chain-propagated damage is excluded. Requires Rule VM ABI 147."
            }
            "owner_source_sha_has_weapon" => {
                "Damage-before trigger filter: the living owner is the source of Sha damage, the event player is its living target, and the owner has a physical weapon equipped. Requires Rule VM ABI 59."
            }
            "owner_source_other_damage_before_once_per_turn" => {
                "Damage-before trigger filter: once per current turn, the living owner is the damage source and the living event player is another player. Requires Rule VM ABI 322."
            }
            "owner_alive_other_turn_ended_no_damage_other" => {
                "Turn-ended trigger filter: the living owner is not the living ending player, and that player dealt no damage to any other player during the ended turn. Requires Rule VM ABI 323."
            }
            "owner_alive_gained_two_this_turn_can_compare_other" => {
                "End-phase trigger filter: the living owner gained at least two physical cards during the current turn, has a hand card, and another living player has a hand card for comparison. Card gains are tracked only when loaded content needs this filter. Requires Rule VM ABI 325."
            }
            "owner_source_or_target_hand_count_differs_once_per_turn" => {
                "Damage trigger filter: the living owner is the damage source or target, some living player's hand count differs from the owner's, and this skill has not fired in the current turn. Requires Rule VM ABI 60."
            }
            "owner_hand_compare_participant" => {
                "Hand-comparison trigger filter: the living owner is either the initiator or target of the current comparison."
            }
            "owner_skill_pile_nonempty_<def>" => {
                "Trigger filter: the owner's named skill pile (keyed by the skill definition id literal) holds at least one card. Matches upstream getExpansions(...).length > 0 gates."
            }
            "owner_is_target_damage_amount_gt_one" => {
                "Trigger filter: the owner is the damage target and the incoming damage amount is greater than one (照月狮子盔 gate)."
            }
            "owner_phase_end_lost_cards" => {
                "Trigger filter: at the owner\'s own end phase, if they have lost at least one card this turn (镇抚 gate)."
            }
            "owner_alive_second_global_dying" => {
                "Trigger filter: after the second dying window of the whole match resolves (觉醒 gate; fires exactly once)."
            }
            "owner_is_target_source_hp_gt_owner_or_source_damaged_owner" => {
                "Trigger filter: you are targeted by another player\'s card, and their hp is higher than yours, or they have damaged you this match (粗遐 gate)."
            }
            "owner_is_target_owner_hp_gt_source_or_source_damaged_owner" => {
                "Trigger filter: you are targeted by another player\'s card, and your hp is higher than theirs, or they have damaged you this match (未命 gate)."
            }
            "owner_is_target_sanshou_type_tracked" => {
                "Trigger filter: the owner is the damage target (carries the used-type bitmap track for 三首)."
            }
            "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" => {
                "Trigger filter: the owner is the damage source during their own turn, the target has never taken damage this match, and the owner\'s lifetime card actions reach the target\'s hp (威临 gate)."
            }
            "owner_used_and_was_targeted_this_turn_in_range" => {
                "Trigger filter: at any player\'s end phase, if this turn you have used or played a card, been named a card target, and someone is within your attack range (直谏 gate)."
            }
            "owner_is_target_source_hp_gt_owner_per_role_turn" => {
                "Trigger filter: once per role turn, when the owner is targeted by a card and the source has more HP than the owner."
            }
            _ => return super::discard_skill_pile::documentation(word),
    })
}
