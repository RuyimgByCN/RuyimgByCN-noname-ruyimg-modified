use super::*;

pub(super) fn items() -> Vec<LspCompletionItem> {
    vec![
            LspCompletionItem {
                label: "effect draw_cards".into(),
                kind: 15, // Snippet
                detail: "Draw N cards for self".into(),
                insert_text: "effect draw_cards(self, ${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect draw_each_by_seat".into(),
                kind: 15,
                detail: "Draw equally for two players in seat order".into(),
                insert_text:
                    "effect draw_each_by_seat(${1:event_player}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_then_give_one".into(),
                kind: 15,
                detail: "Draw, then force one hand/equipment gift".into(),
                insert_text: "effect draw_then_give_one(self, ${1:event_source}, ${2:2}, ${3:any_hand_equip});".into(),
            },
            LspCompletionItem {
                label: "effect draw_then_take_one".into(),
                kind: 15,
                detail: "Draw, then let another player take one hand/equipment card".into(),
                insert_text:
                    "effect draw_then_take_one(self, ${1:event_source}, ${2:3});".into(),
            },
            LspCompletionItem {
                label: "effect draw_then_choose_give_or_discard_hand".into(),
                kind: 15,
                detail: "Draw, then give an exact number of hand cards or discard them".into(),
                insert_text:
                    "effect draw_then_choose_give_or_discard_hand(self, ${1:2}, ${2:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_or_discard_hand".into(),
                kind: 15,
                detail: "Give hand cards to a hand-empty target or discard them".into(),
                insert_text:
                    "effect choose_give_or_discard_hand(self, ${1:2}, hand_empty);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_equip_then_recover_target".into(),
                kind: 15,
                detail: "Optionally discard an exact cost, then recover a specified target"
                    .into(),
                insert_text: "effect choose_discard_hand_equip_then_recover_target(self, ${1:event_player}, ${2:2}, ${3:1});".into(),
            },
            LspCompletionItem {
                label: "effect take_one_card".into(),
                kind: 15,
                detail:
                    "Let a player take one card from hand/equipment/judgment zones".into(),
                insert_text:
                    "effect take_one_card(${1:event_player}, self, ${2:hand_equip_judge});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect take_one_event_target_hand".into(),
                kind: 15,
                detail: "Take one hand card from the event counterpart".into(),
                insert_text: "effect take_one_event_target_hand(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_end_extra_draw_or_play_by_used_suits".into(),
                kind: 15,
                detail: "Choose extra Draw/Play phases after End from used-card suits".into(),
                insert_text:
                    "effect choose_end_extra_draw_or_play_by_used_suits(self);".into(),
            },
            LspCompletionItem {
                label: "effect lose_hp_double_current_damage_then_replace".into(),
                kind: 15,
                detail: "Lose HP, double the bound damage, then replace this skill".into(),
                insert_text:
                    "effect lose_hp_double_current_damage_then_replace(self, ${1:replacement_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect take_target_hand_then_repeat_until_max".into(),
                kind: 15,
                detail: "Take from a larger hand, then optionally repeat until tied for maximum"
                    .into(),
                insert_text:
                    "effect take_target_hand_then_repeat_until_max(self, ${1:event_source}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_event_cards".into(),
                kind: 15,
                detail: "Gain all captured event cards still in discard".into(),
                insert_text: "effect gain_event_cards(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect gain_card_if_discard".into(),
                kind: 15,
                detail: "Gain one physical card if it remains in discard".into(),
                insert_text:
                    "effect gain_card_if_discard(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_discard_target_card_record_sha".into(),
                kind: 15,
                detail: "Choose draw/discard and bind the current Sha target".into(),
                insert_text:
                    "effect choose_draw_or_discard_target_card_record_sha(self, event_card, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_target_equipment".into(),
                kind: 15,
                detail: "Gain every target equipment card in one slot".into(),
                insert_text:
                    "effect gain_target_equipment(${1:self}, ${2:event_source}, ${3:weapon});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect discard_all_target_equipment".into(),
                kind: 15,
                detail: "Discard all currently discardable target equipment".into(),
                insert_text:
                    "effect discard_all_target_equipment(${1:self}, ${2:event_player});".into(),
            },
            LspCompletionItem {
                label: "effect increase_current_sha_target_damage".into(),
                kind: 15,
                detail: "Increase current Sha damage for exactly one target".into(),
                insert_text: "effect increase_current_sha_target_damage(${1:self}, ${2:event_card}, ${3:event_player}, ${4:1});".into(),
            },
            LspCompletionItem {
                label: "effect turn_over".into(),
                kind: 15,
                detail: "Toggle a player's face-up/face-down state".into(),
                insert_text: "effect turn_over(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect restore_character_state".into(),
                kind: 15,
                detail: "Restore a player to unlinked and face-up".into(),
                insert_text: "effect restore_character_state(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect discard_all_hand_equip_judge".into(),
                kind: 15,
                detail: "Discard every hand, equipment, and judgment card".into(),
                insert_text: "effect discard_all_hand_equip_judge(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect damage_then_discard_all_judge".into(),
                kind: 15,
                detail: "Self-damage, then discard the current judgment zone".into(),
                insert_text: "effect damage_then_discard_all_judge(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect replace_draw_gain_top_two_add_skills_if_different_colors".into(),
                kind: 15,
                detail: "Replace draw, gain two revealed cards, grant skills when colors differ"
                    .into(),
                insert_text: "effect replace_draw_gain_top_two_add_skills_if_different_colors(${1:self}, ${2:first_skill_def}, ${3:second_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_target_card_alternate_zone_then_target_draw".into(),
                kind: 15,
                detail: "Gain one target card from each of hand/equipment at most once per Play phase"
                    .into(),
                insert_text: "effect choose_gain_target_card_alternate_zone_then_target_draw(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect remove_skill_by_def".into(),
                kind: 15,
                detail: "Remove all matching skill instances from a player".into(),
                insert_text: "effect remove_skill_by_def(self, ${1:skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_add_skill_by_color".into(),
                kind: 15,
                detail: "Optionally discard a hand card and grant a skill by color".into(),
                insert_text: "effect choose_discard_hand_add_skill_by_color(self, ${1:red_skill_def}, ${2:black_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_matching_equip_color_add_skill".into(),
                kind: 15,
                detail: "Discard a matching-color non-basic card and grant a color lock".into(),
                insert_text: "effect choose_discard_matching_equip_color_add_skill(self, ${1:event_player}, ${2:skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect use_virtual_sha_from_card_color_lock_source_if_no_sha".into(),
                kind: 15,
                detail: "Let the recorded source retaliate when no Sha was used".into(),
                insert_text:
                    "effect use_virtual_sha_from_card_color_lock_source_if_no_sha(self);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_current_card_use_enhancements".into(),
                kind: 15,
                detail: "Choose one or two exact-card enhancements for the suspended use".into(),
                insert_text:
                    "effect choose_current_card_use_enhancements(self, ${1:event_card}, ${2:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect make_current_card_unrespondable".into(),
                kind: 15,
                detail: "Make only the suspended physical card use unrespondable".into(),
                insert_text:
                    "effect make_current_card_unrespondable(self, ${1:event_card}, ${2:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_discard_add_skill".into(),
                kind: 15,
                detail: "Choose draw or discard and grant the matching temporary skill".into(),
                insert_text:
                    "effect choose_draw_or_discard_add_skill(self, ${1:draw_skill_def}, ${2:discard_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_three_way_draw_or_discard_add_skill".into(),
                kind: 15,
                detail:
                    "Cancel or choose draw one, draw two with a helper, or discard with a helper"
                        .into(),
                insert_text:
                    "effect choose_three_way_draw_or_discard_add_skill(self, ${1:draw_skill_def}, ${2:discard_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discarded_black_basic_as_bingliang".into(),
                kind: 15,
                detail: "Repeatedly use eligible triggering discard cards as Bingliang".into(),
                insert_text: "effect choose_discarded_black_basic_as_bingliang(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_judge_use_virtual_sha_draw_if_no_damage".into(),
                kind: 15,
                detail: "Discard a judgment card, use virtual Sha, and draw if it deals no damage"
                    .into(),
                insert_text:
                    "effect choose_discard_judge_use_virtual_sha_draw_if_no_damage(self, ${1:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_top_gain_non_basic_or_swap_basic".into(),
                kind: 15,
                detail: "Reveal the draw-pile top card, then gain or swap it by category".into(),
                insert_text: "effect reveal_top_gain_non_basic_or_swap_basic(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect discard_one_hand_or_equip".into(),
                kind: 15,
                detail: "Force one hand/equipment discard".into(),
                insert_text: "effect discard_one_hand_or_equip(self);".into(),
            },
            LspCompletionItem {
                label: "effect disable_skill_by_def_until_turn_end".into(),
                kind: 15,
                detail: "Temporarily disable one owned skill definition".into(),
                insert_text:
                    "effect disable_skill_by_def_until_turn_end(self, ${1:skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_one_then_gain_target_equipment".into(),
                kind: 15,
                detail: "Optionally discard one card, then gain a target equipment slot".into(),
                insert_text: "effect choose_discard_one_then_gain_target_equipment(self, ${1:event_source}, ${2:weapon});".into(),
            },
            LspCompletionItem {
                label: "effect discard_all_hand_and_equip".into(),
                kind: 15,
                detail: "Discard all hand and equipment cards".into(),
                insert_text: "effect discard_all_hand_and_equip(self);".into(),
            },
            LspCompletionItem {
                label: "effect remove_all_skills".into(),
                kind: 15,
                detail: "Remove all current skills from a player".into(),
                insert_text: "effect remove_all_skills(${1:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect judge_gain_equipment_by_color".into(),
                kind: 15,
                detail: "Judge, then gain equipment by result color".into(),
                insert_text: "effect judge_gain_equipment_by_color(self);".into(),
            },
            LspCompletionItem {
                label: "effect judge_recover_on_suit".into(),
                kind: 15,
                detail: "Judge, then recover when the effective suit matches".into(),
                insert_text:
                    "effect judge_recover_on_suit(${1:event_player}, ${2:heart}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect judge_recover_other_on_suit".into(),
                kind: 15,
                detail: "Judge for one player, then recover another on a suit match".into(),
                insert_text: "effect judge_recover_other_on_suit(${1:event_source}, ${2:self}, ${3:spade}, ${4:1});"
                    .into(),
            },
            LspCompletionItem {
                label: "effect force_others_use_sha_on_nearest_or_lose_hp".into(),
                kind: 15,
                detail: "In seat order, require every other player to Sha a nearest player or lose HP".into(),
                insert_text:
                    "effect force_others_use_sha_on_nearest_or_lose_hp(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect judge_damage_other_on_suit".into(),
                kind: 15,
                detail: "Judge, then force damage to another player on a suit match".into(),
                insert_text: "effect judge_damage_other_on_suit(${1:self}, ${2:spade}, ${3:1}, ${4:thunder});"
                    .into(),
            },
            LspCompletionItem {
                label: "effect discard_selected_record_unique_category_target".into(),
                kind: 15,
                detail: "Record a turn- or play-phase-unique logical category and target, then discard the selected card".into(),
                insert_text:
                    "effect discard_selected_record_unique_category_target(self, ${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded"
                        .into(),
                kind: 15,
                detail: "Choose target draw/discard by use count, then conditionally link owner"
                    .into(),
                insert_text:
                    "effect choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(self, ${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded"
                        .into(),
                kind: 15,
                detail:
                    "Choose target draw/discard by use count, then toggle owner link state if wounded"
                        .into(),
                insert_text:
                    "effect choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded(self, ${1:event_source});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_reveal_other_hand_card_remember_category".into(),
                kind: 3,
                detail: "Reveal another player's hand card and remember its category".into(),
                insert_text:
                    "effect choose_reveal_other_hand_card_remember_category(self);".into(),
            },
            LspCompletionItem {
                label: "effect draw_then_give_exact_record_phase_target".into(),
                kind: 3,
                detail: "Draw, give an exact count, and record the phase target".into(),
                insert_text:
                    "effect draw_then_give_exact_record_phase_target(self, ${1:event_player}, ${2:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect clear_phase_gift_target".into(),
                kind: 3,
                detail: "Clear the current skill's delayed Play-phase gift target".into(),
                insert_text: "effect clear_phase_gift_target(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_resolved_sha_or_shan_then_optional_use_sha".into(),
                kind: 3,
                detail: "Give resolved Sha/Shan cards, then optionally use a physical Sha".into(),
                insert_text:
                    "effect choose_gain_resolved_sha_or_shan_then_optional_use_sha(self, event_card);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_all_resolved_sha_shan_then_color_rewards".into(),
                kind: 3,
                detail: "Give all resolved Sha/Shan cards, then apply red/black rewards".into(),
                insert_text:
                    "effect choose_gain_all_resolved_sha_shan_then_color_rewards(self, event_card);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect skip_current_phase".into(),
                kind: 3,
                detail: "Skip the current non-idle phase".into(),
                insert_text: "effect skip_current_phase(self);".into(),
            },
            LspCompletionItem {
                label: "effect finish_current_phase_abort_resolution".into(),
                kind: 3,
                detail: "Abort the current resolution and finish the phase".into(),
                insert_text: "effect finish_current_phase_abort_resolution(self);".into(),
            },
            LspCompletionItem {
                label: "effect gain_top_card_to_skill_pile".into(),
                kind: 3,
                detail: "Move the draw-pile top card into this skill's public pile".into(),
                insert_text: "effect gain_top_card_to_skill_pile(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_skill_pile_card_add_marked_skill".into(),
                kind: 3,
                detail: "Discard one card from this skill's pile and grant a marked helper"
                    .into(),
                insert_text:
                    "effect choose_discard_skill_pile_card_add_marked_skill(self, ${1:event_player}, ${2:helper_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect gain_one_hand_from_selected_targets".into(),
                kind: 3,
                detail: "Gain one hand card from each active target in submitted order".into(),
                insert_text: "effect gain_one_hand_from_selected_targets(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_repolu_targets_draw".into(),
                kind: 3,
                detail: "Choose living Repolu recipients and draw by activation count".into(),
                insert_text: "effect choose_repolu_targets_draw(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_xinfu_guolun".into(),
                kind: 3,
                detail: "Reveal and optionally exchange Guolun cards, then draw by rank".into(),
                insert_text: "effect resolve_xinfu_guolun(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_targets_by_skipped_phase_count_damage".into(),
                kind: 3,
                detail: "Choose other targets up to the skipped-phase count and damage each"
                    .into(),
                insert_text:
                    "effect choose_other_targets_by_skipped_phase_count_damage(self, ${1:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_uniform_hand_draw_self_choose_others".into(),
                kind: 3,
                detail: "Reveal a uniform-color hand, then draw for self and chosen others"
                    .into(),
                insert_text: "effect reveal_uniform_hand_draw_self_choose_others(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_card_user_or_add_target".into(),
                kind: 3,
                detail: "Choose a card-user transfer or add the chooser as a target".into(),
                insert_text: "effect choose_card_user_or_add_target(self, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_equipment_reduce_current_damage".into(),
                kind: 3,
                detail: "Discard an equipment card to reduce current damage by one".into(),
                insert_text:
                    "effect choose_discard_equipment_reduce_current_damage(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_equipment_prevent_current_damage".into(),
                kind: 3,
                detail: "Discard an equipment card to prevent the current damage".into(),
                insert_text:
                    "effect choose_discard_equipment_prevent_current_damage(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_move_target_equipment_or_draw".into(),
                kind: 3,
                detail: "Move one compatible target equipment, draw one, or cancel".into(),
                insert_text:
                    "effect choose_move_target_equipment_or_draw(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_move_target_equipment_and_or_draw".into(),
                kind: 3,
                detail: "Move compatible equipment, draw one, do both, or cancel".into(),
                insert_text:
                    "effect choose_move_target_equipment_and_or_draw(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both".into(),
                kind: 3,
                detail: "Target discards attack-range cards or takes +1 damage; a weapon forces both".into(),
                insert_text: "effect choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_recast_add_skill".into(),
                kind: 3,
                detail: "Cancel, draw and gain one helper, or recast and gain another".into(),
                insert_text:
                    "effect choose_draw_or_recast_add_skill(self, ${1:draw_skill_def}, ${2:recast_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_sha_for_opening_extra_play".into(),
                kind: 3,
                detail: "Optionally gain a distance-unlimited Sha in this skill's opening Play phase"
                    .into(),
                insert_text: "effect choose_gain_sha_for_opening_extra_play(self);".into(),
            },
            LspCompletionItem {
                label: "effect finish_opening_extra_play_sha_gain".into(),
                kind: 3,
                detail: "Finish the opening-phase Sha gain and apply its no-damage penalty".into(),
                insert_text: "effect finish_opening_extra_play_sha_gain(self);".into(),
            },
            LspCompletionItem {
                label: "effect reveal_target_hand_gain_shan_armor_or_draw_two".into(),
                kind: 3,
                detail: "Reveal a target hand; gain Shan and armor, or draw two".into(),
                insert_text:
                    "effect reveal_target_hand_gain_shan_armor_or_draw_two(self, event_source);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect give_all_equipment_choose_recover_or_damage".into(),
                kind: 3,
                detail: "Give all equipment, then choose count-bound recovery or damage".into(),
                insert_text:
                    "effect give_all_equipment_choose_recover_or_damage(self, event_source);"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_discard_weapon_range_hand_or_increase_damage".into(),
                kind: 3,
                detail: "Target discards weapon-range hand cards or bound Sha damage +1"
                    .into(),
                insert_text:
                    "effect choose_target_discard_weapon_range_hand_or_increase_damage(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_discard_distance_hand_or_increase_damage".into(),
                kind: 3,
                detail: "Target discards max(1, 4-distance) hand cards or bound damage +1"
                    .into(),
                insert_text:
                    "effect choose_target_discard_distance_hand_or_increase_damage(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_view_two_gain_or_move_equipment".into(),
                kind: 3,
                detail: "Draw, privately view two and gain one, or move equipment".into(),
                insert_text:
                    "effect choose_draw_view_two_gain_or_move_equipment(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_hand_count_different_target_draw_or_random_discard".into(),
                kind: 3,
                detail: "Choose a different-hand-count target to draw or randomly discard"
                    .into(),
                insert_text:
                    "effect choose_hand_count_different_target_draw_or_random_discard(self);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect increase_current_draw_count".into(),
                kind: 3,
                detail: "Increase and atomically consume the pending draw-phase count".into(),
                insert_text: "effect increase_current_draw_count(self, ${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect skip_next_play_phase".into(),
                kind: 3,
                detail: "Skip the current player's upcoming Play phase".into(),
                insert_text: "effect skip_next_play_phase(self);".into(),
            },
            LspCompletionItem {
                label: "effect skip_next_judge_phase".into(),
                kind: 3,
                detail: "Skip the player's next judgment phase".into(),
                insert_text: "effect skip_next_judge_phase(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_damage_endpoint_card_increase_if_suit".into(),
                kind: 3,
                detail: "Discard from either damage endpoint and conditionally increase damage"
                    .into(),
                insert_text: "effect choose_discard_damage_endpoint_card_increase_if_suit(self, event_player, ${1:diamond}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_or_recover".into(),
                kind: 3,
                detail: "Choose another player, then that target draws or recovers".into(),
                insert_text:
                    "effect choose_other_draw_or_recover(self, ${1:2}, ${2:1}, ${3:event_amount});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_recover".into(),
                kind: 3,
                detail: "Force choosing another player, then recover that target".into(),
                insert_text: "effect choose_other_recover(self, ${1:2});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_equip_then_remove_self_skill".into(),
                kind: 3,
                detail: "Forced dynamic discard followed by one-shot skill cleanup".into(),
                insert_text:
                    "effect choose_discard_hand_equip_then_remove_self_skill(self, ${1:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_equip_then_recover".into(),
                kind: 3,
                detail: "Discard an exact hand/equipment count, then recover".into(),
                insert_text:
                    "effect choose_discard_hand_equip_then_recover(self, ${1:2}, ${2:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect draw_selected_targets".into(),
                kind: 3,
                detail: "Draw cards for every active-skill target in seat order".into(),
                insert_text: "effect draw_selected_targets(${1:3});".into(),
            },
            LspCompletionItem {
                label: "effect draw_linked_players".into(),
                kind: 3,
                detail: "Draw cards for every linked living player in seat order".into(),
                insert_text: "effect draw_linked_players(${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect reveal_top_gain_if_color".into(),
                kind: 3,
                detail: "Reveal the draw-pile top card and gain it if its printed color matches"
                    .into(),
                insert_text: "effect reveal_top_gain_if_color(self, ${1:red});".into(),
            },
            LspCompletionItem {
                label: "effect recover_selected_targets".into(),
                kind: 3,
                detail: "Recover HP for every active-skill target in seat order".into(),
                insert_text: "effect recover_selected_targets(${1:1});".into(),
            },
            LspCompletionItem {
                label: "effect damage_then_draw_selected_targets".into(),
                kind: 3,
                detail: "Damage then draw for every active-skill target in seat order".into(),
                insert_text:
                    "effect damage_then_draw_selected_targets(self, ${1:1}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_youdi".into(),
                kind: 3,
                detail: "Resolve Youdi's target-owned discard and conditional counter-gain".into(),
                insert_text: "effect resolve_youdi(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_kuangfu".into(),
                kind: 3,
                detail: "Resolve Kuangfu's equipment discard or move choice".into(),
                insert_text: "effect resolve_kuangfu(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_shanjia".into(),
                kind: 3,
                detail: "Resolve Shanjia's draw, discard, and optional virtual Sha".into(),
                insert_text: "effect resolve_shanjia(self, ${1:count});".into(),
            },
            LspCompletionItem {
                label: "effect resolve_new_yajiao".into(),
                kind: 3,
                detail: "Resolve New Yajiao's reveal, recipient, and mismatch discard".into(),
                insert_text: "effect resolve_new_yajiao(self, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_dingpin".into(),
                kind: 3,
                detail: "Resolve Dingpin's judgment and color-dependent result".into(),
                insert_text: "effect resolve_dingpin(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect recover_if_min_current_hp".into(),
                kind: 3,
                detail: "Recover only while tied for the lowest current HP".into(),
                insert_text: "effect recover_if_min_current_hp(self, ${1:1});".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_discard_excess_then_self_discard_or_target_draw".into(),
                kind: 3,
                detail: "Choose a target, discard excess cards, then self-discard or target draw"
                    .into(),
                insert_text:
                    "effect choose_target_discard_excess_then_self_discard_or_target_draw(self);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect resolve_decadezhenjun".into(),
                kind: 3,
                detail:
                    "Resolve Decade Zhenjun in Prepare or End without changing legacy Rezhenjun"
                        .into(),
                insert_text: "effect resolve_decadezhenjun(self);".into(),
            },
            LspCompletionItem {
                label: "effect resolve_sp_xiaoguo".into(),
                kind: 3,
                detail: "Resolve SP Xiaoguo without changing Standard Xiaoguo".into(),
                insert_text: "effect resolve_sp_xiaoguo(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_card_from_each_other_replace_draw".into(),
                kind: 3,
                detail: "Replace the normal draw by gaining one card from each other player"
                    .into(),
                insert_text: "effect choose_gain_card_from_each_other_replace_draw(self);".into(),
            },
            LspCompletionItem {
                label:
                    "effect draw_then_choose_gain_one_from_damage_target_and_neighbors".into(),
                kind: 3,
                detail: "Draw, then gain from the damaged player and both living neighbors"
                    .into(),
                insert_text:
                    "effect draw_then_choose_gain_one_from_damage_target_and_neighbors(self, event_player);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect reveal_discard_suit_gain_or_use".into(),
                kind: 15,
                detail: "Reveal cards, then discard-by-suit gain and/or use one".into(),
                insert_text:
                    "effect reveal_discard_suit_gain_or_use(self, ${1:mark_def}, ${2:3}, ${3:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_or_fire_damage_skip_phases".into(),
                kind: 15,
                detail: "Choose discard and/or fire damage, with conditional phase skips".into(),
                insert_text:
                    "effect choose_discard_or_fire_damage_skip_phases(self, ${1:2}, ${2:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect remove_cards_by_kind".into(),
                kind: 15,
                detail: "Move every card of one kind out of the game".into(),
                insert_text: "effect remove_cards_by_kind(${1:horse_off});".into(),
            },
    ]
}
