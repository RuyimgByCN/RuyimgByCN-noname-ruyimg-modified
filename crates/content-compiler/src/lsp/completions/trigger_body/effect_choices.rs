//! 触发体补全：选择类效果（effect choose_*）。

use super::*;

pub(super) fn items() -> Vec<LspCompletionItem> {
    vec![
            LspCompletionItem {
                label: "effect choose_discard_one_from_each_linked".into(),
                kind: 15,
                detail: "Discard one hand/equipment card from each linked player in seat order"
                    .into(),
                insert_text:
                    "effect choose_discard_one_from_each_linked(${1:self}, ${2:self_skill});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_redirect_current_sha".into(),
                kind: 15,
                detail: "Discard one card, then redirect the saved Sha target slot".into(),
                insert_text:
                    "effect choose_discard_redirect_current_sha(${1:self}, ${2:event_source}, ${3:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_prevent_damage_or_lose_max_hp_increase_damage".into(),
                kind: 15,
                detail: "Forced choice: prevent bound damage, or lose 1 max HP and increase it"
                    .into(),
                insert_text:
                    "effect choose_prevent_damage_or_lose_max_hp_increase_damage(self, 2);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_increase_current_card_damage_or_both_mark".into(),
                kind: 15,
                detail: "Choose draw, current-card damage +1, or both plus a mark".into(),
                insert_text:
                    "effect choose_draw_or_increase_current_card_damage_or_both_mark(self, event_card, 2, 1, ${1:mark_def}, 1);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_or_remove_current_card_target".into(),
                kind: 15,
                detail: "Optionally add or remove one target from the current card use".into(),
                insert_text:
                    "effect choose_add_or_remove_current_card_target(self, event_card);".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_top_cards_by_suit".into(),
                kind: 15,
                detail: "Privately view top cards and optionally gain matching-suit cards"
                    .into(),
                insert_text:
                    "effect choose_gain_top_cards_by_suit(${1:self}, ${2:3}, ${3:heart});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_by_hand_count".into(),
                kind: 15,
                detail: "Repeatedly choose another player; empty hands use a different draw count"
                    .into(),
                insert_text:
                    "effect choose_other_draw_by_hand_count(${1:self}, ${2:1}, ${3:2}, ${4:event_amount});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw".into(),
                kind: 15,
                detail: "Choose a living target from a typed scope and make them draw".into(),
                insert_text:
                    "effect choose_target_draw(${1:self}, ${2:1}, ${3:min_hand});".into(),
            },
            LspCompletionItem {
                label: "effect choose_targets_draw".into(),
                kind: 15,
                detail: "Choose distinct living targets from a typed scope and make each draw"
                    .into(),
                insert_text:
                    "effect choose_targets_draw(${1:self}, ${2:2}, ${3:2}, ${4:any});".into(),
            },
            LspCompletionItem {
                label: "effect choose_optional_targets_draw".into(),
                kind: 15,
                detail: "Optionally choose a bounded number of targets and make each draw"
                    .into(),
                insert_text:
                    "effect choose_optional_targets_draw(${1:self}, ${2:1}, ${3:1}, ${4:255}, ${5:wounded});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_targets_damage".into(),
                kind: 15,
                detail: "Choose a bounded number of targets and damage each in seat order".into(),
                insert_text:
                    "effect choose_targets_damage(${1:self}, ${2:1}, ${3:1}, ${4:hand_empty}, ${5:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_to_max_hp_cap".into(),
                kind: 15,
                detail: "Choose any living target and draw them up to capped maximum HP".into(),
                insert_text:
                    "effect choose_target_draw_to_max_hp_cap(${1:self}, ${2:5});".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_then_owner_if_below_max_hp".into(),
                kind: 15,
                detail:
                    "Choose a living target to draw, then draw one for the owner if that hand remains below max HP"
                        .into(),
                insert_text:
                    "effect choose_target_draw_then_owner_if_below_max_hp(${1:self}, ${2:2});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_target_draw_max_hp_cap_then_discard_excess".into(),
                kind: 15,
                detail:
                    "Choose a target to draw by capped max HP, then make that target discard excess hand cards"
                        .into(),
                insert_text:
                    "effect choose_target_draw_max_hp_cap_then_discard_excess(${1:self}, ${2:5});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_hand_targets_gain_one_replace_draw".into(),
                kind: 15,
                detail:
                    "Choose 1–2 other hand-card targets, gain one from each, and replace the draw"
                        .into(),
                insert_text:
                    "effect choose_other_hand_targets_gain_one_replace_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_hand_targets_gain_one_reduce_draw".into(),
                kind: 15,
                detail:
                    "Gain one hand card from selected players and reduce the draw by that count"
                        .into(),
                insert_text:
                    "effect choose_other_hand_targets_gain_one_reduce_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp".into(),
                kind: 15,
                detail:
                    "Choose a target to self-discard, then lose HP if their hand differs from HP"
                        .into(),
                insert_text:
                    "effect choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_one_hand_equip_or_recover".into(),
                kind: 15,
                detail: "Let another player give one hand/equipment card or recover the target"
                    .into(),
                insert_text:
                    "effect choose_give_one_hand_equip_or_recover(${1:self}, ${2:event_source}, ${3:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_hand_range".into(),
                kind: 15,
                detail: "Force a player to choose a range of hand cards to discard".into(),
                insert_text:
                    "effect choose_discard_hand_range(${1:self}, ${2:1}, ${3:2147483647});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_give_two_different_colors_or_recover".into(),
                kind: 15,
                detail:
                    "Let another player give two differently colored hand/equipment cards or recover the target"
                        .into(),
                insert_text:
                    "effect choose_give_two_different_colors_or_recover(${1:self}, ${2:event_source}, ${3:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp".into(),
                kind: 15,
                detail:
                    "Choose a nonempty card zone, discard it, draw by discarded cards plus lost HP, then lose 1 HP"
                        .into(),
                insert_text:
                    "effect choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_draw_owner_lost_hp_turn_over".into(),
                kind: 15,
                detail: "Choose another player, draw by owner lost HP, then turn them over".into(),
                insert_text:
                    "effect choose_other_draw_owner_lost_hp_turn_over(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_one_for_sha_refund_and_draw_if_red".into(),
                kind: 15,
                detail: "Discard one card, refund the current Sha use once, and draw if red"
                    .into(),
                insert_text:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red(${1:self}, ${2:event_source}, ${3:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank"
                        .into(),
                kind: 15,
                detail:
                    "Discard one card, resolve Longyin, and restore a skill on equal effective rank"
                        .into(),
                insert_text:
                    "effect choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank(${1:self}, ${2:event_source}, ${3:event_card}, ${4:skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_current_sha_target".into(),
                kind: 15,
                detail: "Optionally add one legal target to the current Sha".into(),
                insert_text:
                    "effect choose_add_current_sha_target(${1:self}, ${2:event_card});".into(),
            },
            LspCompletionItem {
                label: "effect choose_store_hand_sha_in_skill_pile".into(),
                kind: 15,
                detail: "Store selected hand Sha in this skill's public pile".into(),
                insert_text: "effect choose_store_hand_sha_in_skill_pile(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_spend_skill_pile_for_dying_jiu".into(),
                kind: 15,
                detail: "Spend one skill-pile card so a dying player uses virtual Jiu".into(),
                insert_text:
                    "effect choose_spend_skill_pile_for_dying_jiu(${1:self}, ${2:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_compare_selected_hand_card".into(),
                kind: 15,
                detail: "Compare the selected hand card and grant a win/lose helper skill".into(),
                insert_text:
                    "effect choose_compare_selected_hand_card(${1:self}, ${2:event_source}, ${3:win_skill_def}, ${4:lose_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_hand_compare".into(),
                kind: 15,
                detail: "Choose a hand-comparison target and both cards".into(),
                insert_text:
                    "effect choose_hand_compare(${1:self}, ${2:win_skill_def}, ${3:lose_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_damage".into(),
                kind: 15,
                detail: "Choose another living player and deal normal damage".into(),
                insert_text: "effect choose_other_damage(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_bound_source_attack_range_damage".into(),
                kind: 15,
                detail: "Choose a target in a bound source's attack range and deal damage from that source".into(),
                insert_text:
                    "effect choose_bound_source_attack_range_damage(${1:self}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_discard_card_by_kind".into(),
                kind: 15,
                detail: "Choose and gain one matching card from the discard pile".into(),
                insert_text:
                    "effect choose_gain_discard_card_by_kind(${1:self}, ${2:sha});".into(),
            },
            LspCompletionItem {
                label: "effect choose_gain_own_lost_discard_card_this_turn".into(),
                kind: 15,
                detail: "Choose one own card lost this turn that remains in discard".into(),
                insert_text:
                    "effect choose_gain_own_lost_discard_card_this_turn(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_move_field_card_to".into(),
                kind: 15,
                detail: "Move one compatible field card to a fixed player".into(),
                insert_text:
                    "effect choose_move_field_card_to(${1:self}, ${2:event_source});".into(),
            },
            LspCompletionItem {
                label: "effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan".into(),
                kind: 15,
                detail: "Choose a highest-marked other player, judge, and conditionally kill"
                    .into(),
                insert_text:
                    "effect choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(${1:self}, ${2:mark_def});"
                        .into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over"
                        .into(),
                kind: 15,
                detail: "Choose a zone priority, gain one random card from each other player, then turn over"
                    .into(),
                insert_text:
                    "effect choose_zone_priority_gain_random_card_from_each_other_then_turn_over(${1:self});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_missing_skill".into(),
                kind: 15,
                detail: "Choose and gain one candidate skill not already owned".into(),
                insert_text:
                    "effect choose_add_missing_skill(${1:self}, ${2:first_skill_def}, ${3:second_skill_def}, ${4:third_skill_def});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_other_discard_target_card".into(),
                kind: 15,
                detail: "Choose another player, then discard one of their scoped cards".into(),
                insert_text:
                    "effect choose_other_discard_target_card(${1:self}, ${2:hand_equip});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_add_skill_bound_card_target".into(),
                kind: 15,
                detail: "Optionally append this skill instance's bound target".into(),
                insert_text:
                    "effect choose_add_skill_bound_card_target(${1:self}, ${2:event_card});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_category_then_target_give_matching_or_sha_unrespondable"
                    .into(),
                kind: 15,
                detail: "Choose a card category; Sha target gives one or cannot respond".into(),
                insert_text: "effect choose_category_then_target_give_matching_or_sha_unrespondable(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_category_then_target_give_matching_or_damage_plus_one"
                    .into(),
                kind: 15,
                detail: "Choose a category; damage target gives one or damage increases".into(),
                insert_text: "effect choose_category_then_target_give_matching_or_damage_plus_one(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip"
                    .into(),
                kind: 15,
                detail: "Choose a non-source role, randomly gain hand/equipment, use equipment"
                    .into(),
                insert_text: "effect choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(self, event_source);".into(),
            },
            LspCompletionItem {
                label: "effect choose_lose_hp_use_virtual_sha_by_basic_variants".into(),
                kind: 15,
                detail: "Choose an ignore-distance virtual Sha after losing HP; distinct basic variants add damage".into(),
                insert_text:
                    "effect choose_lose_hp_use_virtual_sha_by_basic_variants(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_basic_then_target_discard_equip_or_damage".into(),
                kind: 15,
                detail: "Pay a basic hand card, then make the target discard equipment or take damage".into(),
                insert_text: "effect choose_discard_basic_then_target_discard_equip_or_damage(self, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_current_sha_target_discard_redirect_to_owner".into(),
                kind: 15,
                detail: "Let the current target discard, then redirect the Sha to the skill owner".into(),
                insert_text: "effect choose_current_sha_target_discard_redirect_to_owner(self, event_source, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_gain_bottom_card".into(),
                kind: 15,
                detail: "Reveal the bottom card, then give it to a chosen living player".into(),
                insert_text: "effect choose_target_gain_bottom_card(self);".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_target_card_resolve_current_sha".into(),
                kind: 15,
                detail: "Discard a target card, then resolve the equipment/non-equipment Sha branch".into(),
                insert_text: "effect choose_discard_target_card_resolve_current_sha(self, event_card, event_player);".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_discard_target_card_resolve_current_sha_nonbasic_bonus".into(),
                kind: 15,
                detail:
                    "Discard a target card, then resolve Rejianchu's basic/non-basic Sha branch"
                        .into(),
                insert_text: "effect choose_discard_target_card_resolve_current_sha_nonbasic_bonus(self, event_card, event_player, ${1:bonus_skill_def});".into(),
            },
            LspCompletionItem {
                label: "effect choose_draw_or_recover".into(),
                kind: 15,
                detail: "Let a specified player draw or recover".into(),
                insert_text:
                    "effect choose_draw_or_recover(${1:event_source}, ${2:1}, ${3:1}, ${4:0});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_recover_target".into(),
                kind: 15,
                detail: "Let one player optionally recover a specified target".into(),
                insert_text:
                    "effect choose_recover_target(${1:self}, ${2:event_source}, ${3:1});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_sha".into(),
                kind: 15,
                detail: "Choose legal targets and use a suitless virtual Sha".into(),
                insert_text:
                    "effect choose_use_virtual_sha(${1:self}, ${2:ignore_distance});".into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_sha_or_draw".into(),
                kind: 15,
                detail: "Use a normal-distance virtual Sha without counting it, or draw one"
                    .into(),
                insert_text: "effect choose_use_virtual_sha_or_draw(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_lose_hp_discard_hand_sha_if_basic".into(),
                kind: 15,
                detail: "Choose a target, lose HP, then conditionally use a virtual Sha".into(),
                insert_text:
                    "effect choose_target_lose_hp_discard_hand_sha_if_basic(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_reclaim_equipment_draw_if_range_reduced".into(),
                kind: 15,
                detail: "Reclaim target equipment and draw if its attack range shrinks".into(),
                insert_text:
                    "effect choose_target_reclaim_equipment_draw_if_range_reduced(${1:self}, ${2:event_source}, self_skill);"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_use_virtual_card".into(),
                kind: 15,
                detail: "Use a source-free Wanjian or choose a Duel target".into(),
                insert_text:
                    "effect choose_use_virtual_card(${1:self}, ${2:wanjian}, self_skill);".into(),
            },
            LspCompletionItem {
                label: "effect choose_target_reveal_hand_damage_if_sha".into(),
                kind: 15,
                detail: "Choose and reveal a nonempty hand, then damage it when it contains Sha"
                    .into(),
                insert_text:
                    "effect choose_target_reveal_hand_damage_if_sha(${1:self}, ${2:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_other_give_one_hand_equip".into(),
                kind: 15,
                detail: "Choose another living player, then give them one hand/equipment card"
                    .into(),
                insert_text: "effect choose_other_give_one_hand_equip(${1:self});".into(),
            },
            LspCompletionItem {
                label: "effect choose_reveal_hand_card_bind_category_target".into(),
                kind: 15,
                detail: "Reveal a Play-phase player's hand card and bind its category".into(),
                insert_text:
                    "effect choose_reveal_hand_card_bind_category_target(${1:self}, ${2:event_player});"
                        .into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_target_card".into(),
                kind: 15,
                detail: "Choose and discard one scoped card from a target".into(),
                insert_text: "effect choose_discard_target_card(${1:self}, ${2:event_player}, ${3:hand_equip});".into(),
            },
            LspCompletionItem {
                label: "effect choose_optional_discard_target_card".into(),
                kind: 15,
                detail: "Optionally choose and discard one scoped target card".into(),
                insert_text: "effect choose_optional_discard_target_card(${1:self}, ${2:event_source}, ${3:hand});".into(),
            },
            LspCompletionItem {
                label: "effect choose_discard_target_cards_recover_same_category".into(),
                kind: 15,
                detail: "Discard target cards and recover when their categories match".into(),
                insert_text: "effect choose_discard_target_cards_recover_same_category(${1:self}, ${2:event_source}, ${3:2}, ${4:1});".into(),
            },
            LspCompletionItem {
                label: "effect choose_scskuiji_discard".into(),
                kind: 15,
                detail: "Discard up to count distinct-suit cards from owner+target hands".into(),
                insert_text: "effect choose_scskuiji_discard(${1:self}, ${2:event_source}, ${3:4});".into(),
            },
            LspCompletionItem {
                label: "effect choose_pskuiji_discard".into(),
                kind: 15,
                detail: "Like scskuiji; under-discarder loses 1 HP on asymmetric counts".into(),
                insert_text: "effect choose_pskuiji_discard(${1:self}, ${2:event_source}, ${3:4});".into(),
            },
            LspCompletionItem {
                label:
                    "effect choose_compare_selected_hand_card_second_failure_seat_damage".into(),
                kind: 15,
                detail: "Compare, branch on a strict win, and resolve a repeated-failure seat path"
                    .into(),
                insert_text: "effect choose_compare_selected_hand_card_second_failure_seat_damage(${1:self}, ${2:event_source}, ${3:1}, ${4:2}, ${5:1}, ${6:1});".into(),
            },
    ]
}
