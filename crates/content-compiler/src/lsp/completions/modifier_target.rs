use super::*;

pub(in super::super) fn items() -> Vec<LspCompletionItem> {
    vec![
        LspCompletionItem {
            label: "distance_from".into(),
            kind: 14,
            detail: "Distance from owner to others".into(),
            insert_text: "distance_from".into(),
        },
        LspCompletionItem {
            label: "distance_from_if_hp_gte_target".into(),
            kind: 14,
            detail: "Set final distance to one when owner HP is at least target HP".into(),
            insert_text: "distance_from_if_hp_gte_target".into(),
        },
        LspCompletionItem {
            label: "global_distance_to".into(),
            kind: 14,
            detail: "Distance to queried targets while the owner is alive".into(),
            insert_text: "global_distance_to".into(),
        },
        LspCompletionItem {
            label: "attack_range".into(),
            kind: 14,
            detail: "Owner's attack range".into(),
            insert_text: "attack_range".into(),
        },
        LspCompletionItem {
            label: "global_attack_range".into(),
            kind: 14,
            detail: "Every living player's attack range while the owner is alive".into(),
            insert_text: "global_attack_range".into(),
        },
        LspCompletionItem {
            label: "global_hand_limit".into(),
            kind: 14,
            detail: "Every queried player's hand limit while the owner is alive".into(),
            insert_text: "global_hand_limit".into(),
        },
        LspCompletionItem {
            label: "global_damage_taken_amount(nature)".into(),
            kind: 15,
            detail:
                "Adjust every queried player's ordinary incoming damage while the owner is alive"
                    .into(),
            insert_text: "global_damage_taken_amount(${1:any})".into(),
        },
        LspCompletionItem {
            label: "global_damage_taken_final_amount(nature)".into(),
            kind: 15,
            detail: "Adjust every queried player's final incoming damage while the owner is alive"
                .into(),
            insert_text: "global_damage_taken_final_amount(${1:any})".into(),
        },
        LspCompletionItem {
            label: "hand_limit".into(),
            kind: 14,
            detail: "Owner's hand size limit".into(),
            insert_text: "hand_limit".into(),
        },
        LspCompletionItem {
            label: "hand_limit_base".into(),
            kind: 14,
            detail: "Owner's base hand size limit".into(),
            insert_text: "hand_limit_base".into(),
        },
        LspCompletionItem {
            label: "hand_limit_base_max_hp".into(),
            kind: 14,
            detail: "Use maximum HP as the owner's hand-limit base".into(),
            insert_text: "hand_limit_base_max_hp".into(),
        },
        LspCompletionItem {
            label: "skip_judge_phase".into(),
            kind: 14,
            detail: "Skip the owner's judgment phase".into(),
            insert_text: "skip_judge_phase".into(),
        },
        LspCompletionItem {
            label: "skip_discard_phase".into(),
            kind: 14,
            detail: "Skip the owner's discard phase".into(),
            insert_text: "skip_discard_phase".into(),
        },
        LspCompletionItem {
            label: "restore_skipped_phase".into(),
            kind: 14,
            detail: "Restore the owner's phase after a skipped/cancelled audit event".into(),
            insert_text: "restore_skipped_phase".into(),
        },
        LspCompletionItem {
            label: "turn_face_down_cancel".into(),
            kind: 14,
            detail: "Prevent the owner from being turned face-down".into(),
            insert_text: "turn_face_down_cancel".into(),
        },
        LspCompletionItem {
            label: "turn_over_cancel".into(),
            kind: 14,
            detail: "Prevent either face-up/face-down transition".into(),
            insert_text: "turn_over_cancel".into(),
        },
        LspCompletionItem {
            label: "discard_phase_includes_equip".into(),
            kind: 14,
            detail: "Count equipment in the owner's discard-to-limit phase".into(),
            insert_text: "discard_phase_includes_equip".into(),
        },
        LspCompletionItem {
            label: "draw_phase_scry_four_split_two".into(),
            kind: 14,
            detail: "Optionally reorder four draw-pile cards as two top and two bottom".into(),
            insert_text: "draw_phase_scry_four_split_two".into(),
        },
        LspCompletionItem {
            label: "initial_hand_count".into(),
            kind: 14,
            detail: "Adjust the owner's initial hand size".into(),
            insert_text: "initial_hand_count".into(),
        },
        LspCompletionItem {
            label: "card_suit(selector)".into(),
            kind: 15,
            detail: "Override one physical suit for the skill owner".into(),
            insert_text: "card_suit(${1:spade})".into(),
        },
        LspCompletionItem {
            label: "card_name(selector)".into(),
            kind: 15,
            detail: "Override the rule card name for matching owner cards".into(),
            insert_text: "card_name(${1:red})".into(),
        },
        LspCompletionItem {
            label: "card_nature(selector)".into(),
            kind: 15,
            detail: "Override the damage nature of matching owner cards".into(),
            insert_text: "card_nature(${1:sha})".into(),
        },
        LspCompletionItem {
            label: "sha_unrespondable(selector)".into(),
            kind: 15,
            detail: "Prevent targets from answering matching cards resolved as Sha".into(),
            insert_text: "sha_unrespondable(${1:heart})".into(),
        },
        LspCompletionItem {
            label: "card_use_unrespondable(selector)".into(),
            kind: 15,
            detail: "Prevent other players from answering matching cards used by the owner".into(),
            insert_text: "card_use_unrespondable(${1:ordinary_trick})".into(),
        },
        LspCompletionItem {
            label: "sha_target_unrespondable".into(),
            kind: 14,
            detail: "Prevent Sha from being answered when its target owns this skill".into(),
            insert_text: "sha_target_unrespondable".into(),
        },
        LspCompletionItem {
            label: "damage_taken_source_range_threshold".into(),
            kind: 14,
            detail: "Adjust incoming damage around the source attack-range threshold".into(),
            insert_text: "damage_taken_source_range_threshold".into(),
        },
        LspCompletionItem {
            label: "sha_damage_source_range_threshold".into(),
            kind: 14,
            detail: "Adjust the owner's logical Sha damage around an attack-range threshold".into(),
            insert_text: "sha_damage_source_range_threshold".into(),
        },
        LspCompletionItem {
            label: "card_distance_unlimited".into(),
            kind: 14,
            detail: "Ignore target-distance limits for the owner's cards".into(),
            insert_text: "card_distance_unlimited".into(),
        },
        LspCompletionItem {
            label: "card_discard_protected(selector)".into(),
            kind: 15,
            detail: "Protect matching equipment from other players' discard effects".into(),
            insert_text: "card_discard_protected(${1:equip_non_mount})".into(),
        },
        LspCompletionItem {
            label: "card_gain_protected(selector)".into(),
            kind: 15,
            detail: "Protect matching cards from other players' gain effects".into(),
            insert_text: "card_gain_protected(${1:any})".into(),
        },
        LspCompletionItem {
            label: "card_distance_unlimited_if_target_hand_gte_owner(selector)".into(),
            kind: 15,
            detail: "Ignore distance when the target has at least as many hand cards as the owner"
                .into(),
            insert_text: "card_distance_unlimited_if_target_hand_gte_owner(${1:bingliang})".into(),
        },
        LspCompletionItem {
            label: "card_distance_limit(selector)".into(),
            kind: 15,
            detail: "Override the target-distance limit for matching cards".into(),
            insert_text: "card_distance_limit(${1:bingliang})".into(),
        },
        LspCompletionItem {
            label: "virtual_bagua_if_armor_slot_empty".into(),
            kind: 14,
            detail: "Provide a virtual Bagua response while the armor slot is enabled and empty"
                .into(),
            insert_text: "virtual_bagua_if_armor_slot_empty".into(),
        },
        LspCompletionItem {
            label: "sha_response_count_if_other_female".into(),
            kind: 14,
            detail: "Increase Sha Shan count when the other participant has female sex".into(),
            insert_text: "sha_response_count_if_other_female".into(),
        },
        LspCompletionItem {
            label: "sha_response_count_if_source_owner_target_distance_le_one".into(),
            kind: 14,
            detail: "Increase Shan count for the owner's Sha targets within distance 1".into(),
            insert_text: "sha_response_count_if_source_owner_target_distance_le_one".into(),
        },
        LspCompletionItem {
            label: "duel_opponent_response_count".into(),
            kind: 14,
            detail: "Increase the Sha count required from the owner's Duel opponent".into(),
            insert_text: "duel_opponent_response_count".into(),
        },
        LspCompletionItem {
            label: "tao_use_save_cancel_for_non_dying_others_during_owner_turn".into(),
            kind: 14,
            detail: "During the owner's turn, block Tao for other non-dying players".into(),
            insert_text: "tao_use_save_cancel_for_non_dying_others_during_owner_turn".into(),
        },
        LspCompletionItem {
            label: "hand_card_use_response_cancel".into(),
            kind: 14,
            detail: "Prevent the owner from using or responding with hand cards".into(),
            insert_text: "hand_card_use_response_cancel".into(),
        },
        LspCompletionItem {
            label: "hand_card_gained_this_turn_only".into(),
            kind: 12,
            detail: "Only hand cards gained this turn are playable in Play phase".into(),
            insert_text: "hand_card_gained_this_turn_only".into(),
        },
        LspCompletionItem {
            label: "card_targets_all(selector)".into(),
            kind: 15,
            detail: "Require every otherwise-legal matching-card target".into(),
            insert_text: "card_targets_all(${1:sha})".into(),
        },
        LspCompletionItem {
            label: "card_target_cancel(selector)".into(),
            kind: 15,
            detail: "Prevent matching cards from selecting the skill owner".into(),
            insert_text: "card_target_cancel(${1:tao})".into(),
        },
        LspCompletionItem {
            label: "armor_card_target_cancel(selector)".into(),
            kind: 15,
            detail: "Prevent matching cards while the owner's armor effect is active".into(),
            insert_text: "armor_card_target_cancel(${1:ordinary_trick})".into(),
        },
        LspCompletionItem {
            label: "armor_damage_taken_amount(nature)".into(),
            kind: 15,
            detail: "Adjust incoming damage while the owner's armor effect is active".into(),
            insert_text: "armor_damage_taken_amount(${1:fire})".into(),
        },
        LspCompletionItem {
            label: "armor_link_gain_cancel".into(),
            kind: 14,
            detail: "Prevent an unlinked owner from becoming linked while armor is active".into(),
            insert_text: "armor_link_gain_cancel".into(),
        },
        LspCompletionItem {
            label: "link_loss_cancel".into(),
            kind: 14,
            detail: "Prevent a linked owner from becoming unlinked".into(),
            insert_text: "link_loss_cancel".into(),
        },
        LspCompletionItem {
            label: "ignore_armor".into(),
            kind: 14,
            detail: "Make source-side effects ignore armor".into(),
            insert_text: "ignore_armor".into(),
        },
        LspCompletionItem {
            label: "armor_effect_disabled".into(),
            kind: 14,
            detail: "Disable armor effects owned by this player".into(),
            insert_text: "armor_effect_disabled".into(),
        },
        LspCompletionItem {
            label: "card_target_cancel_from_current(any)".into(),
            kind: 15,
            detail: "Prevent the current turn owner from targeting the skill owner".into(),
            insert_text: "card_target_cancel_from_current(any)".into(),
        },
        LspCompletionItem {
            label: "card_target_cancel_if_target_lacks_matching_suit".into(),
            kind: 14,
            detail: "Prevent targeting the owner when their hand lacks the card's suit".into(),
            insert_text: "card_target_cancel_if_target_lacks_matching_suit".into(),
        },
        LspCompletionItem {
            label: "card_target_cancel_if_other_user_hp_gte_target".into(),
            kind: 14,
            detail: "Prevent another player with at least the owner's HP from targeting the owner"
                .into(),
            insert_text: "card_target_cancel_if_other_user_hp_gte_target".into(),
        },
        LspCompletionItem {
            label: "card_target_cancel_if_other_user_distance_gt_one(selector)".into(),
            kind: 15,
            detail: "Prevent matching cards from a player more than one final-distance away".into(),
            insert_text: "card_target_cancel_if_other_user_distance_gt_one(${1:ordinary_trick})"
                .into(),
        },
        LspCompletionItem {
            label: "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty(selector)"
                .into(),
            kind: 15,
            detail:
                "Prevent matching cards from a hand-rich other player targeting an armorless owner"
                    .into(),
            insert_text:
                "card_target_cancel_if_other_user_hand_gte_target_and_armor_empty(${1:sha})".into(),
        },
        LspCompletionItem {
            label: "card_targets_self_only".into(),
            kind: 14,
            detail: "Prevent the owner's cards from targeting other players".into(),
            insert_text: "card_targets_self_only".into(),
        },
        LspCompletionItem {
            label: "card_use_cancel(selector?)".into(),
            kind: 14,
            detail: "Prevent the owner from proactively using all or matching cards".into(),
            insert_text: "card_use_cancel${1:(trick)}".into(),
        },
        LspCompletionItem {
            label: "card_action_cancel_for_skill_color_mask".into(),
            kind: 14,
            detail: "Prevent use, response, and saves matching the skill-state color mask".into(),
            insert_text: "card_action_cancel_for_skill_color_mask".into(),
        },
        LspCompletionItem {
            label: "ignore_armor_for_skill_card_policy".into(),
            kind: 14,
            detail: "Ignore armor for exact cards recorded in this skill's policy state".into(),
            insert_text: "ignore_armor_for_skill_card_policy".into(),
        },
        LspCompletionItem {
            label: "card_use_unrespondable_for_skill_card_policy".into(),
            kind: 14,
            detail: "Prevent responses to exact cards recorded in this skill's policy state".into(),
            insert_text: "card_use_unrespondable_for_skill_card_policy".into(),
        },
        LspCompletionItem {
            label: "card_distance_unlimited_for_skill_card_policy".into(),
            kind: 14,
            detail: "Ignore distance for exact cards recorded in this skill's policy state".into(),
            insert_text: "card_distance_unlimited_for_skill_card_policy".into(),
        },
        LspCompletionItem {
            label: "card_use_unrespondable_within_responder_to_owner_distance".into(),
            kind: 14,
            detail: "Prevent responses when responder-to-owner distance is within the threshold"
                .into(),
            insert_text: "card_use_unrespondable_within_responder_to_owner_distance".into(),
        },
        LspCompletionItem {
            label: "card_use_unrespondable_within_owner_to_responder_distance".into(),
            kind: 14,
            detail: "Prevent responses when owner-to-responder distance is within the threshold"
                .into(),
            insert_text: "card_use_unrespondable_within_owner_to_responder_distance".into(),
        },
        LspCompletionItem {
            label: "card_response_cancel".into(),
            kind: 14,
            detail: "Prevent the owner from responding with any card".into(),
            insert_text: "card_response_cancel".into(),
        },
        LspCompletionItem {
            label: "card_save_cancel(tao?)".into(),
            kind: 14,
            detail: "Prevent all saves, or specifically prevent 桃 saves".into(),
            insert_text: "card_save_cancel${1:(tao)}".into(),
        },
        LspCompletionItem {
            label: "sha_response_cancel".into(),
            kind: 14,
            detail: "Prevent the owner from responding with 杀".into(),
            insert_text: "sha_response_cancel".into(),
        },
        LspCompletionItem {
            label: "sha_ignored_hand_limit".into(),
            kind: 14,
            detail: "Compatibility alias: exclude 杀 from hand-limit count and phase discard"
                .into(),
            insert_text: "sha_ignored_hand_limit".into(),
        },
        LspCompletionItem {
            label: "card_ignored_hand_limit(selector)".into(),
            kind: 15,
            detail: "Exclude cards selected by name/category/suit/color from phase discard".into(),
            insert_text: "card_ignored_hand_limit(${1:sha})".into(),
        },
        LspCompletionItem {
            label: "sha_use_cancel".into(),
            kind: 14,
            detail: "Prevent the owner from proactively using 杀".into(),
            insert_text: "sha_use_cancel".into(),
        },
        LspCompletionItem {
            label: "card_use_unlimited(selector?)".into(),
            kind: 14,
            detail: "Remove ordinary use limits for all cards or a selected card family".into(),
            insert_text: "card_use_unlimited".into(),
        },
        LspCompletionItem {
            label: "sha_distance_unlimited_if_rank_lt_skill_mark".into(),
            kind: 14,
            detail: "Lower-ranked Sha ignores distance while this skill has a rank mark".into(),
            insert_text: "sha_distance_unlimited_if_rank_lt_skill_mark".into(),
        },
        LspCompletionItem {
            label: "sha_distance_unlimited_if_target_distance_lte_rank".into(),
            kind: 14,
            detail: "Sha ignores distance to targets within its physical rank".into(),
            insert_text: "sha_distance_unlimited_if_target_distance_lte_rank".into(),
        },
        LspCompletionItem {
            label: "sha_no_use_count_if_rank_gt_skill_mark".into(),
            kind: 14,
            detail: "Higher-ranked Sha bypasses and does not consume its ordinary use count".into(),
            insert_text: "sha_no_use_count_if_rank_gt_skill_mark".into(),
        },
        LspCompletionItem {
            label: "card_target_limit(selector)".into(),
            kind: 15,
            detail: "Adjust the maximum target count for matching independently-targeted cards"
                .into(),
            insert_text: "card_target_limit(${1:ordinary_trick})".into(),
        },
        LspCompletionItem {
            label: "observer_foreign_hand_faces_visible".into(),
            kind: 14,
            detail: "Reveal other players' hand-card faces to the owning observer".into(),
            insert_text: "observer_foreign_hand_faces_visible".into(),
        },
        LspCompletionItem {
            label: "observer_hidden_identities_visible".into(),
            kind: 14,
            detail: "Reveal normally hidden identities to the owning observer".into(),
            insert_text: "observer_hidden_identities_visible".into(),
        },
    ]
}
